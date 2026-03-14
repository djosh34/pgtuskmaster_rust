use std::{fs, path::PathBuf, time::Duration};

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::{
    config::{ProcessConfig, RoleAuthConfig, RuntimeConfig},
    dcs::DcsView,
    logging::LogHandle,
    pginfo::state::PgSslMode,
    state::{JobId, MemberId, StatePublisher, StateSubscriber, UnixMillis, WorkerError, WorkerStatus},
};

use super::jobs::{
    ActiveJob, ActiveJobKind, BaseBackupSpec, BootstrapSpec, DemoteSpec, PgRewindSpec,
    ProcessCommandRunner, ProcessError, ProcessHandle, ProcessIntent, ProcessLogIdentity,
    PromoteSpec, StartPostgresSpec,
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
    pub(crate) log_identity: ProcessLogIdentity,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ManagedPostgresPaths {
    pub(crate) data_dir: PathBuf,
    pub(crate) socket_dir: PathBuf,
    pub(crate) log_file: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ManagedPostgresRuntime {
    pub(crate) paths: ManagedPostgresPaths,
    pub(crate) port: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RemoteRoleProfile {
    pub(crate) username: String,
    pub(crate) auth: RoleAuthConfig,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ReplicationSourceRuntime {
    pub(crate) replicator: RemoteRoleProfile,
    pub(crate) rewinder: RemoteRoleProfile,
    pub(crate) dbname: String,
    pub(crate) ssl_mode: PgSslMode,
    pub(crate) ssl_root_cert: Option<PathBuf>,
    pub(crate) connect_timeout_s: u32,
}

#[derive(Clone, Debug)]
pub(crate) struct ProcessRuntimePlan {
    pub(crate) postgres: ManagedPostgresRuntime,
    pub(crate) replication_source: ReplicationSourceRuntime,
}

pub(crate) struct ProcessWorkerBootstrap {
    pub(crate) cadence: ProcessCadence,
    pub(crate) config: ProcessConfig,
    pub(crate) identity: ProcessNodeIdentity,
    pub(crate) observed: ProcessObservedState,
    pub(crate) plan: ProcessRuntimePlan,
    pub(crate) state_channel: ProcessStateChannel,
    pub(crate) control: ProcessControlPlane,
    pub(crate) runtime: ProcessRuntime,
}

pub(crate) struct ProcessWorkerCtx {
    pub(crate) cadence: ProcessCadence,
    pub(crate) config: ProcessConfig,
    pub(crate) identity: ProcessNodeIdentity,
    pub(crate) observed: ProcessObservedState,
    pub(crate) plan: ProcessRuntimePlan,
    pub(crate) state_channel: ProcessStateChannel,
    pub(crate) control: ProcessControlPlane,
    pub(crate) runtime: ProcessRuntime,
}

pub(crate) struct ProcessCadence {
    pub(crate) poll_interval: Duration,
    pub(crate) now: Box<dyn FnMut() -> Result<UnixMillis, WorkerError> + Send>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessNodeIdentity {
    pub(crate) self_id: MemberId,
}

#[derive(Clone, Debug)]
pub(crate) struct ProcessObservedState {
    pub(crate) runtime_config: StateSubscriber<RuntimeConfig>,
    pub(crate) dcs: StateSubscriber<DcsView>,
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
    pub(crate) log: LogHandle,
    pub(crate) capture_subprocess_output: bool,
    pub(crate) command_runner: Box<dyn ProcessCommandRunner>,
}

impl ProcessWorkerCtx {
    pub(crate) fn new(bootstrap: ProcessWorkerBootstrap) -> Self {
        let ProcessWorkerBootstrap {
            cadence,
            config,
            identity,
            observed,
            plan,
            state_channel,
            control,
            runtime,
        } = bootstrap;
        Self {
            cadence,
            config,
            identity,
            observed,
            plan,
            state_channel,
            control,
            runtime,
        }
    }
}

impl ProcessRuntimePlan {
    pub(crate) fn from_config(cfg: &RuntimeConfig) -> Self {
        Self {
            postgres: ManagedPostgresRuntime {
                paths: ManagedPostgresPaths {
                    data_dir: cfg.postgres.data_dir.clone(),
                    socket_dir: cfg.postgres.socket_dir.clone(),
                    log_file: cfg.postgres.log_file.clone(),
                },
                port: cfg.postgres.listen_port,
            },
            replication_source: ReplicationSourceRuntime {
                replicator: RemoteRoleProfile {
                    username: cfg.postgres.roles.replicator.username.clone(),
                    auth: cfg.postgres.roles.replicator.auth.clone(),
                },
                rewinder: RemoteRoleProfile {
                    username: cfg.postgres.roles.rewinder.username.clone(),
                    auth: cfg.postgres.roles.rewinder.auth.clone(),
                },
                dbname: cfg.postgres.rewind_conn_identity.dbname.clone(),
                ssl_mode: cfg.postgres.rewind_conn_identity.ssl_mode,
                ssl_root_cert: cfg.postgres.rewind_conn_identity.ca_cert.clone(),
                connect_timeout_s: cfg.postgres.connect_timeout_s,
            },
        }
    }

    pub(crate) fn ensure_start_paths(&self) -> Result<(), ProcessError> {
        let data_dir = &self.postgres.paths.data_dir;
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

        fs::create_dir_all(&self.postgres.paths.socket_dir).map_err(|err| {
            ProcessError::InvalidSpec(format!(
                "failed to create postgres socket dir `{}`: {err}",
                self.postgres.paths.socket_dir.display()
            ))
        })?;

        if let Some(log_parent) = self.postgres.paths.log_file.parent() {
            fs::create_dir_all(log_parent).map_err(|err| {
                ProcessError::InvalidSpec(format!(
                    "failed to create postgres log dir `{}`: {err}",
                    log_parent.display()
                ))
            })?;
        }

        Ok(())
    }
}

impl ProcessState {
    pub(crate) fn starting() -> Self {
        Self::Idle {
            worker: WorkerStatus::Starting,
            last_outcome: None,
        }
    }
}
