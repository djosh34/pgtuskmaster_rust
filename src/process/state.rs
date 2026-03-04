use std::time::Duration;

use tokio::sync::mpsc::UnboundedReceiver;

use crate::{
    config::ProcessConfig,
    logging::LogHandle,
    state::{JobId, StatePublisher, UnixMillis, WorkerError, WorkerStatus},
};

use super::jobs::{
    ActiveJob, BaseBackupSpec, BootstrapSpec, DemoteSpec, FencingSpec, NoopCommandRunner,
    PgRewindSpec, ProcessCommandRunner, ProcessError, ProcessHandle, PromoteSpec,
    ProcessLogIdentity, RestartPostgresSpec, StartPostgresSpec, StopPostgresSpec,
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

impl ProcessState {
    pub(crate) fn running_job_id(&self) -> Option<&JobId> {
        match self {
            Self::Idle { .. } => None,
            Self::Running { active, .. } => Some(&active.id),
        }
    }

    pub(crate) fn with_outcome(self, outcome: JobOutcome) -> Self {
        Self::Idle {
            worker: WorkerStatus::Running,
            last_outcome: Some(outcome),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ProcessJobKind {
    Bootstrap(BootstrapSpec),
    BaseBackup(BaseBackupSpec),
    PgRewind(PgRewindSpec),
    Promote(PromoteSpec),
    Demote(DemoteSpec),
    StartPostgres(StartPostgresSpec),
    StopPostgres(StopPostgresSpec),
    RestartPostgres(RestartPostgresSpec),
    Fencing(FencingSpec),
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
        finished_at: UnixMillis,
    },
    Failure {
        id: JobId,
        error: ProcessError,
        finished_at: UnixMillis,
    },
    Timeout {
        id: JobId,
        finished_at: UnixMillis,
    },
    Cancelled {
        id: JobId,
        finished_at: UnixMillis,
    },
}

pub(crate) struct ActiveRuntime {
    pub(crate) request: ProcessJobRequest,
    pub(crate) timeout_ms: u64,
    pub(crate) started_at: UnixMillis,
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
    pub(crate) command_runner: Box<dyn ProcessCommandRunner>,
    pub(crate) active_runtime: Option<ActiveRuntime>,
    pub(crate) last_rejection: Option<ProcessJobRejection>,
    pub(crate) now: Box<dyn FnMut() -> Result<UnixMillis, WorkerError> + Send>,
}

impl ProcessWorkerCtx {
    pub(crate) fn contract_stub(
        config: ProcessConfig,
        publisher: StatePublisher<ProcessState>,
        inbox: UnboundedReceiver<ProcessJobRequest>,
    ) -> Self {
        Self {
            poll_interval: Duration::from_millis(10),
            config,
            log: LogHandle::null(),
            capture_subprocess_output: false,
            state: ProcessState::Idle {
                worker: WorkerStatus::Starting,
                last_outcome: None,
            },
            publisher,
            inbox,
            command_runner: Box::new(NoopCommandRunner),
            active_runtime: None,
            last_rejection: None,
            now: Box::new(|| Ok(UnixMillis(0))),
        }
    }
}
