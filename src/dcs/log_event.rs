use pgtm_log_derive::LoggableEvent;

#[derive(Clone, Debug, PartialEq, Eq, LoggableEvent)]
#[log_event(producer = "app", transport = "internal", parser = "app")]
pub(crate) enum DcsLogEvent {
    #[log_event(
        name = "dcs.connected_step_store_io_failed",
        severity = "warn",
        result = "failed",
        message = "dcs connected step failed"
    )]
    ConnectedStepStoreIoFailed { cause: String },

    #[log_event(
        name = "dcs.connected_step_decode_failed",
        severity = "error",
        result = "failed",
        message = "dcs connected step failed"
    )]
    ConnectedStepDecodeFailed { cause: String },

    #[log_event(
        name = "dcs.connected_step_already_exists",
        severity = "warn",
        result = "failed",
        message = "dcs connected step failed"
    )]
    ConnectedStepAlreadyExists { cause: String },

    #[log_event(
        name = "dcs.initial_connect_store_io_failed",
        severity = "warn",
        result = "failed",
        message = "dcs initial connect failed"
    )]
    InitialConnectStoreIoFailed { cause: String },

    #[log_event(
        name = "dcs.initial_connect_decode_failed",
        severity = "error",
        result = "failed",
        message = "dcs initial connect failed"
    )]
    InitialConnectDecodeFailed { cause: String },

    #[log_event(
        name = "dcs.initial_connect_already_exists",
        severity = "warn",
        result = "failed",
        message = "dcs initial connect failed"
    )]
    InitialConnectAlreadyExists { cause: String },

    #[log_event(
        name = "dcs.coordination_mode_transition",
        severity = "info",
        result = "ok",
        message = "dcs coordination mode transition"
    )]
    CoordinationModeTransition {
        #[log(key = "dcs.mode.previous")]
        previous: Option<crate::dcs::DcsMode>,
        #[log(key = "dcs.mode.next")]
        next: crate::dcs::DcsMode,
    },
}
