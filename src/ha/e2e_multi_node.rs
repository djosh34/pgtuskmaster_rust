use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
    process::{ExitStatus, Stdio},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    process::{Child, Command},
    task::JoinHandle,
};

use crate::{
    api::{worker::ApiWorkerCtx, AcceptedResponse, HaStateResponse},
    config::{
        schema::{
            ApiConfig, ClusterConfig, DcsConfig, DebugConfig, HaConfig, PostgresConfig,
            SecurityConfig,
        },
        BinaryPaths, ProcessConfig, RuntimeConfig,
    },
    dcs::{
        etcd_store::EtcdDcsStore,
        state::{DcsCache, DcsState, DcsTrust},
    },
    debug_api::{
        snapshot::{build_snapshot, AppLifecycle, DebugSnapshotCtx},
        worker::{DebugApiContractStubInputs, DebugApiCtx},
    },
    ha::{
        state::{
            HaPhase, HaState, HaWorkerContractStubInputs, HaWorkerCtx, ProcessDispatchDefaults,
        },
        worker as ha_worker,
    },
    pginfo::state::{
        PgConfig, PgConnInfo, PgInfoCommon, PgInfoState, PgSslMode, Readiness, SqlStatus,
    },
    process::{
        jobs::{ActiveJobKind, ShutdownMode},
        state::{ProcessState, ProcessWorkerCtx},
        worker::TokioCommandRunner,
    },
    state::{
        new_state_channel, MemberId, StatePublisher, StateSubscriber, UnixMillis, WorkerError,
        WorkerStatus,
    },
    test_harness::{
        binaries::{require_etcd_bin_for_real_tests, require_pg16_bin_for_real_tests},
        etcd3::{
            prepare_etcd_member_data_dir, spawn_etcd3_cluster, EtcdClusterHandle,
            EtcdClusterMemberSpec, EtcdClusterSpec,
        },
        namespace::NamespaceGuard,
        pg16::prepare_pgdata_dir,
        ports::allocate_ha_topology_ports,
    },
};

struct NodeFixture {
    id: String,
    pg_port: u16,
    api_addr: SocketAddr,
    api_ctx: ApiWorkerCtx,
    debug_ctx: DebugApiCtx,
    data_dir: PathBuf,
    process_subscriber: StateSubscriber<ProcessState>,
    _config_publisher: StatePublisher<RuntimeConfig>,
}

struct ClusterFixture {
    _guard: NamespaceGuard,
    scope: String,
    endpoints: Vec<String>,
    pg_ctl_bin: PathBuf,
    psql_bin: PathBuf,
    etcd: Option<EtcdClusterHandle>,
    nodes: Vec<NodeFixture>,
    tasks: Vec<JoinHandle<Result<(), WorkerError>>>,
    timeline: Vec<String>,
}

const E2E_COMMAND_TIMEOUT: Duration = Duration::from_secs(30);
const E2E_COMMAND_KILL_WAIT_TIMEOUT: Duration = Duration::from_secs(3);
const E2E_HTTP_STEP_TIMEOUT: Duration = Duration::from_secs(10);
const E2E_SCENARIO_TIMEOUT: Duration = Duration::from_secs(300);
const STRESS_ARTIFACT_DIR: &str = ".ralph/evidence/27-e2e-ha-stress";
const STRESS_SUMMARY_SCHEMA_VERSION: u32 = 1;

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
    unique_committed_keys: usize,
    committed_keys: Vec<String>,
    committed_at_unix_ms: Vec<u64>,
    worker_stats: Vec<SqlWorkloadWorkerStats>,
    worker_errors: Vec<String>,
}

#[derive(Default, serde::Serialize)]
struct HaObservationStats {
    sample_count: u64,
    api_error_count: u64,
    max_concurrent_primaries: usize,
    leader_change_count: u64,
    failsafe_sample_count: u64,
    recent_samples: Vec<String>,
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
    async fn start(
        node_count: usize,
        binaries: BinaryPaths,
        etcd_bin: PathBuf,
    ) -> Result<Self, WorkerError> {
        let guard = NamespaceGuard::new("ha-e2e-multi-node")
            .map_err(|err| WorkerError::Message(format!("namespace create failed: {err}")))?;
        let namespace = guard
            .namespace()
            .map_err(|err| WorkerError::Message(format!("namespace lookup failed: {err}")))?;
        let scope = "scope-ha-e2e".to_string();

        let reservation = allocate_ha_topology_ports(node_count, 3)
            .map_err(|err| WorkerError::Message(format!("allocate ports failed: {err}")))?;
        let topology = reservation.into_layout();
        let node_ports = topology.node_ports;
        let etcd_members = ["etcd-a", "etcd-b", "etcd-c"];
        let mut members = Vec::with_capacity(etcd_members.len());
        for (index, member_name) in etcd_members.iter().enumerate() {
            let data_dir = prepare_etcd_member_data_dir(namespace, member_name).map_err(|err| {
                WorkerError::Message(format!("prepare etcd data dir failed: {err}"))
            })?;
            let log_dir = namespace.child_dir(format!("logs/{member_name}"));
            members.push(EtcdClusterMemberSpec {
                member_name: (*member_name).to_string(),
                data_dir,
                log_dir,
                client_port: topology.etcd_client_ports[index],
                peer_port: topology.etcd_peer_ports[index],
            });
        }
        let cluster_spec = EtcdClusterSpec {
            etcd_bin,
            namespace_id: namespace.id.clone(),
            startup_timeout: Duration::from_secs(15),
            members,
        };

        let etcd = spawn_etcd3_cluster(cluster_spec)
            .await
            .map_err(|err| WorkerError::Message(format!("spawn etcd failed: {err}")))?;
        let endpoints = etcd.client_endpoints().to_vec();
        let endpoint_count = endpoints.len();
        if endpoint_count == 0 {
            return Err(WorkerError::Message(
                "etcd cluster returned no endpoints".to_string(),
            ));
        }

        let pg_ctl_bin = binaries.pg_ctl.clone();
        let psql_bin = binaries.psql.clone();
        let mut tasks = Vec::new();
        let mut nodes = Vec::new();
        let rewind_source_conninfo = PgConnInfo {
            host: "127.0.0.1".to_string(),
            port: node_ports[0],
            user: "postgres".to_string(),
            dbname: "postgres".to_string(),
            application_name: None,
            connect_timeout_s: None,
            ssl_mode: PgSslMode::Prefer,
            options: None,
        };

        for (index, pg_port) in node_ports.into_iter().enumerate() {
            let node_id = format!("node-{}", index.saturating_add(1));
            let data_dir = prepare_pgdata_dir(namespace, &node_id).map_err(|err| {
                WorkerError::Message(format!("prepare pg data dir failed: {err}"))
            })?;
            initialize_pgdata(&binaries.initdb, &data_dir).await?;

            let socket_dir = namespace.child_dir(format!("run/{node_id}"));
            let log_file = namespace.child_dir(format!("logs/{node_id}/postgres.log"));
            if let Some(parent) = log_file.parent() {
                fs::create_dir_all(parent).map_err(|err| {
                    WorkerError::Message(format!("create node log dir failed: {err}"))
                })?;
            } else {
                return Err(WorkerError::Message(
                    "node log file has no parent directory".to_string(),
                ));
            }
            fs::create_dir_all(&socket_dir).map_err(|err| {
                WorkerError::Message(format!("create node socket dir failed: {err}"))
            })?;

            let runtime_cfg = RuntimeConfig {
                cluster: ClusterConfig {
                    name: "cluster-e2e".to_string(),
                    member_id: node_id.clone(),
                },
                postgres: PostgresConfig {
                    data_dir: data_dir.clone(),
                    connect_timeout_s: 2,
                },
                dcs: DcsConfig {
                    endpoints: endpoints.clone(),
                    scope: scope.clone(),
                },
                ha: HaConfig {
                    loop_interval_ms: 100,
                    lease_ttl_ms: 2_000,
                },
                process: ProcessConfig {
                    pg_rewind_timeout_ms: 5_000,
                    bootstrap_timeout_ms: 5_000,
                    fencing_timeout_ms: 5_000,
                    binaries: binaries.clone(),
                },
                api: ApiConfig {
                    listen_addr: "127.0.0.1:0".to_string(),
                    read_auth_token: None,
                    admin_auth_token: None,
                },
                debug: DebugConfig { enabled: false },
                security: SecurityConfig {
                    tls_enabled: false,
                    auth_token: None,
                },
            };

            let (cfg_publisher, cfg_subscriber) =
                new_state_channel(runtime_cfg.clone(), unix_now()?);

            let (pg_publisher, pg_subscriber) = new_state_channel(initial_pg_state(), unix_now()?);
            let initial_dcs_state = DcsState {
                worker: WorkerStatus::Starting,
                trust: DcsTrust::NotTrusted,
                cache: DcsCache {
                    members: BTreeMap::new(),
                    leader: None,
                    switchover: None,
                    config: runtime_cfg.clone(),
                    init_lock: None,
                },
                last_refresh_at: None,
            };
            let (dcs_publisher, dcs_subscriber) = new_state_channel(initial_dcs_state, unix_now()?);
            let initial_process_state = ProcessState::Idle {
                worker: WorkerStatus::Starting,
                last_outcome: None,
            };
            let (process_publisher, process_subscriber) =
                new_state_channel(initial_process_state, unix_now()?);
            let initial_ha_state = HaState {
                worker: WorkerStatus::Starting,
                phase: HaPhase::Init,
                tick: 0,
                pending: Vec::new(),
                recent_action_ids: BTreeSet::new(),
            };
            let (ha_publisher, ha_subscriber) = new_state_channel(initial_ha_state, unix_now()?);
            let (process_tx, process_rx) = tokio::sync::mpsc::unbounded_channel();
            let api_cfg_subscriber = cfg_subscriber.clone();
            let debug_cfg_subscriber = cfg_subscriber.clone();
            let debug_pg_subscriber = pg_subscriber.clone();
            let debug_dcs_subscriber = dcs_subscriber.clone();
            let debug_process_subscriber = process_subscriber.clone();
            let debug_ha_subscriber = ha_subscriber.clone();

            let pg_ctx = crate::pginfo::state::PgInfoWorkerCtx {
                self_id: MemberId(node_id.clone()),
                postgres_dsn: format!(
                    "host=127.0.0.1 port={pg_port} user=postgres dbname=postgres"
                ),
                poll_interval: Duration::from_millis(100),
                publisher: pg_publisher,
            };

            let dcs_store = EtcdDcsStore::connect(endpoints.clone(), &scope)
                .map_err(|err| WorkerError::Message(format!("dcs store connect failed: {err}")))?;
            let dcs_ctx = crate::dcs::state::DcsWorkerCtx {
                self_id: MemberId(node_id.clone()),
                scope: scope.clone(),
                poll_interval: Duration::from_millis(100),
                pg_subscriber: pg_subscriber.clone(),
                publisher: dcs_publisher,
                store: Box::new(dcs_store),
                cache: DcsCache {
                    members: BTreeMap::new(),
                    leader: None,
                    switchover: None,
                    config: runtime_cfg.clone(),
                    init_lock: None,
                },
                last_published_pg_version: None,
            };

            let mut process_ctx = ProcessWorkerCtx::contract_stub(
                runtime_cfg.process.clone(),
                process_publisher,
                process_rx,
            );
            process_ctx.poll_interval = Duration::from_millis(100);
            process_ctx.command_runner = Box::new(TokioCommandRunner);
            process_ctx.now = system_clock();

            let ha_store = EtcdDcsStore::connect(endpoints.clone(), &scope).map_err(|err| {
                WorkerError::Message(format!("ha dcs store connect failed: {err}"))
            })?;
            let mut ha_ctx = HaWorkerCtx::contract_stub(HaWorkerContractStubInputs {
                publisher: ha_publisher,
                config_subscriber: cfg_subscriber,
                pg_subscriber,
                dcs_subscriber: dcs_subscriber.clone(),
                process_subscriber: process_subscriber.clone(),
                process_inbox: process_tx,
                dcs_store: Box::new(ha_store),
                scope: scope.clone(),
                self_id: MemberId(node_id.clone()),
            });
            ha_ctx.poll_interval = Duration::from_millis(100);
            ha_ctx.process_defaults = ProcessDispatchDefaults {
                postgres_host: "127.0.0.1".to_string(),
                postgres_port: pg_port,
                socket_dir: socket_dir.clone(),
                log_file: log_file.clone(),
                rewind_source_conninfo: rewind_source_conninfo.clone(),
                shutdown_mode: ShutdownMode::Immediate,
            };
            ha_ctx.now = system_clock();

            let debug_now = unix_now()?;
            let initial_debug_snapshot = build_snapshot(
                &DebugSnapshotCtx {
                    app: AppLifecycle::Starting,
                    config: debug_cfg_subscriber.latest(),
                    pg: debug_pg_subscriber.latest(),
                    dcs: debug_dcs_subscriber.latest(),
                    process: debug_process_subscriber.latest(),
                    ha: debug_ha_subscriber.latest(),
                },
                debug_now,
                0,
                &[],
                &[],
            );
            let (debug_publisher, debug_subscriber) =
                new_state_channel(initial_debug_snapshot, debug_now);
            let mut debug_ctx = DebugApiCtx::contract_stub(DebugApiContractStubInputs {
                publisher: debug_publisher,
                config_subscriber: debug_cfg_subscriber,
                pg_subscriber: debug_pg_subscriber,
                dcs_subscriber: debug_dcs_subscriber,
                process_subscriber: debug_process_subscriber,
                ha_subscriber: debug_ha_subscriber,
            });
            debug_ctx.app = AppLifecycle::Running;
            debug_ctx.poll_interval = Duration::from_millis(100);
            debug_ctx.now = system_clock();

            let api_listener = tokio::net::TcpListener::bind(runtime_cfg.api.listen_addr.as_str())
                .await
                .map_err(|err| WorkerError::Message(format!("api bind failed: {err}")))?;
            let api_store = EtcdDcsStore::connect(endpoints.clone(), &scope).map_err(|err| {
                WorkerError::Message(format!("api dcs store connect failed: {err}"))
            })?;
            let mut api_ctx =
                ApiWorkerCtx::contract_stub(api_listener, api_cfg_subscriber, Box::new(api_store));
            api_ctx.set_ha_snapshot_subscriber(debug_subscriber);
            let api_addr = api_ctx.local_addr()?;

            tasks.push(tokio::spawn(async move {
                crate::pginfo::worker::run(pg_ctx).await
            }));
            tasks.push(tokio::spawn(async move {
                crate::dcs::worker::run(dcs_ctx).await
            }));
            tasks.push(tokio::spawn(async move {
                crate::process::worker::run(process_ctx).await
            }));
            tasks.push(tokio::spawn(async move { ha_worker::run(ha_ctx).await }));

            nodes.push(NodeFixture {
                id: node_id,
                pg_port,
                api_addr,
                api_ctx,
                debug_ctx,
                data_dir,
                process_subscriber,
                _config_publisher: cfg_publisher,
            });
        }

        Ok(Self {
            _guard: guard,
            scope,
            endpoints,
            pg_ctl_bin,
            psql_bin,
            etcd: Some(etcd),
            nodes,
            tasks,
            timeline: Vec::new(),
        })
    }

    fn record(&mut self, message: impl Into<String>) {
        let now = match unix_now() {
            Ok(value) => value.0,
            Err(_) => 0,
        };
        self.timeline.push(format!("[{now}] {}", message.into()));
    }

    fn node_by_id(&self, id: &str) -> Option<&NodeFixture> {
        self.nodes.iter().find(|node| node.id == id)
    }

    fn control_node_index(&self) -> Result<usize, WorkerError> {
        if self.nodes.is_empty() {
            return Err(WorkerError::Message(
                "no nodes available for API control".to_string(),
            ));
        }
        Ok(0)
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

    async fn run_sql_on_node(&self, node_id: &str, sql: &str) -> Result<String, WorkerError> {
        let port = self.postgres_port_by_id(node_id)?;
        run_psql_statement(self.psql_bin.as_path(), port, sql).await
    }

    async fn run_sql_on_node_with_retry(
        &self,
        node_id: &str,
        sql: &str,
        timeout: Duration,
    ) -> Result<String, WorkerError> {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            match self.run_sql_on_node(node_id, sql).await {
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

    async fn wait_for_rows_on_node(
        &self,
        node_id: &str,
        sql: &str,
        expected_rows: &[String],
        timeout: Duration,
    ) -> Result<(), WorkerError> {
        let deadline = tokio::time::Instant::now() + timeout;

        loop {
            let observation = match self.run_sql_on_node(node_id, sql).await {
                Ok(output) => {
                    let rows = parse_psql_rows(output.as_str());
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

    fn node_ids(&self) -> Vec<String> {
        self.nodes.iter().map(|node| node.id.clone()).collect()
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

    async fn wait_for_table_readable_on_nodes(
        &self,
        table_name: &str,
        node_ids: &[String],
        timeout: Duration,
    ) -> Result<(), WorkerError> {
        if node_ids.is_empty() {
            return Err(WorkerError::Message(
                "table readability check requires at least one node".to_string(),
            ));
        }
        let count_sql = format!("SELECT COUNT(*)::bigint FROM {table_name}");
        let deadline = tokio::time::Instant::now() + timeout;
        let mut pending: BTreeSet<String> = node_ids.iter().cloned().collect();
        let mut last_observation = "none".to_string();
        loop {
            let pending_nodes: Vec<String> = pending.iter().cloned().collect();
            for node_id in pending_nodes {
                match self
                    .run_sql_on_node(node_id.as_str(), count_sql.as_str())
                    .await
                {
                    Ok(_) => {
                        let _ = pending.remove(&node_id);
                    }
                    Err(err) => {
                        last_observation =
                            format!("node={node_id} table readability probe failed: {err}");
                    }
                }
            }
            if pending.is_empty() {
                return Ok(());
            }
            if tokio::time::Instant::now() >= deadline {
                return Err(WorkerError::Message(format!(
                    "timed out waiting for table {table_name} readability on nodes={pending:?}; last_observation={last_observation}"
                )));
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    }

    async fn start_sql_workload(
        &mut self,
        spec: SqlWorkloadSpec,
    ) -> Result<SqlWorkloadHandle, WorkerError> {
        let workload_ctx = self.sql_workload_ctx(&spec)?;
        let started_at_unix_ms = unix_now()?.0;
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
        stats.finished_at_unix_ms = unix_now()?.0;
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
        let mut stats = HaObservationStats::default();
        let mut last_leader_signature: Option<String> = None;
        loop {
            match self.cluster_ha_states().await {
                Ok(states) => {
                    stats.sample_count = stats.sample_count.saturating_add(1);
                    let primaries = Self::primary_members(states.as_slice());
                    stats.max_concurrent_primaries =
                        stats.max_concurrent_primaries.max(primaries.len());

                    let mut leaders = states
                        .iter()
                        .filter_map(|state| state.leader.clone())
                        .collect::<Vec<_>>();
                    leaders.sort();
                    leaders.dedup();
                    let leader_signature = leaders.join("|");
                    if last_leader_signature
                        .as_deref()
                        .map(|prior| prior != leader_signature.as_str())
                        .unwrap_or(false)
                    {
                        stats.leader_change_count = stats.leader_change_count.saturating_add(1);
                    }
                    last_leader_signature = Some(leader_signature);
                    if !states.is_empty() && states.iter().all(|state| state.ha_phase == "FailSafe")
                    {
                        stats.failsafe_sample_count = stats.failsafe_sample_count.saturating_add(1);
                    }
                    if ring_capacity > 0 {
                        let sample = states
                            .iter()
                            .map(|state| {
                                let leader = state.leader.as_deref().unwrap_or("none");
                                format!(
                                    "{}:{}:leader={leader}",
                                    state.self_member_id, state.ha_phase
                                )
                            })
                            .collect::<Vec<_>>()
                            .join(", ");
                        if stats.recent_samples.len() >= ring_capacity {
                            let _ = stats.recent_samples.remove(0);
                        }
                        stats.recent_samples.push(sample);
                    }
                }
                Err(err) => {
                    stats.api_error_count = stats.api_error_count.saturating_add(1);
                    if ring_capacity > 0 {
                        if stats.recent_samples.len() >= ring_capacity {
                            let _ = stats.recent_samples.remove(0);
                        }
                        stats.recent_samples.push(format!("api_error:{err}"));
                    }
                }
            }
            if tokio::time::Instant::now() >= deadline {
                return Ok(stats);
            }
            tokio::time::sleep(interval).await;
        }
    }

    fn assert_no_dual_primary_in_samples(stats: &HaObservationStats) -> Result<(), WorkerError> {
        if stats.max_concurrent_primaries > 1 {
            return Err(WorkerError::Message(format!(
                "dual primary observed during sampled window; max_concurrent_primaries={}",
                stats.max_concurrent_primaries
            )));
        }
        Ok(())
    }

    async fn assert_former_primary_demoted_after_transition(
        &mut self,
        former_primary: &str,
    ) -> Result<(), WorkerError> {
        let node_index = self.node_index_by_id(former_primary).ok_or_else(|| {
            WorkerError::Message(format!(
                "unknown former primary for demotion assertion: {former_primary}"
            ))
        })?;
        let state = self.fetch_node_ha_state_by_index(node_index).await?;
        if state.ha_phase == "Primary" {
            return Err(WorkerError::Message(format!(
                "former primary {former_primary} still reports Primary phase"
            )));
        }
        Ok(())
    }

    async fn wait_for_table_digest_convergence(
        &self,
        table_name: &str,
        node_ids: &[String],
        expected_min_rows: usize,
        timeout: Duration,
    ) -> Result<BTreeMap<String, String>, WorkerError> {
        if node_ids.is_empty() {
            return Err(WorkerError::Message(
                "cannot verify table digest convergence with empty node list".to_string(),
            ));
        }
        let expected_min_rows_u64 = u64::try_from(expected_min_rows).unwrap_or(u64::MAX);
        let deadline = tokio::time::Instant::now() + timeout;
        let mut last_observation = "none".to_string();
        loop {
            let mut digests = BTreeMap::new();
            let mut row_counts = BTreeMap::new();
            let digest_sql = format!(
                "SELECT COALESCE(string_agg(worker_id::text || ':' || seq::text || ':' || payload, ',' ORDER BY worker_id, seq), '') FROM {table_name}"
            );
            let count_sql = format!("SELECT COUNT(*)::bigint FROM {table_name}");
            let mut query_failed = false;
            for node_id in node_ids {
                let digest_raw = match self.run_sql_on_node(node_id, digest_sql.as_str()).await {
                    Ok(value) => value,
                    Err(err) => {
                        query_failed = true;
                        last_observation =
                            format!("node={node_id} digest query failed during convergence: {err}");
                        break;
                    }
                };
                let count_raw = match self.run_sql_on_node(node_id, count_sql.as_str()).await {
                    Ok(value) => value,
                    Err(err) => {
                        query_failed = true;
                        last_observation =
                            format!("node={node_id} count query failed during convergence: {err}");
                        break;
                    }
                };
                let digest = parse_psql_rows(digest_raw.as_str())
                    .first()
                    .cloned()
                    .unwrap_or_default();
                let row_count = parse_single_u64(count_raw.as_str())?;
                digests.insert(node_id.clone(), digest);
                row_counts.insert(node_id.clone(), row_count);
            }
            if !query_failed {
                let mut digest_values = digests.values();
                let first_digest = digest_values.next().cloned().unwrap_or_default();
                let all_equal = digest_values.all(|digest| digest == &first_digest);
                let all_counts_satisfied = row_counts
                    .values()
                    .all(|count| *count >= expected_min_rows_u64);
                if all_equal && all_counts_satisfied {
                    return Ok(digests);
                }
                last_observation = format!(
                    "digest mismatch or low row counts; row_counts={row_counts:?} all_equal={all_equal}"
                );
            }
            if tokio::time::Instant::now() >= deadline {
                return Err(WorkerError::Message(format!(
                    "timed out waiting for table digest convergence on {table_name}; last_observation={last_observation}"
                )));
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    }

    async fn assert_table_key_integrity_on_node(
        &self,
        node_id: &str,
        table_name: &str,
        min_rows: u64,
        timeout: Duration,
    ) -> Result<u64, WorkerError> {
        let count_sql = format!("SELECT COUNT(*)::bigint FROM {table_name}");
        let duplicate_sql = format!(
            "SELECT COUNT(*)::bigint FROM (SELECT worker_id, seq, COUNT(*) AS c FROM {table_name} GROUP BY worker_id, seq HAVING COUNT(*) > 1) d"
        );
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            let count_raw = match self.run_sql_on_node(node_id, count_sql.as_str()).await {
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
            let duplicate_raw = match self.run_sql_on_node(node_id, duplicate_sql.as_str()).await {
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
            let row_count = parse_single_u64(count_raw.as_str())?;
            let duplicate_count = parse_single_u64(duplicate_raw.as_str())?;
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
                .insert(state.ha_phase.clone());
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

    fn assert_phase_history_contains_failover(
        phase_history: &BTreeMap<String, BTreeSet<String>>,
        former_primary: &str,
        new_primary: &str,
    ) -> Result<(), WorkerError> {
        let former_phases = phase_history.get(former_primary).ok_or_else(|| {
            WorkerError::Message(format!(
                "missing phase history for former primary {former_primary}"
            ))
        })?;
        if !former_phases.contains("Primary") {
            return Err(WorkerError::Message(format!(
                "former primary {former_primary} never observed in Primary phase"
            )));
        }
        if !former_phases.iter().any(|phase| phase != "Primary") {
            return Err(WorkerError::Message(format!(
                "former primary {former_primary} never observed leaving Primary phase"
            )));
        }

        let promoted_phases = phase_history.get(new_primary).ok_or_else(|| {
            WorkerError::Message(format!(
                "missing phase history for promoted primary {new_primary}"
            ))
        })?;
        if !promoted_phases.contains("Primary") {
            return Err(WorkerError::Message(format!(
                "new primary {new_primary} never observed in Primary phase"
            )));
        }

        Ok(())
    }

    async fn send_node_request(
        &mut self,
        node_index: usize,
        method: &str,
        path: &str,
        body: Option<&[u8]>,
        content_type: Option<&str>,
    ) -> Result<ApiHttpResponse, WorkerError> {
        let node_id = self
            .nodes
            .get(node_index)
            .ok_or_else(|| {
                WorkerError::Message(format!("invalid node index for API request: {node_index}"))
            })?
            .id
            .clone();
        self.record(format!("api request start: node={node_id} {method} {path}"));
        let response = {
            let node = self.nodes.get_mut(node_index).ok_or_else(|| {
                WorkerError::Message(format!("invalid node index for API request: {node_index}"))
            })?;
            send_http_request_with_worker(node, method, path, body, content_type).await
        };
        match &response {
            Ok(http) => self.record(format!(
                "api request success: node={node_id} {method} {path} status={}",
                http.status_code
            )),
            Err(err) => self.record(format!(
                "api request failure: node={node_id} {method} {path} error={err}"
            )),
        }
        response
    }

    async fn post_switchover_via_api(&mut self, requested_by: &str) -> Result<(), WorkerError> {
        #[derive(serde::Serialize)]
        struct SwitchoverBody<'a> {
            requested_by: &'a str,
        }

        let body = serde_json::to_vec(&SwitchoverBody { requested_by }).map_err(|err| {
            WorkerError::Message(format!("encode switchover request failed: {err}"))
        })?;
        let response = self
            .send_node_request(
                self.control_node_index()?,
                "POST",
                "/switchover",
                Some(&body),
                Some("application/json"),
            )
            .await?;
        expect_accepted_response("POST /switchover", response)
    }

    async fn fetch_node_ha_state_by_index(
        &mut self,
        node_index: usize,
    ) -> Result<HaStateResponse, WorkerError> {
        let response = self
            .send_node_request(node_index, "GET", "/ha/state", None, None)
            .await?;
        if response.status_code != 200 {
            let body = String::from_utf8_lossy(&response.body);
            return Err(WorkerError::Message(format!(
                "GET /ha/state returned status {} body={}",
                response.status_code,
                body.trim()
            )));
        }
        serde_json::from_slice::<HaStateResponse>(&response.body)
            .map_err(|err| WorkerError::Message(format!("decode /ha/state response failed: {err}")))
    }

    async fn cluster_ha_states(&mut self) -> Result<Vec<HaStateResponse>, WorkerError> {
        let mut states = Vec::with_capacity(self.nodes.len());
        let node_count = self.nodes.len();
        for index in 0..node_count {
            states.push(self.fetch_node_ha_state_by_index(index).await?);
        }
        Ok(states)
    }

    fn primary_members(states: &[HaStateResponse]) -> Vec<String> {
        states
            .iter()
            .filter(|state| state.ha_phase == "Primary")
            .map(|state| state.self_member_id.clone())
            .collect()
    }

    async fn wait_for_primary(&mut self, timeout: Duration) -> Result<String, WorkerError> {
        let deadline = tokio::time::Instant::now() + timeout;
        let mut last_error: Option<String> = None;
        loop {
            match self.cluster_ha_states().await {
                Ok(states) => {
                    let primaries = Self::primary_members(&states);
                    if primaries.len() == 1 {
                        if let Some(primary) = primaries.into_iter().next() {
                            return Ok(primary);
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
                    "timed out waiting for single primary via API; last_error={detail}"
                )));
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
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

    async fn assert_no_dual_primary_window(&mut self, window: Duration) -> Result<(), WorkerError> {
        let deadline = tokio::time::Instant::now() + window;
        let mut last_error: Option<String> = None;
        loop {
            match self.cluster_ha_states().await {
                Ok(states) => {
                    if Self::primary_members(&states).len() > 1 {
                        return Err(WorkerError::Message(
                            "split-brain detected: more than one primary".to_string(),
                        ));
                    }
                }
                Err(err) => {
                    last_error = Some(err.to_string());
                }
            }
            if tokio::time::Instant::now() >= deadline {
                if let Some(err) = last_error {
                    return Err(WorkerError::Message(format!(
                        "split-brain observation window ended with API errors: {err}"
                    )));
                }
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(75)).await;
        }
    }

    async fn wait_for_rewind_or_process_outcome(
        &mut self,
        node_id: &str,
        timeout: Duration,
    ) -> Result<(), WorkerError> {
        let deadline = tokio::time::Instant::now() + timeout;
        let mut last_error: Option<String> = None;
        loop {
            if let Some(index) = self.node_index_by_id(node_id) {
                let saw_rewind_phase = match self.fetch_node_ha_state_by_index(index).await {
                    Ok(state) => state.ha_phase == "Rewinding",
                    Err(err) => {
                        last_error = Some(err.to_string());
                        false
                    }
                };
                let process_state = self
                    .nodes
                    .get(index)
                    .ok_or_else(|| WorkerError::Message("node disappeared".to_string()))?
                    .process_subscriber
                    .latest()
                    .value;
                let saw_rewind_job = matches!(
                    process_state,
                    ProcessState::Running { ref active, .. }
                        if active.kind == ActiveJobKind::PgRewind
                );
                let saw_process_outcome = matches!(
                    process_state,
                    ProcessState::Idle {
                        last_outcome: Some(_),
                        ..
                    }
                );
                if saw_rewind_phase || saw_rewind_job || saw_process_outcome {
                    return Ok(());
                }
            }
            if tokio::time::Instant::now() >= deadline {
                let detail = last_error
                    .as_deref()
                    .map_or_else(|| "none".to_string(), ToString::to_string);
                return Err(WorkerError::Message(format!(
                    "timed out waiting for rewind path on {node_id}; last_error={detail}"
                )));
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    async fn wait_for_all_failsafe(&mut self, timeout: Duration) -> Result<(), WorkerError> {
        let deadline = tokio::time::Instant::now() + timeout;
        let mut last_error: Option<String> = None;
        loop {
            match self.cluster_ha_states().await {
                Ok(states) => {
                    let all_failsafe = !states.is_empty()
                        && states.iter().all(|state| state.ha_phase == "FailSafe");
                    if all_failsafe {
                        return Ok(());
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
                    "timed out waiting for all nodes to enter fail-safe via API; last_error={detail}"
                )));
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    async fn stop_postgres_for_node(&self, node_id: &str) -> Result<(), WorkerError> {
        let Some(node) = self.node_by_id(node_id) else {
            return Err(WorkerError::Message(format!(
                "unknown node for stop request: {node_id}"
            )));
        };
        pg_ctl_stop_immediate(&self.pg_ctl_bin, &node.data_dir).await
    }

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
        let stamp = unix_now()?.0;
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
        let stamp = unix_now()?.0;
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

        for node in &self.nodes {
            let _ = pg_ctl_stop_immediate(&self.pg_ctl_bin, &node.data_dir).await;
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

fn resolve_pg_binaries_for_real_tests() -> Result<BinaryPaths, WorkerError> {
    let postgres = require_pg16_bin_for_real_tests("postgres")
        .map_err(|err| WorkerError::Message(format!("postgres binary lookup failed: {err}")))?;
    let pg_ctl = require_pg16_bin_for_real_tests("pg_ctl")
        .map_err(|err| WorkerError::Message(format!("pg_ctl binary lookup failed: {err}")))?;
    let pg_rewind = require_pg16_bin_for_real_tests("pg_rewind")
        .map_err(|err| WorkerError::Message(format!("pg_rewind binary lookup failed: {err}")))?;
    let initdb = require_pg16_bin_for_real_tests("initdb")
        .map_err(|err| WorkerError::Message(format!("initdb binary lookup failed: {err}")))?;
    let psql = require_pg16_bin_for_real_tests("psql")
        .map_err(|err| WorkerError::Message(format!("psql binary lookup failed: {err}")))?;
    Ok(BinaryPaths {
        postgres,
        pg_ctl,
        pg_rewind,
        initdb,
        pg_basebackup: require_pg16_bin_for_real_tests("pg_basebackup").map_err(|err| {
            WorkerError::Message(format!("pg_basebackup binary lookup failed: {err}"))
        })?,
        psql,
    })
}

fn resolve_etcd_bin_for_real_tests() -> Result<PathBuf, WorkerError> {
    require_etcd_bin_for_real_tests()
        .map_err(|err| WorkerError::Message(format!("etcd binary lookup failed: {err}")))
}

fn initial_pg_state() -> PgInfoState {
    PgInfoState::Unknown {
        common: PgInfoCommon {
            worker: WorkerStatus::Starting,
            sql: SqlStatus::Unknown,
            readiness: Readiness::Unknown,
            timeline: None,
            pg_config: PgConfig {
                port: None,
                hot_standby: None,
                primary_conninfo: None,
                primary_slot_name: None,
                extra: BTreeMap::new(),
            },
            last_refresh_at: None,
        },
    }
}

fn system_clock() -> Box<dyn FnMut() -> Result<UnixMillis, WorkerError> + Send> {
    Box::new(unix_now)
}

fn unix_now() -> Result<UnixMillis, WorkerError> {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| WorkerError::Message(format!("system time before epoch: {err}")))?;
    let millis = u64::try_from(elapsed.as_millis())
        .map_err(|err| WorkerError::Message(format!("millis conversion failed: {err}")))?;
    Ok(UnixMillis(millis))
}

async fn initialize_pgdata(initdb: &Path, data_dir: &Path) -> Result<(), WorkerError> {
    let mut child = Command::new(initdb)
        .arg("-D")
        .arg(data_dir)
        .arg("-A")
        .arg("trust")
        .arg("-U")
        .arg("postgres")
        .spawn()
        .map_err(|err| WorkerError::Message(format!("initdb spawn failed: {err}")))?;
    let label = format!("initdb for {}", data_dir.display());
    let status = wait_for_child_exit_with_timeout(&label, &mut child, E2E_COMMAND_TIMEOUT).await?;
    if status.success() {
        Ok(())
    } else {
        Err(WorkerError::Message(format!(
            "initdb exited unsuccessfully with status {status}"
        )))
    }
}

async fn pg_ctl_stop_immediate(pg_ctl: &Path, data_dir: &Path) -> Result<(), WorkerError> {
    let pid_file = data_dir.join("postmaster.pid");
    if !pid_file.exists() {
        return Ok(());
    }

    let mut child = Command::new(pg_ctl)
        .arg("-D")
        .arg(data_dir)
        .arg("stop")
        .arg("-m")
        .arg("immediate")
        .arg("-w")
        .spawn()
        .map_err(|err| WorkerError::Message(format!("pg_ctl stop spawn failed: {err}")))?;
    let label = format!("pg_ctl stop for {}", data_dir.display());
    let status = wait_for_child_exit_with_timeout(&label, &mut child, E2E_COMMAND_TIMEOUT).await?;

    if status.success() || !pid_file.exists() {
        Ok(())
    } else {
        Err(WorkerError::Message(format!(
            "pg_ctl stop exited unsuccessfully with status {status} for {}",
            data_dir.display()
        )))
    }
}

async fn wait_for_child_exit_with_timeout(
    label: &str,
    child: &mut Child,
    timeout: Duration,
) -> Result<ExitStatus, WorkerError> {
    match tokio::time::timeout(timeout, child.wait()).await {
        Ok(wait_result) => {
            wait_result.map_err(|err| WorkerError::Message(format!("{label} wait failed: {err}")))
        }
        Err(_) => {
            child.start_kill().map_err(|err| {
                WorkerError::Message(format!(
                    "{label} timed out after {}s and kill failed: {err}",
                    timeout.as_secs()
                ))
            })?;
            match tokio::time::timeout(E2E_COMMAND_KILL_WAIT_TIMEOUT, child.wait()).await {
                Ok(Ok(_)) | Ok(Err(_)) | Err(_) => {}
            }
            Err(WorkerError::Message(format!(
                "{label} timed out after {}s and was killed",
                timeout.as_secs()
            )))
        }
    }
}

async fn run_psql_statement(psql: &Path, port: u16, sql: &str) -> Result<String, WorkerError> {
    let mut command = Command::new(psql);
    command
        .arg("-h")
        .arg("127.0.0.1")
        .arg("-p")
        .arg(port.to_string())
        .arg("-U")
        .arg("postgres")
        .arg("-d")
        .arg("postgres")
        .arg("-v")
        .arg("ON_ERROR_STOP=1")
        .arg("-AXqt")
        .arg("-c")
        .arg(sql)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .map_err(|err| WorkerError::Message(format!("psql spawn failed: {err}")))?;
    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| WorkerError::Message("psql stdout pipe unavailable".to_string()))?;
    let mut stderr = child
        .stderr
        .take()
        .ok_or_else(|| WorkerError::Message("psql stderr pipe unavailable".to_string()))?;

    let stdout_task = tokio::spawn(async move {
        let mut buffer = Vec::new();
        stdout
            .read_to_end(&mut buffer)
            .await
            .map(|_| buffer)
            .map_err(|err| WorkerError::Message(format!("psql stdout read failed: {err}")))
    });
    let stderr_task = tokio::spawn(async move {
        let mut buffer = Vec::new();
        stderr
            .read_to_end(&mut buffer)
            .await
            .map(|_| buffer)
            .map_err(|err| WorkerError::Message(format!("psql stderr read failed: {err}")))
    });

    let label = format!("psql port={port}");
    let status = wait_for_child_exit_with_timeout(&label, &mut child, E2E_COMMAND_TIMEOUT).await?;
    let stdout_bytes = stdout_task
        .await
        .map_err(|err| WorkerError::Message(format!("psql stdout join failed: {err}")))??;
    let stderr_bytes = stderr_task
        .await
        .map_err(|err| WorkerError::Message(format!("psql stderr join failed: {err}")))??;

    let stdout_text = String::from_utf8(stdout_bytes)
        .map_err(|err| WorkerError::Message(format!("psql stdout utf8 decode failed: {err}")))?;
    if status.success() {
        return Ok(stdout_text);
    }

    let stderr_text = String::from_utf8(stderr_bytes)
        .map_err(|err| WorkerError::Message(format!("psql stderr utf8 decode failed: {err}")))?;
    Err(WorkerError::Message(format!(
        "psql exited unsuccessfully with status {status}; stderr={}",
        stderr_text.trim()
    )))
}

fn parse_psql_rows(output: &str) -> Vec<String> {
    output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn parse_single_u64(output: &str) -> Result<u64, WorkerError> {
    let rows = parse_psql_rows(output);
    if rows.len() != 1 {
        return Err(WorkerError::Message(format!(
            "expected one scalar row, got {} rows: {rows:?}",
            rows.len()
        )));
    }
    rows[0].parse::<u64>().map_err(|err| {
        WorkerError::Message(format!("parse scalar u64 from '{}' failed: {err}", rows[0]))
    })
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
        match run_psql_statement(workload.psql_bin.as_path(), target.port, write_sql.as_str()).await
        {
            Ok(_) => {
                stats.committed_writes = stats.committed_writes.saturating_add(1);
                stats.committed_keys.push(format!("{worker_id}:{seq}"));
                let committed_at = match unix_now() {
                    Ok(value) => value.0,
                    Err(_) => 0,
                };
                stats.committed_at_unix_ms.push(committed_at);
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
        match run_psql_statement(workload.psql_bin.as_path(), target.port, read_sql.as_str()).await
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

struct ApiHttpResponse {
    status_code: u16,
    body: Vec<u8>,
}

async fn send_http_request_with_worker(
    node: &mut NodeFixture,
    method: &str,
    path: &str,
    body: Option<&[u8]>,
    content_type: Option<&str>,
) -> Result<ApiHttpResponse, WorkerError> {
    let mut stream = match tokio::time::timeout(
        E2E_HTTP_STEP_TIMEOUT,
        tokio::net::TcpStream::connect(node.api_addr),
    )
    .await
    {
        Ok(Ok(stream)) => stream,
        Ok(Err(err)) => {
            return Err(WorkerError::Message(format!(
                "connect {} for {method} {path} failed: {err}",
                node.api_addr
            )))
        }
        Err(_) => {
            return Err(WorkerError::Message(format!(
                "connect {} for {method} {path} timed out after {}s",
                node.api_addr,
                E2E_HTTP_STEP_TIMEOUT.as_secs()
            )))
        }
    };

    let payload = body.unwrap_or(&[]);
    let mut request =
        format!("{method} {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n");
    if !payload.is_empty() {
        let ct = content_type.unwrap_or("application/json");
        request.push_str(&format!("Content-Type: {ct}\r\n"));
        request.push_str(&format!("Content-Length: {}\r\n", payload.len()));
    }
    request.push_str("\r\n");

    match tokio::time::timeout(E2E_HTTP_STEP_TIMEOUT, stream.write_all(request.as_bytes())).await {
        Ok(Ok(())) => {}
        Ok(Err(err)) => {
            return Err(WorkerError::Message(format!(
                "write request headers for {method} {path} failed: {err}"
            )))
        }
        Err(_) => {
            return Err(WorkerError::Message(format!(
                "write request headers for {method} {path} timed out after {}s",
                E2E_HTTP_STEP_TIMEOUT.as_secs()
            )))
        }
    }
    if !payload.is_empty() {
        match tokio::time::timeout(E2E_HTTP_STEP_TIMEOUT, stream.write_all(payload)).await {
            Ok(Ok(())) => {}
            Ok(Err(err)) => {
                return Err(WorkerError::Message(format!(
                    "write request body for {method} {path} failed: {err}"
                )))
            }
            Err(_) => {
                return Err(WorkerError::Message(format!(
                    "write request body for {method} {path} timed out after {}s",
                    E2E_HTTP_STEP_TIMEOUT.as_secs()
                )))
            }
        }
    }

    match tokio::time::timeout(
        E2E_HTTP_STEP_TIMEOUT,
        crate::debug_api::worker::step_once(&mut node.debug_ctx),
    )
    .await
    {
        Ok(Ok(())) => {}
        Ok(Err(err)) => {
            return Err(WorkerError::Message(format!(
                "debug snapshot step for {method} {path} failed: {err}"
            )))
        }
        Err(_) => {
            return Err(WorkerError::Message(format!(
                "debug snapshot step for {method} {path} timed out after {}s",
                E2E_HTTP_STEP_TIMEOUT.as_secs()
            )))
        }
    }
    match tokio::time::timeout(
        E2E_HTTP_STEP_TIMEOUT,
        crate::api::worker::step_once(&mut node.api_ctx),
    )
    .await
    {
        Ok(Ok(())) => {}
        Ok(Err(err)) => {
            return Err(WorkerError::Message(format!(
                "api step for {method} {path} failed: {err}"
            )))
        }
        Err(_) => {
            return Err(WorkerError::Message(format!(
                "api step for {method} {path} timed out after {}s",
                E2E_HTTP_STEP_TIMEOUT.as_secs()
            )))
        }
    }

    let mut raw = Vec::new();
    match tokio::time::timeout(E2E_HTTP_STEP_TIMEOUT, stream.read_to_end(&mut raw)).await {
        Ok(Ok(_)) => {}
        Ok(Err(err)) => {
            return Err(WorkerError::Message(format!(
                "read response for {method} {path} failed: {err}"
            )))
        }
        Err(_) => {
            return Err(WorkerError::Message(format!(
                "read response for {method} {path} timed out after {}s",
                E2E_HTTP_STEP_TIMEOUT.as_secs()
            )))
        }
    }
    parse_http_response(raw.as_slice())
}

fn parse_http_response(raw: &[u8]) -> Result<ApiHttpResponse, WorkerError> {
    let status_line_end = raw
        .windows(2)
        .position(|window| window == b"\r\n")
        .ok_or_else(|| WorkerError::Message("missing status line terminator".to_string()))?;
    let status_line = String::from_utf8_lossy(&raw[..status_line_end]);
    let mut parts = status_line.split_whitespace();
    let _http_version = parts
        .next()
        .ok_or_else(|| WorkerError::Message("missing http version".to_string()))?;
    let status_code = parts
        .next()
        .ok_or_else(|| WorkerError::Message("missing status code".to_string()))?
        .parse::<u16>()
        .map_err(|err| WorkerError::Message(format!("invalid status code: {err}")))?;

    let header_end = raw
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or_else(|| WorkerError::Message("missing header/body separator".to_string()))?;
    let body_start = header_end
        .checked_add(4)
        .ok_or_else(|| WorkerError::Message("response body offset overflow".to_string()))?;
    let body = raw
        .get(body_start..)
        .ok_or_else(|| WorkerError::Message("response body slice out of bounds".to_string()))?
        .to_vec();

    Ok(ApiHttpResponse { status_code, body })
}

fn expect_accepted_response(action: &str, response: ApiHttpResponse) -> Result<(), WorkerError> {
    if response.status_code != 202 {
        let body = String::from_utf8_lossy(&response.body);
        return Err(WorkerError::Message(format!(
            "{action} returned status {} body={}",
            response.status_code,
            body.trim()
        )));
    }

    let decoded = serde_json::from_slice::<AcceptedResponse>(&response.body).map_err(|err| {
        WorkerError::Message(format!("{action} decode accepted response failed: {err}"))
    })?;
    if !decoded.accepted {
        return Err(WorkerError::Message(format!(
            "{action} response accepted=false"
        )));
    }
    Ok(())
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

#[tokio::test(flavor = "current_thread")]
async fn e2e_multi_node_unassisted_failover_sql_consistency() -> Result<(), WorkerError> {
    let binaries = resolve_pg_binaries_for_real_tests()?;
    let etcd_bin = resolve_etcd_bin_for_real_tests()?;
    let mut fixture = ClusterFixture::start(3, binaries, etcd_bin).await?;
    let mut phase_history: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    let run_result = match tokio::time::timeout(E2E_SCENARIO_TIMEOUT, async {
        fixture.record("unassisted failover bootstrap: wait for stable primary");
        let bootstrap_primary = fixture
            .wait_for_stable_primary(
                Duration::from_secs(60),
                None,
                5,
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
        let pre_rows = parse_psql_rows(pre_rows_raw.as_str());
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
                .run_sql_on_node_with_retry(
                    &replica_id,
                    "CREATE TABLE IF NOT EXISTS ha_unassisted_failover_proof (id INTEGER PRIMARY KEY, payload TEXT NOT NULL)",
                    Duration::from_secs(20),
                )
                .await?;
            fixture
                .run_sql_on_node_with_retry(
                    &replica_id,
                    "INSERT INTO ha_unassisted_failover_proof (id, payload) VALUES (1, 'before') ON CONFLICT (id) DO UPDATE SET payload = EXCLUDED.payload",
                    Duration::from_secs(20),
                )
                .await?;
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

        fixture.record("unassisted failover recovery: API-only polling for new stable primary");
        let failover_primary = fixture
            .wait_for_stable_primary(
                Duration::from_secs(90),
                Some(&bootstrap_primary),
                5,
                &mut phase_history,
            )
            .await?;
        fixture
            .assert_no_dual_primary_window(Duration::from_secs(5))
            .await?;
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
        let post_rows = parse_psql_rows(post_rows_raw.as_str());
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
}

#[tokio::test(flavor = "current_thread")]
async fn e2e_multi_node_real_ha_scenario_matrix() -> Result<(), WorkerError> {
    let binaries = resolve_pg_binaries_for_real_tests()?;
    let etcd_bin = resolve_etcd_bin_for_real_tests()?;
    let mut fixture = ClusterFixture::start(3, binaries, etcd_bin).await?;

    let run_result = match tokio::time::timeout(E2E_SCENARIO_TIMEOUT, async {
        fixture.record("scenario bootstrap/election: wait for single primary");
        let bootstrap_primary = fixture.wait_for_primary(Duration::from_secs(45)).await?;
        fixture.record(format!(
            "bootstrap/election success: primary={bootstrap_primary}"
        ));
        fixture
            .assert_no_dual_primary_window(Duration::from_secs(3))
            .await?;

        fixture.record("scenario planned switchover: submit request through API endpoint");
        fixture.post_switchover_via_api("e2e-controller").await?;
        let switchover_primary = fixture
            .wait_for_primary_change(&bootstrap_primary, Duration::from_secs(45))
            .await?;
        fixture.record(format!(
            "planned switchover success: old_primary={bootstrap_primary}, new_primary={switchover_primary}"
        ));

        fixture.record("scenario split-brain prevention: ensure no dual primary after switchover");
        fixture
            .assert_no_dual_primary_window(Duration::from_secs(5))
            .await?;

        fixture.record("scenario no-quorum fail-safe: shutdown etcd majority (2/3)");
        let stopped_members = fixture.stop_etcd_majority(2).await?;
        fixture.record(format!(
            "no-quorum setup: stopped etcd members={}",
            stopped_members.join(",")
        ));
        fixture.wait_for_all_failsafe(Duration::from_secs(45)).await?;
        fixture.record("no-quorum fail-safe observed on all nodes");
        Ok(())
    })
    .await
    {
        Ok(run_result) => run_result,
        Err(_) => {
            fixture.record(format!(
                "scenario timed out after {}s",
                E2E_SCENARIO_TIMEOUT.as_secs()
            ));
            Err(WorkerError::Message(format!(
                "scenario timed out after {}s",
                E2E_SCENARIO_TIMEOUT.as_secs()
            )))
        }
    };

    let artifact_path = fixture.write_timeline_artifact("ha-e2e-scenario-matrix");
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
}

#[tokio::test(flavor = "current_thread")]
async fn e2e_multi_node_stress_planned_switchover_concurrent_sql() -> Result<(), WorkerError> {
    let binaries = resolve_pg_binaries_for_real_tests()?;
    let etcd_bin = resolve_etcd_bin_for_real_tests()?;
    let mut fixture = ClusterFixture::start(3, binaries, etcd_bin).await?;
    let scenario_name = "ha-e2e-stress-planned-switchover-concurrent-sql";

    let run_result = match tokio::time::timeout(E2E_SCENARIO_TIMEOUT, async {
        let started_at_unix_ms = unix_now()?.0;
        let mut phase_history: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let workload_spec = SqlWorkloadSpec {
            scenario_name: scenario_name.to_string(),
            table_name: "ha_stress_switchover".to_string(),
            worker_count: 4,
            run_interval_ms: 250,
        };
        let table_name = sanitize_sql_identifier(workload_spec.table_name.as_str());

        fixture.record("stress switchover bootstrap: wait for stable primary");
        let bootstrap_primary = fixture
            .wait_for_stable_primary(Duration::from_secs(60), None, 5, &mut phase_history)
            .await?;
        fixture
            .prepare_stress_table(&bootstrap_primary, table_name.as_str())
            .await?;
        let workload_handle = fixture.start_sql_workload(workload_spec.clone()).await?;
        tokio::time::sleep(Duration::from_secs(3)).await;

        fixture.record("stress switchover: trigger API switchover while workload is active");
        fixture
            .post_switchover_via_api("e2e-stress-switchover")
            .await?;
        let ha_stats = fixture
            .sample_ha_states_window(Duration::from_secs(8), Duration::from_millis(150), 80)
            .await?;
        let switchover_primary = fixture
            .wait_for_stable_primary(
                Duration::from_secs(90),
                Some(&bootstrap_primary),
                5,
                &mut phase_history,
            )
            .await?;
        fixture
            .assert_former_primary_demoted_after_transition(&bootstrap_primary)
            .await?;
        fixture
            .assert_no_dual_primary_window(Duration::from_secs(5))
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
        let workload = fixture
            .stop_sql_workload_and_collect(workload_handle, Duration::from_secs(2))
            .await?;
        if workload.committed_writes == 0 {
            return Err(WorkerError::Message(
                "stress switchover workload committed zero writes".to_string(),
            ));
        }
        ClusterFixture::assert_no_split_brain_write_evidence(&workload, &ha_stats)?;

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
        let finished_at_unix_ms = unix_now()?.0;
        Ok(StressScenarioSummary {
            schema_version: STRESS_SUMMARY_SCHEMA_VERSION,
            scenario: scenario_name.to_string(),
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
                StressScenarioSummary::failed(scenario_name, message.clone()),
                Some(message),
            )
        }
    };
    let artifacts = fixture.write_stress_artifacts(scenario_name, &summary);
    let shutdown_result = fixture.shutdown().await;
    finalize_stress_scenario_result(run_error, artifacts, shutdown_result)
}

#[tokio::test(flavor = "current_thread")]
async fn e2e_multi_node_stress_unassisted_failover_concurrent_sql() -> Result<(), WorkerError> {
    let binaries = resolve_pg_binaries_for_real_tests()?;
    let etcd_bin = resolve_etcd_bin_for_real_tests()?;
    let mut fixture = ClusterFixture::start(3, binaries, etcd_bin).await?;
    let scenario_name = "ha-e2e-stress-unassisted-failover-concurrent-sql";

    let run_result = match tokio::time::timeout(E2E_SCENARIO_TIMEOUT, async {
        let started_at_unix_ms = unix_now()?.0;
        let mut phase_history: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let workload_spec = SqlWorkloadSpec {
            scenario_name: scenario_name.to_string(),
            table_name: "ha_stress_failover".to_string(),
            worker_count: 4,
            run_interval_ms: 250,
        };
        let table_name = sanitize_sql_identifier(workload_spec.table_name.as_str());

        fixture.record("stress failover bootstrap: wait for stable primary");
        let bootstrap_primary = fixture
            .wait_for_stable_primary(Duration::from_secs(60), None, 5, &mut phase_history)
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
        let failover_primary = fixture
            .wait_for_stable_primary(
                Duration::from_secs(120),
                Some(&bootstrap_primary),
                5,
                &mut phase_history,
            )
            .await?;
        ClusterFixture::assert_phase_history_contains_failover(
            &phase_history,
            &bootstrap_primary,
            &failover_primary,
        )?;
        fixture
            .assert_former_primary_demoted_after_transition(&bootstrap_primary)
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
        let workload = fixture
            .stop_sql_workload_and_collect(workload_handle, Duration::from_secs(2))
            .await?;
        if workload.committed_writes == 0 {
            return Err(WorkerError::Message(
                "stress failover workload committed zero writes".to_string(),
            ));
        }
        ClusterFixture::assert_no_split_brain_write_evidence(&workload, &ha_stats)?;

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

        let finished_at_unix_ms = unix_now()?.0;
        Ok(StressScenarioSummary {
            schema_version: STRESS_SUMMARY_SCHEMA_VERSION,
            scenario: scenario_name.to_string(),
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
                StressScenarioSummary::failed(scenario_name, message.clone()),
                Some(message),
            )
        }
    };
    let artifacts = fixture.write_stress_artifacts(scenario_name, &summary);
    let shutdown_result = fixture.shutdown().await;
    finalize_stress_scenario_result(run_error, artifacts, shutdown_result)
}

#[tokio::test(flavor = "current_thread")]
async fn e2e_multi_node_stress_no_quorum_fencing_with_concurrent_sql() -> Result<(), WorkerError> {
    let binaries = resolve_pg_binaries_for_real_tests()?;
    let etcd_bin = resolve_etcd_bin_for_real_tests()?;
    let mut fixture = ClusterFixture::start(3, binaries, etcd_bin).await?;
    let scenario_name = "ha-e2e-stress-no-quorum-fencing-concurrent-sql";

    let run_result = match tokio::time::timeout(E2E_SCENARIO_TIMEOUT, async {
        let started_at_unix_ms = unix_now()?.0;
        let mut phase_history: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let workload_spec = SqlWorkloadSpec {
            scenario_name: scenario_name.to_string(),
            table_name: "ha_stress_fencing".to_string(),
            worker_count: 4,
            run_interval_ms: 250,
        };
        let table_name = sanitize_sql_identifier(workload_spec.table_name.as_str());
        fixture.record("stress no-quorum bootstrap: wait for stable primary");
        let bootstrap_primary = fixture
            .wait_for_stable_primary(
                Duration::from_secs(60),
                None,
                5,
                &mut phase_history,
            )
            .await?;
        fixture
            .prepare_stress_table(&bootstrap_primary, table_name.as_str())
            .await?;
        let workload_handle = fixture.start_sql_workload(workload_spec.clone()).await?;
        tokio::time::sleep(Duration::from_secs(3)).await;

        fixture.record("stress no-quorum: stop etcd majority while workload active");
        let stopped_members = fixture.stop_etcd_majority(2).await?;
        fixture.record(format!(
            "stress no-quorum members stopped: {}",
            stopped_members.join(",")
        ));
        fixture.wait_for_all_failsafe(Duration::from_secs(60)).await?;
        let failsafe_observed_at_ms = unix_now()?.0;
        let ha_stats = fixture
            .sample_ha_states_window(Duration::from_secs(8), Duration::from_millis(150), 100)
            .await?;

        tokio::time::sleep(Duration::from_secs(7)).await;
        let workload = fixture
            .stop_sql_workload_and_collect(workload_handle, Duration::from_secs(2))
            .await?;
        if workload.committed_writes == 0 {
            return Err(WorkerError::Message(
                "stress no-quorum workload committed zero writes".to_string(),
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
        let fencing_grace_ms = 5_000u64;
        let cutoff_ms = failsafe_observed_at_ms.saturating_add(fencing_grace_ms);
        let commits_after_cutoff = workload
            .committed_at_unix_ms
            .iter()
            .filter(|timestamp| **timestamp > cutoff_ms)
            .count();
        let allowed_post_cutoff_commits = 10usize;
        if commits_after_cutoff > allowed_post_cutoff_commits {
            return Err(WorkerError::Message(format!(
                "writes still committed after fail-safe fencing cutoff beyond tolerance; cutoff_ms={cutoff_ms} commits_after_cutoff={commits_after_cutoff} allowed={allowed_post_cutoff_commits}"
            )));
        }
        ClusterFixture::assert_no_split_brain_write_evidence(&workload, &ha_stats)?;

        let primary_row_count = fixture
            .assert_table_key_integrity_on_node(
                &bootstrap_primary,
                table_name.as_str(),
                1,
                Duration::from_secs(90),
            )
            .await?;
        fixture.record(format!(
            "stress no-quorum key integrity verified on {bootstrap_primary} with row_count={primary_row_count}"
        ));
        let finished_at_unix_ms = unix_now()?.0;
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
                format!("failsafe_observed_at_ms={failsafe_observed_at_ms}"),
                format!("fencing_cutoff_ms={cutoff_ms}"),
                format!("allowed_post_cutoff_commits={allowed_post_cutoff_commits}"),
            ],
        })
    })
    .await
    {
        Ok(run_result) => run_result,
        Err(_) => Err(WorkerError::Message(format!(
            "stress no-quorum scenario timed out after {}s",
            E2E_SCENARIO_TIMEOUT.as_secs()
        ))),
    };

    let (summary, run_error) = match run_result {
        Ok(summary) => (summary, None),
        Err(err) => {
            let message = err.to_string();
            (
                StressScenarioSummary::failed(scenario_name, message.clone()),
                Some(message),
            )
        }
    };
    let artifacts = fixture.write_stress_artifacts(scenario_name, &summary);
    let shutdown_result = fixture.shutdown().await;
    finalize_stress_scenario_result(run_error, artifacts, shutdown_result)
}
