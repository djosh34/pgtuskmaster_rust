use std::borrow::Cow;

use serde_json::Value;

use super::{LogParser, LogProducer, LogTransport, SeverityText};

pub(crate) trait SealedLogEvent {}

pub(crate) trait DomainLogEvent: SealedLogEvent + Send + 'static {
    fn metadata(&self) -> LogEventMetadata;
    fn write_fields(&self, visitor: &mut dyn LogFieldVisitor);
}

pub(crate) trait LogFieldVisitor {
    fn string(&mut self, key: &'static str, value: String);
    fn str(&mut self, key: &'static str, value: &'static str);
    fn bool(&mut self, key: &'static str, value: bool);
    fn u64(&mut self, key: &'static str, value: u64);
    fn usize(&mut self, key: &'static str, value: usize);
    fn json(&mut self, key: &'static str, value: Value);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LogEventResult {
    Ok,
    Failed,
    Recovered,
    Timeout,
}

impl LogEventResult {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Failed => "failed",
            Self::Recovered => "recovered",
            Self::Timeout => "timeout",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LogEventSource {
    pub(crate) producer: LogProducer,
    pub(crate) transport: LogTransport,
    pub(crate) parser: LogParser,
    pub(crate) origin: String,
}

impl LogEventSource {
    pub(crate) fn new(
        producer: LogProducer,
        transport: LogTransport,
        parser: LogParser,
        origin: impl Into<String>,
    ) -> Self {
        Self {
            producer,
            transport,
            parser,
            origin: origin.into(),
        }
    }

    pub(crate) fn app(origin: impl Into<String>) -> Self {
        Self::new(
            LogProducer::App,
            LogTransport::Internal,
            LogParser::App,
            origin,
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LogEventMetadata {
    pub(crate) severity: SeverityText,
    pub(crate) message: Cow<'static, str>,
    pub(crate) event_name: &'static str,
    pub(crate) event_domain: &'static str,
    pub(crate) event_result: LogEventResult,
    pub(crate) source: LogEventSource,
}
