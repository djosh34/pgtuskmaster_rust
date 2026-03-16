use std::{
    fmt,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use serde::Serialize;

use crate::support::{
    error::{HarnessError, Result},
    observer::{
        pgtm::RunnerApiContract,
        sql::{RunnerSqlCommand, RunnerSqlContract},
    },
    topology::ClusterMember,
    world::WritablePrimaryTarget,
};

const MAX_ATTEMPTS: usize = 256;
const POLL_DELAY: Duration = Duration::from_millis(100);

pub struct SqlWorkloadHandle {
    stop: Arc<AtomicBool>,
    events: Arc<Mutex<Vec<WorkloadEvent>>>,
    worker: Option<JoinHandle<std::result::Result<(), String>>>,
}

#[derive(Clone, Debug, Serialize)]
pub struct WorkloadEvent {
    pub token: String,
    pub target_member: Option<String>,
    pub started_at_ms: u128,
    pub finished_at_ms: u128,
    pub outcome: WorkloadOutcome,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WorkloadOutcome {
    Committed,
    Rejected { error: String },
}

#[derive(Clone, Debug, Serialize)]
pub struct WorkloadSummary {
    pub events: Vec<WorkloadEvent>,
}

impl SqlWorkloadHandle {
    pub fn start(
        feature_name: &str,
        table_name: &str,
        runner_api: RunnerApiContract,
        runner_sql: RunnerSqlContract,
    ) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let events = Arc::new(Mutex::new(Vec::new()));
        let stop_signal = Arc::clone(&stop);
        let shared_events = Arc::clone(&events);
        let table_name = table_name.to_string();
        let feature_name = feature_name.to_string();
        let worker = thread::spawn(move || {
            run_workload(
                feature_name.as_str(),
                table_name.as_str(),
                runner_api,
                runner_sql,
                stop_signal,
                shared_events,
            )
        });

        Self {
            stop,
            events,
            worker: Some(worker),
        }
    }

    pub fn stop(mut self) -> Result<WorkloadSummary> {
        self.stop.store(true, Ordering::SeqCst);
        let worker_result = self
            .worker
            .take()
            .ok_or_else(|| HarnessError::message("workload thread was missing"))?
            .join()
            .map_err(|_| HarnessError::message("workload thread panicked"))?;
        if let Err(err) = worker_result {
            return Err(HarnessError::message(format!(
                "workload thread failed: {err}"
            )));
        }

        let events = self
            .events
            .lock()
            .map_err(|_| HarnessError::message("workload event mutex was poisoned"))?
            .clone();
        Ok(WorkloadSummary { events })
    }

    pub fn committed_count_so_far(&self) -> Result<usize> {
        self.events
            .lock()
            .map(|events| {
                events
                    .iter()
                    .filter(|event| matches!(event.outcome, WorkloadOutcome::Committed))
                    .count()
            })
            .map_err(|_| HarnessError::message("workload event mutex was poisoned"))
    }
}

impl WorkloadSummary {
    pub fn committed_tokens(&self) -> Vec<String> {
        self.events
            .iter()
            .filter_map(|event| match &event.outcome {
                WorkloadOutcome::Committed => Some(event.token.clone()),
                WorkloadOutcome::Rejected { .. } => None,
            })
            .collect::<Vec<_>>()
    }

    pub fn committed_count(&self) -> usize {
        self.events
            .iter()
            .filter(|event| matches!(event.outcome, WorkloadOutcome::Committed))
            .count()
    }
}

impl fmt::Debug for SqlWorkloadHandle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SqlWorkloadHandle")
            .field("stop", &self.stop.load(Ordering::SeqCst))
            .finish_non_exhaustive()
    }
}

fn run_workload(
    feature_name: &str,
    table_name: &str,
    runner_api: RunnerApiContract,
    runner_sql: RunnerSqlContract,
    stop_signal: Arc<AtomicBool>,
    shared_events: Arc<Mutex<Vec<WorkloadEvent>>>,
) -> std::result::Result<(), String> {
    for sequence in 0..MAX_ATTEMPTS {
        if stop_signal.load(Ordering::SeqCst) {
            break;
        }

        let token = format!("workload-{feature_name}-{sequence}");
        let started_at_ms = timestamp_millis().map_err(|err| err.to_string())?;
        let event = match resolve_primary_target(&runner_api) {
            Ok(target) => {
                let member_id = target.member().to_string();
                let command = RunnerSqlCommand {
                    dsn: target.target().dsn.clone(),
                    sql: format!(
                        "INSERT INTO {table_name} (token) VALUES ('{token}') ON CONFLICT (token) DO NOTHING;"
                    ),
                };
                match runner_sql.execute_command(command) {
                    Ok(_) => WorkloadEvent {
                        token,
                        target_member: Some(member_id),
                        started_at_ms,
                        finished_at_ms: timestamp_millis().map_err(|err| err.to_string())?,
                        outcome: WorkloadOutcome::Committed,
                    },
                    Err(err) => WorkloadEvent {
                        token,
                        target_member: Some(member_id),
                        started_at_ms,
                        finished_at_ms: timestamp_millis().map_err(|err| err.to_string())?,
                        outcome: WorkloadOutcome::Rejected {
                            error: err.to_string(),
                        },
                    },
                }
            }
            Err(err) => WorkloadEvent {
                token,
                target_member: None,
                started_at_ms,
                finished_at_ms: timestamp_millis().map_err(|err| err.to_string())?,
                outcome: WorkloadOutcome::Rejected {
                    error: err.to_string(),
                },
            },
        };
        let stop_after_event = matches!(event.outcome, WorkloadOutcome::Rejected { .. });

        shared_events
            .lock()
            .map_err(|_| "workload event mutex was poisoned".to_string())?
            .push(event);

        if stop_after_event {
            break;
        }

        thread::sleep(POLL_DELAY);
    }

    Ok(())
}

fn resolve_primary_target(runner_api: &RunnerApiContract) -> Result<WritablePrimaryTarget> {
    runner_api.writable_primary_target().and_then(|target| {
        ClusterMember::parse(target.authority_member_id.as_str())
            .map(|member| WritablePrimaryTarget::new(member, target.route))
    })
}

fn timestamp_millis() -> Result<u128> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .map_err(|err| HarnessError::message(format!("system clock error: {err}")))
}
