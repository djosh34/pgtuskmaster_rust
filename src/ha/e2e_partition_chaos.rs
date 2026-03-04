use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use tokio::task::JoinHandle;

use crate::{
    cli::client::{CliApiClient, HaStateResponse},
    state::WorkerError,
    test_harness::{ha_e2e, net_proxy::ProxyMode},
};

const E2E_COMMAND_TIMEOUT: Duration = Duration::from_secs(30);
const E2E_COMMAND_KILL_WAIT_TIMEOUT: Duration = Duration::from_secs(3);
const E2E_PG_STOP_TIMEOUT: Duration = Duration::from_secs(10);
const E2E_HTTP_STEP_TIMEOUT: Duration = Duration::from_secs(20);
const E2E_BOOTSTRAP_PRIMARY_TIMEOUT: Duration = Duration::from_secs(60);
const E2E_SCENARIO_TIMEOUT: Duration = Duration::from_secs(360);
const PARTITION_ARTIFACT_DIR: &str = ".ralph/evidence/28-e2e-network-partition-chaos";

struct PartitionFixture {
    _guard: crate::test_harness::namespace::NamespaceGuard,
    pg_ctl_bin: PathBuf,
    psql_bin: PathBuf,
    etcd: Option<crate::test_harness::etcd3::EtcdClusterHandle>,
    nodes: Vec<ha_e2e::NodeHandle>,
    tasks: Vec<JoinHandle<Result<(), WorkerError>>>,
    etcd_links_by_node: BTreeMap<String, Vec<String>>,
    etcd_proxies: BTreeMap<String, crate::test_harness::net_proxy::TcpProxyLink>,
    api_proxies: BTreeMap<String, crate::test_harness::net_proxy::TcpProxyLink>,
    pg_proxies: BTreeMap<String, crate::test_harness::net_proxy::TcpProxyLink>,
    timeline: Vec<String>,
}

impl PartitionFixture {
    async fn start(
        node_count: usize,
    ) -> Result<Self, WorkerError> {
        let config = ha_e2e::TestConfig {
            test_name: "ha-e2e-partition".to_string(),
            cluster_name: "cluster-e2e-partition".to_string(),
            scope: "scope-ha-e2e-partition".to_string(),
            node_count,
            etcd_members: vec!["etcd-a".to_string(), "etcd-b".to_string(), "etcd-c".to_string()],
            mode: ha_e2e::Mode::PartitionProxy,
            timeouts: ha_e2e::TimeoutConfig {
                command_timeout: E2E_COMMAND_TIMEOUT,
                command_kill_wait_timeout: E2E_COMMAND_KILL_WAIT_TIMEOUT,
                http_step_timeout: E2E_HTTP_STEP_TIMEOUT,
                api_readiness_timeout: Duration::from_secs(120),
                bootstrap_primary_timeout: E2E_BOOTSTRAP_PRIMARY_TIMEOUT,
                scenario_timeout: E2E_SCENARIO_TIMEOUT,
            },
            artifact_root: None,
        };

        let handle = ha_e2e::start_cluster(config).await?;

        Ok(Self {
            _guard: handle.guard,
            pg_ctl_bin: handle.binaries.pg_ctl.clone(),
            psql_bin: handle.binaries.psql.clone(),
            etcd: handle.etcd,
            nodes: handle.nodes,
            tasks: handle.tasks,
            etcd_links_by_node: handle.etcd_links_by_node,
            etcd_proxies: handle.etcd_proxies,
            api_proxies: handle.api_proxies,
            pg_proxies: handle.pg_proxies,
            timeline: handle.timeline,
        })
    }

    fn record(&mut self, message: impl Into<String>) {
        let now = match ha_e2e::util::unix_now() {
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

    fn node_by_id(&self, node_id: &str) -> Option<&ha_e2e::NodeHandle> {
        self.nodes.iter().find(|node| node.id == node_id)
    }

    async fn set_etcd_mode_for_node(
        &self,
        node_id: &str,
        mode: ProxyMode,
    ) -> Result<(), WorkerError> {
        let links = self.etcd_links_by_node.get(node_id).ok_or_else(|| {
            WorkerError::Message(format!("unknown node for etcd partition: {node_id}"))
        })?;
        for link_name in links {
            let link = self.etcd_proxies.get(link_name.as_str()).ok_or_else(|| {
                WorkerError::Message(format!(
                    "missing etcd proxy link for node {node_id}: {link_name}"
                ))
            })?;
            link.set_mode(mode.clone()).await.map_err(|err| {
                WorkerError::Message(format!("set mode on {link_name} failed: {err}"))
            })?;
        }
        Ok(())
    }

    async fn partition_node_from_etcd(&mut self, node_id: &str) -> Result<(), WorkerError> {
        self.record(format!(
            "network fault: partition node from etcd majority node={node_id}"
        ));
        self.set_etcd_mode_for_node(node_id, ProxyMode::Blocked)
            .await
    }

    async fn partition_primary_from_etcd(&mut self, node_id: &str) -> Result<(), WorkerError> {
        self.record(format!(
            "network fault: partition current primary from etcd node={node_id}"
        ));
        self.set_etcd_mode_for_node(node_id, ProxyMode::Blocked)
            .await
    }

    async fn isolate_api_path(&mut self, node_id: &str) -> Result<(), WorkerError> {
        self.record(format!(
            "network fault: isolate API path for node={node_id}"
        ));
        let link = self.api_proxies.get(node_id).ok_or_else(|| {
            WorkerError::Message(format!("missing api proxy for node: {node_id}"))
        })?;
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
            link.set_mode(ProxyMode::PassThrough).await.map_err(|err| {
                WorkerError::Message(format!("heal postgres proxy failed: {err}"))
            })?;
        }
        Ok(())
    }

    async fn fetch_node_ha_state(&self, node_id: &str) -> Result<HaStateResponse, WorkerError> {
        let node = self.node_by_id(node_id).ok_or_else(|| {
            WorkerError::Message(format!("unknown node id for HA state: {node_id}"))
        })?;
        let timeout_ms = u64::try_from(E2E_HTTP_STEP_TIMEOUT.as_millis()).map_err(|_| {
            WorkerError::Message("e2e HTTP timeout does not fit into u64".to_string())
        })?;
        let client = CliApiClient::new(
            format!("http://{}", node.api_observe_addr),
            timeout_ms,
            None,
            None,
        )
        .map_err(|err| WorkerError::Message(format!("build CliApiClient failed: {err}")))?;
        client.get_ha_state().await.map_err(|err| {
            WorkerError::Message(format!("GET /ha/state failed for node {node_id}: {err}"))
        })
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
                    format!(
                        "{}:{}:leader={leader}",
                        state.self_member_id, state.ha_phase
                    )
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
                    format!("phase={} leader={:?}", state.ha_phase, state.leader)
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
        ha_e2e::util::run_psql_statement(
            self.psql_bin.as_path(),
            node.sql_port,
            sql,
            E2E_COMMAND_TIMEOUT,
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
                let digest = ha_e2e::util::parse_psql_rows(digest_raw.as_str())
                    .first()
                    .cloned()
                    .unwrap_or_default();
                let row_count = ha_e2e::util::parse_single_u64(count_raw.as_str())?;
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
        let stamp = ha_e2e::util::unix_now()?.0;
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

        let mut pg_stops = Vec::with_capacity(self.nodes.len());
        for node in &self.nodes {
            let pg_ctl_bin = self.pg_ctl_bin.clone();
            let data_dir = node.data_dir.clone();
            let node_id = node.id.clone();
            pg_stops.push(tokio::task::spawn_local(async move {
                match ha_e2e::util::pg_ctl_stop_immediate(
                    &pg_ctl_bin,
                    &data_dir,
                    E2E_PG_STOP_TIMEOUT,
                    E2E_COMMAND_KILL_WAIT_TIMEOUT,
                )
                .await
                {
                    Ok(()) => None,
                    Err(err) => Some(format!("postgres stop {node_id} failed: {err}")),
                }
            }));
        }
        for stop in pg_stops {
            match stop.await {
                Ok(Some(message)) => failures.push(message),
                Ok(None) => {}
                Err(err) => failures.push(format!("postgres stop join failed: {err}")),
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

#[tokio::test(flavor = "current_thread")]
async fn e2e_partition_minority_isolation_no_split_brain_rejoin() -> Result<(), WorkerError> {
    ha_e2e::util::run_with_local_set(async {
        let mut fixture = PartitionFixture::start(3).await?;
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
    ha_e2e::util::run_with_local_set(async {
        let mut fixture = PartitionFixture::start(3).await?;
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
    ha_e2e::util::run_with_local_set(async {
        let mut fixture = PartitionFixture::start(3).await?;
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
    ha_e2e::util::run_with_local_set(async {
        let mut fixture = PartitionFixture::start(3).await?;
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
