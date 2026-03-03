use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use tokio::{process::Command, task::JoinHandle};

use crate::{
    api::controller::{post_switchover, SwitchoverRequestInput},
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
        store::DcsStore,
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
        etcd3::{prepare_etcd_data_dir, spawn_etcd3, EtcdHandle, EtcdInstanceSpec},
        namespace::NamespaceGuard,
        pg16::prepare_pgdata_dir,
        ports::allocate_ports,
    },
};

struct NodeFixture {
    id: String,
    data_dir: PathBuf,
    ha_subscriber: StateSubscriber<HaState>,
    dcs_subscriber: StateSubscriber<DcsState>,
    process_subscriber: StateSubscriber<ProcessState>,
    _config_publisher: StatePublisher<RuntimeConfig>,
}

struct ClusterFixture {
    _guard: NamespaceGuard,
    scope: String,
    endpoint: String,
    pg_ctl_bin: PathBuf,
    etcd: Option<EtcdHandle>,
    nodes: Vec<NodeFixture>,
    tasks: Vec<JoinHandle<Result<(), WorkerError>>>,
    timeline: Vec<String>,
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

        let ports_needed = node_count.saturating_add(2);
        let reservation = allocate_ports(ports_needed)
            .map_err(|err| WorkerError::Message(format!("allocate ports failed: {err}")))?;
        let ports = reservation.as_slice().to_vec();

        let etcd_client_port = ports[0];
        let etcd_peer_port = ports[1];
        let node_ports = ports[2..].to_vec();
        let etcd_data_dir = prepare_etcd_data_dir(namespace)
            .map_err(|err| WorkerError::Message(format!("prepare etcd data dir failed: {err}")))?;
        let log_dir = namespace.child_dir("logs/etcd");
        let spec = EtcdInstanceSpec {
            etcd_bin,
            namespace_id: namespace.id.clone(),
            member_name: "etcd-a".to_string(),
            data_dir: etcd_data_dir,
            log_dir,
            client_port: etcd_client_port,
            peer_port: etcd_peer_port,
            startup_timeout: Duration::from_secs(15),
        };

        // Release the reserved ports immediately before child bind.
        drop(reservation);
        let etcd = spawn_etcd3(spec)
            .await
            .map_err(|err| WorkerError::Message(format!("spawn etcd failed: {err}")))?;
        let endpoint = format!("http://127.0.0.1:{etcd_client_port}");

        let pg_ctl_bin = binaries.pg_ctl.clone();
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
                    endpoints: vec![endpoint.clone()],
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

            let pg_ctx = crate::pginfo::state::PgInfoWorkerCtx {
                self_id: MemberId(node_id.clone()),
                postgres_dsn: format!(
                    "host=127.0.0.1 port={pg_port} user=postgres dbname=postgres"
                ),
                poll_interval: Duration::from_millis(100),
                publisher: pg_publisher,
            };

            let dcs_store = EtcdDcsStore::connect(vec![endpoint.clone()], &scope)
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

            let ha_store =
                EtcdDcsStore::connect(vec![endpoint.clone()], &scope).map_err(|err| {
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
                data_dir,
                ha_subscriber,
                dcs_subscriber,
                process_subscriber,
                _config_publisher: cfg_publisher,
            });
        }

        Ok(Self {
            _guard: guard,
            scope,
            endpoint,
            pg_ctl_bin,
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

    fn current_primary(&self) -> Option<String> {
        for node in &self.nodes {
            if node.ha_subscriber.latest().value.phase == HaPhase::Primary {
                return Some(node.id.clone());
            }
        }
        None
    }

    fn primary_count(&self) -> usize {
        self.nodes
            .iter()
            .filter(|node| node.ha_subscriber.latest().value.phase == HaPhase::Primary)
            .count()
    }

    fn node_by_id(&self, id: &str) -> Option<&NodeFixture> {
        self.nodes.iter().find(|node| node.id == id)
    }

    async fn wait_for_primary(&self, timeout: Duration) -> Result<String, WorkerError> {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            if self.primary_count() == 1 {
                if let Some(primary) = self.current_primary() {
                    return Ok(primary);
                }
            }
            if tokio::time::Instant::now() >= deadline {
                return Err(WorkerError::Message(
                    "timed out waiting for single primary".to_string(),
                ));
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    async fn wait_for_primary_change(
        &self,
        previous: &str,
        timeout: Duration,
    ) -> Result<String, WorkerError> {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            if self.primary_count() == 1 {
                if let Some(primary) = self.current_primary() {
                    if primary != previous {
                        return Ok(primary);
                    }
                }
            }
            if tokio::time::Instant::now() >= deadline {
                return Err(WorkerError::Message(format!(
                    "timed out waiting for primary change from {previous}"
                )));
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    async fn wait_for_dcs_leader_target(
        &self,
        target: &str,
        timeout: Duration,
    ) -> Result<(), WorkerError> {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            let leader_seen = self.nodes.iter().any(|node| {
                node.dcs_subscriber
                    .latest()
                    .value
                    .cache
                    .leader
                    .as_ref()
                    .map(|leader| leader.member_id.0.as_str() == target)
                    .unwrap_or(false)
            });
            if leader_seen {
                return Ok(());
            }
            if tokio::time::Instant::now() >= deadline {
                return Err(WorkerError::Message(format!(
                    "timed out waiting for DCS leader target {target}"
                )));
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    async fn assert_no_dual_primary_window(&self, window: Duration) -> Result<(), WorkerError> {
        let deadline = tokio::time::Instant::now() + window;
        loop {
            if self.primary_count() > 1 {
                return Err(WorkerError::Message(
                    "split-brain detected: more than one primary".to_string(),
                ));
            }
            if tokio::time::Instant::now() >= deadline {
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(75)).await;
        }
    }

    async fn wait_for_fencing_signal(
        &self,
        node_id: &str,
        timeout: Duration,
    ) -> Result<(), WorkerError> {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            if let Some(node) = self.node_by_id(node_id) {
                let phase = node.ha_subscriber.latest().value.phase.clone();
                let process_state = node.process_subscriber.latest().value;
                let has_fencing_job = matches!(
                    process_state,
                    ProcessState::Running { ref active, .. }
                        if active.kind == ActiveJobKind::Fencing
                            || active.kind == ActiveJobKind::Demote
                );
                if phase == HaPhase::Fencing || has_fencing_job {
                    return Ok(());
                }
            }
            if tokio::time::Instant::now() >= deadline {
                return Err(WorkerError::Message(format!(
                    "timed out waiting for fencing signal on {node_id}"
                )));
            }
            tokio::time::sleep(Duration::from_millis(75)).await;
        }
    }

    async fn wait_for_rewind_or_process_outcome(
        &self,
        node_id: &str,
        timeout: Duration,
    ) -> Result<(), WorkerError> {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            if let Some(node) = self.node_by_id(node_id) {
                let phase = node.ha_subscriber.latest().value.phase.clone();
                let process_state = node.process_subscriber.latest().value;
                let saw_rewind_phase = phase == HaPhase::Rewinding;
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
                return Err(WorkerError::Message(format!(
                    "timed out waiting for rewind path on {node_id}"
                )));
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    async fn wait_for_all_failsafe(&self, timeout: Duration) -> Result<(), WorkerError> {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            let all_failsafe = self
                .nodes
                .iter()
                .all(|node| node.ha_subscriber.latest().value.phase == HaPhase::FailSafe);
            if all_failsafe {
                return Ok(());
            }
            if tokio::time::Instant::now() >= deadline {
                return Err(WorkerError::Message(
                    "timed out waiting for all nodes to enter fail-safe".to_string(),
                ));
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

    fn write_timeline_artifact(&self) -> Result<PathBuf, WorkerError> {
        let artifact_dir =
            Path::new(env!("CARGO_MANIFEST_DIR")).join(".ralph/evidence/13-e2e-multi-node");
        fs::create_dir_all(&artifact_dir)
            .map_err(|err| WorkerError::Message(format!("create artifact dir failed: {err}")))?;
        let stamp = unix_now()?.0;
        let artifact_path =
            artifact_dir.join(format!("ha-e2e-scenario-matrix-{stamp}.timeline.log"));
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
            etcd.shutdown()
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
    let status = Command::new(initdb)
        .arg("-D")
        .arg(data_dir)
        .arg("-A")
        .arg("trust")
        .arg("-U")
        .arg("postgres")
        .status()
        .await
        .map_err(|err| WorkerError::Message(format!("initdb spawn failed: {err}")))?;
    if status.success() {
        Ok(())
    } else {
        Err(WorkerError::Message(format!(
            "initdb exited unsuccessfully with status {status}"
        )))
    }
}

async fn pg_ctl_stop_immediate(pg_ctl: &Path, data_dir: &Path) -> Result<(), WorkerError> {
    let output = Command::new(pg_ctl)
        .arg("-D")
        .arg(data_dir)
        .arg("stop")
        .arg("-m")
        .arg("immediate")
        .arg("-w")
        .output()
        .await
        .map_err(|err| WorkerError::Message(format!("pg_ctl stop spawn failed: {err}")))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let already_stopped = stderr.contains("PID file") && stderr.contains("does not exist");
        if already_stopped {
            Ok(())
        } else {
            Err(WorkerError::Message(format!(
                "pg_ctl stop exited unsuccessfully with status {}; stderr: {}",
                output.status,
                stderr.trim()
            )))
        }
    }
}

fn leader_path(scope: &str) -> String {
    format!("/{}/leader", scope.trim_matches('/'))
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
    let mut control_store =
        EtcdDcsStore::connect(vec![fixture.endpoint.clone()], &fixture.scope)
            .map_err(|err| WorkerError::Message(format!("control store connect failed: {err}")))?;

    let run_result: Result<(), WorkerError> = async {
        fixture.record("scenario bootstrap/election: wait for single primary");
        let bootstrap_primary = fixture.wait_for_primary(Duration::from_secs(45)).await?;
        fixture.record(format!(
            "bootstrap/election success: primary={bootstrap_primary}"
        ));
        fixture
            .assert_no_dual_primary_window(Duration::from_secs(3))
            .await?;

        fixture.record("scenario planned switchover: submit request through API controller");
        post_switchover(
            &fixture.scope,
            &mut control_store,
            SwitchoverRequestInput {
                requested_by: MemberId("e2e-controller".to_string()),
            },
        )
        .map_err(|err| WorkerError::Message(format!("post switchover failed: {err}")))?;
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
        let leader_payload = serde_json::to_string(&crate::dcs::state::LeaderRecord {
            member_id: MemberId(conflict_target.clone()),
        })
        .map_err(|err| WorkerError::Message(format!("leader encode failed: {err}")))?;
        control_store
            .write_path(&leader_path(&fixture.scope), leader_payload)
            .map_err(|err| WorkerError::Message(format!("inject leader key failed: {err}")))?;
        fixture
            .wait_for_fencing_signal(&switchover_primary, Duration::from_secs(30))
            .await?;
        fixture
            .assert_no_dual_primary_window(Duration::from_secs(3))
            .await?;
        fixture.record(format!(
            "fencing-before-promotion observed on {switchover_primary}"
        ));

        fixture.record("scenario failover: stop current primary and remove stale leader key");
        fixture.stop_postgres_for_node(&switchover_primary).await?;
        control_store
            .delete_path(&leader_path(&fixture.scope))
            .map_err(|err| WorkerError::Message(format!("delete leader key failed: {err}")))?;
        let failover_primary = fixture
            .nodes
            .iter()
            .find(|node| node.id != switchover_primary)
            .map(|node| node.id.clone())
            .ok_or_else(|| WorkerError::Message("no failover target found".to_string()))?;
        let failover_payload = serde_json::to_string(&crate::dcs::state::LeaderRecord {
            member_id: MemberId(failover_primary.clone()),
        })
        .map_err(|err| WorkerError::Message(format!("failover leader encode failed: {err}")))?;
        control_store
            .write_path(&leader_path(&fixture.scope), failover_payload)
            .map_err(|err| WorkerError::Message(format!("set failover leader failed: {err}")))?;
        fixture
            .wait_for_dcs_leader_target(&failover_primary, Duration::from_secs(45))
            .await?;
        fixture.record(format!(
            "failover signal observed: failed_primary={switchover_primary}, dcs_leader={failover_primary}"
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

        fixture.record("scenario no-quorum fail-safe: shutdown etcd");
        if let Some(etcd) = fixture.etcd.as_mut() {
            etcd.shutdown().await.map_err(|err| {
                WorkerError::Message(format!("etcd shutdown for no-quorum failed: {err}"))
            })?;
            fixture.etcd = None;
        }
        fixture.wait_for_all_failsafe(Duration::from_secs(45)).await?;
        fixture.record("no-quorum fail-safe observed on all nodes");
        Ok(())
    }
    .await;

    let artifact_path = fixture.write_timeline_artifact();
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
