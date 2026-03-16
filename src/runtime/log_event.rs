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
        #[log(key = "runtime.startup_run_id")]
        startup_run_id: String,
        #[log(key = "logging.level")]
        logging_level: crate::config::LogLevel,
    },
}
