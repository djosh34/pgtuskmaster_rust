use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    future::Future,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    sync::Mutex,
    thread,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use cucumber::World;
use pgtuskmaster_rust::{
    api::NodeState,
    ha::types::{AuthorityProjection, PublicationState},
};
use tokio::{
    runtime::{Builder, Handle, RuntimeFlavor},
    task::JoinHandle,
};

use crate::support::{
    docker::{cli::DockerCli, ryuk::RyukGuard},
    error::{HarnessError, Result},
    faults::{
        append_fault_rule_script, clear_fault_rules_script, ensure_fault_plumbing_script,
        remove_fault_rule_script, BlockerKind, TrafficPath, DATABASE_MEMBERS, FAULT_DIR,
    },
    feature_metadata,
    givens::{
        resolve_given, ComposeVariant, FixtureMaterialization, HaGivenDefinition, HaGivenId,
        MemberRuntimeConfigMaterialization, NodeRuntimeTemplate, SharedFixtureEntry,
    },
    invariants::{
        PrimaryCountInvariantRunner, WriteConvergenceInvariantError,
        WriteConvergenceInvariantRunner,
    },
    observer::{
        pgtm::{MemberCommandOutcome, PgtmObserver},
        sql::SqlObserver,
    },
    timeouts::TimeoutModel,
    topology::{ClusterMember, ComposeService},
};

#[derive(Debug, Default, World)]
pub struct HaWorld {
    pub harness: Option<HarnessShared>,
    pub scenario: ScenarioState,
}

#[derive(Debug, Default)]
pub struct ScenarioState {
    pub name: Option<String>,
    pub aliases: AliasRegistry,
    pub availability: ScenarioAvailability,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AliasName(String);

impl From<&str> for AliasName {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for AliasName {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MemberSet {
    members: BTreeSet<ClusterMember>,
}

impl MemberSet {
    pub fn insert(&mut self, member: ClusterMember) -> bool {
        self.members.insert(member)
    }

    pub fn remove(&mut self, member: ClusterMember) -> bool {
        self.members.remove(&member)
    }

    pub fn contains(&self, member: ClusterMember) -> bool {
        self.members.contains(&member)
    }

    pub fn clear(&mut self) {
        self.members.clear();
    }
}

#[derive(Debug, Default)]
pub struct AliasRegistry {
    pub aliases_by_name: BTreeMap<AliasName, ClusterMember>,
}

#[derive(Debug, Default)]
pub struct ScenarioAvailability {
    pub stopped_members: MemberSet,
    pub observer_unreachable_members: MemberSet,
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

    pub fn set_scenario_name(&mut self, scenario_name: String) {
        self.scenario.name = Some(scenario_name);
    }

    pub fn scenario_name(&self) -> Result<&str> {
        self.scenario
            .name
            .as_deref()
            .ok_or_else(|| HarnessError::message("scenario name has not been initialized"))
    }

    pub fn set_harness(&mut self, harness: HarnessShared) {
        self.harness = Some(harness);
    }

    pub fn remember_member_alias(&mut self, alias: impl Into<AliasName>, member: ClusterMember) {
        self.scenario
            .aliases
            .aliases_by_name
            .insert(alias.into(), member);
    }

    pub fn require_member_alias(&self, alias: &str) -> Result<ClusterMember> {
        self.scenario
            .aliases
            .aliases_by_name
            .get(&AliasName::from(alias))
            .copied()
            .ok_or_else(|| HarnessError::message(format!("alias `{alias}` was not recorded")))
    }

    pub fn add_stopped_node(&mut self, member: ClusterMember) {
        let _ = self.scenario.availability.stopped_members.insert(member);
    }

    pub fn remove_stopped_node(&mut self, member: ClusterMember) {
        let _ = self.scenario.availability.stopped_members.remove(member);
    }

    pub fn mark_observer_unreachable(&mut self, member: ClusterMember) {
        let _ = self
            .scenario
            .availability
            .observer_unreachable_members
            .insert(member);
    }

    pub fn clear_observer_unreachable(&mut self, member: ClusterMember) {
        let _ = self
            .scenario
            .availability
            .observer_unreachable_members
            .remove(member);
    }

    pub fn clear_observer_unreachable_members(&mut self) {
        self.scenario
            .availability
            .observer_unreachable_members
            .clear();
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
pub struct HarnessWorkspace {
    pub run_id: String,
    pub feature_name: String,
    pub given: HaGivenDefinition,
    pub paths: WorkspacePaths,
}

#[derive(Debug)]
pub struct WorkspacePaths {
    pub run_dir: PathBuf,
    pub materialized_dir: PathBuf,
    pub artifacts_dir: PathBuf,
}

#[derive(Debug)]
pub struct ComposeStack {
    pub file: PathBuf,
    pub project: String,
}

#[derive(Debug)]
pub struct HarnessRuntime {
    pub workspace: HarnessWorkspace,
    pub compose: ComposeStack,
    pub cucumber_test_image_run_id: String,
    pub docker: DockerCli,
    pub ryuk: Option<RyukGuard>,
    pub timeouts: TimeoutModel,
    service_container_ids: Mutex<BTreeMap<ComposeService, String>>,
    timeline: Mutex<Vec<serde_json::Value>>,
    cleaned_up: bool,
}

#[derive(Debug)]
struct BackgroundInvariants {
    primary_count: PrimaryCountInvariantRunner,
    write_convergence: Mutex<WriteConvergenceActivation>,
}

#[derive(Debug)]
enum WriteConvergenceActivation {
    Dormant,
    Armed,
    Activating(WriteConvergenceInvariantAttachment),
    Active(WriteConvergenceInvariantRunner),
    Failed(String),
    Joining,
}

#[derive(Debug)]
struct WriteConvergenceInvariantAttachment {
    started_at: Instant,
    task: Option<JoinHandle<std::result::Result<
        WriteConvergenceInvariantRunner,
        WriteConvergenceInvariantError,
    >>>,
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

#[derive(Clone, Copy, Debug)]
enum InvariantRequirement {
    ClusterReady,
    AcceptedWritesReady,
}

#[derive(Debug)]
pub struct HarnessShared {
    runtime: HarnessRuntime,
    background_invariants: BackgroundInvariants,
}

impl Deref for HarnessShared {
    type Target = HarnessRuntime;

    fn deref(&self) -> &Self::Target {
        &self.runtime
    }
}

impl DerefMut for HarnessShared {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.runtime
    }
}

impl HarnessShared {
    pub async fn initialize(given: HaGivenId, scenario_name: &str) -> Result<Self> {
        let feature = feature_metadata()?;
        let docker = DockerCli::discover()?;
        docker.verify_daemon()?;

        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let given = resolve_given(repo_root.as_path(), given)?;
        let run_id = build_run_id(feature.feature_name.as_str(), scenario_name)?;
        let compose = ComposeStack {
            file: PathBuf::new(),
            project: build_compose_project(feature.feature_name.as_str(), run_id.as_str()),
        };
        let cucumber_test_image_run_id = required_env("PGTM_CUCUMBER_TEST_RUN_ID")?;
        let paths = WorkspacePaths {
            run_dir: repo_root
                .join("tests/ha/runs")
                .join(feature.feature_name.as_str())
                .join(run_id.as_str()),
            materialized_dir: repo_root
                .join("tests/ha/runs")
                .join(feature.feature_name.as_str())
                .join(run_id.as_str())
                .join("materialized"),
            artifacts_dir: repo_root
                .join("tests/ha/runs")
                .join(feature.feature_name.as_str())
                .join(run_id.as_str())
                .join("artifacts"),
        };
        create_dir_all(paths.run_dir.as_path())?;
        create_dir_all(paths.materialized_dir.as_path())?;
        create_dir_all(paths.artifacts_dir.as_path())?;
        materialize_given_fixture(&given, paths.materialized_dir.as_path())?;
        create_fault_directories(paths.materialized_dir.as_path())?;

        let compose = ComposeStack {
            file: paths.materialized_dir.join("compose.yml"),
            project: compose.project,
        };
        let timeouts = TimeoutModel::from_runtime_config(
            paths
                .materialized_dir
                .join(ClusterMember::SEED_PRIMARY.runtime_config_relative_path())
                .as_path(),
        )?;
        let ryuk = RyukGuard::start(docker.clone(), compose.project.as_str())?;
        let dcs_service_names = given
            .dcs_services()
            .into_iter()
            .map(|service| service.service_name())
            .collect::<Vec<_>>();
        docker.compose_up_services(
            compose.file.as_path(),
            compose.project.as_str(),
            dcs_service_names.as_slice(),
        )?;
        let service_container_ids = Mutex::new(BTreeMap::new());
        let runtime = HarnessRuntime {
            workspace: HarnessWorkspace {
                run_id,
                feature_name: feature.feature_name.clone(),
                given,
                paths,
            },
            compose,
            cucumber_test_image_run_id,
            docker,
            ryuk: Some(ryuk),
            timeouts,
            service_container_ids,
            timeline: Mutex::new(Vec::new()),
            cleaned_up: false,
        };
        let background_invariants = BackgroundInvariants::start_primary_count(
            runtime.observer(),
            runtime.timeouts.poll_interval,
            runtime.timeouts.failover_deadline,
        )
        .await?;
        let mut harness = Self {
            runtime,
            background_invariants,
        };
        harness.record_startup_phase(
            StartupPhase::WorkspaceReady,
            format!(
                "materialized fixture `{}` and started DCS services",
                harness.given_name()
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
        if let Err(err) = harness.start_write_convergence_invariant_attachment() {
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

    pub fn feature_name(&self) -> &str {
        self.workspace.feature_name.as_str()
    }

    pub fn run_id(&self) -> &str {
        self.workspace.run_id.as_str()
    }

    pub fn given_name(&self) -> &str {
        self.workspace.given.id.as_str()
    }

    pub fn compose_file(&self) -> &Path {
        self.compose.file.as_path()
    }

    pub fn compose_project(&self) -> &str {
        self.compose.project.as_str()
    }

    pub fn run_dir(&self) -> &Path {
        self.workspace.paths.run_dir.as_path()
    }

    pub fn materialized_dir(&self) -> &Path {
        self.workspace.paths.materialized_dir.as_path()
    }

    pub fn artifacts_dir(&self) -> &Path {
        self.workspace.paths.artifacts_dir.as_path()
    }

    pub fn observer(&self) -> PgtmObserver {
        PgtmObserver::new(
            self.docker.clone(),
            self.compose.file.clone(),
            self.compose.project.clone(),
            self.materialized_dir().to_path_buf(),
        )
    }

    pub fn ensure_primary_count_healthy(&self) -> Result<()> {
        self.background_invariants
            .ensure_healthy(self, InvariantRequirement::ClusterReady)
    }

    pub fn ensure_accepted_writes_ready(&self) -> Result<()> {
        self.background_invariants
            .ensure_healthy(self, InvariantRequirement::AcceptedWritesReady)
    }

    fn record_startup_phase(&self, phase: StartupPhase, detail: impl Into<String>) -> Result<()> {
        let detail = detail.into();
        eprintln!("[ha][{}][{}] {}", self.run_id(), phase.as_str(), detail);
        self.record_note(phase.as_str(), detail)
    }

    pub fn sql(&self) -> SqlObserver {
        SqlObserver::new(self.materialized_dir().to_path_buf())
    }

    pub fn kill_node(&self, member: ClusterMember) -> Result<()> {
        let container_id = self.service_container_id(member.into())?;
        self.record_note("docker.kill", format!("killing `{member}`"))?;
        self.docker.kill_container(container_id.as_str())
    }

    pub fn start_node(&self, member: ClusterMember) -> Result<()> {
        let container_id = self.service_container_id(member.into())?;
        self.record_note("docker.start", format!("starting `{member}`"))?;
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

    pub fn record_status_snapshot(&self, phase: &str, status: &NodeState) -> Result<()> {
        self.push_timeline_entry(serde_json::json!({
            "kind": "status",
            "phase": phase,
            "timestamp_ms": timestamp_millis()?,
            "status": status,
        }))
    }

    pub fn service_container_id(&self, service: ComposeService) -> Result<String> {
        if let Some(container_id) = self.cached_service_container_id(service)? {
            return Ok(container_id);
        }
        self.refresh_service_container_ids()?;
        self.cached_service_container_id(service)?.ok_or_else(|| {
            HarnessError::message(format!(
                "docker compose service `{service}` has no container in project `{}`",
                self.compose_project()
            ))
        })
    }

    pub fn stop_service(&self, service: ComposeService) -> Result<()> {
        let container_id = self.service_container_id(service)?;
        self.record_note("docker.stop_service", format!("stopping `{service}`"))?;
        self.docker.kill_container(container_id.as_str())
    }

    pub fn start_service(&self, service: ComposeService) -> Result<()> {
        let container_id = self.service_container_id(service)?;
        self.record_note("docker.start_service", format!("starting `{service}`"))?;
        self.docker.start_container(container_id.as_str())
    }

    pub fn run_shell_as_root(&self, service: ComposeService, script: &str) -> Result<String> {
        let container_id = self.service_container_id(service)?;
        self.docker.exec_as_user(
            container_id.as_str(),
            "root",
            Path::new("/bin/sh"),
            &["-lc", script],
        )
    }

    pub fn ensure_fault_plumbing(&self, service: ComposeService) -> Result<()> {
        let script = ensure_fault_plumbing_script();
        let _ = self.run_shell_as_root(service, script.as_str())?;
        self.record_note("fault.ensure_plumbing", format!("service={service}"))?;
        Ok(())
    }

    pub fn clear_network_faults(&self, service: ComposeService) -> Result<()> {
        if !self.service_is_running(service)? {
            self.record_note(
                "fault.clear_network",
                format!("service={service} skipped=container_not_running"),
            )?;
            return Ok(());
        }
        let script = clear_fault_rules_script();
        if let Err(err) = self.run_shell_as_root(service, script.as_str()) {
            if container_not_running_error(&err) {
                self.record_note(
                    "fault.clear_network",
                    format!("service={service} skipped=container_not_running"),
                )?;
                return Ok(());
            }
            return Err(err);
        }
        self.record_note("fault.clear_network", format!("service={service}"))?;
        Ok(())
    }

    pub fn heal_member_network_faults(&self, member: ClusterMember) -> Result<()> {
        self.clear_network_faults(member.into())?;
        for peer in DATABASE_MEMBERS {
            if peer == member {
                continue;
            }
            for path in [TrafficPath::Postgres, TrafficPath::Api, TrafficPath::Dcs] {
                self.unblock_member_path_to_host(peer, path, member.into())?;
            }
        }
        self.record_note("fault.heal_member_network", format!("member={member}"))?;
        Ok(())
    }

    pub fn block_member_path_to_host(
        &self,
        member: ClusterMember,
        path: TrafficPath,
        peer_service: ComposeService,
    ) -> Result<()> {
        let peer_container_id = self.service_container_id(peer_service)?;
        let peer_ip = self
            .docker
            .container_ipv4_address(peer_container_id.as_str())?;
        self.block_member_path_to_address(
            member,
            path,
            peer_ip.as_str(),
            peer_service.service_name(),
        )
    }

    pub fn unblock_member_path_to_host(
        &self,
        member: ClusterMember,
        path: TrafficPath,
        peer_service: ComposeService,
    ) -> Result<()> {
        if !self.service_is_running(member.into())? {
            self.record_note(
                "fault.unblock_path",
                format!(
                    "member={member} path={} peer={peer_service} skipped=container_not_running",
                    path.label()
                ),
            )?;
            return Ok(());
        }
        let peer_container_id = self.service_container_id(peer_service)?;
        let peer_ip = self
            .docker
            .container_ipv4_address(peer_container_id.as_str())?;
        let script = remove_fault_rule_script(peer_ip.as_str(), path.port());
        let _ = self.run_shell_as_root(member.into(), script.as_str())?;
        self.record_note(
            "fault.unblock_path",
            format!("member={member} path={} peer={peer_service}", path.label()),
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
        self.ensure_fault_plumbing(member.into())?;
        let script = append_fault_rule_script(peer_ip, path.port());
        let _ = self.run_shell_as_root(member.into(), script.as_str())?;
        self.record_note(
            "fault.block_path",
            format!("member={member} path={} peer={peer_label}", path.label()),
        )?;
        Ok(())
    }

    pub fn isolate_member_from_peer_on_path(
        &self,
        member: ClusterMember,
        peer: ClusterMember,
        path: TrafficPath,
    ) -> Result<()> {
        self.block_member_path_to_host(member, path, peer.into())?;
        self.block_member_path_to_host(peer, path, member.into())
    }

    pub fn isolate_member_from_all_peers_on_path(
        &self,
        member: ClusterMember,
        path: TrafficPath,
    ) -> Result<()> {
        DATABASE_MEMBERS
            .into_iter()
            .filter(|peer| *peer != member)
            .try_for_each(|peer| self.isolate_member_from_peer_on_path(member, peer, path))
    }

    pub fn isolate_member_from_observer_on_api(&self, member: ClusterMember) -> Result<()> {
        let gateway_ip = self.member_network_gateway_ipv4(member)?;
        self.block_member_path_to_address(
            member,
            TrafficPath::Api,
            gateway_ip.as_str(),
            "host-operator",
        )
    }

    pub fn cut_member_off_from_dcs(&self, member: ClusterMember) -> Result<()> {
        self.block_member_path_to_host(
            member,
            TrafficPath::Dcs,
            self.workspace.given.local_dcs_service_for(member).into(),
        )
    }

    pub fn stop_all_dcs_services(&self) -> Result<()> {
        self.workspace
            .given
            .dcs_services()
            .into_iter()
            .try_for_each(|service| self.stop_service(service.into()))
    }

    pub fn start_all_dcs_services(&self) -> Result<()> {
        self.workspace
            .given
            .dcs_services()
            .into_iter()
            .try_for_each(|service| self.start_service(service.into()))
    }

    pub fn stop_dcs_quorum_majority(&self) -> Result<()> {
        self.workspace
            .given
            .quorum_majority_dcs_services()
            .into_iter()
            .try_for_each(|service| self.stop_service(service.into()))
    }

    pub fn start_dcs_quorum_majority(&self) -> Result<()> {
        self.workspace
            .given
            .quorum_majority_dcs_services()
            .into_iter()
            .try_for_each(|service| self.start_service(service.into()))
    }

    pub fn stop_member_local_dcs(&self, member: ClusterMember) -> Result<()> {
        self.stop_service(self.workspace.given.local_dcs_service_for(member).into())
    }

    pub fn start_member_local_dcs(&self, member: ClusterMember) -> Result<()> {
        self.start_service(self.workspace.given.local_dcs_service_for(member).into())
    }

    pub fn set_blocker(
        &self,
        member: ClusterMember,
        blocker: BlockerKind,
        enabled: bool,
    ) -> Result<()> {
        if enabled {
            self.write_fault_marker(member, blocker.marker_path())?;
            self.remove_fault_marker(member, blocker.clear_on_start_marker_path())?;
        } else {
            self.remove_fault_marker(member, blocker.marker_path())?;
            self.remove_fault_marker(member, blocker.clear_on_start_marker_path())?;
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
        let marker_path = "/var/lib/pgtuskmaster/faults/wipe-data-on-start";
        self.write_fault_marker(member, marker_path)?;
        self.record_note("fault.wipe_data_dir", format!("member={member}"))?;
        Ok(())
    }

    pub fn clear_all_network_faults(&self) -> Result<()> {
        for service in DATABASE_MEMBERS.into_iter().map(ComposeService::from) {
            self.clear_network_faults(service)?;
        }
        Ok(())
    }

    fn service_is_running(&self, service: ComposeService) -> Result<bool> {
        let container_id = self.service_container_id(service)?;
        Ok(self.docker.container_state_status(container_id.as_str())? == "running")
    }

    fn host_fault_dir(&self, member: ClusterMember) -> PathBuf {
        self.materialized_dir()
            .join("faults")
            .join(member.service_name())
    }

    fn member_network_gateway_ipv4(&self, member: ClusterMember) -> Result<String> {
        let container_id = self.service_container_id(member.into())?;
        self.docker.container_network_gateway(container_id.as_str())
    }

    fn host_fault_marker_path(&self, member: ClusterMember, marker_path: &str) -> Result<PathBuf> {
        let relative_path = Path::new(marker_path)
            .strip_prefix(FAULT_DIR)
            .map_err(|_| {
                HarnessError::message(format!(
                    "fault marker `{marker_path}` does not live under `{FAULT_DIR}`"
                ))
            })?;
        Ok(self.host_fault_dir(member).join(relative_path))
    }

    fn write_fault_marker(&self, member: ClusterMember, marker_path: &str) -> Result<()> {
        let marker_file = self.host_fault_marker_path(member, marker_path)?;
        if let Some(parent) = marker_file.parent() {
            create_dir_all(parent)?;
        }
        write_text_file(marker_file.as_path(), "")?;
        Ok(())
    }

    fn remove_fault_marker(&self, member: ClusterMember, marker_path: &str) -> Result<()> {
        let marker_file = self.host_fault_marker_path(member, marker_path)?;
        match fs::remove_file(marker_file.as_path()) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(source) => Err(HarnessError::Io {
                path: marker_file,
                source,
            }),
        }
    }

    async fn bootstrap_cluster(&self) -> Result<()> {
        self.record_startup_phase(
            StartupPhase::DcsReady,
            "waiting for DCS services to report healthy",
        )?;
        for service in self.workspace.given.dcs_services() {
            self.wait_for_service_health(service.into()).await?;
        }
        self.record_startup_phase(StartupPhase::DcsReady, "all DCS services are healthy")?;
        self.record_startup_phase(StartupPhase::SeedPrimaryReady, "starting seed primary node-b")?;
        self.docker.compose_up_services(
            self.compose_file(),
            self.compose_project(),
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
            self.compose_file(),
            self.compose_project(),
            &["node-a", "node-c"],
        )?;
        self.record_startup_phase(
            StartupPhase::ClusterReady,
            "cluster bootstrapped enough for scenario execution",
        )
    }

    async fn wait_for_service_health(&self, service: ComposeService) -> Result<()> {
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
            let result = match self
                .observer()
                .state_via_member(ClusterMember::SEED_PRIMARY)
            {
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
        let compose_network = format!("{}_ha", self.compose_project());
        let invariant_result = self.ensure_background_invariants_ready();
        if let Err(err) = &invariant_result {
            failures.push(format!("background invariant check failed: {err}"));
        }
        let capture_result = self.capture_artifacts();
        if let Err(err) = &capture_result {
            failures.push(format!("artifact capture failed: {err}"));
        }
        let compose_result = self
            .docker
            .compose_down(self.compose_file(), self.compose_project());
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
            self.artifacts_dir().join("compose-ps.json").as_path(),
            serde_json::to_string_pretty(
                &self
                    .docker
                    .compose_ps_entries(self.compose_file(), self.compose_project())?,
            )
            .map_err(|source| HarnessError::Json {
                context: "serializing docker compose ps json".to_string(),
                source,
            })?
            .as_str(),
        )?;
        write_text_file(
            self.artifacts_dir().join("compose-logs.txt").as_path(),
            self.docker
                .compose_logs(self.compose_file(), self.compose_project())?
                .as_str(),
        )?;
        write_text_file(
            self.artifacts_dir().join("run-metadata.json").as_path(),
            serde_json::to_string_pretty(&serde_json::json!({
                "feature_name": self.feature_name(),
                "given_name": self.given_name(),
                "run_id": self.run_id(),
                "run_dir": self.run_dir(),
                "materialized_dir": self.materialized_dir(),
                "artifacts_dir": self.artifacts_dir(),
                "compose_project": self.compose_project(),
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
            self.artifacts_dir().join("timeline.json").as_path(),
            serde_json::to_string_pretty(&timeline)
                .map_err(|source| HarnessError::Json {
                    context: "serializing cucumber timeline".to_string(),
                    source,
                })?
                .as_str(),
        )?;
        match self.observer().observe_states() {
            Ok(states) => {
                let serialized = states
                    .members()
                    .iter()
                    .map(|observation| match &observation.outcome {
                        MemberCommandOutcome::Observed(output) => serde_json::json!({
                            "member": observation.member.service_name(),
                            "state": output,
                        }),
                        MemberCommandOutcome::Failed(message) => serde_json::json!({
                            "member": observation.member.service_name(),
                            "failure": message,
                        }),
                    })
                    .collect::<Vec<_>>();
                write_text_file(
                    self.artifacts_dir().join("operator-state.json").as_path(),
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

        for service in self.workspace.given.artifact_services() {
            match self.service_container_id(service) {
                Ok(container_id) => match self.docker.inspect_container(container_id.as_str()) {
                    Ok(inspect) => {
                        let artifact = self
                            .artifacts_dir()
                            .join(format!("inspect-{}.json", service.service_name()));
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

    fn cached_service_container_id(&self, service: ComposeService) -> Result<Option<String>> {
        self.service_container_ids
            .lock()
            .map(|cache| cache.get(&service).cloned())
            .map_err(|_| HarnessError::message("service container cache mutex was poisoned"))
    }

    fn refresh_service_container_ids(&self) -> Result<()> {
        let compose_entries = self
            .docker
            .compose_ps_entries(self.compose_file(), self.compose_project())?;
        let mut cache = self
            .service_container_ids
            .lock()
            .map_err(|_| HarnessError::message("service container cache mutex was poisoned"))?;
        compose_entries
            .into_iter()
            .filter_map(|entry| {
                self.compose_service_for_name(entry.service.as_str())
                    .map(|service| (service, entry.id))
            })
            .for_each(|(service, container_id)| {
                let _ = cache.insert(service, container_id);
            });
        Ok(())
    }

    fn compose_service_for_name(&self, service_name: &str) -> Option<ComposeService> {
        self.workspace
            .given
            .artifact_services()
            .into_iter()
            .find(|service| service.service_name() == service_name)
    }

    fn start_write_convergence_invariant_attachment(&self) -> Result<()> {
        self.background_invariants.attach_write_convergence(
            self.observer(),
            self.timeouts.poll_interval,
            self.timeouts.write_convergence_deadline,
        )?;
        self.record_startup_phase(
            StartupPhase::InvariantWriteConvergence,
            "started background accepted-write convergence attachment",
        )
    }
}

impl HarnessRuntime {
    fn observer(&self) -> PgtmObserver {
        PgtmObserver::new(
            self.docker.clone(),
            self.compose.file.clone(),
            self.compose.project.clone(),
            self.workspace.paths.materialized_dir.clone(),
        )
    }
}

impl BackgroundInvariants {
    async fn start_primary_count(
        observer: PgtmObserver,
        poll_interval: std::time::Duration,
        health_deadline: std::time::Duration,
    ) -> Result<Self> {
        let primary_count =
            PrimaryCountInvariantRunner::start(observer, poll_interval, health_deadline).await?;
        Ok(Self {
            primary_count,
            write_convergence: Mutex::new(WriteConvergenceActivation::Dormant),
        })
    }

    fn attach_write_convergence(
        &self,
        observer: PgtmObserver,
        poll_interval: std::time::Duration,
        write_deadline: std::time::Duration,
    ) -> Result<()> {
        let mut capability = self
            .write_convergence
            .lock()
            .map_err(|_| HarnessError::message("write-convergence state mutex was poisoned"))?;
        *capability = WriteConvergenceActivation::Activating(
            WriteConvergenceInvariantAttachment::spawn(observer, poll_interval, write_deadline),
        );
        Ok(())
    }

    fn ensure_healthy(
        &self,
        harness: &HarnessShared,
        requirement: InvariantRequirement,
    ) -> Result<()> {
        self.primary_count.ensure_running()?;
        self.ensure_write_convergence_healthy(harness, requirement)
    }

    fn ensure_write_convergence_healthy(
        &self,
        harness: &HarnessShared,
        requirement: InvariantRequirement,
    ) -> Result<()> {
        let pending_attachment = {
            let mut capability = self.write_convergence.lock().map_err(|_| {
                HarnessError::message("write-convergence state mutex was poisoned")
            })?;
            match &mut *capability {
                WriteConvergenceActivation::Active(runner) => {
                    return runner.ensure_healthy().map_err(HarnessError::from);
                }
                WriteConvergenceActivation::Dormant
                    if matches!(requirement, InvariantRequirement::ClusterReady) =>
                {
                    return Ok(());
                }
                WriteConvergenceActivation::Armed
                    if matches!(requirement, InvariantRequirement::ClusterReady) =>
                {
                    return Ok(());
                }
                WriteConvergenceActivation::Dormant => {
                    return Err(HarnessError::message(
                        "write-convergence activation has not been armed",
                    ));
                }
                WriteConvergenceActivation::Armed => {
                    return Err(HarnessError::message(
                        "write-convergence activation has not been started",
                    ));
                }
                WriteConvergenceActivation::Failed(message) => {
                    return Err(HarnessError::message(message.clone()));
                }
                WriteConvergenceActivation::Activating(attachment)
                    if matches!(requirement, InvariantRequirement::ClusterReady)
                        && !attachment.is_finished() =>
                {
                    return Ok(());
                }
                WriteConvergenceActivation::Activating(_) => {}
                WriteConvergenceActivation::Joining => {
                    return Err(HarnessError::message(
                        "write-convergence invariant attachment was re-entered while joining",
                    ));
                }
            }

            match std::mem::replace(
                &mut *capability,
                WriteConvergenceActivation::Joining,
            ) {
                WriteConvergenceActivation::Activating(attachment) => attachment,
                WriteConvergenceActivation::Active(runner) => {
                    *capability = WriteConvergenceActivation::Active(runner);
                    return Ok(());
                }
                WriteConvergenceActivation::Dormant => {
                    *capability = WriteConvergenceActivation::Dormant;
                    return Err(HarnessError::message(
                        "write-convergence activation has not been armed",
                    ));
                }
                WriteConvergenceActivation::Armed => {
                    *capability = WriteConvergenceActivation::Armed;
                    return Err(HarnessError::message(
                        "write-convergence activation has not been started",
                    ));
                }
                WriteConvergenceActivation::Failed(message) => {
                    *capability = WriteConvergenceActivation::Failed(message.clone());
                    return Err(HarnessError::message(message));
                }
                WriteConvergenceActivation::Joining => {
                    return Err(HarnessError::message(
                        "write-convergence invariant attachment was already joining",
                    ));
                }
            }
        };
        let (result, elapsed) = wait_for_write_convergence_attachment(pending_attachment)?;

        let mut capability = self
            .write_convergence
            .lock()
            .map_err(|_| HarnessError::message("write-convergence state mutex was poisoned"))?;
        match result {
            Ok(runner) => {
                harness.record_startup_phase(
                    StartupPhase::InvariantWriteConvergence,
                    format!("accepted-write convergence attached after {:?}", elapsed),
                )?;
                *capability = WriteConvergenceActivation::Active(runner);
            }
            Err(err) => {
                harness.record_startup_phase(
                    StartupPhase::InvariantWriteConvergence,
                    format!(
                        "accepted-write convergence attachment failed after {:?}: {err}",
                        elapsed
                    ),
                )?;
                *capability = WriteConvergenceActivation::Failed(err.to_string());
                return Err(err);
            }
        }

        match &*capability {
            WriteConvergenceActivation::Active(runner) => {
                runner.ensure_healthy().map_err(HarnessError::from)
            }
            WriteConvergenceActivation::Failed(message) => {
                Err(HarnessError::message(message.clone()))
            }
            WriteConvergenceActivation::Dormant
            | WriteConvergenceActivation::Armed
            | WriteConvergenceActivation::Activating(_)
            | WriteConvergenceActivation::Joining => Err(HarnessError::message(
                "write-convergence invariant attachment did not settle",
            )),
        }
    }
}

impl WriteConvergenceInvariantAttachment {
    fn spawn(
        observer: PgtmObserver,
        poll_interval: std::time::Duration,
        write_deadline: std::time::Duration,
    ) -> Self {
        Self {
            started_at: Instant::now(),
            task: Some(tokio::spawn(async move {
                WriteConvergenceInvariantRunner::start(observer, poll_interval, write_deadline)
                    .await
            })),
        }
    }

    fn is_finished(&self) -> bool {
        self.task.as_ref().is_some_and(JoinHandle::is_finished)
    }

    fn take_task(
        &mut self,
    ) -> Result<
        JoinHandle<std::result::Result<
            WriteConvergenceInvariantRunner,
            WriteConvergenceInvariantError,
        >>,
    > {
        self.task.take().ok_or_else(|| {
            HarnessError::message("write-convergence invariant attachment task was missing")
        })
    }
}

impl Drop for WriteConvergenceInvariantAttachment {
    fn drop(&mut self) {
        if let Some(task) = self.task.take() {
            task.abort();
        }
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
    mut attachment: WriteConvergenceInvariantAttachment,
) -> Result<(
    Result<WriteConvergenceInvariantRunner>,
    std::time::Duration,
)> {
    let elapsed = attachment.started_at.elapsed();
    let task = attachment.take_task()?;
    let result = block_on_harness_future(
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
    );
    Ok((result, elapsed))
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
        })?;
    apply_private_key_permissions(to)
}

fn materialize_given_fixture(given: &HaGivenDefinition, materialized_root: &Path) -> Result<()> {
    let FixtureMaterialization {
        shared_root,
        compose_variant,
        copies,
        runtime_configs,
    } = &given.materialization;
    for entry in copies {
        copy_shared_fixture_entry(shared_root.as_path(), materialized_root, entry)?;
    }
    for runtime_config in runtime_configs {
        materialize_runtime_config(materialized_root, runtime_config)?;
    }
    materialize_compose_include_file(materialized_root, *compose_variant)?;
    Ok(())
}

fn write_text_file(path: &Path, content: &str) -> Result<()> {
    fs::write(path, content).map_err(|source| HarnessError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn apply_private_key_permissions(path: &Path) -> Result<()> {
    if path.extension().and_then(|extension| extension.to_str()) != Some("key") {
        return Ok(());
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let permissions = fs::Permissions::from_mode(0o600);
        fs::set_permissions(path, permissions).map_err(|source| HarnessError::Io {
            path: path.to_path_buf(),
            source,
        })?;
    }

    Ok(())
}

fn copy_shared_fixture_entry(
    shared_root: &Path,
    materialized_root: &Path,
    entry: &SharedFixtureEntry,
) -> Result<()> {
    match entry {
        SharedFixtureEntry::Directory {
            source_relative_path,
            target_relative_path,
        } => copy_directory(
            shared_root.join(source_relative_path).as_path(),
            materialized_root.join(target_relative_path).as_path(),
        ),
        SharedFixtureEntry::File {
            source_relative_path,
            target_relative_path,
        } => {
            let target_path = materialized_root.join(target_relative_path);
            if let Some(parent) = target_path.parent() {
                create_dir_all(parent)?;
            }
            copy_file(
                shared_root.join(source_relative_path).as_path(),
                target_path.as_path(),
            )
        }
    }
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

fn materialize_runtime_config(
    materialized_root: &Path,
    runtime_config: &MemberRuntimeConfigMaterialization,
) -> Result<()> {
    let target_path = materialized_root.join(runtime_config.member.runtime_config_relative_path());
    if let Some(parent) = target_path.parent() {
        create_dir_all(parent)?;
    }
    let rendered = render_member_runtime_template(&runtime_config.template);
    write_text_file(target_path.as_path(), rendered.as_str())
}

fn materialize_compose_include_file(
    materialized_root: &Path,
    compose_variant: ComposeVariant,
) -> Result<()> {
    let compose_variant_path = compose_variant_absolute_path(compose_variant)?;
    let rendered = format!(
        "include:\n  - path: {}\n    project_directory: {}\n",
        toml_path_string(compose_variant_path.as_path()),
        toml_path_string(materialized_root),
    );
    write_text_file(
        materialized_root.join("compose.yml").as_path(),
        rendered.as_str(),
    )
}

fn compose_variant_absolute_path(compose_variant: ComposeVariant) -> Result<PathBuf> {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let absolute = repo_root
        .join("tests/ha/givens")
        .join(compose_variant.relative_path());
    if absolute.is_file() {
        Ok(absolute)
    } else {
        Err(HarnessError::message(format!(
            "static compose variant is missing: {}",
            absolute.display()
        )))
    }
}

fn toml_path_string(path: &Path) -> String {
    format!("{:?}", path.display().to_string())
}

fn render_member_runtime_template(template: &NodeRuntimeTemplate) -> String {
    let member = template.binding.member.service_name();
    let dcs_endpoint = template.binding.dcs_service.client_url();
    let replicator = template.postgres_roles.replicator.as_str();
    let rewinder = template.postgres_roles.rewinder.as_str();
    format!(
        r#"[cluster]
name = "ha-cucumber-cluster"
scope = "ha-cucumber-cluster"
member_id = "{member}"

[postgres.paths]
data_dir = "/var/lib/postgresql/data"
socket_dir = "/var/lib/pgtuskmaster/socket"
log_file = "/var/log/pgtuskmaster/postgres.log"

[postgres.network]
listen_host = "{member}"
listen_port = 5432

[postgres.rewind.transport]
ssl_mode = "verify-full"
ca_cert = {{ path = "/etc/pgtuskmaster/tls/ca.crt" }}

[postgres]
tls = {{ mode = "enabled", identity = {{ cert_chain = {{ path = "/etc/pgtuskmaster/tls/{member}.crt" }}, private_key = {{ path = "/etc/pgtuskmaster/tls/{member}.key" }} }}, client_auth = {{ client_ca = {{ path = "/etc/pgtuskmaster/tls/ca.crt" }}, client_certificate = "optional" }} }}

[postgres.access]
hba = {{ path = "/etc/pgtuskmaster/pg_hba.conf" }}
ident = {{ path = "/etc/pgtuskmaster/pg_ident.conf" }}

[postgres.extra_gucs]
wal_keep_size = "128MB"

[postgres.roles.mandatory.superuser]
username = "postgres"
auth = {{ type = "password", password = {{ type = "file", path = "/run/secrets/postgres-superuser-password" }} }}

[postgres.roles.mandatory.replicator]
username = "{replicator}"
auth = {{ type = "password", password = {{ type = "file", path = "/run/secrets/replicator-password" }} }}

[postgres.roles.mandatory.rewinder]
username = "{rewinder}"
auth = {{ type = "password", password = {{ type = "file", path = "/run/secrets/rewinder-password" }} }}

[dcs]
endpoints = ["{dcs_endpoint}"]

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process.timeouts]
pg_rewind_ms = 120000
bootstrap_ms = 300000
fencing_ms = 30000

[process.binaries.overrides]
postgres = "/usr/local/lib/pgtuskmaster/wrappers/postgres"
pg_ctl = "/usr/lib/postgresql/16/bin/pg_ctl"
pg_rewind = "/usr/local/lib/pgtuskmaster/wrappers/pg_rewind"
initdb = "/usr/lib/postgresql/16/bin/initdb"
pg_basebackup = "/usr/local/lib/pgtuskmaster/wrappers/pg_basebackup"
psql = "/usr/lib/postgresql/16/bin/psql"

[logging]
level = "info"
capture_subprocess_output = true

[logging.postgres]
enabled = true
poll_interval_ms = 200
cleanup = {{ enabled = true, max_files = 20, max_age_seconds = 86400, protect_recent_seconds = 300 }}

[logging.sinks.stderr]
enabled = true

[logging.sinks.file]
enabled = true
path = "/var/log/pgtuskmaster/runtime.jsonl"
mode = "append"

[api]
listen_addr = "0.0.0.0:8443"
transport = {{ transport = "https", tls = {{ identity = {{ cert_chain = {{ path = "/etc/pgtuskmaster/tls/{member}.crt" }}, private_key = {{ path = "/etc/pgtuskmaster/tls/{member}.key" }} }} }} }}
auth = {{ type = "role_tokens", tokens = {{ read_token = {{ type = "file", path = "/run/secrets/api-read-token" }}, admin_token = {{ type = "file", path = "/run/secrets/api-admin-token" }} }} }}

[pgtm.api]
base_url = "https://{member}:8443"
auth = {{ type = "role_tokens", tokens = {{ read_token = {{ type = "file", path = "/run/secrets/api-read-token" }}, admin_token = {{ type = "file", path = "/run/secrets/api-admin-token" }} }} }}
tls = {{ ca_cert = {{ path = "/etc/pgtuskmaster/tls/ca.crt" }} }}

[pgtm.postgres.tls]
ca_cert = {{ path = "/etc/pgtuskmaster/tls/ca.crt" }}

[debug]
enabled = true
"#
    )
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
    if operator_visible_primary(status).is_none() {
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
    match &status.ha.publication {
        PublicationState::Projected(AuthorityProjection::Primary(epoch)) => {
            Some(epoch.holder.0.clone())
        }
        PublicationState::Unknown
        | PublicationState::Projected(AuthorityProjection::NoPrimary(_)) => None,
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Mutex, time::Duration};

    use super::*;
    use crate::support::{
        docker::cli::DockerCli,
        invariants::WriteConvergenceInvariantError,
        timeouts::TimeoutModel,
    };

    fn temporary_directory(name: &str) -> Result<PathBuf> {
        let root = std::env::temp_dir().join(format!(
            "pgtm-ha-world-{name}-{}-{}",
            std::process::id(),
            timestamp_millis()?
        ));
        match fs::remove_dir_all(root.as_path()) {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(source) => {
                return Err(HarnessError::Io { path: root, source });
            }
        }
        create_dir_all(root.as_path())?;
        Ok(root)
    }

    fn cleanup_directory(path: &Path) -> Result<()> {
        match fs::remove_dir_all(path) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(source) => Err(HarnessError::Io {
                path: path.to_path_buf(),
                source,
            }),
        }
    }

    #[test]
    fn materializes_plain_fixture_from_shared_assets_and_static_include_compose() -> Result<()> {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let given = resolve_given(repo_root.as_path(), HaGivenId::Plain)?;
        let output_root = temporary_directory("plain")?;

        let result = (|| -> Result<()> {
            materialize_given_fixture(&given, output_root.as_path())?;

            let compose =
                fs::read_to_string(output_root.join("compose.yml")).map_err(|source| {
                    HarnessError::Io {
                        path: output_root.join("compose.yml"),
                        source,
                    }
                })?;
            let expected_variant = compose_variant_absolute_path(ComposeVariant::SharedSingleDcs)?;
            assert!(compose.contains("include:"));
            assert!(compose.contains(expected_variant.display().to_string().as_str()));
            assert!(compose.contains(output_root.display().to_string().as_str()));

            let runtime = fs::read_to_string(
                output_root.join(ClusterMember::NodeA.runtime_config_relative_path()),
            )
            .map_err(|source| HarnessError::Io {
                path: output_root.join(ClusterMember::NodeA.runtime_config_relative_path()),
                source,
            })?;
            toml::from_str::<pgtuskmaster_rust::config::RuntimeConfig>(runtime.as_str()).map_err(
                |source| {
                    HarnessError::message(format!(
                        "materialized node runtime config failed to parse: {source}"
                    ))
                },
            )?;
            assert!(runtime.contains(r#"username = "replicator""#));
            assert!(runtime.contains(r#"username = "rewinder""#));
            assert!(output_root.join("configs/tls/ca.crt").is_file());
            assert!(output_root.join("secrets/replicator-password").is_file());
            Ok(())
        })();

        let cleanup_result = cleanup_directory(output_root.as_path());
        match (result, cleanup_result) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(err), Ok(())) => Err(err),
            (Ok(()), Err(cleanup)) => Err(cleanup),
            (Err(err), Err(cleanup)) => Err(HarnessError::message(format!(
                "{err}\ncleanup also failed: {cleanup}"
            ))),
        }
    }

    #[test]
    fn materializes_custom_roles_without_custom_compose_variant() -> Result<()> {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let given = resolve_given(repo_root.as_path(), HaGivenId::CustomRoles)?;
        let output_root = temporary_directory("custom-roles")?;

        let result = (|| -> Result<()> {
            materialize_given_fixture(&given, output_root.as_path())?;

            let compose =
                fs::read_to_string(output_root.join("compose.yml")).map_err(|source| {
                    HarnessError::Io {
                        path: output_root.join("compose.yml"),
                        source,
                    }
                })?;
            let expected_variant = compose_variant_absolute_path(ComposeVariant::SharedSingleDcs)?;
            assert!(compose.contains(expected_variant.display().to_string().as_str()));

            let runtime = fs::read_to_string(
                output_root.join(ClusterMember::NodeB.runtime_config_relative_path()),
            )
            .map_err(|source| HarnessError::Io {
                path: output_root.join(ClusterMember::NodeB.runtime_config_relative_path()),
                source,
            })?;
            assert!(runtime.contains(r#"username = "mirrorbot""#));
            assert!(runtime.contains(r#"username = "rewindbot""#));
            Ok(())
        })();

        let cleanup_result = cleanup_directory(output_root.as_path());
        match (result, cleanup_result) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(err), Ok(())) => Err(err),
            (Ok(()), Err(cleanup)) => Err(cleanup),
            (Err(err), Err(cleanup)) => Err(HarnessError::message(format!(
                "{err}\ncleanup also failed: {cleanup}"
            ))),
        }
    }

    #[test]
    fn materializes_three_etcd_fixture_with_node_local_dcs_bindings() -> Result<()> {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let given = resolve_given(repo_root.as_path(), HaGivenId::ThreeEtcd)?;
        let output_root = temporary_directory("three-etcd")?;

        let result = (|| -> Result<()> {
            materialize_given_fixture(&given, output_root.as_path())?;

            let compose =
                fs::read_to_string(output_root.join("compose.yml")).map_err(|source| {
                    HarnessError::Io {
                        path: output_root.join("compose.yml"),
                        source,
                    }
                })?;
            let expected_variant =
                compose_variant_absolute_path(ComposeVariant::ColocatedThreeMemberDcs)?;
            assert!(compose.contains(expected_variant.display().to_string().as_str()));

            let node_a_runtime = fs::read_to_string(
                output_root.join(ClusterMember::NodeA.runtime_config_relative_path()),
            )
            .map_err(|source| HarnessError::Io {
                path: output_root.join(ClusterMember::NodeA.runtime_config_relative_path()),
                source,
            })?;
            toml::from_str::<pgtuskmaster_rust::config::RuntimeConfig>(node_a_runtime.as_str())
                .map_err(|source| {
                    HarnessError::message(format!(
                        "materialized node-a runtime config failed to parse: {source}"
                    ))
                })?;
            assert!(node_a_runtime.contains(r#"endpoints = ["http://etcd-a:2379"]"#));

            let node_b_runtime = fs::read_to_string(
                output_root.join(ClusterMember::NodeB.runtime_config_relative_path()),
            )
            .map_err(|source| HarnessError::Io {
                path: output_root.join(ClusterMember::NodeB.runtime_config_relative_path()),
                source,
            })?;
            toml::from_str::<pgtuskmaster_rust::config::RuntimeConfig>(node_b_runtime.as_str())
                .map_err(|source| {
                    HarnessError::message(format!(
                        "materialized node-b runtime config failed to parse: {source}"
                    ))
                })?;
            assert!(node_b_runtime.contains(r#"endpoints = ["http://etcd-b:2379"]"#));
            Ok(())
        })();

        let cleanup_result = cleanup_directory(output_root.as_path());
        match (result, cleanup_result) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(err), Ok(())) => Err(err),
            (Ok(()), Err(cleanup)) => Err(cleanup),
            (Err(err), Err(cleanup)) => Err(HarnessError::message(format!(
                "{err}\ncleanup also failed: {cleanup}"
            ))),
        }
    }

    #[test]
    fn member_alias_round_trips() -> Result<()> {
        let mut world = HaWorld::default();
        world.remember_member_alias("replica", ClusterMember::NodeB);

        assert_eq!(world.require_member_alias("replica")?, ClusterMember::NodeB);
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
        write_convergence: WriteConvergenceActivation,
    ) -> Result<HarnessShared> {
        Ok(HarnessShared {
            runtime: HarnessRuntime {
                workspace: HarnessWorkspace {
                    run_id: "test-run".to_string(),
                    feature_name: "test-feature".to_string(),
                    given: resolve_given(
                        PathBuf::from(env!("CARGO_MANIFEST_DIR")).as_path(),
                        HaGivenId::Plain,
                    )?,
                    paths: WorkspacePaths {
                        run_dir: PathBuf::from("tests/ha/runs/test-feature/test-run"),
                        materialized_dir: PathBuf::from(
                            "tests/ha/runs/test-feature/test-run/materialized",
                        ),
                        artifacts_dir: PathBuf::from("tests/ha/runs/test-feature/test-run/artifacts"),
                    },
                },
                compose: ComposeStack {
                    file: PathBuf::from("tests/ha/runs/test-feature/test-run/materialized/compose.yml"),
                    project: "ha-test-run".to_string(),
                },
                cucumber_test_image_run_id: "cucumber-test-run".to_string(),
                docker: DockerCli::fake_for_tests(),
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
            },
            background_invariants: BackgroundInvariants {
                primary_count: PrimaryCountInvariantRunner::healthy_for_tests(),
                write_convergence: Mutex::new(write_convergence),
            },
        })
    }

    fn delayed_failed_attachment(message: &str) -> WriteConvergenceActivation {
        let message = message.to_string();
            WriteConvergenceActivation::Activating(WriteConvergenceInvariantAttachment {
                started_at: Instant::now(),
                task: Some(tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_millis(20)).await;
                    Err(WriteConvergenceInvariantError::Failed(message))
                })),
        })
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn background_health_check_allows_pending_write_convergence_attachment() -> Result<()> {
        let harness = test_harness_with_write_convergence(delayed_failed_attachment("pending"))?;
        harness.ensure_primary_count_healthy()
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn cleanup_check_waits_for_write_convergence_attachment_failure() -> Result<()> {
        let harness = test_harness_with_write_convergence(delayed_failed_attachment("boom"))?;
        let err = match harness.ensure_accepted_writes_ready() {
            Ok(()) => {
                return Err(HarnessError::message(
                    "required invariant readiness unexpectedly succeeded",
                ));
            }
            Err(err) => err,
        };
        assert!(err.to_string().contains("boom"));
        let write_convergence_state = harness
            .background_invariants
            .write_convergence
            .lock()
            .map_err(|_| HarnessError::message("write-convergence state mutex was poisoned"))?;
        match &*write_convergence_state {
            WriteConvergenceActivation::Failed(message) => {
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
