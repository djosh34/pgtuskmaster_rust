use std::borrow::Cow;

use crate::config::LogLevel;
use crate::logging::{
    DomainLogEvent, LogEventMetadata, LogEventResult, LogEventSource, LogFieldVisitor,
    SealedLogEvent, SeverityText,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RuntimeLogOrigin {
    RunNodeFromConfig,
}

impl RuntimeLogOrigin {
    fn label(self) -> &'static str {
        match self {
            Self::RunNodeFromConfig => "runtime::run_node_from_config",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RuntimeNodeIdentity {
    pub(crate) scope: String,
    pub(crate) member_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum RuntimeLogEvent {
    StartupEntered {
        origin: RuntimeLogOrigin,
        identity: RuntimeNodeIdentity,
        startup_run_id: String,
        logging_level: LogLevel,
    },
}

impl SealedLogEvent for RuntimeLogEvent {}

impl DomainLogEvent for RuntimeLogEvent {
    fn metadata(&self) -> LogEventMetadata {
        match self {
            Self::StartupEntered { origin, .. } => LogEventMetadata {
                severity: SeverityText::Info,
                message: Cow::Borrowed("runtime starting"),
                event_name: "runtime.startup.entered",
                event_domain: "runtime",
                event_result: LogEventResult::Ok,
                source: LogEventSource::app(origin.label()),
            },
        }
    }

    fn write_fields(&self, visitor: &mut dyn LogFieldVisitor) {
        match self {
            Self::StartupEntered {
                identity,
                startup_run_id,
                logging_level,
                ..
            } => {
                visitor.string("scope", identity.scope.clone());
                visitor.string("member_id", identity.member_id.clone());
                visitor.string("startup_run_id", startup_run_id.clone());
                visitor.str("logging.level", log_level_label(*logging_level));
            }
        }
    }
}

fn log_level_label(level: LogLevel) -> &'static str {
    match level {
        LogLevel::Trace => "trace",
        LogLevel::Debug => "debug",
        LogLevel::Info => "info",
        LogLevel::Warn => "warn",
        LogLevel::Error => "error",
        LogLevel::Fatal => "fatal",
    }
}
