use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::LineWriter;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use serde_json::Value;
use thiserror::Error;
use tokio::sync::mpsc;
use tracing::{dispatcher, Dispatch};
use tracing_subscriber::layer::{Context, Layer};
use tracing_subscriber::prelude::*;
use tracing_subscriber::Registry;

mod event;
mod raw_record;

pub(crate) mod postgres_ingest;
pub(crate) mod tailer;

pub(crate) use event::{
    DomainLogEvent, LogEventMetadata, LogEventResult, LogEventSource, LogFieldVisitor,
    SealedLogEvent,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum SeverityText {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

impl SeverityText {
    pub(crate) fn number(self) -> u8 {
        // OpenTelemetry severity_number mapping.
        match self {
            Self::Trace => 1,
            Self::Debug => 5,
            Self::Info => 9,
            Self::Warn => 13,
            Self::Error => 17,
            Self::Fatal => 21,
        }
    }
}

impl From<crate::config::LogLevel> for SeverityText {
    fn from(value: crate::config::LogLevel) -> Self {
        match value {
            crate::config::LogLevel::Trace => Self::Trace,
            crate::config::LogLevel::Debug => Self::Debug,
            crate::config::LogLevel::Info => Self::Info,
            crate::config::LogLevel::Warn => Self::Warn,
            crate::config::LogLevel::Error => Self::Error,
            crate::config::LogLevel::Fatal => Self::Fatal,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LogProducer {
    App,
    Postgres,
    PgTool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LogTransport {
    Internal,
    FileTail,
    ChildStdout,
    ChildStderr,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LogParser {
    App,
    PostgresJson,
    PostgresPlain,
    Raw,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct LogSource {
    pub(crate) producer: LogProducer,
    pub(crate) transport: LogTransport,
    pub(crate) parser: LogParser,
    pub(crate) origin: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct LogRecord {
    pub(crate) timestamp_ms: u64,
    pub(crate) hostname: String,
    pub(crate) severity_text: SeverityText,
    pub(crate) severity_number: u8,
    pub(crate) message: String,
    pub(crate) source: LogSource,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub(crate) attributes: BTreeMap<String, Value>,
}

impl LogRecord {
    #[cfg(test)]
    pub(crate) fn new(
        timestamp_ms: u64,
        hostname: String,
        severity_text: SeverityText,
        message: String,
        source: LogSource,
    ) -> Self {
        Self {
            timestamp_ms,
            hostname,
            severity_text,
            severity_number: severity_text.number(),
            message,
            source,
            attributes: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Error)]
pub(crate) enum LogError {
    #[error("json serialize failed: {0}")]
    Json(String),
    #[error("sink write failed: {0}")]
    SinkIo(String),
}

#[derive(Debug, Error)]
pub(crate) enum LogBootstrapError {
    #[error("logging misconfigured: {0}")]
    Misconfigured(String),
    #[error("sink init failed: {0}")]
    SinkInit(String),
}

pub(crate) trait LogSink: Send + Sync {
    fn emit(&self, record: &LogRecord) -> Result<(), LogError>;
}

pub(crate) struct JsonlStderrSink {
    stderr: Mutex<std::io::Stderr>,
}

impl JsonlStderrSink {
    pub(crate) fn new() -> Self {
        Self {
            stderr: Mutex::new(std::io::stderr()),
        }
    }
}

impl LogSink for JsonlStderrSink {
    fn emit(&self, record: &LogRecord) -> Result<(), LogError> {
        let line = serde_json::to_string(record).map_err(|err| LogError::Json(err.to_string()))?;
        let mut stderr = self
            .stderr
            .lock()
            .map_err(|_| LogError::SinkIo("stderr lock poisoned".to_string()))?;
        stderr
            .write_all(line.as_bytes())
            .and_then(|()| stderr.write_all(b"\n"))
            .map_err(|err| LogError::SinkIo(err.to_string()))?;
        Ok(())
    }
}

struct NullSink;

impl LogSink for NullSink {
    fn emit(&self, record: &LogRecord) -> Result<(), LogError> {
        let _ = record;
        Ok(())
    }
}

pub(crate) struct JsonlFileSink {
    path: PathBuf,
    writer: Mutex<LineWriter<File>>,
}

impl JsonlFileSink {
    pub(crate) fn new(path: PathBuf, mode: crate::config::FileSinkMode) -> Result<Self, LogError> {
        if path.as_os_str().is_empty() {
            return Err(LogError::SinkIo("file sink path is empty".to_string()));
        }

        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|err| {
                    LogError::SinkIo(format!(
                        "create log directory {} for {} failed: {err}",
                        parent.display(),
                        path.display()
                    ))
                })?;
            }
        }

        let mut options = OpenOptions::new();
        options.create(true).write(true);
        match mode {
            crate::config::FileSinkMode::Append => {
                options.append(true);
            }
            crate::config::FileSinkMode::Truncate => {
                options.truncate(true);
            }
        }

        let file = options.open(&path).map_err(|err| {
            LogError::SinkIo(format!("open log file {} failed: {err}", path.display()))
        })?;

        Ok(Self {
            path,
            writer: Mutex::new(LineWriter::new(file)),
        })
    }
}

impl LogSink for JsonlFileSink {
    fn emit(&self, record: &LogRecord) -> Result<(), LogError> {
        let line = serde_json::to_string(record).map_err(|err| LogError::Json(err.to_string()))?;
        let mut writer = self
            .writer
            .lock()
            .map_err(|_| LogError::SinkIo("file sink lock poisoned".to_string()))?;
        writer
            .write_all(line.as_bytes())
            .and_then(|()| writer.write_all(b"\n"))
            .map_err(|err| {
                LogError::SinkIo(format!(
                    "write to log file {} failed: {err}",
                    self.path.display()
                ))
            })?;
        Ok(())
    }
}

struct FanoutSink {
    sinks: Vec<(String, Arc<dyn LogSink>)>,
}

static FANOUT_DIAGNOSTIC_ACTIVE: AtomicBool = AtomicBool::new(false);

#[cfg(test)]
static FANOUT_DIAGNOSTIC_COUNT: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

struct AtomicResetGuard<'a> {
    value: &'a AtomicBool,
}

impl Drop for AtomicResetGuard<'_> {
    fn drop(&mut self) {
        self.value.store(false, Ordering::SeqCst);
    }
}

impl FanoutSink {
    fn new(sinks: Vec<(String, Arc<dyn LogSink>)>) -> Self {
        Self { sinks }
    }

    fn write_diagnostic(label: &str, err: &LogError) {
        let acquired = FANOUT_DIAGNOSTIC_ACTIVE
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok();
        if !acquired {
            return;
        }

        let _guard = AtomicResetGuard {
            value: &FANOUT_DIAGNOSTIC_ACTIVE,
        };

        #[cfg(test)]
        {
            FANOUT_DIAGNOSTIC_COUNT.fetch_add(1, Ordering::SeqCst);
        }

        let mut stderr = std::io::stderr();
        let _ = stderr.write_all(b"fanout sink failure: ");
        let _ = stderr.write_all(label.as_bytes());
        let _ = stderr.write_all(b": ");
        let _ = stderr.write_all(err.to_string().as_bytes());
        let _ = stderr.write_all(b"\n");
    }
}

impl LogSink for FanoutSink {
    fn emit(&self, record: &LogRecord) -> Result<(), LogError> {
        let mut ok_count: u64 = 0;
        let mut failures: Vec<(String, String)> = Vec::new();

        for (label, sink) in &self.sinks {
            match sink.emit(record) {
                Ok(()) => {
                    ok_count += 1;
                }
                Err(err) => {
                    Self::write_diagnostic(label.as_str(), &err);
                    failures.push((label.clone(), err.to_string()));
                }
            }
        }

        if ok_count > 0 {
            return Ok(());
        }

        let mut message = "all sinks failed".to_string();
        if !failures.is_empty() {
            message.push_str(": ");
            for (idx, (label, err)) in failures.iter().enumerate() {
                if idx > 0 {
                    message.push_str("; ");
                }
                message.push_str(label.as_str());
                message.push_str(" => ");
                message.push_str(err.as_str());
            }
        }
        Err(LogError::SinkIo(message))
    }
}

const TRACING_LOG_TARGET: &str = "pgtuskmaster::logging::record";

thread_local! {
    static CURRENT_TRACING_RECORD: RefCell<Option<LogRecord>> = const { RefCell::new(None) };
    static CURRENT_TRACING_RESULT: RefCell<Option<Result<(), LogError>>> = const { RefCell::new(None) };
}

struct ActiveTracingRecordGuard;

impl ActiveTracingRecordGuard {
    fn new(record: &LogRecord) -> Result<Self, LogError> {
        CURRENT_TRACING_RECORD.with(|slot| {
            let mut slot = slot.borrow_mut();
            if slot.is_some() {
                return Err(LogError::SinkIo(
                    "nested tracing-backed log emission is not supported".to_string(),
                ));
            }
            *slot = Some(record.clone());
            Ok(())
        })?;
        CURRENT_TRACING_RESULT.with(|slot| {
            *slot.borrow_mut() = None;
        });
        Ok(Self)
    }
}

impl Drop for ActiveTracingRecordGuard {
    fn drop(&mut self) {
        CURRENT_TRACING_RECORD.with(|slot| {
            let _ = slot.borrow_mut().take();
        });
        CURRENT_TRACING_RESULT.with(|slot| {
            let _ = slot.borrow_mut().take();
        });
    }
}

struct TracingRecordLayer {
    sink: Arc<dyn LogSink>,
}

impl<S> Layer<S> for TracingRecordLayer
where
    S: tracing::Subscriber,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        if event.metadata().target() != TRACING_LOG_TARGET {
            return;
        }

        let result = CURRENT_TRACING_RECORD.with(|slot| {
            let slot = slot.borrow();
            match slot.as_ref() {
                Some(record) => self.sink.emit(record),
                None => Err(LogError::SinkIo(
                    "tracing backend event emitted without an active record".to_string(),
                )),
            }
        });

        CURRENT_TRACING_RESULT.with(|slot| {
            *slot.borrow_mut() = Some(result);
        });
    }
}

#[derive(Clone)]
struct TracingBackend {
    dispatch: Dispatch,
}

impl TracingBackend {
    fn new(sink: Arc<dyn LogSink>) -> Self {
        let subscriber = Registry::default().with(TracingRecordLayer { sink });
        Self {
            dispatch: Dispatch::new(subscriber),
        }
    }

    fn emit(&self, record: &LogRecord) -> Result<(), LogError> {
        let _guard = ActiveTracingRecordGuard::new(record)?;
        dispatcher::with_default(&self.dispatch, || dispatch_tracing_record_event(record));
        CURRENT_TRACING_RESULT.with(|slot| {
            slot.borrow_mut().take().unwrap_or_else(|| {
                Err(LogError::SinkIo(
                    "tracing backend did not produce an emission result".to_string(),
                ))
            })
        })
    }
}

fn dispatch_tracing_record_event(record: &LogRecord) {
    match record.severity_text {
        SeverityText::Trace => tracing::event!(
            target: TRACING_LOG_TARGET,
            tracing::Level::TRACE,
            origin = record.source.origin.as_str(),
            producer = ?record.source.producer,
            transport = ?record.source.transport,
            parser = ?record.source.parser,
            severity_number = record.severity_number,
            message = record.message.as_str()
        ),
        SeverityText::Debug => tracing::event!(
            target: TRACING_LOG_TARGET,
            tracing::Level::DEBUG,
            origin = record.source.origin.as_str(),
            producer = ?record.source.producer,
            transport = ?record.source.transport,
            parser = ?record.source.parser,
            severity_number = record.severity_number,
            message = record.message.as_str()
        ),
        SeverityText::Info => tracing::event!(
            target: TRACING_LOG_TARGET,
            tracing::Level::INFO,
            origin = record.source.origin.as_str(),
            producer = ?record.source.producer,
            transport = ?record.source.transport,
            parser = ?record.source.parser,
            severity_number = record.severity_number,
            message = record.message.as_str()
        ),
        SeverityText::Warn => tracing::event!(
            target: TRACING_LOG_TARGET,
            tracing::Level::WARN,
            origin = record.source.origin.as_str(),
            producer = ?record.source.producer,
            transport = ?record.source.transport,
            parser = ?record.source.parser,
            severity_number = record.severity_number,
            message = record.message.as_str()
        ),
        SeverityText::Error | SeverityText::Fatal => tracing::event!(
            target: TRACING_LOG_TARGET,
            tracing::Level::ERROR,
            origin = record.source.origin.as_str(),
            producer = ?record.source.producer,
            transport = ?record.source.transport,
            parser = ?record.source.parser,
            severity_number = record.severity_number,
            message = record.message.as_str()
        ),
    }
}

#[derive(Clone)]
enum LogSenderMode {
    Disabled,
    Queue(mpsc::UnboundedSender<raw_record::QueuedRecord>),
}

#[derive(Clone)]
pub(crate) struct LogSender {
    hostname: String,
    mode: LogSenderMode,
    min_app_severity_number: u8,
}

impl std::fmt::Debug for LogSender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LogSender")
            .field("hostname", &self.hostname)
            .field("min_app_severity_number", &self.min_app_severity_number)
            .finish()
    }
}

#[derive(Debug, Error)]
pub(crate) enum LogSendError {
    #[error("log queue is closed")]
    QueueClosed,
}

pub(crate) struct LogWorker {
    receiver: mpsc::UnboundedReceiver<raw_record::QueuedRecord>,
    backend: Arc<TracingBackend>,
}

impl LogWorker {
    pub(crate) async fn run(mut self) {
        while let Some(record) = self.receiver.recv().await {
            let materialized = record.into_record();
            let _ = self.backend.emit(&materialized);
        }
    }
}

impl LogSender {
    pub(crate) fn new(
        hostname: String,
        sender: mpsc::UnboundedSender<raw_record::QueuedRecord>,
        min_app_severity: SeverityText,
    ) -> Self {
        Self {
            hostname,
            mode: LogSenderMode::Queue(sender),
            min_app_severity_number: min_app_severity.number(),
        }
    }

    pub(crate) fn disabled() -> Self {
        Self {
            hostname: "unknown".to_string(),
            mode: LogSenderMode::Disabled,
            min_app_severity_number: SeverityText::Trace.number(),
        }
    }

    pub(crate) fn send<E>(&self, event: E) -> Result<(), LogSendError>
    where
        E: DomainLogEvent,
    {
        if event.metadata().severity.number() < self.min_app_severity_number {
            return Ok(());
        }
        let record = raw_record::QueuedRecord::from_event(
            system_now_unix_millis(),
            self.hostname.clone(),
            event,
        );
        match &self.mode {
            LogSenderMode::Disabled => Ok(()),
            LogSenderMode::Queue(sender) => sender.send(record).map_err(|_| LogSendError::QueueClosed),
        }
    }
}

pub(crate) fn system_now_unix_millis() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_millis() as u64,
        Err(_) => 0,
    }
}

fn detect_hostname() -> String {
    match std::env::var("HOSTNAME") {
        Ok(value) if !value.trim().is_empty() => value,
        _ => "unknown".to_string(),
    }
}

pub(crate) struct LoggingSystem {
    pub(crate) sender: LogSender,
    pub(crate) worker: LogWorker,
}

pub(crate) fn bootstrap(
    cfg: &crate::config::RuntimeConfig,
) -> Result<LoggingSystem, LogBootstrapError> {
    let hostname = detect_hostname();
    let mut sinks: Vec<(String, Arc<dyn LogSink>)> = Vec::new();

    if cfg.logging.sinks.stderr.enabled {
        sinks.push((
            "stderr".to_string(),
            Arc::new(JsonlStderrSink::new()) as Arc<dyn LogSink>,
        ));
    }

    if cfg.logging.sinks.file.enabled {
        let path = cfg.logging.sinks.file.path.clone().ok_or_else(|| {
            LogBootstrapError::Misconfigured(
                "logging.sinks.file.enabled=true but logging.sinks.file.path is not set"
                    .to_string(),
            )
        })?;

        let label = format!("file:{}", path.display());
        let sink = JsonlFileSink::new(path, cfg.logging.sinks.file.mode)
            .map_err(|err| LogBootstrapError::SinkInit(err.to_string()))?;
        sinks.push((label, Arc::new(sink) as Arc<dyn LogSink>));
    }

    let sink: Arc<dyn LogSink> = match sinks.len() {
        0 => Arc::new(NullSink),
        1 => sinks
            .pop()
            .map(|(_label, sink)| sink)
            .ok_or_else(|| LogBootstrapError::SinkInit("unexpected empty sink list".to_string()))?,
        _ => Arc::new(FanoutSink::new(sinks)),
    };

    let backend = Arc::new(TracingBackend::new(sink));
    let (sender, receiver) = mpsc::unbounded_channel();

    Ok(LoggingSystem {
        sender: LogSender::new(hostname, sender, SeverityText::from(cfg.logging.level)),
        worker: LogWorker { receiver, backend },
    })
}

#[cfg(test)]
#[derive(Clone, Default)]
pub(crate) struct TestSink {
    records: Arc<Mutex<Vec<LogRecord>>>,
}

#[cfg(test)]
impl TestSink {
    pub(crate) fn take(&self) -> Vec<LogRecord> {
        let mut locked = match self.records.lock() {
            Ok(locked) => locked,
            Err(poisoned) => poisoned.into_inner(),
        };
        std::mem::take(&mut *locked)
    }
}

#[cfg(test)]
impl LogSink for TestSink {
    fn emit(&self, record: &LogRecord) -> Result<(), LogError> {
        let mut locked = self
            .records
            .lock()
            .map_err(|_| LogError::SinkIo("test sink lock poisoned".to_string()))?;
        locked.push(record.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::config::{
        DebugConfig, LogCleanupConfig, LogLevel, LoggingConfig, PostgresLoggingConfig,
        RuntimeConfig,
    };
    use crate::process::jobs::ProcessJobKind;
    use crate::process::log_event::{
        CapturedStream, ProcessExecutionIdentity, ProcessJobIdentity, ProcessLogOrigin,
        SubprocessLogEvent,
    };
    use crate::runtime::log_event::{RuntimeLogEvent, RuntimeLogOrigin, RuntimeNodeIdentity};

    fn unique_temp_root(label: &str) -> PathBuf {
        let pid = std::process::id();
        static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let unique = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        std::env::temp_dir().join(format!("pgtuskmaster-{label}-{pid}-{unique}"))
    }

    fn remove_dir_all_if_exists(path: &std::path::Path) -> Result<(), std::io::Error> {
        match std::fs::remove_dir_all(path) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(err),
        }
    }

    fn remove_file_if_exists(path: &std::path::Path) -> Result<(), std::io::Error> {
        match std::fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(err),
        }
    }

    fn sample_record(message: &str) -> LogRecord {
        LogRecord::new(
            1,
            "host-a".to_string(),
            SeverityText::Info,
            message.to_string(),
            LogSource {
                producer: LogProducer::App,
                transport: LogTransport::Internal,
                parser: LogParser::App,
                origin: "test".to_string(),
            },
        )
    }

    fn read_lines(path: &std::path::Path) -> Result<Vec<String>, std::io::Error> {
        let bytes = std::fs::read(path)?;
        let text = String::from_utf8_lossy(&bytes);
        Ok(text
            .lines()
            .map(|line| line.to_string())
            .filter(|line| !line.trim().is_empty())
            .collect())
    }

    fn sample_runtime_config() -> RuntimeConfig {
        crate::dev_support::runtime_config::RuntimeConfigBuilder::new()
            .with_logging(LoggingConfig {
                level: LogLevel::Trace,
                postgres: PostgresLoggingConfig {
                    poll_interval_ms: 50,
                    cleanup: LogCleanupConfig {
                        enabled: false,
                        ..crate::dev_support::runtime_config::sample_postgres_logging_config()
                            .cleanup
                    },
                    ..crate::dev_support::runtime_config::sample_postgres_logging_config()
                },
                ..crate::dev_support::runtime_config::sample_logging_config()
            })
            .with_debug(DebugConfig { enabled: false })
            .build()
    }

    fn sample_runtime_event() -> RuntimeLogEvent {
        RuntimeLogEvent::StartupEntered {
            origin: RuntimeLogOrigin::RunNodeFromConfig,
            identity: RuntimeNodeIdentity {
                scope: "scope-a".to_string(),
                member_id: "member-a".to_string(),
            },
            startup_run_id: "run-1".to_string(),
            logging_level: crate::config::LogLevel::Info,
        }
    }

    fn test_log_system(min_app_severity: SeverityText) -> (LogSender, LogWorker, TestSink) {
        let (sender, receiver) = mpsc::unbounded_channel();
        let sink = TestSink::default();
        let sink_dyn: Arc<dyn LogSink> = Arc::new(sink.clone());
        (
            LogSender::new("host-a".to_string(), sender, min_app_severity),
            LogWorker {
                receiver,
                backend: Arc::new(TracingBackend::new(sink_dyn)),
            },
            sink,
        )
    }

    fn run_worker(worker: LogWorker) -> Result<(), Box<dyn std::error::Error>> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        runtime.block_on(worker.run());
        Ok(())
    }

    fn collect_records<E>(
        min_app_severity: SeverityText,
        event: E,
    ) -> Result<Vec<LogRecord>, Box<dyn std::error::Error>>
    where
        E: DomainLogEvent,
    {
        let (log, worker, sink) = test_log_system(min_app_severity);
        log.send(event)?;
        drop(log);
        run_worker(worker)?;
        Ok(sink.take())
    }

    #[test]
    fn typed_runtime_event_encodes_headers_and_fields(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let records = collect_records(SeverityText::Trace, sample_runtime_event())?;
        assert_eq!(records.len(), 1);
        assert_eq!(
            records[0].attributes.get("event.name"),
            Some(&Value::String("runtime.startup.entered".to_string()))
        );
        assert_eq!(
            records[0].attributes.get("event.domain"),
            Some(&Value::String("runtime".to_string()))
        );
        assert_eq!(
            records[0].attributes.get("event.result"),
            Some(&Value::String("ok".to_string()))
        );
        assert_eq!(records[0].source.origin, "runtime::run_node_from_config");
        assert_eq!(records[0].message, "runtime starting");
        assert_eq!(
            records[0].attributes.get("scope"),
            Some(&Value::String("scope-a".to_string()))
        );
        assert_eq!(
            records[0].attributes.get("member_id"),
            Some(&Value::String("member-a".to_string()))
        );
        assert_eq!(
            records[0].attributes.get("startup_run_id"),
            Some(&Value::String("run-1".to_string()))
        );
        assert_eq!(
            records[0].attributes.get("logging.level"),
            Some(&Value::String("info".to_string()))
        );
        Ok(())
    }

    #[test]
    fn typed_event_respects_min_severity() -> Result<(), Box<dyn std::error::Error>> {
        let records = collect_records(SeverityText::Warn, sample_runtime_event())?;
        assert!(records.is_empty());
        Ok(())
    }

    #[test]
    fn subprocess_line_event_encodes_stream_metadata(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let records = collect_records(
            SeverityText::Trace,
            SubprocessLogEvent {
            producer: LogProducer::PgTool,
            origin: ProcessLogOrigin::EmitSubprocessLine,
            execution: ProcessExecutionIdentity {
                job: ProcessJobIdentity {
                    job_id: "job-1".to_string(),
                    kind: ProcessJobKind::StartPostgres,
                },
                binary: "postgres".to_string(),
            },
            stream: CapturedStream::Stderr,
            bytes: vec![0xff_u8, 0x00, b'a', 0x80],
        },
        )?;

        assert_eq!(records.len(), 1);
        let record = &records[0];
        assert_eq!(record.source.producer, LogProducer::PgTool);
        assert_eq!(record.source.transport, LogTransport::ChildStderr);
        assert_eq!(record.source.parser, LogParser::Raw);
        assert_eq!(record.source.origin, "process_worker::emit_subprocess_line");
        assert_eq!(record.severity_text, SeverityText::Warn);
        assert!(record.message.contains('a'));
        assert_eq!(
            record.attributes.get("job.id"),
            Some(&Value::String("job-1".to_string()))
        );
        assert_eq!(
            record.attributes.get("job.kind"),
            Some(&Value::String("start_postgres".to_string()))
        );
        assert_eq!(
            record.attributes.get("stream"),
            Some(&Value::String("stderr".to_string()))
        );
        Ok(())
    }

    #[test]
    fn jsonl_file_sink_creates_parent_dirs_and_writes_jsonl_line(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = unique_temp_root("file-sink-create");
        remove_dir_all_if_exists(&root)?;

        let path = root.join("a").join("b").join("log.jsonl");
        let sink = JsonlFileSink::new(path.clone(), crate::config::FileSinkMode::Append)?;
        sink.emit(&sample_record("hello"))?;
        drop(sink);

        let lines = read_lines(&path)?;
        assert_eq!(lines.len(), 1);
        let v: serde_json::Value = serde_json::from_str(lines[0].as_str())?;
        assert_eq!(v["message"], "hello");

        remove_dir_all_if_exists(&root)?;
        Ok(())
    }

    #[test]
    fn jsonl_file_sink_append_mode_preserves_existing_content(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = unique_temp_root("file-sink-append");
        remove_dir_all_if_exists(&root)?;
        std::fs::create_dir_all(&root)?;

        let path = root.join("log.jsonl");
        std::fs::write(&path, b"{\"pre\":1}\n")?;

        let sink = JsonlFileSink::new(path.clone(), crate::config::FileSinkMode::Append)?;
        sink.emit(&sample_record("post"))?;
        drop(sink);

        let lines = read_lines(&path)?;
        assert_eq!(lines.len(), 2);
        let pre: serde_json::Value = serde_json::from_str(lines[0].as_str())?;
        assert_eq!(pre["pre"], 1);
        let post: serde_json::Value = serde_json::from_str(lines[1].as_str())?;
        assert_eq!(post["message"], "post");

        remove_dir_all_if_exists(&root)?;
        Ok(())
    }

    #[test]
    fn jsonl_file_sink_truncate_mode_replaces_existing_content(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = unique_temp_root("file-sink-truncate");
        remove_dir_all_if_exists(&root)?;
        std::fs::create_dir_all(&root)?;

        let path = root.join("log.jsonl");
        std::fs::write(&path, b"{\"stale\":true}\n{\"stale\":true}\n")?;

        let sink = JsonlFileSink::new(path.clone(), crate::config::FileSinkMode::Truncate)?;
        sink.emit(&sample_record("fresh"))?;
        drop(sink);

        let lines = read_lines(&path)?;
        assert_eq!(lines.len(), 1);
        let v: serde_json::Value = serde_json::from_str(lines[0].as_str())?;
        assert_eq!(v["message"], "fresh");

        remove_dir_all_if_exists(&root)?;
        Ok(())
    }

    #[test]
    fn jsonl_file_sink_errors_when_parent_is_file() -> Result<(), Box<dyn std::error::Error>> {
        let root = unique_temp_root("file-sink-parent-file");
        remove_dir_all_if_exists(&root)?;
        std::fs::create_dir_all(&root)?;

        let not_a_dir = root.join("not_a_dir");
        remove_file_if_exists(&not_a_dir)?;
        let write_res = std::fs::write(&not_a_dir, b"im a file");
        assert!(write_res.is_ok());

        let path = not_a_dir.join("app.jsonl");
        let err = JsonlFileSink::new(path.clone(), crate::config::FileSinkMode::Append);
        assert!(matches!(err, Err(LogError::SinkIo(_))));

        if let Err(LogError::SinkIo(msg)) = err {
            assert!(msg.contains(path.display().to_string().as_str()));
        }

        remove_dir_all_if_exists(&root)?;
        Ok(())
    }

    #[derive(Clone)]
    struct FailSink;

    impl LogSink for FailSink {
        fn emit(&self, _record: &LogRecord) -> Result<(), LogError> {
            Err(LogError::SinkIo("fail sink".to_string()))
        }
    }

    #[test]
    fn fanout_sink_ok_when_any_sink_succeeds_and_emits_diagnostic() {
        FANOUT_DIAGNOSTIC_COUNT.store(0, Ordering::SeqCst);

        let ok = Arc::new(TestSink::default());
        let ok_dyn: Arc<dyn LogSink> = ok;
        let fail_dyn: Arc<dyn LogSink> = Arc::new(FailSink);

        let sink = FanoutSink::new(vec![
            ("ok".to_string(), ok_dyn),
            ("fail".to_string(), fail_dyn),
        ]);

        assert!(sink.emit(&sample_record("x")).is_ok());
        assert!(FANOUT_DIAGNOSTIC_COUNT.load(Ordering::SeqCst) >= 1);
    }

    #[test]
    fn fanout_sink_err_when_all_sinks_fail() {
        let fail_a: Arc<dyn LogSink> = Arc::new(FailSink);
        let fail_b: Arc<dyn LogSink> = Arc::new(FailSink);

        let sink = FanoutSink::new(vec![("a".to_string(), fail_a), ("b".to_string(), fail_b)]);

        let err = sink.emit(&sample_record("x"));
        assert!(matches!(err, Err(LogError::SinkIo(_))));
    }

    #[test]
    fn sender_reports_only_queue_closed_to_callers() {
        let (sender, receiver) = mpsc::unbounded_channel();
        drop(receiver);
        let log = LogSender::new("host-a".to_string(), sender, SeverityText::Trace);

        let err = log.send(sample_runtime_event());
        assert!(matches!(err, Err(LogSendError::QueueClosed)));
    }

    #[test]
    fn worker_keeps_sink_failures_internal_after_enqueue(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (sender, receiver) = mpsc::unbounded_channel();
        let sink: Arc<dyn LogSink> = Arc::new(FailSink);
        let worker = LogWorker {
            receiver,
            backend: Arc::new(TracingBackend::new(sink)),
        };
        let log = LogSender::new("host-a".to_string(), sender, SeverityText::Trace);

        assert!(log.send(sample_runtime_event()).is_ok());
        drop(log);
        run_worker(worker)?;
        Ok(())
    }

    #[test]
    fn worker_preserves_partial_fanout_success() -> Result<(), Box<dyn std::error::Error>> {
        FANOUT_DIAGNOSTIC_COUNT.store(0, Ordering::SeqCst);

        let ok = TestSink::default();
        let ok_records = ok.clone();
        let sink: Arc<dyn LogSink> = Arc::new(FanoutSink::new(vec![
            ("ok".to_string(), Arc::new(ok) as Arc<dyn LogSink>),
            ("fail".to_string(), Arc::new(FailSink) as Arc<dyn LogSink>),
        ]));
        let (sender, receiver) = mpsc::unbounded_channel();
        let worker = LogWorker {
            receiver,
            backend: Arc::new(TracingBackend::new(sink)),
        };
        let log = LogSender::new("host-a".to_string(), sender, SeverityText::Trace);

        log.send(sample_runtime_event())?;
        drop(log);
        run_worker(worker)?;

        let records = ok_records.take();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].message, "runtime starting");
        assert!(FANOUT_DIAGNOSTIC_COUNT.load(Ordering::SeqCst) >= 1);
        Ok(())
    }

    #[test]
    fn bootstrap_file_enabled_without_path_returns_misconfigured() {
        let mut cfg = sample_runtime_config();
        cfg.logging.sinks.stderr.enabled = false;
        cfg.logging.sinks.file.enabled = true;
        cfg.logging.sinks.file.path = None;

        let res = bootstrap(&cfg);
        assert!(matches!(res, Err(LogBootstrapError::Misconfigured(_))));
    }

    #[test]
    fn bootstrap_file_enabled_with_path_writes_jsonl() -> Result<(), Box<dyn std::error::Error>> {
        let root = unique_temp_root("bootstrap-file-enabled");
        remove_dir_all_if_exists(&root)?;
        std::fs::create_dir_all(&root)?;

        let path = root.join("app.jsonl");

        let mut cfg = sample_runtime_config();
        cfg.logging.sinks.stderr.enabled = false;
        cfg.logging.sinks.file.enabled = true;
        cfg.logging.sinks.file.path = Some(path.clone());

        let LoggingSystem { sender, worker } = bootstrap(&cfg)?;
        sender.send(sample_runtime_event())?;
        drop(sender);
        run_worker(worker)?;

        let lines = read_lines(&path)?;
        assert_eq!(lines.len(), 1);
        let v: serde_json::Value = serde_json::from_str(lines[0].as_str())?;
        assert_eq!(v["message"], "runtime starting");
        assert_eq!(v["severity_text"], "info");

        remove_dir_all_if_exists(&root)?;
        Ok(())
    }

    #[test]
    fn bootstrap_with_stderr_and_file_still_writes_file() -> Result<(), Box<dyn std::error::Error>>
    {
        let root = unique_temp_root("bootstrap-stderr-and-file");
        remove_dir_all_if_exists(&root)?;
        std::fs::create_dir_all(&root)?;

        let path = root.join("app.jsonl");

        let mut cfg = sample_runtime_config();
        cfg.logging.sinks.stderr.enabled = true;
        cfg.logging.sinks.file.enabled = true;
        cfg.logging.sinks.file.path = Some(path.clone());

        let LoggingSystem { sender, worker } = bootstrap(&cfg)?;
        sender.send(sample_runtime_event())?;
        drop(sender);
        run_worker(worker)?;

        let lines = read_lines(&path)?;
        assert_eq!(lines.len(), 1);
        let v: serde_json::Value = serde_json::from_str(lines[0].as_str())?;
        assert_eq!(v["message"], "runtime starting");

        remove_dir_all_if_exists(&root)?;
        Ok(())
    }

    #[test]
    fn bootstrap_with_all_sinks_disabled_is_non_fatal() -> Result<(), LogBootstrapError> {
        let mut cfg = sample_runtime_config();
        cfg.logging.sinks.stderr.enabled = false;
        cfg.logging.sinks.file.enabled = false;

        let system = bootstrap(&cfg)?;
        let res = system.sender.send(sample_runtime_event());
        assert!(res.is_ok(), "expected null sink to accept record: {res:?}");
        Ok(())
    }
}
