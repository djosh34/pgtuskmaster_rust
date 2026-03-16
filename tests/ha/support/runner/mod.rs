use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex, OnceLock,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use pgtuskmaster_test_support::ha_runner::{
    RunnerCommand, RunnerRequest, RunnerResponse, RunnerResponsePayload, CONTAINER_ARTIFACTS_DIR,
    CONTAINER_CONTRACT_DIR, CONTAINER_MATERIALIZED_DIR, CONTAINER_SCENARIO_DIR,
};
use serde_json::error::Category as JsonErrorCategory;

use crate::support::error::{HarnessError, Result};

use crate::support::topology::{ClusterMember, RunnerService};

pub const HA_RUNNER_SERVICE_NAME: &str = "ha-runner";
pub const HA_RUNNER_BINARY_PATH: &str = "/usr/local/bin/pgtm-ha-runner";
pub const HA_RUNNER_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
pub const HA_RUNNER_CONTAINER_SCENARIO_DIR: &str = CONTAINER_SCENARIO_DIR;
pub const HA_RUNNER_CONTAINER_MATERIALIZED_DIR: &str = CONTAINER_MATERIALIZED_DIR;
pub const HA_RUNNER_CONTAINER_CONTRACT_DIR: &str = CONTAINER_CONTRACT_DIR;
pub const HA_RUNNER_CONTAINER_ARTIFACTS_DIR: &str = CONTAINER_ARTIFACTS_DIR;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunnerReadPlane {
    DirectNetwork,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunnerControlPlane {
    InContainerDockerSocket,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunnerExecutable {
    pub binary_path: PathBuf,
    pub arguments: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunnerSeed {
    pub member: ClusterMember,
    pub config_path: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunnerSeedSet {
    pub seeds: Vec<RunnerSeed>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunnerMountSet {
    pub scenario_dir: PathBuf,
    pub materialized_dir: PathBuf,
    pub contract_dir: PathBuf,
    pub artifacts_dir: PathBuf,
    pub docker_socket: Option<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunnerSessionContract {
    pub launch_request_path: PathBuf,
    pub progress_path: PathBuf,
    pub result_path: PathBuf,
    pub timeline_path: PathBuf,
    pub stdout_path: PathBuf,
    pub stderr_path: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunnerServiceSpec {
    pub service: RunnerService,
    pub executable: RunnerExecutable,
    pub read_plane: RunnerReadPlane,
    pub control_plane: RunnerControlPlane,
    pub seeds: RunnerSeedSet,
    pub mounts: RunnerMountSet,
    pub contract: RunnerSessionContract,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunnerProcessHandle {
    pub service: RunnerService,
    pub container_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunnerSessionHandle {
    pub process: RunnerProcessHandle,
    pub contract: RunnerSessionContract,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InNetworkScenarioRunner {
    pub spec: RunnerServiceSpec,
    pub session: RunnerSessionHandle,
}

static RUNNER_REQUEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
static RUNNER_REQUEST_COUNTER: AtomicU64 = AtomicU64::new(1);

pub fn run_feature_test(feature_name: &str, feature_path: &str) -> std::result::Result<(), String> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|err| format!("failed to build tokio runtime: {err}"))?;
    runtime.block_on(crate::support::run_feature(feature_name, feature_path))
}

pub fn run_contract_command(
    session: &RunnerSessionHandle,
    command: RunnerCommand,
) -> Result<RunnerResponsePayload> {
    let request_lock = RUNNER_REQUEST_LOCK.get_or_init(|| Mutex::new(()));
    let _guard = request_lock
        .lock()
        .map_err(|_| HarnessError::message("runner request mutex was poisoned"))?;
    clear_contract_file(session.contract.result_path.as_path())?;
    let request = RunnerRequest {
        request_id: next_request_id()?,
        command,
    };
    write_request(session.contract.launch_request_path.as_path(), &request)?;

    let started = std::time::Instant::now();
    while started.elapsed() < HA_RUNNER_REQUEST_TIMEOUT {
        match read_response(session.contract.result_path.as_path())? {
            Some(response) if response.request_id == request.request_id => {
                return match response.payload {
                    RunnerResponsePayload::Error { message } => Err(HarnessError::message(message)),
                    payload => Ok(payload),
                };
            }
            Some(_) | None => {
                std::thread::sleep(Duration::from_millis(100));
            }
        }
    }

    Err(HarnessError::message(format!(
        "timed out waiting for runner response at `{}`",
        session.contract.result_path.display()
    )))
}

fn next_request_id() -> Result<String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| HarnessError::message(format!("system clock error: {err}")))?;
    let sequence = RUNNER_REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    Ok(format!(
        "{}-{}-{}",
        std::process::id(),
        now.as_millis(),
        sequence
    ))
}

fn write_request(path: &Path, request: &RunnerRequest) -> Result<()> {
    let rendered = serde_json::to_string_pretty(request).map_err(|source| HarnessError::Json {
        context: "serializing runner request".to_string(),
        source,
    })?;
    fs::write(path, rendered).map_err(|source| HarnessError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn read_response(path: &Path) -> Result<Option<RunnerResponse>> {
    match fs::read_to_string(path) {
        Ok(contents) => match serde_json::from_str(contents.as_str()) {
            Ok(response) => Ok(Some(response)),
            Err(source) if is_transient_json_read_error(&source) => Ok(None),
            Err(source) => Err(HarnessError::Json {
                context: format!("parsing runner response `{}`", path.display()),
                source,
            }),
        },
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(HarnessError::Io {
            path: path.to_path_buf(),
            source,
        }),
    }
}

fn clear_contract_file(path: &Path) -> Result<()> {
    fs::write(path, "").map_err(|source| HarnessError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn is_transient_json_read_error(source: &serde_json::Error) -> bool {
    source.classify() == JsonErrorCategory::Eof
}
