use std::borrow::Cow;

use crate::logging::{
    DomainLogEvent, LogEventMetadata, LogEventResult, LogEventSource, LogFieldVisitor,
    SealedLogEvent, SeverityText,
};

use super::state::SqlStatus;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PgInfoLogOrigin {
    StepOnce,
}

impl PgInfoLogOrigin {
    fn label(self) -> &'static str {
        match self {
            Self::StepOnce => "pginfo_worker::step_once",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PgInfoMemberIdentity {
    pub(crate) member_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PgInfoSqlTransition {
    pub(crate) previous: SqlStatus,
    pub(crate) next: SqlStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PgInfoLogEvent {
    PollFailed {
        origin: PgInfoLogOrigin,
        member: PgInfoMemberIdentity,
        error: String,
    },
    SqlTransition {
        origin: PgInfoLogOrigin,
        member: PgInfoMemberIdentity,
        transition: PgInfoSqlTransition,
    },
}

impl SealedLogEvent for PgInfoLogEvent {}

impl DomainLogEvent for PgInfoLogEvent {
    fn metadata(&self) -> LogEventMetadata {
        match self {
            Self::PollFailed { origin, .. } => LogEventMetadata {
                severity: SeverityText::Warn,
                message: Cow::Borrowed("pginfo poll failed"),
                event_name: "pginfo.poll_failed",
                event_domain: "pginfo",
                event_result: LogEventResult::Failed,
                source: LogEventSource::app(origin.label()),
            },
            Self::SqlTransition {
                origin,
                transition,
                ..
            } => LogEventMetadata {
                severity: sql_transition_severity(transition),
                message: Cow::Borrowed("pginfo sql status transition"),
                event_name: "pginfo.sql_transition",
                event_domain: "pginfo",
                event_result: sql_transition_result(transition),
                source: LogEventSource::app(origin.label()),
            },
        }
    }

    fn write_fields(&self, visitor: &mut dyn LogFieldVisitor) {
        match self {
            Self::PollFailed { member, error, .. } => {
                visitor.string("member_id", member.member_id.clone());
                visitor.string("error", error.clone());
            }
            Self::SqlTransition {
                member,
                transition,
                ..
            } => {
                visitor.string("member_id", member.member_id.clone());
                visitor.str("sql_status_prev", sql_status_label(&transition.previous));
                visitor.str("sql_status_next", sql_status_label(&transition.next));
            }
        }
    }
}

fn sql_transition_severity(transition: &PgInfoSqlTransition) -> SeverityText {
    match (&transition.previous, &transition.next) {
        (SqlStatus::Healthy, SqlStatus::Unreachable) => SeverityText::Warn,
        (SqlStatus::Unreachable, SqlStatus::Healthy) => SeverityText::Info,
        _ => SeverityText::Debug,
    }
}

fn sql_transition_result(transition: &PgInfoSqlTransition) -> LogEventResult {
    match (&transition.previous, &transition.next) {
        (SqlStatus::Healthy, SqlStatus::Unreachable) => LogEventResult::Failed,
        (SqlStatus::Unreachable, SqlStatus::Healthy) => LogEventResult::Recovered,
        _ => LogEventResult::Ok,
    }
}

fn sql_status_label(status: &SqlStatus) -> &'static str {
    match status {
        SqlStatus::Unknown => "unknown",
        SqlStatus::Healthy => "healthy",
        SqlStatus::Unreachable => "unreachable",
    }
}
