use std::{future::Future, path::PathBuf, pin::Pin};

use pgtm_log_derive::LogValue;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config_v2::{types::Secret, RuntimeConfigV2};
use crate::postgres_managed::ManagedRecoverySignal;
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

    pub(crate) fn timeout_ms(&self, cfg: &RuntimeConfigV2) -> u64 {
        let duration = match self.job_kind() {
            ProcessJobKind::PgRewind => cfg.process.timeouts.pg_rewind,
            ProcessJobKind::Demote => cfg.process.timeouts.fencing,
            ProcessJobKind::Bootstrap
            | ProcessJobKind::BaseBackup
            | ProcessJobKind::Promote
            | ProcessJobKind::StartPrimary
            | ProcessJobKind::StartDetachedStandby
            | ProcessJobKind::StartReplica => cfg.process.timeouts.bootstrap,
        };
        duration_millis_u64(duration)
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

    pub(crate) fn hot_standby(&self) -> bool {
        !matches!(self, Self::Primary)
    }

    pub(crate) fn managed_recovery_signal(&self) -> ManagedRecoverySignal {
        match self {
            Self::Primary => ManagedRecoverySignal::None,
            Self::DetachedStandby | Self::Replica { .. } => ManagedRecoverySignal::Standby,
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

fn duration_millis_u64(duration: std::time::Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}
