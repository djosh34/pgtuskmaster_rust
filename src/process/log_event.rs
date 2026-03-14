use std::borrow::Cow;

use crate::logging::{
    DomainLogEvent, LogEventMetadata, LogEventResult, LogEventSource, LogFieldVisitor,
    LogParser, LogProducer, LogTransport, SealedLogEvent, SeverityText,
};

use super::jobs::ProcessJobKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProcessLogOrigin {
    Run,
    StepOnce,
    StartJob,
    TickActiveJob,
    EmitSubprocessLine,
}

impl ProcessLogOrigin {
    fn label(self) -> &'static str {
        match self {
            Self::Run => "process_worker::run",
            Self::StepOnce => "process_worker::step_once",
            Self::StartJob => "process_worker::start_job",
            Self::TickActiveJob => "process_worker::tick_active_job",
            Self::EmitSubprocessLine => "process_worker::emit_subprocess_line",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessJobIdentity {
    pub(crate) job_id: String,
    pub(crate) kind: ProcessJobKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessExecutionIdentity {
    pub(crate) job: ProcessJobIdentity,
    pub(crate) binary: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CapturedStream {
    Stdout,
    Stderr,
}

impl CapturedStream {
    fn severity(self) -> SeverityText {
        match self {
            Self::Stdout => SeverityText::Info,
            Self::Stderr => SeverityText::Warn,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Stdout => "stdout",
            Self::Stderr => "stderr",
        }
    }

    fn transport(self) -> LogTransport {
        match self {
            Self::Stdout => LogTransport::ChildStdout,
            Self::Stderr => LogTransport::ChildStderr,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ProcessLogEvent {
    WorkerRunStarted {
        origin: ProcessLogOrigin,
        capture_subprocess_output: bool,
    },
    RequestReceived {
        origin: ProcessLogOrigin,
        job: ProcessJobIdentity,
    },
    InboxDisconnected {
        origin: ProcessLogOrigin,
    },
    BusyRejected {
        origin: ProcessLogOrigin,
        job: ProcessJobIdentity,
    },
    StartPostgresAlreadyRunning {
        origin: ProcessLogOrigin,
        job: ProcessJobIdentity,
        data_dir: String,
    },
    StartPostgresPreflightFailed {
        origin: ProcessLogOrigin,
        job: ProcessJobIdentity,
        error: String,
    },
    IntentMaterializationFailed {
        origin: ProcessLogOrigin,
        job: ProcessJobIdentity,
        error: String,
    },
    BuildCommandFailed {
        origin: ProcessLogOrigin,
        job: ProcessJobIdentity,
        error: String,
    },
    SpawnFailed {
        origin: ProcessLogOrigin,
        job: ProcessJobIdentity,
        error: String,
    },
    Started {
        origin: ProcessLogOrigin,
        execution: ProcessExecutionIdentity,
    },
    OutputDrainFailed {
        origin: ProcessLogOrigin,
        execution: ProcessExecutionIdentity,
        error: String,
    },
    Timeout {
        origin: ProcessLogOrigin,
        execution: ProcessExecutionIdentity,
    },
    ExitedSuccessfully {
        origin: ProcessLogOrigin,
        execution: ProcessExecutionIdentity,
    },
    ExitedUnsuccessfully {
        origin: ProcessLogOrigin,
        execution: ProcessExecutionIdentity,
        error: String,
    },
    PollFailed {
        origin: ProcessLogOrigin,
        execution: ProcessExecutionIdentity,
        error: String,
    },
    OutputEmitFailed {
        origin: ProcessLogOrigin,
        execution: ProcessExecutionIdentity,
        stream: CapturedStream,
        bytes_len: usize,
        error: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SubprocessLogEvent {
    pub(crate) producer: LogProducer,
    pub(crate) origin: ProcessLogOrigin,
    pub(crate) execution: ProcessExecutionIdentity,
    pub(crate) stream: CapturedStream,
    pub(crate) bytes: Vec<u8>,
}

impl SealedLogEvent for ProcessLogEvent {}

impl DomainLogEvent for ProcessLogEvent {
    fn metadata(&self) -> LogEventMetadata {
        match self {
            Self::WorkerRunStarted { origin, .. } => event_metadata(
                SeverityText::Debug,
                "process worker run started",
                "process.worker.run_started",
                LogEventResult::Ok,
                *origin,
            ),
            Self::RequestReceived { origin, .. } => event_metadata(
                SeverityText::Debug,
                "process job request received",
                "process.worker.request_received",
                LogEventResult::Ok,
                *origin,
            ),
            Self::InboxDisconnected { origin } => event_metadata(
                SeverityText::Warn,
                "process worker inbox disconnected",
                "process.worker.inbox_disconnected",
                LogEventResult::Failed,
                *origin,
            ),
            Self::BusyRejected { origin, .. } => event_metadata(
                SeverityText::Warn,
                "process worker busy; rejecting job",
                "process.worker.busy_reject",
                LogEventResult::Failed,
                *origin,
            ),
            Self::StartPostgresAlreadyRunning { origin, .. } => event_metadata(
                SeverityText::Info,
                "start postgres preflight: postgres already running",
                "process.job.start_postgres_noop",
                LogEventResult::Ok,
                *origin,
            ),
            Self::StartPostgresPreflightFailed { origin, .. } => event_metadata(
                SeverityText::Error,
                "start postgres preflight failed",
                "process.job.start_postgres_preflight_failed",
                LogEventResult::Failed,
                *origin,
            ),
            Self::IntentMaterializationFailed { origin, .. } => event_metadata(
                SeverityText::Error,
                "process intent materialization failed",
                "process.worker.intent_materialization_failed",
                LogEventResult::Failed,
                *origin,
            ),
            Self::BuildCommandFailed { origin, .. } => event_metadata(
                SeverityText::Error,
                "process build command failed",
                "process.job.build_command_failed",
                LogEventResult::Failed,
                *origin,
            ),
            Self::SpawnFailed { origin, .. } => event_metadata(
                SeverityText::Error,
                "process spawn failed",
                "process.job.spawn_failed",
                LogEventResult::Failed,
                *origin,
            ),
            Self::Started { origin, .. } => event_metadata(
                SeverityText::Info,
                "process job started",
                "process.job.started",
                LogEventResult::Ok,
                *origin,
            ),
            Self::OutputDrainFailed { origin, .. } => event_metadata(
                SeverityText::Warn,
                "process output drain failed",
                "process.worker.output_drain_failed",
                LogEventResult::Failed,
                *origin,
            ),
            Self::Timeout { origin, .. } => event_metadata(
                SeverityText::Warn,
                "process job timed out; cancelling",
                "process.job.timeout",
                LogEventResult::Timeout,
                *origin,
            ),
            Self::ExitedSuccessfully { origin, .. } => event_metadata(
                SeverityText::Info,
                "process job exited successfully",
                "process.job.exited",
                LogEventResult::Ok,
                *origin,
            ),
            Self::ExitedUnsuccessfully { origin, .. } => event_metadata(
                SeverityText::Warn,
                "process job exited unsuccessfully",
                "process.job.exited",
                LogEventResult::Failed,
                *origin,
            ),
            Self::PollFailed { origin, .. } => event_metadata(
                SeverityText::Error,
                "process job poll failed",
                "process.job.poll_failed",
                LogEventResult::Failed,
                *origin,
            ),
            Self::OutputEmitFailed { origin, .. } => event_metadata(
                SeverityText::Warn,
                "process subprocess output emit failed",
                "process.worker.output_emit_failed",
                LogEventResult::Failed,
                *origin,
            ),
        }
    }

    fn write_fields(&self, visitor: &mut dyn LogFieldVisitor) {
        match self {
            Self::WorkerRunStarted {
                capture_subprocess_output,
                ..
            } => visitor.bool("capture_subprocess_output", *capture_subprocess_output),
            Self::RequestReceived { job, .. } | Self::BusyRejected { job, .. } => {
                write_job(visitor, job);
            }
            Self::InboxDisconnected { .. } => {}
            Self::StartPostgresAlreadyRunning { job, data_dir, .. } => {
                write_job(visitor, job);
                visitor.string("data_dir", data_dir.clone());
            }
            Self::StartPostgresPreflightFailed { job, error, .. }
            | Self::IntentMaterializationFailed { job, error, .. }
            | Self::BuildCommandFailed { job, error, .. }
            | Self::SpawnFailed { job, error, .. } => {
                write_job(visitor, job);
                visitor.string("error", error.clone());
            }
            Self::Started { execution, .. }
            | Self::Timeout { execution, .. }
            | Self::ExitedSuccessfully { execution, .. } => {
                write_execution(visitor, execution);
            }
            Self::OutputDrainFailed {
                execution,
                error,
                ..
            }
            | Self::ExitedUnsuccessfully {
                execution,
                error,
                ..
            }
            | Self::PollFailed {
                execution,
                error,
                ..
            } => {
                write_execution(visitor, execution);
                visitor.string("error", error.clone());
            }
            Self::OutputEmitFailed {
                execution,
                stream,
                bytes_len,
                error,
                ..
            } => {
                write_execution(visitor, execution);
                visitor.str("stream", stream.label());
                visitor.usize("bytes_len", *bytes_len);
                visitor.string("error", error.clone());
            }
        }
    }
}

impl SealedLogEvent for SubprocessLogEvent {}

impl DomainLogEvent for SubprocessLogEvent {
    fn metadata(&self) -> LogEventMetadata {
        LogEventMetadata {
            severity: self.stream.severity(),
            message: Cow::Owned(String::from_utf8_lossy(self.bytes.as_slice()).into_owned()),
            event_name: "process.subprocess.line",
            event_domain: "process",
            event_result: LogEventResult::Ok,
            source: LogEventSource::new(
                self.producer,
                self.stream.transport(),
                LogParser::Raw,
                self.origin.label(),
            ),
        }
    }

    fn write_fields(&self, visitor: &mut dyn LogFieldVisitor) {
        write_execution(visitor, &self.execution);
        visitor.str("stream", self.stream.label());
    }
}

fn event_metadata(
    severity: SeverityText,
    message: &'static str,
    event_name: &'static str,
    event_result: LogEventResult,
    origin: ProcessLogOrigin,
) -> LogEventMetadata {
    LogEventMetadata {
        severity,
        message: Cow::Borrowed(message),
        event_name,
        event_domain: "process",
        event_result,
        source: LogEventSource::app(origin.label()),
    }
}

fn write_job(visitor: &mut dyn LogFieldVisitor, identity: &ProcessJobIdentity) {
    visitor.string("job.id", identity.job_id.clone());
    visitor.str("job.kind", process_job_kind_label(identity.kind));
}

fn write_execution(visitor: &mut dyn LogFieldVisitor, execution: &ProcessExecutionIdentity) {
    write_job(visitor, &execution.job);
    visitor.string("binary", execution.binary.clone());
}

fn process_job_kind_label(kind: ProcessJobKind) -> &'static str {
    match kind {
        ProcessJobKind::Bootstrap => "bootstrap",
        ProcessJobKind::BaseBackup => "basebackup",
        ProcessJobKind::PgRewind => "pg_rewind",
        ProcessJobKind::Promote => "promote",
        ProcessJobKind::Demote => "demote",
        ProcessJobKind::StartPostgres => "start_postgres",
        ProcessJobKind::StartPrimary => "start_primary",
        ProcessJobKind::StartDetachedStandby => "start_detached_standby",
        ProcessJobKind::StartReplica => "start_replica",
    }
}
