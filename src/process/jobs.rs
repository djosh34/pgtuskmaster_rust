use std::{future::Future, path::PathBuf, pin::Pin};

use thiserror::Error;

use crate::pginfo::state::PgConnInfo;
use crate::state::{JobId, UnixMillis};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BootstrapSpec {
    pub(crate) data_dir: PathBuf,
    pub(crate) timeout_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PgRewindSpec {
    pub(crate) target_data_dir: PathBuf,
    pub(crate) source_conninfo: PgConnInfo,
    pub(crate) timeout_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BaseBackupSpec {
    pub(crate) data_dir: PathBuf,
    pub(crate) source_conninfo: PgConnInfo,
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
    pub(crate) wait_seconds: Option<u64>,
    pub(crate) timeout_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct StopPostgresSpec {
    pub(crate) data_dir: PathBuf,
    pub(crate) mode: ShutdownMode,
    pub(crate) timeout_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RestartPostgresSpec {
    pub(crate) data_dir: PathBuf,
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) socket_dir: PathBuf,
    pub(crate) log_file: PathBuf,
    pub(crate) mode: ShutdownMode,
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
pub(crate) enum ShutdownMode {
    Smart,
    Fast,
    Immediate,
}

impl ShutdownMode {
    pub(crate) fn as_pg_ctl_arg(&self) -> &'static str {
        match self {
            Self::Smart => "smart",
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
    StopPostgres,
    RestartPostgres,
    Fencing,
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
    pub(crate) capture_output: bool,
    pub(crate) log_identity: ProcessLogIdentity,
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

pub(crate) struct NoopCommandRunner;

impl ProcessCommandRunner for NoopCommandRunner {
    fn spawn(&mut self, _spec: ProcessCommandSpec) -> Result<Box<dyn ProcessHandle>, ProcessError> {
        Err(ProcessError::UnsupportedInput(
            "noop runner cannot spawn commands".to_string(),
        ))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum CancelReason {
    Superseded,
    Shutdown,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(crate) enum ProcessError {
    #[error("process worker operation failed")]
    OperationFailed,
    #[error("job rejected because another job is active")]
    Busy,
    #[error("invalid job spec: {0}")]
    InvalidSpec(String),
    #[error("unsupported job input: {0}")]
    UnsupportedInput(String),
    #[error("spawn failed for `{binary}`: {message}")]
    SpawnFailure { binary: String, message: String },
    #[error("process exited unsuccessfully (code: {code:?})")]
    EarlyExit { code: Option<i32> },
    #[error("job timed out after {timeout_ms} ms")]
    Timeout { timeout_ms: u64 },
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
