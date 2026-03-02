use crate::state::{JobId, UnixMillis, WorkerStatus};

use super::jobs::{
    ActiveJob, BootstrapSpec, DemoteSpec, FencingSpec, PgRewindSpec, ProcessError, PromoteSpec,
    RestartPostgresSpec, StartPostgresSpec, StopPostgresSpec,
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
    PgRewind(PgRewindSpec),
    Promote(PromoteSpec),
    Demote(DemoteSpec),
    StartPostgres(StartPostgresSpec),
    StopPostgres(StopPostgresSpec),
    RestartPostgres(RestartPostgresSpec),
    Fencing(FencingSpec),
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessWorkerCtx {
    pub(crate) _private: (),
}
