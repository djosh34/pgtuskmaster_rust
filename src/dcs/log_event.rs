use std::borrow::Cow;

use crate::logging::{
    DomainLogEvent, LogEventMetadata, LogEventResult, LogEventSource, LogFieldVisitor,
    SealedLogEvent, SeverityText,
};

use super::DcsMode;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DcsLogOrigin {
    ConnectedFailure,
    InitialConnectFailure,
    PublishCurrentView,
}

impl DcsLogOrigin {
    fn label(self) -> &'static str {
        match self {
            Self::ConnectedFailure => "dcs_worker::connected_failure",
            Self::InitialConnectFailure => "dcs_worker::initial_connect_failure",
            Self::PublishCurrentView => "dcs_worker::publish_current_view",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DcsLogIdentity {
    pub(crate) scope: String,
    pub(crate) member_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DcsFailure {
    StoreIo { error: String },
    Decode { error: String },
    AlreadyExists { error: String },
}

impl DcsFailure {
    fn severity(&self) -> SeverityText {
        match self {
            Self::Decode { .. } => SeverityText::Error,
            Self::StoreIo { .. } | Self::AlreadyExists { .. } => SeverityText::Warn,
        }
    }

    fn error(&self) -> &str {
        match self {
            Self::StoreIo { error } | Self::Decode { error } | Self::AlreadyExists { error } => {
                error.as_str()
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DcsLogEvent {
    ConnectedStepFailed {
        origin: DcsLogOrigin,
        identity: DcsLogIdentity,
        failure: DcsFailure,
    },
    InitialConnectFailed {
        origin: DcsLogOrigin,
        identity: DcsLogIdentity,
        failure: DcsFailure,
    },
    CoordinationModeTransition {
        origin: DcsLogOrigin,
        identity: DcsLogIdentity,
        previous: Option<DcsMode>,
        next: DcsMode,
    },
}

impl SealedLogEvent for DcsLogEvent {}

impl DomainLogEvent for DcsLogEvent {
    fn metadata(&self) -> LogEventMetadata {
        match self {
            Self::ConnectedStepFailed {
                origin, failure, ..
            } => LogEventMetadata {
                severity: failure.severity(),
                message: Cow::Borrowed("dcs connected step failed"),
                event_name: "dcs.connected.failed",
                event_domain: "dcs",
                event_result: LogEventResult::Failed,
                source: LogEventSource::app(origin.label()),
            },
            Self::InitialConnectFailed {
                origin, failure, ..
            } => LogEventMetadata {
                severity: failure.severity(),
                message: Cow::Borrowed("dcs initial connect failed"),
                event_name: "dcs.initial_connect.failed",
                event_domain: "dcs",
                event_result: LogEventResult::Failed,
                source: LogEventSource::app(origin.label()),
            },
            Self::CoordinationModeTransition { origin, .. } => LogEventMetadata {
                severity: SeverityText::Info,
                message: Cow::Borrowed("dcs coordination mode transition"),
                event_name: "dcs.coordination_mode.transition",
                event_domain: "dcs",
                event_result: LogEventResult::Ok,
                source: LogEventSource::app(origin.label()),
            },
        }
    }

    fn write_fields(&self, visitor: &mut dyn LogFieldVisitor) {
        match self {
            Self::ConnectedStepFailed {
                identity, failure, ..
            }
            | Self::InitialConnectFailed {
                identity, failure, ..
            } => {
                write_identity(visitor, identity);
                visitor.string("error", failure.error().to_string());
            }
            Self::CoordinationModeTransition {
                identity,
                previous,
                next,
                ..
            } => {
                write_identity(visitor, identity);
                visitor.str(
                    "mode_prev",
                    previous.as_ref().map(dcs_mode_label).unwrap_or("unknown"),
                );
                visitor.str("mode_next", dcs_mode_label(next));
            }
        }
    }
}

fn write_identity(visitor: &mut dyn LogFieldVisitor, identity: &DcsLogIdentity) {
    visitor.string("scope", identity.scope.clone());
    visitor.string("member_id", identity.member_id.clone());
}

fn dcs_mode_label(mode: &DcsMode) -> &'static str {
    match mode {
        DcsMode::NotTrusted => "not_trusted",
        DcsMode::Degraded => "degraded",
        DcsMode::Coordinated => "coordinated",
    }
}
