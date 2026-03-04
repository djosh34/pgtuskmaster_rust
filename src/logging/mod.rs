use std::collections::BTreeMap;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

pub(crate) mod postgres_ingest;
pub(crate) mod tailer;
pub(crate) mod archive_wrapper;

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
    PostgresArchive,
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

#[derive(Clone)]
pub(crate) struct LogHandle {
    hostname: String,
    sink: Arc<dyn LogSink>,
    min_app_severity_number: u8,
}

impl LogHandle {
    pub(crate) fn new(hostname: String, sink: Arc<dyn LogSink>, min_app_severity: SeverityText) -> Self {
        Self {
            hostname,
            sink,
            min_app_severity_number: min_app_severity.number(),
        }
    }

    pub(crate) fn null() -> Self {
        Self {
            hostname: "unknown".to_string(),
            sink: Arc::new(NullSink),
            min_app_severity_number: SeverityText::Trace.number(),
        }
    }

    pub(crate) fn hostname(&self) -> &str {
        self.hostname.as_str()
    }

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
        self.sink.emit(&record)
    }

    pub(crate) fn emit_record(&self, record: &LogRecord) -> Result<(), LogError> {
        self.sink.emit(record)
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

pub(crate) fn bootstrap(_cfg: &crate::config::RuntimeConfig) -> LoggingSystem {
    let hostname = detect_hostname();
    let sink: Arc<dyn LogSink> = Arc::new(JsonlStderrSink::new());
    LoggingSystem {
        handle: LogHandle::new(hostname, sink, SeverityText::from(_cfg.logging.level)),
    }
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
