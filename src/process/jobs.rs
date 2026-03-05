use std::{collections::BTreeMap, future::Future, path::PathBuf, pin::Pin};

use thiserror::Error;

use crate::config::{InlineOrPath, RoleAuthConfig, SecretSource};
use crate::pginfo::state::PgConnInfo;
use crate::state::{JobId, UnixMillis};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BootstrapSpec {
    pub(crate) data_dir: PathBuf,
    pub(crate) superuser_username: String,
    pub(crate) timeout_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ReplicatorSourceConn {
    pub(crate) conninfo: PgConnInfo,
    pub(crate) auth: RoleAuthConfig,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RewinderSourceConn {
    pub(crate) conninfo: PgConnInfo,
    pub(crate) auth: RoleAuthConfig,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PgRewindSpec {
    pub(crate) target_data_dir: PathBuf,
    pub(crate) source: RewinderSourceConn,
    pub(crate) timeout_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BaseBackupSpec {
    pub(crate) data_dir: PathBuf,
    pub(crate) source: ReplicatorSourceConn,
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
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) socket_dir: PathBuf,
    pub(crate) log_file: PathBuf,
    pub(crate) extra_postgres_settings: BTreeMap<String, String>,
    pub(crate) wait_seconds: Option<u64>,
    pub(crate) timeout_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FencingSpec {
    pub(crate) data_dir: PathBuf,
    pub(crate) mode: ShutdownMode,
    pub(crate) timeout_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PgBackRestVersionSpec {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PgBackRestInfoSpec {
    pub(crate) stanza: String,
    pub(crate) repo: String,
    pub(crate) options: Vec<String>,
    pub(crate) timeout_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PgBackRestCheckSpec {
    pub(crate) stanza: String,
    pub(crate) repo: String,
    pub(crate) options: Vec<String>,
    pub(crate) timeout_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PgBackRestBackupSpec {
    pub(crate) stanza: String,
    pub(crate) repo: String,
    pub(crate) options: Vec<String>,
    pub(crate) timeout_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PgBackRestRestoreSpec {
    pub(crate) stanza: String,
    pub(crate) repo: String,
    pub(crate) pg1_path: PathBuf,
    pub(crate) options: Vec<String>,
    pub(crate) timeout_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PgBackRestArchivePushSpec {
    pub(crate) stanza: String,
    pub(crate) repo: String,
    pub(crate) pg1_path: PathBuf,
    pub(crate) wal_path: String,
    pub(crate) options: Vec<String>,
    pub(crate) timeout_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PgBackRestArchiveGetSpec {
    pub(crate) stanza: String,
    pub(crate) repo: String,
    pub(crate) pg1_path: PathBuf,
    pub(crate) wal_segment: String,
    pub(crate) destination_path: String,
    pub(crate) options: Vec<String>,
    pub(crate) timeout_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ShutdownMode {
    Fast,
    Immediate,
}

impl ShutdownMode {
    pub(crate) fn as_pg_ctl_arg(&self) -> &'static str {
        match self {
            Self::Fast => "fast",
            Self::Immediate => "immediate",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ActiveJobKind {
    Bootstrap,
    BaseBackup,
    PgRewind,
    Promote,
    Demote,
    StartPostgres,
    Fencing,
    PgBackRestVersion,
    PgBackRestInfo,
    PgBackRestCheck,
    PgBackRestBackup,
    PgBackRestRestore,
    PgBackRestArchivePush,
    PgBackRestArchiveGet,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ActiveJob {
    pub(crate) id: JobId,
    pub(crate) kind: ActiveJobKind,
    pub(crate) started_at: UnixMillis,
    pub(crate) deadline_at: UnixMillis,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessCommandSpec {
    pub(crate) program: PathBuf,
    pub(crate) args: Vec<String>,
    pub(crate) env: Vec<ProcessEnvVar>,
    pub(crate) capture_output: bool,
    pub(crate) log_identity: ProcessLogIdentity,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessEnvVar {
    pub(crate) key: String,
    pub(crate) value: ProcessEnvValue,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ProcessEnvValue {
    Secret(SecretSource),
}

impl ProcessEnvValue {
    pub(crate) fn resolve_string_for_key(&self, key: &str) -> Result<String, ProcessError> {
        match self {
            Self::Secret(secret) => resolve_secret_source_string(key, secret),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessLogIdentity {
    pub(crate) job_id: JobId,
    pub(crate) job_kind: String,
    pub(crate) binary: String,
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

#[cfg(test)]
pub(crate) struct NoopCommandRunner;

#[cfg(test)]
impl ProcessCommandRunner for NoopCommandRunner {
    fn spawn(&mut self, _spec: ProcessCommandSpec) -> Result<Box<dyn ProcessHandle>, ProcessError> {
        Err(ProcessError::InvalidSpec(
            "noop runner cannot spawn commands".to_string(),
        ))
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(crate) enum ProcessError {
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

fn resolve_secret_source_string(key: &str, secret: &SecretSource) -> Result<String, ProcessError> {
    let value = match &secret.0 {
        InlineOrPath::Path(path) | InlineOrPath::PathConfig { path } => {
            std::fs::read_to_string(path).map_err(|err| ProcessError::EnvSecretResolutionFailed {
                key: key.to_string(),
                message: format!("failed to read {}: {err}", path.display()),
            })?
        }
        InlineOrPath::Inline { content } => content.clone(),
    };
    Ok(value
        .trim_end_matches(['\n', '\r'])
        .to_string())
}
