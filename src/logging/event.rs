use std::borrow::Cow;
use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::Value;

use super::{LogEventResult, LogParser, LogProducer, LogSeverity, LogSource, LogTransport};

pub(crate) mod sealed {
    pub trait Sealed {}
}

pub(crate) trait LoggableEvent: sealed::Sealed + Send + 'static {
    fn into_log_event(self) -> LogEventDto;
}

pub(crate) trait LogValue {
    fn into_log_field_value(self) -> LogFieldValue;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LogComputedMeta {
    pub(crate) severity: LogSeverity,
    pub(crate) result: LogEventResult,
    pub(crate) message: Cow<'static, str>,
    pub(crate) source: LogSource,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LogEventDto {
    pub(crate) severity: LogSeverity,
    pub(crate) event_name: &'static str,
    pub(crate) result: LogEventResult,
    pub(crate) message: Cow<'static, str>,
    pub(crate) source: LogSource,
    pub(crate) fields: Vec<LogField>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LogField {
    pub(crate) key: &'static str,
    pub(crate) value: LogFieldValue,
}

impl LogField {
    pub(crate) fn new(key: &'static str, value: LogFieldValue) -> Self {
        Self { key, value }
    }

    pub(crate) fn new_owned(key: String, value: LogFieldValue) -> Self {
        Self {
            key: Box::leak(key.into_boxed_str()),
            value,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(untagged)]
pub(crate) enum LogFieldValue {
    String(String),
    Bool(bool),
    I64(i64),
    U64(u64),
    Json(Value),
}

impl LogValue for String {
    fn into_log_field_value(self) -> LogFieldValue {
        LogFieldValue::String(self)
    }
}

impl LogValue for &'static str {
    fn into_log_field_value(self) -> LogFieldValue {
        LogFieldValue::String(self.to_string())
    }
}

impl LogValue for bool {
    fn into_log_field_value(self) -> LogFieldValue {
        LogFieldValue::Bool(self)
    }
}

impl LogValue for i64 {
    fn into_log_field_value(self) -> LogFieldValue {
        LogFieldValue::I64(self)
    }
}

impl LogValue for u32 {
    fn into_log_field_value(self) -> LogFieldValue {
        LogFieldValue::U64(u64::from(self))
    }
}

impl LogValue for u64 {
    fn into_log_field_value(self) -> LogFieldValue {
        LogFieldValue::U64(self)
    }
}

impl LogValue for usize {
    fn into_log_field_value(self) -> LogFieldValue {
        LogFieldValue::U64(self as u64)
    }
}

impl LogValue for Value {
    fn into_log_field_value(self) -> LogFieldValue {
        LogFieldValue::Json(self)
    }
}

impl LogValue for BTreeMap<String, Value> {
    fn into_log_field_value(self) -> LogFieldValue {
        LogFieldValue::Json(Value::Object(self.into_iter().collect()))
    }
}

impl LogValue for LogSeverity {
    fn into_log_field_value(self) -> LogFieldValue {
        LogFieldValue::String(
            match self {
                LogSeverity::Trace => "trace",
                LogSeverity::Debug => "debug",
                LogSeverity::Info => "info",
                LogSeverity::Warn => "warn",
                LogSeverity::Error => "error",
                LogSeverity::Fatal => "fatal",
            }
            .to_string(),
        )
    }
}

impl LogValue for LogProducer {
    fn into_log_field_value(self) -> LogFieldValue {
        LogFieldValue::String(
            match self {
                LogProducer::App => "app",
                LogProducer::Postgres => "postgres",
                LogProducer::PgTool => "pg_tool",
            }
            .to_string(),
        )
    }
}

impl LogValue for LogTransport {
    fn into_log_field_value(self) -> LogFieldValue {
        LogFieldValue::String(
            match self {
                LogTransport::Internal => "internal",
                LogTransport::FileTail => "file_tail",
                LogTransport::ChildStdout => "child_stdout",
                LogTransport::ChildStderr => "child_stderr",
            }
            .to_string(),
        )
    }
}

impl LogValue for LogParser {
    fn into_log_field_value(self) -> LogFieldValue {
        LogFieldValue::String(
            match self {
                LogParser::App => "app",
                LogParser::PostgresJson => "postgres_json",
                LogParser::PostgresPlain => "postgres_plain",
                LogParser::Raw => "raw",
            }
            .to_string(),
        )
    }
}

pub(crate) fn push_flattened_json_fields(
    fields: &mut Vec<LogField>,
    prefix: &'static str,
    values: BTreeMap<String, Value>,
) {
    for (key, value) in values {
        fields.push(LogField::new_owned(
            format!("{prefix}.{key}"),
            LogFieldValue::Json(value),
        ));
    }
}
