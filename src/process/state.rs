use std::{fs, time::Duration};

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::{
    config_v2::RuntimeConfigV2,
    dcs::DcsSnapshot,
    logging::LogSender,
    pginfo::state::PgInfoState,
    postgres_managed_conf::ManagedRecoverySignal,
    state::{
        JobId, NodeIdentity, StatePublisher, StateSubscriber, UnixMillis, WorkerError, WorkerStatus,
    },
};

use super::jobs::{
    ActiveJob, ActiveJobKind, BaseBackupSpec, BootstrapSpec, DemoteSpec, PgRewindSpec,
    ProcessCommandRunner, ProcessError, ProcessHandle, ProcessIntent, ProcessJobKind, PromoteSpec,
    StartPostgresSpec,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessState {
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
pub(crate) enum ProcessExecutionKind {
    Bootstrap(BootstrapSpec),
    BaseBackup(BaseBackupSpec),
    PgRewind(PgRewindSpec),
    Promote(PromoteSpec),
    Demote(DemoteSpec),
    StartPostgres(StartPostgresSpec),
}

impl ProcessExecutionKind {
    pub(crate) fn active_job_kind(&self) -> ActiveJobKind {
        match self {
            Self::Bootstrap(_) => ActiveJobKind::Bootstrap,
            Self::BaseBackup(_) => ActiveJobKind::BaseBackup,
            Self::PgRewind(_) => ActiveJobKind::PgRewind,
            Self::Promote(_) => ActiveJobKind::Promote,
            Self::Demote(_) => ActiveJobKind::Demote,
            Self::StartPostgres(spec) => spec.mode.active_job_kind(),
        }
    }

    pub(crate) fn process_job_kind(&self) -> ProcessJobKind {
        match self {
            Self::Bootstrap(_) => ProcessJobKind::Bootstrap,
            Self::BaseBackup(_) => ProcessJobKind::BaseBackup,
            Self::PgRewind(_) => ProcessJobKind::PgRewind,
            Self::Promote(_) => ProcessJobKind::Promote,
            Self::Demote(_) => ProcessJobKind::Demote,
            Self::StartPostgres(_) => ProcessJobKind::StartPostgres,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessIntentRequest {
    pub(crate) id: JobId,
    pub(crate) intent: ProcessIntent,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessExecutionRequest {
    pub(crate) id: JobId,
    pub(crate) kind: ProcessExecutionKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessJobRejection {
    pub(crate) id: JobId,
    pub(crate) error: ProcessError,
    pub(crate) rejected_at: UnixMillis,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobOutcome {
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
    pub(crate) request: ProcessExecutionRequest,
    pub(crate) deadline_at: UnixMillis,
    pub(crate) handle: Box<dyn ProcessHandle>,
    pub(crate) job_kind: ProcessJobKind,
}

pub(crate) struct ProcessWorkerCtx<'a> {
    pub(crate) cfg: &'a RuntimeConfigV2,
    pub(crate) cadence: ProcessCadence,
    pub(crate) identity: NodeIdentity,
    pub(crate) observed: ProcessObservedState,
    pub(crate) state_channel: ProcessStateChannel,
    pub(crate) control: ProcessControlPlane,
    pub(crate) runtime: ProcessRuntime,
}

pub(crate) struct ProcessCadence {
    pub(crate) poll_interval: Duration,
    pub(crate) now: Box<dyn FnMut() -> Result<UnixMillis, WorkerError> + Send>,
}

#[derive(Clone, Debug)]
pub(crate) struct ProcessObservedState {
    pub(crate) dcs: StateSubscriber<DcsSnapshot>,
}

#[derive(Clone, Debug)]
pub(crate) struct ProcessObservedSnapshot {
    pub(crate) dcs: DcsSnapshot,
    pub(crate) managed_recovery_state: ManagedRecoverySignal,
}

pub(crate) struct ProcessStateChannel {
    pub(crate) current: ProcessState,
    pub(crate) publisher: StatePublisher<ProcessState>,
    pub(crate) last_rejection: Option<ProcessJobRejection>,
}

pub(crate) struct ProcessControlPlane {
    pub(crate) inbox: UnboundedReceiver<ProcessIntentRequest>,
    pub(crate) inbox_disconnected_logged: bool,
    pub(crate) active_runtime: Option<ActiveRuntime>,
}

pub(crate) struct ProcessRuntime {
    pub(crate) log: LogSender,
    pub(crate) command_runner: Box<dyn ProcessCommandRunner>,
}

pub(crate) fn ensure_start_paths(cfg: &RuntimeConfigV2) -> Result<(), ProcessError> {
    for (field, path) in [
        (
            "process.binaries.overrides.postgres",
            &cfg.binaries.postgres,
        ),
        ("process.binaries.overrides.pg_ctl", &cfg.binaries.pg_ctl),
        ("process.binaries.overrides.initdb", &cfg.binaries.initdb),
        (
            "process.binaries.overrides.pg_rewind",
            &cfg.binaries.pg_rewind,
        ),
        (
            "process.binaries.overrides.pg_basebackup",
            &cfg.binaries.pg_basebackup,
        ),
        ("process.binaries.overrides.psql", &cfg.binaries.psql),
    ] {
        if !path.is_absolute() {
            return Err(ProcessError::InvalidSpec(format!(
                "{field} must be an absolute path, got `{}`",
                path.display()
            )));
        }
    }

    let data_dir = &cfg.postgres.data_dir;
    if let Some(parent) = data_dir.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            ProcessError::InvalidSpec(format!(
                "failed to create postgres data dir parent `{}`: {err}",
                parent.display()
            ))
        })?;
    }

    fs::create_dir_all(data_dir).map_err(|err| {
        ProcessError::InvalidSpec(format!(
            "failed to create postgres data dir `{}`: {err}",
            data_dir.display()
        ))
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(data_dir, fs::Permissions::from_mode(0o700)).map_err(|err| {
            ProcessError::InvalidSpec(format!(
                "failed to set postgres data dir permissions on `{}`: {err}",
                data_dir.display()
            ))
        })?;
    }

    fs::create_dir_all(&cfg.postgres.socket_dir).map_err(|err| {
        ProcessError::InvalidSpec(format!(
            "failed to create postgres socket dir `{}`: {err}",
            cfg.postgres.socket_dir.display()
        ))
    })?;

    if let Some(log_parent) = cfg.postgres.log_file.parent() {
        fs::create_dir_all(log_parent).map_err(|err| {
            ProcessError::InvalidSpec(format!(
                "failed to create postgres log dir `{}`: {err}",
                log_parent.display()
            ))
        })?;
    }

    Ok(())
}

impl ProcessState {
    pub(crate) fn starting() -> Self {
        Self::Idle {
            worker: WorkerStatus::Starting,
            last_outcome: None,
        }
    }

    pub(crate) fn active_job(&self) -> Option<&ActiveJobKind> {
        match self {
            Self::Running { active, .. } => Some(&active.kind),
            Self::Idle { .. } => None,
        }
    }

    pub(crate) fn waiting_for_pg_observation(
        &self,
        pg: &PgInfoState,
        expected: ActiveJobKind,
    ) -> bool {
        let Some(last_refresh_at) = pg.last_refresh_at() else {
            return false;
        };

        self.last_success(expected)
            .is_some_and(|finished_at| finished_at.0 >= last_refresh_at.0)
    }

    pub(crate) fn basebackup_completed_awaiting_pg_start(&self, pg: &PgInfoState) -> bool {
        let Some(basebackup_finished_at) = self.last_success(ActiveJobKind::BaseBackup) else {
            return false;
        };

        let last_start = [
            ActiveJobKind::StartPrimary,
            ActiveJobKind::StartDetachedStandby,
            ActiveJobKind::StartReplica,
        ]
        .into_iter()
        .filter_map(|job| self.last_success(job))
        .max_by_key(|finished_at| finished_at.0);

        if last_start.is_some_and(|started_at| started_at.0 >= basebackup_finished_at.0) {
            return false;
        }

        !self.waiting_for_pg_observation(pg, ActiveJobKind::BaseBackup)
    }

    pub(crate) fn rewind_failed_requires_basebackup(&self) -> bool {
        matches!(
            self,
            Self::Idle {
                last_outcome: Some(
                    JobOutcome::Failure {
                        job_kind: ActiveJobKind::PgRewind,
                        ..
                    } | JobOutcome::Timeout {
                        job_kind: ActiveJobKind::PgRewind,
                        ..
                    }
                ),
                ..
            }
        )
    }

    fn last_success(&self, expected: ActiveJobKind) -> Option<UnixMillis> {
        match self {
            Self::Idle {
                last_outcome:
                    Some(JobOutcome::Success {
                        job_kind,
                        finished_at,
                        ..
                    }),
                ..
            } if *job_kind == expected => Some(*finished_at),
            Self::Idle { .. } | Self::Running { .. } => None,
        }
    }
}
