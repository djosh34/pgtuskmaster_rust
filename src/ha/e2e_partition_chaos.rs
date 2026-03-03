use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
    process::{ExitStatus, Stdio},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use tokio::{
    io::AsyncReadExt,
    process::{Child, Command},
    task::JoinHandle,
};

use crate::{
    api::HaStateResponse,
    config::{
        schema::{
            ApiConfig, ClusterConfig, DcsConfig, DebugConfig, HaConfig, PostgresConfig,
            SecurityConfig,
        },
        BinaryPaths, ProcessConfig, RuntimeConfig,
    },
    state::{UnixMillis, WorkerError},
    test_harness::{
        binaries::{require_etcd_bin_for_real_tests, require_pg16_bin_for_real_tests},
        etcd3::{
            prepare_etcd_member_data_dir, spawn_etcd3_cluster, EtcdClusterHandle,
            EtcdClusterMemberSpec, EtcdClusterSpec,
        },
        namespace::NamespaceGuard,
        net_proxy::{ProxyLinkSpec, ProxyMode, TcpProxyLink},
        pg16::prepare_pgdata_dir,
        ports::{allocate_ha_topology_ports, allocate_ports},
    },
};

const E2E_COMMAND_TIMEOUT: Duration = Duration::from_secs(30);
const E2E_COMMAND_KILL_WAIT_TIMEOUT: Duration = Duration::from_secs(3);
const E2E_HTTP_STEP_TIMEOUT: Duration = Duration::from_secs(20);
const E2E_BOOTSTRAP_PRIMARY_TIMEOUT: Duration = Duration::from_secs(60);
const E2E_SCENARIO_TIMEOUT: Duration = Duration::from_secs(360);
const PARTITION_ARTIFACT_DIR: &str = ".ralph/evidence/28-e2e-network-partition-chaos";

struct NodeFixture {
    id: String,
    pg_port: u16,
    pg_proxy_addr: SocketAddr,
    api_addr: SocketAddr,
    api_proxy_addr: SocketAddr,
    data_dir: PathBuf,
    log_file: PathBuf,
}

struct PartitionFixture {
    _guard: NamespaceGuard,
    _scope: String,
    pg_ctl_bin: PathBuf,
    psql_bin: PathBuf,
    etcd: Option<EtcdClusterHandle>,
    nodes: Vec<NodeFixture>,
    tasks: Vec<JoinHandle<Result<(), WorkerError>>>,
    etcd_links_by_node: BTreeMap<String, Vec<String>>,
    etcd_proxies: BTreeMap<String, TcpProxyLink>,
    api_proxies: BTreeMap<String, TcpProxyLink>,
    pg_proxies: BTreeMap<String, TcpProxyLink>,
    timeline: Vec<String>,
}

impl PartitionFixture {
    async fn start(
        node_count: usize,
        binaries: BinaryPaths,
        etcd_bin: PathBuf,
    ) -> Result<Self, WorkerError> {
        let guard = NamespaceGuard::new("ha-e2e-partition")
            .map_err(|err| WorkerError::Message(format!("namespace create failed: {err}")))?;
        let namespace = guard
            .namespace()
            .map_err(|err| WorkerError::Message(format!("namespace lookup failed: {err}")))?;
        let scope = "scope-ha-e2e-partition".to_string();

        let reservation = allocate_ha_topology_ports(node_count, 3)
            .map_err(|err| WorkerError::Message(format!("allocate ports failed: {err}")))?;
        let topology = reservation.into_layout();
        let node_ports = topology.node_ports;

        let mut forbidden_ports: BTreeSet<u16> = topology
            .etcd_client_ports
            .iter()
            .chain(topology.etcd_peer_ports.iter())
            .chain(node_ports.iter())
            .copied()
            .collect();

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
        let member_names = etcd.member_names();

        if endpoints.is_empty() {
            return Err(WorkerError::Message(
                "etcd cluster returned no endpoints".to_string(),
            ));
        }
        if member_names.len() != endpoints.len() {
            return Err(WorkerError::Message(format!(
                "etcd members/endpoints mismatch: members={} endpoints={}",
                member_names.len(),
                endpoints.len()
            )));
        }

        let api_ports = allocate_non_overlapping_ports(node_count, &forbidden_ports)?;
        for port in &api_ports {
            forbidden_ports.insert(*port);
        }

        let total_proxy_ports = node_count
            .checked_add(node_count)
            .and_then(|value| value.checked_add(node_count))
            .ok_or_else(|| {
                WorkerError::Message("proxy port count overflow for partition fixture".to_string())
            })?;
        let proxy_ports = allocate_non_overlapping_ports(total_proxy_ports, &forbidden_ports)?;

        let mut cursor = 0usize;
        let next_port = |ports: &[u16], cursor_ref: &mut usize| -> Result<u16, WorkerError> {
            if *cursor_ref >= ports.len() {
                return Err(WorkerError::Message(
                    "proxy port allocation cursor out of bounds".to_string(),
                ));
            }
            let selected = ports[*cursor_ref];
            *cursor_ref = cursor_ref.saturating_add(1);
            Ok(selected)
        };

        let mut etcd_proxies: BTreeMap<String, TcpProxyLink> = BTreeMap::new();
        let mut etcd_links_by_node: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut dcs_endpoints_by_node: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for node_index in 0..node_count {
            let node_id = format!("node-{}", node_index.saturating_add(1));
            let mut proxy_urls = Vec::with_capacity(1);
            let mut link_keys = Vec::with_capacity(1);
            let endpoint_index = node_index % endpoints.len();
            let member_name = member_names.get(endpoint_index).ok_or_else(|| {
                WorkerError::Message(format!(
                    "missing etcd member name for endpoint index {endpoint_index}"
                ))
            })?;
            let endpoint = endpoints.get(endpoint_index).ok_or_else(|| {
                WorkerError::Message(format!(
                    "missing etcd endpoint for index {endpoint_index}"
                ))
            })?;
            let proxy_port = next_port(proxy_ports.as_slice(), &mut cursor)?;
            let target_addr = parse_http_endpoint(endpoint.as_str())?;
            let link_name = format!("{node_id}-to-{member_name}-etcd");
            let listen_addr = parse_loopback_socket(proxy_port)?;
            let link = TcpProxyLink::spawn(ProxyLinkSpec {
                name: link_name.clone(),
                listen_addr,
                target_addr,
            })
            .await
            .map_err(|err| {
                WorkerError::Message(format!("spawn etcd proxy {link_name} failed: {err}"))
            })?;
            let proxy_url = format!("http://{}", link.listen_addr());
            proxy_urls.push(proxy_url);
            link_keys.push(link_name.clone());
            etcd_proxies.insert(link_name, link);
            dcs_endpoints_by_node.insert(node_id.clone(), proxy_urls);
            etcd_links_by_node.insert(node_id, link_keys);
        }

        let mut api_proxies: BTreeMap<String, TcpProxyLink> = BTreeMap::new();
        let mut pg_proxies: BTreeMap<String, TcpProxyLink> = BTreeMap::new();
        let pg_ctl_bin = binaries.pg_ctl.clone();
        let psql_bin = binaries.psql.clone();
        let mut tasks = Vec::new();
        let mut nodes = Vec::new();
        let rewind_source_port = *node_ports.first().ok_or_else(|| {
            WorkerError::Message("missing postgres ports for cluster startup".to_string())
        })?;

        for (index, (pg_port, api_port)) in node_ports.into_iter().zip(api_ports).enumerate() {
            let node_id = format!("node-{}", index.saturating_add(1));
            let data_dir = prepare_pgdata_dir(namespace, &node_id).map_err(|err| {
                WorkerError::Message(format!("prepare pg data dir failed: {err}"))
            })?;
            let socket_dir = namespace.child_dir(format!("run/{node_id}"));
            let log_file = namespace.child_dir(format!("logs/{node_id}/postgres.log"));
            let api_addr: SocketAddr = format!("127.0.0.1:{api_port}")
                .parse()
                .map_err(|err| WorkerError::Message(format!("parse api addr failed: {err}")))?;

            let api_proxy_port = next_port(proxy_ports.as_slice(), &mut cursor)?;
            let api_proxy_name = format!("{node_id}-api-proxy");
            let api_proxy = TcpProxyLink::spawn(ProxyLinkSpec {
                name: api_proxy_name,
                listen_addr: parse_loopback_socket(api_proxy_port)?,
                target_addr: api_addr,
            })
            .await
            .map_err(|err| {
                WorkerError::Message(format!("spawn API proxy for {node_id} failed: {err}"))
            })?;
            let api_proxy_addr = api_proxy.listen_addr();
            api_proxies.insert(node_id.clone(), api_proxy);

            let pg_proxy_port = next_port(proxy_ports.as_slice(), &mut cursor)?;
            let pg_proxy_name = format!("{node_id}-pg-proxy");
            let pg_target_addr = parse_loopback_socket(pg_port)?;
            let pg_proxy = TcpProxyLink::spawn(ProxyLinkSpec {
                name: pg_proxy_name,
                listen_addr: parse_loopback_socket(pg_proxy_port)?,
                target_addr: pg_target_addr,
            })
            .await
            .map_err(|err| {
                WorkerError::Message(format!("spawn postgres proxy for {node_id} failed: {err}"))
            })?;
            let pg_proxy_addr = pg_proxy.listen_addr();
            pg_proxies.insert(node_id.clone(), pg_proxy);

            let dcs_endpoints = dcs_endpoints_by_node
                .get(node_id.as_str())
                .cloned()
                .ok_or_else(|| {
                    WorkerError::Message(format!(
                        "missing proxy DCS endpoints for node runtime config: {node_id}"
                    ))
                })?;

            let runtime_cfg = RuntimeConfig {
                cluster: ClusterConfig {
                    name: "cluster-e2e-partition".to_string(),
                    member_id: node_id.clone(),
                },
                postgres: PostgresConfig {
                    data_dir: data_dir.clone(),
                    connect_timeout_s: 2,
                    listen_host: "127.0.0.1".to_string(),
                    listen_port: pg_port,
                    socket_dir,
                    log_file: log_file.clone(),
                    rewind_source_host: "127.0.0.1".to_string(),
                    rewind_source_port,
                },
                dcs: DcsConfig {
                    endpoints: dcs_endpoints,
                    scope: scope.clone(),
                },
                ha: HaConfig {
                    loop_interval_ms: 100,
                    lease_ttl_ms: 2_000,
                },
                process: ProcessConfig {
                    pg_rewind_timeout_ms: 5_000,
                    bootstrap_timeout_ms: 30_000,
                    fencing_timeout_ms: 5_000,
                    binaries: binaries.clone(),
                },
                api: ApiConfig {
                    listen_addr: api_addr.to_string(),
                    read_auth_token: None,
                    admin_auth_token: None,
                },
                debug: DebugConfig { enabled: false },
                security: SecurityConfig {
                    tls_enabled: false,
                    auth_token: None,
                },
            };

            let task_node_id = node_id.clone();
            tasks.push(tokio::task::spawn_local(async move {
                match crate::runtime::run_node_from_config(runtime_cfg).await {
                    Ok(()) => Ok(()),
                    Err(err) => Err(WorkerError::Message(format!(
                        "runtime node {task_node_id} exited with error: {err}"
                    ))),
                }
            }));

            nodes.push(NodeFixture {
                id: node_id.clone(),
                pg_port,
                pg_proxy_addr,
                api_addr,
                api_proxy_addr,
                data_dir,
                log_file: log_file.clone(),
            });

            let task_handle = tasks.last_mut().ok_or_else(|| {
                WorkerError::Message("missing runtime task after node spawn".to_string())
            })?;
            wait_for_node_api_ready_or_task_exit(
                api_proxy_addr,
                node_id.as_str(),
                log_file.as_path(),
                task_handle,
                Duration::from_secs(120),
            )
            .await?;
            if index == 0 {
                let expected_member_id = format!("node-{}", index.saturating_add(1));
                wait_for_bootstrap_primary(
                    api_proxy_addr,
                    expected_member_id.as_str(),
                    E2E_BOOTSTRAP_PRIMARY_TIMEOUT,
                )
                .await?;
            }
        }

        if cursor != proxy_ports.len() {
            return Err(WorkerError::Message(format!(
                "proxy port cursor mismatch: used={cursor} allocated={}",
                proxy_ports.len()
            )));
        }

        Ok(Self {
            _guard: guard,
            _scope: scope,
            pg_ctl_bin,
            psql_bin,
            etcd: Some(etcd),
            nodes,
            tasks,
            etcd_links_by_node,
            etcd_proxies,
            api_proxies,
            pg_proxies,
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

    fn node_ids(&self) -> Vec<String> {
        self.nodes.iter().map(|node| node.id.clone()).collect()
    }

    fn control_node_id(&self) -> Result<String, WorkerError> {
        self.nodes
            .first()
            .map(|node| node.id.clone())
            .ok_or_else(|| WorkerError::Message("no nodes available".to_string()))
    }

    fn node_by_id(&self, node_id: &str) -> Option<&NodeFixture> {
        self.nodes.iter().find(|node| node.id == node_id)
    }

    async fn set_etcd_mode_for_node(
        &self,
        node_id: &str,
        mode: ProxyMode,
    ) -> Result<(), WorkerError> {
        let links = self
            .etcd_links_by_node
            .get(node_id)
            .ok_or_else(|| WorkerError::Message(format!("unknown node for etcd partition: {node_id}")))?;
        for link_name in links {
            let link = self.etcd_proxies.get(link_name.as_str()).ok_or_else(|| {
                WorkerError::Message(format!(
                    "missing etcd proxy link for node {node_id}: {link_name}"
                ))
            })?;
            link.set_mode(mode.clone())
                .await
                .map_err(|err| WorkerError::Message(format!("set mode on {link_name} failed: {err}")))?;
        }
        Ok(())
    }

    async fn partition_node_from_etcd(&mut self, node_id: &str) -> Result<(), WorkerError> {
        self.record(format!(
            "network fault: partition node from etcd majority node={node_id}"
        ));
        self.set_etcd_mode_for_node(node_id, ProxyMode::Blocked).await
    }

    async fn partition_primary_from_etcd(&mut self, node_id: &str) -> Result<(), WorkerError> {
        self.record(format!(
            "network fault: partition current primary from etcd node={node_id}"
        ));
        self.set_etcd_mode_for_node(node_id, ProxyMode::Blocked).await
    }

    async fn isolate_api_path(&mut self, node_id: &str) -> Result<(), WorkerError> {
        self.record(format!("network fault: isolate API path for node={node_id}"));
        let link = self
            .api_proxies
            .get(node_id)
            .ok_or_else(|| WorkerError::Message(format!("missing api proxy for node: {node_id}")))?;
        link.set_mode(ProxyMode::Blocked)
            .await
            .map_err(|err| WorkerError::Message(format!("set api proxy mode failed: {err}")))
    }

    async fn heal_all_network_faults(&mut self) -> Result<(), WorkerError> {
        self.record("network heal: reset all proxy links to pass-through".to_string());
        for link in self.etcd_proxies.values() {
            link.set_mode(ProxyMode::PassThrough)
                .await
                .map_err(|err| WorkerError::Message(format!("heal etcd proxy failed: {err}")))?;
        }
        for link in self.api_proxies.values() {
            link.set_mode(ProxyMode::PassThrough)
                .await
                .map_err(|err| WorkerError::Message(format!("heal api proxy failed: {err}")))?;
        }
        for link in self.pg_proxies.values() {
            link.set_mode(ProxyMode::PassThrough)
                .await
                .map_err(|err| WorkerError::Message(format!("heal postgres proxy failed: {err}")))?;
        }
        Ok(())
    }

    async fn fetch_node_ha_state(&self, node_id: &str) -> Result<HaStateResponse, WorkerError> {
        let node = self
            .node_by_id(node_id)
            .ok_or_else(|| WorkerError::Message(format!("unknown node id for HA state: {node_id}")))?;
        let url = format!("http://{}/ha/state", node.api_proxy_addr);
        let client = reqwest::Client::builder()
            .timeout(E2E_HTTP_STEP_TIMEOUT)
            .build()
            .map_err(|err| WorkerError::Message(format!("build reqwest client failed: {err}")))?;
        let response = client
            .get(url.as_str())
            .send()
            .await
            .map_err(|err| WorkerError::Message(format!("GET /ha/state request failed: {err}")))?;
        if response.status() != reqwest::StatusCode::OK {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "<body read failed>".to_string());
            return Err(WorkerError::Message(format!(
                "GET /ha/state returned status={status} body={} node={node_id}",
                body.trim()
            )));
        }
        response
            .json::<HaStateResponse>()
            .await
            .map_err(|err| WorkerError::Message(format!("decode /ha/state response failed: {err}")))
    }

    async fn cluster_ha_states_best_effort(
        &mut self,
    ) -> Result<(Vec<HaStateResponse>, Vec<String>), WorkerError> {
        self.ensure_runtime_tasks_healthy().await?;
        let mut states = Vec::new();
        let mut errors = Vec::new();
        for node_id in self.node_ids() {
            match self.fetch_node_ha_state(node_id.as_str()).await {
                Ok(state) => states.push(state),
                Err(err) => errors.push(format!("node={node_id} error={err}")),
            }
        }
        Ok((states, errors))
    }

    async fn wait_for_stable_primary(
        &mut self,
        timeout: Duration,
        excluded_primary: Option<&str>,
        required_consecutive: usize,
    ) -> Result<String, WorkerError> {
        if required_consecutive == 0 {
            return Err(WorkerError::Message(
                "required_consecutive must be greater than zero".to_string(),
            ));
        }

        let deadline = tokio::time::Instant::now() + timeout;
        let mut stable_count = 0usize;
        let mut last_candidate: Option<String> = None;

        loop {
            let (states, errors) = self.cluster_ha_states_best_effort().await?;
            let primaries = Self::primary_members(states.as_slice());
            let state_summary = states
                .iter()
                .map(|state| {
                    let leader = state.leader.as_deref().unwrap_or("none");
                    format!("{}:{}:leader={leader}", state.self_member_id, state.ha_phase)
                })
                .collect::<Vec<_>>()
                .join(", ");
            let error_summary = errors.join(" | ");

            if primaries.len() == 1 {
                let candidate = primaries[0].clone();
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

            if tokio::time::Instant::now() >= deadline {
                return Err(WorkerError::Message(format!(
                    "timed out waiting for stable primary; excluded={excluded_primary:?}; last_observation=states=[{state_summary}] errors=[{error_summary}]"
                )));
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    fn primary_members(states: &[HaStateResponse]) -> Vec<String> {
        states
            .iter()
            .filter(|state| state.ha_phase == "Primary")
            .map(|state| state.self_member_id.clone())
            .collect()
    }

    async fn assert_no_dual_primary_window(&mut self, window: Duration) -> Result<(), WorkerError> {
        let deadline = tokio::time::Instant::now() + window;
        loop {
            let (states, errors) = self.cluster_ha_states_best_effort().await?;
            let primary_count = Self::primary_members(states.as_slice()).len();
            if primary_count > 1 {
                return Err(WorkerError::Message(format!(
                    "split-brain detected: more than one primary; observations={} errors={}",
                    states
                        .iter()
                        .map(|state| format!("{}:{}", state.self_member_id, state.ha_phase))
                        .collect::<Vec<_>>()
                        .join(","),
                    errors.join(" | ")
                )));
            }

            if tokio::time::Instant::now() >= deadline {
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(75)).await;
        }
    }

    async fn wait_for_node_phase(
        &self,
        node_id: &str,
        expected_phase: &str,
        timeout: Duration,
    ) -> Result<(), WorkerError> {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            let observation = match self.fetch_node_ha_state(node_id).await {
                Ok(state) => {
                    if state.ha_phase == expected_phase {
                        return Ok(());
                    }
                    format!(
                        "phase={} leader={:?}",
                        state.ha_phase, state.leader
                    )
                }
                Err(err) => err.to_string(),
            };
            if tokio::time::Instant::now() >= deadline {
                return Err(WorkerError::Message(format!(
                    "timed out waiting for node {node_id} phase {expected_phase}; last_observation={observation}"
                )));
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    async fn run_sql_on_node(&self, node_id: &str, sql: &str) -> Result<String, WorkerError> {
        let node = self
            .node_by_id(node_id)
            .ok_or_else(|| WorkerError::Message(format!("unknown node for SQL: {node_id}")))?;
        run_psql_statement(self.psql_bin.as_path(), node.pg_proxy_addr.port(), sql).await
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
                "SELECT COALESCE(string_agg(id::text || ':' || payload, ',' ORDER BY id), '') FROM {table_name}"
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

    fn write_timeline_artifact(&self, scenario: &str) -> Result<PathBuf, WorkerError> {
        let artifact_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(PARTITION_ARTIFACT_DIR);
        fs::create_dir_all(&artifact_dir)
            .map_err(|err| WorkerError::Message(format!("create artifact dir failed: {err}")))?;
        let stamp = unix_now()?.0;
        let safe_scenario = sanitize_component(scenario);
        let artifact_path = artifact_dir.join(format!("{safe_scenario}-{stamp}.timeline.log"));
        fs::write(&artifact_path, self.timeline.join("\n"))
            .map_err(|err| WorkerError::Message(format!("write timeline failed: {err}")))?;
        Ok(artifact_path)
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

    async fn shutdown(&mut self) -> Result<(), WorkerError> {
        let mut failures = Vec::new();

        for task in &self.tasks {
            task.abort();
        }
        while let Some(task) = self.tasks.pop() {
            let _ = task.await;
        }

        for node in &self.nodes {
            if let Err(err) = pg_ctl_stop_immediate(&self.pg_ctl_bin, &node.data_dir).await {
                failures.push(format!("postgres stop {} failed: {err}", node.id));
            }
        }

        let etcd_proxy_map = std::mem::take(&mut self.etcd_proxies);
        for (name, proxy) in etcd_proxy_map {
            if let Err(err) = proxy.shutdown().await {
                failures.push(format!("etcd proxy {name} shutdown failed: {err}"));
            }
        }

        let api_proxy_map = std::mem::take(&mut self.api_proxies);
        for (name, proxy) in api_proxy_map {
            if let Err(err) = proxy.shutdown().await {
                failures.push(format!("api proxy {name} shutdown failed: {err}"));
            }
        }

        let pg_proxy_map = std::mem::take(&mut self.pg_proxies);
        for (name, proxy) in pg_proxy_map {
            if let Err(err) = proxy.shutdown().await {
                failures.push(format!("postgres proxy {name} shutdown failed: {err}"));
            }
        }

        if let Some(etcd) = self.etcd.as_mut() {
            if let Err(err) = etcd.shutdown_all().await {
                failures.push(format!("etcd shutdown failed: {err}"));
            }
        }
        self.etcd = None;

        if failures.is_empty() {
            Ok(())
        } else {
            Err(WorkerError::Message(format!(
                "partition fixture shutdown failures: {}",
                failures.join("; ")
            )))
        }
    }
}

fn parse_loopback_socket(port: u16) -> Result<SocketAddr, WorkerError> {
    format!("127.0.0.1:{port}")
        .parse::<SocketAddr>()
        .map_err(|err| WorkerError::Message(format!("parse socket failed for port={port}: {err}")))
}

fn parse_http_endpoint(endpoint: &str) -> Result<SocketAddr, WorkerError> {
    let host_port = endpoint.strip_prefix("http://").ok_or_else(|| {
        WorkerError::Message(format!("unsupported endpoint format for proxy target: {endpoint}"))
    })?;
    host_port
        .parse::<SocketAddr>()
        .map_err(|err| WorkerError::Message(format!("parse endpoint socket failed: {endpoint} ({err})")))
}

fn allocate_non_overlapping_ports(
    count: usize,
    forbidden: &BTreeSet<u16>,
) -> Result<Vec<u16>, WorkerError> {
    if count == 0 {
        return Ok(Vec::new());
    }

    for _attempt in 0..30 {
        let candidate = allocate_ports(count)
            .map_err(|err| WorkerError::Message(format!("allocate ports failed: {err}")))?
            .into_vec();
        let overlaps = candidate.iter().any(|port| forbidden.contains(port));
        if !overlaps {
            return Ok(candidate);
        }
    }

    Err(WorkerError::Message(format!(
        "failed to allocate {count} non-overlapping ports after retries"
    )))
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

async fn wait_for_node_api_ready_or_task_exit(
    node_addr: SocketAddr,
    node_id: &str,
    postgres_log_file: &Path,
    task: &mut JoinHandle<Result<(), WorkerError>>,
    timeout: Duration,
) -> Result<(), WorkerError> {
    let deadline = tokio::time::Instant::now() + timeout;
    let client = reqwest::Client::builder()
        .timeout(E2E_HTTP_STEP_TIMEOUT)
        .build()
        .map_err(|err| WorkerError::Message(format!("build reqwest client failed: {err}")))?;

    loop {
        if task.is_finished() {
            let joined = task.await.map_err(|err| {
                WorkerError::Message(format!("runtime task join failed for {node_id}: {err}"))
            })?;
            return match joined {
                Ok(()) => Err(WorkerError::Message(format!(
                    "runtime task exited unexpectedly for {node_id} before API became ready"
                ))),
                Err(err) => Err(WorkerError::Message(format!(
                    "runtime task failed for {node_id} before API became ready: {err}; postgres_log_tail={}",
                    read_log_tail(postgres_log_file, 40)
                ))),
            };
        }

        let url = format!("http://{node_addr}/ha/state");
        let observation = match client.get(url.as_str()).send().await {
            Ok(response) if response.status() == reqwest::StatusCode::OK => return Ok(()),
            Ok(response) => {
                let status = response.status();
                let body = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "<body read failed>".to_string());
                format!("status={status} body={}", body.trim())
            }
            Err(err) => err.to_string(),
        };

        if tokio::time::Instant::now() >= deadline {
            return Err(WorkerError::Message(format!(
                "timed out waiting for api readiness for {node_id} at {node_addr}; last_observation={observation}; postgres_log_tail={}",
                read_log_tail(postgres_log_file, 40)
            )));
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

fn read_log_tail(path: &Path, max_lines: usize) -> String {
    let content = match fs::read_to_string(path) {
        Ok(value) => value,
        Err(err) => return format!("log-read-failed: {err}"),
    };
    let mut lines = content.lines().collect::<Vec<_>>();
    if lines.is_empty() {
        return "empty".to_string();
    }
    if lines.len() > max_lines {
        let start = lines.len().saturating_sub(max_lines);
        lines = lines[start..].to_vec();
    }
    lines.join(" | ")
}

async fn wait_for_bootstrap_primary(
    node_addr: SocketAddr,
    expected_member_id: &str,
    timeout: Duration,
) -> Result<(), WorkerError> {
    let deadline = tokio::time::Instant::now() + timeout;
    let client = reqwest::Client::builder()
        .timeout(E2E_HTTP_STEP_TIMEOUT)
        .build()
        .map_err(|err| WorkerError::Message(format!("build reqwest client failed: {err}")))?;

    loop {
        let url = format!("http://{node_addr}/ha/state");
        let observation = match client.get(url.as_str()).send().await {
            Ok(response) if response.status() == reqwest::StatusCode::OK => {
                let state = response.json::<HaStateResponse>().await.map_err(|err| {
                    WorkerError::Message(format!("decode /ha/state response failed: {err}"))
                })?;
                let is_expected_primary =
                    state.self_member_id == expected_member_id && state.ha_phase == "Primary";
                if is_expected_primary {
                    return Ok(());
                }
                let leader = state.leader.as_deref().unwrap_or("none");
                format!(
                    "member={} phase={} leader={leader}",
                    state.self_member_id, state.ha_phase
                )
            }
            Ok(response) => {
                let status = response.status();
                let body = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "<body read failed>".to_string());
                format!("status={status} body={}", body.trim())
            }
            Err(err) => err.to_string(),
        };

        if tokio::time::Instant::now() >= deadline {
            return Err(WorkerError::Message(format!(
                "timed out waiting for bootstrap primary {expected_member_id} at {node_addr}; last_observation={observation}"
            )));
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
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

fn unix_now() -> Result<UnixMillis, WorkerError> {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| WorkerError::Message(format!("system time before epoch: {err}")))?;
    let millis = u64::try_from(elapsed.as_millis())
        .map_err(|err| WorkerError::Message(format!("millis conversion failed: {err}")))?;
    Ok(UnixMillis(millis))
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

async fn finalize_partition_scenario(
    fixture: &mut PartitionFixture,
    scenario_name: &str,
    run_result: Result<(), WorkerError>,
) -> Result<(), WorkerError> {
    let artifact_result = fixture.write_timeline_artifact(scenario_name);
    let shutdown_result = fixture.shutdown().await;

    match (run_result, artifact_result, shutdown_result) {
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

async fn run_with_local_set<F>(future: F) -> Result<(), WorkerError>
where
    F: std::future::Future<Output = Result<(), WorkerError>>,
{
    tokio::task::LocalSet::new().run_until(future).await
}

#[tokio::test(flavor = "current_thread")]
async fn e2e_partition_minority_isolation_no_split_brain_rejoin() -> Result<(), WorkerError> {
    run_with_local_set(async {
        let binaries = resolve_pg_binaries_for_real_tests()?;
        let etcd_bin = resolve_etcd_bin_for_real_tests()?;
        let mut fixture = PartitionFixture::start(3, binaries, etcd_bin).await?;
        let scenario_name = "ha-e2e-partition-minority-isolation";

        let run_result = match tokio::time::timeout(E2E_SCENARIO_TIMEOUT, async {
            fixture.record("minority isolation: wait for initial stable primary");
            let bootstrap_primary = fixture
                .wait_for_stable_primary(Duration::from_secs(90), None, 5)
                .await?;
            fixture.record(format!("minority isolation: initial primary={bootstrap_primary}"));

            let isolated_replica = fixture
                .node_ids()
                .into_iter()
                .find(|node_id| node_id != &bootstrap_primary)
                .ok_or_else(|| WorkerError::Message("no replica found for isolation scenario".to_string()))?;

            fixture
                .run_sql_on_node_with_retry(
                    &bootstrap_primary,
                    "CREATE TABLE IF NOT EXISTS ha_partition_minority (id INTEGER PRIMARY KEY, payload TEXT NOT NULL)",
                    Duration::from_secs(30),
                )
                .await?;
            fixture
                .run_sql_on_node_with_retry(
                    &bootstrap_primary,
                    "INSERT INTO ha_partition_minority (id, payload) VALUES (1, 'before') ON CONFLICT (id) DO UPDATE SET payload = EXCLUDED.payload",
                    Duration::from_secs(30),
                )
                .await?;

            fixture
                .partition_node_from_etcd(isolated_replica.as_str())
                .await?;
            fixture
                .assert_no_dual_primary_window(Duration::from_secs(8))
                .await?;

            tokio::time::sleep(Duration::from_secs(4)).await;
            fixture.heal_all_network_faults().await?;

            let healed_primary = fixture
                .wait_for_stable_primary(Duration::from_secs(90), None, 5)
                .await?;
            fixture.record(format!("minority isolation: healed primary={healed_primary}"));
            fixture
                .run_sql_on_node_with_retry(
                    &healed_primary,
                    "INSERT INTO ha_partition_minority (id, payload) VALUES (2, 'after') ON CONFLICT (id) DO UPDATE SET payload = EXCLUDED.payload",
                    Duration::from_secs(45),
                )
                .await?;

            let expected_rows = vec!["1:before".to_string(), "2:after".to_string()];
            for node_id in fixture.node_ids() {
                fixture
                    .wait_for_rows_on_node(
                        node_id.as_str(),
                        "SELECT id::text || ':' || payload FROM ha_partition_minority ORDER BY id",
                        expected_rows.as_slice(),
                        Duration::from_secs(90),
                    )
                    .await?;
            }
            fixture
                .wait_for_table_digest_convergence(
                    "ha_partition_minority",
                    fixture.node_ids().as_slice(),
                    2,
                    Duration::from_secs(90),
                )
                .await?;
            fixture
                .assert_no_dual_primary_window(Duration::from_secs(5))
                .await?;
            Ok(())
        })
        .await
        {
            Ok(run_result) => run_result,
            Err(_) => Err(WorkerError::Message(format!(
                "{scenario_name} timed out after {}s",
                E2E_SCENARIO_TIMEOUT.as_secs()
            ))),
        };

        finalize_partition_scenario(&mut fixture, scenario_name, run_result).await
    })
    .await
}

#[tokio::test(flavor = "current_thread")]
async fn e2e_partition_primary_isolation_failover_no_split_brain() -> Result<(), WorkerError> {
    run_with_local_set(async {
        let binaries = resolve_pg_binaries_for_real_tests()?;
        let etcd_bin = resolve_etcd_bin_for_real_tests()?;
        let mut fixture = PartitionFixture::start(3, binaries, etcd_bin).await?;
        let scenario_name = "ha-e2e-partition-primary-isolation";

        let run_result = match tokio::time::timeout(E2E_SCENARIO_TIMEOUT, async {
            fixture.record("primary isolation: wait for initial stable primary");
            let bootstrap_primary = fixture
                .wait_for_stable_primary(Duration::from_secs(90), None, 5)
                .await?;
            fixture.record(format!("primary isolation: initial primary={bootstrap_primary}"));

            fixture
                .run_sql_on_node_with_retry(
                    &bootstrap_primary,
                    "CREATE TABLE IF NOT EXISTS ha_partition_primary (id INTEGER PRIMARY KEY, payload TEXT NOT NULL)",
                    Duration::from_secs(30),
                )
                .await?;
            fixture
                .run_sql_on_node_with_retry(
                    &bootstrap_primary,
                    "INSERT INTO ha_partition_primary (id, payload) VALUES (1, 'before') ON CONFLICT (id) DO UPDATE SET payload = EXCLUDED.payload",
                    Duration::from_secs(30),
                )
                .await?;

            fixture
                .partition_primary_from_etcd(bootstrap_primary.as_str())
                .await?;
            fixture
                .wait_for_node_phase(
                    bootstrap_primary.as_str(),
                    "FailSafe",
                    Duration::from_secs(120),
                )
                .await?;
            fixture.record(format!(
                "primary isolation: isolated primary entered FailSafe node={bootstrap_primary}"
            ));
            fixture
                .assert_no_dual_primary_window(Duration::from_secs(10))
                .await?;

            fixture.heal_all_network_faults().await?;
            let healed_primary = fixture
                .wait_for_stable_primary(Duration::from_secs(120), None, 5)
                .await?;
            fixture.record(format!("primary isolation: healed primary={healed_primary}"));
            fixture
                .run_sql_on_node_with_retry(
                    &healed_primary,
                    "INSERT INTO ha_partition_primary (id, payload) VALUES (2, 'after') ON CONFLICT (id) DO UPDATE SET payload = EXCLUDED.payload",
                    Duration::from_secs(45),
                )
                .await?;

            let expected_rows = vec!["1:before".to_string(), "2:after".to_string()];
            for node_id in fixture.node_ids() {
                fixture
                    .wait_for_rows_on_node(
                        node_id.as_str(),
                        "SELECT id::text || ':' || payload FROM ha_partition_primary ORDER BY id",
                        expected_rows.as_slice(),
                        Duration::from_secs(90),
                    )
                    .await?;
            }
            fixture
                .wait_for_table_digest_convergence(
                    "ha_partition_primary",
                    fixture.node_ids().as_slice(),
                    2,
                    Duration::from_secs(90),
                )
                .await?;
            fixture
                .assert_no_dual_primary_window(Duration::from_secs(5))
                .await?;
            Ok(())
        })
        .await
        {
            Ok(run_result) => run_result,
            Err(_) => Err(WorkerError::Message(format!(
                "{scenario_name} timed out after {}s",
                E2E_SCENARIO_TIMEOUT.as_secs()
            ))),
        };

        finalize_partition_scenario(&mut fixture, scenario_name, run_result).await
    })
    .await
}

#[tokio::test(flavor = "current_thread")]
async fn e2e_partition_api_path_isolation_preserves_primary() -> Result<(), WorkerError> {
    run_with_local_set(async {
        let binaries = resolve_pg_binaries_for_real_tests()?;
        let etcd_bin = resolve_etcd_bin_for_real_tests()?;
        let mut fixture = PartitionFixture::start(3, binaries, etcd_bin).await?;
        let scenario_name = "ha-e2e-partition-api-path-isolation";

        let run_result = match tokio::time::timeout(E2E_SCENARIO_TIMEOUT, async {
            fixture.record("api-path isolation: wait for initial stable primary");
            let bootstrap_primary = fixture
                .wait_for_stable_primary(Duration::from_secs(90), None, 5)
                .await?;
            fixture.record(format!("api-path isolation: initial primary={bootstrap_primary}"));

            let isolated_node = fixture
                .node_ids()
                .into_iter()
                .find(|node_id| node_id != &bootstrap_primary)
                .ok_or_else(|| WorkerError::Message("no replica found for API isolation scenario".to_string()))?;

            fixture.isolate_api_path(isolated_node.as_str()).await?;
            if let Ok(state) = fixture.fetch_node_ha_state(isolated_node.as_str()).await {
                return Err(WorkerError::Message(format!(
                    "expected API isolation for node {isolated_node}, but /ha/state succeeded with phase={} leader={:?}",
                    state.ha_phase,
                    state.leader
                )));
            }

            fixture
                .assert_no_dual_primary_window(Duration::from_secs(8))
                .await?;
            tokio::time::sleep(Duration::from_secs(4)).await;

            fixture.heal_all_network_faults().await?;
            let healed_primary = fixture
                .wait_for_stable_primary(Duration::from_secs(90), None, 5)
                .await?;
            if healed_primary != bootstrap_primary {
                return Err(WorkerError::Message(format!(
                    "api-path isolation should not rotate primary; expected={bootstrap_primary} observed={healed_primary}"
                )));
            }

            fixture
                .run_sql_on_node_with_retry(
                    &healed_primary,
                    "CREATE TABLE IF NOT EXISTS ha_partition_api_only (id INTEGER PRIMARY KEY, payload TEXT NOT NULL)",
                    Duration::from_secs(30),
                )
                .await?;
            fixture
                .run_sql_on_node_with_retry(
                    &healed_primary,
                    "INSERT INTO ha_partition_api_only (id, payload) VALUES (1, 'api-only') ON CONFLICT (id) DO UPDATE SET payload = EXCLUDED.payload",
                    Duration::from_secs(45),
                )
                .await?;
            let expected_rows = vec!["1:api-only".to_string()];
            for node_id in fixture.node_ids() {
                fixture
                    .wait_for_rows_on_node(
                        node_id.as_str(),
                        "SELECT id::text || ':' || payload FROM ha_partition_api_only ORDER BY id",
                        expected_rows.as_slice(),
                        Duration::from_secs(90),
                    )
                    .await?;
            }
            fixture
                .assert_no_dual_primary_window(Duration::from_secs(5))
                .await?;
            Ok(())
        })
        .await
        {
            Ok(run_result) => run_result,
            Err(_) => Err(WorkerError::Message(format!(
                "{scenario_name} timed out after {}s",
                E2E_SCENARIO_TIMEOUT.as_secs()
            ))),
        };

        finalize_partition_scenario(&mut fixture, scenario_name, run_result).await
    })
    .await
}

#[tokio::test(flavor = "current_thread")]
async fn e2e_partition_mixed_faults_heal_converges() -> Result<(), WorkerError> {
    run_with_local_set(async {
        let binaries = resolve_pg_binaries_for_real_tests()?;
        let etcd_bin = resolve_etcd_bin_for_real_tests()?;
        let mut fixture = PartitionFixture::start(3, binaries, etcd_bin).await?;
        let scenario_name = "ha-e2e-partition-mixed-faults-heal";

        let run_result = match tokio::time::timeout(E2E_SCENARIO_TIMEOUT, async {
            fixture.record("mixed faults: wait for initial stable primary");
            let bootstrap_primary = fixture
                .wait_for_stable_primary(Duration::from_secs(90), None, 5)
                .await?;
            fixture.record(format!("mixed faults: initial primary={bootstrap_primary}"));

            let api_isolated_node = fixture
                .node_ids()
                .into_iter()
                .find(|node_id| node_id != &bootstrap_primary)
                .ok_or_else(|| WorkerError::Message("no non-primary node for mixed-fault API isolation".to_string()))?;

            fixture
                .run_sql_on_node_with_retry(
                    &bootstrap_primary,
                    "CREATE TABLE IF NOT EXISTS ha_partition_mixed (id INTEGER PRIMARY KEY, payload TEXT NOT NULL)",
                    Duration::from_secs(30),
                )
                .await?;
            fixture
                .run_sql_on_node_with_retry(
                    &bootstrap_primary,
                    "INSERT INTO ha_partition_mixed (id, payload) VALUES (1, 'before') ON CONFLICT (id) DO UPDATE SET payload = EXCLUDED.payload",
                    Duration::from_secs(30),
                )
                .await?;

            fixture
                .partition_primary_from_etcd(bootstrap_primary.as_str())
                .await?;
            fixture.isolate_api_path(api_isolated_node.as_str()).await?;

            fixture
                .wait_for_node_phase(
                    bootstrap_primary.as_str(),
                    "FailSafe",
                    Duration::from_secs(150),
                )
                .await?;
            fixture.record(format!(
                "mixed faults: isolated primary entered FailSafe node={bootstrap_primary}"
            ));
            fixture
                .assert_no_dual_primary_window(Duration::from_secs(10))
                .await?;

            fixture.heal_all_network_faults().await?;
            let healed_primary = fixture
                .wait_for_stable_primary(Duration::from_secs(120), None, 5)
                .await?;
            fixture.record(format!("mixed faults: healed primary={healed_primary}"));

            fixture
                .run_sql_on_node_with_retry(
                    &healed_primary,
                    "INSERT INTO ha_partition_mixed (id, payload) VALUES (2, 'after') ON CONFLICT (id) DO UPDATE SET payload = EXCLUDED.payload",
                    Duration::from_secs(45),
                )
                .await?;

            let expected_rows = vec!["1:before".to_string(), "2:after".to_string()];
            for node_id in fixture.node_ids() {
                fixture
                    .wait_for_rows_on_node(
                        node_id.as_str(),
                        "SELECT id::text || ':' || payload FROM ha_partition_mixed ORDER BY id",
                        expected_rows.as_slice(),
                        Duration::from_secs(120),
                    )
                    .await?;
            }
            fixture
                .wait_for_table_digest_convergence(
                    "ha_partition_mixed",
                    fixture.node_ids().as_slice(),
                    2,
                    Duration::from_secs(120),
                )
                .await?;
            fixture
                .assert_no_dual_primary_window(Duration::from_secs(6))
                .await?;
            Ok(())
        })
        .await
        {
            Ok(run_result) => run_result,
            Err(_) => Err(WorkerError::Message(format!(
                "{scenario_name} timed out after {}s",
                E2E_SCENARIO_TIMEOUT.as_secs()
            ))),
        };

        finalize_partition_scenario(&mut fixture, scenario_name, run_result).await
    })
    .await
}
