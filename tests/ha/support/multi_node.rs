#![allow(dead_code)]

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use clap::Parser;
use tokio::task::JoinHandle;

use super::observer::{
    assert_no_dual_primary_in_samples, HaInvariantObserver, HaObservationStats, HaObserverConfig,
};

use pgtuskmaster_rust::{
    api::{AcceptedResponse as CliAcceptedResponse, HaStateResponse},
    cli::{self, args::Cli, client::CliApiClient, error::CliError},
    state::WorkerError,
    test_harness::ha_e2e,
};

use pgtuskmaster_rust::test_harness::ha_e2e::handle::TestClusterHandle;

struct ClusterFixture {
    _guard: pgtuskmaster_rust::test_harness::namespace::NamespaceGuard,
    pg_ctl_bin: PathBuf,
    psql_bin: PathBuf,
    superuser_username: String,
    superuser_dbname: String,
    etcd: Option<pgtuskmaster_rust::test_harness::etcd3::EtcdClusterHandle>,
    nodes: Vec<ha_e2e::NodeHandle>,
    tasks: Vec<JoinHandle<Result<(), WorkerError>>>,
    timeline: Vec<String>,
}

const E2E_COMMAND_TIMEOUT: Duration = Duration::from_secs(30);
const E2E_COMMAND_KILL_WAIT_TIMEOUT: Duration = Duration::from_secs(3);
const E2E_SQL_WORKLOAD_COMMAND_TIMEOUT: Duration = Duration::from_secs(3);
const E2E_SQL_WORKLOAD_COMMAND_KILL_WAIT_TIMEOUT: Duration = Duration::from_secs(1);
const E2E_PG_STOP_TIMEOUT: Duration = Duration::from_secs(10);
const E2E_HTTP_STEP_TIMEOUT: Duration = Duration::from_secs(20);
const E2E_BOOTSTRAP_PRIMARY_TIMEOUT: Duration = Duration::from_secs(45);
const E2E_SCENARIO_TIMEOUT: Duration = Duration::from_secs(300);
const STRESS_ARTIFACT_DIR: &str = ".ralph/evidence/27-e2e-ha-stress";
const STRESS_SUMMARY_SCHEMA_VERSION: u32 = 1;

static E2E_UNIQUE_SEQ: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy)]
struct StablePrimaryWaitPlan<'a> {
    context: &'a str,
    timeout: Duration,
    excluded_primary: Option<&'a str>,
    required_consecutive: usize,
    fallback_timeout: Duration,
    fallback_required_consecutive: usize,
    min_observed_nodes: usize,
}

fn unique_e2e_token() -> Result<String, WorkerError> {
    let now = ha_e2e::util::unix_now()?.0;
    let seq = E2E_UNIQUE_SEQ.fetch_add(1, Ordering::Relaxed);
    Ok(format!("{now}-{seq}"))
}

fn e2e_http_timeout_ms() -> Result<u64, WorkerError> {
    u64::try_from(E2E_HTTP_STEP_TIMEOUT.as_millis())
        .map_err(|_| WorkerError::Message("e2e HTTP timeout does not fit into u64".to_string()))
}

#[derive(Clone)]
struct SqlWorkloadSpec {
    scenario_name: String,
    table_name: String,
    worker_count: usize,
    run_interval_ms: u64,
}

impl SqlWorkloadSpec {
    fn interval(&self) -> Duration {
        Duration::from_millis(self.run_interval_ms.max(1))
    }
}

#[derive(Clone)]
struct SqlWorkloadTarget {
    node_id: String,
    port: u16,
}

#[derive(Clone)]
struct SqlWorkloadCtx {
    psql_bin: PathBuf,
    superuser_username: String,
    superuser_dbname: String,
    scenario_name: String,
    table_name: String,
    interval: Duration,
    targets: Vec<SqlWorkloadTarget>,
}

struct SqlWorkloadHandle {
    spec: SqlWorkloadSpec,
    started_at_unix_ms: u64,
    stop_tx: tokio::sync::watch::Sender<bool>,
    joins: Vec<JoinHandle<Result<SqlWorkloadWorkerStats, WorkerError>>>,
}

#[derive(Default, serde::Serialize)]
struct SqlWorkloadWorkerStats {
    worker_id: usize,
    attempted_writes: u64,
    committed_writes: u64,
    attempted_reads: u64,
    read_successes: u64,
    transient_failures: u64,
    fencing_failures: u64,
    hard_failures: u64,
    commit_timestamp_capture_failures: u64,
    write_latency_total_ms: u64,
    write_latency_max_ms: u64,
    committed_keys: Vec<String>,
    committed_at_unix_ms: Vec<u64>,
    last_error: Option<String>,
}

#[derive(Default, serde::Serialize)]
struct SqlWorkloadStats {
    scenario_name: String,
    table_name: String,
    worker_count: usize,
    started_at_unix_ms: u64,
    finished_at_unix_ms: u64,
    duration_ms: u64,
    attempted_writes: u64,
    committed_writes: u64,
    attempted_reads: u64,
    read_successes: u64,
    transient_failures: u64,
    fencing_failures: u64,
    hard_failures: u64,
    commit_timestamp_capture_failures: u64,
    unique_committed_keys: usize,
    committed_keys: Vec<String>,
    committed_at_unix_ms: Vec<u64>,
    worker_stats: Vec<SqlWorkloadWorkerStats>,
    worker_errors: Vec<String>,
}

#[derive(serde::Serialize)]
struct SqlWorkloadSpecSummary {
    worker_count: usize,
    run_interval_ms: u64,
    table_name: String,
}

#[derive(serde::Serialize)]
struct StressScenarioSummary {
    schema_version: u32,
    scenario: String,
    status: String,
    started_at_unix_ms: u64,
    finished_at_unix_ms: u64,
    bootstrap_primary: Option<String>,
    final_primary: Option<String>,
    former_primary_demoted: Option<bool>,
    workload_spec: SqlWorkloadSpecSummary,
    workload: SqlWorkloadStats,
    ha_observations: HaObservationStats,
    notes: Vec<String>,
}

impl StressScenarioSummary {
    fn failed(scenario: &str, failure: String) -> Self {
        Self {
            schema_version: STRESS_SUMMARY_SCHEMA_VERSION,
            scenario: scenario.to_string(),
            status: "failed".to_string(),
            started_at_unix_ms: 0,
            finished_at_unix_ms: 0,
            bootstrap_primary: None,
            final_primary: None,
            former_primary_demoted: None,
            workload_spec: SqlWorkloadSpecSummary {
                worker_count: 0,
                run_interval_ms: 0,
                table_name: String::new(),
            },
            workload: SqlWorkloadStats::default(),
            ha_observations: HaObservationStats::default(),
            notes: vec![failure],
        }
    }
}

#[derive(Clone, Copy)]
enum SqlErrorClass {
    Transient,
    Fencing,
    Hard,
}

fn classify_sql_error(message: &str) -> SqlErrorClass {
    let normalized = message.to_ascii_lowercase();
    if normalized.contains("read-only")
        || normalized.contains("read only")
        || normalized.contains("recovery is in progress")
        || normalized.contains("cannot execute insert")
    {
        return SqlErrorClass::Fencing;
    }
    if normalized.contains("connection refused")
        || normalized.contains("could not connect")
        || normalized.contains("connection reset")
        || normalized.contains("server closed the connection")
        || normalized.contains("timed out")
        || normalized.contains("timeout")
        || normalized.contains("the database system is starting up")
        || normalized.contains("the database system is shutting down")
        || normalized.contains("no route to host")
        || normalized.contains("broken pipe")
        || normalized.contains("does not exist")
        || normalized.contains("not yet accepting connections")
    {
        return SqlErrorClass::Transient;
    }
    if normalized.contains("syntax error")
        || normalized.contains("permission denied")
        || normalized.contains("invalid input syntax")
        || normalized.contains("unterminated quoted string")
    {
        return SqlErrorClass::Hard;
    }
    SqlErrorClass::Transient
}

fn sanitize_component(raw: &str) -> String {
    let mut safe: String = raw
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect();
    if safe.is_empty() {
        safe = "unknown".to_string();
    }
    safe
}

fn sanitize_sql_identifier(raw: &str) -> String {
    let mut value: String = raw
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect();
    if value.is_empty() {
        value = "ha_stress_table".to_string();
    }
    let first_is_alpha = value
        .chars()
        .next()
        .map(|ch| ch.is_ascii_alphabetic())
        .unwrap_or(false);
    if !first_is_alpha {
        value = format!("ha_stress_{value}");
    }
    value
}

impl ClusterFixture {
    async fn start(node_count: usize) -> Result<Self, WorkerError> {
        let config = ha_e2e::TestConfig {
            test_name: "ha-e2e-multi-node".to_string(),
            cluster_name: "cluster-e2e".to_string(),
            scope: "scope-ha-e2e".to_string(),
            node_count,
            etcd_members: vec![
                "etcd-a".to_string(),
                "etcd-b".to_string(),
                "etcd-c".to_string(),
            ],
            mode: ha_e2e::Mode::Plain,
            timeouts: ha_e2e::TimeoutConfig {
                command_timeout: E2E_COMMAND_TIMEOUT,
                command_kill_wait_timeout: E2E_COMMAND_KILL_WAIT_TIMEOUT,
                http_step_timeout: E2E_HTTP_STEP_TIMEOUT,
                api_readiness_timeout: Duration::from_secs(120),
                bootstrap_primary_timeout: E2E_BOOTSTRAP_PRIMARY_TIMEOUT,
                scenario_timeout: E2E_SCENARIO_TIMEOUT,
            },
        };

        let handle = ha_e2e::start_cluster(config).await?;

        let TestClusterHandle {
            guard,
            timeouts: _,
            binaries,
            superuser_username,
            superuser_dbname,
            etcd,
            nodes,
            tasks,
            etcd_proxies: _,
            api_proxies: _,
            pg_proxies: _,
        } = handle;

        Ok(Self {
            _guard: guard,
            pg_ctl_bin: binaries.pg_ctl.clone(),
            psql_bin: binaries.psql.clone(),
            superuser_username,
            superuser_dbname,
            etcd,
            nodes,
            tasks,
            timeline: Vec::new(),
        })
    }

    fn record(&mut self, message: impl Into<String>) {
        let stamp = match ha_e2e::util::unix_now() {
            Ok(value) => value.0.to_string(),
            Err(err) => format!("time_error:{err}"),
        };
        self.timeline.push(format!("[{stamp}] {}", message.into()));
    }

    fn node_by_id(&self, id: &str) -> Option<&ha_e2e::NodeHandle> {
        self.nodes.iter().find(|node| node.id == id)
    }

    fn node_index_by_id(&self, id: &str) -> Option<usize> {
        self.nodes.iter().position(|node| node.id == id)
    }

    fn postgres_port_by_id(&self, id: &str) -> Result<u16, WorkerError> {
        let node = self.node_by_id(id).ok_or_else(|| {
            WorkerError::Message(format!("unknown node id for postgres port lookup: {id}"))
        })?;
        Ok(node.pg_port)
    }

    async fn run_sql_on_node(
        &self,
        node_id: &str,
        sql: &str,
        command_timeout: Duration,
    ) -> Result<String, WorkerError> {
        let port = self.postgres_port_by_id(node_id)?;
        ha_e2e::util::run_psql_statement(
            self.psql_bin.as_path(),
            port,
            self.superuser_username.as_str(),
            self.superuser_dbname.as_str(),
            sql,
            command_timeout,
            E2E_COMMAND_KILL_WAIT_TIMEOUT,
        )
        .await
    }

    async fn run_sql_on_node_with_retry(
        &self,
        node_id: &str,
        sql: &str,
        timeout: Duration,
    ) -> Result<String, WorkerError> {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            match self
                .run_sql_on_node(node_id, sql, E2E_COMMAND_TIMEOUT)
                .await
            {
                Ok(output) => return Ok(output),
                Err(err) => {
                    if tokio::time::Instant::now() >= deadline {
                        return Err(WorkerError::Message(format!(
                            "timed out running SQL on {node_id}; last_error={err}"
                        )));
                    }
                    tokio::time::sleep(Duration::from_millis(200)).await;
                }
            }
        }
    }

    async fn cluster_sql_roles_best_effort(
        &self,
    ) -> Result<(Vec<(String, String)>, Vec<String>), WorkerError> {
        self.cluster_sql_roles_best_effort_with_timeout(E2E_COMMAND_TIMEOUT)
            .await
    }

    async fn cluster_sql_roles_best_effort_with_timeout(
        &self,
        command_timeout: Duration,
    ) -> Result<(Vec<(String, String)>, Vec<String>), WorkerError> {
        let mut roles = Vec::new();
        let mut errors = Vec::new();

        for node in &self.nodes {
            match self
                .run_sql_on_node(
                    node.id.as_str(),
                    "SELECT CASE WHEN pg_is_in_recovery() THEN 'replica' ELSE 'primary' END",
                    command_timeout,
                )
                .await
            {
                Ok(output) => {
                    let rows = ha_e2e::util::parse_psql_rows(output.as_str());
                    let role = rows
                        .first()
                        .cloned()
                        .unwrap_or_else(|| "unknown".to_string());
                    roles.push((node.id.clone(), role));
                }
                Err(err) => {
                    errors.push(format!("node={} error={err}", node.id));
                }
            }
        }

        Ok((roles, errors))
    }

    async fn wait_for_rows_on_node(
        &self,
        node_id: &str,
        sql: &str,
        expected_rows: &[String],
        timeout: Duration,
    ) -> Result<(), WorkerError> {
        let deadline = tokio::time::Instant::now() + timeout;

        loop {
            let observation = match self
                .run_sql_on_node(node_id, sql, E2E_COMMAND_TIMEOUT)
                .await
            {
                Ok(output) => {
                    let rows = ha_e2e::util::parse_psql_rows(output.as_str());
                    if rows == expected_rows {
                        return Ok(());
                    }
                    format!("rows={rows:?}")
                }
                Err(err) => err.to_string(),
            };

            if tokio::time::Instant::now() >= deadline {
                return Err(WorkerError::Message(format!(
                    "timed out waiting for expected rows on {node_id}; expected={expected_rows:?}; last_observation={observation}"
                )));
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    }

    fn sql_workload_ctx(&self, spec: &SqlWorkloadSpec) -> Result<SqlWorkloadCtx, WorkerError> {
        if spec.worker_count == 0 {
            return Err(WorkerError::Message(
                "sql workload requires at least one worker".to_string(),
            ));
        }
        if self.nodes.is_empty() {
            return Err(WorkerError::Message(
                "sql workload cannot start: cluster has no nodes".to_string(),
            ));
        }
        let targets = self
            .nodes
            .iter()
            .map(|node| SqlWorkloadTarget {
                node_id: node.id.clone(),
                port: node.pg_port,
            })
            .collect::<Vec<_>>();
        Ok(SqlWorkloadCtx {
            psql_bin: self.psql_bin.clone(),
            superuser_username: self.superuser_username.clone(),
            superuser_dbname: self.superuser_dbname.clone(),
            scenario_name: spec.scenario_name.clone(),
            table_name: sanitize_sql_identifier(spec.table_name.as_str()),
            interval: spec.interval(),
            targets,
        })
    }

    async fn prepare_stress_table(
        &self,
        bootstrap_primary: &str,
        table_name: &str,
    ) -> Result<(), WorkerError> {
        let sql = format!(
            "CREATE TABLE IF NOT EXISTS {table_name} (worker_id INTEGER NOT NULL, seq BIGINT NOT NULL, payload TEXT NOT NULL, PRIMARY KEY (worker_id, seq))"
        );
        self.run_sql_on_node_with_retry(bootstrap_primary, sql.as_str(), Duration::from_secs(30))
            .await?;
        Ok(())
    }

    async fn start_sql_workload(
        &mut self,
        spec: SqlWorkloadSpec,
    ) -> Result<SqlWorkloadHandle, WorkerError> {
        let workload_ctx = self.sql_workload_ctx(&spec)?;
        let started_at_unix_ms = ha_e2e::util::unix_now()?.0;
        let (stop_tx, stop_rx) = tokio::sync::watch::channel(false);
        let mut joins = Vec::with_capacity(spec.worker_count);
        for worker_id in 0..spec.worker_count {
            let worker_ctx = workload_ctx.clone();
            let worker_stop_rx = stop_rx.clone();
            joins.push(tokio::spawn(async move {
                run_sql_workload_worker(worker_ctx, worker_id, worker_stop_rx).await
            }));
        }
        self.record(format!(
            "sql workload started: scenario={} table={} workers={} interval_ms={}",
            spec.scenario_name, workload_ctx.table_name, spec.worker_count, spec.run_interval_ms
        ));
        Ok(SqlWorkloadHandle {
            spec,
            started_at_unix_ms,
            stop_tx,
            joins,
        })
    }

    async fn stop_sql_workload_and_collect(
        &mut self,
        handle: SqlWorkloadHandle,
        drain: Duration,
    ) -> Result<SqlWorkloadStats, WorkerError> {
        let SqlWorkloadHandle {
            spec,
            started_at_unix_ms,
            stop_tx,
            joins,
        } = handle;
        let _ = stop_tx.send(true);
        tokio::time::sleep(drain).await;

        let mut stats = SqlWorkloadStats {
            scenario_name: spec.scenario_name.clone(),
            table_name: sanitize_sql_identifier(spec.table_name.as_str()),
            worker_count: spec.worker_count,
            started_at_unix_ms,
            ..SqlWorkloadStats::default()
        };
        let mut committed_key_set: BTreeSet<String> = BTreeSet::new();
        for join in joins {
            match join.await {
                Ok(Ok(worker)) => {
                    stats.attempted_writes = stats
                        .attempted_writes
                        .saturating_add(worker.attempted_writes);
                    stats.committed_writes = stats
                        .committed_writes
                        .saturating_add(worker.committed_writes);
                    stats.attempted_reads =
                        stats.attempted_reads.saturating_add(worker.attempted_reads);
                    stats.read_successes =
                        stats.read_successes.saturating_add(worker.read_successes);
                    stats.transient_failures = stats
                        .transient_failures
                        .saturating_add(worker.transient_failures);
                    stats.fencing_failures = stats
                        .fencing_failures
                        .saturating_add(worker.fencing_failures);
                    stats.hard_failures = stats.hard_failures.saturating_add(worker.hard_failures);
                    stats.commit_timestamp_capture_failures = stats
                        .commit_timestamp_capture_failures
                        .saturating_add(worker.commit_timestamp_capture_failures);
                    stats
                        .committed_at_unix_ms
                        .extend(worker.committed_at_unix_ms.iter().copied());
                    for key in &worker.committed_keys {
                        committed_key_set.insert(key.clone());
                    }
                    stats.worker_stats.push(worker);
                }
                Ok(Err(err)) => {
                    stats.worker_errors.push(err.to_string());
                }
                Err(err) => {
                    stats
                        .worker_errors
                        .push(format!("workload worker join failed: {err}"));
                }
            }
        }
        let worker_error_count_u64 = u64::try_from(stats.worker_errors.len()).unwrap_or(u64::MAX);
        stats.hard_failures = stats.hard_failures.saturating_add(worker_error_count_u64);
        stats.committed_keys = committed_key_set.into_iter().collect();
        stats.unique_committed_keys = stats.committed_keys.len();
        stats.finished_at_unix_ms = ha_e2e::util::unix_now()?.0;
        stats.duration_ms = stats
            .finished_at_unix_ms
            .saturating_sub(stats.started_at_unix_ms);
        self.record(format!(
            "sql workload stopped: scenario={} committed={} unique_keys={} transient={} fencing={} hard={}",
            stats.scenario_name,
            stats.committed_writes,
            stats.unique_committed_keys,
            stats.transient_failures,
            stats.fencing_failures,
            stats.hard_failures
        ));
        Ok(stats)
    }

    async fn sample_ha_states_window(
        &mut self,
        window: Duration,
        interval: Duration,
        ring_capacity: usize,
    ) -> Result<HaObservationStats, WorkerError> {
        let deadline = tokio::time::Instant::now() + window;
        let mut observer = HaInvariantObserver::new(HaObserverConfig {
            min_successful_samples: 1,
            ring_capacity,
        });
        loop {
            self.ensure_runtime_tasks_healthy().await?;
            match self
                .poll_node_ha_states_best_effort_with_timeout(Duration::from_secs(8))
                .await
            {
                Ok(polled) => {
                    let mut states = Vec::new();
                    let mut errors = Vec::new();
                    for (node_id, state_result) in polled {
                        match state_result {
                            Ok(state) => states.push(state),
                            Err(err) => {
                                errors.push(format!("node={node_id} error={err}"));
                            }
                        }
                    }

                    observer.record_api_states(&states, &errors)?;
                }
                Err(err) => {
                    observer.record_transport_error(err.to_string());
                }
            };
            if tokio::time::Instant::now() >= deadline {
                return Ok(observer.into_stats());
            }
            tokio::time::sleep(interval).await;
        }
    }

    fn count_commits_after_cutoff_strict(
        workload: &SqlWorkloadStats,
        cutoff_ms: u64,
    ) -> Result<usize, WorkerError> {
        if workload.commit_timestamp_capture_failures > 0 {
            return Err(WorkerError::Message(format!(
                "cannot evaluate fencing cutoff: commit_timestamp_capture_failures={}",
                workload.commit_timestamp_capture_failures
            )));
        }
        if workload.committed_writes == 0 {
            return Err(WorkerError::Message(
                "cannot evaluate fencing cutoff with zero committed writes".to_string(),
            ));
        }
        let committed_writes_usize = usize::try_from(workload.committed_writes).map_err(|_| {
            WorkerError::Message("committed_writes does not fit into usize".to_string())
        })?;
        if workload.committed_at_unix_ms.len() != committed_writes_usize {
            return Err(WorkerError::Message(format!(
                "cannot evaluate fencing cutoff: committed_at_unix_ms incomplete (timestamps={} committed_writes={})",
                workload.committed_at_unix_ms.len(),
                workload.committed_writes
            )));
        }
        if workload.committed_at_unix_ms.contains(&0) {
            return Err(WorkerError::Message(
                "cannot evaluate fencing cutoff: committed_at_unix_ms contains 0 timestamp"
                    .to_string(),
            ));
        }

        Ok(workload
            .committed_at_unix_ms
            .iter()
            .filter(|timestamp| **timestamp > cutoff_ms)
            .count())
    }

    async fn assert_former_primary_demoted_or_unreachable_after_transition(
        &mut self,
        former_primary: &str,
    ) -> Result<(), WorkerError> {
        let node_index = self.node_index_by_id(former_primary).ok_or_else(|| {
            WorkerError::Message(format!(
                "unknown former primary for demotion assertion: {former_primary}"
            ))
        })?;
        match self.fetch_node_ha_state_by_index(node_index).await {
            Ok(state) => {
                if state.ha_phase == "Primary" {
                    return Err(WorkerError::Message(format!(
                        "former primary {former_primary} still reports Primary phase"
                    )));
                }
                Ok(())
            }
            Err(err) => {
                self.record(format!(
                    "former primary {former_primary} API remained unreachable after transition; treating unreachable API as demotion evidence: {err}"
                ));
                Ok(())
            }
        }
    }

    async fn assert_table_key_integrity_on_node(
        &self,
        node_id: &str,
        table_name: &str,
        min_rows: u64,
        timeout: Duration,
    ) -> Result<u64, WorkerError> {
        let port = self.postgres_port_by_id(node_id)?;
        let count_sql = format!("SELECT COUNT(*)::bigint FROM {table_name}");
        let duplicate_sql = format!(
            "SELECT COUNT(*)::bigint FROM (SELECT worker_id, seq, COUNT(*) AS c FROM {table_name} GROUP BY worker_id, seq HAVING COUNT(*) > 1) d"
        );
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            let count_raw = match ha_e2e::util::run_psql_statement(
                self.psql_bin.as_path(),
                port,
                self.superuser_username.as_str(),
                self.superuser_dbname.as_str(),
                count_sql.as_str(),
                E2E_SQL_WORKLOAD_COMMAND_TIMEOUT,
                E2E_SQL_WORKLOAD_COMMAND_KILL_WAIT_TIMEOUT,
            )
            .await
            {
                Ok(value) => value,
                Err(err) => {
                    let detail = format!("row count query failed: {err}");
                    if tokio::time::Instant::now() >= deadline {
                        return Err(WorkerError::Message(format!(
                            "timed out verifying table integrity on {node_id}; last_observation={detail}"
                        )));
                    }
                    tokio::time::sleep(Duration::from_millis(200)).await;
                    continue;
                }
            };
            let duplicate_raw = match ha_e2e::util::run_psql_statement(
                self.psql_bin.as_path(),
                port,
                self.superuser_username.as_str(),
                self.superuser_dbname.as_str(),
                duplicate_sql.as_str(),
                E2E_SQL_WORKLOAD_COMMAND_TIMEOUT,
                E2E_SQL_WORKLOAD_COMMAND_KILL_WAIT_TIMEOUT,
            )
            .await
            {
                Ok(value) => value,
                Err(err) => {
                    let detail = format!("duplicate query failed: {err}");
                    if tokio::time::Instant::now() >= deadline {
                        return Err(WorkerError::Message(format!(
                            "timed out verifying table integrity on {node_id}; last_observation={detail}"
                        )));
                    }
                    tokio::time::sleep(Duration::from_millis(200)).await;
                    continue;
                }
            };
            let row_count = ha_e2e::util::parse_single_u64(count_raw.as_str())?;
            let duplicate_count = ha_e2e::util::parse_single_u64(duplicate_raw.as_str())?;
            if duplicate_count > 0 {
                return Err(WorkerError::Message(format!(
                    "duplicate (worker_id,seq) rows detected on {node_id}: {duplicate_count}"
                )));
            }
            if row_count >= min_rows {
                return Ok(row_count);
            }
            let detail = format!("row_count={row_count} below min_rows={min_rows}");
            if tokio::time::Instant::now() >= deadline {
                return Err(WorkerError::Message(format!(
                    "timed out verifying table integrity on {node_id}; last_observation={detail}"
                )));
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    }

    async fn assert_table_key_integrity_strict(
        &mut self,
        preferred_node_id: &str,
        table_name: &str,
        min_rows: u64,
        per_node_timeout: Duration,
    ) -> Result<(String, u64), WorkerError> {
        let mut node_ids = Vec::new();
        if self.node_by_id(preferred_node_id).is_some() {
            node_ids.push(preferred_node_id.to_string());
        }
        for node in &self.nodes {
            if node.id != preferred_node_id {
                node_ids.push(node.id.clone());
            }
        }

        if node_ids.is_empty() {
            return Err(WorkerError::Message(format!(
                "cannot verify table integrity: no nodes available for {table_name}"
            )));
        }

        let mut errors = Vec::new();
        for node_id in node_ids {
            match self
                .assert_table_key_integrity_on_node(
                    node_id.as_str(),
                    table_name,
                    min_rows,
                    per_node_timeout,
                )
                .await
            {
                Ok(row_count) => return Ok((node_id, row_count)),
                Err(err) => {
                    let message = err.to_string();
                    // Duplicate rows / empty table are hard failures when a node is reachable enough
                    // to answer queries (this indicates a real integrity problem).
                    if message.contains("duplicate (worker_id,seq) rows detected")
                        || message.contains("below min_rows")
                    {
                        return Err(err);
                    }
                    errors.push(format!("{node_id}: {message}"));
                }
            }
        }

        Err(WorkerError::Message(format!(
            "table integrity could not be verified on any node for {table_name}; errors={errors:?}"
        )))
    }

    fn assert_no_split_brain_write_evidence(
        workload: &SqlWorkloadStats,
        _ha_stats: &HaObservationStats,
    ) -> Result<(), WorkerError> {
        if workload.unique_committed_keys
            != usize::try_from(workload.committed_writes).unwrap_or(usize::MAX)
        {
            return Err(WorkerError::Message(format!(
                "duplicate committed write keys detected: committed_writes={} unique_keys={}",
                workload.committed_writes, workload.unique_committed_keys
            )));
        }
        if workload.hard_failures > 0 {
            return Err(WorkerError::Message(format!(
                "hard SQL failures detected during stress workload: hard_failures={} worker_errors={:?}",
                workload.hard_failures, workload.worker_errors
            )));
        }
        Ok(())
    }

    fn update_phase_history(
        phase_history: &mut BTreeMap<String, BTreeSet<String>>,
        states: &[HaStateResponse],
    ) {
        for state in states {
            phase_history
                .entry(state.self_member_id.clone())
                .or_default()
                .insert(state.ha_phase.to_string());
        }
    }

    fn format_phase_history(phase_history: &BTreeMap<String, BTreeSet<String>>) -> String {
        let mut node_entries = Vec::with_capacity(phase_history.len());
        for (node_id, phases) in phase_history {
            let phase_list: Vec<&str> = phases.iter().map(String::as_str).collect();
            node_entries.push(format!("{node_id}:{}", phase_list.join("|")));
        }
        node_entries.join(",")
    }

    async fn wait_for_stable_primary(
        &mut self,
        timeout: Duration,
        excluded_primary: Option<&str>,
        required_consecutive: usize,
        phase_history: &mut BTreeMap<String, BTreeSet<String>>,
    ) -> Result<String, WorkerError> {
        if required_consecutive == 0 {
            return Err(WorkerError::Message(
                "required_consecutive must be greater than zero".to_string(),
            ));
        }

        let deadline = tokio::time::Instant::now() + timeout;
        let mut last_error = "none".to_string();
        let mut last_candidate: Option<String> = None;
        let mut last_state_summary: Option<String> = None;
        let mut stable_count = 0usize;

        loop {
            match self.cluster_ha_states().await {
                Ok(states) => {
                    Self::update_phase_history(phase_history, states.as_slice());
                    let state_summary = states
                        .iter()
                        .map(|state| {
                            let leader = state.leader.as_deref().unwrap_or("none");
                            format!(
                                "{}:{}:leader={}",
                                state.self_member_id, state.ha_phase, leader
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    if last_state_summary
                        .as_deref()
                        .map(|prior| prior != state_summary.as_str())
                        .unwrap_or(true)
                    {
                        self.record(format!("stable-primary poll states: {state_summary}"));
                        last_state_summary = Some(state_summary);
                    }
                    let primaries = Self::primary_members(states.as_slice());
                    if primaries.len() == 1 {
                        if let Some(primary) = primaries.into_iter().next() {
                            let excluded = excluded_primary
                                .map(|excluded_id| excluded_id == primary)
                                .unwrap_or(false);
                            if !excluded {
                                if last_candidate.as_deref() == Some(primary.as_str()) {
                                    stable_count = stable_count.saturating_add(1);
                                } else {
                                    stable_count = 1;
                                    last_candidate = Some(primary.clone());
                                }
                                if stable_count >= required_consecutive {
                                    return Ok(primary);
                                }
                            } else {
                                stable_count = 0;
                                last_candidate = None;
                            }
                        }
                    } else {
                        stable_count = 0;
                        last_candidate = None;
                    }
                }
                Err(err) => {
                    stable_count = 0;
                    last_candidate = None;
                    last_error = err.to_string();
                }
            }

            if tokio::time::Instant::now() >= deadline {
                return Err(WorkerError::Message(format!(
                    "timed out waiting for stable primary via API; last_error={last_error}"
                )));
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    async fn wait_for_stable_primary_best_effort(
        &mut self,
        timeout: Duration,
        excluded_primary: Option<&str>,
        required_consecutive: usize,
        min_observed_nodes: usize,
        phase_history: &mut BTreeMap<String, BTreeSet<String>>,
    ) -> Result<String, WorkerError> {
        if required_consecutive == 0 {
            return Err(WorkerError::Message(
                "required_consecutive must be greater than zero".to_string(),
            ));
        }
        if min_observed_nodes == 0 {
            return Err(WorkerError::Message(
                "min_observed_nodes must be greater than zero".to_string(),
            ));
        }

        let deadline = tokio::time::Instant::now() + timeout;
        let mut last_error = "none".to_string();
        let mut last_candidate: Option<String> = None;
        let mut last_state_summary: Option<String> = None;
        let mut stable_count = 0usize;

        loop {
            self.ensure_runtime_tasks_healthy().await?;
            match self.poll_node_ha_states_best_effort().await {
                Ok(polled) => {
                    let mut states = Vec::new();
                    let mut fragments = Vec::with_capacity(polled.len());

                    for (node_id, state_result) in polled {
                        match state_result {
                            Ok(state) => {
                                let leader = state.leader.as_deref().unwrap_or("none");
                                fragments.push(format!(
                                    "{}:{}:leader={leader}",
                                    state.self_member_id, state.ha_phase
                                ));
                                states.push(state);
                            }
                            Err(err) => {
                                fragments.push(format!("{node_id}:error={err}"));
                                last_error = format!("HA state poll failed for {node_id}: {err}");
                            }
                        }
                    }

                    let state_summary = fragments.join(", ");
                    if last_state_summary
                        .as_deref()
                        .map(|prior| prior != state_summary.as_str())
                        .unwrap_or(true)
                    {
                        self.record(format!(
                            "stable-primary best-effort poll states: {state_summary}"
                        ));
                        last_state_summary = Some(state_summary);
                    }

                    if states.len() < min_observed_nodes {
                        stable_count = 0;
                        last_candidate = None;
                        last_error = format!(
                            "insufficient observed HA states: observed={} required={min_observed_nodes}",
                            states.len()
                        );
                    } else {
                        Self::update_phase_history(phase_history, states.as_slice());
                        let primaries = Self::primary_members(states.as_slice());
                        if primaries.len() == 1 {
                            if let Some(primary) = primaries.into_iter().next() {
                                let excluded = excluded_primary
                                    .map(|excluded_id| excluded_id == primary)
                                    .unwrap_or(false);
                                if !excluded {
                                    if last_candidate.as_deref() == Some(primary.as_str()) {
                                        stable_count = stable_count.saturating_add(1);
                                    } else {
                                        stable_count = 1;
                                        last_candidate = Some(primary.clone());
                                    }
                                    if stable_count >= required_consecutive {
                                        return Ok(primary);
                                    }
                                } else {
                                    stable_count = 0;
                                    last_candidate = None;
                                }
                            }
                        } else {
                            stable_count = 0;
                            last_candidate = None;
                        }
                    }
                }
                Err(err) => {
                    stable_count = 0;
                    last_candidate = None;
                    last_error = err.to_string();
                }
            }

            if tokio::time::Instant::now() >= deadline {
                return Err(WorkerError::Message(format!(
                    "timed out waiting for stable primary via best-effort API polling; last_error={last_error}"
                )));
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    fn assert_phase_history_contains_failover(
        phase_history: &BTreeMap<String, BTreeSet<String>>,
        former_primary: &str,
        new_primary: &str,
    ) -> Result<(), WorkerError> {
        const PRIMARY_PHASE: &str = "primary";

        let former_phases = phase_history.get(former_primary).ok_or_else(|| {
            WorkerError::Message(format!(
                "missing phase history for former primary {former_primary}"
            ))
        })?;
        if !former_phases.contains(PRIMARY_PHASE) {
            return Err(WorkerError::Message(format!(
                "former primary {former_primary} never observed in Primary phase"
            )));
        }
        if !former_phases.iter().any(|phase| phase != PRIMARY_PHASE) {
            return Err(WorkerError::Message(format!(
                "former primary {former_primary} never observed leaving Primary phase"
            )));
        }

        let promoted_phases = phase_history.get(new_primary).ok_or_else(|| {
            WorkerError::Message(format!(
                "missing phase history for promoted primary {new_primary}"
            ))
        })?;
        if !promoted_phases.contains(PRIMARY_PHASE) {
            return Err(WorkerError::Message(format!(
                "new primary {new_primary} never observed in Primary phase"
            )));
        }

        Ok(())
    }

    fn node_api_base_url_by_index(
        &self,
        node_index: usize,
    ) -> Result<(String, String), WorkerError> {
        let node = self.nodes.get(node_index).ok_or_else(|| {
            WorkerError::Message(format!("invalid node index for API request: {node_index}"))
        })?;
        Ok((node.id.clone(), format!("http://{}", node.api_observe_addr)))
    }

    fn cli_api_client_for_node_index(
        &self,
        node_index: usize,
    ) -> Result<(String, CliApiClient), WorkerError> {
        let (node_id, base_url) = self.node_api_base_url_by_index(node_index)?;
        let timeout_ms = e2e_http_timeout_ms()?;
        let client = CliApiClient::new(base_url, timeout_ms, None, None)
            .map_err(|err| WorkerError::Message(format!("build CliApiClient failed: {err}")))?;
        Ok((node_id, client))
    }

    async fn request_switchover_via_cli(&mut self, requested_by: &str) -> Result<(), WorkerError> {
        if self.nodes.is_empty() {
            return Err(WorkerError::Message(
                "no nodes available for API control".to_string(),
            ));
        }

        let timeout_ms = e2e_http_timeout_ms()?;

        // Any node API can write the switchover intent. Iterate across all node APIs because the
        // former primary can be transiently unavailable while replicas are still healthy enough to
        // accept the operator request.
        let max_transport_rounds: usize = 5;
        let mut last_transport_error = "transport error".to_string();
        let mut output: Option<String> = None;

        for round in 1..=max_transport_rounds {
            for node_index in 0..self.nodes.len() {
                let (node_id, base_url) = self.node_api_base_url_by_index(node_index)?;
                self.record(format!(
                    "cli request start: round={round}/{max_transport_rounds} node={node_id} ha switchover request requested_by={requested_by}"
                ));
                let argv: Vec<String> = vec![
                    "pgtuskmasterctl".to_string(),
                    "--base-url".to_string(),
                    base_url,
                    "--timeout-ms".to_string(),
                    timeout_ms.to_string(),
                    "--output".to_string(),
                    "json".to_string(),
                    "ha".to_string(),
                    "switchover".to_string(),
                    "request".to_string(),
                    "--requested-by".to_string(),
                    requested_by.to_string(),
                ];
                let cli = Cli::try_parse_from(argv).map_err(|err| {
                    WorkerError::Message(format!("parse switchover CLI args failed: {err}"))
                })?;
                match cli::run(cli).await {
                    Ok(out) => {
                        self.record(format!(
                            "cli request success: round={round}/{max_transport_rounds} node={node_id} ha switchover request accepted=true requested_by={requested_by}"
                        ));
                        output = Some(out);
                        break;
                    }
                    Err(err) => match err {
                        CliError::Transport(_) => {
                            let err_string = err.to_string();
                            last_transport_error =
                                format!("node={node_id} round={round} err={err_string}");
                            self.record(format!(
                                "cli request transport failure: round={round}/{max_transport_rounds} node={node_id} requested_by={requested_by} err={err_string}"
                            ));
                        }
                        _ => {
                            return Err(WorkerError::Message(format!(
                                "run switchover CLI command failed via {node_id}: {err}"
                            )));
                        }
                    },
                }
            }

            if output.is_some() {
                break;
            }

            if round < max_transport_rounds {
                let backoff_ms = 200_u64.saturating_mul(round as u64);
                tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
            }
        }

        let output = match output {
            Some(out) => out,
            None => {
                return Err(WorkerError::Message(format!(
                    "run switchover CLI command failed after {max_transport_rounds} round(s) across {} node(s): {last_transport_error}",
                    self.nodes.len()
                )));
            }
        };

        let accepted =
            serde_json::from_str::<CliAcceptedResponse>(output.as_str()).map_err(|err| {
                WorkerError::Message(format!(
                    "decode switchover CLI response failed: {err}; output={}",
                    output.trim()
                ))
            })?;
        if !accepted.accepted {
            return Err(WorkerError::Message(
                "switchover CLI response returned accepted=false".to_string(),
            ));
        }
        Ok(())
    }

    async fn request_switchover_until_stable_primary_changes(
        &mut self,
        previous_primary: &str,
        requested_by: &str,
        max_attempts: usize,
        per_attempt_timeout: Duration,
        required_consecutive: usize,
        phase_history: &mut BTreeMap<String, BTreeSet<String>>,
    ) -> Result<String, WorkerError> {
        if max_attempts == 0 {
            return Err(WorkerError::Message(
                "switchover attempts must be greater than zero".to_string(),
            ));
        }
        if required_consecutive == 0 {
            return Err(WorkerError::Message(
                "required_consecutive must be greater than zero".to_string(),
            ));
        }

        let mut last_error = "none".to_string();
        for attempt in 1..=max_attempts {
            self.request_switchover_via_cli(requested_by).await?;
            match self
                .wait_for_stable_primary_best_effort(
                    per_attempt_timeout,
                    Some(previous_primary),
                    required_consecutive,
                    1,
                    phase_history,
                )
                .await
            {
                Ok(primary) => return Ok(primary),
                Err(err) => {
                    let stable_wait_error = err.to_string();
                    self.record(format!(
                        "switchover attempt {attempt}/{max_attempts} stable-primary wait failed after accepted request: {stable_wait_error}; retrying with relaxed primary-change detection"
                    ));
                    match self
                        .wait_for_primary_change_best_effort(
                            per_attempt_timeout,
                            previous_primary,
                            1,
                            phase_history,
                        )
                        .await
                    {
                        Ok(primary) => return Ok(primary),
                        Err(change_err) => {
                            last_error = format!(
                                "{stable_wait_error}; fallback primary-change detection failed: {change_err}"
                            );
                            self.record(format!(
                                "switchover attempt {attempt}/{max_attempts} did not change primary from {previous_primary}: {last_error}"
                            ));
                        }
                    }
                }
            }
            if attempt < max_attempts {
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }

        Err(WorkerError::Message(format!(
            "switchover did not change primary from {previous_primary} after {max_attempts} attempt(s); last_error={last_error}"
        )))
    }

    // /ha/state polling is the canonical post-start observation path.
    async fn fetch_node_ha_state_by_index(
        &mut self,
        node_index: usize,
    ) -> Result<HaStateResponse, WorkerError> {
        let node_addr = self
            .nodes
            .get(node_index)
            .ok_or_else(|| {
                WorkerError::Message(format!(
                    "invalid node index for HA state fetch: {node_index}"
                ))
            })?
            .api_observe_addr;
        let (node_id, client) = self.cli_api_client_for_node_index(node_index)?;
        ha_e2e::util::get_ha_state_with_fallback(
            &client,
            node_id.as_str(),
            node_addr,
            E2E_HTTP_STEP_TIMEOUT,
        )
        .await
    }

    async fn poll_node_ha_states_best_effort(
        &self,
    ) -> Result<Vec<(String, Result<HaStateResponse, WorkerError>)>, WorkerError> {
        self.poll_node_ha_states_best_effort_with_timeout(E2E_HTTP_STEP_TIMEOUT)
            .await
    }

    async fn poll_node_ha_states_best_effort_with_timeout(
        &self,
        http_step_timeout: Duration,
    ) -> Result<Vec<(String, Result<HaStateResponse, WorkerError>)>, WorkerError> {
        let node_count = self.nodes.len();
        let mut joins = Vec::with_capacity(node_count);

        for node_index in 0..node_count {
            let node = self.nodes.get(node_index).ok_or_else(|| {
                WorkerError::Message(format!(
                    "invalid node index for HA state poll: {node_index}"
                ))
            })?;
            let (node_id, client) = self.cli_api_client_for_node_index(node_index)?;
            let node_addr = node.api_addr;
            joins.push(tokio::task::spawn_local(async move {
                let result = ha_e2e::util::get_ha_state_with_fallback(
                    &client,
                    node_id.as_str(),
                    node_addr,
                    http_step_timeout,
                )
                .await;
                (node_id, result)
            }));
        }

        let mut results = Vec::with_capacity(node_count);
        for join in joins {
            let joined = join
                .await
                .map_err(|err| WorkerError::Message(format!("HA state poll join failed: {err}")))?;
            results.push(joined);
        }

        Ok(results)
    }

    async fn cluster_ha_states(&mut self) -> Result<Vec<HaStateResponse>, WorkerError> {
        self.ensure_runtime_tasks_healthy().await?;
        let polled = self.poll_node_ha_states_best_effort().await?;
        let mut states = Vec::with_capacity(polled.len());
        for (node_id, result) in polled {
            let state = result.map_err(|err| {
                WorkerError::Message(format!("HA state poll failed for {node_id}: {err}"))
            })?;
            states.push(state);
        }
        Ok(states)
    }

    async fn ensure_runtime_tasks_healthy(&mut self) -> Result<(), WorkerError> {
        let mut index = 0usize;
        while index < self.tasks.len() {
            if !self.tasks[index].is_finished() {
                index = index.saturating_add(1);
                continue;
            }

            let node_id = self
                .nodes
                .get(index)
                .map(|node| node.id.clone())
                .unwrap_or_else(|| format!("index-{index}"));
            let task = self.tasks.swap_remove(index);
            let joined = task
                .await
                .map_err(|err| WorkerError::Message(format!("runtime task join failed: {err}")))?;
            match joined {
                Ok(()) => {
                    return Err(WorkerError::Message(format!(
                        "runtime task for {node_id} exited unexpectedly"
                    )));
                }
                Err(err) => {
                    return Err(WorkerError::Message(format!(
                        "runtime task for {node_id} failed: {err}"
                    )));
                }
            }
        }
        Ok(())
    }

    fn primary_members(states: &[HaStateResponse]) -> Vec<String> {
        states
            .iter()
            .filter(|state| state.ha_phase == "Primary")
            .map(|state| state.self_member_id.clone())
            .collect()
    }

    async fn wait_for_primary_change(
        &mut self,
        previous: &str,
        timeout: Duration,
    ) -> Result<String, WorkerError> {
        let deadline = tokio::time::Instant::now() + timeout;
        let mut last_error: Option<String> = None;
        loop {
            match self.cluster_ha_states().await {
                Ok(states) => {
                    let primaries = Self::primary_members(&states);
                    if primaries.len() == 1 {
                        if let Some(primary) = primaries.into_iter().next() {
                            if primary != previous {
                                return Ok(primary);
                            }
                        }
                    }
                }
                Err(err) => {
                    last_error = Some(err.to_string());
                }
            }
            if tokio::time::Instant::now() >= deadline {
                let detail = last_error
                    .as_deref()
                    .map_or_else(|| "none".to_string(), ToString::to_string);
                return Err(WorkerError::Message(format!(
                    "timed out waiting for primary change from {previous} via API; last_error={detail}"
                )));
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    async fn wait_for_primary_change_best_effort(
        &mut self,
        timeout: Duration,
        previous: &str,
        min_observed_nodes: usize,
        phase_history: &mut BTreeMap<String, BTreeSet<String>>,
    ) -> Result<String, WorkerError> {
        if min_observed_nodes == 0 {
            return Err(WorkerError::Message(
                "min_observed_nodes must be greater than zero".to_string(),
            ));
        }

        let deadline = tokio::time::Instant::now() + timeout;
        let mut last_error = "none".to_string();
        let mut last_state_summary: Option<String> = None;

        loop {
            self.ensure_runtime_tasks_healthy().await?;
            match self.poll_node_ha_states_best_effort().await {
                Ok(polled) => {
                    let mut states = Vec::new();
                    let mut fragments = Vec::with_capacity(polled.len());

                    for (node_id, state_result) in polled {
                        match state_result {
                            Ok(state) => {
                                let leader = state.leader.as_deref().unwrap_or("none");
                                fragments.push(format!(
                                    "{}:{}:leader={leader}",
                                    state.self_member_id, state.ha_phase
                                ));
                                states.push(state);
                            }
                            Err(err) => {
                                fragments.push(format!("{node_id}:error={err}"));
                                last_error = format!("HA state poll failed for {node_id}: {err}");
                            }
                        }
                    }

                    let state_summary = fragments.join(", ");
                    if last_state_summary
                        .as_deref()
                        .map(|prior| prior != state_summary.as_str())
                        .unwrap_or(true)
                    {
                        self.record(format!(
                            "primary-change best-effort poll states: {state_summary}"
                        ));
                        last_state_summary = Some(state_summary);
                    }

                    if states.len() < min_observed_nodes {
                        last_error = format!(
                            "insufficient observed HA states: observed={} required={min_observed_nodes}",
                            states.len()
                        );
                    } else {
                        Self::update_phase_history(phase_history, states.as_slice());
                        let primaries = Self::primary_members(states.as_slice());
                        if primaries.len() == 1 {
                            if let Some(primary) = primaries.into_iter().next() {
                                if primary != previous {
                                    return Ok(primary);
                                }
                            }
                        }
                    }
                }
                Err(err) => {
                    last_error = err.to_string();
                }
            }

            if tokio::time::Instant::now() >= deadline {
                return Err(WorkerError::Message(format!(
                    "timed out waiting for primary change from {previous} via best-effort API polling; last_error={last_error}"
                )));
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    async fn wait_for_stable_primary_via_sql(
        &mut self,
        timeout: Duration,
        excluded_primary: Option<&str>,
        required_consecutive: usize,
        min_observed_nodes: usize,
    ) -> Result<String, WorkerError> {
        if required_consecutive == 0 {
            return Err(WorkerError::Message(
                "required_consecutive must be greater than zero".to_string(),
            ));
        }
        if min_observed_nodes == 0 {
            return Err(WorkerError::Message(
                "min_observed_nodes must be greater than zero".to_string(),
            ));
        }

        let deadline = tokio::time::Instant::now() + timeout;
        let mut stable_count = 0usize;
        let mut last_candidate: Option<String> = None;
        let mut last_observation = "none".to_string();

        loop {
            if tokio::time::Instant::now() >= deadline {
                return Err(WorkerError::Message(format!(
                    "timed out waiting for stable primary via SQL; excluded={excluded_primary:?}; last_observation={last_observation}"
                )));
            }

            let (sql_roles, sql_errors) = self.cluster_sql_roles_best_effort().await?;
            let observed_nodes = sql_roles.len();
            let primary_nodes = sql_roles
                .iter()
                .filter(|(_, role)| role == "primary")
                .map(|(node_id, _)| node_id.clone())
                .collect::<Vec<_>>();
            let role_fragments = sql_roles
                .iter()
                .map(|(node_id, role)| format!("{node_id}:{role}"))
                .collect::<Vec<_>>();
            let error_fragments = sql_errors
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .join(" | ");

            last_observation = format!(
                "observed_nodes={observed_nodes} roles=[{}] errors={error_fragments}",
                role_fragments.join(", ")
            );

            if observed_nodes < min_observed_nodes {
                stable_count = 0;
                last_candidate = None;
            } else if primary_nodes.len() == 1 {
                let candidate = primary_nodes[0].clone();
                let excluded = excluded_primary
                    .map(|excluded_id| excluded_id == candidate)
                    .unwrap_or(false);
                if !excluded {
                    if last_candidate.as_deref() == Some(candidate.as_str()) {
                        stable_count = stable_count.saturating_add(1);
                    } else {
                        stable_count = 1;
                        last_candidate = Some(candidate.clone());
                    }
                    if stable_count >= required_consecutive {
                        self.record(format!(
                            "stable-primary SQL poll converged: {}",
                            role_fragments.join(", ")
                        ));
                        return Ok(candidate);
                    }
                } else {
                    stable_count = 0;
                    last_candidate = None;
                }
            } else {
                stable_count = 0;
                last_candidate = None;
            }

            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    }

    async fn wait_for_stable_primary_resilient(
        &mut self,
        plan: StablePrimaryWaitPlan<'_>,
        phase_history: &mut BTreeMap<String, BTreeSet<String>>,
    ) -> Result<String, WorkerError> {
        if plan.required_consecutive == 0 {
            return Err(WorkerError::Message(
                "required_consecutive must be greater than zero".to_string(),
            ));
        }
        if plan.fallback_required_consecutive == 0 {
            return Err(WorkerError::Message(
                "fallback_required_consecutive must be greater than zero".to_string(),
            ));
        }
        if plan.min_observed_nodes == 0 {
            return Err(WorkerError::Message(
                "min_observed_nodes must be greater than zero".to_string(),
            ));
        }

        let strict_timeout = std::cmp::min(plan.timeout, Duration::from_secs(45));
        let api_fallback_timeout = std::cmp::min(plan.fallback_timeout, Duration::from_secs(45));
        let sql_fallback_timeout = std::cmp::min(plan.fallback_timeout, Duration::from_secs(90));
        let strict_required_consecutive = plan.required_consecutive.min(3);
        let relaxed_required_consecutive = plan.fallback_required_consecutive.min(2);

        match self
            .wait_for_stable_primary(
                strict_timeout,
                plan.excluded_primary,
                strict_required_consecutive,
                phase_history,
            )
            .await
        {
            Ok(primary) => Ok(primary),
            Err(wait_err) => {
                self.record(format!(
                    "{}: strict stable-primary wait failed: {wait_err}; retrying with best-effort API polling",
                    plan.context
                ));
                match self
                    .wait_for_stable_primary_best_effort(
                        api_fallback_timeout,
                        plan.excluded_primary,
                        relaxed_required_consecutive,
                        plan.min_observed_nodes,
                        phase_history,
                    )
                    .await
                {
                    Ok(primary) => Ok(primary),
                    Err(best_effort_err) => {
                        self.record(format!(
                            "{}: best-effort API stable-primary wait failed: {best_effort_err}; retrying with SQL role polling",
                            plan.context
                        ));
                        self.wait_for_stable_primary_via_sql(
                            sql_fallback_timeout,
                            plan.excluded_primary,
                            relaxed_required_consecutive,
                            plan.min_observed_nodes,
                        )
                        .await
                    }
                }
            }
        }
    }

    async fn assert_no_dual_primary_window(&mut self, window: Duration) -> Result<(), WorkerError> {
        let deadline = tokio::time::Instant::now() + window;
        let mut observer = HaInvariantObserver::new(HaObserverConfig {
            min_successful_samples: 1,
            ring_capacity: 16,
        });
        loop {
            observer.record_poll_attempt();
            self.ensure_runtime_tasks_healthy().await?;
            match self.poll_node_ha_states_best_effort().await {
                Ok(polled) => {
                    let mut states = Vec::new();
                    let mut errors = Vec::new();
                    for (node_id, result) in polled {
                        match result {
                            Ok(state) => states.push(state),
                            Err(err) => errors.push(format!("node={node_id} error={err}")),
                        }
                    }

                    if states.is_empty() {
                        let (sql_roles, sql_errors) = self.cluster_sql_roles_best_effort().await?;
                        if sql_roles.is_empty() {
                            observer.record_observation_gap(&errors, &sql_errors);
                        } else {
                            observer.record_sql_roles(&sql_roles, &sql_errors)?;
                        }
                    } else {
                        observer.record_api_states(&states, &errors)?;
                    }
                }
                Err(err) => {
                    observer.record_transport_error(err.to_string());
                }
            }
            if tokio::time::Instant::now() >= deadline {
                return observer.finalize_no_dual_primary_window();
            }
            tokio::time::sleep(Duration::from_millis(75)).await;
        }
    }

    async fn wait_for_all_nodes_failsafe(&mut self, timeout: Duration) -> Result<(), WorkerError> {
        let deadline = tokio::time::Instant::now() + timeout;
        let mut observed_api_failsafe_nodes: BTreeSet<String> = BTreeSet::new();
        let mut observed_api_non_primary_nodes: BTreeSet<String> = BTreeSet::new();
        let mut last_observation: Option<String> = None;
        let mut last_recorded_at = tokio::time::Instant::now();
        let node_count = self.nodes.len();
        if node_count == 0 {
            return Err(WorkerError::Message(
                "cannot wait for fail-safe with zero nodes".to_string(),
            ));
        }

        loop {
            if tokio::time::Instant::now() >= deadline {
                let detail = last_observation
                    .as_deref()
                    .map_or_else(|| "none".to_string(), ToString::to_string);
                return Err(WorkerError::Message(format!(
                    "timed out waiting for no-quorum fail-safe API observability (all nodes must answer /ha/state, at least one node must report FailSafe, and no node may report Primary); last_observation={detail}"
                )));
            }
            self.ensure_runtime_tasks_healthy().await?;
            let mut poll_details = Vec::new();
            let polled = match self
                .poll_node_ha_states_best_effort_with_timeout(Duration::from_secs(3))
                .await
            {
                Ok(values) => values,
                Err(err) => {
                    last_observation = Some(format!("poll:error={err}"));
                    if last_recorded_at.elapsed() >= Duration::from_secs(5) {
                        self.record(format!("no-quorum wait poll: poll:error={err}"));
                        last_recorded_at = tokio::time::Instant::now();
                    }
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    continue;
                }
            };
            let (sql_roles, sql_errors) = self
                .cluster_sql_roles_best_effort_with_timeout(Duration::from_secs(3))
                .await?;
            let mut api_success_count = 0usize;
            let mut current_api_failsafe_nodes: BTreeSet<String> = BTreeSet::new();
            let mut current_api_non_primary_nodes: BTreeSet<String> = BTreeSet::new();
            let mut current_api_primary_nodes: BTreeSet<String> = BTreeSet::new();

            for (node_id, state_result) in polled {
                match state_result {
                    Ok(state) => {
                        api_success_count = api_success_count.saturating_add(1);
                        if state.ha_phase == "Primary" {
                            current_api_primary_nodes.insert(node_id.clone());
                        } else {
                            current_api_non_primary_nodes.insert(node_id.clone());
                            observed_api_non_primary_nodes.insert(node_id.clone());
                        }
                        if state.ha_phase == "FailSafe" {
                            current_api_failsafe_nodes.insert(node_id.clone());
                            observed_api_failsafe_nodes.insert(node_id.clone());
                        }
                        poll_details.push(format!(
                            "{node_id}:phase={} leader={:?}",
                            state.ha_phase, state.leader
                        ));
                    }
                    Err(err) => {
                        poll_details.push(format!("{node_id}:error={err}"));
                    }
                }
            }

            if api_success_count == node_count
                && current_api_primary_nodes.is_empty()
                && !current_api_failsafe_nodes.is_empty()
                && current_api_non_primary_nodes.len() == node_count
            {
                return Ok(());
            }

            last_observation = Some(format!(
                "api_success_count={api_success_count}/{node_count}; current_api_failsafe_nodes={:?}; current_api_non_primary_nodes={:?}; current_api_primary_nodes={:?}; observed_api_failsafe_nodes={:?}; observed_api_non_primary_nodes={:?}; poll={}",
                current_api_failsafe_nodes,
                current_api_non_primary_nodes,
                current_api_primary_nodes,
                observed_api_failsafe_nodes,
                observed_api_non_primary_nodes,
                poll_details.join(" | ")
            ));
            if !sql_roles.is_empty() {
                last_observation = Some(format!(
                    "{}; sql_roles={}",
                    last_observation.as_deref().unwrap_or("none"),
                    sql_roles
                        .iter()
                        .map(|(node_id, role)| format!("{node_id}:{role}"))
                        .collect::<Vec<_>>()
                        .join(" | ")
                ));
            }
            if !sql_errors.is_empty() {
                last_observation = Some(format!(
                    "{}; sql_errors={}",
                    last_observation.as_deref().unwrap_or("none"),
                    sql_errors.join(" | ")
                ));
            }
            if last_recorded_at.elapsed() >= Duration::from_secs(5) {
                if let Some(observation) = last_observation.as_deref() {
                    self.record(format!("no-quorum wait poll: {observation}"));
                }
                last_recorded_at = tokio::time::Instant::now();
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    // Process/network failures are allowed external stimuli for HA behavior validation.
    async fn stop_postgres_for_node(&self, node_id: &str) -> Result<(), WorkerError> {
        let Some(node) = self.node_by_id(node_id) else {
            return Err(WorkerError::Message(format!(
                "unknown node for stop request: {node_id}"
            )));
        };
        ha_e2e::util::pg_ctl_stop_immediate(
            &self.pg_ctl_bin,
            &node.data_dir,
            E2E_COMMAND_TIMEOUT,
            E2E_COMMAND_KILL_WAIT_TIMEOUT,
        )
        .await
    }

    // This fixture-level etcd shutdown models external quorum loss; it is not direct DCS key steering.
    async fn stop_etcd_majority(&mut self, stop_count: usize) -> Result<Vec<String>, WorkerError> {
        let Some(etcd_cluster) = self.etcd.as_mut() else {
            return Err(WorkerError::Message(
                "cannot stop etcd majority: cluster is not running".to_string(),
            ));
        };

        let member_names = etcd_cluster.member_names();
        if member_names.len() < stop_count {
            return Err(WorkerError::Message(format!(
                "cannot stop etcd majority: requested {stop_count}, available {}",
                member_names.len()
            )));
        }

        let mut stopped = Vec::with_capacity(stop_count);
        for member_name in member_names.into_iter().take(stop_count) {
            let stopped_member =
                etcd_cluster
                    .shutdown_member(&member_name)
                    .await
                    .map_err(|err| {
                        WorkerError::Message(format!(
                            "failed to stop etcd member {member_name}: {err}"
                        ))
                    })?;
            if !stopped_member {
                return Err(WorkerError::Message(format!(
                    "etcd member {member_name} was not found for shutdown"
                )));
            }
            stopped.push(member_name);
        }

        Ok(stopped)
    }

    fn write_timeline_artifact(&self, scenario: &str) -> Result<PathBuf, WorkerError> {
        let artifact_dir =
            Path::new(env!("CARGO_MANIFEST_DIR")).join(".ralph/evidence/13-e2e-multi-node");
        fs::create_dir_all(&artifact_dir)
            .map_err(|err| WorkerError::Message(format!("create artifact dir failed: {err}")))?;
        let stamp = ha_e2e::util::unix_now()?.0;
        let safe_scenario = sanitize_component(scenario);
        let artifact_path = artifact_dir.join(format!("{safe_scenario}-{stamp}.timeline.log"));
        fs::write(&artifact_path, self.timeline.join("\n"))
            .map_err(|err| WorkerError::Message(format!("write timeline failed: {err}")))?;
        Ok(artifact_path)
    }

    fn write_stress_artifacts(
        &self,
        scenario: &str,
        summary: &StressScenarioSummary,
    ) -> Result<(PathBuf, PathBuf), WorkerError> {
        let artifact_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(STRESS_ARTIFACT_DIR);
        fs::create_dir_all(&artifact_dir).map_err(|err| {
            WorkerError::Message(format!("create stress artifact dir failed: {err}"))
        })?;
        let stamp = ha_e2e::util::unix_now()?.0;
        let safe_scenario = sanitize_component(scenario);
        let timeline_path = artifact_dir.join(format!("{safe_scenario}-{stamp}.timeline.log"));
        fs::write(&timeline_path, self.timeline.join("\n")).map_err(|err| {
            WorkerError::Message(format!("write stress timeline artifact failed: {err}"))
        })?;
        let summary_path = artifact_dir.join(format!("{safe_scenario}-{stamp}.summary.json"));
        let summary_json = serde_json::to_string_pretty(summary)
            .map_err(|err| WorkerError::Message(format!("encode stress summary failed: {err}")))?;
        fs::write(&summary_path, summary_json)
            .map_err(|err| WorkerError::Message(format!("write stress summary failed: {err}")))?;
        Ok((timeline_path, summary_path))
    }

    async fn shutdown(&mut self) -> Result<(), WorkerError> {
        for task in &self.tasks {
            task.abort();
        }
        while let Some(task) = self.tasks.pop() {
            let _ = task.await;
        }

        let mut pg_stops = Vec::with_capacity(self.nodes.len());
        for node in &self.nodes {
            let pg_ctl_bin = self.pg_ctl_bin.clone();
            let data_dir = node.data_dir.clone();
            pg_stops.push(tokio::task::spawn_local(async move {
                let _ = ha_e2e::util::pg_ctl_stop_immediate(
                    &pg_ctl_bin,
                    &data_dir,
                    E2E_PG_STOP_TIMEOUT,
                    E2E_COMMAND_KILL_WAIT_TIMEOUT,
                )
                .await;
            }));
        }
        for stop in pg_stops {
            let _ = stop.await;
        }

        if let Some(etcd) = self.etcd.as_mut() {
            etcd.shutdown_all()
                .await
                .map_err(|err| WorkerError::Message(format!("etcd shutdown failed: {err}")))?;
        }
        self.etcd = None;
        Ok(())
    }
}

async fn run_sql_workload_worker(
    workload: SqlWorkloadCtx,
    worker_id: usize,
    mut stop_rx: tokio::sync::watch::Receiver<bool>,
) -> Result<SqlWorkloadWorkerStats, WorkerError> {
    if workload.targets.is_empty() {
        return Err(WorkerError::Message(
            "sql workload worker cannot run without targets".to_string(),
        ));
    }
    let mut stats = SqlWorkloadWorkerStats {
        worker_id,
        ..SqlWorkloadWorkerStats::default()
    };
    let mut seq = 0u64;
    let mut target_index = worker_id % workload.targets.len();
    loop {
        if *stop_rx.borrow() {
            break;
        }
        let target = workload.targets.get(target_index).ok_or_else(|| {
            WorkerError::Message(format!(
                "sql workload target index out of bounds: index={} len={}",
                target_index,
                workload.targets.len()
            ))
        })?;
        target_index = (target_index + 1) % workload.targets.len();

        let payload = format!("{}-{worker_id}-{seq}", workload.scenario_name);
        let write_sql = format!(
            "INSERT INTO {} (worker_id, seq, payload) VALUES ({worker_id}, {seq}, '{}') ON CONFLICT (worker_id, seq) DO UPDATE SET payload = EXCLUDED.payload",
            workload.table_name, payload
        );
        stats.attempted_writes = stats.attempted_writes.saturating_add(1);
        let write_started = tokio::time::Instant::now();
        match ha_e2e::util::run_psql_statement(
            workload.psql_bin.as_path(),
            target.port,
            workload.superuser_username.as_str(),
            workload.superuser_dbname.as_str(),
            write_sql.as_str(),
            E2E_SQL_WORKLOAD_COMMAND_TIMEOUT,
            E2E_SQL_WORKLOAD_COMMAND_KILL_WAIT_TIMEOUT,
        )
        .await
        {
            Ok(_) => {
                stats.committed_writes = stats.committed_writes.saturating_add(1);
                stats.committed_keys.push(format!("{worker_id}:{seq}"));
                match ha_e2e::util::unix_now() {
                    Ok(value) => {
                        stats.committed_at_unix_ms.push(value.0);
                    }
                    Err(err) => {
                        stats.commit_timestamp_capture_failures =
                            stats.commit_timestamp_capture_failures.saturating_add(1);
                        stats.hard_failures = stats.hard_failures.saturating_add(1);
                        stats.last_error = Some(format!(
                            "target={} write seq={seq} committed but timestamp capture failed: {err}",
                            target.node_id
                        ));
                    }
                }
            }
            Err(err) => {
                let err_text = err.to_string();
                match classify_sql_error(err_text.as_str()) {
                    SqlErrorClass::Transient => {
                        stats.transient_failures = stats.transient_failures.saturating_add(1);
                    }
                    SqlErrorClass::Fencing => {
                        stats.fencing_failures = stats.fencing_failures.saturating_add(1);
                    }
                    SqlErrorClass::Hard => {
                        stats.hard_failures = stats.hard_failures.saturating_add(1);
                    }
                }
                stats.last_error = Some(format!(
                    "target={} write seq={seq} error={err_text}",
                    target.node_id
                ));
            }
        }
        let latency_ms = u64::try_from(write_started.elapsed().as_millis()).unwrap_or(u64::MAX);
        stats.write_latency_total_ms = stats.write_latency_total_ms.saturating_add(latency_ms);
        stats.write_latency_max_ms = stats.write_latency_max_ms.max(latency_ms);

        let read_sql = format!("SELECT COUNT(*)::bigint FROM {}", workload.table_name);
        stats.attempted_reads = stats.attempted_reads.saturating_add(1);
        match ha_e2e::util::run_psql_statement(
            workload.psql_bin.as_path(),
            target.port,
            workload.superuser_username.as_str(),
            workload.superuser_dbname.as_str(),
            read_sql.as_str(),
            E2E_SQL_WORKLOAD_COMMAND_TIMEOUT,
            E2E_SQL_WORKLOAD_COMMAND_KILL_WAIT_TIMEOUT,
        )
        .await
        {
            Ok(_) => {
                stats.read_successes = stats.read_successes.saturating_add(1);
            }
            Err(err) => {
                let err_text = err.to_string();
                match classify_sql_error(err_text.as_str()) {
                    SqlErrorClass::Transient => {
                        stats.transient_failures = stats.transient_failures.saturating_add(1);
                    }
                    SqlErrorClass::Fencing => {
                        stats.fencing_failures = stats.fencing_failures.saturating_add(1);
                    }
                    SqlErrorClass::Hard => {
                        stats.hard_failures = stats.hard_failures.saturating_add(1);
                    }
                }
                stats.last_error = Some(format!(
                    "target={} read seq={seq} error={err_text}",
                    target.node_id
                ));
            }
        }

        seq = seq.saturating_add(1);
        tokio::select! {
            changed = stop_rx.changed() => {
                match changed {
                    Ok(()) => {
                        if *stop_rx.borrow() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
            _ = tokio::time::sleep(workload.interval) => {}
        }
    }
    Ok(stats)
}

fn finalize_stress_scenario_result(
    run_error: Option<String>,
    artifacts: Result<(PathBuf, PathBuf), WorkerError>,
    shutdown_result: Result<(), WorkerError>,
) -> Result<(), WorkerError> {
    match (run_error, artifacts, shutdown_result) {
        (None, Ok(_), Ok(())) => Ok(()),
        (Some(run_err), Ok((timeline, summary)), Ok(())) => Err(WorkerError::Message(format!(
            "{run_err}; timeline: {}; summary: {}",
            timeline.display(),
            summary.display()
        ))),
        (Some(run_err), Err(artifact_err), Ok(())) => Err(WorkerError::Message(format!(
            "{run_err}; stress artifact write failed: {artifact_err}"
        ))),
        (None, Ok((timeline, summary)), Err(shutdown_err)) => Err(WorkerError::Message(format!(
            "shutdown failed: {shutdown_err}; timeline: {}; summary: {}",
            timeline.display(),
            summary.display()
        ))),
        (None, Err(artifact_err), Err(shutdown_err)) => Err(WorkerError::Message(format!(
            "stress artifact write failed: {artifact_err}; shutdown failed: {shutdown_err}"
        ))),
        (Some(run_err), Ok((timeline, summary)), Err(shutdown_err)) => Err(WorkerError::Message(
            format!(
                "{run_err}; shutdown failed: {shutdown_err}; timeline: {}; summary: {}",
                timeline.display(),
                summary.display()
            ),
        )),
        (Some(run_err), Err(artifact_err), Err(shutdown_err)) => Err(WorkerError::Message(
            format!(
                "{run_err}; stress artifact write failed: {artifact_err}; shutdown failed: {shutdown_err}"
            ),
        )),
        (None, Err(artifact_err), Ok(())) => Err(WorkerError::Message(format!(
            "stress artifact write failed: {artifact_err}"
        ))),
    }
}

async fn stop_etcd_majority_and_wait_failsafe_strict_all_nodes(
    fixture: &mut ClusterFixture,
    stop_count: usize,
    timeout: Duration,
) -> Result<u64, WorkerError> {
    fixture.record("no-quorum: stop etcd majority");
    let stopped_members = fixture.stop_etcd_majority(stop_count).await?;
    fixture.record(format!(
        "no-quorum: etcd members stopped: {}",
        stopped_members.join(",")
    ));

    fixture.wait_for_all_nodes_failsafe(timeout).await?;
    fixture.record("no-quorum: fail-safe observed on all nodes");
    Ok(ha_e2e::util::unix_now()?.0)
}

pub async fn e2e_multi_node_unassisted_failover_sql_consistency() -> Result<(), WorkerError> {
    ha_e2e::util::run_with_local_set(async {
    let mut fixture = ClusterFixture::start(3).await?;
    let mut phase_history: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    let run_result = match tokio::time::timeout(E2E_SCENARIO_TIMEOUT, async {
        fixture.record("unassisted failover bootstrap: wait for stable primary");
        let bootstrap_primary = fixture
            .wait_for_stable_primary_resilient(
                StablePrimaryWaitPlan {
                    context: "unassisted failover bootstrap stable-primary",
                    timeout: Duration::from_secs(60),
                    excluded_primary: None,
                    required_consecutive: 5,
                    fallback_timeout: Duration::from_secs(90),
                    fallback_required_consecutive: 2,
                    min_observed_nodes: 2,
                },
                &mut phase_history,
            )
            .await?;
        fixture.record(format!(
            "unassisted failover bootstrap success: primary={bootstrap_primary}"
        ));
        fixture
            .assert_no_dual_primary_window(Duration::from_secs(3))
            .await?;

        fixture.record("unassisted failover SQL pre-check: create table and insert pre-failure row");
        fixture
            .run_sql_on_node_with_retry(
                &bootstrap_primary,
                "CREATE TABLE IF NOT EXISTS ha_unassisted_failover_proof (id INTEGER PRIMARY KEY, payload TEXT NOT NULL)",
                Duration::from_secs(20),
            )
            .await?;
        fixture
            .run_sql_on_node_with_retry(
                &bootstrap_primary,
                "INSERT INTO ha_unassisted_failover_proof (id, payload) VALUES (1, 'before') ON CONFLICT (id) DO UPDATE SET payload = EXCLUDED.payload",
                Duration::from_secs(20),
            )
            .await?;
        let pre_rows_raw = fixture
            .run_sql_on_node_with_retry(
                &bootstrap_primary,
                "SELECT id::text || ':' || payload FROM ha_unassisted_failover_proof ORDER BY id",
                Duration::from_secs(20),
            )
            .await?;
        let pre_rows = ha_e2e::util::parse_psql_rows(pre_rows_raw.as_str());
        let expected_pre_rows = vec!["1:before".to_string()];
        if pre_rows != expected_pre_rows {
            return Err(WorkerError::Message(format!(
                "pre-failure SQL rows mismatch: expected {:?}, got {:?}",
                expected_pre_rows, pre_rows
            )));
        }
        let replica_ids: Vec<String> = fixture
            .nodes
            .iter()
            .filter(|node| node.id != bootstrap_primary)
            .map(|node| node.id.clone())
            .collect();
        for replica_id in replica_ids {
            fixture
                .wait_for_rows_on_node(
                    &replica_id,
                    "SELECT id::text || ':' || payload FROM ha_unassisted_failover_proof ORDER BY id",
                    expected_pre_rows.as_slice(),
                    Duration::from_secs(20),
                )
                .await?;
            fixture.record(format!(
                "unassisted failover SQL pre-check seeded/validated on replica={replica_id}"
            ));
        }
        fixture.record("unassisted failover SQL pre-check succeeded");

        fixture.record(format!(
            "unassisted failover failure injection: stop postgres on {bootstrap_primary}"
        ));
        fixture.stop_postgres_for_node(&bootstrap_primary).await?;

        fixture.record(
            "unassisted failover recovery: best-effort API-only polling for new stable primary",
        );
        let failover_primary = match fixture
            .wait_for_stable_primary_best_effort(
                Duration::from_secs(120),
                Some(&bootstrap_primary),
                3,
                1,
                &mut phase_history,
            )
            .await
        {
            Ok(primary) => primary,
            Err(wait_err) => {
                fixture.record(format!(
                    "unassisted failover stable-primary wait failed after forced stop: {wait_err}; retrying with relaxed primary-change detection"
                ));
                fixture
                    .wait_for_primary_change(&bootstrap_primary, Duration::from_secs(90))
                    .await?
            }
        };
        fixture
            .assert_no_dual_primary_window(Duration::from_secs(10))
            .await?;
        fixture.record(
            "unassisted failover recovery: confirm SQL-visible primary after API recovery",
        );
        let sql_confirmed_primary = fixture
            .wait_for_stable_primary_via_sql(
                Duration::from_secs(60),
                Some(&bootstrap_primary),
                2,
                1,
            )
            .await?;
        if sql_confirmed_primary != failover_primary {
            fixture.record(format!(
                "unassisted failover SQL confirmation chose primary={sql_confirmed_primary} after API-selected primary={failover_primary}"
            ));
        }
        if let Ok(polled) = fixture
            .poll_node_ha_states_best_effort_with_timeout(Duration::from_secs(3))
            .await
        {
            let states = polled
                .into_iter()
                .filter_map(|(_, result)| result.ok())
                .collect::<Vec<_>>();
            ClusterFixture::update_phase_history(&mut phase_history, states.as_slice());
        }
        let failover_primary = sql_confirmed_primary;
        ClusterFixture::assert_phase_history_contains_failover(
            &phase_history,
            &bootstrap_primary,
            &failover_primary,
        )?;
        fixture.record(format!(
            "unassisted failover recovery success: former_primary={bootstrap_primary}, new_primary={failover_primary}"
        ));
        fixture.record(format!(
            "phase history evidence: {}",
            ClusterFixture::format_phase_history(&phase_history)
        ));

        fixture.record("unassisted failover SQL post-check: insert post-failure row");
        fixture
            .run_sql_on_node_with_retry(
                &failover_primary,
                "INSERT INTO ha_unassisted_failover_proof (id, payload) VALUES (2, 'after') ON CONFLICT (id) DO UPDATE SET payload = EXCLUDED.payload",
                Duration::from_secs(45),
            )
            .await?;
        let post_rows_raw = fixture
            .run_sql_on_node_with_retry(
                &failover_primary,
                "SELECT id::text || ':' || payload FROM ha_unassisted_failover_proof ORDER BY id",
                Duration::from_secs(45),
            )
            .await?;
        let post_rows = ha_e2e::util::parse_psql_rows(post_rows_raw.as_str());
        let expected_post_rows = vec!["1:before".to_string(), "2:after".to_string()];
        if post_rows != expected_post_rows {
            return Err(WorkerError::Message(format!(
                "post-failure SQL rows mismatch: expected {:?}, got {:?}",
                expected_post_rows, post_rows
            )));
        }
        fixture.record("unassisted failover SQL continuity proof succeeded");
        Ok(())
    })
    .await
    {
        Ok(run_result) => run_result,
        Err(_) => {
            fixture.record(format!(
                "unassisted failover scenario timed out after {}s",
                E2E_SCENARIO_TIMEOUT.as_secs()
            ));
            Err(WorkerError::Message(format!(
                "unassisted failover scenario timed out after {}s",
                E2E_SCENARIO_TIMEOUT.as_secs()
            )))
        }
    };

    let artifact_path =
        fixture.write_timeline_artifact("ha-e2e-unassisted-failover-sql-consistency");
    let shutdown_result = fixture.shutdown().await;

    match (run_result, artifact_path, shutdown_result) {
        (Ok(()), Ok(_), Ok(())) => Ok(()),
        (Err(run_err), Ok(path), Ok(())) => Err(WorkerError::Message(format!(
            "{run_err}; timeline: {}",
            path.display()
        ))),
        (Err(run_err), Err(artifact_err), Ok(())) => Err(WorkerError::Message(format!(
            "{run_err}; timeline write failed: {artifact_err}"
        ))),
        (Ok(()), Ok(path), Err(shutdown_err)) => Err(WorkerError::Message(format!(
            "shutdown failed: {shutdown_err}; timeline: {}",
            path.display()
        ))),
        (Ok(()), Err(artifact_err), Err(shutdown_err)) => Err(WorkerError::Message(format!(
            "timeline write failed: {artifact_err}; shutdown failed: {shutdown_err}"
        ))),
        (Err(run_err), Ok(path), Err(shutdown_err)) => Err(WorkerError::Message(format!(
            "{run_err}; shutdown failed: {shutdown_err}; timeline: {}",
            path.display()
        ))),
        (Err(run_err), Err(artifact_err), Err(shutdown_err)) => Err(WorkerError::Message(format!(
            "{run_err}; timeline write failed: {artifact_err}; shutdown failed: {shutdown_err}"
        ))),
        (Ok(()), Err(artifact_err), Ok(())) => Err(WorkerError::Message(format!(
            "timeline write failed: {artifact_err}"
        ))),
    }
    })
    .await
}

pub async fn e2e_multi_node_stress_planned_switchover_concurrent_sql() -> Result<(), WorkerError> {
    ha_e2e::util::run_with_local_set(async {
    let mut fixture = ClusterFixture::start(3).await?;
    let scenario_name = "ha-e2e-stress-planned-switchover-concurrent-sql".to_string();

    let run_result = match tokio::time::timeout(E2E_SCENARIO_TIMEOUT, async {
        let started_at_unix_ms = ha_e2e::util::unix_now()?.0;
        let mut phase_history: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let workload_spec = SqlWorkloadSpec {
            scenario_name: scenario_name.clone(),
            table_name: "ha_stress_switchover".to_string(),
            worker_count: 4,
            run_interval_ms: 250,
        };
        let table_name = sanitize_sql_identifier(workload_spec.table_name.as_str());

        fixture.record("stress switchover bootstrap: wait for stable primary");
        let bootstrap_primary = fixture
            .wait_for_stable_primary_resilient(
                StablePrimaryWaitPlan {
                    context: "stress switchover bootstrap stable-primary",
                    timeout: Duration::from_secs(90),
                    excluded_primary: None,
                    required_consecutive: 3,
                    fallback_timeout: Duration::from_secs(90),
                    fallback_required_consecutive: 2,
                    min_observed_nodes: 2,
                },
                &mut phase_history,
            )
            .await?;
        fixture
            .prepare_stress_table(&bootstrap_primary, table_name.as_str())
            .await?;
        let workload_handle = fixture.start_sql_workload(workload_spec.clone()).await?;
        tokio::time::sleep(Duration::from_secs(3)).await;

        fixture.record("stress switchover: trigger API switchover while workload is active");
        fixture
            .request_switchover_via_cli("e2e-stress-switchover")
            .await?;
        let ha_stats = fixture
            .sample_ha_states_window(Duration::from_secs(8), Duration::from_millis(150), 80)
            .await?;
        let workload = fixture
            .stop_sql_workload_and_collect(workload_handle, Duration::from_secs(2))
            .await?;
        if workload.committed_writes == 0 {
            return Err(WorkerError::Message(
                "stress switchover workload committed zero writes".to_string(),
            ));
        }
        ClusterFixture::assert_no_split_brain_write_evidence(&workload, &ha_stats)?;
        let switchover_primary = match fixture
            .wait_for_stable_primary_resilient(
                StablePrimaryWaitPlan {
                    context: "stress switchover primary convergence",
                    // Keep enough global scenario budget for an explicit second switchover
                    // request when the first accepted intent does not move leadership.
                    timeout: Duration::from_secs(25),
                    excluded_primary: Some(&bootstrap_primary),
                    required_consecutive: 2,
                    fallback_timeout: Duration::from_secs(35),
                    fallback_required_consecutive: 1,
                    min_observed_nodes: 1,
                },
                &mut phase_history,
            )
            .await
        {
            Ok(primary) => primary,
            Err(wait_err) => {
                fixture.record(format!(
                    "stress switchover stable-primary wait failed after first request: {wait_err}; retrying switchover request"
                ));
                fixture
                    .request_switchover_until_stable_primary_changes(
                        &bootstrap_primary,
                        "e2e-stress-switchover-retry",
                        2,
                        Duration::from_secs(35),
                        1,
                        &mut phase_history,
                    )
                    .await?
            }
        };
        fixture
            .assert_former_primary_demoted_or_unreachable_after_transition(&bootstrap_primary)
            .await?;
        fixture
            .assert_no_dual_primary_window(Duration::from_secs(10))
            .await?;
        fixture
            .prepare_stress_table(&switchover_primary, table_name.as_str())
            .await?;
        fixture
            .run_sql_on_node_with_retry(
                &switchover_primary,
                format!(
                    "INSERT INTO {table_name} (worker_id, seq, payload) VALUES (9999, 1, 'post-switchover-proof') ON CONFLICT (worker_id, seq) DO UPDATE SET payload = EXCLUDED.payload"
                )
                .as_str(),
                Duration::from_secs(30),
            )
            .await?;

        let primary_row_count = fixture
            .assert_table_key_integrity_on_node(
                &switchover_primary,
                table_name.as_str(),
                1,
                Duration::from_secs(90),
            )
            .await?;

        fixture.record(format!(
            "stress switchover key integrity verified on {switchover_primary} with row_count={primary_row_count}"
        ));
        let finished_at_unix_ms = ha_e2e::util::unix_now()?.0;
        Ok(StressScenarioSummary {
            schema_version: STRESS_SUMMARY_SCHEMA_VERSION,
            scenario: scenario_name.clone(),
            status: "passed".to_string(),
            started_at_unix_ms,
            finished_at_unix_ms,
            bootstrap_primary: Some(bootstrap_primary.clone()),
            final_primary: Some(switchover_primary.clone()),
            former_primary_demoted: Some(true),
            workload_spec: SqlWorkloadSpecSummary {
                worker_count: workload_spec.worker_count,
                run_interval_ms: workload_spec.run_interval_ms,
                table_name,
            },
            workload,
            ha_observations: ha_stats,
            notes: vec![
                format!(
                    "phase_history={}",
                    ClusterFixture::format_phase_history(&phase_history)
                ),
                format!(
                    "primary_transition={}=>{}",
                    bootstrap_primary, switchover_primary
                ),
            ],
        })
    })
    .await
    {
        Ok(run_result) => run_result,
        Err(_) => Err(WorkerError::Message(format!(
            "stress switchover scenario timed out after {}s",
            E2E_SCENARIO_TIMEOUT.as_secs()
        ))),
    };

    let (summary, run_error) = match run_result {
        Ok(summary) => (summary, None),
        Err(err) => {
            let message = err.to_string();
            (
                StressScenarioSummary::failed(scenario_name.as_str(), message.clone()),
                Some(message),
            )
        }
    };
    let artifacts = fixture.write_stress_artifacts(scenario_name.as_str(), &summary);
    let shutdown_result = fixture.shutdown().await;
    finalize_stress_scenario_result(run_error, artifacts, shutdown_result)
    })
    .await
}

pub async fn e2e_multi_node_stress_unassisted_failover_concurrent_sql() -> Result<(), WorkerError> {
    ha_e2e::util::run_with_local_set(async {
    let mut fixture = ClusterFixture::start(3).await?;
    let scenario_name = "ha-e2e-stress-unassisted-failover-concurrent-sql".to_string();

    let run_result = match tokio::time::timeout(E2E_SCENARIO_TIMEOUT, async {
        let started_at_unix_ms = ha_e2e::util::unix_now()?.0;
        let mut phase_history: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let workload_spec = SqlWorkloadSpec {
            scenario_name: scenario_name.clone(),
            table_name: "ha_stress_failover".to_string(),
            worker_count: 4,
            run_interval_ms: 250,
        };
        let table_name = sanitize_sql_identifier(workload_spec.table_name.as_str());

        fixture.record("stress failover bootstrap: wait for stable primary");
        let bootstrap_primary = fixture
            .wait_for_stable_primary_resilient(
                StablePrimaryWaitPlan {
                    context: "stress failover bootstrap stable-primary",
                    timeout: Duration::from_secs(60),
                    excluded_primary: None,
                    required_consecutive: 5,
                    fallback_timeout: Duration::from_secs(90),
                    fallback_required_consecutive: 2,
                    min_observed_nodes: 2,
                },
                &mut phase_history,
            )
            .await?;
        fixture
            .prepare_stress_table(&bootstrap_primary, table_name.as_str())
            .await?;
        let workload_handle = fixture.start_sql_workload(workload_spec.clone()).await?;
        tokio::time::sleep(Duration::from_secs(3)).await;

        fixture.record(format!(
            "stress failover: stop postgres on bootstrap primary {bootstrap_primary}"
        ));
        fixture.stop_postgres_for_node(&bootstrap_primary).await?;
        let ha_stats = fixture
            .sample_ha_states_window(Duration::from_secs(10), Duration::from_millis(150), 100)
            .await?;
        let workload = fixture
            .stop_sql_workload_and_collect(workload_handle, Duration::from_secs(2))
            .await?;
        if workload.committed_writes == 0 {
            return Err(WorkerError::Message(
                "stress failover workload committed zero writes".to_string(),
            ));
        }
        ClusterFixture::assert_no_split_brain_write_evidence(&workload, &ha_stats)?;
        let failover_primary = match fixture
            .wait_for_stable_primary(
                Duration::from_secs(180),
                Some(&bootstrap_primary),
                3,
                &mut phase_history,
            )
            .await
        {
            Ok(primary) => primary,
            Err(wait_err) => {
                fixture.record(format!(
                    "stress failover stable-primary wait failed under load: {wait_err}; retrying with relaxed single-sample promotion detection"
                ));
                fixture
                    .wait_for_primary_change(&bootstrap_primary, Duration::from_secs(90))
                    .await?
            }
        };
        ClusterFixture::assert_phase_history_contains_failover(
            &phase_history,
            &bootstrap_primary,
            &failover_primary,
        )?;
        fixture
            .assert_former_primary_demoted_or_unreachable_after_transition(&bootstrap_primary)
            .await?;
        fixture
            .assert_no_dual_primary_window(Duration::from_secs(6))
            .await?;
        fixture
            .prepare_stress_table(&failover_primary, table_name.as_str())
            .await?;
        fixture
            .run_sql_on_node_with_retry(
                &failover_primary,
                format!(
                    "INSERT INTO {table_name} (worker_id, seq, payload) VALUES (9999, 2, 'post-failover-proof') ON CONFLICT (worker_id, seq) DO UPDATE SET payload = EXCLUDED.payload"
                )
                .as_str(),
                Duration::from_secs(30),
            )
            .await?;

        let primary_row_count = fixture
            .assert_table_key_integrity_on_node(
                &failover_primary,
                table_name.as_str(),
                1,
                Duration::from_secs(90),
            )
            .await?;
        fixture.record(format!(
            "stress failover key integrity verified on {failover_primary} with row_count={primary_row_count}"
        ));

        let finished_at_unix_ms = ha_e2e::util::unix_now()?.0;
        Ok(StressScenarioSummary {
            schema_version: STRESS_SUMMARY_SCHEMA_VERSION,
            scenario: scenario_name.clone(),
            status: "passed".to_string(),
            started_at_unix_ms,
            finished_at_unix_ms,
            bootstrap_primary: Some(bootstrap_primary.clone()),
            final_primary: Some(failover_primary.clone()),
            former_primary_demoted: Some(true),
            workload_spec: SqlWorkloadSpecSummary {
                worker_count: workload_spec.worker_count,
                run_interval_ms: workload_spec.run_interval_ms,
                table_name,
            },
            workload,
            ha_observations: ha_stats,
            notes: vec![
                format!(
                    "phase_history={}",
                    ClusterFixture::format_phase_history(&phase_history)
                ),
                format!(
                    "primary_transition={}=>{}",
                    bootstrap_primary, failover_primary
                ),
            ],
        })
    })
    .await
    {
        Ok(run_result) => run_result,
        Err(_) => Err(WorkerError::Message(format!(
            "stress failover scenario timed out after {}s",
            E2E_SCENARIO_TIMEOUT.as_secs()
        ))),
    };

    let (summary, run_error) = match run_result {
        Ok(summary) => (summary, None),
        Err(err) => {
            let message = err.to_string();
            (
                StressScenarioSummary::failed(scenario_name.as_str(), message.clone()),
                Some(message),
            )
        }
    };
    let artifacts = fixture.write_stress_artifacts(scenario_name.as_str(), &summary);
    let shutdown_result = fixture.shutdown().await;
    finalize_stress_scenario_result(run_error, artifacts, shutdown_result)
    })
    .await
}

pub async fn e2e_no_quorum_enters_failsafe_strict_all_nodes() -> Result<(), WorkerError> {
    ha_e2e::util::run_with_local_set(async {
    let mut fixture = ClusterFixture::start(3).await?;
    let token = unique_e2e_token()?;
    let scenario_name = format!("ha-e2e-no-quorum-enters-failsafe-strict-all-nodes-{token}");

    let run_result = (async {
        let started_at_unix_ms = ha_e2e::util::unix_now()?.0;
        let mut phase_history: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

        fixture.record("no-quorum: wait for stable primary");
        let bootstrap_primary = fixture
            .wait_for_stable_primary_resilient(
                StablePrimaryWaitPlan {
                    context: "no-quorum bootstrap stable-primary",
                    timeout: Duration::from_secs(60),
                    excluded_primary: None,
                    required_consecutive: 5,
                    fallback_timeout: Duration::from_secs(90),
                    fallback_required_consecutive: 2,
                    min_observed_nodes: 2,
                },
                &mut phase_history,
            )
            .await?;
        let failsafe_observed_at_ms = stop_etcd_majority_and_wait_failsafe_strict_all_nodes(
            &mut fixture,
            2,
            Duration::from_secs(60),
        )
        .await?;
        fixture.ensure_runtime_tasks_healthy().await?;
        let polled = fixture
            .poll_node_ha_states_best_effort_with_timeout(Duration::from_secs(8))
            .await?;
        let mut observed = Vec::new();
        let mut observed_primary = false;
        for (node_id, state_result) in polled {
            match state_result {
                Ok(state) => {
                    if state.ha_phase == "Primary" {
                        observed_primary = true;
                    }
                    observed.push(format!("{node_id}:{}", state.ha_phase));
                }
                Err(err) => {
                    fixture.record(format!("no-quorum: best-effort ha poll error for {node_id}: {err}"));
                }
            }
        }
        if observed_primary {
            return Err(WorkerError::Message(format!(
                "expected no Primary phase after quorum loss in best-effort poll; observed={observed:?}"
            )));
        }
        let ha_stats = fixture
            .sample_ha_states_window(Duration::from_secs(4), Duration::from_millis(150), 60)
            .await?;
        assert_no_dual_primary_in_samples(&ha_stats, 1)?;

        let finished_at_unix_ms = ha_e2e::util::unix_now()?.0;
        Ok(StressScenarioSummary {
            schema_version: STRESS_SUMMARY_SCHEMA_VERSION,
            scenario: scenario_name.to_string(),
            status: "passed".to_string(),
            started_at_unix_ms,
            finished_at_unix_ms,
            bootstrap_primary: Some(bootstrap_primary),
            final_primary: None,
            former_primary_demoted: None,
            workload_spec: SqlWorkloadSpecSummary {
                worker_count: 0,
                run_interval_ms: 0,
                table_name: String::new(),
            },
            workload: SqlWorkloadStats::default(),
            ha_observations: ha_stats,
            notes: vec![
                format!(
                    "phase_history={}",
                    ClusterFixture::format_phase_history(&phase_history)
                ),
                format!("failsafe_observed_at_ms={failsafe_observed_at_ms}"),
            ],
        })
    })
    .await;

    let (summary, run_error) = match run_result {
        Ok(summary) => (summary, None),
        Err(err) => {
            let message = err.to_string();
            (
                StressScenarioSummary::failed(scenario_name.as_str(), message.clone()),
                Some(message),
            )
        }
    };
    let artifacts = fixture.write_stress_artifacts(scenario_name.as_str(), &summary);
    let shutdown_result = fixture.shutdown().await;
    finalize_stress_scenario_result(run_error, artifacts, shutdown_result)
    })
    .await
}

pub async fn e2e_no_quorum_fencing_blocks_post_cutoff_commits_and_preserves_integrity(
) -> Result<(), WorkerError> {
    ha_e2e::util::run_with_local_set(async {
    let mut fixture = ClusterFixture::start(3).await?;
    let token = unique_e2e_token()?;
    let scenario_name =
        format!("ha-e2e-no-quorum-fencing-blocks-post-cutoff-commits-{token}");

    let run_result = (async {
        let started_at_unix_ms = ha_e2e::util::unix_now()?.0;
        let mut phase_history: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let workload_spec = SqlWorkloadSpec {
            scenario_name: scenario_name.to_string(),
            table_name: format!("ha_no_quorum_fencing_{token}"),
            worker_count: 4,
            run_interval_ms: 250,
        };
        let table_name = sanitize_sql_identifier(workload_spec.table_name.as_str());

        fixture.record("no-quorum fencing: wait for stable primary");
        let bootstrap_primary = fixture
            .wait_for_stable_primary_resilient(
                StablePrimaryWaitPlan {
                    context: "no-quorum fencing bootstrap stable-primary",
                    timeout: Duration::from_secs(60),
                    excluded_primary: None,
                    required_consecutive: 5,
                    fallback_timeout: Duration::from_secs(90),
                    fallback_required_consecutive: 2,
                    min_observed_nodes: 2,
                },
                &mut phase_history,
            )
            .await?;
        fixture
            .prepare_stress_table(&bootstrap_primary, table_name.as_str())
            .await?;
        let workload_handle = fixture.start_sql_workload(workload_spec.clone()).await?;
        tokio::time::sleep(Duration::from_secs(2)).await;

        fixture.record("no-quorum fencing: stop etcd majority while workload active");
        fixture.record("no-quorum: stop etcd majority");
        let stopped_members = fixture.stop_etcd_majority(2).await?;
        fixture.record(format!(
            "no-quorum: etcd members stopped: {}",
            stopped_members.join(",")
        ));
        let quorum_lost_at_ms = ha_e2e::util::unix_now()?.0;
        let ha_stats = fixture
            .sample_ha_states_window(Duration::from_secs(2), Duration::from_millis(150), 80)
            .await?;

        let fencing_grace_ms = 7_000u64;
        tokio::time::sleep(Duration::from_secs(8)).await;
        let workload = fixture
            .stop_sql_workload_and_collect(workload_handle, Duration::from_millis(200))
            .await?;
        if workload.committed_writes == 0 {
            return Err(WorkerError::Message(
                "no-quorum fencing workload committed zero writes".to_string(),
            ));
        }
        let rejected_writes = workload
            .fencing_failures
            .saturating_add(workload.transient_failures);
        if rejected_writes == 0 {
            return Err(WorkerError::Message(
                "expected write rejections (fencing or transient) during fail-safe window"
                    .to_string(),
            ));
        }

        let cutoff_ms = quorum_lost_at_ms.saturating_add(fencing_grace_ms);
        let commits_after_cutoff =
            ClusterFixture::count_commits_after_cutoff_strict(&workload, cutoff_ms)?;
        let allowed_post_cutoff_commits = 10usize;
        if commits_after_cutoff > allowed_post_cutoff_commits {
            return Err(WorkerError::Message(format!(
                "writes still committed after fail-safe fencing cutoff beyond tolerance; cutoff_ms={cutoff_ms} commits_after_cutoff={commits_after_cutoff} allowed={allowed_post_cutoff_commits}"
            )));
        }
        ClusterFixture::assert_no_split_brain_write_evidence(&workload, &ha_stats)?;

        let (node_id, row_count) = fixture
            .assert_table_key_integrity_strict(
                bootstrap_primary.as_str(),
                table_name.as_str(),
                1,
                Duration::from_secs(5),
            )
            .await?;
        fixture.record(format!(
            "no-quorum fencing key integrity verified on {node_id} with row_count={row_count}"
        ));

        let finished_at_unix_ms = ha_e2e::util::unix_now()?.0;
        Ok(StressScenarioSummary {
            schema_version: STRESS_SUMMARY_SCHEMA_VERSION,
            scenario: scenario_name.to_string(),
            status: "passed".to_string(),
            started_at_unix_ms,
            finished_at_unix_ms,
            bootstrap_primary: Some(bootstrap_primary),
            final_primary: None,
            former_primary_demoted: None,
            workload_spec: SqlWorkloadSpecSummary {
                worker_count: workload_spec.worker_count,
                run_interval_ms: workload_spec.run_interval_ms,
                table_name,
            },
            workload,
            ha_observations: ha_stats,
            notes: vec![
                format!("phase_history={}", ClusterFixture::format_phase_history(&phase_history)),
                format!("quorum_lost_at_ms={quorum_lost_at_ms}"),
                format!("fencing_cutoff_ms={cutoff_ms}"),
                format!("allowed_post_cutoff_commits={allowed_post_cutoff_commits}"),
            ],
        })
    })
    .await;

    let (summary, run_error) = match run_result {
        Ok(summary) => (summary, None),
        Err(err) => {
            let message = err.to_string();
            (
                StressScenarioSummary::failed(scenario_name.as_str(), message.clone()),
                Some(message),
            )
        }
    };
    let artifacts = fixture.write_stress_artifacts(scenario_name.as_str(), &summary);
    let shutdown_result = fixture.shutdown().await;
    finalize_stress_scenario_result(run_error, artifacts, shutdown_result)
    })
    .await
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn dual_primary_sample_assertion_fails_on_zero_samples() {
        let stats = HaObservationStats {
            sample_count: 0,
            api_error_count: 3,
            ..HaObservationStats::default()
        };
        assert!(assert_no_dual_primary_in_samples(&stats, 1).is_err());
    }

    #[test]
    fn dual_primary_sample_assertion_fails_on_dual_primary() {
        let stats = HaObservationStats {
            sample_count: 1,
            api_error_count: 0,
            max_concurrent_primaries: 2,
            ..HaObservationStats::default()
        };
        assert!(assert_no_dual_primary_in_samples(&stats, 1).is_err());
    }

    #[test]
    fn dual_primary_sample_assertion_passes_with_single_primary() -> Result<(), WorkerError> {
        let stats = HaObservationStats {
            sample_count: 1,
            api_error_count: 0,
            max_concurrent_primaries: 1,
            ..HaObservationStats::default()
        };
        assert_no_dual_primary_in_samples(&stats, 1)
    }

    #[test]
    fn fencing_cutoff_count_fails_when_timestamp_capture_failed() {
        let workload = SqlWorkloadStats {
            committed_writes: 1,
            commit_timestamp_capture_failures: 1,
            committed_at_unix_ms: vec![1234],
            ..SqlWorkloadStats::default()
        };
        assert!(ClusterFixture::count_commits_after_cutoff_strict(&workload, 1000).is_err());
    }

    #[test]
    fn fencing_cutoff_count_fails_when_timestamps_incomplete() {
        let workload = SqlWorkloadStats {
            committed_writes: 3,
            commit_timestamp_capture_failures: 0,
            committed_at_unix_ms: vec![1001, 1002],
            ..SqlWorkloadStats::default()
        };
        assert!(ClusterFixture::count_commits_after_cutoff_strict(&workload, 1000).is_err());
    }

    #[test]
    fn fencing_cutoff_count_fails_on_zero_timestamp() {
        let workload = SqlWorkloadStats {
            committed_writes: 1,
            commit_timestamp_capture_failures: 0,
            committed_at_unix_ms: vec![0],
            ..SqlWorkloadStats::default()
        };
        assert!(ClusterFixture::count_commits_after_cutoff_strict(&workload, 1000).is_err());
    }

    #[test]
    fn fencing_cutoff_count_counts_strictly_greater_than_cutoff() -> Result<(), WorkerError> {
        let workload = SqlWorkloadStats {
            committed_writes: 3,
            commit_timestamp_capture_failures: 0,
            committed_at_unix_ms: vec![1000, 1001, 999],
            ..SqlWorkloadStats::default()
        };
        let count = ClusterFixture::count_commits_after_cutoff_strict(&workload, 1000)?;
        assert_eq!(count, 1);
        Ok(())
    }

    #[test]
    fn family_symbols_remain_reachable_for_split_targets() {
        let _ = E2E_COMMAND_TIMEOUT;
        let _ = E2E_COMMAND_KILL_WAIT_TIMEOUT;
        let _ = E2E_SQL_WORKLOAD_COMMAND_TIMEOUT;
        let _ = E2E_SQL_WORKLOAD_COMMAND_KILL_WAIT_TIMEOUT;
        let _ = E2E_PG_STOP_TIMEOUT;
        let _ = E2E_HTTP_STEP_TIMEOUT;
        let _ = E2E_BOOTSTRAP_PRIMARY_TIMEOUT;
        let _ = E2E_SCENARIO_TIMEOUT;
        let _ = STRESS_ARTIFACT_DIR;
        let _ = STRESS_SUMMARY_SCHEMA_VERSION;
        let _: Option<StablePrimaryWaitPlan<'static>> = None;
        let _: Option<SqlWorkloadSpec> = None;
        let _: Option<SqlWorkloadTarget> = None;
        let _: Option<SqlWorkloadCtx> = None;
        let _: Option<SqlWorkloadHandle> = None;
        let _: Option<SqlWorkloadSpecSummary> = None;
        let _: Option<StressScenarioSummary> = None;
        let _ = SqlErrorClass::Transient;
        let _ = unique_e2e_token as fn() -> Result<String, WorkerError>;
        let _ = e2e_http_timeout_ms as fn() -> Result<u64, WorkerError>;
        let _ = classify_sql_error as fn(&str) -> SqlErrorClass;
        let _ = sanitize_component as fn(&str) -> String;
        let _ = sanitize_sql_identifier as fn(&str) -> String;
        let _ = StressScenarioSummary::failed as fn(&str, String) -> StressScenarioSummary;
        let _ = ClusterFixture::start;
        let _: fn(&mut ClusterFixture, String) = ClusterFixture::record;
        let _ = ClusterFixture::node_by_id;
        let _ = ClusterFixture::node_index_by_id;
        let _ = ClusterFixture::postgres_port_by_id;
        let _ = ClusterFixture::run_sql_on_node;
        let _ = ClusterFixture::run_sql_on_node_with_retry;
        let _ = ClusterFixture::cluster_sql_roles_best_effort;
        let _ = ClusterFixture::wait_for_rows_on_node;
        let _ = ClusterFixture::sql_workload_ctx;
        let _ = ClusterFixture::prepare_stress_table;
        let _ = ClusterFixture::start_sql_workload;
        let _ = ClusterFixture::stop_sql_workload_and_collect;
        let _ = ClusterFixture::sample_ha_states_window;
        let _ = ClusterFixture::assert_former_primary_demoted_or_unreachable_after_transition;
        let _ = ClusterFixture::assert_table_key_integrity_on_node;
        let _ = ClusterFixture::assert_table_key_integrity_strict;
        let _ = ClusterFixture::assert_no_split_brain_write_evidence;
        let _ = ClusterFixture::update_phase_history;
        let _ = ClusterFixture::format_phase_history;
        let _ = ClusterFixture::wait_for_stable_primary;
        let _ = ClusterFixture::wait_for_stable_primary_best_effort;
        let _ = ClusterFixture::assert_phase_history_contains_failover;
        let _ = ClusterFixture::node_api_base_url_by_index;
        let _ = ClusterFixture::cli_api_client_for_node_index;
        let _ = ClusterFixture::request_switchover_via_cli;
        let _ = ClusterFixture::request_switchover_until_stable_primary_changes;
        let _ = ClusterFixture::fetch_node_ha_state_by_index;
        let _ = ClusterFixture::poll_node_ha_states_best_effort;
        let _ = ClusterFixture::poll_node_ha_states_best_effort_with_timeout;
        let _ = ClusterFixture::cluster_ha_states;
        let _ = ClusterFixture::ensure_runtime_tasks_healthy;
        let _ = ClusterFixture::primary_members;
        let _ = ClusterFixture::wait_for_primary_change;
        let _ = ClusterFixture::wait_for_primary_change_best_effort;
        let _ = ClusterFixture::wait_for_stable_primary_via_sql;
        let _ = ClusterFixture::wait_for_stable_primary_resilient;
        let _ = ClusterFixture::assert_no_dual_primary_window;
        let _ = ClusterFixture::wait_for_all_nodes_failsafe;
        let _ = ClusterFixture::stop_postgres_for_node;
        let _ = ClusterFixture::stop_etcd_majority;
        let _ = ClusterFixture::write_timeline_artifact;
        let _ = ClusterFixture::write_stress_artifacts;
        let _ = ClusterFixture::shutdown;
        let _ = run_sql_workload_worker;
        let _ = finalize_stress_scenario_result;
        let _ = stop_etcd_majority_and_wait_failsafe_strict_all_nodes;
        let _ = e2e_multi_node_unassisted_failover_sql_consistency;
        let _ = e2e_multi_node_stress_planned_switchover_concurrent_sql;
        let _ = e2e_multi_node_stress_unassisted_failover_concurrent_sql;
        let _ = e2e_no_quorum_enters_failsafe_strict_all_nodes;
        let _ = e2e_no_quorum_fencing_blocks_post_cutoff_commits_and_preserves_integrity;
    }
}
