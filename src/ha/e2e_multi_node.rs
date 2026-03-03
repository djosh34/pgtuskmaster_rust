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

    async fn post_set_leader_via_api(&mut self, member_id: &str) -> Result<(), WorkerError> {
        #[derive(serde::Serialize)]
        struct SetLeaderBody<'a> {
            member_id: &'a str,
        }

        let body = serde_json::to_vec(&SetLeaderBody { member_id }).map_err(|err| {
            WorkerError::Message(format!("encode set leader request failed: {err}"))
        })?;
        let response = self
            .send_node_request(
                self.control_node_index()?,
                "POST",
                "/ha/leader",
                Some(&body),
                Some("application/json"),
            )
            .await?;
        expect_accepted_response("POST /ha/leader", response)
    }

    async fn delete_leader_via_api(&mut self) -> Result<(), WorkerError> {
        let response = self
            .send_node_request(
                self.control_node_index()?,
                "DELETE",
                "/ha/leader",
                None,
                None,
            )
            .await?;
        expect_accepted_response("DELETE /ha/leader", response)
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

    async fn wait_for_api_leader_target(
        &mut self,
        target: &str,
        timeout: Duration,
    ) -> Result<(), WorkerError> {
        let deadline = tokio::time::Instant::now() + timeout;
        let mut last_error: Option<String> = None;
        loop {
            match self.cluster_ha_states().await {
                Ok(states) => {
                    let leaders_match = !states.is_empty()
                        && states
                            .iter()
                            .all(|state| state.leader.as_deref() == Some(target));
                    if leaders_match {
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
                    "timed out waiting for API leader target {target}; last_error={detail}"
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

    async fn wait_for_fencing_signal(
        &mut self,
        node_id: &str,
        timeout: Duration,
    ) -> Result<(), WorkerError> {
        let deadline = tokio::time::Instant::now() + timeout;
        let mut last_error: Option<String> = None;
        loop {
            if let Some(index) = self.node_index_by_id(node_id) {
                let phase_is_fencing = match self.fetch_node_ha_state_by_index(index).await {
                    Ok(state) => state.ha_phase == "Fencing",
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
                let has_fencing_job = matches!(
                    process_state,
                    ProcessState::Running { ref active, .. }
                        if active.kind == ActiveJobKind::Fencing
                            || active.kind == ActiveJobKind::Demote
                );
                if phase_is_fencing || has_fencing_job {
                    return Ok(());
                }
            }
            if tokio::time::Instant::now() >= deadline {
                let detail = last_error
                    .as_deref()
                    .map_or_else(|| "none".to_string(), ToString::to_string);
                return Err(WorkerError::Message(format!(
                    "timed out waiting for fencing signal on {node_id}; last_error={detail}"
                )));
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
        let safe_scenario: String = scenario
            .chars()
            .map(|ch| {
                if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                    ch
                } else {
                    '-'
                }
            })
            .collect();
        let artifact_path = artifact_dir.join(format!("{safe_scenario}-{stamp}.timeline.log"));
        fs::write(&artifact_path, self.timeline.join("\n"))
            .map_err(|err| WorkerError::Message(format!("write timeline failed: {err}")))?;
        Ok(artifact_path)
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

fn resolve_pg_binaries_for_real_tests() -> Result<Option<BinaryPaths>, WorkerError> {
    let postgres = match require_pg16_bin_for_real_tests("postgres") {
        Ok(Some(path)) => path,
        Ok(None) => return Ok(None),
        Err(err) => {
            return Err(WorkerError::Message(format!(
                "postgres binary lookup failed: {err}"
            )))
        }
    };
    let pg_ctl = match require_pg16_bin_for_real_tests("pg_ctl") {
        Ok(Some(path)) => path,
        Ok(None) => return Ok(None),
        Err(err) => {
            return Err(WorkerError::Message(format!(
                "pg_ctl binary lookup failed: {err}"
            )))
        }
    };
    let pg_rewind = match require_pg16_bin_for_real_tests("pg_rewind") {
        Ok(Some(path)) => path,
        Ok(None) => return Ok(None),
        Err(err) => {
            return Err(WorkerError::Message(format!(
                "pg_rewind binary lookup failed: {err}"
            )))
        }
    };
    let initdb = match require_pg16_bin_for_real_tests("initdb") {
        Ok(Some(path)) => path,
        Ok(None) => return Ok(None),
        Err(err) => {
            return Err(WorkerError::Message(format!(
                "initdb binary lookup failed: {err}"
            )))
        }
    };
    let psql = match require_pg16_bin_for_real_tests("psql") {
        Ok(Some(path)) => path,
        Ok(None) => return Ok(None),
        Err(err) => {
            return Err(WorkerError::Message(format!(
                "psql binary lookup failed: {err}"
            )))
        }
    };
    Ok(Some(BinaryPaths {
        postgres,
        pg_ctl,
        pg_rewind,
        initdb,
        psql,
    }))
}

fn resolve_etcd_bin_for_real_tests() -> Result<Option<PathBuf>, WorkerError> {
    match require_etcd_bin_for_real_tests() {
        Ok(path) => Ok(path),
        Err(err) => Err(WorkerError::Message(format!(
            "etcd binary lookup failed: {err}"
        ))),
    }
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

#[tokio::test(flavor = "current_thread")]
async fn e2e_multi_node_unassisted_failover_sql_consistency() -> Result<(), WorkerError> {
    let binaries = match resolve_pg_binaries_for_real_tests()? {
        Some(paths) => paths,
        None => return Ok(()),
    };
    let etcd_bin = match resolve_etcd_bin_for_real_tests()? {
        Some(path) => path,
        None => return Ok(()),
    };
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
    let binaries = match resolve_pg_binaries_for_real_tests()? {
        Some(paths) => paths,
        None => return Ok(()),
    };
    let etcd_bin = match resolve_etcd_bin_for_real_tests()? {
        Some(path) => path,
        None => return Ok(()),
    };
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

        fixture.record("scenario fencing-before-promotion: inject conflicting leader");
        let conflict_target = fixture
            .nodes
            .iter()
            .find(|node| node.id != switchover_primary)
            .map(|node| node.id.clone())
            .ok_or_else(|| WorkerError::Message("no conflict target node found".to_string()))?;
        fixture.post_set_leader_via_api(&conflict_target).await?;
        fixture
            .wait_for_fencing_signal(&switchover_primary, Duration::from_secs(30))
            .await?;
        fixture
            .assert_no_dual_primary_window(Duration::from_secs(3))
            .await?;
        fixture.record(format!(
            "fencing-before-promotion observed on {switchover_primary}"
        ));

        fixture.record("scenario failover: stop current primary and clear stale leader through API");
        fixture.stop_postgres_for_node(&switchover_primary).await?;
        fixture.delete_leader_via_api().await?;
        let failover_primary = fixture
            .nodes
            .iter()
            .find(|node| node.id != switchover_primary)
            .map(|node| node.id.clone())
            .ok_or_else(|| WorkerError::Message("no failover target found".to_string()))?;
        fixture.post_set_leader_via_api(&failover_primary).await?;
        fixture
            .wait_for_api_leader_target(&failover_primary, Duration::from_secs(45))
            .await?;
        fixture.record(format!(
            "failover signal observed: failed_primary={switchover_primary}, api_leader={failover_primary}"
        ));

        fixture.record("scenario rewind path: verify former primary enters rewind/process recovery path");
        fixture
            .wait_for_rewind_or_process_outcome(&switchover_primary, Duration::from_secs(45))
            .await?;
        fixture.record(format!(
            "rewind/recovery path observed for former primary {switchover_primary}"
        ));

        fixture.record("scenario split-brain prevention: ensure no dual primary after failover");
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
