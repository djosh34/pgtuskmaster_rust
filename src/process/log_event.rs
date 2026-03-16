use pgtm_log_derive::{LogValue, LoggableEvent};

#[derive(Clone, Copy, Debug, PartialEq, Eq, LogValue)]
#[log_value(rename_all = "snake_case")]
pub(crate) enum CapturedStream {
    Stdout,
    Stderr,
}

#[derive(Clone, Debug, PartialEq, Eq, LoggableEvent)]
#[log_event(producer = "app", transport = "internal", parser = "app")]
pub(crate) enum ProcessLogEvent {
    #[log_event(
        name = "process.worker_run_started",
        severity = "debug",
        result = "ok",
        message = "process worker run started"
    )]
    WorkerRunStarted {
        #[log(key = "process.capture_subprocess_output")]
        capture_subprocess_output: bool,
    },

    #[log_event(
        name = "process.request_received",
        severity = "debug",
        result = "ok",
        message = "process job request received"
    )]
    RequestReceived {
        #[log(key = "job.kind")]
        job_kind: crate::process::jobs::ProcessJobKind,
    },

    #[log_event(
        name = "process.inbox_disconnected",
        severity = "warn",
        result = "failed",
        message = "process worker inbox disconnected"
    )]
    InboxDisconnected,

    #[log_event(
        name = "process.busy_rejected",
        severity = "warn",
        result = "failed",
        message = "process worker busy; rejecting job"
    )]
    BusyRejected {
        #[log(key = "job.kind")]
        job_kind: crate::process::jobs::ProcessJobKind,
    },

    #[log_event(
        name = "process.start_postgres_already_running",
        severity = "info",
        result = "ok",
        message = "start postgres preflight: postgres already running"
    )]
    StartPostgresAlreadyRunning {
        #[log(key = "postgres.data_dir")]
        data_dir: String,
    },

    #[log_event(
        name = "process.start_postgres_preflight_failed",
        severity = "error",
        result = "failed",
        message = "start postgres preflight failed"
    )]
    StartPostgresPreflightFailed { cause: String },

    #[log_event(
        name = "process.intent_materialization_failed",
        severity = "error",
        result = "failed",
        message = "process intent materialization failed"
    )]
    IntentMaterializationFailed {
        #[log(key = "job.kind")]
        job_kind: crate::process::jobs::ProcessJobKind,
        cause: String,
    },

    #[log_event(
        name = "process.build_command_failed",
        severity = "error",
        result = "failed",
        message = "process build command failed"
    )]
    BuildCommandFailed {
        #[log(key = "job.kind")]
        job_kind: crate::process::jobs::ProcessJobKind,
        cause: String,
    },

    #[log_event(
        name = "process.spawn_failed",
        severity = "error",
        result = "failed",
        message = "process spawn failed"
    )]
    SpawnFailed {
        #[log(key = "job.kind")]
        job_kind: crate::process::jobs::ProcessJobKind,
        cause: String,
    },

    #[log_event(
        name = "process.started",
        severity = "info",
        result = "ok",
        message = "process job started"
    )]
    Started {
        #[log(key = "job.kind")]
        job_kind: crate::process::jobs::ProcessJobKind,
    },

    #[log_event(
        name = "process.output_drain_failed",
        severity = "warn",
        result = "failed",
        message = "process output drain failed"
    )]
    OutputDrainFailed {
        #[log(key = "job.kind")]
        job_kind: crate::process::jobs::ProcessJobKind,
        cause: String,
    },

    #[log_event(
        name = "process.timeout",
        severity = "warn",
        result = "timeout",
        message = "process job timed out; cancelling"
    )]
    Timeout {
        #[log(key = "job.kind")]
        job_kind: crate::process::jobs::ProcessJobKind,
    },

    #[log_event(
        name = "process.exited_successfully",
        severity = "info",
        result = "ok",
        message = "process job exited successfully"
    )]
    ExitedSuccessfully {
        #[log(key = "job.kind")]
        job_kind: crate::process::jobs::ProcessJobKind,
    },

    #[log_event(
        name = "process.exited_unsuccessfully",
        severity = "warn",
        result = "failed",
        message = "process job exited unsuccessfully"
    )]
    ExitedUnsuccessfully {
        #[log(key = "job.kind")]
        job_kind: crate::process::jobs::ProcessJobKind,
        cause: String,
    },

    #[log_event(
        name = "process.poll_failed",
        severity = "error",
        result = "failed",
        message = "process job poll failed"
    )]
    PollFailed {
        #[log(key = "job.kind")]
        job_kind: crate::process::jobs::ProcessJobKind,
        cause: String,
    },

    #[log_event(
        name = "process.output_emit_failed",
        severity = "warn",
        result = "failed",
        message = "process subprocess output emit failed"
    )]
    OutputEmitFailed {
        #[log(key = "job.kind")]
        job_kind: crate::process::jobs::ProcessJobKind,
        #[log(key = "process.stream")]
        stream: CapturedStream,
        cause: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, LoggableEvent)]
pub(crate) enum SubprocessLogEvent {
    #[log_event(name = "process.subprocess_line", meta = "computed")]
    Line {
        #[log(key = "job.kind")]
        job_kind: crate::process::jobs::ProcessJobKind,
        #[log(key = "process.stream")]
        stream: CapturedStream,
        #[log(skip)]
        line: String,
    },
}
