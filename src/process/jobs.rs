use std::{future::Future, path::PathBuf, pin::Pin};

use pgtm_log_derive::LogValue;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config_v2::types::Secret;
use crate::pginfo::state::PgConnInfo;
use crate::state::{JobId, MemberId, UnixMillis};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessIntent {
    Bootstrap,
    ProvisionReplica(ReplicaProvisionIntent),
    Start(PostgresStartIntent),
    Promote,
    Demote(ShutdownMode),
}

impl ProcessIntent {
    pub(crate) fn job_kind(&self) -> ProcessJobKind {
        match self {
            Self::Bootstrap => ProcessJobKind::Bootstrap,
            Self::ProvisionReplica(ReplicaProvisionIntent::BaseBackup { .. }) => {
                ProcessJobKind::BaseBackup
            }
            Self::ProvisionReplica(ReplicaProvisionIntent::PgRewind { .. }) => {
                ProcessJobKind::PgRewind
            }
            Self::Start(start) => start.job_kind(),
            Self::Promote => ProcessJobKind::Promote,
            Self::Demote(_) => ProcessJobKind::Demote,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplicaProvisionIntent {
    BaseBackup { leader: MemberId },
    PgRewind { leader: MemberId },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PostgresStartIntent {
    Primary,
    DetachedStandby,
    Replica { leader: MemberId },
}

impl PostgresStartIntent {
    pub(crate) fn job_kind(&self) -> ProcessJobKind {
        match self {
            Self::Primary => ProcessJobKind::StartPrimary,
            Self::DetachedStandby => ProcessJobKind::StartDetachedStandby,
            Self::Replica { .. } => ProcessJobKind::StartReplica,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BootstrapSpec {
    pub(crate) data_dir: PathBuf,
    pub(crate) superuser: String,
    pub(crate) timeout_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum MandatorySourceRole {
    Replicator,
    Rewinder,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MandatoryRoleSourceConn {
    pub(crate) role: MandatorySourceRole,
    pub(crate) conninfo: PgConnInfo,
    pub(crate) auth: Secret,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PgRewindSpec {
    pub(crate) target_data_dir: PathBuf,
    pub(crate) source: MandatoryRoleSourceConn,
    pub(crate) timeout_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BaseBackupSpec {
    pub(crate) data_dir: PathBuf,
    pub(crate) source: MandatoryRoleSourceConn,
    pub(crate) timeout_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PromoteSpec {
    pub(crate) data_dir: PathBuf,
    pub(crate) wait_seconds: Option<u64>,
    pub(crate) timeout_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DemoteSpec {
    pub(crate) data_dir: PathBuf,
    pub(crate) mode: ShutdownMode,
    pub(crate) timeout_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct StartPostgresSpec {
    pub(crate) data_dir: PathBuf,
    pub(crate) config_file: PathBuf,
    pub(crate) log_file: PathBuf,
    pub(crate) primary_conninfo: Option<PgConnInfo>,
    pub(crate) primary_slot_name: Option<String>,
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
    pub(crate) fn job_kind(&self) -> ProcessJobKind {
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

impl ShutdownMode {
    pub(crate) fn as_pg_ctl_arg(&self) -> &'static str {
        match self {
            Self::Fast => "fast",
            Self::Immediate => "immediate",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShutdownMode {
    Fast,
    Immediate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, LogValue)]
#[log_value(rename_all = "snake_case")]
pub enum ProcessJobKind {
    Bootstrap,
    BaseBackup,
    PgRewind,
    Promote,
    Demote,
    StartPostgres,
    StartPrimary,
    StartDetachedStandby,
    StartReplica,
}

impl ProcessJobKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Bootstrap => "bootstrap",
            Self::BaseBackup => "basebackup",
            Self::PgRewind => "pg_rewind",
            Self::Promote => "promote",
            Self::Demote => "demote",
            Self::StartPostgres => "start_postgres",
            Self::StartPrimary => "start_primary",
            Self::StartDetachedStandby => "start_detached_standby",
            Self::StartReplica => "start_replica",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveJob {
    pub id: JobId,
    pub kind: ProcessJobKind,
    pub started_at: UnixMillis,
    pub deadline_at: UnixMillis,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessCommandSpec {
    pub(crate) program: PathBuf,
    pub(crate) args: Vec<String>,
    pub(crate) env: Vec<ProcessEnvVar>,
    pub(crate) capture_output: bool,
    pub(crate) job_kind: ProcessJobKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessEnvVar {
    pub(crate) key: String,
    pub(crate) value: ProcessEnvValue,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ProcessEnvValue {
    Secret(Secret),
}

impl ProcessEnvValue {
    pub(crate) fn resolve_string_for_key(&self, key: &str) -> Result<String, ProcessError> {
        match self {
            Self::Secret(secret) => {
                if key.trim().is_empty() {
                    return Err(ProcessError::EnvSecretResolutionFailed {
                        key: key.to_string(),
                        message: "environment key must not be empty".to_string(),
                    });
                }
                Ok(secret.as_str().to_string())
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProcessOutputStream {
    Stdout,
    Stderr,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessOutputLine {
    pub(crate) stream: ProcessOutputStream,
    pub(crate) bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ProcessExit {
    Success,
    Failure { code: Option<i32> },
}

pub(crate) trait ProcessHandle: Send {
    fn poll_exit(&mut self) -> Result<Option<ProcessExit>, ProcessError>;
    fn drain_output<'a>(
        &'a mut self,
        max_bytes: usize,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<ProcessOutputLine>, ProcessError>> + Send + 'a>>;
    fn cancel<'a>(
        &'a mut self,
    ) -> Pin<Box<dyn Future<Output = Result<(), ProcessError>> + Send + 'a>>;
}

pub(crate) trait ProcessCommandRunner: Send {
    fn spawn(&mut self, spec: ProcessCommandSpec) -> Result<Box<dyn ProcessHandle>, ProcessError>;
}

#[derive(Clone, Debug, Error, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessError {
    #[error("process worker operation failed")]
    OperationFailed,
    #[error("job rejected because another job is active")]
    Busy,
    #[error("invalid job spec: {0}")]
    InvalidSpec(String),
    #[error("failed to resolve secret for env `{key}`: {message}")]
    EnvSecretResolutionFailed { key: String, message: String },
    #[error("spawn failed for `{binary}`: {message}")]
    SpawnFailure { binary: String, message: String },
    #[error("process exited unsuccessfully (code: {code:?})")]
    EarlyExit { code: Option<i32> },
    #[error("job cancellation failed: {0}")]
    CancelFailure(String),
}

impl ProcessError {
    pub(crate) fn from_exit(exit: ProcessExit) -> Self {
        match exit {
            ProcessExit::Success => Self::OperationFailed,
            ProcessExit::Failure { code } => Self::EarlyExit { code },
        }
    }
}
