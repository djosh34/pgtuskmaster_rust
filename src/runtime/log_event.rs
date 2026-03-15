use pgtm_log_derive::LoggableEvent;

#[derive(Clone, Debug, PartialEq, Eq, LoggableEvent)]
#[log_event(producer = "app", transport = "internal", parser = "app")]
pub(crate) enum RuntimeLogEvent {
    #[log_event(
        name = "runtime.startup_entered",
        severity = "info",
        result = "ok",
        message = "runtime starting"
    )]
    StartupEntered {
        startup_run_id: String,
        logging_level: crate::config::LogLevel,
    },
}
