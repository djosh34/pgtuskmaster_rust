use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use cucumber::World;

use crate::support::{
    binary_paths,
    docker::{cli::DockerCli, ryuk::RyukGuard},
    error::{HarnessError, Result},
    faults::{
        append_fault_rule_script, clear_fault_rules_script, ensure_fault_plumbing_script,
        remove_file_script, signal_named_process_script, touch_file_script, BlockerKind,
        TrafficPath, ALL_CLUSTER_MEMBERS, ETCD_SERVICE, OBSERVER_SERVICE,
    },
    feature_metadata,
    givens::given_root,
    observer::{
        pgtm::{ClusterStatusView, PgtmObserver},
        sql::SqlObserver,
    },
    timeouts::TimeoutModel,
    workload::{SqlWorkloadHandle, WorkloadSummary},
};

#[derive(Debug, Default, World)]
pub struct HaWorld {
    pub harness: Option<HarnessShared>,
    pub scenario: ScenarioState,
}

#[derive(Debug, Default)]
pub struct ScenarioState {
    pub aliases: BTreeMap<String, String>,
    pub active_workload: Option<SqlWorkloadHandle>,
    pub last_command_output: Option<String>,
    pub last_workload_summary: Option<WorkloadSummary>,
    pub markers: BTreeMap<String, u128>,
    pub unsampled_nodes: BTreeSet<String>,
    pub stopped_nodes: BTreeSet<String>,
    pub proof_rows: Vec<String>,
    pub proof_table: Option<String>,
    pub observed_primaries: Vec<String>,
}

impl HaWorld {
    pub fn reset(&mut self) {
        self.harness = None;
        self.scenario = ScenarioState::default();
    }

    pub fn harness(&self) -> Result<&HarnessShared> {
        self.harness
            .as_ref()
            .ok_or_else(|| HarnessError::message("scenario harness has not been initialized"))
    }

    pub fn set_harness(&mut self, harness: HarnessShared) {
        self.harness = Some(harness);
    }

    pub fn record_marker(&mut self, marker: &str, timestamp_ms: u128) {
        self.scenario
            .markers
            .insert(marker.to_string(), timestamp_ms);
    }

    pub fn marker(&self, marker: &str) -> Result<u128> {
        self.scenario
            .markers
            .get(marker)
            .copied()
            .ok_or_else(|| HarnessError::message(format!("marker `{marker}` was not recorded")))
    }

    pub fn remember_alias(&mut self, alias: &str, member_id: String) {
        self.scenario.aliases.insert(alias.to_string(), member_id);
    }

    pub fn require_alias(&self, alias: &str) -> Result<String> {
        self.scenario
            .aliases
            .get(alias)
            .cloned()
            .ok_or_else(|| HarnessError::message(format!("alias `{alias}` was not recorded")))
    }

    pub fn add_stopped_node(&mut self, member_id: &str) {
        let _ = self.scenario.stopped_nodes.insert(member_id.to_string());
    }

    pub fn remove_stopped_node(&mut self, member_id: &str) {
        let _ = self.scenario.stopped_nodes.remove(member_id);
    }

    pub fn add_unsampled_node(&mut self, member_id: &str) {
        let _ = self.scenario.unsampled_nodes.insert(member_id.to_string());
    }

    pub fn remove_unsampled_node(&mut self, member_id: &str) {
        let _ = self.scenario.unsampled_nodes.remove(member_id);
    }

    pub fn clear_unsampled_nodes(&mut self) {
        self.scenario.unsampled_nodes.clear();
    }

    pub fn clear_primary_history(&mut self) {
        self.scenario.observed_primaries.clear();
    }

    pub fn record_primary_observation(&mut self, member_id: &str) {
        let already_recorded = self
            .scenario
            .observed_primaries
            .iter()
            .any(|observed| observed == member_id);
        if !already_recorded {
            self.scenario.observed_primaries.push(member_id.to_string());
        }
    }

    pub fn cleanup(&mut self) -> Result<()> {
        let workload_result = self
            .scenario
            .active_workload
            .take()
            .map(SqlWorkloadHandle::stop)
            .transpose();
        let cleanup_result = match self.harness.as_mut() {
            Some(harness) => harness.cleanup(),
            None => Ok(()),
        };
        self.reset();

        match (workload_result, cleanup_result) {
            (Ok(None), Ok(())) | (Ok(Some(_)), Ok(())) => Ok(()),
            (Err(workload), Ok(())) => Err(workload),
            (Ok(None), Err(cleanup)) | (Ok(Some(_)), Err(cleanup)) => Err(cleanup),
            (Err(workload), Err(cleanup)) => Err(HarnessError::message(format!(
                "{workload}\ncleanup also failed: {cleanup}"
            ))),
        }
    }
}

#[derive(Debug)]
pub struct HarnessShared {
    pub run_id: String,
    pub feature_name: String,
    pub given_name: String,
    pub run_dir: PathBuf,
    pub source_copy_dir: PathBuf,
    pub artifacts_dir: PathBuf,
    pub compose_file: PathBuf,
    pub compose_project: String,
    pub docker: DockerCli,
    pub ryuk: Option<RyukGuard>,
    pub observer_container: String,
    pub timeouts: TimeoutModel,
    timeline: Mutex<Vec<serde_json::Value>>,
    cleaned_up: bool,
}

impl HarnessShared {
    pub async fn initialize(given_name: &str) -> Result<Self> {
        let feature = feature_metadata()?;
        let docker = DockerCli::discover()?;
        docker.verify_daemon()?;

        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let given_root = given_root(repo_root.as_path(), given_name)?;
        let run_id = build_run_id(feature.feature_name.as_str())?;
        let compose_project = build_compose_project(feature.feature_name.as_str(), run_id.as_str());
        let run_dir = repo_root
            .join("cucumber_tests/ha/runs")
            .join(feature.feature_name.as_str())
            .join(run_id.as_str());
        let source_copy_dir = run_dir.join("source-copy");
        let artifacts_dir = run_dir.join("artifacts");
        create_dir_all(run_dir.as_path())?;
        create_dir_all(source_copy_dir.as_path())?;
        create_dir_all(artifacts_dir.as_path())?;
        copy_directory(given_root.as_path(), source_copy_dir.as_path())?;
        copy_test_binaries(source_copy_dir.as_path())?;

        let compose_file = source_copy_dir.join("compose.yml");
        let timeouts = TimeoutModel::from_runtime_config(
            source_copy_dir
                .join("configs/node-a/runtime.toml")
                .as_path(),
        )?;
        let ryuk = RyukGuard::start(docker.clone(), compose_project.as_str())?;
        docker.compose_up_services(compose_file.as_path(), compose_project.as_str(), &["etcd"])?;
        docker.compose_up_services(
            compose_file.as_path(),
            compose_project.as_str(),
            &["observer"],
        )?;
        let observer_container = docker.compose_container_id(
            compose_file.as_path(),
            compose_project.as_str(),
            "observer",
        )?;

        let mut harness = Self {
            run_id,
            feature_name: feature.feature_name.clone(),
            given_name: given_name.to_string(),
            run_dir,
            source_copy_dir,
            artifacts_dir,
            compose_file,
            compose_project,
            docker,
            ryuk: Some(ryuk),
            observer_container,
            timeouts,
            timeline: Mutex::new(Vec::new()),
            cleaned_up: false,
        };
        harness.record_note("initialize", "created per-feature run workspace")?;
        if let Err(err) = harness.bootstrap_cluster().await {
            let cleanup_error = harness.cleanup().err();
            return match cleanup_error {
                None => Err(err),
                Some(cleanup) => Err(HarnessError::message(format!(
                    "{err}\ncleanup after bootstrap failure also failed: {cleanup}"
                ))),
            };
        }
        Ok(harness)
    }

    pub fn observer(&self) -> PgtmObserver {
        PgtmObserver::new(self.docker.clone(), self.observer_container.clone())
    }

    pub fn sql(&self) -> SqlObserver {
        SqlObserver::new(self.docker.clone(), self.observer_container.clone())
    }

    pub fn kill_node(&self, member_id: &str) -> Result<()> {
        let container_id = self.service_container_id(member_id)?;
        self.record_note("docker.kill", format!("killing `{member_id}`"))?;
        self.docker.kill_container(container_id.as_str())
    }

    pub fn start_node(&self, member_id: &str) -> Result<()> {
        let container_id = self.service_container_id(member_id)?;
        self.record_note("docker.start", format!("starting `{member_id}`"))?;
        self.docker.start_container(container_id.as_str())
    }

    pub fn record_note(&self, phase: &str, detail: impl Into<String>) -> Result<()> {
        self.push_timeline_entry(serde_json::json!({
            "kind": "note",
            "phase": phase,
            "detail": detail.into(),
            "timestamp_ms": timestamp_millis()?,
        }))
    }

    pub fn record_status_snapshot(&self, phase: &str, status: &ClusterStatusView) -> Result<()> {
        self.push_timeline_entry(serde_json::json!({
            "kind": "status",
            "phase": phase,
            "timestamp_ms": timestamp_millis()?,
            "status": status,
        }))
    }

    pub fn service_container_id(&self, service: &str) -> Result<String> {
        self.docker.compose_container_id(
            self.compose_file.as_path(),
            self.compose_project.as_str(),
            service,
        )
    }

    pub fn service_logs(&self, service: &str) -> Result<String> {
        let container_id = self.service_container_id(service)?;
        self.docker.container_logs(container_id.as_str())
    }

    pub fn write_artifact_json(
        &self,
        artifact_name: &str,
        value: &serde_json::Value,
    ) -> Result<()> {
        let path = self.artifacts_dir.join(artifact_name);
        let content = serde_json::to_string_pretty(value).map_err(|source| HarnessError::Json {
            context: format!("serializing artifact `{artifact_name}`"),
            source,
        })?;
        write_text_file(path.as_path(), content.as_str())
    }

    pub fn stop_service(&self, service: &str) -> Result<()> {
        let container_id = self.service_container_id(service)?;
        self.record_note("docker.stop_service", format!("stopping `{service}`"))?;
        self.docker.kill_container(container_id.as_str())
    }

    pub fn start_service(&self, service: &str) -> Result<()> {
        let container_id = self.service_container_id(service)?;
        self.record_note("docker.start_service", format!("starting `{service}`"))?;
        self.docker.start_container(container_id.as_str())
    }

    pub fn run_shell_as_root(&self, service: &str, script: &str) -> Result<String> {
        let container_id = self.service_container_id(service)?;
        self.docker.exec_as_user(
            container_id.as_str(),
            "root",
            Path::new("/bin/sh"),
            &["-lc", script],
        )
    }

    pub fn ensure_fault_plumbing(&self, member_id: &str) -> Result<()> {
        let script = ensure_fault_plumbing_script();
        let _ = self.run_shell_as_root(member_id, script.as_str())?;
        self.record_note("fault.ensure_plumbing", format!("member={member_id}"))?;
        Ok(())
    }

    pub fn clear_network_faults(&self, member_id: &str) -> Result<()> {
        if !self.service_is_running(member_id)? {
            self.record_note(
                "fault.clear_network",
                format!("member={member_id} skipped=container_not_running"),
            )?;
            return Ok(());
        }
        let script = clear_fault_rules_script();
        if let Err(err) = self.run_shell_as_root(member_id, script.as_str()) {
            if container_not_running_error(&err) {
                self.record_note(
                    "fault.clear_network",
                    format!("member={member_id} skipped=container_not_running"),
                )?;
                return Ok(());
            }
            return Err(err);
        }
        self.record_note("fault.clear_network", format!("member={member_id}"))?;
        Ok(())
    }

    pub fn block_member_path_to_host(
        &self,
        member_id: &str,
        path: TrafficPath,
        peer_service: &str,
    ) -> Result<()> {
        let peer_container_id = self.service_container_id(peer_service)?;
        let peer_ip = self
            .docker
            .container_ipv4_address(peer_container_id.as_str())?;
        self.ensure_fault_plumbing(member_id)?;
        let script = append_fault_rule_script(peer_ip.as_str(), path.port());
        let _ = self.run_shell_as_root(member_id, script.as_str())?;
        self.record_note(
            "fault.block_path",
            format!(
                "member={member_id} path={} peer={peer_service}",
                path.label()
            ),
        )?;
        Ok(())
    }

    pub fn isolate_member_from_peer_on_path(
        &self,
        member_id: &str,
        peer_id: &str,
        path: TrafficPath,
    ) -> Result<()> {
        self.block_member_path_to_host(member_id, path, peer_id)?;
        self.block_member_path_to_host(peer_id, path, member_id)
    }

    pub fn isolate_member_from_all_peers_on_path(
        &self,
        member_id: &str,
        path: TrafficPath,
    ) -> Result<()> {
        ALL_CLUSTER_MEMBERS
            .iter()
            .filter(|peer_id| **peer_id != member_id)
            .try_for_each(|peer_id| self.isolate_member_from_peer_on_path(member_id, peer_id, path))
    }

    pub fn isolate_member_from_observer_on_api(&self, member_id: &str) -> Result<()> {
        self.block_member_path_to_host(member_id, TrafficPath::Api, OBSERVER_SERVICE)
    }

    pub fn cut_member_off_from_dcs(&self, member_id: &str) -> Result<()> {
        self.block_member_path_to_host(member_id, TrafficPath::Dcs, ETCD_SERVICE)
    }

    pub fn set_blocker(&self, member_id: &str, blocker: BlockerKind, enabled: bool) -> Result<()> {
        let container_id = self.service_container_id(member_id)?;
        let script = if enabled {
            touch_file_script(blocker.marker_path())
        } else {
            remove_file_script(blocker.marker_path())
        };
        if enabled && !self.service_is_running(member_id)? {
            self.docker
                .touch_file_in_container(container_id.as_str(), blocker.marker_path())?;
        } else if let Err(err) = self.run_shell_as_root(member_id, script.as_str()) {
            if enabled && container_not_running_error(&err) {
                self.docker
                    .touch_file_in_container(container_id.as_str(), blocker.marker_path())?;
            } else {
                return Err(err);
            }
        }
        self.record_note(
            "fault.blocker",
            format!(
                "member={member_id} blocker={} enabled={enabled}",
                blocker.label()
            ),
        )?;
        Ok(())
    }

    pub fn wipe_member_data_dir(&self, member_id: &str) -> Result<()> {
        let container_id = self.service_container_id(member_id)?;
        let marker_path = "/var/lib/pgtuskmaster/faults/wipe-data-on-start";
        if self.service_is_running(member_id)? {
            let script = touch_file_script(marker_path);
            if let Err(err) = self.run_shell_as_root(member_id, script.as_str()) {
                if container_not_running_error(&err) {
                    self.docker
                        .touch_file_in_container(container_id.as_str(), marker_path)?;
                } else {
                    return Err(err);
                }
            }
        } else {
            self.docker
                .touch_file_in_container(container_id.as_str(), marker_path)?;
        }
        self.record_note("fault.wipe_data_dir", format!("member={member_id}"))?;
        Ok(())
    }

    pub fn wedge_member_postgres(&self, member_id: &str) -> Result<()> {
        let script = signal_named_process_script("STOP", "postgres");
        let _ = self.run_shell_as_root(member_id, script.as_str())?;
        self.record_note("fault.wedge_postgres", format!("member={member_id}"))?;
        Ok(())
    }

    pub fn unwedge_member_postgres(&self, member_id: &str) -> Result<()> {
        let script = signal_named_process_script("CONT", "postgres");
        let _ = self.run_shell_as_root(member_id, script.as_str())?;
        self.record_note("fault.unwedge_postgres", format!("member={member_id}"))?;
        Ok(())
    }

    pub fn assert_no_dual_primary_evidence(&self) -> Result<()> {
        self.assert_no_dual_primary_evidence_since(None)
    }

    pub fn assert_no_dual_primary_evidence_since(&self, since_ms: Option<u128>) -> Result<()> {
        let timeline = self.timeline_entries()?;
        let offending = timeline.iter().find_map(|entry| {
            let timestamp_ms = entry.get("timestamp_ms")?.as_u64()? as u128;
            if since_ms.is_some_and(|threshold| timestamp_ms < threshold) {
                return None;
            }
            let status = entry.get("status")?;
            let nodes = status.get("nodes")?.as_array()?;
            let primaries = nodes
                .iter()
                .filter(|node| {
                    node.get("sampled").and_then(serde_json::Value::as_bool) == Some(true)
                        && node.get("role").and_then(serde_json::Value::as_str) == Some("primary")
                })
                .map(|node| {
                    node.get("member_id")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("<unknown>")
                        .to_string()
                })
                .collect::<Vec<_>>();
            if primaries.len() > 1 {
                Some(primaries)
            } else {
                None
            }
        });
        match offending {
            Some(primaries) => Err(HarnessError::message(format!(
                "timeline captured dual-primary evidence: {}",
                primaries.join(", ")
            ))),
            None => Ok(()),
        }
    }

    pub fn assert_member_never_primary_since(&self, member_id: &str, since_ms: u128) -> Result<()> {
        let timeline = self.timeline_entries()?;
        let offending = timeline.iter().find(|entry| {
            let timestamp_ms = entry
                .get("timestamp_ms")
                .and_then(serde_json::Value::as_u64)
                .map(u128::from);
            if timestamp_ms.is_none_or(|value| value < since_ms) {
                return false;
            }
            entry
                .get("status")
                .and_then(|status| status.get("nodes"))
                .and_then(serde_json::Value::as_array)
                .is_some_and(|nodes| {
                    nodes.iter().any(|node| {
                        node.get("sampled").and_then(serde_json::Value::as_bool) == Some(true)
                            && node.get("role").and_then(serde_json::Value::as_str)
                                == Some("primary")
                            && node.get("member_id").and_then(serde_json::Value::as_str)
                                == Some(member_id)
                    })
                })
        });
        match offending {
            Some(_) => Err(HarnessError::message(format!(
                "timeline captured `{member_id}` as primary after marker {since_ms}"
            ))),
            None => Ok(()),
        }
    }

    pub fn clear_all_network_faults(&self) -> Result<()> {
        for service in ALL_CLUSTER_MEMBERS
            .iter()
            .copied()
            .chain([OBSERVER_SERVICE].into_iter())
        {
            self.clear_network_faults(service)?;
        }
        Ok(())
    }

    fn service_is_running(&self, service: &str) -> Result<bool> {
        let container_id = self.service_container_id(service)?;
        Ok(self.docker.container_state_status(container_id.as_str())? == "running")
    }

    async fn bootstrap_cluster(&self) -> Result<()> {
        self.wait_for_service_health("etcd").await?;
        self.record_note("bootstrap", "starting seed primary node-b")?;
        self.docker.compose_up_services(
            self.compose_file.as_path(),
            self.compose_project.as_str(),
            &["node-b"],
        )?;
        self.wait_for_seed_primary().await?;
        self.record_note("bootstrap", "starting remaining nodes node-a and node-c")?;
        self.docker.compose_up_services(
            self.compose_file.as_path(),
            self.compose_project.as_str(),
            &["node-a", "node-c"],
        )
    }

    async fn wait_for_service_health(&self, service: &str) -> Result<()> {
        let deadline = Instant::now() + self.timeouts.startup_deadline;
        let mut last_error = None;
        while Instant::now() < deadline {
            let result = match self
                .service_container_id(service)
                .and_then(|container_id| self.docker.container_health_status(container_id.as_str()))
            {
                Ok(Some(status)) if status == "healthy" => Ok(()),
                Ok(Some(status)) => Err(HarnessError::message(format!(
                    "service `{service}` health is `{status}`"
                ))),
                Ok(None) => Err(HarnessError::message(format!(
                    "service `{service}` does not expose a docker health status"
                ))),
                Err(err) => Err(err),
            };
            match result {
                Ok(()) => return Ok(()),
                Err(err) => last_error = Some(err.to_string()),
            }
            tokio::time::sleep(self.timeouts.poll_interval).await;
        }

        Err(HarnessError::message(format!(
            "timed out waiting for service `{service}` to become healthy; last observed error: {}",
            last_error.unwrap_or_else(|| "no health state was observed".to_string())
        )))
    }

    async fn wait_for_seed_primary(&self) -> Result<()> {
        let deadline = Instant::now() + self.timeouts.startup_deadline;
        let mut last_error = None;
        while Instant::now() < deadline {
            let result = match self.observer().status() {
                Ok(status) => {
                    self.record_status_snapshot("bootstrap.seed_primary", &status)?;
                    validate_seed_primary(&status)
                }
                Err(err) => Err(err),
            };
            match result {
                Ok(()) => return Ok(()),
                Err(err) => last_error = Some(err.to_string()),
            }
            tokio::time::sleep(self.timeouts.poll_interval).await;
        }

        Err(HarnessError::message(format!(
            "timed out waiting for bootstrap primary before starting replicas; last observed error: {}",
            last_error.unwrap_or_else(|| "no status was observed".to_string())
        )))
    }

    pub fn cleanup(&mut self) -> Result<()> {
        if self.cleaned_up {
            return Ok(());
        }

        let mut failures = Vec::new();
        let capture_result = self.capture_artifacts();
        if let Err(err) = &capture_result {
            failures.push(format!("artifact capture failed: {err}"));
        }
        let compose_result = self
            .docker
            .compose_down(self.compose_file.as_path(), self.compose_project.as_str())
            ;
        if let Err(err) = &compose_result {
            failures.push(format!("docker compose down failed: {err}"));
        }
        let ryuk_result = self
            .ryuk
            .as_mut()
            .map(RyukGuard::close)
            .transpose();
        if let Err(err) = &ryuk_result {
                failures.push(format!("ryuk cleanup failed: {err}"));
        }
        if compose_result.is_ok() && ryuk_result.is_ok() {
            self.cleaned_up = true;
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(HarnessError::message(failures.join("\n")))
        }
    }

    fn capture_artifacts(&self) -> Result<()> {
        let mut failures = Vec::new();
        write_text_file(
            self.artifacts_dir.join("compose-ps.json").as_path(),
            serde_json::to_string_pretty(
                &self.docker.compose_ps_entries(
                    self.compose_file.as_path(),
                    self.compose_project.as_str(),
                )?,
            )
            .map_err(|source| HarnessError::Json {
                context: "serializing docker compose ps json".to_string(),
                source,
            })?
            .as_str(),
        )?;
        write_text_file(
            self.artifacts_dir.join("compose-logs.txt").as_path(),
            self.docker
                .compose_logs(self.compose_file.as_path(), self.compose_project.as_str())?
                .as_str(),
        )?;
        write_text_file(
            self.artifacts_dir.join("run-metadata.json").as_path(),
            serde_json::to_string_pretty(&serde_json::json!({
                "feature_name": self.feature_name,
                "given_name": self.given_name,
                "run_id": self.run_id,
                "run_dir": self.run_dir,
                "source_copy_dir": self.source_copy_dir,
                "artifacts_dir": self.artifacts_dir,
                "compose_project": self.compose_project,
            }))
            .map_err(|source| HarnessError::Json {
                context: "serializing run metadata".to_string(),
                source,
            })?
            .as_str(),
        )?;
        let timeline = self.timeline_entries()?;
        write_text_file(
            self.artifacts_dir.join("timeline.json").as_path(),
            serde_json::to_string_pretty(&timeline)
                .map_err(|source| HarnessError::Json {
                    context: "serializing cucumber timeline".to_string(),
                    source,
                })?
                .as_str(),
        )?;
        match self.observer().debug_verbose() {
            Ok(debug) => write_text_file(
                self.artifacts_dir
                    .join("observer-debug-verbose.json")
                    .as_path(),
                serde_json::to_string_pretty(&debug)
                    .map_err(|source| HarnessError::Json {
                        context: "serializing observer debug verbose payload".to_string(),
                        source,
                    })?
                    .as_str(),
            )?,
            Err(err) => failures.push(format!("observer debug verbose capture failed: {err}")),
        }

        for service in ["observer", "node-a", "node-b", "node-c", "etcd"] {
            match self.service_container_id(service) {
                Ok(container_id) => match self.docker.inspect_container(container_id.as_str()) {
                    Ok(inspect) => {
                        let artifact = self.artifacts_dir.join(format!("inspect-{service}.json"));
                        write_text_file(artifact.as_path(), inspect.as_str())?;
                    }
                    Err(err) => failures.push(format!(
                        "docker inspect artifact capture failed for `{service}`: {err}"
                    )),
                },
                Err(err) => failures.push(format!(
                    "container resolution failed for artifact capture `{service}`: {err}"
                )),
            }
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(HarnessError::message(failures.join("\n")))
        }
    }

    fn push_timeline_entry(&self, entry: serde_json::Value) -> Result<()> {
        let mut timeline = self
            .timeline
            .lock()
            .map_err(|_| HarnessError::message("timeline mutex was poisoned"))?;
        timeline.push(entry);
        Ok(())
    }

    fn timeline_entries(&self) -> Result<Vec<serde_json::Value>> {
        self.timeline
            .lock()
            .map(|guard| guard.clone())
            .map_err(|_| HarnessError::message("timeline mutex was poisoned"))
    }
}

fn container_not_running_error(err: &HarnessError) -> bool {
    matches!(err, HarnessError::CommandFailed { stderr, .. } if stderr.contains("is not running"))
}

fn copy_test_binaries(source_copy_dir: &Path) -> Result<()> {
    let binaries = binary_paths()?;
    let destination_root = source_copy_dir.join("docker_files/bin");
    create_dir_all(destination_root.as_path())?;
    copy_file(
        binaries.pgtuskmaster.as_path(),
        destination_root.join("pgtuskmaster").as_path(),
    )?;
    copy_file(
        binaries.pgtm.as_path(),
        destination_root.join("pgtm").as_path(),
    )?;
    Ok(())
}

fn build_run_id(feature_name: &str) -> Result<String> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| HarnessError::message(format!("system clock error: {err}")))?;
    Ok(format!(
        "{}-{}-{}",
        sanitize(feature_name),
        timestamp.as_millis(),
        std::process::id()
    ))
}

fn timestamp_millis() -> Result<u128> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .map_err(|err| HarnessError::message(format!("system clock error: {err}")))
}

fn build_compose_project(feature_name: &str, run_id: &str) -> String {
    let feature = sanitize(feature_name);
    let run = sanitize(run_id);
    format!("ha-{}-{}", feature, run)
}

fn sanitize(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
}

fn create_dir_all(path: &Path) -> Result<()> {
    fs::create_dir_all(path).map_err(|source| HarnessError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn copy_file(from: &Path, to: &Path) -> Result<()> {
    fs::copy(from, to)
        .map(|_| ())
        .map_err(|source| HarnessError::Io {
            path: to.to_path_buf(),
            source,
        })
}

fn write_text_file(path: &Path, content: &str) -> Result<()> {
    fs::write(path, content).map_err(|source| HarnessError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn copy_directory(from: &Path, to: &Path) -> Result<()> {
    if !from.is_dir() {
        return Err(HarnessError::message(format!(
            "source directory does not exist: {}",
            from.display()
        )));
    }

    let mut directories = vec![(from.to_path_buf(), to.to_path_buf())];
    while let Some((current_from, current_to)) = directories.pop() {
        create_dir_all(current_to.as_path())?;
        for entry in fs::read_dir(current_from.as_path()).map_err(|source| HarnessError::Io {
            path: current_from.clone(),
            source,
        })? {
            let entry = entry.map_err(|source| HarnessError::Io {
                path: current_from.clone(),
                source,
            })?;
            let source_path = entry.path();
            let destination_path = current_to.join(entry.file_name());
            if source_path.is_dir() {
                directories.push((source_path, destination_path));
            } else {
                copy_file(source_path.as_path(), destination_path.as_path())?;
            }
        }
    }
    Ok(())
}

fn validate_seed_primary(status: &ClusterStatusView) -> Result<()> {
    if status.sampled_member_count != 1 {
        return Err(HarnessError::message(format!(
            "expected exactly one sampled member during bootstrap, observed {}; warnings={}",
            status.sampled_member_count,
            format_bootstrap_warnings(status),
        )));
    }

    let primaries = status
        .nodes
        .iter()
        .filter(|node| node.sampled && node.role == "primary")
        .map(|node| node.member_id.as_str())
        .collect::<Vec<_>>();
    match primaries.as_slice() {
        ["node-b"] => Ok(()),
        [] => Err(HarnessError::message(format!(
            "bootstrap status has no sampled primary; queried via {} and warnings={}",
            status.queried_via.member_id,
            format_bootstrap_warnings(status),
        ))),
        [primary] => Err(HarnessError::message(format!(
            "expected node-b to bootstrap as the seed primary, observed `{primary}`"
        ))),
        many => Err(HarnessError::message(format!(
            "bootstrap status has multiple sampled primaries: {}",
            many.join(", ")
        ))),
    }
}

fn format_bootstrap_warnings(status: &ClusterStatusView) -> String {
    if status.warnings.is_empty() {
        "none".to_string()
    } else {
        status
            .warnings
            .iter()
            .map(|warning| format!("{}:{}", warning.code, warning.message))
            .collect::<Vec<_>>()
            .join(" | ")
    }
}
