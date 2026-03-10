use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    future::Future,
    net::SocketAddr,
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
    api::{
        AcceptedResponse as CliAcceptedResponse, HaStateResponse, ReadinessResponse,
        SqlStatusResponse,
    },
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
    runtime_nodes: ha_e2e::RuntimeNodeSet,
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
const E2E_API_READINESS_TIMEOUT: Duration = Duration::from_secs(120);
const E2E_STABLE_PRIMARY_API_POLL_INTERVAL: Duration = Duration::from_millis(100);
const E2E_STABLE_PRIMARY_SQL_POLL_INTERVAL: Duration = Duration::from_millis(200);
const E2E_NO_DUAL_PRIMARY_SAMPLE_INTERVAL: Duration = Duration::from_millis(75);
const E2E_NO_QUORUM_OBSERVATION_TIMEOUT: Duration = Duration::from_secs(3);
const E2E_NO_QUORUM_LOG_INTERVAL: Duration = Duration::from_secs(5);
const E2E_NO_QUORUM_RETRY_INTERVAL: Duration = Duration::from_millis(100);
const E2E_SQL_RETRY_INTERVAL: Duration = Duration::from_millis(200);
const E2E_MEMBER_RECORD_STALE_AFTER_MS: u64 = 2_000;
const E2E_STABLE_PRIMARY_STRICT_TIMEOUT_CAP: Duration = Duration::from_secs(45);
const E2E_STABLE_PRIMARY_API_FALLBACK_TIMEOUT_CAP: Duration = Duration::from_secs(45);
const E2E_STABLE_PRIMARY_SQL_FALLBACK_TIMEOUT_CAP: Duration = Duration::from_secs(90);
const E2E_STABLE_PRIMARY_STRICT_CONSECUTIVE_CAP: usize = 3;
const E2E_STABLE_PRIMARY_RELAXED_CONSECUTIVE_CAP: usize = 2;
const E2E_STRESS_WORKLOAD_RUN_INTERVAL_MS: u64 = 250;
const E2E_STRESS_SAMPLE_INTERVAL: Duration = Duration::from_millis(150);
const E2E_STRESS_WORKLOAD_STOP_TIMEOUT: Duration = Duration::from_secs(2);
const E2E_NO_QUORUM_WORKLOAD_STOP_TIMEOUT: Duration = Duration::from_millis(200);
const E2E_SWITCHOVER_RETRY_BACKOFF: Duration = Duration::from_millis(500);
const E2E_PRIMARY_CONVERGENCE_TIMEOUT: Duration = Duration::from_secs(60);
const E2E_PRIMARY_CONVERGENCE_FALLBACK_TIMEOUT: Duration = Duration::from_secs(90);
const E2E_SQL_REPLICATION_ASSERT_TIMEOUT: Duration = Duration::from_secs(20);
const E2E_SHORT_NO_DUAL_PRIMARY_WINDOW: Duration = Duration::from_secs(3);
const E2E_LONG_NO_DUAL_PRIMARY_WINDOW: Duration = Duration::from_secs(10);
const E2E_STRESS_WORKLOAD_SETTLE_WAIT: Duration = Duration::from_secs(3);
const E2E_STRESS_SHORT_OBSERVATION_WINDOW: Duration = Duration::from_secs(8);
const E2E_STRESS_LONG_OBSERVATION_WINDOW: Duration = Duration::from_secs(10);
const E2E_POST_TRANSITION_SQL_TIMEOUT: Duration = Duration::from_secs(30);
const E2E_TABLE_INTEGRITY_TIMEOUT: Duration = Duration::from_secs(90);
const E2E_LOADED_FAILOVER_TIMEOUT: Duration = Duration::from_secs(180);
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

#[derive(Clone)]
struct ObservedNodeClient {
    node_id: String,
    node_addr: SocketAddr,
    client: CliApiClient,
}

#[derive(Clone, Copy, Debug)]
enum RecoveryBinaryKind {
    PgBasebackup,
    PgRewind,
}

impl RecoveryBinaryKind {
    fn binary_name(self) -> &'static str {
        match self {
            Self::PgBasebackup => "pg_basebackup",
            Self::PgRewind => "pg_rewind",
        }
    }
}

#[derive(Clone, Debug)]
struct FailureWrapper {
    wrapper_path: PathBuf,
    fail_enabled_marker: PathBuf,
    invoked_marker: PathBuf,
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

fn sql_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn shell_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn failure_wrapper_dir_for_namespace(
    namespace_root_dir: &Path,
    scenario_name: &str,
) -> Result<PathBuf, WorkerError> {
    let token = unique_e2e_token()?;
    let dir = namespace_root_dir.join("fault-injection").join(format!(
        "{}-{}",
        sanitize_component(scenario_name),
        token
    ));
    fs::create_dir_all(&dir).map_err(|err| {
        WorkerError::Message(format!(
            "create failure wrapper dir failed for {scenario_name}: {err}"
        ))
    })?;
    Ok(dir)
}

fn create_failure_wrapper_for_namespace(
    namespace_root_dir: &Path,
    scenario_name: &str,
    node_id: &str,
    binary_kind: RecoveryBinaryKind,
    real_binary: &Path,
) -> Result<FailureWrapper, WorkerError> {
    let wrapper_dir = failure_wrapper_dir_for_namespace(namespace_root_dir, scenario_name)?;
    let wrapper_path = wrapper_dir.join(format!(
        "{}-{}",
        sanitize_component(node_id),
        binary_kind.binary_name()
    ));
    let fail_enabled_marker = wrapper_dir.join("fail-enabled");
    let invoked_marker = wrapper_dir.join("invoked.log");
    let script = format!(
        concat!(
            "#!/bin/bash\n",
            "set -euo pipefail\n",
            "invoked_marker={invoked_marker}\n",
            "fail_enabled_marker={fail_enabled_marker}\n",
            "real_binary={real_binary}\n",
            "printf '%s %s\\n' \"$(date +%s)\" \"$*\" >> \"$invoked_marker\"\n",
            "if [[ -e \"$fail_enabled_marker\" ]]; then\n",
            "  exit 97\n",
            "fi\n",
            "exec \"$real_binary\" \"$@\"\n"
        ),
        invoked_marker = shell_literal(invoked_marker.to_string_lossy().as_ref()),
        fail_enabled_marker = shell_literal(fail_enabled_marker.to_string_lossy().as_ref()),
        real_binary = shell_literal(real_binary.to_string_lossy().as_ref()),
    );
    fs::write(&wrapper_path, script).map_err(|err| {
        WorkerError::Message(format!(
            "write failure wrapper script failed for {scenario_name} {node_id} {}: {err}",
            binary_kind.binary_name()
        ))
    })?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut permissions = fs::metadata(&wrapper_path)
            .map_err(|err| {
                WorkerError::Message(format!(
                    "read failure wrapper metadata failed for {}: {err}",
                    wrapper_path.display()
                ))
            })?
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&wrapper_path, permissions).map_err(|err| {
            WorkerError::Message(format!(
                "set failure wrapper executable bit failed for {}: {err}",
                wrapper_path.display()
            ))
        })?;
    }

    Ok(FailureWrapper {
        wrapper_path,
        fail_enabled_marker,
        invoked_marker,
    })
}

fn set_failure_wrapper_enabled(wrapper: &FailureWrapper, enabled: bool) -> Result<(), WorkerError> {
    if enabled {
        fs::write(&wrapper.fail_enabled_marker, b"enabled\n").map_err(|err| {
            WorkerError::Message(format!(
                "enable failure wrapper marker failed at {}: {err}",
                wrapper.fail_enabled_marker.display()
            ))
        })?;
    } else {
        match fs::remove_file(&wrapper.fail_enabled_marker) {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => {
                return Err(WorkerError::Message(format!(
                    "disable failure wrapper marker failed at {}: {err}",
                    wrapper.fail_enabled_marker.display()
                )));
            }
        }
    }
    Ok(())
}

fn write_pgtm_cli_config(api_observe_addr: &std::net::SocketAddr) -> Result<PathBuf, WorkerError> {
    let token = unique_e2e_token()?;
    let data_dir = std::env::temp_dir().join(format!("pgtm-cli-data-{token}"));
    let path = std::env::temp_dir().join(format!("pgtm-cli-config-{token}.toml"));
    let contents = format!(
        r##"
[cluster]
name = "cluster-a"
member_id = "node-a"

[postgres]
data_dir = "{data_dir}"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtm/socket"
log_file = "/tmp/pgtm/postgres.log"
local_conn_identity = {{ user = "postgres", dbname = "postgres", ssl_mode = "prefer" }}
rewind_conn_identity = {{ user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }}
tls = {{ mode = "disabled" }}
roles = {{ superuser = {{ username = "postgres", auth = {{ type = "password", password = {{ content = "secret-password" }} }} }}, replicator = {{ username = "replicator", auth = {{ type = "password", password = {{ content = "secret-password" }} }} }}, rewinder = {{ username = "rewinder", auth = {{ type = "password", password = {{ content = "secret-password" }} }} }} }}
pg_hba = {{ source = {{ content = "local all all trust" }} }}
pg_ident = {{ source = {{ content = "# empty" }} }}

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
pg_rewind_timeout_ms = 1000
bootstrap_timeout_ms = 1000
fencing_timeout_ms = 1000
binaries = {{ postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }}

[api]
listen_addr = "{api_observe_addr}"
security = {{ tls = {{ mode = "disabled" }}, auth = {{ type = "disabled" }} }}

[pgtm]
api_url = "http://{api_observe_addr}"
"##,
        data_dir = data_dir.display(),
        api_observe_addr = api_observe_addr
    );
    fs::write(&path, contents)
        .map_err(|err| WorkerError::Message(format!("write pgtm cli config failed: {err}")))?;
    Ok(path)
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

fn sample_key_set(keys: &BTreeSet<String>) -> String {
    keys.iter().take(5).cloned().collect::<Vec<_>>().join(",")
}

fn committed_key_set_through_cutoff(
    workload: &SqlWorkloadStats,
    cutoff_ms: u64,
) -> Result<BTreeSet<String>, WorkerError> {
    let mut required_keys = BTreeSet::new();
    for worker in &workload.worker_stats {
        if worker.committed_keys.len() != worker.committed_at_unix_ms.len() {
            return Err(WorkerError::Message(format!(
                "worker {} committed key/timestamp length mismatch: keys={} timestamps={}",
                worker.worker_id,
                worker.committed_keys.len(),
                worker.committed_at_unix_ms.len()
            )));
        }
        for (key, committed_at_ms) in worker
            .committed_keys
            .iter()
            .zip(worker.committed_at_unix_ms.iter())
        {
            if *committed_at_ms <= cutoff_ms {
                required_keys.insert(key.clone());
            }
        }
    }
    Ok(required_keys)
}

fn assert_recovered_committed_keys_match_bounds(
    observed_rows: &[String],
    required_keys: &BTreeSet<String>,
    allowed_keys: &BTreeSet<String>,
    node_id: &str,
    table_name: &str,
) -> Result<u64, WorkerError> {
    let observed_row_count = u64::try_from(observed_rows.len()).map_err(|_| {
        WorkerError::Message(format!(
            "observed row count overflow while verifying {table_name} on {node_id}"
        ))
    })?;
    let observed_key_set: BTreeSet<String> = observed_rows.iter().cloned().collect();
    let observed_unique_count = u64::try_from(observed_key_set.len()).map_err(|_| {
        WorkerError::Message(format!(
            "observed unique key count overflow while verifying {table_name} on {node_id}"
        ))
    })?;
    if observed_unique_count != observed_row_count {
        return Err(WorkerError::Message(format!(
            "duplicate (worker_id,seq) rows detected on {node_id} for {table_name}: observed_rows={observed_row_count} unique_keys={observed_unique_count}"
        )));
    }

    let missing_keys: BTreeSet<String> = required_keys
        .difference(&observed_key_set)
        .cloned()
        .collect();
    let unexpected_keys: BTreeSet<String> =
        observed_key_set.difference(allowed_keys).cloned().collect();
    if !missing_keys.is_empty() || !unexpected_keys.is_empty() {
        return Err(WorkerError::Message(format!(
            "recovered key-set mismatch on {node_id} for {table_name}: missing_required_count={} missing_sample=[{}] unexpected_count={} unexpected_sample=[{}]",
            missing_keys.len(),
            sample_key_set(&missing_keys),
            unexpected_keys.len(),
            sample_key_set(&unexpected_keys),
        )));
    }

    Ok(observed_row_count)
}

impl ClusterFixture {
    async fn start(node_count: usize) -> Result<Self, WorkerError> {
        Self::start_with_config(ha_e2e::TestConfig {
            test_name: "ha-e2e-multi-node".to_string(),
            cluster_name: "cluster-e2e".to_string(),
            scope: "scope-ha-e2e".to_string(),
            node_count,
            namespace: None,
            etcd_members: vec![
                "etcd-a".to_string(),
                "etcd-b".to_string(),
                "etcd-c".to_string(),
            ],
            recovery_binary_overrides: BTreeMap::new(),
            postgres_roles: None,
            mode: ha_e2e::Mode::Plain,
            timeouts: ha_e2e::TimeoutConfig {
                command_timeout: E2E_COMMAND_TIMEOUT,
                command_kill_wait_timeout: E2E_COMMAND_KILL_WAIT_TIMEOUT,
                http_step_timeout: E2E_HTTP_STEP_TIMEOUT,
                api_readiness_timeout: E2E_API_READINESS_TIMEOUT,
                bootstrap_primary_timeout: E2E_BOOTSTRAP_PRIMARY_TIMEOUT,
                scenario_timeout: E2E_SCENARIO_TIMEOUT,
            },
        })
        .await
    }

    async fn start_with_config(config: ha_e2e::TestConfig) -> Result<Self, WorkerError> {
        let handle = ha_e2e::start_cluster(config).await?;

        let TestClusterHandle {
            guard,
            timeouts: _,
            binaries,
            superuser_username,
            superuser_dbname,
            etcd,
            nodes,
            runtime_nodes,
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
            runtime_nodes,
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

    fn disable_failure_wrapper(
        &mut self,
        scenario_name: &str,
        wrapper: &FailureWrapper,
    ) -> Result<(), WorkerError> {
        set_failure_wrapper_enabled(wrapper, false)?;
        self.record(format!(
            "{scenario_name}: disabled failure wrapper {}",
            wrapper.wrapper_path.display()
        ));
        Ok(())
    }

    fn assert_failure_wrapper_invoked(
        &mut self,
        scenario_name: &str,
        wrapper: &FailureWrapper,
    ) -> Result<(), WorkerError> {
        let contents = fs::read_to_string(&wrapper.invoked_marker).map_err(|err| {
            WorkerError::Message(format!(
                "{scenario_name}: failure wrapper was not invoked at {}: {err}",
                wrapper.invoked_marker.display()
            ))
        })?;
        if contents.trim().is_empty() {
            return Err(WorkerError::Message(format!(
                "{scenario_name}: failure wrapper invocation log is empty at {}",
                wrapper.invoked_marker.display()
            )));
        }
        self.record(format!(
            "{scenario_name}: observed failure wrapper invocation {}",
            wrapper.wrapper_path.display()
        ));
        Ok(())
    }

    fn proof_table_name(&self, prefix: &str) -> Result<String, WorkerError> {
        let token = unique_e2e_token()?;
        Ok(sanitize_sql_identifier(
            format!("{prefix}_{token}").as_str(),
        ))
    }

    fn node_ids_excluding(&self, excluded_node_id: &str) -> Vec<String> {
        self.nodes
            .iter()
            .filter(|node| node.id != excluded_node_id)
            .map(|node| node.id.clone())
            .collect()
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

    async fn restart_runtime_process_for_node(&mut self, node_id: &str) -> Result<(), WorkerError> {
        let node = self.node_by_id(node_id).cloned().ok_or_else(|| {
            WorkerError::Message(format!("unknown node id for runtime restart: {node_id}"))
        })?;
        self.record(format!(
            "runtime restart: stop requested for node={node_id}"
        ));
        self.runtime_nodes
            .restart_node(&node, E2E_HTTP_STEP_TIMEOUT, E2E_API_READINESS_TIMEOUT)
            .await?;
        self.record(format!(
            "runtime restart: api loss observed and process respawned for node={node_id}"
        ));
        self.record(format!("runtime restart: api recovered for node={node_id}"));
        Ok(())
    }

    async fn stop_runtime_process_for_node(&mut self, node_id: &str) -> Result<(), WorkerError> {
        let node = self.node_by_id(node_id).cloned().ok_or_else(|| {
            WorkerError::Message(format!("unknown node id for runtime stop: {node_id}"))
        })?;
        self.record(format!("runtime stop: stop requested for node={node_id}"));
        self.runtime_nodes
            .stop_node(
                &node,
                E2E_COMMAND_TIMEOUT,
                E2E_COMMAND_KILL_WAIT_TIMEOUT,
                E2E_HTTP_STEP_TIMEOUT,
                E2E_API_READINESS_TIMEOUT,
            )
            .await?;
        self.record(format!("runtime stop: api unavailable for node={node_id}"));
        Ok(())
    }

    async fn create_proof_table(
        &self,
        primary_node_id: &str,
        table_name: &str,
    ) -> Result<(), WorkerError> {
        let sql = format!(
            "CREATE TABLE IF NOT EXISTS {table_name} (id BIGINT PRIMARY KEY, payload TEXT NOT NULL)"
        );
        self.run_sql_on_node_with_retry(
            primary_node_id,
            sql.as_str(),
            E2E_SQL_REPLICATION_ASSERT_TIMEOUT,
        )
        .await
        .map(|_| ())
    }

    async fn insert_proof_row(
        &self,
        primary_node_id: &str,
        table_name: &str,
        row_id: u64,
        payload: &str,
    ) -> Result<(), WorkerError> {
        let sql = format!(
            "INSERT INTO {table_name} (id, payload) VALUES ({row_id}, {}) ON CONFLICT (id) DO UPDATE SET payload = EXCLUDED.payload",
            sql_literal(payload)
        );
        self.run_sql_on_node_with_retry(
            primary_node_id,
            sql.as_str(),
            E2E_POST_TRANSITION_SQL_TIMEOUT,
        )
        .await
        .map(|_| ())
    }

    async fn wait_for_proof_rows_on_all_nodes(
        &self,
        table_name: &str,
        expected_rows: &[String],
        timeout: Duration,
    ) -> Result<(), WorkerError> {
        let node_ids = self
            .nodes
            .iter()
            .map(|node| node.id.clone())
            .collect::<Vec<_>>();
        self.wait_for_proof_rows_on_nodes(node_ids.as_slice(), table_name, expected_rows, timeout)
            .await
    }

    async fn wait_for_proof_rows_on_nodes(
        &self,
        node_ids: &[String],
        table_name: &str,
        expected_rows: &[String],
        timeout: Duration,
    ) -> Result<(), WorkerError> {
        let sql = format!("SELECT id::text || ':' || payload FROM {table_name} ORDER BY id");
        for node_id in node_ids {
            self.wait_for_rows_on_node(node_id.as_str(), sql.as_str(), expected_rows, timeout)
                .await?;
        }
        Ok(())
    }

    async fn wait_for_queryable_nodes(
        &self,
        required_node_id: &str,
        min_nodes: usize,
        timeout: Duration,
    ) -> Result<Vec<String>, WorkerError> {
        if min_nodes == 0 {
            return Err(WorkerError::Message(
                "min_nodes must be greater than zero".to_string(),
            ));
        }

        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            let (sql_roles, sql_errors) = self.cluster_sql_roles_best_effort().await?;
            let node_ids = sql_roles
                .iter()
                .map(|(node_id, _)| node_id.clone())
                .collect::<Vec<_>>();
            if node_ids.len() >= min_nodes
                && node_ids.iter().any(|node_id| node_id == required_node_id)
            {
                return Ok(node_ids);
            }

            let observation = format!("queryable_nodes={node_ids:?} sql_errors={sql_errors:?}");
            if tokio::time::Instant::now() >= deadline {
                return Err(WorkerError::Message(format!(
                    "timed out waiting for at least {min_nodes} queryable nodes including {required_node_id}; last_observation={observation}"
                )));
            }
            tokio::time::sleep(E2E_SQL_RETRY_INTERVAL).await;
        }
    }

    async fn wait_for_node_sql_role(
        &self,
        node_id: &str,
        expected_role: &str,
        timeout: Duration,
    ) -> Result<(), WorkerError> {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            let (sql_roles, sql_errors) = self.cluster_sql_roles_best_effort().await?;
            let observed_role = sql_roles.iter().find_map(|(observed_node_id, role)| {
                (observed_node_id == node_id).then_some(role.as_str())
            });
            if observed_role == Some(expected_role) {
                return Ok(());
            }
            let observation = format!("roles={sql_roles:?} sql_errors={sql_errors:?}");
            if tokio::time::Instant::now() >= deadline {
                return Err(WorkerError::Message(format!(
                    "timed out waiting for {node_id} to report SQL role {expected_role}; last_observation={observation}"
                )));
            }
            tokio::time::sleep(E2E_SQL_RETRY_INTERVAL).await;
        }
    }

    async fn assert_node_not_queryable_for_window(
        &mut self,
        node_id: &str,
        window: Duration,
    ) -> Result<(), WorkerError> {
        let deadline = tokio::time::Instant::now() + window;
        loop {
            match self
                .run_sql_on_node(node_id, "SELECT 1", E2E_SQL_WORKLOAD_COMMAND_TIMEOUT)
                .await
            {
                Ok(output) => {
                    return Err(WorkerError::Message(format!(
                        "expected {node_id} to remain non-queryable during failure window, but SQL succeeded with output={output:?}"
                    )));
                }
                Err(err) => {
                    self.record(format!(
                        "expected non-queryable node {node_id} still failing SQL during observation window: {err}"
                    ));
                }
            }
            if tokio::time::Instant::now() >= deadline {
                return Ok(());
            }
            tokio::time::sleep(E2E_SQL_RETRY_INTERVAL).await;
        }
    }

    fn observed_node_clients(&self) -> Result<Vec<ObservedNodeClient>, WorkerError> {
        let mut clients = Vec::with_capacity(self.nodes.len());
        for node_index in 0..self.nodes.len() {
            let node = self.nodes.get(node_index).ok_or_else(|| {
                WorkerError::Message(format!(
                    "invalid node index while building observer clients: {node_index}"
                ))
            })?;
            let (node_id, client) = self.cli_api_client_for_node_index(node_index)?;
            clients.push(ObservedNodeClient {
                node_id,
                node_addr: node.api_addr,
                client,
            });
        }
        Ok(clients)
    }

    async fn sample_ha_states_window_from_clients(
        clients: Vec<ObservedNodeClient>,
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
            observer.record_poll_attempt();
            let mut states = Vec::new();
            let mut errors = Vec::new();
            for observed in &clients {
                match ha_e2e::util::get_ha_state_with_fallback(
                    &observed.client,
                    observed.node_id.as_str(),
                    observed.node_addr,
                    E2E_HTTP_STEP_TIMEOUT,
                )
                .await
                {
                    Ok(state) => states.push(state),
                    Err(err) => errors.push(format!("node={} error={err}", observed.node_id)),
                }
            }

            if states.is_empty() {
                observer.record_observation_gap(&errors, &[]);
            } else {
                observer.record_api_states(&states, &errors)?;
            }

            if tokio::time::Instant::now() >= deadline {
                return Ok(observer.into_stats());
            }
            tokio::time::sleep(interval).await;
        }
    }

    async fn observe_no_dual_primary_while_clients<T, F>(
        clients: Vec<ObservedNodeClient>,
        window: Duration,
        ring_capacity: usize,
        action: F,
    ) -> Result<(T, HaObservationStats), WorkerError>
    where
        F: Future<Output = Result<T, WorkerError>>,
    {
        let observer_task = tokio::task::spawn_local(async move {
            ClusterFixture::sample_ha_states_window_from_clients(
                clients,
                window,
                E2E_NO_DUAL_PRIMARY_SAMPLE_INTERVAL,
                ring_capacity,
            )
            .await
        });
        let action_result = action.await;
        let ha_stats = observer_task.await.map_err(|err| {
            WorkerError::Message(format!("HA observer task join failed: {err}"))
        })??;
        assert_no_dual_primary_in_samples(&ha_stats, 1)?;
        let result = action_result?;
        Ok((result, ha_stats))
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
                    tokio::time::sleep(E2E_SQL_RETRY_INTERVAL).await;
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
            tokio::time::sleep(E2E_SQL_RETRY_INTERVAL).await;
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
                    tokio::time::sleep(E2E_SQL_RETRY_INTERVAL).await;
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
                    tokio::time::sleep(E2E_SQL_RETRY_INTERVAL).await;
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
            tokio::time::sleep(E2E_SQL_RETRY_INTERVAL).await;
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

    async fn assert_table_recovery_key_integrity_on_node(
        &mut self,
        node_id: &str,
        table_name: &str,
        required_keys: &BTreeSet<String>,
        allowed_keys: &BTreeSet<String>,
        timeout: Duration,
    ) -> Result<u64, WorkerError> {
        let query = format!(
            "SELECT worker_id::text || ':' || seq::text FROM {table_name} ORDER BY worker_id, seq"
        );
        let rows_raw = self
            .run_sql_on_node_with_retry(node_id, query.as_str(), timeout)
            .await
            .map_err(|err| {
                WorkerError::Message(format!(
                    "recovery key verification query failed on {node_id} for {table_name}: {err}"
                ))
            })?;
        let observed_rows = ha_e2e::util::parse_psql_rows(rows_raw.as_str());
        assert_recovered_committed_keys_match_bounds(
            observed_rows.as_slice(),
            required_keys,
            allowed_keys,
            node_id,
            table_name,
        )
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
            tokio::time::sleep(E2E_STABLE_PRIMARY_API_POLL_INTERVAL).await;
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
            tokio::time::sleep(E2E_STABLE_PRIMARY_API_POLL_INTERVAL).await;
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

    async fn run_observe_cli_command_via_node(
        &mut self,
        node_id: &str,
        command_args: &[&str],
        json: bool,
    ) -> Result<String, WorkerError> {
        let node_index = self.node_index_by_id(node_id).ok_or_else(|| {
            WorkerError::Message(format!(
                "unknown node id for CLI observe command: {node_id}"
            ))
        })?;
        let timeout_ms = e2e_http_timeout_ms()?;
        let config_path = write_pgtm_cli_config(&self.nodes[node_index].api_observe_addr)?;
        let command_label = command_args.join(" ");
        self.record(format!(
            "cli observe start: node={node_id} command={command_label}"
        ));

        let mut argv = vec![
            "pgtm".to_string(),
            "-c".to_string(),
            config_path.display().to_string(),
            "--timeout-ms".to_string(),
            timeout_ms.to_string(),
        ];
        if json {
            argv.push("--json".to_string());
        }
        argv.extend(command_args.iter().map(|arg| (*arg).to_string()));

        let cli = Cli::try_parse_from(argv)
            .map_err(|err| WorkerError::Message(format!("parse observe CLI args failed: {err}")))?;
        let result = cli::run(cli).await;
        let _ = fs::remove_file(&config_path);
        match result {
            Ok(output) => {
                self.record(format!(
                    "cli observe success: node={node_id} command={command_label}"
                ));
                Ok(output)
            }
            Err(err) => Err(WorkerError::Message(format!(
                "run observe CLI command failed via {node_id}: {err}"
            ))),
        }
    }

    async fn run_observe_cli_command_via_node_with_retry(
        &mut self,
        node_id: &str,
        command_args: &[&str],
        json: bool,
        timeout: Duration,
    ) -> Result<String, WorkerError> {
        let started = tokio::time::Instant::now();
        let command_label = command_args.join(" ");
        let timeout_error = loop {
            match self
                .run_observe_cli_command_via_node(node_id, command_args, json)
                .await
            {
                Ok(output) => return Ok(output),
                Err(err) if started.elapsed() >= timeout => break err.to_string(),
                Err(_) => {
                    tokio::time::sleep(E2E_STABLE_PRIMARY_API_POLL_INTERVAL).await;
                }
            }
        };

        Err(WorkerError::Message(format!(
            "timed out waiting for CLI observe command `{command_label}` via {node_id}: {timeout_error}"
        )))
    }

    async fn request_switchover_via_cli(&mut self) -> Result<(), WorkerError> {
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
                let config_path = write_pgtm_cli_config(&self.nodes[node_index].api_observe_addr)?;
                self.record(format!(
                    "cli request start: round={round}/{max_transport_rounds} node={node_id} switchover request"
                ));
                let argv: Vec<String> = vec![
                    "pgtm".to_string(),
                    "-c".to_string(),
                    config_path.display().to_string(),
                    "--timeout-ms".to_string(),
                    timeout_ms.to_string(),
                    "--json".to_string(),
                    "switchover".to_string(),
                    "request".to_string(),
                ];
                let cli = Cli::try_parse_from(argv).map_err(|err| {
                    WorkerError::Message(format!("parse switchover CLI args failed: {err}"))
                })?;
                let result = cli::run(cli).await;
                let _ = fs::remove_file(&config_path);
                match result {
                    Ok(out) => {
                        self.record(format!(
                            "cli request success: round={round}/{max_transport_rounds} node={node_id} switchover request accepted=true"
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
                                "cli request transport failure: round={round}/{max_transport_rounds} node={node_id} err={err_string}"
                            ));
                        }
                        _ => {
                            return Err(WorkerError::Message(format!(
                                "run switchover CLI command failed via {node_id} ({base_url}): {err}"
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

    async fn request_targeted_switchover_via_api(
        &mut self,
        target_node_id: &str,
    ) -> Result<(), WorkerError> {
        if self.nodes.is_empty() {
            return Err(WorkerError::Message(
                "no nodes available for targeted switchover request".to_string(),
            ));
        }

        let max_transport_rounds: usize = 5;
        let mut last_transport_error = "transport error".to_string();

        for round in 1..=max_transport_rounds {
            for node_index in 0..self.nodes.len() {
                let (node_id, client) = self.cli_api_client_for_node_index(node_index)?;
                self.record(format!(
                    "api targeted switchover request: round={round}/{max_transport_rounds} node={node_id} target={target_node_id}"
                ));
                match client
                    .post_switchover(Some(target_node_id.to_string()))
                    .await
                {
                    Ok(_) => {
                        self.record(format!(
                            "api targeted switchover accepted: round={round}/{max_transport_rounds} node={node_id} target={target_node_id}"
                        ));
                        return Ok(());
                    }
                    Err(CliError::Transport(err)) => {
                        last_transport_error = format!(
                            "node={node_id} round={round} target={target_node_id} err={err}"
                        );
                        self.record(format!(
                            "api targeted switchover transport failure: round={round}/{max_transport_rounds} node={node_id} target={target_node_id} err={err}"
                        ));
                    }
                    Err(err) => {
                        return Err(WorkerError::Message(format!(
                            "targeted switchover request failed via {node_id} for target {target_node_id}: {err}"
                        )));
                    }
                }
            }

            if round < max_transport_rounds {
                let backoff_ms = 200_u64.saturating_mul(round as u64);
                tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
            }
        }

        Err(WorkerError::Message(format!(
            "targeted switchover request transport failed after {max_transport_rounds} round(s) for target {target_node_id}: {last_transport_error}"
        )))
    }

    async fn request_targeted_switchover_rejected_via_api(
        &mut self,
        target_node_id: &str,
        expected_error_fragment: &str,
    ) -> Result<String, WorkerError> {
        if self.nodes.is_empty() {
            return Err(WorkerError::Message(
                "no nodes available for targeted switchover rejection check".to_string(),
            ));
        }

        let max_transport_rounds: usize = 5;
        let mut last_transport_error = "transport error".to_string();

        for round in 1..=max_transport_rounds {
            for node_index in 0..self.nodes.len() {
                let (node_id, client) = self.cli_api_client_for_node_index(node_index)?;
                self.record(format!(
                    "api switchover rejection probe: round={round}/{max_transport_rounds} node={node_id} target={target_node_id}"
                ));
                match client
                    .post_switchover(Some(target_node_id.to_string()))
                    .await
                {
                    Ok(_) => {
                        return Err(WorkerError::Message(format!(
                            "targeted switchover unexpectedly succeeded for ineligible target {target_node_id}"
                        )));
                    }
                    Err(CliError::Transport(err)) => {
                        last_transport_error = format!("node={node_id} round={round} err={err}");
                        self.record(format!(
                            "api switchover rejection transport failure: round={round}/{max_transport_rounds} node={node_id} target={target_node_id} err={err}"
                        ));
                    }
                    Err(CliError::ApiStatus { body, .. }) => {
                        if body.contains(expected_error_fragment) {
                            self.record(format!(
                                "api switchover rejection observed: node={node_id} target={target_node_id} body={body}"
                            ));
                            return Ok(body);
                        }
                        return Err(WorkerError::Message(format!(
                            "targeted switchover rejection body for {target_node_id} did not contain `{expected_error_fragment}`: {body}"
                        )));
                    }
                    Err(err) => {
                        return Err(WorkerError::Message(format!(
                            "targeted switchover rejection probe failed via {node_id} for target {target_node_id}: {err}"
                        )));
                    }
                }
            }

            if round < max_transport_rounds {
                tokio::time::sleep(E2E_SWITCHOVER_RETRY_BACKOFF).await;
            }
        }

        Err(WorkerError::Message(format!(
            "targeted switchover rejection probe failed after {max_transport_rounds} transport round(s) for target {target_node_id}: {last_transport_error}"
        )))
    }

    async fn request_switchover_until_stable_primary_changes(
        &mut self,
        previous_primary: &str,
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
            self.request_switchover_via_cli().await?;
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
                tokio::time::sleep(E2E_SWITCHOVER_RETRY_BACKOFF).await;
            }
        }

        Err(WorkerError::Message(format!(
            "switchover did not change primary from {previous_primary} after {max_attempts} attempt(s); last_error={last_error}"
        )))
    }

    async fn wait_for_expected_primary_best_effort(
        &mut self,
        expected_primary: &str,
        timeout: Duration,
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
        let mut stable_count = 0usize;
        let mut last_error = "none".to_string();

        loop {
            self.ensure_runtime_tasks_healthy().await?;
            match self.poll_node_ha_states_best_effort().await {
                Ok(polled) => {
                    let mut states = Vec::new();
                    for (node_id, state_result) in polled {
                        match state_result {
                            Ok(state) => states.push(state),
                            Err(err) => {
                                last_error = format!("HA state poll failed for {node_id}: {err}");
                            }
                        }
                    }

                    if states.len() >= min_observed_nodes {
                        Self::update_phase_history(phase_history, states.as_slice());
                        let primaries = Self::primary_members(states.as_slice());
                        if primaries.len() == 1
                            && primaries.first().map(String::as_str) == Some(expected_primary)
                        {
                            stable_count = stable_count.saturating_add(1);
                            if stable_count >= required_consecutive {
                                return Ok(expected_primary.to_string());
                            }
                        } else {
                            stable_count = 0;
                            last_error = format!("observed primaries={primaries:?}");
                        }
                    } else {
                        stable_count = 0;
                        last_error = format!(
                            "insufficient observed HA states: observed={} required={min_observed_nodes}",
                            states.len()
                        );
                    }
                }
                Err(err) => {
                    stable_count = 0;
                    last_error = err.to_string();
                }
            }

            if tokio::time::Instant::now() >= deadline {
                return Err(WorkerError::Message(format!(
                    "timed out waiting for expected stable primary {expected_primary}; last_error={last_error}"
                )));
            }
            tokio::time::sleep(E2E_STABLE_PRIMARY_API_POLL_INTERVAL).await;
        }
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

    async fn wait_for_member_to_be_ineligible(
        &mut self,
        observer_node_id: &str,
        target_node_id: &str,
        timeout: Duration,
    ) -> Result<(), WorkerError> {
        let observer_index = self.node_index_by_id(observer_node_id).ok_or_else(|| {
            WorkerError::Message(format!(
                "unknown observer node for eligibility wait: {observer_node_id}"
            ))
        })?;
        let deadline = tokio::time::Instant::now() + timeout;

        loop {
            let state = self.fetch_node_ha_state_by_index(observer_index).await?;
            let observed_at_ms = ha_e2e::util::unix_now()?.0;
            let observation = match state
                .members
                .iter()
                .find(|member| member.member_id == target_node_id)
            {
                Some(member_state)
                    if member_state.sql != SqlStatusResponse::Healthy
                        || member_state.readiness != ReadinessResponse::Ready =>
                {
                    self.record(format!(
                        "target member became ineligible: observer={observer_node_id} target={target_node_id} sql={} readiness={}",
                        member_state.sql, member_state.readiness
                    ));
                    return Ok(());
                }
                Some(member_state)
                    if observed_at_ms.saturating_sub(member_state.updated_at_ms)
                        > E2E_MEMBER_RECORD_STALE_AFTER_MS =>
                {
                    self.record(format!(
                        "target member became ineligible: observer={observer_node_id} target={target_node_id} stale_ms={}",
                        observed_at_ms.saturating_sub(member_state.updated_at_ms)
                    ));
                    return Ok(());
                }
                Some(member_state) => format!(
                    "sql={} readiness={} role={} stale_ms={}",
                    member_state.sql,
                    member_state.readiness,
                    member_state.role,
                    observed_at_ms.saturating_sub(member_state.updated_at_ms)
                ),
                None => "target member missing from ha state".to_string(),
            };

            if tokio::time::Instant::now() >= deadline {
                return Err(WorkerError::Message(format!(
                    "timed out waiting for {target_node_id} to become ineligible from observer {observer_node_id}; last_observation={}",
                    observation
                )));
            }
            tokio::time::sleep(E2E_STABLE_PRIMARY_API_POLL_INTERVAL).await;
        }
    }

    async fn ensure_runtime_tasks_healthy(&mut self) -> Result<(), WorkerError> {
        self.runtime_nodes.ensure_healthy().await
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
            tokio::time::sleep(E2E_STABLE_PRIMARY_API_POLL_INTERVAL).await;
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
            tokio::time::sleep(E2E_STABLE_PRIMARY_API_POLL_INTERVAL).await;
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

            tokio::time::sleep(E2E_STABLE_PRIMARY_SQL_POLL_INTERVAL).await;
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

        let strict_timeout = std::cmp::min(plan.timeout, E2E_STABLE_PRIMARY_STRICT_TIMEOUT_CAP);
        let api_fallback_timeout = std::cmp::min(
            plan.fallback_timeout,
            E2E_STABLE_PRIMARY_API_FALLBACK_TIMEOUT_CAP,
        );
        let sql_fallback_timeout = std::cmp::min(
            plan.fallback_timeout,
            E2E_STABLE_PRIMARY_SQL_FALLBACK_TIMEOUT_CAP,
        );
        let strict_required_consecutive = plan
            .required_consecutive
            .min(E2E_STABLE_PRIMARY_STRICT_CONSECUTIVE_CAP);
        let relaxed_required_consecutive = plan
            .fallback_required_consecutive
            .min(E2E_STABLE_PRIMARY_RELAXED_CONSECUTIVE_CAP);

        match self
            .wait_for_stable_primary(
                strict_timeout,
                plan.excluded_primary,
                strict_required_consecutive,
                phase_history,
            )
            .await
        {
            Ok(primary) => {
                if let Some(confirmed_primary) = self
                    .confirm_stable_primary_candidate_via_sql(
                        plan.context,
                        primary.as_str(),
                        plan.excluded_primary,
                        sql_fallback_timeout,
                        relaxed_required_consecutive,
                        plan.min_observed_nodes,
                    )
                    .await?
                {
                    return Ok(confirmed_primary);
                }
            }
            Err(wait_err) => {
                self.record(format!(
                    "{}: strict stable-primary wait failed: {wait_err}; retrying with best-effort API polling",
                    plan.context
                ));
            }
        }

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
            Ok(primary) => {
                if let Some(confirmed_primary) = self
                    .confirm_stable_primary_candidate_via_sql(
                        plan.context,
                        primary.as_str(),
                        plan.excluded_primary,
                        sql_fallback_timeout,
                        relaxed_required_consecutive,
                        plan.min_observed_nodes,
                    )
                    .await?
                {
                    return Ok(confirmed_primary);
                }
                self.record(format!(
                    "{}: best-effort API stable-primary candidate {primary} was not corroborated by SQL; retrying with SQL role polling",
                    plan.context
                ));
            }
            Err(best_effort_err) => {
                self.record(format!(
                    "{}: best-effort API stable-primary wait failed: {best_effort_err}; retrying with SQL role polling",
                    plan.context
                ));
            }
        }

        self.wait_for_stable_primary_via_sql(
            sql_fallback_timeout,
            plan.excluded_primary,
            relaxed_required_consecutive,
            plan.min_observed_nodes,
        )
        .await
    }

    async fn confirm_stable_primary_candidate_via_sql(
        &mut self,
        context: &str,
        api_primary: &str,
        excluded_primary: Option<&str>,
        timeout: Duration,
        required_consecutive: usize,
        min_observed_nodes: usize,
    ) -> Result<Option<String>, WorkerError> {
        match self
            .wait_for_stable_primary_via_sql(
                timeout,
                excluded_primary,
                required_consecutive,
                min_observed_nodes,
            )
            .await
        {
            Ok(sql_primary) => {
                if sql_primary != api_primary {
                    self.record(format!(
                        "{context}: API stable-primary candidate {api_primary} disagreed with SQL stable primary {sql_primary}; preferring SQL result"
                    ));
                }
                Ok(Some(sql_primary))
            }
            Err(err) => {
                self.record(format!(
                    "{context}: SQL corroboration for API stable-primary candidate {api_primary} failed: {err}"
                ));
                Ok(None)
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
            tokio::time::sleep(E2E_NO_DUAL_PRIMARY_SAMPLE_INTERVAL).await;
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
                .poll_node_ha_states_best_effort_with_timeout(E2E_NO_QUORUM_OBSERVATION_TIMEOUT)
                .await
            {
                Ok(values) => values,
                Err(err) => {
                    last_observation = Some(format!("poll:error={err}"));
                    if last_recorded_at.elapsed() >= E2E_NO_QUORUM_LOG_INTERVAL {
                        self.record(format!("no-quorum wait poll: poll:error={err}"));
                        last_recorded_at = tokio::time::Instant::now();
                    }
                    tokio::time::sleep(E2E_NO_QUORUM_RETRY_INTERVAL).await;
                    continue;
                }
            };
            let (sql_roles, sql_errors) = self
                .cluster_sql_roles_best_effort_with_timeout(E2E_NO_QUORUM_OBSERVATION_TIMEOUT)
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
            if last_recorded_at.elapsed() >= E2E_NO_QUORUM_LOG_INTERVAL {
                if let Some(observation) = last_observation.as_deref() {
                    self.record(format!("no-quorum wait poll: {observation}"));
                }
                last_recorded_at = tokio::time::Instant::now();
            }
            tokio::time::sleep(E2E_NO_QUORUM_RETRY_INTERVAL).await;
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

    fn wipe_node_data_dir(&mut self, node_id: &str) -> Result<(), WorkerError> {
        let Some(node) = self.node_by_id(node_id) else {
            return Err(WorkerError::Message(format!(
                "unknown node for data-dir wipe: {node_id}"
            )));
        };
        if node.data_dir.as_os_str().is_empty() {
            return Err(WorkerError::Message(format!(
                "cannot wipe empty data dir for {node_id}"
            )));
        }
        if node.data_dir.exists() {
            fs::remove_dir_all(&node.data_dir).map_err(|err| {
                WorkerError::Message(format!(
                    "remove data dir failed for {node_id} at {}: {err}",
                    node.data_dir.display()
                ))
            })?;
        }
        fs::create_dir_all(&node.data_dir).map_err(|err| {
            WorkerError::Message(format!(
                "recreate data dir failed for {node_id} at {}: {err}",
                node.data_dir.display()
            ))
        })?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            fs::set_permissions(&node.data_dir, fs::Permissions::from_mode(0o700)).map_err(
                |err| {
                    WorkerError::Message(format!(
                        "set data dir permissions failed for {node_id} at {}: {err}",
                        node.data_dir.display()
                    ))
                },
            )?;
        }
        self.record(format!(
            "wiped postgres data dir for {node_id} at {}",
            node.data_dir.display()
        ));
        Ok(())
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
            etcd_cluster
                .shutdown_member(&member_name)
                .await
                .map_err(|err| {
                    WorkerError::Message(format!("failed to stop etcd member {member_name}: {err}"))
                })?;
            stopped.push(member_name);
        }

        Ok(stopped)
    }

    async fn restore_etcd_members(&mut self, member_names: &[String]) -> Result<(), WorkerError> {
        if member_names.is_empty() {
            return Err(WorkerError::Message(
                "cannot restore etcd members: no member names provided".to_string(),
            ));
        }
        let Some(etcd_cluster) = self.etcd.as_mut() else {
            return Err(WorkerError::Message(
                "cannot restore etcd members: cluster is not running".to_string(),
            ));
        };
        etcd_cluster
            .restart_members(member_names)
            .await
            .map_err(|err| {
                WorkerError::Message(format!(
                    "failed to restore etcd members {}: {err}",
                    member_names.join(",")
                ))
            })
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
        self.runtime_nodes.shutdown_all().await;

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
        (Some(run_err), Ok((timeline, summary)), Err(shutdown_err)) => {
            Err(WorkerError::Message(format!(
                "{run_err}; shutdown failed: {shutdown_err}; timeline: {}; summary: {}",
                timeline.display(),
                summary.display()
            )))
        }
        (Some(run_err), Err(artifact_err), Err(shutdown_err)) => {
            Err(WorkerError::Message(format!(
                "{run_err}; stress artifact write failed: {artifact_err}; shutdown failed: {shutdown_err}"
            )))
        }
        (None, Err(artifact_err), Ok(())) => Err(WorkerError::Message(format!(
            "stress artifact write failed: {artifact_err}"
        ))),
    }
}

fn finalize_timeline_scenario_result(
    run_result: Result<(), WorkerError>,
    artifact_path: Result<PathBuf, WorkerError>,
    shutdown_result: Result<(), WorkerError>,
) -> Result<(), WorkerError> {
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
}

async fn stop_etcd_majority_and_wait_failsafe_strict_all_nodes(
    fixture: &mut ClusterFixture,
    stop_count: usize,
    timeout: Duration,
) -> Result<(Vec<String>, u64), WorkerError> {
    fixture.record("no-quorum: stop etcd majority");
    let stopped_members = fixture.stop_etcd_majority(stop_count).await?;
    fixture.record(format!(
        "no-quorum: etcd members stopped: {}",
        stopped_members.join(",")
    ));

    fixture.wait_for_all_nodes_failsafe(timeout).await?;
    fixture.record("no-quorum: fail-safe observed on all nodes");
    Ok((stopped_members, ha_e2e::util::unix_now()?.0))
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
                    timeout: E2E_PRIMARY_CONVERGENCE_TIMEOUT,
                    excluded_primary: None,
                    required_consecutive: 5,
                    fallback_timeout: E2E_PRIMARY_CONVERGENCE_FALLBACK_TIMEOUT,
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
            .assert_no_dual_primary_window(E2E_SHORT_NO_DUAL_PRIMARY_WINDOW)
            .await?;

        fixture.record("unassisted failover SQL pre-check: create table and insert pre-failure row");
        fixture
            .run_sql_on_node_with_retry(
                &bootstrap_primary,
                "CREATE TABLE IF NOT EXISTS ha_unassisted_failover_proof (id INTEGER PRIMARY KEY, payload TEXT NOT NULL)",
                E2E_SQL_REPLICATION_ASSERT_TIMEOUT,
            )
            .await?;
        fixture
            .run_sql_on_node_with_retry(
                &bootstrap_primary,
                "INSERT INTO ha_unassisted_failover_proof (id, payload) VALUES (1, 'before') ON CONFLICT (id) DO UPDATE SET payload = EXCLUDED.payload",
                E2E_SQL_REPLICATION_ASSERT_TIMEOUT,
            )
            .await?;
        let pre_rows_raw = fixture
            .run_sql_on_node_with_retry(
                &bootstrap_primary,
                "SELECT id::text || ':' || payload FROM ha_unassisted_failover_proof ORDER BY id",
                E2E_SQL_REPLICATION_ASSERT_TIMEOUT,
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
                    E2E_SQL_REPLICATION_ASSERT_TIMEOUT,
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
                E2E_API_READINESS_TIMEOUT,
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
                    .wait_for_primary_change(
                        &bootstrap_primary,
                        E2E_PRIMARY_CONVERGENCE_FALLBACK_TIMEOUT,
                    )
                    .await?
            }
        };
        fixture
            .assert_no_dual_primary_window(E2E_LONG_NO_DUAL_PRIMARY_WINDOW)
            .await?;
        fixture.record(
            "unassisted failover recovery: confirm SQL-visible primary after API recovery",
        );
        let sql_confirmed_primary = fixture
            .wait_for_stable_primary_via_sql(
                E2E_PRIMARY_CONVERGENCE_TIMEOUT,
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
            .poll_node_ha_states_best_effort_with_timeout(E2E_SHORT_NO_DUAL_PRIMARY_WINDOW)
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

pub async fn e2e_multi_node_custom_postgres_role_names_survive_bootstrap_and_rewind(
) -> Result<(), WorkerError> {
    ha_e2e::util::run_with_local_set(async {
    let mut fixture = ClusterFixture::start_with_config(ha_e2e::TestConfig {
        test_name: "ha-e2e-custom-postgres-roles".to_string(),
        cluster_name: "cluster-e2e-custom-postgres-roles".to_string(),
        scope: "scope-ha-e2e-custom-postgres-roles".to_string(),
        node_count: 3,
        namespace: None,
        etcd_members: vec![
            "etcd-a".to_string(),
            "etcd-b".to_string(),
            "etcd-c".to_string(),
        ],
        recovery_binary_overrides: BTreeMap::new(),
        postgres_roles: Some(ha_e2e::PostgresRoleOverrides {
            replicator_username: "replicator_custom".to_string(),
            replicator_password: "replicator-secret".to_string(),
            rewinder_username: "rewinder_custom".to_string(),
            rewinder_password: "rewinder-secret".to_string(),
        }),
        mode: ha_e2e::Mode::Plain,
        timeouts: ha_e2e::TimeoutConfig {
            command_timeout: E2E_COMMAND_TIMEOUT,
            command_kill_wait_timeout: E2E_COMMAND_KILL_WAIT_TIMEOUT,
            http_step_timeout: E2E_HTTP_STEP_TIMEOUT,
            api_readiness_timeout: E2E_API_READINESS_TIMEOUT,
            bootstrap_primary_timeout: E2E_BOOTSTRAP_PRIMARY_TIMEOUT,
            scenario_timeout: E2E_SCENARIO_TIMEOUT,
        },
    })
    .await?;
    let mut phase_history: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    let run_result = match tokio::time::timeout(E2E_SCENARIO_TIMEOUT, async {
        fixture.record("custom-role bootstrap: wait for stable primary");
        let bootstrap_primary = fixture
            .wait_for_stable_primary_resilient(
                StablePrimaryWaitPlan {
                    context: "custom-role bootstrap stable-primary",
                    timeout: E2E_PRIMARY_CONVERGENCE_TIMEOUT,
                    excluded_primary: None,
                    required_consecutive: 5,
                    fallback_timeout: E2E_PRIMARY_CONVERGENCE_FALLBACK_TIMEOUT,
                    fallback_required_consecutive: 2,
                    min_observed_nodes: 2,
                },
                &mut phase_history,
            )
            .await?;
        fixture.record(format!(
            "custom-role bootstrap success: primary={bootstrap_primary}"
        ));
        fixture
            .assert_no_dual_primary_window(E2E_SHORT_NO_DUAL_PRIMARY_WINDOW)
            .await?;

        fixture.record("custom-role bootstrap proof: create table and seed row");
        fixture
            .run_sql_on_node_with_retry(
                &bootstrap_primary,
                "CREATE TABLE IF NOT EXISTS ha_custom_role_rewind_proof (id INTEGER PRIMARY KEY, payload TEXT NOT NULL)",
                E2E_SQL_REPLICATION_ASSERT_TIMEOUT,
            )
            .await?;
        fixture
            .run_sql_on_node_with_retry(
                &bootstrap_primary,
                "INSERT INTO ha_custom_role_rewind_proof (id, payload) VALUES (1, 'before-failover') ON CONFLICT (id) DO UPDATE SET payload = EXCLUDED.payload",
                E2E_SQL_REPLICATION_ASSERT_TIMEOUT,
            )
            .await?;
        let expected_pre_rows = vec!["1:before-failover".to_string()];
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
                    "SELECT id::text || ':' || payload FROM ha_custom_role_rewind_proof ORDER BY id",
                    expected_pre_rows.as_slice(),
                    E2E_SQL_REPLICATION_ASSERT_TIMEOUT,
                )
                .await?;
            fixture.record(format!(
                "custom-role bootstrap proof replicated to {replica_id}"
            ));
        }

        fixture.record(format!(
            "custom-role failover injection: stop postgres on {bootstrap_primary}"
        ));
        fixture.stop_postgres_for_node(&bootstrap_primary).await?;
        let failover_primary = match fixture
            .wait_for_stable_primary_best_effort(
                E2E_API_READINESS_TIMEOUT,
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
                    "custom-role failover stable-primary wait failed after forced stop: {wait_err}; retrying with relaxed primary-change detection"
                ));
                fixture
                    .wait_for_primary_change(
                        &bootstrap_primary,
                        E2E_PRIMARY_CONVERGENCE_FALLBACK_TIMEOUT,
                    )
                    .await?
            }
        };
        fixture
            .assert_no_dual_primary_window(E2E_LONG_NO_DUAL_PRIMARY_WINDOW)
            .await?;
        let failover_primary = fixture
            .wait_for_stable_primary_via_sql(
                E2E_PRIMARY_CONVERGENCE_TIMEOUT,
                Some(&bootstrap_primary),
                2,
                1,
            )
            .await
            .unwrap_or(failover_primary);
        ClusterFixture::assert_phase_history_contains_failover(
            &phase_history,
            &bootstrap_primary,
            &failover_primary,
        )?;

        fixture.record(format!(
            "custom-role recovery proof: insert post-failover row on {failover_primary}"
        ));
        fixture
            .run_sql_on_node_with_retry(
                &failover_primary,
                "INSERT INTO ha_custom_role_rewind_proof (id, payload) VALUES (2, 'after-failover') ON CONFLICT (id) DO UPDATE SET payload = EXCLUDED.payload",
                Duration::from_secs(45),
            )
            .await?;
        let expected_post_rows =
            vec!["1:before-failover".to_string(), "2:after-failover".to_string()];
        fixture
            .wait_for_rows_on_node(
                &bootstrap_primary,
                "SELECT id::text || ':' || payload FROM ha_custom_role_rewind_proof ORDER BY id",
                expected_post_rows.as_slice(),
                Duration::from_secs(90),
            )
            .await?;
        fixture.record(format!(
            "custom-role rewind proof succeeded: former_primary={bootstrap_primary} rejoined with post-failover rows from {failover_primary}"
        ));
        Ok(())
    })
    .await
    {
        Ok(run_result) => run_result,
        Err(_) => {
            fixture.record(format!(
                "custom-role scenario timed out after {}s",
                E2E_SCENARIO_TIMEOUT.as_secs()
            ));
            Err(WorkerError::Message(format!(
                "custom-role scenario timed out after {}s",
                E2E_SCENARIO_TIMEOUT.as_secs()
            )))
        }
    };

    let artifact_path = fixture.write_timeline_artifact("ha-e2e-custom-postgres-roles");
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

pub async fn e2e_multi_node_clone_failure_recovers_after_fault_removed() -> Result<(), WorkerError>
{
    ha_e2e::util::run_with_local_set(async {
        let scenario_name = "ha-e2e-clone-failure-recovers-after-fix";
        let namespace = pgtuskmaster_rust::test_harness::namespace::create_namespace(
            scenario_name,
        )
        .map_err(|err| {
            WorkerError::Message(format!(
                "create namespace failed for {scenario_name}: {err}"
            ))
        })?;
        let real_binaries =
            pgtuskmaster_rust::test_harness::binaries::require_pg16_process_binaries_for_real_tests(
            )
            .map_err(|err| {
                WorkerError::Message(format!(
                    "load real postgres binaries failed for {scenario_name}: {err}"
                ))
            })?;
        let basebackup_wrapper = create_failure_wrapper_for_namespace(
            &namespace.root_dir,
            scenario_name,
            "node-3",
            RecoveryBinaryKind::PgBasebackup,
            real_binaries.pg_basebackup.as_path(),
        )?;
        set_failure_wrapper_enabled(&basebackup_wrapper, false)?;

        let mut recovery_binary_overrides = BTreeMap::new();
        recovery_binary_overrides.insert(
            "node-3".to_string(),
            ha_e2e::RecoveryBinaryOverrides {
                pg_basebackup: Some(basebackup_wrapper.wrapper_path.clone()),
                pg_rewind: None,
            },
        );
        let mut fixture = ClusterFixture::start_with_config(ha_e2e::TestConfig {
            test_name: scenario_name.to_string(),
            cluster_name: "cluster-e2e-clone-failure".to_string(),
            scope: "scope-ha-e2e-clone-failure".to_string(),
            node_count: 3,
            namespace: Some(namespace),
            etcd_members: vec![
                "etcd-a".to_string(),
                "etcd-b".to_string(),
                "etcd-c".to_string(),
            ],
            recovery_binary_overrides,
            postgres_roles: None,
            mode: ha_e2e::Mode::Plain,
            timeouts: ha_e2e::TimeoutConfig {
                command_timeout: E2E_COMMAND_TIMEOUT,
                command_kill_wait_timeout: E2E_COMMAND_KILL_WAIT_TIMEOUT,
                http_step_timeout: E2E_HTTP_STEP_TIMEOUT,
                api_readiness_timeout: E2E_API_READINESS_TIMEOUT,
                bootstrap_primary_timeout: E2E_BOOTSTRAP_PRIMARY_TIMEOUT,
                scenario_timeout: E2E_SCENARIO_TIMEOUT,
            },
        })
        .await?;
        let mut phase_history: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

        let run_result = match tokio::time::timeout(E2E_SCENARIO_TIMEOUT, async {
            fixture.record(format!(
                "{scenario_name}: basebackup wrapper ready at {}",
                basebackup_wrapper.wrapper_path.display()
            ));
            let bootstrap_primary = fixture
                .wait_for_stable_primary_resilient(
                    StablePrimaryWaitPlan {
                        context: "clone-failure bootstrap stable-primary",
                        timeout: E2E_PRIMARY_CONVERGENCE_TIMEOUT,
                        excluded_primary: None,
                        required_consecutive: 5,
                        fallback_timeout: E2E_PRIMARY_CONVERGENCE_FALLBACK_TIMEOUT,
                        fallback_required_consecutive: 2,
                        min_observed_nodes: 3,
                    },
                    &mut phase_history,
                )
                .await?;
            fixture.record(format!(
                "{scenario_name}: bootstrap primary={bootstrap_primary}"
            ));
            fixture
                .wait_for_queryable_nodes(&bootstrap_primary, 3, E2E_PRIMARY_CONVERGENCE_TIMEOUT)
                .await?;
            fixture
                .wait_for_node_sql_role("node-3", "replica", E2E_PRIMARY_CONVERGENCE_TIMEOUT)
                .await?;
            fixture
                .assert_no_dual_primary_window(E2E_SHORT_NO_DUAL_PRIMARY_WINDOW)
                .await?;

            let healthy_replica = fixture
                .nodes
                .iter()
                .find_map(|node| {
                    (node.id.as_str() != bootstrap_primary.as_str() && node.id.as_str() != "node-3")
                        .then_some(node.id.clone())
                })
                .ok_or_else(|| {
                    WorkerError::Message(format!(
                        "{scenario_name}: expected a non-failing healthy replica"
                    ))
                })?;
            fixture.record(format!(
                "{scenario_name}: healthy replica before failure={healthy_replica}"
            ));

            let table_name = fixture.proof_table_name("ha_clone_failure_proof")?;
            fixture.create_proof_table(&bootstrap_primary, &table_name).await?;
            fixture
                .insert_proof_row(&bootstrap_primary, &table_name, 1, "before-failure")
                .await?;
            fixture
                .wait_for_proof_rows_on_all_nodes(
                    &table_name,
                    &["1:before-failure".to_string()],
                    E2E_SQL_REPLICATION_ASSERT_TIMEOUT,
                )
                .await?;
            set_failure_wrapper_enabled(&basebackup_wrapper, true)?;
            fixture.record(format!(
                "{scenario_name}: enabled failure wrapper {}",
                basebackup_wrapper.wrapper_path.display()
            ));
            fixture.record(format!(
                "{scenario_name}: forcing node-3 into a fresh clone attempt"
            ));
            fixture.stop_postgres_for_node("node-3").await?;
            fixture.wipe_node_data_dir("node-3")?;
            fixture
                .insert_proof_row(&bootstrap_primary, &table_name, 2, "during-failure")
                .await?;
            fixture
                .wait_for_proof_rows_on_nodes(
                    &[bootstrap_primary.clone(), healthy_replica.clone()],
                    &table_name,
                    &[
                        "1:before-failure".to_string(),
                        "2:during-failure".to_string(),
                    ],
                    E2E_SQL_REPLICATION_ASSERT_TIMEOUT,
                )
                .await?;
            fixture
                .assert_node_not_queryable_for_window("node-3", Duration::from_secs(5))
                .await?;
            fixture
                .assert_no_dual_primary_window(E2E_LONG_NO_DUAL_PRIMARY_WINDOW)
                .await?;
            fixture.assert_failure_wrapper_invoked(
                scenario_name,
                &basebackup_wrapper,
            )?;

            fixture.disable_failure_wrapper(scenario_name, &basebackup_wrapper)?;
            fixture.record(format!(
                "{scenario_name}: restarting node-3 runtime so startup re-enters clone after removing the pg_basebackup blocker"
            ));
            fixture.stop_runtime_process_for_node("node-3").await?;
            fixture.stop_postgres_for_node("node-3").await?;
            fixture.wipe_node_data_dir("node-3")?;
            fixture.restart_runtime_process_for_node("node-3").await?;
            let recovered_nodes = fixture
                .wait_for_queryable_nodes("node-3", 3, Duration::from_secs(120))
                .await?;
            if recovered_nodes.len() < 3 {
                return Err(WorkerError::Message(format!(
                    "{scenario_name}: expected all three nodes queryable after recovery, got {recovered_nodes:?}"
                )));
            }
            fixture
                .wait_for_node_sql_role("node-3", "replica", Duration::from_secs(120))
                .await?;
            fixture
                .insert_proof_row(&bootstrap_primary, &table_name, 3, "after-recovery")
                .await?;
            fixture
                .wait_for_proof_rows_on_all_nodes(
                    &table_name,
                    &[
                        "1:before-failure".to_string(),
                        "2:during-failure".to_string(),
                        "3:after-recovery".to_string(),
                    ],
                    Duration::from_secs(120),
                )
                .await?;
            fixture.record(format!(
                "{scenario_name}: node-3 rejoined cleanly after removing the pg_basebackup blocker"
            ));
            Ok(())
        })
        .await
        {
            Ok(run_result) => run_result,
            Err(_) => {
                fixture.record(format!(
                    "{scenario_name}: timed out after {}s",
                    E2E_SCENARIO_TIMEOUT.as_secs()
                ));
                Err(WorkerError::Message(format!(
                    "{scenario_name}: timed out after {}s",
                    E2E_SCENARIO_TIMEOUT.as_secs()
                )))
            }
        };

        let artifact_path = fixture.write_timeline_artifact(scenario_name);
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
            (Err(run_err), Err(artifact_err), Err(shutdown_err)) => Err(WorkerError::Message(
                format!("{run_err}; timeline write failed: {artifact_err}; shutdown failed: {shutdown_err}"),
            )),
            (Ok(()), Err(artifact_err), Ok(())) => Err(WorkerError::Message(format!(
                "timeline write failed: {artifact_err}"
            ))),
        }
    })
    .await
}

pub async fn e2e_multi_node_rewind_failure_falls_back_to_basebackup() -> Result<(), WorkerError> {
    ha_e2e::util::run_with_local_set(async {
        let scenario_name = "ha-e2e-rewind-failure-fallback";
        let namespace = pgtuskmaster_rust::test_harness::namespace::create_namespace(
            scenario_name,
        )
        .map_err(|err| {
            WorkerError::Message(format!(
                "create namespace failed for {scenario_name}: {err}"
            ))
        })?;
        let real_binaries =
            pgtuskmaster_rust::test_harness::binaries::require_pg16_process_binaries_for_real_tests(
            )
            .map_err(|err| {
                WorkerError::Message(format!(
                    "load real postgres binaries failed for {scenario_name}: {err}"
                ))
            })?;
        let rewind_wrapper = create_failure_wrapper_for_namespace(
            &namespace.root_dir,
            scenario_name,
            "node-1",
            RecoveryBinaryKind::PgRewind,
            real_binaries.pg_rewind.as_path(),
        )?;
        set_failure_wrapper_enabled(&rewind_wrapper, true)?;

        let mut recovery_binary_overrides = BTreeMap::new();
        recovery_binary_overrides.insert(
            "node-1".to_string(),
            ha_e2e::RecoveryBinaryOverrides {
                pg_basebackup: None,
                pg_rewind: Some(rewind_wrapper.wrapper_path.clone()),
            },
        );
        let mut fixture = ClusterFixture::start_with_config(ha_e2e::TestConfig {
            test_name: scenario_name.to_string(),
            cluster_name: "cluster-e2e-rewind-failure".to_string(),
            scope: "scope-ha-e2e-rewind-failure".to_string(),
            node_count: 3,
            namespace: Some(namespace),
            etcd_members: vec![
                "etcd-a".to_string(),
                "etcd-b".to_string(),
                "etcd-c".to_string(),
            ],
            recovery_binary_overrides,
            postgres_roles: None,
            mode: ha_e2e::Mode::Plain,
            timeouts: ha_e2e::TimeoutConfig {
                command_timeout: E2E_COMMAND_TIMEOUT,
                command_kill_wait_timeout: E2E_COMMAND_KILL_WAIT_TIMEOUT,
                http_step_timeout: E2E_HTTP_STEP_TIMEOUT,
                api_readiness_timeout: E2E_API_READINESS_TIMEOUT,
                bootstrap_primary_timeout: E2E_BOOTSTRAP_PRIMARY_TIMEOUT,
                scenario_timeout: E2E_SCENARIO_TIMEOUT,
            },
        })
        .await?;
        let mut phase_history: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

        let run_result = match tokio::time::timeout(E2E_SCENARIO_TIMEOUT, async {
            fixture.record(format!(
                "{scenario_name}: rewind wrapper active at {}",
                rewind_wrapper.wrapper_path.display()
            ));
            let bootstrap_primary = fixture
                .wait_for_stable_primary_resilient(
                    StablePrimaryWaitPlan {
                        context: "rewind-failure bootstrap stable-primary",
                        timeout: E2E_PRIMARY_CONVERGENCE_TIMEOUT,
                        excluded_primary: None,
                        required_consecutive: 5,
                        fallback_timeout: E2E_PRIMARY_CONVERGENCE_FALLBACK_TIMEOUT,
                        fallback_required_consecutive: 2,
                        min_observed_nodes: 3,
                    },
                    &mut phase_history,
                )
                .await?;
            if bootstrap_primary != "node-1" {
                return Err(WorkerError::Message(format!(
                    "{scenario_name}: expected node-1 to bootstrap as primary, observed {bootstrap_primary}"
                )));
            }
            fixture
                .wait_for_queryable_nodes(&bootstrap_primary, 3, E2E_PRIMARY_CONVERGENCE_TIMEOUT)
                .await?;
            fixture
                .assert_no_dual_primary_window(E2E_SHORT_NO_DUAL_PRIMARY_WINDOW)
                .await?;

            let table_name = fixture.proof_table_name("ha_rewind_failure_proof")?;
            fixture.create_proof_table(&bootstrap_primary, &table_name).await?;
            fixture
                .insert_proof_row(&bootstrap_primary, &table_name, 1, "before-failover")
                .await?;
            fixture
                .wait_for_proof_rows_on_all_nodes(
                    &table_name,
                    &["1:before-failover".to_string()],
                    E2E_SQL_REPLICATION_ASSERT_TIMEOUT,
                )
                .await?;

            fixture.record(format!(
                "{scenario_name}: forcing failover by stopping postgres on {bootstrap_primary}"
            ));
            fixture.stop_postgres_for_node(&bootstrap_primary).await?;
            let failover_primary = match fixture
                .wait_for_stable_primary_best_effort(
                    E2E_API_READINESS_TIMEOUT,
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
                        "{scenario_name}: primary wait failed after forced stop: {wait_err}; retrying via primary-change helper"
                    ));
                    fixture
                        .wait_for_primary_change(
                            &bootstrap_primary,
                            E2E_PRIMARY_CONVERGENCE_FALLBACK_TIMEOUT,
                        )
                        .await?
                }
            };
            fixture
                .assert_no_dual_primary_window(E2E_LONG_NO_DUAL_PRIMARY_WINDOW)
                .await?;
            fixture
                .assert_former_primary_demoted_or_unreachable_after_transition(
                    &bootstrap_primary,
                )
                .await?;
            ClusterFixture::assert_phase_history_contains_failover(
                &phase_history,
                &bootstrap_primary,
                &failover_primary,
            )?;
            fixture.assert_failure_wrapper_invoked(
                scenario_name,
                &rewind_wrapper,
            )?;

            fixture
                .insert_proof_row(&failover_primary, &table_name, 2, "after-failover")
                .await?;
            fixture.record(format!(
                "{scenario_name}: waiting for former primary to rejoin while rewind failure remains enabled"
            ));
            fixture
                .wait_for_rows_on_node(
                    &bootstrap_primary,
                    format!(
                        "SELECT id::text || ':' || payload FROM {table_name} ORDER BY id"
                    )
                    .as_str(),
                    &[
                        "1:before-failover".to_string(),
                        "2:after-failover".to_string(),
                    ],
                    Duration::from_secs(120),
                )
                .await?;
            fixture
                .wait_for_node_sql_role(&bootstrap_primary, "replica", Duration::from_secs(120))
                .await?;
            if !rewind_wrapper.fail_enabled_marker.exists() {
                return Err(WorkerError::Message(format!(
                    "{scenario_name}: rewind failure marker unexpectedly disappeared before recovery completed"
                )));
            }
            fixture.record(format!(
                "{scenario_name}: former primary rejoined as replica after a failed pg_rewind attempt"
            ));
            Ok(())
        })
        .await
        {
            Ok(run_result) => run_result,
            Err(_) => {
                fixture.record(format!(
                    "{scenario_name}: timed out after {}s",
                    E2E_SCENARIO_TIMEOUT.as_secs()
                ));
                Err(WorkerError::Message(format!(
                    "{scenario_name}: timed out after {}s",
                    E2E_SCENARIO_TIMEOUT.as_secs()
                )))
            }
        };

        let artifact_path = fixture.write_timeline_artifact(scenario_name);
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
            (Err(run_err), Err(artifact_err), Err(shutdown_err)) => Err(WorkerError::Message(
                format!("{run_err}; timeline write failed: {artifact_err}; shutdown failed: {shutdown_err}"),
            )),
            (Ok(()), Err(artifact_err), Ok(())) => Err(WorkerError::Message(format!(
                "timeline write failed: {artifact_err}"
            ))),
        }
    })
    .await
}

pub async fn e2e_multi_node_cli_primary_and_replicas_follow_switchover() -> Result<(), WorkerError>
{
    ha_e2e::util::run_with_local_set(async {
        let mut fixture = ClusterFixture::start(3).await?;
        let mut phase_history: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

        let run_result = match tokio::time::timeout(E2E_SCENARIO_TIMEOUT, async {
            fixture.record("cli connect bootstrap: wait for stable primary");
            let bootstrap_primary = fixture
                .wait_for_stable_primary_resilient(
                    StablePrimaryWaitPlan {
                        context: "cli connect bootstrap stable-primary",
                        timeout: E2E_PRIMARY_CONVERGENCE_TIMEOUT,
                        excluded_primary: None,
                        required_consecutive: 5,
                        fallback_timeout: E2E_PRIMARY_CONVERGENCE_FALLBACK_TIMEOUT,
                        fallback_required_consecutive: 2,
                        min_observed_nodes: 2,
                    },
                    &mut phase_history,
                )
                .await?;
            fixture.record(format!(
                "cli connect bootstrap success: primary={bootstrap_primary}"
            ));

            let bootstrap_primary_output = fixture
                .run_observe_cli_command_via_node(&bootstrap_primary, &["primary"], false)
                .await?;
            let expected_bootstrap_primary = format!(
                "host=127.0.0.1 port={} user=postgres dbname=postgres",
                fixture.postgres_port_by_id(&bootstrap_primary)?
            );
            if bootstrap_primary_output.trim() != expected_bootstrap_primary {
                return Err(WorkerError::Message(format!(
                    "bootstrap primary CLI output mismatch: expected `{expected_bootstrap_primary}`, got `{}`",
                    bootstrap_primary_output.trim()
                )));
            }

            let bootstrap_replicas_output = fixture
                .run_observe_cli_command_via_node_with_retry(
                    &bootstrap_primary,
                    &["replicas"],
                    false,
                    E2E_API_READINESS_TIMEOUT,
                )
                .await?;
            let mut expected_bootstrap_replicas = fixture
                .nodes
                .iter()
                .filter(|node| node.id != bootstrap_primary)
                .map(|node| {
                    format!(
                        "host=127.0.0.1 port={} user=postgres dbname=postgres",
                        node.pg_port
                    )
                })
                .collect::<Vec<_>>();
            expected_bootstrap_replicas.sort();
            let mut actual_bootstrap_replicas = bootstrap_replicas_output
                .lines()
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>();
            actual_bootstrap_replicas.sort();
            if actual_bootstrap_replicas != expected_bootstrap_replicas {
                return Err(WorkerError::Message(format!(
                    "bootstrap replicas CLI output mismatch: expected {:?}, got {:?}",
                    expected_bootstrap_replicas, actual_bootstrap_replicas
                )));
            }

            fixture.record("cli connect switchover: request planned switchover via CLI");
            fixture.request_switchover_via_cli().await?;
            let switchover_primary = match fixture
                .wait_for_stable_primary_best_effort(
                    E2E_API_READINESS_TIMEOUT,
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
                        "cli connect stable-primary wait failed after switchover: {wait_err}; retrying with relaxed primary-change detection"
                    ));
                    fixture
                        .wait_for_primary_change(
                            &bootstrap_primary,
                            E2E_PRIMARY_CONVERGENCE_FALLBACK_TIMEOUT,
                        )
                        .await?
                }
            };
            fixture.record(format!(
                "cli connect switchover success: new_primary={switchover_primary}"
            ));

            let switchover_primary_output = fixture
                .run_observe_cli_command_via_node(&switchover_primary, &["primary"], false)
                .await?;
            let expected_switchover_primary = format!(
                "host=127.0.0.1 port={} user=postgres dbname=postgres",
                fixture.postgres_port_by_id(&switchover_primary)?
            );
            if switchover_primary_output.trim() != expected_switchover_primary {
                return Err(WorkerError::Message(format!(
                    "switchover primary CLI output mismatch: expected `{expected_switchover_primary}`, got `{}`",
                    switchover_primary_output.trim()
                )));
            }

            let switchover_replicas_output = fixture
                .run_observe_cli_command_via_node_with_retry(
                    &switchover_primary,
                    &["replicas"],
                    false,
                    E2E_API_READINESS_TIMEOUT,
                )
                .await?;
            let expected_switchover_replicas = fixture
                .nodes
                .iter()
                .filter(|node| node.id != switchover_primary)
                .map(|node| {
                    format!(
                        "host=127.0.0.1 port={} user=postgres dbname=postgres",
                        node.pg_port
                    )
                })
                .collect::<Vec<_>>();
            let mut actual_switchover_replicas = switchover_replicas_output
                .lines()
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>();
            actual_switchover_replicas.sort();
            if actual_switchover_replicas.is_empty() {
                return Err(WorkerError::Message(format!(
                    "switchover replicas CLI output was empty; expected at least one sampled replica from {:?}",
                    expected_switchover_replicas
                )));
            }
            if actual_switchover_replicas
                .iter()
                .any(|line| !expected_switchover_replicas.contains(line))
            {
                return Err(WorkerError::Message(format!(
                    "switchover replicas CLI output contained unexpected targets: expected subset of {:?}, got {:?}",
                    expected_switchover_replicas, actual_switchover_replicas
                )));
            }

            Ok(())
        })
        .await
        {
            Ok(run_result) => run_result,
            Err(_) => {
                fixture.record(format!(
                    "cli connect scenario timed out after {}s",
                    E2E_SCENARIO_TIMEOUT.as_secs()
                ));
                Err(WorkerError::Message(format!(
                    "cli connect scenario timed out after {}s",
                    E2E_SCENARIO_TIMEOUT.as_secs()
                )))
            }
        };

        let artifact_path =
            fixture.write_timeline_artifact("ha-e2e-cli-primary-and-replicas-follow-switchover");
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
            (Err(run_err), Err(artifact_err), Err(shutdown_err)) => {
                Err(WorkerError::Message(format!(
                    "{run_err}; timeline write failed: {artifact_err}; shutdown failed: {shutdown_err}"
                )))
            }
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
            run_interval_ms: E2E_STRESS_WORKLOAD_RUN_INTERVAL_MS,
        };
        let table_name = sanitize_sql_identifier(workload_spec.table_name.as_str());

        fixture.record("stress switchover bootstrap: wait for stable primary");
        let bootstrap_primary = fixture
            .wait_for_stable_primary_resilient(
                StablePrimaryWaitPlan {
                    context: "stress switchover bootstrap stable-primary",
                    timeout: E2E_PRIMARY_CONVERGENCE_FALLBACK_TIMEOUT,
                    excluded_primary: None,
                    required_consecutive: 3,
                    fallback_timeout: E2E_PRIMARY_CONVERGENCE_FALLBACK_TIMEOUT,
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
        tokio::time::sleep(E2E_STRESS_WORKLOAD_SETTLE_WAIT).await;

        fixture.record("stress switchover: trigger API switchover while workload is active");
        fixture.request_switchover_via_cli().await?;
        let ha_stats = fixture
            .sample_ha_states_window(
                E2E_STRESS_SHORT_OBSERVATION_WINDOW,
                E2E_STRESS_SAMPLE_INTERVAL,
                80,
            )
            .await?;
        let workload = fixture
            .stop_sql_workload_and_collect(workload_handle, E2E_STRESS_WORKLOAD_STOP_TIMEOUT)
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
            .assert_no_dual_primary_window(E2E_LONG_NO_DUAL_PRIMARY_WINDOW)
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
                E2E_POST_TRANSITION_SQL_TIMEOUT,
            )
            .await?;

        let primary_row_count = fixture
            .assert_table_key_integrity_on_node(
                &switchover_primary,
                table_name.as_str(),
                1,
                E2E_TABLE_INTEGRITY_TIMEOUT,
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
            run_interval_ms: E2E_STRESS_WORKLOAD_RUN_INTERVAL_MS,
        };
        let table_name = sanitize_sql_identifier(workload_spec.table_name.as_str());

        fixture.record("stress failover bootstrap: wait for stable primary");
        let bootstrap_primary = fixture
            .wait_for_stable_primary_resilient(
                StablePrimaryWaitPlan {
                    context: "stress failover bootstrap stable-primary",
                    timeout: E2E_PRIMARY_CONVERGENCE_TIMEOUT,
                    excluded_primary: None,
                    required_consecutive: 5,
                    fallback_timeout: E2E_PRIMARY_CONVERGENCE_FALLBACK_TIMEOUT,
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
        tokio::time::sleep(E2E_STRESS_WORKLOAD_SETTLE_WAIT).await;

        fixture.record(format!(
            "stress failover: stop postgres on bootstrap primary {bootstrap_primary}"
        ));
        fixture.stop_postgres_for_node(&bootstrap_primary).await?;
        let ha_stats = fixture
            .sample_ha_states_window(
                E2E_STRESS_LONG_OBSERVATION_WINDOW,
                E2E_STRESS_SAMPLE_INTERVAL,
                100,
            )
            .await?;
        let workload = fixture
            .stop_sql_workload_and_collect(workload_handle, E2E_STRESS_WORKLOAD_STOP_TIMEOUT)
            .await?;
        if workload.committed_writes == 0 {
            return Err(WorkerError::Message(
                "stress failover workload committed zero writes".to_string(),
            ));
        }
        ClusterFixture::assert_no_split_brain_write_evidence(&workload, &ha_stats)?;
        let failover_primary = match fixture
            .wait_for_stable_primary(
                E2E_LOADED_FAILOVER_TIMEOUT,
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
                    .wait_for_primary_change(
                        &bootstrap_primary,
                        E2E_PRIMARY_CONVERGENCE_FALLBACK_TIMEOUT,
                    )
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
                E2E_POST_TRANSITION_SQL_TIMEOUT,
            )
            .await?;

        let primary_row_count = fixture
            .assert_table_key_integrity_on_node(
                &failover_primary,
                table_name.as_str(),
                1,
                E2E_TABLE_INTEGRITY_TIMEOUT,
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

pub async fn e2e_multi_node_primary_runtime_restart_recovers_without_split_brain(
) -> Result<(), WorkerError> {
    ha_e2e::util::run_with_local_set(async {
        let mut fixture = ClusterFixture::start(3).await?;
        let mut phase_history: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let scenario_name = "ha-e2e-primary-runtime-restart-recovers-without-split-brain";

        let run_result = match tokio::time::timeout(E2E_SCENARIO_TIMEOUT, async {
            fixture.record("runtime restart bootstrap: wait for stable primary");
            let bootstrap_primary = fixture
                .wait_for_stable_primary_resilient(
                    StablePrimaryWaitPlan {
                        context: "runtime restart bootstrap stable-primary",
                        timeout: E2E_PRIMARY_CONVERGENCE_TIMEOUT,
                        excluded_primary: None,
                        required_consecutive: 5,
                        fallback_timeout: E2E_PRIMARY_CONVERGENCE_FALLBACK_TIMEOUT,
                        fallback_required_consecutive: 2,
                        min_observed_nodes: 2,
                    },
                    &mut phase_history,
                )
                .await?;
            let table_name = fixture.proof_table_name("ha_restart_proof")?;
            let expected_pre_rows = vec!["1:before-restart".to_string()];

            fixture.create_proof_table(&bootstrap_primary, table_name.as_str()).await?;
            fixture
                .insert_proof_row(&bootstrap_primary, table_name.as_str(), 1, "before-restart")
                .await?;
            fixture
                .wait_for_proof_rows_on_all_nodes(
                    table_name.as_str(),
                    expected_pre_rows.as_slice(),
                    E2E_SQL_REPLICATION_ASSERT_TIMEOUT,
                )
                .await?;

            let restart_observer_clients = fixture.observed_node_clients()?;
            let ((), ha_stats) = ClusterFixture::observe_no_dual_primary_while_clients(
                    restart_observer_clients,
                    E2E_LONG_NO_DUAL_PRIMARY_WINDOW,
                    128,
                    async {
                        fixture
                            .restart_runtime_process_for_node(&bootstrap_primary)
                            .await
                    },
                )
                .await?;

            let recovered_primary = fixture
                .wait_for_stable_primary_resilient(
                    StablePrimaryWaitPlan {
                        context: "runtime restart recovery stable-primary",
                        timeout: E2E_PRIMARY_CONVERGENCE_TIMEOUT,
                        excluded_primary: None,
                        required_consecutive: 3,
                        fallback_timeout: E2E_PRIMARY_CONVERGENCE_FALLBACK_TIMEOUT,
                        fallback_required_consecutive: 1,
                        min_observed_nodes: 2,
                    },
                    &mut phase_history,
                )
                .await?;

            let sql_primary = fixture
                .wait_for_stable_primary_via_sql(
                    E2E_PRIMARY_CONVERGENCE_TIMEOUT,
                    None,
                    2,
                    2,
                )
                .await?;
            if sql_primary != recovered_primary {
                return Err(WorkerError::Message(format!(
                    "runtime restart primary mismatch between API and SQL convergence: api={recovered_primary} sql={sql_primary}"
                )));
            }
            if recovered_primary != bootstrap_primary {
                fixture
                    .assert_former_primary_demoted_or_unreachable_after_transition(
                        &bootstrap_primary,
                    )
                    .await?;
            }
            let recovered_nodes = fixture
                .wait_for_queryable_nodes(
                    &bootstrap_primary,
                    3,
                    E2E_PRIMARY_CONVERGENCE_TIMEOUT,
                )
                .await?;
            if recovered_nodes.len() < 3 {
                return Err(WorkerError::Message(format!(
                    "{scenario_name}: expected all three nodes queryable after runtime restart recovery, got {recovered_nodes:?}"
                )));
            }
            let bootstrap_expected_role = if recovered_primary == bootstrap_primary {
                "primary"
            } else {
                "replica"
            };
            fixture
                .wait_for_node_sql_role(
                    &bootstrap_primary,
                    bootstrap_expected_role,
                    E2E_PRIMARY_CONVERGENCE_TIMEOUT,
                )
                .await?;

            fixture.record(format!(
                "runtime restart no-dual-primary stats: samples={} max_concurrent_primaries={}",
                ha_stats.sample_count, ha_stats.max_concurrent_primaries
            ));
            fixture
                .insert_proof_row(&recovered_primary, table_name.as_str(), 2, "after-restart")
                .await?;
            let expected_post_rows = vec![
                "1:before-restart".to_string(),
                "2:after-restart".to_string(),
            ];
            let post_restart_validation_nodes = if recovered_primary == bootstrap_primary {
                recovered_nodes.clone()
            } else {
                vec![recovered_primary.clone(), bootstrap_primary.clone()]
            };
            fixture
                .wait_for_proof_rows_on_nodes(
                    post_restart_validation_nodes.as_slice(),
                    table_name.as_str(),
                    expected_post_rows.as_slice(),
                    E2E_TABLE_INTEGRITY_TIMEOUT,
                )
                .await?;
            fixture.record(format!(
                "runtime restart recovery succeeded: bootstrap_primary={bootstrap_primary} recovered_primary={recovered_primary} phase_history={}",
                ClusterFixture::format_phase_history(&phase_history)
            ));
            Ok(())
        })
        .await
        {
            Ok(run_result) => run_result,
            Err(_) => Err(WorkerError::Message(format!(
                "runtime restart scenario timed out after {}s",
                E2E_SCENARIO_TIMEOUT.as_secs()
            ))),
        };

        let artifact_path = fixture.write_timeline_artifact(scenario_name);
        let shutdown_result = fixture.shutdown().await;
        finalize_timeline_scenario_result(run_result, artifact_path, shutdown_result)
    })
    .await
}

pub async fn e2e_multi_node_repeated_leadership_changes_preserve_single_primary(
) -> Result<(), WorkerError> {
    ha_e2e::util::run_with_local_set(async {
        let mut fixture = ClusterFixture::start(3).await?;
        let mut phase_history: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let scenario_name = "ha-e2e-repeated-leadership-changes-preserve-single-primary";

        let run_result = match tokio::time::timeout(E2E_SCENARIO_TIMEOUT, async {
            fixture.record("leadership churn bootstrap: wait for stable primary");
            let bootstrap_primary = fixture
                .wait_for_stable_primary_resilient(
                    StablePrimaryWaitPlan {
                        context: "leadership churn bootstrap stable-primary",
                        timeout: E2E_PRIMARY_CONVERGENCE_TIMEOUT,
                        excluded_primary: None,
                        required_consecutive: 5,
                        fallback_timeout: E2E_PRIMARY_CONVERGENCE_FALLBACK_TIMEOUT,
                        fallback_required_consecutive: 2,
                        min_observed_nodes: 2,
                    },
                    &mut phase_history,
                )
                .await?;

            let churn_observer_clients = fixture.observed_node_clients()?;
            let ((first_successor, final_primary), ha_stats) = ClusterFixture::observe_no_dual_primary_while_clients(
                    churn_observer_clients,
                    Duration::from_secs(30),
                    160,
                    async {
                        fixture.record(format!(
                            "leadership churn first failover: stop postgres on {bootstrap_primary}"
                        ));
                        fixture.stop_postgres_for_node(&bootstrap_primary).await?;
                        let first_successor = fixture
                            .wait_for_stable_primary_resilient(
                                StablePrimaryWaitPlan {
                                    context: "leadership churn first failover stable-primary",
                                    timeout: E2E_PRIMARY_CONVERGENCE_TIMEOUT,
                                    excluded_primary: Some(&bootstrap_primary),
                                    required_consecutive: 3,
                                    fallback_timeout: E2E_PRIMARY_CONVERGENCE_FALLBACK_TIMEOUT,
                                    fallback_required_consecutive: 1,
                                    min_observed_nodes: 1,
                                },
                                &mut phase_history,
                            )
                            .await?;
                        fixture
                            .assert_former_primary_demoted_or_unreachable_after_transition(
                                &bootstrap_primary,
                            )
                            .await?;

                        fixture.record(format!(
                            "leadership churn second failover: stop postgres on {first_successor}"
                        ));
                        fixture.stop_postgres_for_node(&first_successor).await?;
                        let second_successor = fixture
                            .wait_for_stable_primary_resilient(
                                StablePrimaryWaitPlan {
                                    context: "leadership churn second failover stable-primary",
                                    timeout: E2E_PRIMARY_CONVERGENCE_TIMEOUT,
                                    excluded_primary: Some(&first_successor),
                                    required_consecutive: 3,
                                    fallback_timeout: E2E_PRIMARY_CONVERGENCE_FALLBACK_TIMEOUT,
                                    fallback_required_consecutive: 1,
                                    min_observed_nodes: 1,
                                },
                                &mut phase_history,
                            )
                            .await?;
                        fixture
                            .assert_former_primary_demoted_or_unreachable_after_transition(
                                &first_successor,
                            )
                            .await?;
                        Ok((first_successor, second_successor))
                    },
                )
                .await?;

            if final_primary == first_successor {
                return Err(WorkerError::Message(format!(
                    "expected second transition to move leadership away from {first_successor}, but final primary remained unchanged"
                )));
            }
            let ordered_sequence = [
                bootstrap_primary.clone(),
                first_successor.clone(),
                final_primary.clone(),
            ];
            fixture.record(format!(
                "leadership churn succeeded: primary_sequence={} no_dual_samples={} phase_history={}",
                ordered_sequence.join(" -> "),
                ha_stats.sample_count,
                ClusterFixture::format_phase_history(&phase_history)
            ));
            Ok(())
        })
        .await
        {
            Ok(run_result) => run_result,
            Err(_) => Err(WorkerError::Message(format!(
                "leadership churn scenario timed out after {}s",
                E2E_SCENARIO_TIMEOUT.as_secs()
            ))),
        };

        let artifact_path = fixture.write_timeline_artifact(scenario_name);
        let shutdown_result = fixture.shutdown().await;
        finalize_timeline_scenario_result(run_result, artifact_path, shutdown_result)
    })
    .await
}

pub async fn e2e_multi_node_degraded_replica_failover_promotes_only_healthy_target(
) -> Result<(), WorkerError> {
    ha_e2e::util::run_with_local_set(async {
        let mut fixture = ClusterFixture::start(3).await?;
        let mut phase_history: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let scenario_name = "ha-e2e-degraded-replica-failover-promotes-only-healthy-target";

        let run_result = match tokio::time::timeout(E2E_SCENARIO_TIMEOUT, async {
            fixture.record("degraded failover bootstrap: wait for stable primary");
            let bootstrap_primary = fixture
                .wait_for_stable_primary_resilient(
                    StablePrimaryWaitPlan {
                        context: "degraded failover bootstrap stable-primary",
                        timeout: E2E_PRIMARY_CONVERGENCE_TIMEOUT,
                        excluded_primary: None,
                        required_consecutive: 5,
                        fallback_timeout: E2E_PRIMARY_CONVERGENCE_FALLBACK_TIMEOUT,
                        fallback_required_consecutive: 2,
                        min_observed_nodes: 2,
                    },
                    &mut phase_history,
                )
                .await?;
            let replica_ids = fixture.node_ids_excluding(&bootstrap_primary);
            let degraded_replica = replica_ids.first().cloned().ok_or_else(|| {
                WorkerError::Message("missing degraded replica candidate".to_string())
            })?;
            let healthy_failover_target = replica_ids
                .iter()
                .find(|node_id| node_id.as_str() != degraded_replica.as_str())
                .cloned()
                .ok_or_else(|| {
                    WorkerError::Message("missing healthy failover target".to_string())
                })?;
            let table_name = fixture.proof_table_name("ha_degraded_failover")?;

            fixture.create_proof_table(&bootstrap_primary, table_name.as_str()).await?;
            fixture
                .insert_proof_row(&bootstrap_primary, table_name.as_str(), 1, "before-degraded-failover")
                .await?;
            let expected_pre_rows = vec!["1:before-degraded-failover".to_string()];
            fixture
                .wait_for_proof_rows_on_all_nodes(
                    table_name.as_str(),
                    expected_pre_rows.as_slice(),
                    E2E_SQL_REPLICATION_ASSERT_TIMEOUT,
                )
                .await?;

            fixture.record(format!(
                "degraded failover: stop runtime on replica={degraded_replica}"
            ));
            fixture
                .stop_runtime_process_for_node(&degraded_replica)
                .await?;
            fixture.record(format!(
                "degraded failover: stop postgres on runtime-stopped replica={degraded_replica}"
            ));
            fixture.stop_postgres_for_node(&degraded_replica).await?;
            fixture
                .wait_for_member_to_be_ineligible(
                    &bootstrap_primary,
                    &degraded_replica,
                    Duration::from_secs(30),
                )
                .await?;
            let (sql_roles, _sql_errors) = fixture.cluster_sql_roles_best_effort().await?;
            let primary_nodes = sql_roles
                .iter()
                .filter(|(_, role)| role == "primary")
                .map(|(node_id, _)| node_id.clone())
                .collect::<Vec<_>>();
            if primary_nodes != vec![bootstrap_primary.clone()] {
                return Err(WorkerError::Message(format!(
                    "expected only bootstrap primary before degraded failover, observed primaries={primary_nodes:?}"
                )));
            }

            let degraded_observer_clients = fixture.observed_node_clients()?;
            let (promoted_primary, ha_stats) = ClusterFixture::observe_no_dual_primary_while_clients(
                    degraded_observer_clients,
                    E2E_LONG_NO_DUAL_PRIMARY_WINDOW,
                    128,
                    async {
                        fixture.stop_postgres_for_node(&bootstrap_primary).await?;
                        fixture
                            .wait_for_expected_primary_best_effort(
                                &healthy_failover_target,
                                E2E_PRIMARY_CONVERGENCE_TIMEOUT,
                                2,
                                1,
                                &mut phase_history,
                            )
                            .await
                    },
                )
                .await?;
            if promoted_primary != healthy_failover_target {
                return Err(WorkerError::Message(format!(
                    "degraded failover promoted unexpected node: expected {healthy_failover_target}, got {promoted_primary}"
                )));
            }

            fixture.record(format!(
                "degraded failover no-dual-primary stats: samples={} max_concurrent_primaries={}",
                ha_stats.sample_count, ha_stats.max_concurrent_primaries
            ));
            fixture
                .stop_postgres_for_node(&degraded_replica)
                .await?;
            fixture.wipe_node_data_dir(&degraded_replica)?;
            fixture
                .restart_runtime_process_for_node(&degraded_replica)
                .await?;
            let recovered_nodes = fixture
                .wait_for_queryable_nodes(&promoted_primary, 3, Duration::from_secs(180))
                .await?;
            fixture
                .wait_for_node_sql_role(&degraded_replica, "replica", Duration::from_secs(120))
                .await?;
            fixture
                .insert_proof_row(&promoted_primary, table_name.as_str(), 2, "after-degraded-failover")
                .await?;
            let expected_rows = vec![
                "1:before-degraded-failover".to_string(),
                "2:after-degraded-failover".to_string(),
            ];
            fixture
                .wait_for_proof_rows_on_nodes(
                    recovered_nodes.as_slice(),
                    table_name.as_str(),
                    expected_rows.as_slice(),
                    E2E_TABLE_INTEGRITY_TIMEOUT,
                )
                .await?;
            fixture.record(format!(
                "degraded failover succeeded: bootstrap_primary={bootstrap_primary} degraded_replica={degraded_replica} promoted_primary={promoted_primary} phase_history={}",
                ClusterFixture::format_phase_history(&phase_history)
            ));
            Ok(())
        })
        .await
        {
            Ok(run_result) => run_result,
            Err(_) => Err(WorkerError::Message(format!(
                "degraded failover scenario timed out after {}s",
                E2E_SCENARIO_TIMEOUT.as_secs()
            ))),
        };

        let artifact_path = fixture.write_timeline_artifact(scenario_name);
        let shutdown_result = fixture.shutdown().await;
        finalize_timeline_scenario_result(run_result, artifact_path, shutdown_result)
    })
    .await
}

pub async fn e2e_multi_node_rejects_targeted_switchover_to_ineligible_member(
) -> Result<(), WorkerError> {
    ha_e2e::util::run_with_local_set(async {
        let mut fixture = ClusterFixture::start(3).await?;
        let mut phase_history: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let scenario_name = "ha-e2e-rejects-targeted-switchover-to-ineligible-member";

        let run_result = match tokio::time::timeout(E2E_SCENARIO_TIMEOUT, async {
            fixture.record("targeted rejection bootstrap: wait for stable primary");
            let bootstrap_primary = fixture
                .wait_for_stable_primary_resilient(
                    StablePrimaryWaitPlan {
                        context: "targeted rejection bootstrap stable-primary",
                        timeout: E2E_PRIMARY_CONVERGENCE_TIMEOUT,
                        excluded_primary: None,
                        required_consecutive: 5,
                        fallback_timeout: E2E_PRIMARY_CONVERGENCE_FALLBACK_TIMEOUT,
                        fallback_required_consecutive: 2,
                        min_observed_nodes: 2,
                    },
                    &mut phase_history,
                )
                .await?;
            let degraded_replica = fixture
                .node_ids_excluding(&bootstrap_primary)
                .first()
                .cloned()
                .ok_or_else(|| WorkerError::Message("missing degraded target replica".to_string()))?;
            fixture.record(format!(
                "targeted rejection: stop runtime on replica={degraded_replica}"
            ));
            fixture
                .stop_runtime_process_for_node(&degraded_replica)
                .await?;
            fixture
                .wait_for_member_to_be_ineligible(
                    &bootstrap_primary,
                    &degraded_replica,
                    Duration::from_secs(30),
                )
                .await?;

            let rejection_observer_clients = fixture.observed_node_clients()?;
            let (rejection_body, ha_stats) = ClusterFixture::observe_no_dual_primary_while_clients(
                    rejection_observer_clients,
                    E2E_SHORT_NO_DUAL_PRIMARY_WINDOW,
                    96,
                    async {
                        let rejection_body = fixture
                            .request_targeted_switchover_rejected_via_api(
                                &degraded_replica,
                                "not an eligible switchover target",
                            )
                            .await?;
                        let stable_primary = fixture
                            .wait_for_stable_primary_resilient(
                                StablePrimaryWaitPlan {
                                    context: "targeted rejection post-request stable-primary",
                                    timeout: E2E_PRIMARY_CONVERGENCE_TIMEOUT,
                                    excluded_primary: None,
                                    required_consecutive: 3,
                                    fallback_timeout: E2E_PRIMARY_CONVERGENCE_FALLBACK_TIMEOUT,
                                    fallback_required_consecutive: 1,
                                    min_observed_nodes: 1,
                                },
                                &mut phase_history,
                            )
                            .await?;
                        if stable_primary != bootstrap_primary {
                            return Err(WorkerError::Message(format!(
                                "targeted rejection changed primary unexpectedly: expected {bootstrap_primary}, got {stable_primary}"
                            )));
                        }
                        Ok(rejection_body)
                    },
                )
                .await?;

            fixture
                .restart_runtime_process_for_node(&degraded_replica)
                .await?;
            let table_name = fixture.proof_table_name("ha_targeted_rejection")?;
            fixture.create_proof_table(&bootstrap_primary, table_name.as_str()).await?;
            fixture
                .insert_proof_row(&bootstrap_primary, table_name.as_str(), 1, "post-rejection")
                .await?;
            let expected_rows = vec!["1:post-rejection".to_string()];
            fixture
                .wait_for_proof_rows_on_all_nodes(
                    table_name.as_str(),
                    expected_rows.as_slice(),
                    E2E_TABLE_INTEGRITY_TIMEOUT,
                )
                .await?;
            fixture.record(format!(
                "targeted rejection succeeded: primary={bootstrap_primary} target={degraded_replica} body={rejection_body} no_dual_samples={} phase_history={}",
                ha_stats.sample_count,
                ClusterFixture::format_phase_history(&phase_history)
            ));
            Ok(())
        })
        .await
        {
            Ok(run_result) => run_result,
            Err(_) => Err(WorkerError::Message(format!(
                "targeted rejection scenario timed out after {}s",
                E2E_SCENARIO_TIMEOUT.as_secs()
            ))),
        };

        let artifact_path = fixture.write_timeline_artifact(scenario_name);
        let shutdown_result = fixture.shutdown().await;
        finalize_timeline_scenario_result(run_result, artifact_path, shutdown_result)
    })
    .await
}

pub async fn e2e_multi_node_targeted_switchover_promotes_requested_replica(
) -> Result<(), WorkerError> {
    ha_e2e::util::run_with_local_set(async {
        let mut fixture = ClusterFixture::start(3).await?;
        let mut phase_history: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let scenario_name = "ha-e2e-targeted-switchover-promotes-requested-replica";

        let run_result = match tokio::time::timeout(E2E_SCENARIO_TIMEOUT, async {
            fixture.record("targeted switchover bootstrap: wait for stable primary");
            let bootstrap_primary = fixture
                .wait_for_stable_primary_resilient(
                    StablePrimaryWaitPlan {
                        context: "targeted switchover bootstrap stable-primary",
                        timeout: E2E_PRIMARY_CONVERGENCE_TIMEOUT,
                        excluded_primary: None,
                        required_consecutive: 5,
                        fallback_timeout: E2E_PRIMARY_CONVERGENCE_FALLBACK_TIMEOUT,
                        fallback_required_consecutive: 2,
                        min_observed_nodes: 2,
                    },
                    &mut phase_history,
                )
                .await?;
            let replica_ids = fixture.node_ids_excluding(&bootstrap_primary);
            let requested_successor = replica_ids.first().cloned().ok_or_else(|| {
                WorkerError::Message("missing requested switchover successor".to_string())
            })?;
            let disallowed_alternate = replica_ids.get(1).cloned().ok_or_else(|| {
                WorkerError::Message("missing alternate healthy replica".to_string())
            })?;

            let table_name = fixture.proof_table_name("ha_targeted_switchover")?;
            fixture
                .create_proof_table(&bootstrap_primary, table_name.as_str())
                .await?;
            fixture
                .insert_proof_row(
                    &bootstrap_primary,
                    table_name.as_str(),
                    1,
                    "before-targeted-switchover",
                )
                .await?;
            let bootstrap_rows = vec!["1:before-targeted-switchover".to_string()];
            fixture
                .wait_for_proof_rows_on_all_nodes(
                    table_name.as_str(),
                    bootstrap_rows.as_slice(),
                    E2E_SQL_REPLICATION_ASSERT_TIMEOUT,
                )
                .await?;

            let targeted_observer_clients = fixture.observed_node_clients()?;
            let (promoted_primary, ha_stats) = ClusterFixture::observe_no_dual_primary_while_clients(
                    targeted_observer_clients,
                    E2E_LONG_NO_DUAL_PRIMARY_WINDOW,
                    128,
                    async {
                        fixture.record(format!(
                            "targeted switchover request accepted path: old_primary={bootstrap_primary} requested_successor={requested_successor} disallowed_alternate={disallowed_alternate}"
                        ));
                        fixture
                            .request_targeted_switchover_via_api(&requested_successor)
                            .await?;
                        fixture
                            .wait_for_expected_primary_best_effort(
                                &requested_successor,
                                E2E_PRIMARY_CONVERGENCE_TIMEOUT,
                                2,
                                1,
                                &mut phase_history,
                            )
                            .await
                    },
                )
                .await?;
            if promoted_primary != requested_successor {
                return Err(WorkerError::Message(format!(
                    "targeted switchover promoted unexpected node: expected {requested_successor}, got {promoted_primary}"
                )));
            }
            if promoted_primary == disallowed_alternate {
                return Err(WorkerError::Message(format!(
                    "targeted switchover incorrectly promoted alternate healthy replica {disallowed_alternate} instead of requested successor {requested_successor}"
                )));
            }

            fixture.record(format!(
                "targeted switchover no-dual-primary stats: samples={} max_concurrent_primaries={}",
                ha_stats.sample_count, ha_stats.max_concurrent_primaries
            ));
            fixture
                .assert_former_primary_demoted_or_unreachable_after_transition(&bootstrap_primary)
                .await?;
            fixture
                .insert_proof_row(
                    &promoted_primary,
                    table_name.as_str(),
                    2,
                    "after-targeted-switchover",
                )
                .await?;
            let expected_rows = vec![
                "1:before-targeted-switchover".to_string(),
                "2:after-targeted-switchover".to_string(),
            ];
            let expected_queryable_nodes =
                vec![promoted_primary.clone(), disallowed_alternate.clone()];
            fixture
                .wait_for_proof_rows_on_nodes(
                    expected_queryable_nodes.as_slice(),
                    table_name.as_str(),
                    expected_rows.as_slice(),
                    E2E_TABLE_INTEGRITY_TIMEOUT,
                )
                .await?;
            fixture.record(format!(
                "targeted switchover succeeded: bootstrap_primary={bootstrap_primary} requested_successor={requested_successor} disallowed_alternate={disallowed_alternate} promoted_primary={promoted_primary} phase_history={}",
                ClusterFixture::format_phase_history(&phase_history)
            ));
            Ok(())
        })
        .await
        {
            Ok(run_result) => run_result,
            Err(_) => Err(WorkerError::Message(format!(
                "targeted switchover scenario timed out after {}s",
                E2E_SCENARIO_TIMEOUT.as_secs()
            ))),
        };

        let artifact_path = fixture.write_timeline_artifact(scenario_name);
        let shutdown_result = fixture.shutdown().await;
        finalize_timeline_scenario_result(run_result, artifact_path, shutdown_result)
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
        let (_stopped_members, failsafe_observed_at_ms) =
            stop_etcd_majority_and_wait_failsafe_strict_all_nodes(
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
            .sample_ha_states_window(Duration::from_secs(4), E2E_STRESS_SAMPLE_INTERVAL, 60)
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
            run_interval_ms: E2E_STRESS_WORKLOAD_RUN_INTERVAL_MS,
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
        let quorum_lost_at_ms = ha_e2e::util::unix_now()?.0;
        let (stopped_members, failsafe_observed_at_ms) =
            stop_etcd_majority_and_wait_failsafe_strict_all_nodes(
                &mut fixture,
                2,
                Duration::from_secs(60),
            )
            .await?;
        let ha_stats = fixture
            .sample_ha_states_window(Duration::from_secs(2), E2E_STRESS_SAMPLE_INTERVAL, 80)
            .await?;

        let fencing_grace_ms = 7_000u64;
        tokio::time::sleep(Duration::from_secs(8)).await;
        let workload = fixture
            .stop_sql_workload_and_collect(workload_handle, E2E_NO_QUORUM_WORKLOAD_STOP_TIMEOUT)
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

        let cutoff_ms = failsafe_observed_at_ms.saturating_add(fencing_grace_ms);
        let commits_after_cutoff =
            ClusterFixture::count_commits_after_cutoff_strict(&workload, cutoff_ms)?;
        let allowed_post_cutoff_commits = 10usize;
        if commits_after_cutoff > allowed_post_cutoff_commits {
            return Err(WorkerError::Message(format!(
                "writes still committed after fail-safe fencing cutoff beyond tolerance; cutoff_ms={cutoff_ms} commits_after_cutoff={commits_after_cutoff} allowed={allowed_post_cutoff_commits}"
            )));
        }
        ClusterFixture::assert_no_split_brain_write_evidence(&workload, &ha_stats)?;
        let required_committed_keys = committed_key_set_through_cutoff(&workload, cutoff_ms)?;
        let allowed_committed_keys: BTreeSet<String> =
            workload.committed_keys.iter().cloned().collect();
        let recovered_subset_required_keys = BTreeSet::new();

        fixture.record(format!(
            "no-quorum fencing recovery: restore etcd members {}",
            stopped_members.join(",")
        ));
        fixture.restore_etcd_members(stopped_members.as_slice()).await?;
        fixture.ensure_runtime_tasks_healthy().await?;
        let recovered_primary = fixture
            .wait_for_stable_primary_resilient(
                StablePrimaryWaitPlan {
                    context: "no-quorum fencing recovery stable-primary",
                    timeout: Duration::from_secs(90),
                    excluded_primary: None,
                    required_consecutive: 3,
                    fallback_timeout: Duration::from_secs(90),
                    fallback_required_consecutive: 1,
                    min_observed_nodes: 2,
                },
                &mut phase_history,
            )
            .await?;
        fixture.record(format!(
            "no-quorum fencing recovery: stable primary={recovered_primary}"
        ));

        let row_count = fixture
            .assert_table_recovery_key_integrity_on_node(
                recovered_primary.as_str(),
                table_name.as_str(),
                &recovered_subset_required_keys,
                &allowed_committed_keys,
                Duration::from_secs(45),
            )
            .await?;
        fixture.record(format!(
            "no-quorum fencing recovery subset integrity verified on {recovered_primary} with row_count={row_count} required_pre_cutoff_keys={} allowed_committed_keys={}",
            required_committed_keys.len(),
            allowed_committed_keys.len(),
        ));

        let finished_at_unix_ms = ha_e2e::util::unix_now()?.0;
        Ok(StressScenarioSummary {
            schema_version: STRESS_SUMMARY_SCHEMA_VERSION,
            scenario: scenario_name.to_string(),
            status: "passed".to_string(),
            started_at_unix_ms,
            finished_at_unix_ms,
            bootstrap_primary: Some(bootstrap_primary.clone()),
            final_primary: Some(recovered_primary.clone()),
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
                format!("failsafe_observed_at_ms={failsafe_observed_at_ms}"),
                format!("fencing_cutoff_ms={cutoff_ms}"),
                format!("allowed_post_cutoff_commits={allowed_post_cutoff_commits}"),
                format!(
                    "required_pre_cutoff_keys={}",
                    required_committed_keys.len()
                ),
                format!("allowed_committed_keys={}", allowed_committed_keys.len()),
                format!("recovered_primary={recovered_primary}"),
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
    fn recovered_committed_keys_match_bounds_passes_with_allowed_post_cutoff_extra(
    ) -> Result<(), WorkerError> {
        let required_keys = BTreeSet::from(["1:1".to_string(), "1:2".to_string()]);
        let allowed_keys =
            BTreeSet::from(["1:1".to_string(), "1:2".to_string(), "1:3".to_string()]);
        let observed_rows = vec!["1:1".to_string(), "1:2".to_string(), "1:3".to_string()];
        let row_count = assert_recovered_committed_keys_match_bounds(
            observed_rows.as_slice(),
            &required_keys,
            &allowed_keys,
            "node-1",
            "ha_table",
        )?;
        assert_eq!(row_count, 3);
        Ok(())
    }

    #[test]
    fn recovered_committed_keys_match_bounds_fails_on_duplicates() {
        let required_keys = BTreeSet::from(["1:1".to_string(), "1:2".to_string()]);
        let allowed_keys = required_keys.clone();
        let observed_rows = vec!["1:1".to_string(), "1:1".to_string()];
        assert!(assert_recovered_committed_keys_match_bounds(
            observed_rows.as_slice(),
            &required_keys,
            &allowed_keys,
            "node-1",
            "ha_table"
        )
        .is_err());
    }

    #[test]
    fn recovered_committed_keys_match_bounds_fails_on_missing_required_key() {
        let required_keys = BTreeSet::from(["1:1".to_string(), "1:2".to_string()]);
        let allowed_keys =
            BTreeSet::from(["1:1".to_string(), "1:2".to_string(), "9:9".to_string()]);
        let observed_rows = vec!["1:1".to_string(), "9:9".to_string()];
        assert!(assert_recovered_committed_keys_match_bounds(
            observed_rows.as_slice(),
            &required_keys,
            &allowed_keys,
            "node-1",
            "ha_table"
        )
        .is_err());
    }

    #[test]
    fn recovered_committed_keys_match_bounds_fails_on_unexpected_key() {
        let required_keys = BTreeSet::from(["1:1".to_string()]);
        let allowed_keys = required_keys.clone();
        let observed_rows = vec!["1:1".to_string(), "2:1".to_string()];
        assert!(assert_recovered_committed_keys_match_bounds(
            observed_rows.as_slice(),
            &required_keys,
            &allowed_keys,
            "node-1",
            "ha_table"
        )
        .is_err());
    }

    #[test]
    fn committed_key_set_through_cutoff_uses_per_worker_timestamp_alignment(
    ) -> Result<(), WorkerError> {
        let workload = SqlWorkloadStats {
            worker_stats: vec![SqlWorkloadWorkerStats {
                worker_id: 7,
                committed_keys: vec!["7:1".to_string(), "7:2".to_string(), "7:3".to_string()],
                committed_at_unix_ms: vec![100, 200, 300],
                ..SqlWorkloadWorkerStats::default()
            }],
            ..SqlWorkloadStats::default()
        };
        let observed = committed_key_set_through_cutoff(&workload, 200)?;
        let expected = BTreeSet::from(["7:1".to_string(), "7:2".to_string()]);
        assert_eq!(observed, expected);
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
        let _ = sample_key_set as fn(&BTreeSet<String>) -> String;
        let _ = committed_key_set_through_cutoff
            as fn(&SqlWorkloadStats, u64) -> Result<BTreeSet<String>, WorkerError>;
        let _ = assert_recovered_committed_keys_match_bounds
            as fn(
                &[String],
                &BTreeSet<String>,
                &BTreeSet<String>,
                &str,
                &str,
            ) -> Result<u64, WorkerError>;
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
        let _ = ClusterFixture::assert_table_recovery_key_integrity_on_node;
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
        let _ = ClusterFixture::restore_etcd_members;
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
