use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use pgtuskmaster_rust::{api::NodeState, pginfo::state::PgInfoState};
use serde::Serialize;

use crate::support::{
    error::{HarnessError, Result},
    observer::pgtm::{ClusterStateObservation, MemberCommandOutcome, PgtmObserver},
    topology::ClusterMember,
};

const VIOLATION_ARTIFACT_NAME: &str = "primary-count-invariant-violation.json";

#[derive(Debug)]
pub struct PrimaryCountInvariantRunner {
    shared: Arc<SharedPrimaryCountInvariantState>,
    join_handle: Option<JoinHandle<Result<()>>>,
}

#[derive(Debug)]
struct SharedPrimaryCountInvariantState {
    stop_requested: AtomicBool,
    failure: Mutex<Option<PrimaryCountInvariantFailure>>,
}

#[derive(Clone, Debug)]
enum PrimaryCountInvariantFailure {
    Violation(PrimaryCountInvariantViolation),
    RunnerError(String),
}

#[derive(Clone, Debug)]
struct PrimaryCountInvariantViolation {
    artifact_path: PathBuf,
    sample: PrimaryCountSample,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct PrimaryCountSample {
    observed_at_ms: u128,
    allowed_primary_counts: [usize; 2],
    primary_count: usize,
    members: Vec<MemberPrimaryCountSample>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct MemberPrimaryCountSample {
    member: String,
    self_report: MemberSelfReport,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum MemberSelfReport {
    Primary,
    NotPrimary {
        pg_state: NonPrimaryPgState,
    },
    CommandFailed {
        message: String,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum NonPrimaryPgState {
    Replica,
    Unknown,
}

impl PrimaryCountInvariantRunner {
    pub fn start(
        observer: PgtmObserver,
        artifacts_dir: PathBuf,
        poll_interval: Duration,
    ) -> Result<Self> {
        let shared = Arc::new(SharedPrimaryCountInvariantState::new());
        let thread_shared = Arc::clone(&shared);
        let thread_name = "ha-primary-count-invariant".to_string();
        let join_handle = thread::Builder::new()
            .name(thread_name.clone())
            .spawn(move || {
                let result =
                    run_primary_count_invariant_loop(observer, artifacts_dir, poll_interval, &thread_shared);
                if let Err(err) = &result {
                    let _ = thread_shared.store_failure(PrimaryCountInvariantFailure::RunnerError(
                        err.to_string(),
                    ));
                }
                result
            })
            .map_err(|err| {
                HarnessError::message(format!(
                    "failed to spawn `{thread_name}` background runner: {err}"
                ))
            })?;

        Ok(Self {
            shared,
            join_handle: Some(join_handle),
        })
    }

    pub fn ensure_healthy(&self) -> Result<()> {
        self.shared.load_failure()?.map_or(Ok(()), |failure| {
            Err(HarnessError::message(failure.message()))
        })
    }

    pub fn stop(&mut self) -> Result<()> {
        self.shared.stop_requested.store(true, Ordering::SeqCst);
        let joined = self.join_handle.take().map(|handle| {
            handle.join().map_err(|_| {
                HarnessError::message("primary-count invariant runner thread panicked")
            })
        });

        if let Some(result) = joined.transpose()? {
            result?;
        }

        self.ensure_healthy()
    }
}

impl SharedPrimaryCountInvariantState {
    fn new() -> Self {
        Self {
            stop_requested: AtomicBool::new(false),
            failure: Mutex::new(None),
        }
    }

    fn stop_requested(&self) -> bool {
        self.stop_requested.load(Ordering::SeqCst)
    }

    fn load_failure(&self) -> Result<Option<PrimaryCountInvariantFailure>> {
        self.failure
            .lock()
            .map(|failure| failure.clone())
            .map_err(|_| HarnessError::message("primary-count invariant mutex was poisoned"))
    }

    fn store_failure(&self, failure: PrimaryCountInvariantFailure) -> Result<()> {
        self.failure
            .lock()
            .map(|mut slot| {
                if slot.is_none() {
                    *slot = Some(failure);
                }
            })
            .map_err(|_| HarnessError::message("primary-count invariant mutex was poisoned"))
    }
}

impl PrimaryCountInvariantFailure {
    fn message(&self) -> String {
        match self {
            Self::Violation(violation) => format!(
                "primary-count invariant violated: {}. structured sample: {}",
                violation.sample.summary(),
                violation.artifact_path.display()
            ),
            Self::RunnerError(message) => {
                format!("primary-count invariant runner failed: {message}")
            }
        }
    }
}

impl PrimaryCountInvariantViolation {
    fn new(artifact_path: PathBuf, sample: PrimaryCountSample) -> Self {
        Self {
            artifact_path,
            sample,
        }
    }
}

impl PrimaryCountSample {
    fn from_observation(observation: &ClusterStateObservation) -> Result<Self> {
        let members = observation
            .members()
            .iter()
            .map(MemberPrimaryCountSample::from_observation)
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            observed_at_ms: timestamp_millis()?,
            allowed_primary_counts: [0, 1],
            primary_count: members
                .iter()
                .filter(|member| member.self_report.is_primary())
                .count(),
            members,
        })
    }

    fn violates_allowed_primary_counts(&self) -> bool {
        !self.allowed_primary_counts.contains(&self.primary_count)
    }

    fn summary(&self) -> String {
        format!(
            "observed {} self-reported primaries ({})",
            self.primary_count,
            self.members
                .iter()
                .map(MemberPrimaryCountSample::summary)
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl MemberPrimaryCountSample {
    fn from_observation(
        observation: &crate::support::observer::pgtm::MemberStateObservation,
    ) -> Result<Self> {
        let member = observation.member;
        let self_report = match &observation.outcome {
            MemberCommandOutcome::Observed(output) => classify_self_report(member, &output.state)?,
            MemberCommandOutcome::Failed(message) => MemberSelfReport::CommandFailed {
                message: message.clone(),
            },
        };

        Ok(Self {
            member: member.service_name().to_string(),
            self_report,
        })
    }

    fn summary(&self) -> String {
        format!("{}={}", self.member, self.self_report.summary())
    }
}

impl MemberSelfReport {
    fn is_primary(&self) -> bool {
        matches!(self, Self::Primary)
    }

    fn summary(&self) -> String {
        match self {
            Self::Primary => "primary".to_string(),
            Self::NotPrimary { pg_state } => format!("not_primary({})", pg_state.label()),
            Self::CommandFailed { .. } => "command_failed".to_string(),
        }
    }
}

impl NonPrimaryPgState {
    fn label(&self) -> &'static str {
        match self {
            Self::Replica => "replica",
            Self::Unknown => "unknown",
        }
    }
}

fn run_primary_count_invariant_loop(
    observer: PgtmObserver,
    artifacts_dir: PathBuf,
    poll_interval: Duration,
    shared: &SharedPrimaryCountInvariantState,
) -> Result<()> {
    while !shared.stop_requested() {
        let observation = observer.observe_states()?;
        let sample = PrimaryCountSample::from_observation(&observation)?;
        if sample.violates_allowed_primary_counts() {
            let artifact_path = artifacts_dir.join(VIOLATION_ARTIFACT_NAME);
            persist_violation_sample(artifact_path.as_path(), &sample)?;
            shared.store_failure(PrimaryCountInvariantFailure::Violation(
                PrimaryCountInvariantViolation::new(artifact_path, sample),
            ))?;
            return Ok(());
        }
        thread::sleep(poll_interval);
    }

    Ok(())
}

fn classify_self_report(member: ClusterMember, state: &NodeState) -> Result<MemberSelfReport> {
    let reported_member = state.identity.member_id.as_str();
    if reported_member != member.service_name() {
        return Err(HarnessError::message(format!(
            "pgtm status via `{member}` returned local identity `{reported_member}`"
        )));
    }

    Ok(match state.pg {
        PgInfoState::Primary { .. } => MemberSelfReport::Primary,
        PgInfoState::Replica { .. } => MemberSelfReport::NotPrimary {
            pg_state: NonPrimaryPgState::Replica,
        },
        PgInfoState::Unknown { .. } => MemberSelfReport::NotPrimary {
            pg_state: NonPrimaryPgState::Unknown,
        },
    })
}

fn persist_violation_sample(path: &Path, sample: &PrimaryCountSample) -> Result<()> {
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }
    let rendered = serde_json::to_string_pretty(sample).map_err(|source| HarnessError::Json {
        context: "serializing primary-count invariant violation".to_string(),
        source,
    })?;
    write_text_file(path, rendered.as_str())
}

fn create_dir_all(path: &Path) -> Result<()> {
    fs::create_dir_all(path).map_err(|source| HarnessError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn write_text_file(path: &Path, content: &str) -> Result<()> {
    fs::write(path, content).map_err(|source| HarnessError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn timestamp_millis() -> Result<u128> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .map_err(|err| HarnessError::message(format!("system clock error: {err}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primary_count_sample_detects_dual_primary_violation() {
        let sample = PrimaryCountSample {
            observed_at_ms: 1,
            allowed_primary_counts: [0, 1],
            primary_count: 2,
            members: vec![
                MemberPrimaryCountSample {
                    member: "node-a".to_string(),
                    self_report: MemberSelfReport::Primary,
                },
                MemberPrimaryCountSample {
                    member: "node-b".to_string(),
                    self_report: MemberSelfReport::Primary,
                },
                MemberPrimaryCountSample {
                    member: "node-c".to_string(),
                    self_report: MemberSelfReport::NotPrimary {
                        pg_state: NonPrimaryPgState::Replica,
                    },
                },
            ],
        };

        assert!(sample.violates_allowed_primary_counts());
    }
}
