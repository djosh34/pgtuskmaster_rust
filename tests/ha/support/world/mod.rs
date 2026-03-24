use std::{
    collections::{BTreeMap, BTreeSet},
    future::Future,
    path::{Path, PathBuf},
    sync::Mutex,
    thread,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use cucumber::World;
use pgtuskmaster_rust::api::{authoritative_primary_member, NodeState};
use tokio::{
    runtime::{Builder, Handle, RuntimeFlavor},
    task::JoinHandle,
};

use crate::support::{
    docker::{cli::DockerCli, ryuk::RyukGuard},
    error::{HarnessError, Result},
    faults::{
        append_fault_rule_script, clear_fault_rules_script, ensure_fault_plumbing_script,
        remove_fault_marker, remove_fault_rule_script, write_fault_marker, BlockerKind,
        TrafficPath, DATABASE_MEMBERS, WIPE_DATA_ON_START_MARKER_PATH,
    },
    feature_name,
    files::{create_dir_all, write_text_file},
    givens::HaGivenId,
    invariants::{
        PrimaryCountInvariantRunner, WriteConvergenceInvariantError,
        WriteConvergenceInvariantRunner,
    },
    observer::pgtm::PgtmObserver,
    timeouts::TimeoutModel,
    topology::{ClusterMember, DcsService},
};

#[derive(Debug, Default, World)]
pub struct HaWorld {
    pub harness: Option<HarnessShared>,
    pub stopped_members: BTreeSet<ClusterMember>,
    pub observer_unreachable_members: BTreeSet<ClusterMember>,
    scenario_name: Option<String>,
    aliases_by_name: BTreeMap<String, ClusterMember>,
}

impl HaWorld {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn harness(&self) -> Result<&HarnessShared> {
        self.harness
            .as_ref()
            .ok_or_else(|| HarnessError::message("scenario harness has not been initialized"))
    }

    pub fn set_scenario_name(&mut self, scenario_name: String) {
        self.scenario_name = Some(scenario_name);
    }

    pub fn scenario_name(&self) -> Result<&str> {
        self.scenario_name
            .as_deref()
            .ok_or_else(|| HarnessError::message("scenario name has not been initialized"))
    }

    pub fn set_harness(&mut self, harness: HarnessShared) {
        self.harness = Some(harness);
    }

    pub fn remember_member_alias(&mut self, alias: impl Into<String>, member: ClusterMember) {
        self.aliases_by_name.insert(alias.into(), member);
    }

    pub fn record_member_alias(
        &mut self,
        alias: &str,
        member: ClusterMember,
        phase: &str,
    ) -> Result<()> {
        self.harness()?
            .record_note(phase, format!("alias `{alias}` -> `{member}`"))?;
        self.remember_member_alias(alias, member);
        Ok(())
    }

    pub fn require_member_alias(&self, alias: &str) -> Result<ClusterMember> {
        self.aliases_by_name
            .get(alias)
            .copied()
            .ok_or_else(|| HarnessError::message(format!("alias `{alias}` was not recorded")))
    }

    pub fn resolve_member_reference(&self, member_ref: &str) -> Result<ClusterMember> {
        self.require_member_alias(member_ref)
            .or_else(|_| ClusterMember::parse(member_ref))
    }

    pub fn kill_nodes(&mut self, members: impl IntoIterator<Item = ClusterMember>) -> Result<()> {
        for member in members {
            self.harness()?.run_container_action(
                member.service_name(),
                "docker.kill",
                format!("killing `{member}`"),
                DockerCli::kill_container,
            )?;
            let _ = self.stopped_members.insert(member);
        }
        Ok(())
    }

    pub fn start_nodes(&mut self, members: impl IntoIterator<Item = ClusterMember>) -> Result<()> {
        for member in members {
            self.harness()?.run_container_action(
                member.service_name(),
                "docker.start",
                format!("starting `{member}`"),
                DockerCli::start_container,
            )?;
            let _ = self.stopped_members.remove(&member);
        }
        Ok(())
    }

    pub fn fully_isolate_node_from_cluster(&mut self, member: ClusterMember) -> Result<()> {
        let harness = self.harness()?;
        for path in [TrafficPath::Dcs, TrafficPath::Api, TrafficPath::Postgres] {
            harness.isolate_member_from_all_peers_on_path(member, path)?;
        }
        harness.block_member_path_to_host(
            member,
            TrafficPath::Dcs,
            harness.given.local_dcs_service_for(member).service_name(),
        )?;
        let gateway_ip = harness.member_network_gateway_ipv4(member)?;
        for (path, peer_label) in [
            (TrafficPath::Api, "host-operator"),
            (TrafficPath::Postgres, "host-observer-postgres"),
        ] {
            harness.block_member_path_to_address(member, path, gateway_ip.as_str(), peer_label)?;
        }
        let _ = self.observer_unreachable_members.insert(member);
        Ok(())
    }

    pub fn isolate_node_from_observer_api_access(&mut self, member: ClusterMember) -> Result<()> {
        let harness = self.harness()?;
        let gateway_ip = harness.member_network_gateway_ipv4(member)?;
        harness.block_member_path_to_address(
            member,
            TrafficPath::Api,
            gateway_ip.as_str(),
            "host-operator",
        )?;
        let _ = self.observer_unreachable_members.insert(member);
        Ok(())
    }

    pub fn heal_node_network_faults(&mut self, member: ClusterMember) -> Result<()> {
        self.harness()?.heal_member_network_faults(member)?;
        let _ = self.observer_unreachable_members.remove(&member);
        Ok(())
    }

    pub fn heal_all_network_faults(&mut self) -> Result<()> {
        let harness = self.harness()?;
        for service in DATABASE_MEMBERS {
            harness.clear_network_faults(service.service_name())?;
        }
        self.observer_unreachable_members.clear();
        Ok(())
    }

    pub fn online_expected_count(&self) -> usize {
        self.online_member_ids().len()
    }

    pub fn online_member_ids(&self) -> Vec<ClusterMember> {
        ClusterMember::ALL
            .into_iter()
            .filter(|member| {
                !self.stopped_members.contains(member)
                    && !self.observer_unreachable_members.contains(member)
            })
            .collect()
    }

    pub fn cleanup(&mut self) -> Result<()> {
        let cleanup_result = match self.harness.as_mut() {
            Some(harness) => harness.cleanup(),
            None => Ok(()),
        };
        self.reset();

        cleanup_result
    }
}

#[derive(Debug)]
enum WriteConvergenceState {
    NotStarted,
    Starting {
        started_at: Instant,
        task: Option<
            JoinHandle<
                std::result::Result<
                    WriteConvergenceInvariantRunner,
                    WriteConvergenceInvariantError,
                >,
            >,
        >,
    },
    Running(WriteConvergenceInvariantRunner),
    Failed(String),
}

#[derive(Clone, Copy, Debug)]
enum StartupPhase {
    WorkspaceReady,
    InvariantPrimaryCount,
    DcsReady,
    SeedPrimaryReady,
    ClusterReady,
    InvariantWriteConvergence,
}

#[derive(Debug)]
pub struct HarnessShared {
    pub run_id: String,
    pub feature_name: String,
    pub given: HaGivenId,
    pub run_dir: PathBuf,
    pub materialized_dir: PathBuf,
    pub artifacts_dir: PathBuf,
    pub compose_file: PathBuf,
    pub compose_project: String,
    pub cucumber_test_image_run_id: String,
    pub docker: DockerCli,
    pub(crate) observer: PgtmObserver,
    pub ryuk: Option<RyukGuard>,
    pub timeouts: TimeoutModel,
    service_container_ids: Mutex<BTreeMap<String, String>>,
    timeline: Mutex<Vec<serde_json::Value>>,
    cleaned_up: bool,
    primary_count_invariant: PrimaryCountInvariantRunner,
    write_convergence_state: Mutex<WriteConvergenceState>,
}

impl HarnessShared {
    pub async fn initialize(given: HaGivenId, scenario_name: &str) -> Result<Self> {
        let feature_name = feature_name()?;
        let docker = DockerCli::discover()?;
        docker.verify_daemon()?;

        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let run_id = build_run_id(feature_name, scenario_name)?;
        let cucumber_test_image_run_id = required_env("PGTM_CUCUMBER_TEST_RUN_ID")?;
        let run_dir = repo_root
            .join("tests/ha/runs")
            .join(feature_name)
            .join(run_id.as_str());
        let materialized_dir = run_dir.join("materialized");
        let artifacts_dir = run_dir.join("artifacts");
        let compose_project = build_compose_project(feature_name, run_id.as_str());
        let compose_file = materialized_dir.join("compose.yml");
        create_dir_all(run_dir.as_path())?;
        create_dir_all(materialized_dir.as_path())?;
        create_dir_all(artifacts_dir.as_path())?;
        given.materialize_fixture(repo_root.as_path(), materialized_dir.as_path())?;
        create_fault_directories(materialized_dir.as_path())?;

        let timeouts = TimeoutModel::from_runtime_config(
            materialized_dir
                .join(ClusterMember::SEED_PRIMARY.runtime_config_relative_path())
                .as_path(),
        )?;
        let ryuk = RyukGuard::start(docker.clone(), compose_project.as_str())?;
        let dcs_service_names = given
            .dcs_services()
            .iter()
            .copied()
            .map(|service| service.service_name())
            .collect::<Vec<_>>();
        docker.compose_up_services(
            compose_file.as_path(),
            compose_project.as_str(),
            dcs_service_names.as_slice(),
        )?;
        let observer = PgtmObserver::new(
            docker.clone(),
            compose_file.clone(),
            compose_project.clone(),
            materialized_dir.clone(),
        );
        let primary_count_invariant = PrimaryCountInvariantRunner::start(
            observer.clone(),
            timeouts.poll_interval,
            timeouts.failover_deadline,
        )
        .await?;
        let mut harness = Self {
            run_id,
            feature_name: feature_name.to_string(),
            given,
            run_dir,
            materialized_dir,
            artifacts_dir,
            compose_file,
            compose_project,
            cucumber_test_image_run_id,
            docker,
            observer,
            ryuk: Some(ryuk),
            timeouts,
            service_container_ids: Mutex::new(BTreeMap::new()),
            timeline: Mutex::new(Vec::new()),
            cleaned_up: false,
            primary_count_invariant,
            write_convergence_state: Mutex::new(WriteConvergenceState::NotStarted),
        };
        harness.record_startup_phase(
            StartupPhase::WorkspaceReady,
            format!(
                "materialized fixture `{}` and started DCS services",
                harness.given.as_str()
            ),
        )?;
        harness.record_note(
            "initialize",
            format!(
                "created per-feature run workspace using cucumber image run id `{}`",
                harness.cucumber_test_image_run_id
            ),
        )?;
        harness.record_startup_phase(
            StartupPhase::InvariantPrimaryCount,
            "started perpetual self-reported primary-count runner",
        )?;
        if let Err(err) = harness.bootstrap_cluster().await {
            let cleanup_error = harness.cleanup().err();
            return match cleanup_error {
                None => Err(err),
                Some(cleanup) => Err(HarnessError::message(format!(
                    "{err}\ncleanup after bootstrap failure also failed: {cleanup}"
                ))),
            };
        }
        if let Err(err) = harness.start_write_convergence_invariant() {
            let cleanup_error = harness.cleanup().err();
            return match cleanup_error {
                None => Err(err),
                Some(cleanup) => Err(HarnessError::message(format!(
                    "{err}\ncleanup after invariant startup failure also failed: {cleanup}"
                ))),
            };
        }
        Ok(harness)
    }

    pub fn ensure_primary_count_healthy(&self) -> Result<()> {
        self.primary_count_invariant.ensure_healthy()
    }

    pub fn ensure_accepted_writes_healthy(&self, online_members: &[ClusterMember]) -> Result<()> {
        self.with_write_convergence_runner(|runner| {
            runner
                .ensure_healthy(online_members)
                .map_err(HarnessError::from)
        })
    }

    pub fn ensure_accepted_writes_running(&self) -> Result<()> {
        self.with_write_convergence_runner(|runner| {
            runner.ensure_running().map_err(HarnessError::from)
        })
    }

    fn with_write_convergence_runner<T>(
        &self,
        check: impl Fn(&WriteConvergenceInvariantRunner) -> Result<T>,
    ) -> Result<T> {
        let (pending_started_at, pending_task) = {
            let mut state = self
                .write_convergence_state
                .lock()
                .map_err(|_| HarnessError::message("write-convergence state mutex was poisoned"))?;
            match std::mem::replace(&mut *state, WriteConvergenceState::NotStarted) {
                WriteConvergenceState::Running(runner) => {
                    *state = WriteConvergenceState::Running(runner);
                    return match &*state {
                        WriteConvergenceState::Running(runner) => check(runner),
                        _ => unreachable!("write-convergence state must remain running"),
                    };
                }
                WriteConvergenceState::Failed(message) => {
                    *state = WriteConvergenceState::Failed(message.clone());
                    return Err(HarnessError::message(message.clone()));
                }
                WriteConvergenceState::NotStarted => {
                    *state = WriteConvergenceState::NotStarted;
                    return Err(HarnessError::message(
                        "write-convergence invariant was not started during harness bootstrap",
                    ));
                }
                WriteConvergenceState::Starting { started_at, task } => (started_at, task),
            }
        };

        let (settled_state, elapsed) =
            wait_for_write_convergence_attachment(pending_started_at, pending_task);

        let mut state = self
            .write_convergence_state
            .lock()
            .map_err(|_| HarnessError::message("write-convergence state mutex was poisoned"))?;
        let startup_detail = match &settled_state {
            WriteConvergenceState::Running(_) => {
                format!("accepted-write convergence attached after {:?}", elapsed)
            }
            WriteConvergenceState::Failed(message) => format!(
                "accepted-write convergence attachment failed after {:?}: {message}",
                elapsed
            ),
            WriteConvergenceState::NotStarted | WriteConvergenceState::Starting { .. } => {
                unreachable!("write-convergence attachment must settle into a final state")
            }
        };
        self.record_startup_phase(StartupPhase::InvariantWriteConvergence, startup_detail)?;
        *state = settled_state;

        match &*state {
            WriteConvergenceState::Running(runner) => check(runner),
            WriteConvergenceState::Failed(message) => Err(HarnessError::message(message.clone())),
            WriteConvergenceState::NotStarted | WriteConvergenceState::Starting { .. } => {
                Err(HarnessError::message(
                    "write-convergence invariant startup did not settle into a running state",
                ))
            }
        }
    }

    fn record_startup_phase(&self, phase: StartupPhase, detail: impl Into<String>) -> Result<()> {
        let detail = detail.into();
        eprintln!("[ha][{}][{}] {}", self.run_id, phase.as_str(), detail);
        self.record_note(phase.as_str(), detail)
    }

    fn run_container_action(
        &self,
        service_name: &str,
        phase: &str,
        detail: impl Into<String>,
        action: fn(&DockerCli, &str) -> Result<()>,
    ) -> Result<()> {
        let container_id = self.service_container_id(service_name)?;
        self.record_note(phase, detail)?;
        action(&self.docker, container_id.as_str())
    }

    pub fn record_note(&self, phase: &str, detail: impl Into<String>) -> Result<()> {
        self.push_timeline_entry(serde_json::json!({
            "kind": "note",
            "phase": phase,
            "detail": detail.into(),
            "timestamp_ms": timestamp_millis()?,
        }))
    }

    pub fn record_status_snapshot(&self, phase: &str, status: &NodeState) -> Result<()> {
        self.push_timeline_entry(serde_json::json!({
            "kind": "status",
            "phase": phase,
            "timestamp_ms": timestamp_millis()?,
            "status": status,
        }))
    }

    pub fn service_container_id(&self, service_name: &str) -> Result<String> {
        if let Some(container_id) = self.cached_service_container_id(service_name)? {
            return Ok(container_id);
        }
        self.refresh_service_container_ids()?;
        self.cached_service_container_id(service_name)?
            .ok_or_else(|| {
                HarnessError::message(format!(
                    "docker compose service `{service_name}` has no container in project `{}`",
                    self.compose_project
                ))
            })
    }

    fn run_shell_as_root(&self, service_name: &str, script: &str) -> Result<String> {
        let container_id = self.service_container_id(service_name)?;
        self.docker.exec_as_user(
            container_id.as_str(),
            "root",
            Path::new("/bin/sh"),
            &["-lc", script],
        )
    }

    fn ensure_fault_plumbing(&self, service_name: &str) -> Result<()> {
        let script = ensure_fault_plumbing_script();
        let _ = self.run_shell_as_root(service_name, script.as_str())?;
        self.record_note("fault.ensure_plumbing", format!("service={service_name}"))?;
        Ok(())
    }

    fn clear_network_faults(&self, service_name: &str) -> Result<()> {
        if !self.service_is_running(service_name)? {
            self.record_note(
                "fault.clear_network",
                format!("service={service_name} skipped=container_not_running"),
            )?;
            return Ok(());
        }
        let script = clear_fault_rules_script();
        if let Err(err) = self.run_shell_as_root(service_name, script.as_str()) {
            if container_not_running_error(&err) {
                self.record_note(
                    "fault.clear_network",
                    format!("service={service_name} skipped=container_not_running"),
                )?;
                return Ok(());
            }
            return Err(err);
        }
        self.record_note("fault.clear_network", format!("service={service_name}"))?;
        Ok(())
    }

    pub fn heal_member_network_faults(&self, member: ClusterMember) -> Result<()> {
        self.clear_network_faults(member.service_name())?;
        for peer in DATABASE_MEMBERS {
            if peer == member {
                continue;
            }
            for path in [TrafficPath::Postgres, TrafficPath::Api, TrafficPath::Dcs] {
                self.unblock_member_path_to_host(peer, path, member.service_name())?;
            }
        }
        self.record_note("fault.heal_member_network", format!("member={member}"))?;
        Ok(())
    }

    pub fn block_member_path_to_host(
        &self,
        member: ClusterMember,
        path: TrafficPath,
        peer_service_name: &str,
    ) -> Result<()> {
        let peer_ip = self.service_ipv4_address(peer_service_name)?;
        self.block_member_path_to_address(member, path, peer_ip.as_str(), peer_service_name)
    }

    pub fn unblock_member_path_to_host(
        &self,
        member: ClusterMember,
        path: TrafficPath,
        peer_service_name: &str,
    ) -> Result<()> {
        if !self.service_is_running(member.service_name())? {
            self.record_note(
                "fault.unblock_path",
                format!(
                    "member={member} path={} peer={peer_service_name} skipped=container_not_running",
                    path.label()
                ),
            )?;
            return Ok(());
        }
        let peer_ip = self.service_ipv4_address(peer_service_name)?;
        let script = remove_fault_rule_script(peer_ip.as_str(), path.port());
        let _ = self.run_shell_as_root(member.service_name(), script.as_str())?;
        self.record_note(
            "fault.unblock_path",
            format!(
                "member={member} path={} peer={peer_service_name}",
                path.label()
            ),
        )?;
        Ok(())
    }

    fn block_member_path_to_address(
        &self,
        member: ClusterMember,
        path: TrafficPath,
        peer_ip: &str,
        peer_label: &str,
    ) -> Result<()> {
        self.ensure_fault_plumbing(member.service_name())?;
        let script = append_fault_rule_script(peer_ip, path.port());
        let _ = self.run_shell_as_root(member.service_name(), script.as_str())?;
        self.record_note(
            "fault.block_path",
            format!("member={member} path={} peer={peer_label}", path.label()),
        )?;
        Ok(())
    }

    pub fn isolate_member_from_all_peers_on_path(
        &self,
        member: ClusterMember,
        path: TrafficPath,
    ) -> Result<()> {
        DATABASE_MEMBERS
            .into_iter()
            .filter(|peer| *peer != member)
            .try_for_each(|peer| {
                self.block_member_path_to_host(member, path, peer.service_name())?;
                self.block_member_path_to_host(peer, path, member.service_name())
            })
    }

    pub fn stop_dcs_services(&self, services: impl IntoIterator<Item = DcsService>) -> Result<()> {
        services.into_iter().try_for_each(|service| {
            self.run_container_action(
                service.service_name(),
                "docker.stop_service",
                format!("stopping `{service}`"),
                DockerCli::kill_container,
            )
        })
    }

    pub fn start_dcs_services(&self, services: impl IntoIterator<Item = DcsService>) -> Result<()> {
        services.into_iter().try_for_each(|service| {
            self.run_container_action(
                service.service_name(),
                "docker.start_service",
                format!("starting `{service}`"),
                DockerCli::start_container,
            )
        })
    }

    pub fn set_blocker(
        &self,
        member: ClusterMember,
        blocker: BlockerKind,
        enabled: bool,
    ) -> Result<()> {
        if enabled {
            write_fault_marker(
                self.materialized_dir.as_path(),
                member,
                blocker.marker_path(),
            )?;
            remove_fault_marker(
                self.materialized_dir.as_path(),
                member,
                blocker.clear_on_start_marker_path(),
            )?;
        } else {
            remove_fault_marker(
                self.materialized_dir.as_path(),
                member,
                blocker.marker_path(),
            )?;
            remove_fault_marker(
                self.materialized_dir.as_path(),
                member,
                blocker.clear_on_start_marker_path(),
            )?;
        }
        self.record_note(
            "fault.blocker",
            format!(
                "member={member} blocker={} enabled={enabled}",
                blocker.label()
            ),
        )?;
        Ok(())
    }

    pub fn wipe_member_data_dir(&self, member: ClusterMember) -> Result<()> {
        write_fault_marker(
            self.materialized_dir.as_path(),
            member,
            WIPE_DATA_ON_START_MARKER_PATH,
        )?;
        self.record_note("fault.wipe_data_dir", format!("member={member}"))?;
        Ok(())
    }

    fn service_is_running(&self, service_name: &str) -> Result<bool> {
        let container_id = self.service_container_id(service_name)?;
        Ok(self.docker.container_state_status(container_id.as_str())? == "running")
    }

    fn service_ipv4_address(&self, service_name: &str) -> Result<String> {
        let container_id = self.service_container_id(service_name)?;
        self.docker.container_ipv4_address(container_id.as_str())
    }

    fn member_network_gateway_ipv4(&self, member: ClusterMember) -> Result<String> {
        let container_id = self.service_container_id(member.service_name())?;
        self.docker.container_network_gateway(container_id.as_str())
    }

    async fn bootstrap_cluster(&self) -> Result<()> {
        self.record_startup_phase(
            StartupPhase::DcsReady,
            "waiting for DCS services to report healthy",
        )?;
        for service in self.given.dcs_services() {
            self.wait_for_service_health(service.service_name()).await?;
        }
        self.record_startup_phase(StartupPhase::DcsReady, "all DCS services are healthy")?;
        self.record_startup_phase(
            StartupPhase::SeedPrimaryReady,
            "starting seed primary node-b",
        )?;
        self.docker.compose_up_services(
            self.compose_file.as_path(),
            self.compose_project.as_str(),
            &["node-b"],
        )?;
        self.wait_for_seed_primary().await?;
        self.record_startup_phase(
            StartupPhase::SeedPrimaryReady,
            "seed primary node-b is authoritative",
        )?;
        self.record_startup_phase(
            StartupPhase::ClusterReady,
            "starting remaining nodes node-a and node-c",
        )?;
        self.docker.compose_up_services(
            self.compose_file.as_path(),
            self.compose_project.as_str(),
            &["node-a", "node-c"],
        )?;
        self.record_startup_phase(
            StartupPhase::ClusterReady,
            "cluster bootstrapped enough for scenario execution",
        )
    }

    async fn wait_for_service_health(&self, service_name: &str) -> Result<()> {
        let deadline = Instant::now() + self.timeouts.startup_deadline;
        let mut last_error = None;
        while Instant::now() < deadline {
            let result = match self
                .service_container_id(service_name)
                .and_then(|container_id| self.docker.container_health_status(container_id.as_str()))
            {
                Ok(Some(status)) if status == "healthy" => Ok(()),
                Ok(Some(status)) => Err(HarnessError::message(format!(
                    "service `{service_name}` health is `{status}`"
                ))),
                Ok(None) => Err(HarnessError::message(format!(
                    "service `{service_name}` does not expose a docker health status"
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
            "timed out waiting for service `{service_name}` to become healthy; last observed error: {}",
            last_error.unwrap_or_else(|| "no health state was observed".to_string())
        )))
    }

    async fn wait_for_seed_primary(&self) -> Result<()> {
        let deadline = Instant::now() + self.timeouts.startup_deadline;
        let mut last_error = None;
        while Instant::now() < deadline {
            let result = match self.observer.state_via_member(ClusterMember::SEED_PRIMARY) {
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
        let compose_network = format!("{}_ha", self.compose_project);
        let invariant_result = self.ensure_accepted_writes_running();
        if let Err(err) = &invariant_result {
            failures.push(format!("background invariant check failed: {err}"));
        }
        let capture_result = self.capture_artifacts();
        if let Err(err) = &capture_result {
            failures.push(format!("artifact capture failed: {err}"));
        }
        let compose_result = self
            .docker
            .compose_down(self.compose_file.as_path(), self.compose_project.as_str());
        if let Err(err) = &compose_result {
            failures.push(format!("docker compose down failed: {err}"));
        }
        let network_result = if compose_result.is_ok() {
            self.docker
                .wait_for_network_absent(compose_network.as_str())
        } else {
            Ok(())
        };
        if let Err(err) = &network_result {
            failures.push(format!("compose network cleanup failed: {err}"));
        }
        let ryuk_result = self.ryuk.as_mut().map(RyukGuard::close).transpose();
        if let Err(err) = &ryuk_result {
            failures.push(format!("ryuk cleanup failed: {err}"));
        }
        if compose_result.is_ok() && network_result.is_ok() && ryuk_result.is_ok() {
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
                "given_name": self.given.as_str(),
                "run_id": self.run_id,
                "run_dir": self.run_dir,
                "materialized_dir": self.materialized_dir,
                "artifacts_dir": self.artifacts_dir,
                "compose_project": self.compose_project,
                "cucumber_test_image_run_id": self.cucumber_test_image_run_id,
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
        match self.observer.observe_states() {
            Ok(states) => {
                let serialized = states
                    .iter()
                    .map(|(member, outcome)| match outcome {
                        Ok(state) => serde_json::json!({
                            "member": member.service_name(),
                            "state": state,
                        }),
                        Err(message) => serde_json::json!({
                            "member": member.service_name(),
                            "failure": message,
                        }),
                    })
                    .collect::<Vec<_>>();
                write_text_file(
                    self.artifacts_dir.join("operator-state.json").as_path(),
                    serde_json::to_string_pretty(&serialized)
                        .map_err(|source| HarnessError::Json {
                            context: "serializing operator state payload".to_string(),
                            source,
                        })?
                        .as_str(),
                )?
            }
            Err(err) => failures.push(format!("operator state capture failed: {err}")),
        }

        for service_name in self.given.artifact_service_names() {
            match self.service_container_id(service_name) {
                Ok(container_id) => match self.docker.inspect_container(container_id.as_str()) {
                    Ok(inspect) => {
                        let artifact = self
                            .artifacts_dir
                            .join(format!("inspect-{service_name}.json"));
                        write_text_file(artifact.as_path(), inspect.as_str())?;
                    }
                    Err(err) => failures.push(format!(
                        "docker inspect artifact capture failed for `{service_name}`: {err}"
                    )),
                },
                Err(err) => failures.push(format!(
                    "container resolution failed for artifact capture `{service_name}`: {err}"
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

    fn cached_service_container_id(&self, service_name: &str) -> Result<Option<String>> {
        self.service_container_ids
            .lock()
            .map(|cache| cache.get(service_name).cloned())
            .map_err(|_| HarnessError::message("service container cache mutex was poisoned"))
    }

    fn refresh_service_container_ids(&self) -> Result<()> {
        let artifact_service_names = self.given.artifact_service_names();
        let compose_entries = self
            .docker
            .compose_ps_entries(self.compose_file.as_path(), self.compose_project.as_str())?;
        let mut cache = self
            .service_container_ids
            .lock()
            .map_err(|_| HarnessError::message("service container cache mutex was poisoned"))?;
        compose_entries
            .into_iter()
            .filter(|entry| artifact_service_names.contains(&entry.service.as_str()))
            .for_each(|entry| {
                let _ = cache.insert(entry.service, entry.id);
            });
        Ok(())
    }

    fn start_write_convergence_invariant(&self) -> Result<()> {
        let observer = self.observer.clone();
        let poll_interval = self.timeouts.poll_interval;
        let write_deadline = self.timeouts.write_convergence_deadline;
        let mut state = self
            .write_convergence_state
            .lock()
            .map_err(|_| HarnessError::message("write-convergence state mutex was poisoned"))?;
        *state = WriteConvergenceState::Starting {
            started_at: Instant::now(),
            task: Some(tokio::spawn(async move {
                WriteConvergenceInvariantRunner::start(observer, poll_interval, write_deadline)
                    .await
            })),
        };
        self.record_startup_phase(
            StartupPhase::InvariantWriteConvergence,
            "started background accepted-write convergence attachment",
        )
    }
}

impl StartupPhase {
    fn as_str(self) -> &'static str {
        match self {
            Self::WorkspaceReady => "startup.workspace_ready",
            Self::InvariantPrimaryCount => "startup.invariant.primary_count",
            Self::DcsReady => "startup.dcs_ready",
            Self::SeedPrimaryReady => "startup.seed_primary_ready",
            Self::ClusterReady => "startup.cluster_ready",
            Self::InvariantWriteConvergence => "startup.invariant.write_convergence",
        }
    }
}

fn wait_for_write_convergence_attachment(
    started_at: Instant,
    maybe_task: Option<
        JoinHandle<
            std::result::Result<WriteConvergenceInvariantRunner, WriteConvergenceInvariantError>,
        >,
    >,
) -> (WriteConvergenceState, std::time::Duration) {
    let elapsed = started_at.elapsed();
    let settled_state = match maybe_task
        .ok_or_else(|| {
            HarnessError::message("write-convergence invariant attachment task was missing")
        })
        .and_then(|task| {
            block_on_harness_future(
                async move {
                    task.await
                        .map_err(|err| {
                            HarnessError::message(format!(
                                "write-convergence invariant attachment task failed to join: {err}"
                            ))
                        })?
                        .map_err(HarnessError::from)
                },
                "write-convergence invariant attachment",
            )
        }) {
        Ok(runner) => WriteConvergenceState::Running(runner),
        Err(err) => WriteConvergenceState::Failed(err.to_string()),
    };
    (settled_state, elapsed)
}

fn block_on_harness_future<T>(
    future: impl Future<Output = Result<T>> + Send + 'static,
    context: &'static str,
) -> Result<T>
where
    T: Send + 'static,
{
    match Handle::try_current() {
        Ok(handle) if handle.runtime_flavor() == RuntimeFlavor::MultiThread => {
            tokio::task::block_in_place(|| handle.block_on(future))
        }
        Ok(_) | Err(_) => thread::spawn(move || {
            Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|err| {
                    HarnessError::message(format!("build runtime for {context} failed: {err}"))
                })?
                .block_on(future)
        })
        .join()
        .map_err(|_| HarnessError::message(format!("{context} thread panicked")))?,
    }
}

fn container_not_running_error(err: &HarnessError) -> bool {
    matches!(err, HarnessError::CommandFailed { stderr, .. } if stderr.contains("is not running"))
}

fn build_run_id(feature_name: &str, scenario_name: &str) -> Result<String> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| HarnessError::message(format!("system clock error: {err}")))?;
    Ok(format!(
        "{}-{}-{}-{}",
        sanitize(feature_name),
        sanitize(scenario_name),
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

fn required_env(key: &str) -> Result<String> {
    std::env::var(key).map_err(|err| {
        HarnessError::message(format!(
            "required environment variable `{key}` is missing: {err}"
        ))
    })
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

fn create_fault_directories(root: &Path) -> Result<()> {
    let faults_root = root.join("faults");
    create_dir_all(faults_root.as_path())?;
    for member in ClusterMember::ALL {
        create_dir_all(faults_root.join(member.service_name()).as_path())?;
    }
    Ok(())
}

fn validate_seed_primary(status: &NodeState) -> Result<()> {
    let discovered_member_count = status.dcs.member_count();
    if discovered_member_count != 1 {
        return Err(HarnessError::message(format!(
            "expected exactly one discovered member during bootstrap, observed {}; warnings={}",
            discovered_member_count,
            format_bootstrap_warnings(status),
        )));
    }

    match operator_visible_primary(status).as_deref() {
        Some("node-b") => Ok(()),
        Some(primary) => Err(HarnessError::message(format!(
            "expected node-b to bootstrap as the seed primary, observed `{primary}`"
        ))),
        None => Err(HarnessError::message(format!(
            "bootstrap state has no authoritative primary; warnings={}",
            format_bootstrap_warnings(status),
        ))),
    }
}

fn format_bootstrap_warnings(status: &NodeState) -> String {
    let mut warnings = Vec::new();
    if authoritative_primary_member(status).is_none() {
        warnings.push("no_primary".to_string());
    }
    if status.dcs.member_count() == 0 {
        warnings.push("no_members".to_string());
    }
    if warnings.is_empty() {
        "none".to_string()
    } else {
        warnings.join(" | ")
    }
}

fn operator_visible_primary(status: &NodeState) -> Option<String> {
    authoritative_primary_member(status).map(|member_id| member_id.0.clone())
}

#[cfg(test)]
mod tests {
    use std::{sync::Mutex, time::Duration};

    use super::*;
    use crate::support::{
        docker::cli::DockerCli, invariants::WriteConvergenceInvariantError, timeouts::TimeoutModel,
    };

    #[test]
    fn member_alias_round_trips() -> Result<()> {
        let mut world = HaWorld::default();
        world.remember_member_alias("replica", ClusterMember::NodeB);

        assert_eq!(world.require_member_alias("replica")?, ClusterMember::NodeB);
        Ok(())
    }

    #[test]
    fn member_reference_resolves_alias_before_service_name() -> Result<()> {
        let mut world = HaWorld::default();
        world.remember_member_alias("candidate", ClusterMember::NodeC);

        assert_eq!(
            world.resolve_member_reference("candidate")?,
            ClusterMember::NodeC
        );
        assert_eq!(
            world.resolve_member_reference("node-a")?,
            ClusterMember::NodeA
        );
        Ok(())
    }

    #[test]
    fn missing_alias_reports_error() -> Result<()> {
        let world = HaWorld::default();
        match world.require_member_alias("missing") {
            Ok(member) => Err(HarnessError::message(format!(
                "unexpectedly resolved missing alias to `{member}`"
            ))),
            Err(err) => {
                assert!(err.to_string().contains("missing"));
                Ok(())
            }
        }
    }

    fn test_harness_with_write_convergence(
        write_convergence: WriteConvergenceState,
    ) -> Result<HarnessShared> {
        let docker = DockerCli::fake_for_tests();
        let compose_file =
            PathBuf::from("tests/ha/runs/test-feature/test-run/materialized/compose.yml");
        let compose_project = "ha-test-run".to_string();
        let materialized_dir = PathBuf::from("tests/ha/runs/test-feature/test-run/materialized");
        let observer = PgtmObserver::new(
            docker.clone(),
            compose_file.clone(),
            compose_project.clone(),
            materialized_dir.clone(),
        );
        Ok(HarnessShared {
            run_id: "test-run".to_string(),
            feature_name: "test-feature".to_string(),
            given: HaGivenId::Plain,
            run_dir: PathBuf::from("tests/ha/runs/test-feature/test-run"),
            materialized_dir,
            artifacts_dir: PathBuf::from("tests/ha/runs/test-feature/test-run/artifacts"),
            compose_file,
            compose_project,
            cucumber_test_image_run_id: "cucumber-test-run".to_string(),
            docker,
            observer,
            ryuk: None,
            timeouts: TimeoutModel {
                poll_interval: Duration::from_millis(1),
                startup_deadline: Duration::from_millis(10),
                failover_deadline: Duration::from_millis(10),
                recovery_deadline: Duration::from_millis(10),
                write_convergence_deadline: Duration::from_millis(10),
            },
            service_container_ids: Mutex::new(BTreeMap::new()),
            timeline: Mutex::new(Vec::new()),
            cleaned_up: false,
            primary_count_invariant: PrimaryCountInvariantRunner::healthy_for_tests(),
            write_convergence_state: Mutex::new(write_convergence),
        })
    }

    fn delayed_failed_attachment(message: &str) -> WriteConvergenceState {
        let message = message.to_string();
        WriteConvergenceState::Starting {
            started_at: Instant::now(),
            task: Some(tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(20)).await;
                Err(WriteConvergenceInvariantError::Failed(message))
            })),
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn background_health_check_allows_pending_write_convergence_attachment() -> Result<()> {
        let harness = test_harness_with_write_convergence(delayed_failed_attachment("pending"))?;
        harness.ensure_primary_count_healthy()
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn cleanup_liveness_check_waits_for_write_convergence_attachment_failure() -> Result<()> {
        let harness = test_harness_with_write_convergence(delayed_failed_attachment("boom"))?;
        let err = match harness.ensure_accepted_writes_running() {
            Ok(()) => {
                return Err(HarnessError::message(
                    "required invariant readiness unexpectedly succeeded",
                ));
            }
            Err(err) => err,
        };
        assert!(err.to_string().contains("boom"));
        let write_convergence_state = harness
            .write_convergence_state
            .lock()
            .map_err(|_| HarnessError::message("write-convergence state mutex was poisoned"))?;
        match &*write_convergence_state {
            WriteConvergenceState::Failed(message) => {
                assert!(message.contains("boom"));
            }
            other => {
                return Err(HarnessError::message(format!(
                    "expected failed write-convergence state after cleanup check, observed {other:?}"
                )));
            }
        }
        Ok(())
    }
}
