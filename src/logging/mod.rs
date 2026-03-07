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
use tracing::{dispatcher, Dispatch};
use tracing_subscriber::layer::{Context, Layer};
use tracing_subscriber::prelude::*;
use tracing_subscriber::Registry;

mod event;
mod raw_record;

pub(crate) mod postgres_ingest;
pub(crate) mod tailer;

#[cfg(test)]
pub(crate) use event::decode_app_event;
pub(crate) use event::{AppEvent, AppEventHeader, StructuredFields};
pub(crate) use raw_record::{
    PostgresLineRecordBuilder, RawRecordBuilder, SubprocessLineRecord, SubprocessStream,
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
        dispatcher::with_default(&self.dispatch, || emit_tracing_record_event(record));
        CURRENT_TRACING_RESULT.with(|slot| {
            slot.borrow_mut().take().unwrap_or_else(|| {
                Err(LogError::SinkIo(
                    "tracing backend did not produce an emission result".to_string(),
                ))
            })
        })
    }
}

fn emit_tracing_record_event(record: &LogRecord) {
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
pub(crate) struct LogHandle {
    hostname: String,
    backend: Arc<TracingBackend>,
    min_app_severity_number: u8,
}

impl std::fmt::Debug for LogHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LogHandle")
            .field("hostname", &self.hostname)
            .field("min_app_severity_number", &self.min_app_severity_number)
            .finish()
    }
}

impl LogHandle {
    pub(crate) fn new(
        hostname: String,
        sink: Arc<dyn LogSink>,
        min_app_severity: SeverityText,
    ) -> Self {
        Self {
            hostname,
            backend: Arc::new(TracingBackend::new(sink)),
            min_app_severity_number: min_app_severity.number(),
        }
    }

    #[cfg(test)]
    pub(crate) fn null() -> Self {
        Self {
            hostname: "unknown".to_string(),
            backend: Arc::new(TracingBackend::new(Arc::new(NullSink))),
            min_app_severity_number: SeverityText::Trace.number(),
        }
    }

    pub(crate) fn disabled() -> Self {
        Self {
            hostname: "unknown".to_string(),
            backend: Arc::new(TracingBackend::new(Arc::new(NullSink))),
            min_app_severity_number: SeverityText::Trace.number(),
        }
    }

    #[cfg(test)]
    pub(crate) fn emit(
        &self,
        severity_text: SeverityText,
        message: impl Into<String>,
        source: LogSource,
    ) -> Result<(), LogError> {
        if severity_text.number() < self.min_app_severity_number {
            return Ok(());
        }
        let record = LogRecord::new(
            system_now_unix_millis(),
            self.hostname.clone(),
            severity_text,
            message.into(),
            source,
        );
        self.backend.emit(&record)
    }

    pub(crate) fn emit_app_event(
        &self,
        origin: impl Into<String>,
        event: AppEvent,
    ) -> Result<(), LogError> {
        if event.severity().number() < self.min_app_severity_number {
            return Ok(());
        }
        let record = event.into_record(
            system_now_unix_millis(),
            self.hostname.clone(),
            origin.into(),
        );
        self.backend.emit(&record)
    }

    pub(crate) fn emit_raw_record(&self, record: RawRecordBuilder) -> Result<(), LogError> {
        let final_record = record.into_record(system_now_unix_millis(), self.hostname.clone());
        self.emit_record(&final_record)
    }

    pub(crate) fn emit_record(&self, record: &LogRecord) -> Result<(), LogError> {
        self.backend.emit(record)
    }
}

pub(crate) fn system_now_unix_millis() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_millis() as u64,
        Err(_) => 0,
    }
}

pub(crate) fn detect_hostname() -> String {
    match std::env::var("HOSTNAME") {
        Ok(value) if !value.trim().is_empty() => value,
        _ => "unknown".to_string(),
    }
}

pub(crate) struct LoggingSystem {
    pub(crate) handle: LogHandle,
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

    Ok(LoggingSystem {
        handle: LogHandle::new(hostname, sink, SeverityText::from(cfg.logging.level)),
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

    pub(crate) fn snapshot(&self) -> Result<Vec<LogRecord>, LogError> {
        let locked = self
            .records
            .lock()
            .map_err(|_| LogError::SinkIo("test sink lock poisoned".to_string()))?;
        Ok(locked.clone())
    }

    pub(crate) fn collect_matching(
        &self,
        matcher: impl Fn(&LogRecord) -> bool,
    ) -> Result<Vec<LogRecord>, LogError> {
        Ok(self.snapshot()?.into_iter().filter(matcher).collect())
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

    fn unique_temp_root(label: &str) -> PathBuf {
        let pid = std::process::id();
        static COUNTER: std::sync::atomic::AtomicUsize =
            std::sync::atomic::AtomicUsize::new(0);
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
        crate::test_harness::runtime_config::RuntimeConfigBuilder::new()
            .with_logging(LoggingConfig {
                level: LogLevel::Trace,
                postgres: PostgresLoggingConfig {
                    poll_interval_ms: 50,
                    cleanup: LogCleanupConfig {
                        enabled: false,
                        ..crate::test_harness::runtime_config::sample_postgres_logging_config()
                            .cleanup
                    },
                    ..crate::test_harness::runtime_config::sample_postgres_logging_config()
                },
                ..crate::test_harness::runtime_config::sample_logging_config()
            })
            .with_debug(DebugConfig { enabled: false })
            .build()
    }

    fn test_log_handle(min_app_severity: SeverityText) -> (LogHandle, TestSink) {
        let sink = TestSink::default();
        let sink_dyn: Arc<dyn LogSink> = Arc::new(sink.clone());
        (
            LogHandle::new("host-a".to_string(), sink_dyn, min_app_severity),
            sink,
        )
    }

    #[test]
    fn emit_app_event_encodes_typed_headers_and_fields() -> Result<(), Box<dyn std::error::Error>> {
        let (log, sink) = test_log_handle(SeverityText::Trace);
        let mut event = AppEvent::new(
            SeverityText::Info,
            "runtime starting",
            AppEventHeader::new("runtime.startup.entered", "runtime", "ok"),
        );
        event.fields_mut().insert("scope", "scope-a");
        event.fields_mut().insert("member_count", 3_u64);
        event
            .fields_mut()
            .insert_optional("optional_field", Option::<String>::None);

        log.emit_app_event("runtime::run_node_from_config", event)?;

        let records = sink.take();
        assert_eq!(records.len(), 1);
        let decoded = decode_app_event(&records[0])?;
        assert_eq!(
            decoded.header,
            AppEventHeader::new("runtime.startup.entered", "runtime", "ok")
        );
        assert_eq!(decoded.origin, "runtime::run_node_from_config");
        assert_eq!(decoded.message, "runtime starting");
        assert_eq!(
            decoded.fields.get("scope"),
            Some(&Value::String("scope-a".to_string()))
        );
        assert_eq!(
            decoded.fields.get("member_count"),
            Some(&Value::Number(3_u64.into()))
        );
        assert!(!decoded.fields.contains_key("optional_field"));
        Ok(())
    }

    #[test]
    fn structured_fields_encode_scalars_and_serialized_values(
    ) -> Result<(), Box<dyn std::error::Error>> {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
        #[serde(rename_all = "snake_case")]
        enum DemoState {
            NeedsRetry,
        }

        let mut fields = StructuredFields::new();
        fields.insert("bool_field", true);
        fields.insert("signed_field", -5_i64);
        fields.insert("unsigned_field", 7_u64);
        fields.insert("string_field", "value");
        fields.insert_optional("absent_field", Option::<u64>::None);
        fields.insert_serialized("state", &DemoState::NeedsRetry)?;

        let attributes = fields.into_attributes();
        assert_eq!(attributes.get("bool_field"), Some(&Value::Bool(true)));
        assert_eq!(
            attributes.get("signed_field"),
            Some(&Value::Number((-5_i64).into()))
        );
        assert_eq!(
            attributes.get("unsigned_field"),
            Some(&Value::Number(7_u64.into()))
        );
        assert_eq!(
            attributes.get("string_field"),
            Some(&Value::String("value".to_string()))
        );
        assert_eq!(
            attributes.get("state"),
            Some(&Value::String("needs_retry".to_string()))
        );
        assert!(!attributes.contains_key("absent_field"));
        Ok(())
    }

    #[test]
    fn emit_app_event_respects_min_severity() -> Result<(), Box<dyn std::error::Error>> {
        let (log, sink) = test_log_handle(SeverityText::Warn);
        log.emit_app_event(
            "runtime::run_node_from_config",
            AppEvent::new(
                SeverityText::Info,
                "filtered",
                AppEventHeader::new("runtime.filtered", "runtime", "ok"),
            ),
        )?;
        assert!(sink.take().is_empty());
        Ok(())
    }

    #[test]
    fn subprocess_line_record_builder_encodes_stream_metadata(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let record = SubprocessLineRecord::new(
            LogProducer::PgTool,
            "process_worker",
            "job-1",
            "start_postgres",
            "postgres",
            SubprocessStream::Stderr,
            vec![0xff_u8, 0x00, b'a', 0x80],
        )
        .into_raw_record()?
        .into_record(5, "host-a".to_string());

        assert_eq!(record.source.producer, LogProducer::PgTool);
        assert_eq!(record.source.transport, LogTransport::ChildStderr);
        assert_eq!(record.source.parser, LogParser::Raw);
        assert_eq!(record.source.origin, "process_worker");
        assert_eq!(record.severity_text, SeverityText::Warn);
        assert_eq!(record.message, "non_utf8_bytes_hex=ff006180");
        assert_eq!(
            record.attributes.get("job_id"),
            Some(&Value::String("job-1".to_string()))
        );
        assert_eq!(
            record.attributes.get("stream"),
            Some(&Value::String("stderr".to_string()))
        );
        assert_eq!(
            record.attributes.get("raw_bytes_hex"),
            Some(&Value::String("ff006180".to_string()))
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
    fn tracing_backend_preserves_emit_errors_when_sink_fails() {
        let sink: Arc<dyn LogSink> = Arc::new(FailSink);
        let log = LogHandle::new("host-a".to_string(), sink, SeverityText::Trace);

        let err = log.emit(
            SeverityText::Info,
            "should fail",
            LogSource {
                producer: LogProducer::App,
                transport: LogTransport::Internal,
                parser: LogParser::App,
                origin: "test".to_string(),
            },
        );

        assert!(matches!(err, Err(LogError::SinkIo(_))));
    }

    #[test]
    fn tracing_backend_preserves_partial_fanout_success() -> Result<(), Box<dyn std::error::Error>>
    {
        FANOUT_DIAGNOSTIC_COUNT.store(0, Ordering::SeqCst);

        let ok = TestSink::default();
        let ok_records = ok.clone();
        let sink: Arc<dyn LogSink> = Arc::new(FanoutSink::new(vec![
            ("ok".to_string(), Arc::new(ok) as Arc<dyn LogSink>),
            ("fail".to_string(), Arc::new(FailSink) as Arc<dyn LogSink>),
        ]));
        let log = LogHandle::new("host-a".to_string(), sink, SeverityText::Trace);

        log.emit(
            SeverityText::Info,
            "fanout-through-tracing",
            LogSource {
                producer: LogProducer::App,
                transport: LogTransport::Internal,
                parser: LogParser::App,
                origin: "test".to_string(),
            },
        )?;

        let records = ok_records.take();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].message, "fanout-through-tracing");
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

        let system = bootstrap(&cfg)?;
        system.handle.emit(
            SeverityText::Info,
            "hello",
            LogSource {
                producer: LogProducer::App,
                transport: LogTransport::Internal,
                parser: LogParser::App,
                origin: "test".to_string(),
            },
        )?;
        drop(system);

        let lines = read_lines(&path)?;
        assert_eq!(lines.len(), 1);
        let v: serde_json::Value = serde_json::from_str(lines[0].as_str())?;
        assert_eq!(v["message"], "hello");
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

        let system = bootstrap(&cfg)?;
        system.handle.emit(
            SeverityText::Info,
            "fanout",
            LogSource {
                producer: LogProducer::App,
                transport: LogTransport::Internal,
                parser: LogParser::App,
                origin: "test".to_string(),
            },
        )?;
        drop(system);

        let lines = read_lines(&path)?;
        assert_eq!(lines.len(), 1);
        let v: serde_json::Value = serde_json::from_str(lines[0].as_str())?;
        assert_eq!(v["message"], "fanout");

        remove_dir_all_if_exists(&root)?;
        Ok(())
    }

    #[test]
    fn bootstrap_with_all_sinks_disabled_is_non_fatal() -> Result<(), LogBootstrapError> {
        let mut cfg = sample_runtime_config();
        cfg.logging.sinks.stderr.enabled = false;
        cfg.logging.sinks.file.enabled = false;

        let system = bootstrap(&cfg)?;
        let record = sample_record("dropped");
        let res = system.handle.emit_record(&record);
        assert!(res.is_ok(), "expected null sink to accept record: {res:?}");
        Ok(())
    }
}
