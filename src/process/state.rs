use std::time::Duration;

use tokio::sync::mpsc::UnboundedReceiver;

use crate::{
    config::ProcessConfig,
    logging::LogHandle,
    state::{JobId, StatePublisher, UnixMillis, WorkerError, WorkerStatus},
};

use super::jobs::{
    ActiveJob, ActiveJobKind, BaseBackupSpec, BootstrapSpec, DemoteSpec, PgRewindSpec,
    ProcessCommandRunner, ProcessError, ProcessHandle, ProcessLogIdentity, PromoteSpec,
    StartPostgresSpec,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ProcessState {
    Idle {
        worker: WorkerStatus,
        last_outcome: Option<JobOutcome>,
    },
    Running {
        worker: WorkerStatus,
        active: ActiveJob,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ProcessJobKind {
    Bootstrap(BootstrapSpec),
    BaseBackup(BaseBackupSpec),
    PgRewind(PgRewindSpec),
    Promote(PromoteSpec),
    Demote(DemoteSpec),
    StartPostgres(StartPostgresSpec),
}

impl ProcessJobKind {
    pub(crate) fn label(&self) -> &'static str {
        match self {
            Self::Bootstrap(_) => "bootstrap",
            Self::BaseBackup(_) => "basebackup",
            Self::PgRewind(_) => "pg_rewind",
            Self::Promote(_) => "promote",
            Self::Demote(_) => "demote",
            Self::StartPostgres(_) => "start_postgres",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessJobRequest {
    pub(crate) id: JobId,
    pub(crate) kind: ProcessJobKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessJobRejection {
    pub(crate) id: JobId,
    pub(crate) error: ProcessError,
    pub(crate) rejected_at: UnixMillis,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum JobOutcome {
    Success {
        id: JobId,
        job_kind: ActiveJobKind,
        finished_at: UnixMillis,
    },
    Failure {
        id: JobId,
        job_kind: ActiveJobKind,
        error: ProcessError,
        finished_at: UnixMillis,
    },
    Timeout {
        id: JobId,
        job_kind: ActiveJobKind,
        finished_at: UnixMillis,
    },
}

pub(crate) struct ActiveRuntime {
    pub(crate) request: ProcessJobRequest,
    pub(crate) deadline_at: UnixMillis,
    pub(crate) handle: Box<dyn ProcessHandle>,
    pub(crate) log_identity: ProcessLogIdentity,
}

pub(crate) struct ProcessWorkerCtx {
    pub(crate) poll_interval: Duration,
    pub(crate) config: ProcessConfig,
    pub(crate) log: LogHandle,
    pub(crate) capture_subprocess_output: bool,
    pub(crate) state: ProcessState,
    pub(crate) publisher: StatePublisher<ProcessState>,
    pub(crate) inbox: UnboundedReceiver<ProcessJobRequest>,
    pub(crate) inbox_disconnected_logged: bool,
    pub(crate) command_runner: Box<dyn ProcessCommandRunner>,
    pub(crate) active_runtime: Option<ActiveRuntime>,
    pub(crate) last_rejection: Option<ProcessJobRejection>,
    pub(crate) now: Box<dyn FnMut() -> Result<UnixMillis, WorkerError> + Send>,
}
