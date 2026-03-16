use pgtm_log_derive::LoggableEvent;

#[derive(Clone, Debug, PartialEq, Eq, LoggableEvent)]
#[log_event(producer = "app", transport = "internal", parser = "app")]
pub(crate) enum PgInfoLogEvent {
    #[log_event(
        name = "pginfo.poll_failed",
        severity = "warn",
        result = "failed",
        message = "pginfo poll failed"
    )]
    PollFailed { cause: String },

    #[log_event(name = "pginfo.sql_transition", meta = "computed")]
    SqlTransition {
        #[log(key = "pginfo.sql.previous")]
        previous: Option<crate::pginfo::state::SqlStatus>,
        #[log(key = "pginfo.sql.next")]
        next: crate::pginfo::state::SqlStatus,
    },
}
