use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use super::observer::{HaInvariantObserver, HaObserverConfig};

use pgtuskmaster_rust::{
    api::HaStateResponse,
    cli::client::CliApiClient,
    state::WorkerError,
    test_harness::{ha_e2e, net_proxy::ProxyMode},
};

use pgtuskmaster_rust::test_harness::ha_e2e::handle::TestClusterHandle;

const E2E_COMMAND_TIMEOUT: Duration = Duration::from_secs(30);
const E2E_COMMAND_KILL_WAIT_TIMEOUT: Duration = Duration::from_secs(3);
const E2E_PG_STOP_TIMEOUT: Duration = Duration::from_secs(10);
const E2E_HTTP_STEP_TIMEOUT: Duration = Duration::from_secs(20);
const E2E_BOOTSTRAP_PRIMARY_TIMEOUT: Duration = Duration::from_secs(60);
const E2E_SCENARIO_TIMEOUT: Duration = Duration::from_secs(360);
const E2E_API_READINESS_TIMEOUT: Duration = Duration::from_secs(120);
const E2E_STABLE_PRIMARY_API_POLL_INTERVAL: Duration = Duration::from_millis(100);
const E2E_SQL_RETRY_INTERVAL: Duration = Duration::from_millis(200);
const E2E_NO_DUAL_PRIMARY_SAMPLE_INTERVAL: Duration = Duration::from_millis(75);
const E2E_STABLE_PRIMARY_STRICT_TIMEOUT_CAP: Duration = Duration::from_secs(25);
const E2E_STABLE_PRIMARY_API_FALLBACK_TIMEOUT_CAP: Duration = Duration::from_secs(20);
const E2E_STABLE_PRIMARY_SQL_FALLBACK_TIMEOUT_CAP: Duration = Duration::from_secs(30);
const E2E_STABLE_PRIMARY_STRICT_CONSECUTIVE_CAP: usize = 3;
const E2E_STABLE_PRIMARY_RELAXED_CONSECUTIVE_CAP: usize = 2;
const E2E_PARTITION_PRIMARY_TIMEOUT: Duration = Duration::from_secs(90);
const E2E_PARTITION_RECOVERY_TIMEOUT: Duration = Duration::from_secs(120);
const E2E_PARTITION_WRITE_TIMEOUT: Duration = Duration::from_secs(30);
const E2E_PARTITION_POST_HEAL_WRITE_TIMEOUT: Duration = Duration::from_secs(45);
const E2E_PARTITION_REPLICATION_CONVERGENCE_TIMEOUT: Duration = Duration::from_secs(90);
const E2E_PARTITION_REPLICATION_PATH_TIMEOUT: Duration = Duration::from_secs(30);
const E2E_PARTITION_HEAL_SETTLE_WAIT: Duration = Duration::from_secs(4);
const E2E_PARTITION_SHORT_NO_DUAL_PRIMARY_WINDOW: Duration = Duration::from_secs(5);
const E2E_PARTITION_MEDIUM_NO_DUAL_PRIMARY_WINDOW: Duration = Duration::from_secs(8);
const E2E_PARTITION_LONG_NO_DUAL_PRIMARY_WINDOW: Duration = Duration::from_secs(10);
const PARTITION_ARTIFACT_DIR: &str = ".ralph/evidence/28-e2e-network-partition-chaos";

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

struct PartitionFixture {
    _guard: pgtuskmaster_rust::test_harness::namespace::NamespaceGuard,
    pg_ctl_bin: PathBuf,
    psql_bin: PathBuf,
    superuser_username: String,
    superuser_dbname: String,
    etcd: Option<pgtuskmaster_rust::test_harness::etcd3::EtcdClusterHandle>,
    nodes: Vec<ha_e2e::NodeHandle>,
    runtime_nodes: ha_e2e::RuntimeNodeSet,
    etcd_proxies: BTreeMap<String, pgtuskmaster_rust::test_harness::net_proxy::TcpProxyLink>,
    api_proxies: BTreeMap<String, pgtuskmaster_rust::test_harness::net_proxy::TcpProxyLink>,
    pg_proxies: BTreeMap<String, pgtuskmaster_rust::test_harness::net_proxy::TcpProxyLink>,
    timeline: Vec<String>,
}

impl PartitionFixture {
    async fn start(node_count: usize) -> Result<Self, WorkerError> {
        let config = ha_e2e::TestConfig {
            test_name: "ha-e2e-partition".to_string(),
            cluster_name: "cluster-e2e-partition".to_string(),
            scope: "scope-ha-e2e-partition".to_string(),
            node_count,
            etcd_members: vec![
                "etcd-a".to_string(),
                "etcd-b".to_string(),
                "etcd-c".to_string(),
            ],
            postgres_roles: None,
            mode: ha_e2e::Mode::PartitionProxy,
            timeouts: ha_e2e::TimeoutConfig {
                command_timeout: E2E_COMMAND_TIMEOUT,
                command_kill_wait_timeout: E2E_COMMAND_KILL_WAIT_TIMEOUT,
                http_step_timeout: E2E_HTTP_STEP_TIMEOUT,
                api_readiness_timeout: E2E_API_READINESS_TIMEOUT,
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
            runtime_nodes,
            etcd_proxies,
            api_proxies,
            pg_proxies,
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
            etcd_proxies,
            api_proxies,
            pg_proxies,
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

    fn node_ids(&self) -> Vec<String> {
        self.nodes.iter().map(|node| node.id.clone()).collect()
    }

    fn node_by_id(&self, node_id: &str) -> Option<&ha_e2e::NodeHandle> {
        self.nodes.iter().find(|node| node.id == node_id)
    }

    async fn set_etcd_mode_for_node(
        &self,
        node_id: &str,
        mode: ProxyMode,
    ) -> Result<(), WorkerError> {
        let prefix = format!("{node_id}-to-");
        let mut matched = 0usize;
        for (link_name, link) in &self.etcd_proxies {
            if link_name.starts_with(prefix.as_str()) && link_name.ends_with("-etcd") {
                matched = matched.saturating_add(1);
                link.set_mode(mode.clone()).await.map_err(|err| {
                    WorkerError::Message(format!(
                        "set mode on {link_name} failed for node {node_id}: {err}"
                    ))
                })?;
            }
        }

        if matched == 0 {
            return Err(WorkerError::Message(format!(
                "no etcd proxy links found for node for etcd partition: {node_id}"
            )));
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

    async fn set_pg_mode_for_node(
        &self,
        node_id: &str,
        mode: ProxyMode,
    ) -> Result<(), WorkerError> {
        let link = self.pg_proxies.get(node_id).ok_or_else(|| {
            WorkerError::Message(format!("missing postgres proxy for node: {node_id}"))
        })?;
        link.set_mode(mode)
            .await
            .map_err(|err| WorkerError::Message(format!("set postgres proxy mode failed: {err}")))
    }

    async fn isolate_postgres_path(&mut self, node_id: &str) -> Result<(), WorkerError> {
        self.record(format!(
            "network fault: block advertised postgres data path via pg proxy node={node_id}"
        ));
        self.set_pg_mode_for_node(node_id, ProxyMode::Blocked).await
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

    async fn cluster_ha_states_strict(&mut self) -> Result<Vec<HaStateResponse>, WorkerError> {
        self.ensure_runtime_tasks_healthy().await?;
        let mut states = Vec::new();
        for node_id in self.node_ids() {
            let state = self.fetch_node_ha_state(node_id.as_str()).await?;
            states.push(state);
        }
        Ok(states)
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
        let mut last_observation: Option<String> = None;

        loop {
            if tokio::time::Instant::now() >= deadline {
                let detail = last_observation
                    .as_deref()
                    .map_or_else(|| "none".to_string(), ToString::to_string);
                return Err(WorkerError::Message(format!(
                    "timed out waiting for stable primary; excluded={excluded_primary:?}; last_observation={detail}"
                )));
            }

            let states = match self.cluster_ha_states_strict().await {
                Ok(states) => states,
                Err(err) => {
                    stable_count = 0;
                    last_candidate = None;
                    last_observation = Some(format!("poll:error={err}"));
                    if tokio::time::Instant::now() >= deadline {
                        let detail = last_observation
                            .as_deref()
                            .map_or_else(|| "none".to_string(), ToString::to_string);
                        return Err(WorkerError::Message(format!(
                            "timed out waiting for stable primary; excluded={excluded_primary:?}; last_observation={detail}"
                        )));
                    }
                    tokio::time::sleep(E2E_STABLE_PRIMARY_API_POLL_INTERVAL).await;
                    continue;
                }
            };

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
            last_observation = Some(format!("states=[{state_summary}]"));

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
                let detail = last_observation
                    .as_deref()
                    .map_or_else(|| "none".to_string(), ToString::to_string);
                return Err(WorkerError::Message(format!(
                    "timed out waiting for stable primary; excluded={excluded_primary:?}; last_observation={detail}"
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
                    "timed out waiting for stable primary via best-effort polling; excluded={excluded_primary:?}; last_observation={last_observation}"
                )));
            }

            let (states, errors) = self.cluster_ha_states_best_effort().await?;
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
            let error_summary = if errors.is_empty() {
                "none".to_string()
            } else {
                errors.join("; ")
            };
            last_observation = format!(
                "observed_nodes={} states=[{state_summary}] errors=[{error_summary}]",
                states.len()
            );

            if states.len() < min_observed_nodes {
                stable_count = 0;
                last_candidate = None;
            } else {
                let primaries = Self::primary_members(states.as_slice());
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

            let mut observed_nodes = 0usize;
            let mut primary_nodes = Vec::new();
            let mut fragments = Vec::new();
            for node_id in self.node_ids() {
                match self
                    .run_sql_on_node(
                        node_id.as_str(),
                        "SELECT CASE WHEN pg_is_in_recovery() THEN 'replica' ELSE 'primary' END",
                    )
                    .await
                {
                    Ok(output) => {
                        let rows = ha_e2e::util::parse_psql_rows(output.as_str());
                        observed_nodes = observed_nodes.saturating_add(1);
                        let role = rows
                            .first()
                            .map(|value| value.as_str())
                            .unwrap_or("unknown");
                        fragments.push(format!("{node_id}:{role}"));
                        if role == "primary" {
                            primary_nodes.push(node_id);
                        }
                    }
                    Err(err) => {
                        fragments.push(format!("{node_id}:error={err}"));
                    }
                }
            }

            last_observation = format!(
                "observed_nodes={observed_nodes} roles=[{}]",
                fragments.join(", ")
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

            tokio::time::sleep(E2E_SQL_RETRY_INTERVAL).await;
        }
    }

    async fn wait_for_stable_primary_resilient(
        &mut self,
        plan: StablePrimaryWaitPlan<'_>,
    ) -> Result<String, WorkerError> {
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
            )
            .await
        {
            Ok(primary) => Ok(primary),
            Err(wait_err) => {
                self.record(format!(
                    "{}: strict stable-primary wait failed: {wait_err}; retrying with best-effort polling",
                    plan.context
                ));
                match self
                    .wait_for_stable_primary_best_effort(
                        api_fallback_timeout,
                        plan.excluded_primary,
                        relaxed_required_consecutive,
                        plan.min_observed_nodes,
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

    fn primary_members(states: &[HaStateResponse]) -> Vec<String> {
        states
            .iter()
            .filter(|state| state.ha_phase == "Primary")
            .map(|state| state.self_member_id.clone())
            .collect()
    }

    fn replica_node_ids(&self, primary_id: &str) -> Vec<String> {
        self.node_ids()
            .into_iter()
            .filter(|node_id| node_id != primary_id)
            .collect()
    }

    async fn assert_no_dual_primary_window(&mut self, window: Duration) -> Result<(), WorkerError> {
        let deadline = tokio::time::Instant::now() + window;
        let mut observer = HaInvariantObserver::new(HaObserverConfig {
            min_successful_samples: 1,
            ring_capacity: 16,
        });
        loop {
            observer.record_poll_attempt();
            let (states, errors) = self.cluster_ha_states_best_effort().await?;
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

            if tokio::time::Instant::now() >= deadline {
                return observer.finalize_no_dual_primary_window();
            }
            tokio::time::sleep(E2E_NO_DUAL_PRIMARY_SAMPLE_INTERVAL).await;
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
            tokio::time::sleep(E2E_STABLE_PRIMARY_API_POLL_INTERVAL).await;
        }
    }

    async fn run_sql_on_port(&self, port: u16, sql: &str) -> Result<String, WorkerError> {
        ha_e2e::util::run_psql_statement(
            self.psql_bin.as_path(),
            port,
            self.superuser_username.as_str(),
            self.superuser_dbname.as_str(),
            sql,
            E2E_COMMAND_TIMEOUT,
            E2E_COMMAND_KILL_WAIT_TIMEOUT,
        )
        .await
    }

    async fn run_sql_on_node(&self, node_id: &str, sql: &str) -> Result<String, WorkerError> {
        let node = self
            .node_by_id(node_id)
            .ok_or_else(|| WorkerError::Message(format!("unknown node for SQL: {node_id}")))?;
        self.run_sql_on_port(node.sql_port, sql).await
    }

    async fn run_sql_on_node_direct_postgres(
        &self,
        node_id: &str,
        sql: &str,
    ) -> Result<String, WorkerError> {
        let node = self.node_by_id(node_id).ok_or_else(|| {
            WorkerError::Message(format!("unknown node for direct postgres SQL: {node_id}"))
        })?;
        self.run_sql_on_port(node.pg_port, sql).await
    }

    async fn cluster_sql_roles_best_effort(
        &self,
    ) -> Result<(Vec<(String, String)>, Vec<String>), WorkerError> {
        let mut roles = Vec::new();
        let mut errors = Vec::new();
        for node_id in self.node_ids() {
            match self
                .run_sql_on_node(
                    node_id.as_str(),
                    "SELECT CASE WHEN pg_is_in_recovery() THEN 'replica' ELSE 'primary' END",
                )
                .await
            {
                Ok(output) => {
                    let rows = ha_e2e::util::parse_psql_rows(output.as_str());
                    let role = rows
                        .first()
                        .cloned()
                        .unwrap_or_else(|| "unknown".to_string());
                    roles.push((node_id, role));
                }
                Err(err) => {
                    errors.push(format!("node={node_id} error={err}"));
                }
            }
        }
        Ok((roles, errors))
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
                    tokio::time::sleep(E2E_SQL_RETRY_INTERVAL).await;
                }
            }
        }
    }

    async fn wait_for_replication_sender_port_on_replicas(
        &mut self,
        primary_id: &str,
        expected_sender_port: u16,
        timeout: Duration,
    ) -> Result<(), WorkerError> {
        let replica_ids = self.replica_node_ids(primary_id);
        if replica_ids.is_empty() {
            return Err(WorkerError::Message(format!(
                "no replicas available to validate replication sender port for primary {primary_id}"
            )));
        }

        let deadline = tokio::time::Instant::now() + timeout;
        let observation_sql = "SELECT COALESCE(string_agg(status || '@' || sender_host || ':' || sender_port::text, ',' ORDER BY sender_host, sender_port, status), 'no-wal-receiver') FROM pg_stat_wal_receiver";
        let expected_port_fragment = format!(":{expected_sender_port}");

        loop {
            let mut observations = Vec::new();
            let mut all_match = true;
            for replica_id in &replica_ids {
                let output = match self
                    .run_sql_on_node(replica_id.as_str(), observation_sql)
                    .await
                {
                    Ok(value) => value,
                    Err(err) => {
                        all_match = false;
                        observations.push(format!("{replica_id}=error:{err}"));
                        continue;
                    }
                };
                let summary = ha_e2e::util::parse_psql_rows(output.as_str())
                    .first()
                    .cloned()
                    .unwrap_or_default();
                if !(summary.contains("streaming@")
                    && summary.contains(expected_port_fragment.as_str()))
                {
                    all_match = false;
                }
                observations.push(format!("{replica_id}={summary}"));
            }
            let last_observation = observations.join(", ");
            if all_match {
                self.record(format!(
                    "replication baseline: replicas stream from primary node={primary_id} advertised_postgres_port={expected_sender_port}; observations=[{last_observation}]"
                ));
                return Ok(());
            }
            if tokio::time::Instant::now() >= deadline {
                return Err(WorkerError::Message(format!(
                    "timed out waiting for replicas to stream from advertised postgres port {expected_sender_port} for primary {primary_id}; last_observation=[{last_observation}]"
                )));
            }
            tokio::time::sleep(E2E_SQL_RETRY_INTERVAL).await;
        }
    }

    async fn wait_for_replication_path_interruption(
        &mut self,
        primary_id: &str,
        timeout: Duration,
    ) -> Result<(), WorkerError> {
        let replica_ids = self.replica_node_ids(primary_id);
        if replica_ids.is_empty() {
            return Err(WorkerError::Message(format!(
                "no replicas available to validate replication-path interruption for primary {primary_id}"
            )));
        }

        let deadline = tokio::time::Instant::now() + timeout;
        let observation_sql = "SELECT COALESCE(string_agg(status || '@' || sender_host || ':' || sender_port::text, ',' ORDER BY sender_host, sender_port, status), 'no-wal-receiver') FROM pg_stat_wal_receiver";

        loop {
            let mut observations = Vec::new();
            let mut all_interrupted = true;
            for replica_id in &replica_ids {
                let output = match self
                    .run_sql_on_node(replica_id.as_str(), observation_sql)
                    .await
                {
                    Ok(value) => value,
                    Err(err) => {
                        all_interrupted = false;
                        observations.push(format!("{replica_id}=error:{err}"));
                        continue;
                    }
                };
                let summary = ha_e2e::util::parse_psql_rows(output.as_str())
                    .first()
                    .cloned()
                    .unwrap_or_default();
                if summary.contains("streaming@") {
                    all_interrupted = false;
                }
                observations.push(format!("{replica_id}={summary}"));
            }
            let last_observation = observations.join(", ");
            if all_interrupted {
                self.record(format!(
                    "replication interruption observed after blocking advertised postgres path for primary node={primary_id}; observations=[{last_observation}]"
                ));
                return Ok(());
            }
            if tokio::time::Instant::now() >= deadline {
                return Err(WorkerError::Message(format!(
                    "timed out waiting for replica wal receivers to leave streaming state for primary {primary_id}; last_observation=[{last_observation}]"
                )));
            }
            tokio::time::sleep(E2E_SQL_RETRY_INTERVAL).await;
        }
    }

    async fn assert_query_rows_stay_equal_on_nodes(
        &self,
        node_ids: &[String],
        sql: &str,
        expected_rows: &[String],
        window: Duration,
        context: &str,
    ) -> Result<(), WorkerError> {
        let deadline = tokio::time::Instant::now() + window;
        loop {
            for node_id in node_ids {
                let output = self.run_sql_on_node(node_id.as_str(), sql).await?;
                let rows = ha_e2e::util::parse_psql_rows(output.as_str());
                if rows != expected_rows {
                    return Err(WorkerError::Message(format!(
                        "{context}: unexpected rows on {node_id}; expected={expected_rows:?} observed={rows:?}"
                    )));
                }
            }
            if tokio::time::Instant::now() >= deadline {
                return Ok(());
            }
            tokio::time::sleep(E2E_SQL_RETRY_INTERVAL).await;
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
            tokio::time::sleep(E2E_SQL_RETRY_INTERVAL).await;
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
            tokio::time::sleep(E2E_SQL_RETRY_INTERVAL).await;
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
        self.runtime_nodes.ensure_healthy().await
    }

    async fn shutdown(&mut self) -> Result<(), WorkerError> {
        let mut failures = Vec::new();

        self.runtime_nodes.shutdown_all().await;

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

pub async fn e2e_partition_minority_isolation_no_split_brain_rejoin() -> Result<(), WorkerError> {
    ha_e2e::util::run_with_local_set(async {
        let mut fixture = PartitionFixture::start(3).await?;
        let scenario_name = "ha-e2e-partition-minority-isolation";

        let run_result = match tokio::time::timeout(E2E_SCENARIO_TIMEOUT, async {
            fixture.record("minority isolation: wait for initial stable primary");
            let bootstrap_primary = fixture
                .wait_for_stable_primary_resilient(StablePrimaryWaitPlan {
                    context: "minority isolation: initial stable primary",
                    timeout: E2E_PARTITION_PRIMARY_TIMEOUT,
                    excluded_primary: None,
                    required_consecutive: 5,
                    fallback_timeout: E2E_PARTITION_PRIMARY_TIMEOUT,
                    fallback_required_consecutive: 3,
                    min_observed_nodes: 2,
                })
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
                    E2E_PARTITION_WRITE_TIMEOUT,
                )
                .await?;
            fixture
                .run_sql_on_node_with_retry(
                    &bootstrap_primary,
                    "INSERT INTO ha_partition_minority (id, payload) VALUES (1, 'before') ON CONFLICT (id) DO UPDATE SET payload = EXCLUDED.payload",
                    E2E_PARTITION_WRITE_TIMEOUT,
                )
                .await?;

            fixture
                .partition_node_from_etcd(isolated_replica.as_str())
                .await?;
            fixture
                .assert_no_dual_primary_window(E2E_PARTITION_MEDIUM_NO_DUAL_PRIMARY_WINDOW)
                .await?;

            tokio::time::sleep(E2E_PARTITION_HEAL_SETTLE_WAIT).await;
            fixture.heal_all_network_faults().await?;

            let healed_primary = fixture
                .wait_for_stable_primary_resilient(StablePrimaryWaitPlan {
                    context: "minority isolation: healed stable primary",
                    timeout: E2E_PARTITION_PRIMARY_TIMEOUT,
                    excluded_primary: None,
                    required_consecutive: 5,
                    fallback_timeout: E2E_PARTITION_PRIMARY_TIMEOUT,
                    fallback_required_consecutive: 3,
                    min_observed_nodes: 2,
                })
                .await?;
            fixture.record(format!("minority isolation: healed primary={healed_primary}"));
            fixture
                .run_sql_on_node_with_retry(
                    &healed_primary,
                    "INSERT INTO ha_partition_minority (id, payload) VALUES (2, 'after') ON CONFLICT (id) DO UPDATE SET payload = EXCLUDED.payload",
                    E2E_PARTITION_POST_HEAL_WRITE_TIMEOUT,
                )
                .await?;

            let expected_rows = vec!["1:before".to_string(), "2:after".to_string()];
            for node_id in fixture.node_ids() {
                fixture
                    .wait_for_rows_on_node(
                        node_id.as_str(),
                        "SELECT id::text || ':' || payload FROM ha_partition_minority ORDER BY id",
                        expected_rows.as_slice(),
                        E2E_PARTITION_REPLICATION_CONVERGENCE_TIMEOUT,
                    )
                    .await?;
            }
            fixture
                .wait_for_table_digest_convergence(
                    "ha_partition_minority",
                    fixture.node_ids().as_slice(),
                    2,
                    E2E_PARTITION_REPLICATION_CONVERGENCE_TIMEOUT,
                )
                .await?;
            fixture
                .assert_no_dual_primary_window(E2E_PARTITION_SHORT_NO_DUAL_PRIMARY_WINDOW)
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

pub async fn e2e_partition_primary_isolation_failover_no_split_brain() -> Result<(), WorkerError> {
    ha_e2e::util::run_with_local_set(async {
        let mut fixture = PartitionFixture::start(3).await?;
        let scenario_name = "ha-e2e-partition-primary-isolation";

        let run_result = match tokio::time::timeout(E2E_SCENARIO_TIMEOUT, async {
            fixture.record("primary isolation: wait for initial stable primary");
            let bootstrap_primary = fixture
                .wait_for_stable_primary_resilient(StablePrimaryWaitPlan {
                    context: "primary isolation: initial stable primary",
                    timeout: E2E_PARTITION_PRIMARY_TIMEOUT,
                    excluded_primary: None,
                    required_consecutive: 5,
                    fallback_timeout: E2E_PARTITION_PRIMARY_TIMEOUT,
                    fallback_required_consecutive: 3,
                    min_observed_nodes: 2,
                })
                .await?;
            fixture.record(format!("primary isolation: initial primary={bootstrap_primary}"));

            fixture
                .run_sql_on_node_with_retry(
                    &bootstrap_primary,
                    "CREATE TABLE IF NOT EXISTS ha_partition_primary (id INTEGER PRIMARY KEY, payload TEXT NOT NULL)",
                    E2E_PARTITION_WRITE_TIMEOUT,
                )
                .await?;
            fixture
                .run_sql_on_node_with_retry(
                    &bootstrap_primary,
                    "INSERT INTO ha_partition_primary (id, payload) VALUES (1, 'before') ON CONFLICT (id) DO UPDATE SET payload = EXCLUDED.payload",
                    E2E_PARTITION_WRITE_TIMEOUT,
                )
                .await?;

            fixture
                .partition_primary_from_etcd(bootstrap_primary.as_str())
                .await?;
            fixture
                .wait_for_node_phase(
                    bootstrap_primary.as_str(),
                    "FailSafe",
                    E2E_PARTITION_RECOVERY_TIMEOUT,
                )
                .await?;
            fixture.record(format!(
                "primary isolation: isolated primary entered FailSafe node={bootstrap_primary}"
            ));
            fixture
                .assert_no_dual_primary_window(E2E_PARTITION_LONG_NO_DUAL_PRIMARY_WINDOW)
                .await?;

            fixture.heal_all_network_faults().await?;
            let healed_primary = fixture
                .wait_for_stable_primary_resilient(StablePrimaryWaitPlan {
                    context: "primary isolation: healed stable primary",
                    timeout: E2E_PARTITION_RECOVERY_TIMEOUT,
                    excluded_primary: None,
                    required_consecutive: 5,
                    fallback_timeout: E2E_PARTITION_RECOVERY_TIMEOUT,
                    fallback_required_consecutive: 3,
                    min_observed_nodes: 2,
                })
                .await?;
            fixture.record(format!("primary isolation: healed primary={healed_primary}"));
            fixture
                .run_sql_on_node_with_retry(
                    &healed_primary,
                    "INSERT INTO ha_partition_primary (id, payload) VALUES (2, 'after') ON CONFLICT (id) DO UPDATE SET payload = EXCLUDED.payload",
                    E2E_PARTITION_POST_HEAL_WRITE_TIMEOUT,
                )
                .await?;

            let expected_rows = vec!["1:before".to_string(), "2:after".to_string()];
            for node_id in fixture.node_ids() {
                fixture
                    .wait_for_rows_on_node(
                        node_id.as_str(),
                        "SELECT id::text || ':' || payload FROM ha_partition_primary ORDER BY id",
                        expected_rows.as_slice(),
                        E2E_PARTITION_REPLICATION_CONVERGENCE_TIMEOUT,
                    )
                    .await?;
            }
            fixture
                .wait_for_table_digest_convergence(
                    "ha_partition_primary",
                    fixture.node_ids().as_slice(),
                    2,
                    E2E_PARTITION_REPLICATION_CONVERGENCE_TIMEOUT,
                )
                .await?;
            fixture
                .assert_no_dual_primary_window(E2E_PARTITION_SHORT_NO_DUAL_PRIMARY_WINDOW)
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

pub async fn e2e_partition_api_path_isolation_preserves_primary() -> Result<(), WorkerError> {
    ha_e2e::util::run_with_local_set(async {
        let mut fixture = PartitionFixture::start(3).await?;
        let scenario_name = "ha-e2e-partition-api-path-isolation";

        let run_result = match tokio::time::timeout(E2E_SCENARIO_TIMEOUT, async {
            fixture.record("api-path isolation: wait for initial stable primary");
            let bootstrap_primary = fixture
                .wait_for_stable_primary_resilient(StablePrimaryWaitPlan {
                    context: "api-path isolation: initial stable primary",
                    timeout: E2E_PARTITION_PRIMARY_TIMEOUT,
                    excluded_primary: None,
                    required_consecutive: 5,
                    fallback_timeout: E2E_PARTITION_PRIMARY_TIMEOUT,
                    fallback_required_consecutive: 3,
                    min_observed_nodes: 2,
                })
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
                .assert_no_dual_primary_window(E2E_PARTITION_MEDIUM_NO_DUAL_PRIMARY_WINDOW)
                .await?;
            tokio::time::sleep(E2E_PARTITION_HEAL_SETTLE_WAIT).await;

            fixture.heal_all_network_faults().await?;
            let healed_primary = fixture
                .wait_for_stable_primary_resilient(StablePrimaryWaitPlan {
                    context: "api-path isolation: healed stable primary",
                    timeout: E2E_PARTITION_PRIMARY_TIMEOUT,
                    excluded_primary: None,
                    required_consecutive: 5,
                    fallback_timeout: E2E_PARTITION_PRIMARY_TIMEOUT,
                    fallback_required_consecutive: 3,
                    min_observed_nodes: 2,
                })
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
                    E2E_PARTITION_WRITE_TIMEOUT,
                )
                .await?;
            fixture
                .run_sql_on_node_with_retry(
                    &healed_primary,
                    "INSERT INTO ha_partition_api_only (id, payload) VALUES (1, 'api-only') ON CONFLICT (id) DO UPDATE SET payload = EXCLUDED.payload",
                    E2E_PARTITION_POST_HEAL_WRITE_TIMEOUT,
                )
                .await?;
            let expected_rows = vec!["1:api-only".to_string()];
            for node_id in fixture.node_ids() {
                fixture
                    .wait_for_rows_on_node(
                        node_id.as_str(),
                        "SELECT id::text || ':' || payload FROM ha_partition_api_only ORDER BY id",
                        expected_rows.as_slice(),
                        E2E_PARTITION_REPLICATION_CONVERGENCE_TIMEOUT,
                    )
                    .await?;
            }
            fixture
                .assert_no_dual_primary_window(E2E_PARTITION_SHORT_NO_DUAL_PRIMARY_WINDOW)
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

pub async fn e2e_partition_primary_postgres_path_blocked_replicas_catch_up_after_heal(
) -> Result<(), WorkerError> {
    ha_e2e::util::run_with_local_set(async {
        let mut fixture = PartitionFixture::start(3).await?;
        let scenario_name = "ha-e2e-partition-primary-postgres-path-blocked";

        let run_result = match tokio::time::timeout(E2E_SCENARIO_TIMEOUT, async {
            fixture.record("postgres-path isolation: wait for initial stable primary");
            let bootstrap_primary = fixture
                .wait_for_stable_primary_resilient(StablePrimaryWaitPlan {
                    context: "postgres-path isolation: initial stable primary",
                    timeout: E2E_PARTITION_PRIMARY_TIMEOUT,
                    excluded_primary: None,
                    required_consecutive: 5,
                    fallback_timeout: E2E_PARTITION_PRIMARY_TIMEOUT,
                    fallback_required_consecutive: 3,
                    min_observed_nodes: 2,
                })
                .await?;
            fixture.record(format!(
                "postgres-path isolation: initial primary={bootstrap_primary}"
            ));

            let primary_node = fixture
                .node_by_id(bootstrap_primary.as_str())
                .cloned()
                .ok_or_else(|| {
                    WorkerError::Message(format!(
                        "missing node handle for primary {bootstrap_primary}"
                    ))
                })?;
            let replica_ids = fixture.replica_node_ids(bootstrap_primary.as_str());
            if replica_ids.is_empty() {
                return Err(WorkerError::Message(
                    "no replicas found for postgres-path isolation scenario".to_string(),
                ));
            }

            fixture
                .run_sql_on_node_with_retry(
                    bootstrap_primary.as_str(),
                    "CREATE TABLE IF NOT EXISTS ha_partition_pg_path_primary (id INTEGER PRIMARY KEY, payload TEXT NOT NULL)",
                    E2E_PARTITION_WRITE_TIMEOUT,
                )
                .await?;
            fixture
                .run_sql_on_node_with_retry(
                    bootstrap_primary.as_str(),
                    "INSERT INTO ha_partition_pg_path_primary (id, payload) VALUES (1, 'before-pg-path-fault') ON CONFLICT (id) DO UPDATE SET payload = EXCLUDED.payload",
                    E2E_PARTITION_WRITE_TIMEOUT,
                )
                .await?;

            let baseline_rows = vec!["1:before-pg-path-fault".to_string()];
            for node_id in fixture.node_ids() {
                fixture
                    .wait_for_rows_on_node(
                        node_id.as_str(),
                        "SELECT id::text || ':' || payload FROM ha_partition_pg_path_primary ORDER BY id",
                        baseline_rows.as_slice(),
                        E2E_PARTITION_REPLICATION_CONVERGENCE_TIMEOUT,
                    )
                    .await?;
            }

            fixture
                .wait_for_replication_sender_port_on_replicas(
                    bootstrap_primary.as_str(),
                    primary_node.sql_port,
                    E2E_PARTITION_REPLICATION_PATH_TIMEOUT,
                )
                .await?;

            fixture
                .isolate_postgres_path(bootstrap_primary.as_str())
                .await?;
            fixture
                .wait_for_replication_path_interruption(
                    bootstrap_primary.as_str(),
                    E2E_PARTITION_REPLICATION_PATH_TIMEOUT,
                )
                .await?;
            fixture
                .assert_no_dual_primary_window(E2E_PARTITION_MEDIUM_NO_DUAL_PRIMARY_WINDOW)
                .await?;

            fixture.record(format!(
                "postgres-path isolation: write second row via raw postgres port node={} raw_port={} advertised_port={}",
                bootstrap_primary, primary_node.pg_port, primary_node.sql_port
            ));
            // This direct write proves the primary stays locally writable even while the
            // advertised postgres endpoint used by replicas is blocked by the pg proxy.
            fixture
                .run_sql_on_node_direct_postgres(
                    bootstrap_primary.as_str(),
                    "INSERT INTO ha_partition_pg_path_primary (id, payload) VALUES (2, 'during-pg-path-fault') ON CONFLICT (id) DO UPDATE SET payload = EXCLUDED.payload",
                )
                .await?;

            fixture
                .assert_query_rows_stay_equal_on_nodes(
                    replica_ids.as_slice(),
                    "SELECT COUNT(*)::bigint FROM ha_partition_pg_path_primary WHERE id = 2",
                    &["0".to_string()],
                    E2E_PARTITION_SHORT_NO_DUAL_PRIMARY_WINDOW,
                    "postgres-path isolation: row written during fault should stay absent on replicas",
                )
                .await?;
            fixture.record(format!(
                "postgres-path isolation: replicas kept row id=2 absent during fault window nodes={replica_ids:?}"
            ));

            fixture.heal_all_network_faults().await?;
            let healed_primary = fixture
                .wait_for_stable_primary_resilient(StablePrimaryWaitPlan {
                    context: "postgres-path isolation: healed stable primary",
                    timeout: E2E_PARTITION_RECOVERY_TIMEOUT,
                    excluded_primary: None,
                    required_consecutive: 5,
                    fallback_timeout: E2E_PARTITION_RECOVERY_TIMEOUT,
                    fallback_required_consecutive: 3,
                    min_observed_nodes: 2,
                })
                .await?;
            fixture.record(format!(
                "postgres-path isolation: healed primary={healed_primary}"
            ));
            if healed_primary != bootstrap_primary {
                return Err(WorkerError::Message(format!(
                    "postgres-path isolation should not rotate primary; expected={bootstrap_primary} observed={healed_primary}"
                )));
            }

            let expected_rows = vec![
                "1:before-pg-path-fault".to_string(),
                "2:during-pg-path-fault".to_string(),
            ];
            for node_id in fixture.node_ids() {
                fixture
                    .wait_for_rows_on_node(
                        node_id.as_str(),
                        "SELECT id::text || ':' || payload FROM ha_partition_pg_path_primary ORDER BY id",
                        expected_rows.as_slice(),
                        E2E_PARTITION_REPLICATION_CONVERGENCE_TIMEOUT,
                    )
                    .await?;
            }
            let digests = fixture
                .wait_for_table_digest_convergence(
                    "ha_partition_pg_path_primary",
                    fixture.node_ids().as_slice(),
                    2,
                    E2E_PARTITION_REPLICATION_CONVERGENCE_TIMEOUT,
                )
                .await?;
            fixture.record(format!(
                "postgres-path isolation: digest convergence observed after heal digests={digests:?}"
            ));
            fixture
                .assert_no_dual_primary_window(E2E_PARTITION_SHORT_NO_DUAL_PRIMARY_WINDOW)
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

pub async fn e2e_partition_mixed_faults_heal_converges() -> Result<(), WorkerError> {
    ha_e2e::util::run_with_local_set(async {
        let mut fixture = PartitionFixture::start(3).await?;
        let scenario_name = "ha-e2e-partition-mixed-faults-heal";

        let run_result = match tokio::time::timeout(E2E_SCENARIO_TIMEOUT, async {
            fixture.record("mixed faults: wait for initial stable primary");
            let bootstrap_primary = fixture
                .wait_for_stable_primary_resilient(StablePrimaryWaitPlan {
                    context: "mixed faults: initial stable primary",
                    timeout: Duration::from_secs(90),
                    excluded_primary: None,
                    required_consecutive: 5,
                    fallback_timeout: Duration::from_secs(90),
                    fallback_required_consecutive: 3,
                    min_observed_nodes: 2,
                })
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
                .wait_for_stable_primary_resilient(StablePrimaryWaitPlan {
                    context: "mixed faults: healed stable primary",
                    timeout: Duration::from_secs(120),
                    excluded_primary: None,
                    required_consecutive: 5,
                    fallback_timeout: Duration::from_secs(120),
                    fallback_required_consecutive: 3,
                    min_observed_nodes: 2,
                })
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

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn family_symbols_remain_reachable_for_split_targets() {
        let _ = E2E_COMMAND_TIMEOUT;
        let _ = E2E_COMMAND_KILL_WAIT_TIMEOUT;
        let _ = E2E_PG_STOP_TIMEOUT;
        let _ = E2E_HTTP_STEP_TIMEOUT;
        let _ = E2E_BOOTSTRAP_PRIMARY_TIMEOUT;
        let _ = E2E_SCENARIO_TIMEOUT;
        let _ = PARTITION_ARTIFACT_DIR;
        let _: Option<StablePrimaryWaitPlan<'static>> = None;
        let _: Option<PartitionFixture> = None;
        let _ = PartitionFixture::start;
        let _: fn(&mut PartitionFixture, String) = PartitionFixture::record;
        let _ = PartitionFixture::node_ids;
        let _ = PartitionFixture::node_by_id;
        let _ = PartitionFixture::set_etcd_mode_for_node;
        let _ = PartitionFixture::partition_node_from_etcd;
        let _ = PartitionFixture::partition_primary_from_etcd;
        let _ = PartitionFixture::isolate_api_path;
        let _ = PartitionFixture::set_pg_mode_for_node;
        let _ = PartitionFixture::isolate_postgres_path;
        let _ = PartitionFixture::heal_all_network_faults;
        let _ = PartitionFixture::fetch_node_ha_state;
        let _ = PartitionFixture::cluster_ha_states_best_effort;
        let _ = PartitionFixture::cluster_ha_states_strict;
        let _ = PartitionFixture::wait_for_stable_primary;
        let _ = PartitionFixture::wait_for_stable_primary_best_effort;
        let _ = PartitionFixture::wait_for_stable_primary_via_sql;
        let _ = PartitionFixture::wait_for_stable_primary_resilient;
        let _ = PartitionFixture::primary_members;
        let _ = PartitionFixture::replica_node_ids;
        let _ = PartitionFixture::assert_no_dual_primary_window;
        let _ = PartitionFixture::wait_for_node_phase;
        let _ = PartitionFixture::run_sql_on_port;
        let _ = PartitionFixture::run_sql_on_node;
        let _ = PartitionFixture::run_sql_on_node_direct_postgres;
        let _ = PartitionFixture::cluster_sql_roles_best_effort;
        let _ = PartitionFixture::run_sql_on_node_with_retry;
        let _ = PartitionFixture::wait_for_replication_sender_port_on_replicas;
        let _ = PartitionFixture::wait_for_replication_path_interruption;
        let _ = PartitionFixture::assert_query_rows_stay_equal_on_nodes;
        let _ = PartitionFixture::wait_for_rows_on_node;
        let _ = PartitionFixture::wait_for_table_digest_convergence;
        let _ = PartitionFixture::write_timeline_artifact;
        let _ = PartitionFixture::ensure_runtime_tasks_healthy;
        let _ = PartitionFixture::shutdown;
        let _ = sanitize_component as fn(&str) -> String;
        let _ = finalize_partition_scenario;
        let _ = e2e_partition_minority_isolation_no_split_brain_rejoin;
        let _ = e2e_partition_primary_isolation_failover_no_split_brain;
        let _ = e2e_partition_api_path_isolation_preserves_primary;
        let _ = e2e_partition_primary_postgres_path_blocked_replicas_catch_up_after_heal;
        let _ = e2e_partition_mixed_faults_heal_converges;
    }
}
