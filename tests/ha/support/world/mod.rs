use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use cucumber::World;
use pgtuskmaster_rust::{
    api::NodeState,
    dcs::DcsMemberPostgresView,
    ha::types::{AuthorityProjection, PublicationState},
};

use crate::support::{
    docker::{cli::DockerCli, ryuk::RyukGuard},
    error::{HarnessError, Result},
    faults::{
        append_fault_rule_script, clear_fault_rules_script, ensure_fault_plumbing_script,
        remove_fault_rule_script, signal_named_process_script, BlockerKind, TrafficPath,
        DATABASE_MEMBERS, FAULT_DIR,
    },
    feature_metadata,
    givens::{
        resolve_given, ComposeTemplate, FixtureMaterialization, FixtureRenderTarget,
        FixtureTemplate, HaGivenDefinition, HaGivenId, ObserverNetAdmin, ObserverTemplate,
        RenderedFixtureFile, SharedFixtureEntry, ThreeNodeDcsLayout,
    },
    observer::{pgtm::PgtmObserver, sql::SqlObserver},
    timeouts::TimeoutModel,
    topology::{ClusterMember, ComposeService, DcsMember},
    workload::{SqlWorkloadHandle, WorkloadSummary},
};

#[derive(Debug, Default, World)]
pub struct HaWorld {
    pub harness: Option<HarnessShared>,
    pub scenario: ScenarioState,
}

#[derive(Debug, Default)]
pub struct ScenarioState {
    pub aliases: AliasRegistry,
    pub workload: WorkloadState,
    pub command: CommandState,
    pub transition: TransitionWindow,
    pub invariants: InvariantHistory,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemberAlias(String);

impl From<&str> for MemberAlias {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for MemberAlias {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MarkerName(String);

impl From<&str> for MarkerName {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for MarkerName {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProofRow(String);

impl From<&str> for ProofRow {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for ProofRow {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl ProofRow {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProofTableName(String);

impl From<&str> for ProofTableName {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for ProofTableName {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl ProofTableName {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
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

    pub fn len(&self) -> usize {
        self.members.len()
    }

    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }
}

#[derive(Debug, Default)]
pub struct AliasRegistry {
    pub members_by_alias: BTreeMap<MemberAlias, ClusterMember>,
}

#[derive(Debug, Default)]
pub struct ProofLedger {
    pub table: Option<ProofTableName>,
    pub recorded_rows: Vec<ProofRow>,
    pub convergence_blocked_members: MemberSet,
}

#[derive(Debug, Default)]
pub struct WorkloadState {
    pub active: Option<SqlWorkloadHandle>,
    pub last_summary: Option<WorkloadSummary>,
    pub proof: ProofLedger,
}

#[derive(Debug, Default)]
pub struct CommandState {
    pub last_output: Option<String>,
}

#[derive(Debug, Default)]
pub struct ObservationScope {
    pub observer_unreachable_members: MemberSet,
}

#[derive(Debug, Default)]
pub struct TransitionWindow {
    pub markers: BTreeMap<MarkerName, u128>,
    pub stopped_members: MemberSet,
    pub wedged_members: MemberSet,
    pub observation_scope: ObservationScope,
}

#[derive(Debug, Default)]
pub struct InvariantHistory {
    pub observed_authoritative_primaries: Vec<ClusterMember>,
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

    pub fn record_marker(&mut self, marker: impl Into<MarkerName>, timestamp_ms: u128) {
        self.scenario
            .transition
            .markers
            .insert(marker.into(), timestamp_ms);
    }

    pub fn marker(&self, marker: &str) -> Result<u128> {
        self.scenario
            .transition
            .markers
            .get(&MarkerName::from(marker))
            .copied()
            .ok_or_else(|| HarnessError::message(format!("marker `{marker}` was not recorded")))
    }

    pub fn remember_alias(&mut self, alias: impl Into<MemberAlias>, member: ClusterMember) {
        self.scenario
            .aliases
            .members_by_alias
            .insert(alias.into(), member);
    }

    pub fn require_alias(&self, alias: &str) -> Result<ClusterMember> {
        self.scenario
            .aliases
            .members_by_alias
            .get(&MemberAlias::from(alias))
            .cloned()
            .ok_or_else(|| HarnessError::message(format!("alias `{alias}` was not recorded")))
    }

    pub fn add_stopped_node(&mut self, member: ClusterMember) {
        let _ = self.scenario.transition.stopped_members.insert(member);
    }

    pub fn remove_stopped_node(&mut self, member: ClusterMember) {
        let _ = self.scenario.transition.stopped_members.remove(member);
    }

    pub fn mark_observer_unreachable(&mut self, member: ClusterMember) {
        let _ = self
            .scenario
            .transition
            .observation_scope
            .observer_unreachable_members
            .insert(member);
    }

    pub fn clear_observer_unreachable(&mut self, member: ClusterMember) {
        let _ = self
            .scenario
            .transition
            .observation_scope
            .observer_unreachable_members
            .remove(member);
    }

    pub fn add_wedged_node(&mut self, member: ClusterMember) {
        let _ = self.scenario.transition.wedged_members.insert(member);
    }

    pub fn remove_wedged_node(&mut self, member: ClusterMember) {
        let _ = self.scenario.transition.wedged_members.remove(member);
    }

    pub fn add_proof_convergence_blocker(&mut self, member: ClusterMember) {
        let _ = self
            .scenario
            .workload
            .proof
            .convergence_blocked_members
            .insert(member);
    }

    pub fn remove_proof_convergence_blocker(&mut self, member: ClusterMember) {
        let _ = self
            .scenario
            .workload
            .proof
            .convergence_blocked_members
            .remove(member);
    }

    pub fn clear_observer_unreachable_members(&mut self) {
        self.scenario
            .transition
            .observation_scope
            .observer_unreachable_members
            .clear();
    }

    pub fn clear_proof_convergence_blockers(&mut self) {
        self.scenario
            .workload
            .proof
            .convergence_blocked_members
            .clear();
    }

    pub fn clear_primary_history(&mut self) {
        self.scenario
            .invariants
            .observed_authoritative_primaries
            .clear();
    }

    pub fn record_primary_observation(&mut self, member: ClusterMember) {
        let already_recorded = self
            .scenario
            .invariants
            .observed_authoritative_primaries
            .iter()
            .any(|observed| observed == &member);
        if !already_recorded {
            self.scenario
                .invariants
                .observed_authoritative_primaries
                .push(member);
        }
    }

    pub fn cleanup(&mut self) -> Result<()> {
        let workload_result = self
            .scenario
            .workload
            .active
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
pub struct HarnessShared {
    pub workspace: HarnessWorkspace,
    pub compose: ComposeStack,
    pub cucumber_test_image_run_id: String,
    pub docker: DockerCli,
    pub ryuk: Option<RyukGuard>,
    pub observer_container: String,
    pub timeouts: TimeoutModel,
    timeline: Mutex<Vec<serde_json::Value>>,
    cleaned_up: bool,
}

impl HarnessShared {
    pub async fn initialize(given: HaGivenId) -> Result<Self> {
        let feature = feature_metadata()?;
        let docker = DockerCli::discover()?;
        docker.verify_daemon()?;

        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let given = resolve_given(repo_root.as_path(), given)?;
        let run_id = build_run_id(feature.feature_name.as_str())?;
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
        docker.compose_up_services(
            compose.file.as_path(),
            compose.project.as_str(),
            &["observer"],
        )?;
        let observer_container = docker.compose_container_id(
            compose.file.as_path(),
            compose.project.as_str(),
            "observer",
        )?;

        let mut harness = Self {
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
            observer_container,
            timeouts,
            timeline: Mutex::new(Vec::new()),
            cleaned_up: false,
        };
        harness.record_note(
            "initialize",
            format!(
                "created per-feature run workspace using cucumber image run id `{}`",
                harness.cucumber_test_image_run_id
            ),
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
        PgtmObserver::new(self.docker.clone(), self.observer_container.clone())
    }

    pub fn sql(&self) -> SqlObserver {
        SqlObserver::new(self.docker.clone(), self.observer_container.clone())
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
        self.docker.compose_container_id(
            self.compose_file(),
            self.compose_project(),
            service.service_name(),
        )
    }

    pub fn service_logs(&self, service: ComposeService) -> Result<String> {
        let container_id = self.service_container_id(service)?;
        self.docker.container_logs(container_id.as_str())
    }

    pub fn write_artifact_json(
        &self,
        artifact_name: &str,
        value: &serde_json::Value,
    ) -> Result<()> {
        let path = self.artifacts_dir().join(artifact_name);
        let content = serde_json::to_string_pretty(value).map_err(|source| HarnessError::Json {
            context: format!("serializing artifact `{artifact_name}`"),
            source,
        })?;
        write_text_file(path.as_path(), content.as_str())
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
        self.ensure_fault_plumbing(member.into())?;
        let script = append_fault_rule_script(peer_ip.as_str(), path.port());
        let _ = self.run_shell_as_root(member.into(), script.as_str())?;
        self.record_note(
            "fault.block_path",
            format!("member={member} path={} peer={peer_service}", path.label()),
        )?;
        Ok(())
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
        self.block_member_path_to_host(member, TrafficPath::Api, ComposeService::Observer)
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

    pub fn wedge_member_postgres(&self, member: ClusterMember) -> Result<()> {
        let script = signal_named_process_script("STOP", "postgres");
        let _ = self.run_shell_as_root(member.into(), script.as_str())?;
        self.record_note("fault.wedge_postgres", format!("member={member}"))?;
        Ok(())
    }

    pub fn unwedge_member_postgres(&self, member: ClusterMember) -> Result<()> {
        let script = signal_named_process_script("CONT", "postgres");
        let _ = self.run_shell_as_root(member.into(), script.as_str())?;
        self.record_note("fault.unwedge_postgres", format!("member={member}"))?;
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
            let status = serde_json::from_value::<NodeState>(entry.get("status")?.clone()).ok()?;
            let primaries = dcs_primary_members(&status);
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

    pub fn assert_member_never_primary_since(
        &self,
        member: ClusterMember,
        since_ms: u128,
    ) -> Result<()> {
        let timeline = self.timeline_entries()?;
        let mut member_was_primary = false;
        let mut member_relinquished_primary = false;
        let offending = timeline.iter().find(|entry| {
            let timestamp_ms = entry
                .get("timestamp_ms")
                .and_then(serde_json::Value::as_u64)
                .map(u128::from);
            if timestamp_ms.is_none_or(|value| value < since_ms) {
                return false;
            }
            let member_is_primary = entry
                .get("status")
                .cloned()
                .and_then(|status| serde_json::from_value::<NodeState>(status).ok())
                .and_then(|status| operator_visible_primary(&status))
                .is_some_and(|primary| primary == member.service_name());
            if member_is_primary {
                let regained_primary = member_was_primary && member_relinquished_primary;
                member_was_primary = true;
                regained_primary
            } else {
                if member_was_primary {
                    member_relinquished_primary = true;
                }
                false
            }
        });
        match offending {
            Some(_) => Err(HarnessError::message(format!(
                "timeline captured `{member}` regaining primary after marker {since_ms}"
            ))),
            None => Ok(()),
        }
    }

    pub fn clear_all_network_faults(&self) -> Result<()> {
        for service in DATABASE_MEMBERS
            .into_iter()
            .map(ComposeService::from)
            .chain(std::iter::once(ComposeService::Observer))
        {
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
        for service in self.workspace.given.dcs_services() {
            self.wait_for_service_health(service.into()).await?;
        }
        self.record_note("bootstrap", "starting seed primary node-b")?;
        self.docker.compose_up_services(
            self.compose_file(),
            self.compose_project(),
            &["node-b"],
        )?;
        self.wait_for_seed_primary().await?;
        self.record_note("bootstrap", "starting remaining nodes node-a and node-c")?;
        self.docker.compose_up_services(
            self.compose_file(),
            self.compose_project(),
            &["node-a", "node-c"],
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
            let result = match self.observer().state() {
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
            .compose_down(self.compose_file(), self.compose_project());
        if let Err(err) = &compose_result {
            failures.push(format!("docker compose down failed: {err}"));
        }
        let ryuk_result = self.ryuk.as_mut().map(RyukGuard::close).transpose();
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
        match self.observer().state() {
            Ok(state) => write_text_file(
                self.artifacts_dir().join("observer-state.json").as_path(),
                serde_json::to_string_pretty(&state)
                    .map_err(|source| HarnessError::Json {
                        context: "serializing observer state payload".to_string(),
                        source,
                    })?
                    .as_str(),
            )?,
            Err(err) => failures.push(format!("observer state capture failed: {err}")),
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
}

fn container_not_running_error(err: &HarnessError) -> bool {
    matches!(err, HarnessError::CommandFailed { stderr, .. } if stderr.contains("is not running"))
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
        })
}

fn materialize_given_fixture(given: &HaGivenDefinition, materialized_root: &Path) -> Result<()> {
    let FixtureMaterialization {
        shared_root,
        copies,
        renders,
    } = &given.materialization;
    for entry in copies {
        copy_shared_fixture_entry(shared_root.as_path(), materialized_root, entry)?;
    }
    for render in renders {
        render_fixture_file(materialized_root, render)?;
    }
    Ok(())
}

fn write_text_file(path: &Path, content: &str) -> Result<()> {
    fs::write(path, content).map_err(|source| HarnessError::Io {
        path: path.to_path_buf(),
        source,
    })
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

fn render_fixture_file(materialized_root: &Path, file: &RenderedFixtureFile) -> Result<()> {
    let target_path = materialized_root.join(render_target_relative_path(&file.target));
    if let Some(parent) = target_path.parent() {
        create_dir_all(parent)?;
    }
    write_text_file(
        target_path.as_path(),
        render_fixture_template(&file.template).as_str(),
    )
}

fn render_target_relative_path(target: &FixtureRenderTarget) -> PathBuf {
    match target {
        FixtureRenderTarget::ComposeFile => PathBuf::from("compose.yml"),
        FixtureRenderTarget::MemberRuntimeConfig(member) => {
            PathBuf::from(member.runtime_config_relative_path())
        }
        FixtureRenderTarget::ObserverConfig(member) => {
            PathBuf::from(member.observer_config_relative_path())
        }
    }
}

fn render_fixture_template(template: &FixtureTemplate) -> String {
    match template {
        FixtureTemplate::Compose(template) => render_compose_template(*template),
        FixtureTemplate::Runtime(template) => render_member_runtime_template(template),
        FixtureTemplate::Observer(template) => render_observer_template(template),
    }
}

fn render_compose_template(template: ComposeTemplate) -> String {
    let observer_cap_add = match template.observer_net_admin {
        ObserverNetAdmin::Enabled => "    cap_add:\n      - NET_ADMIN\n",
        ObserverNetAdmin::Disabled => "",
    };
    let dcs_services = render_dcs_services(template.dcs_layout);
    let dcs_volumes = render_dcs_volumes(template.dcs_layout);
    format!(
        r#"services:
{dcs_services}

  node-a:
    image: pgtm-cucumber-test:${{PGTM_CUCUMBER_TEST_RUN_ID:?missing PGTM_CUCUMBER_TEST_RUN_ID}}
    pull_policy: never
    cap_add:
      - NET_ADMIN
    networks:
      - ha
    configs:
      - source: node_a_runtime
        target: /etc/pgtuskmaster/runtime.toml
      - source: pg_hba
        target: /etc/pgtuskmaster/pg_hba.conf
      - source: pg_ident
        target: /etc/pgtuskmaster/pg_ident.conf
      - source: tls_ca
        target: /etc/pgtuskmaster/tls/ca.crt
      - source: tls_node_a_crt
        target: /etc/pgtuskmaster/tls/node-a.crt
      - source: tls_node_a_key
        target: /etc/pgtuskmaster/tls/node-a.key
    secrets:
      - source: postgres_superuser_password
        target: postgres-superuser-password
      - source: api_admin_token
        target: api-admin-token
      - source: api_read_token
        target: api-read-token
      - source: replicator_password
        target: replicator-password
      - source: rewinder_password
        target: rewinder-password
    volumes:
      - node-a-data:/var/lib/postgresql
      - node-a-logs:/var/log/pgtuskmaster
      - ./faults/node-a:/var/lib/pgtuskmaster/faults

  node-b:
    image: pgtm-cucumber-test:${{PGTM_CUCUMBER_TEST_RUN_ID:?missing PGTM_CUCUMBER_TEST_RUN_ID}}
    pull_policy: never
    cap_add:
      - NET_ADMIN
    networks:
      - ha
    configs:
      - source: node_b_runtime
        target: /etc/pgtuskmaster/runtime.toml
      - source: pg_hba
        target: /etc/pgtuskmaster/pg_hba.conf
      - source: pg_ident
        target: /etc/pgtuskmaster/pg_ident.conf
      - source: tls_ca
        target: /etc/pgtuskmaster/tls/ca.crt
      - source: tls_node_b_crt
        target: /etc/pgtuskmaster/tls/node-b.crt
      - source: tls_node_b_key
        target: /etc/pgtuskmaster/tls/node-b.key
    secrets:
      - source: postgres_superuser_password
        target: postgres-superuser-password
      - source: api_admin_token
        target: api-admin-token
      - source: api_read_token
        target: api-read-token
      - source: replicator_password
        target: replicator-password
      - source: rewinder_password
        target: rewinder-password
    volumes:
      - node-b-data:/var/lib/postgresql
      - node-b-logs:/var/log/pgtuskmaster
      - ./faults/node-b:/var/lib/pgtuskmaster/faults

  node-c:
    image: pgtm-cucumber-test:${{PGTM_CUCUMBER_TEST_RUN_ID:?missing PGTM_CUCUMBER_TEST_RUN_ID}}
    pull_policy: never
    cap_add:
      - NET_ADMIN
    networks:
      - ha
    configs:
      - source: node_c_runtime
        target: /etc/pgtuskmaster/runtime.toml
      - source: pg_hba
        target: /etc/pgtuskmaster/pg_hba.conf
      - source: pg_ident
        target: /etc/pgtuskmaster/pg_ident.conf
      - source: tls_ca
        target: /etc/pgtuskmaster/tls/ca.crt
      - source: tls_node_c_crt
        target: /etc/pgtuskmaster/tls/node-c.crt
      - source: tls_node_c_key
        target: /etc/pgtuskmaster/tls/node-c.key
    secrets:
      - source: postgres_superuser_password
        target: postgres-superuser-password
      - source: api_admin_token
        target: api-admin-token
      - source: api_read_token
        target: api-read-token
      - source: replicator_password
        target: replicator-password
      - source: rewinder_password
        target: rewinder-password
    volumes:
      - node-c-data:/var/lib/postgresql
      - node-c-logs:/var/log/pgtuskmaster
      - ./faults/node-c:/var/lib/pgtuskmaster/faults

  observer:
    image: pgtm-cucumber-test:${{PGTM_CUCUMBER_TEST_RUN_ID:?missing PGTM_CUCUMBER_TEST_RUN_ID}}
    pull_policy: never
    entrypoint:
      - /usr/bin/tail
    command:
      - -f
      - /dev/null
{observer_cap_add}    networks:
      - ha
    configs:
      - source: observer_node_a
        target: /etc/pgtuskmaster/observer/node-a.toml
      - source: observer_node_b
        target: /etc/pgtuskmaster/observer/node-b.toml
      - source: observer_node_c
        target: /etc/pgtuskmaster/observer/node-c.toml
      - source: pg_hba
        target: /etc/pgtuskmaster/pg_hba.conf
      - source: pg_ident
        target: /etc/pgtuskmaster/pg_ident.conf
      - source: tls_ca
        target: /etc/pgtuskmaster/tls/ca.crt
      - source: tls_observer_crt
        target: /etc/pgtuskmaster/tls/observer.crt
      - source: tls_observer_key
        target: /etc/pgtuskmaster/tls/observer.key
    secrets:
      - source: postgres_superuser_password
        target: postgres-superuser-password
      - source: api_admin_token
        target: api-admin-token
      - source: api_read_token
        target: api-read-token
      - source: replicator_password
        target: replicator-password
      - source: rewinder_password
        target: rewinder-password

networks:
  ha:
    driver: bridge

volumes:
{dcs_volumes}  node-a-data:
  node-a-logs:
  node-b-data:
  node-b-logs:
  node-c-data:
  node-c-logs:

configs:
  node_a_runtime:
    file: ./configs/node-a/runtime.toml
  node_b_runtime:
    file: ./configs/node-b/runtime.toml
  node_c_runtime:
    file: ./configs/node-c/runtime.toml
  observer_node_a:
    file: ./configs/observer/node-a.toml
  observer_node_b:
    file: ./configs/observer/node-b.toml
  observer_node_c:
    file: ./configs/observer/node-c.toml
  pg_hba:
    file: ./configs/pg_hba.conf
  pg_ident:
    file: ./configs/pg_ident.conf
  tls_ca:
    file: ./configs/tls/ca.crt
  tls_node_a_crt:
    file: ./configs/tls/node-a.crt
  tls_node_a_key:
    file: ./configs/tls/node-a.key
  tls_node_b_crt:
    file: ./configs/tls/node-b.crt
  tls_node_b_key:
    file: ./configs/tls/node-b.key
  tls_node_c_crt:
    file: ./configs/tls/node-c.crt
  tls_node_c_key:
    file: ./configs/tls/node-c.key
  tls_observer_crt:
    file: ./configs/tls/observer.crt
  tls_observer_key:
    file: ./configs/tls/observer.key

secrets:
  postgres_superuser_password:
    file: ./secrets/postgres-superuser-password
  api_admin_token:
    file: ./secrets/api-admin-token
  api_read_token:
    file: ./secrets/api-read-token
  replicator_password:
    file: ./secrets/replicator-password
  rewinder_password:
    file: ./secrets/rewinder-password
"#
    )
}

fn render_member_runtime_template(
    template: &crate::support::givens::NodeRuntimeTemplate,
) -> String {
    let member = template.binding.member.service_name();
    let dcs_endpoint = template.binding.dcs_service.client_url();
    let replicator = template.postgres_roles.replicator.as_str();
    let rewinder = template.postgres_roles.rewinder.as_str();
    format!(
        r#"[cluster]
name = "ha-cucumber-cluster"
member_id = "{member}"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "{member}"
listen_port = 5432
socket_dir = "/var/lib/pgtuskmaster/socket"
log_file = "/var/log/pgtuskmaster/postgres.log"
local_conn_identity = {{ user = "postgres", dbname = "postgres", ssl_mode = "disable" }}
rewind_conn_identity = {{ user = "{rewinder}", dbname = "postgres", ssl_mode = "verify-full", ca_cert = {{ path = "/etc/pgtuskmaster/tls/ca.crt" }} }}
tls = {{ mode = "enabled", identity = {{ cert_chain = {{ path = "/etc/pgtuskmaster/tls/{member}.crt" }}, private_key = {{ path = "/etc/pgtuskmaster/tls/{member}.key" }} }}, client_auth = {{ client_ca = {{ path = "/etc/pgtuskmaster/tls/ca.crt" }}, client_certificate = "optional" }} }}
pg_hba = {{ source = {{ path = "/etc/pgtuskmaster/pg_hba.conf" }} }}
pg_ident = {{ source = {{ path = "/etc/pgtuskmaster/pg_ident.conf" }} }}

[postgres.extra_gucs]
wal_keep_size = "128MB"

[postgres.roles.superuser]
username = "postgres"
auth = {{ type = "password", password = {{ path = "/run/secrets/postgres-superuser-password" }} }}

[postgres.roles.replicator]
username = "{replicator}"
auth = {{ type = "password", password = {{ path = "/run/secrets/replicator-password" }} }}

[postgres.roles.rewinder]
username = "{rewinder}"
auth = {{ type = "password", password = {{ path = "/run/secrets/rewinder-password" }} }}

[dcs]
endpoints = ["{dcs_endpoint}"]
scope = "ha-cucumber-cluster"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
pg_rewind_timeout_ms = 120000
bootstrap_timeout_ms = 300000
fencing_timeout_ms = 30000

[process.binaries]
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
security = {{ transport = {{ transport = "https", tls = {{ identity = {{ cert_chain = {{ path = "/etc/pgtuskmaster/tls/{member}.crt" }}, private_key = {{ path = "/etc/pgtuskmaster/tls/{member}.key" }} }} }} }}, auth = {{ type = "role_tokens", read_token = {{ path = "/run/secrets/api-read-token" }}, admin_token = {{ path = "/run/secrets/api-admin-token" }} }} }}

[pgtm]
api_url = "https://{member}:8443"

[pgtm.api_client]
ca_cert = {{ path = "/etc/pgtuskmaster/tls/ca.crt" }}

[pgtm.postgres_client]
ca_cert = {{ path = "/etc/pgtuskmaster/tls/ca.crt" }}

[debug]
enabled = true
"#
    )
}

fn render_observer_template(template: &ObserverTemplate) -> String {
    let member = template.binding.member.service_name();
    let dcs_endpoint = template.binding.dcs_service.client_url();
    let replicator = template.postgres_roles.replicator.as_str();
    let rewinder = template.postgres_roles.rewinder.as_str();
    format!(
        r#"[cluster]
name = "ha-cucumber-cluster"
member_id = "observer-{member}"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "observer"
listen_port = 5432
socket_dir = "/var/lib/pgtuskmaster/socket"
log_file = "/var/log/pgtuskmaster/postgres.log"
local_conn_identity = {{ user = "postgres", dbname = "postgres", ssl_mode = "disable" }}
rewind_conn_identity = {{ user = "{rewinder}", dbname = "postgres", ssl_mode = "verify-full", ca_cert = {{ path = "/etc/pgtuskmaster/tls/ca.crt" }} }}
tls = {{ mode = "enabled", identity = {{ cert_chain = {{ path = "/etc/pgtuskmaster/tls/observer.crt" }}, private_key = {{ path = "/etc/pgtuskmaster/tls/observer.key" }} }} }}
pg_hba = {{ source = {{ path = "/etc/pgtuskmaster/pg_hba.conf" }} }}
pg_ident = {{ source = {{ path = "/etc/pgtuskmaster/pg_ident.conf" }} }}

[postgres.roles.superuser]
username = "postgres"
auth = {{ type = "password", password = {{ path = "/run/secrets/postgres-superuser-password" }} }}

[postgres.roles.replicator]
username = "{replicator}"
auth = {{ type = "password", password = {{ path = "/run/secrets/replicator-password" }} }}

[postgres.roles.rewinder]
username = "{rewinder}"
auth = {{ type = "password", password = {{ path = "/run/secrets/rewinder-password" }} }}

[dcs]
endpoints = ["{dcs_endpoint}"]
scope = "ha-cucumber-cluster"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
pg_rewind_timeout_ms = 120000
bootstrap_timeout_ms = 300000
fencing_timeout_ms = 30000

[process.binaries]
postgres = "/usr/lib/postgresql/16/bin/postgres"
pg_ctl = "/usr/lib/postgresql/16/bin/pg_ctl"
pg_rewind = "/usr/lib/postgresql/16/bin/pg_rewind"
initdb = "/usr/lib/postgresql/16/bin/initdb"
pg_basebackup = "/usr/lib/postgresql/16/bin/pg_basebackup"
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
enabled = false
mode = "append"

[api]
listen_addr = "127.0.0.1:8443"
security = {{ transport = {{ transport = "https", tls = {{ identity = {{ cert_chain = {{ path = "/etc/pgtuskmaster/tls/observer.crt" }}, private_key = {{ path = "/etc/pgtuskmaster/tls/observer.key" }} }} }} }}, auth = {{ type = "role_tokens", read_token = {{ path = "/run/secrets/api-read-token" }}, admin_token = {{ path = "/run/secrets/api-admin-token" }} }} }}

[pgtm]
api_url = "https://{member}:8443"

[pgtm.api_client]
ca_cert = {{ path = "/etc/pgtuskmaster/tls/ca.crt" }}

[pgtm.postgres_client]
ca_cert = {{ path = "/etc/pgtuskmaster/tls/ca.crt" }}
client_cert = {{ path = "/etc/pgtuskmaster/tls/observer.crt" }}
client_key = {{ path = "/etc/pgtuskmaster/tls/observer.key" }}

[debug]
enabled = true
"#
    )
}

fn render_dcs_services(layout: ThreeNodeDcsLayout) -> String {
    match layout {
        ThreeNodeDcsLayout::SharedSingle => r#"  etcd:
    image: quay.io/coreos/etcd:v3.5.21
    command:
      - /usr/local/bin/etcd
      - --name=etcd
      - --data-dir=/etcd-data
      - --listen-client-urls=http://0.0.0.0:2379
      - --advertise-client-urls=http://etcd:2379
      - --listen-peer-urls=http://0.0.0.0:2380
      - --initial-advertise-peer-urls=http://etcd:2380
      - --initial-cluster=etcd=http://etcd:2380
      - --initial-cluster-state=new
    healthcheck:
      test:
        [
          "CMD",
          "/usr/local/bin/etcdctl",
          "--endpoints=http://127.0.0.1:2379",
          "endpoint",
          "health",
        ]
      interval: 5s
      timeout: 5s
      retries: 20
    networks:
      - ha
    volumes:
      - etcd-data:/etcd-data"#
            .to_string(),
        ThreeNodeDcsLayout::ColocatedThreeMember => {
            let initial_cluster = DcsMember::ALL
                .into_iter()
                .map(|member| format!("{member}={}", member.peer_url()))
                .collect::<Vec<_>>()
                .join(",");
            DcsMember::ALL
                .into_iter()
                .map(|member| render_three_member_dcs_service(member, initial_cluster.as_str()))
                .collect::<Vec<_>>()
                .join("\n\n")
        }
    }
}

fn render_three_member_dcs_service(member: DcsMember, initial_cluster: &str) -> String {
    let service_name = member.service_name();
    let client_url = member.client_url();
    let peer_url = member.peer_url();
    let volume_name = member.volume_name();
    format!(
        r#"  {service_name}:
    image: quay.io/coreos/etcd:v3.5.21
    command:
      - /usr/local/bin/etcd
      - --name={service_name}
      - --data-dir=/etcd-data
      - --listen-client-urls=http://0.0.0.0:2379
      - --advertise-client-urls={client_url}
      - --listen-peer-urls=http://0.0.0.0:2380
      - --initial-advertise-peer-urls={peer_url}
      - --initial-cluster={initial_cluster}
      - --initial-cluster-state=new
    healthcheck:
      test:
        [
          "CMD",
          "/usr/local/bin/etcdctl",
          "--endpoints=http://127.0.0.1:2379",
          "endpoint",
          "health",
        ]
      interval: 5s
      timeout: 5s
      retries: 20
    networks:
      - ha
    volumes:
      - {volume_name}:/etcd-data"#
    )
}

fn render_dcs_volumes(layout: ThreeNodeDcsLayout) -> String {
    match layout {
        ThreeNodeDcsLayout::SharedSingle => "  etcd-data:\n".to_string(),
        ThreeNodeDcsLayout::ColocatedThreeMember => DcsMember::ALL
            .into_iter()
            .map(|member| format!("  {}:\n", member.volume_name()))
            .collect(),
    }
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
    let discovered_member_count = status.dcs.members.len();
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
    if status.dcs.members.is_empty() {
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

fn dcs_primary_members(status: &NodeState) -> Vec<String> {
    status
        .dcs
        .members
        .iter()
        .filter(|(_member_id, slot)| matches!(slot.postgres, DcsMemberPostgresView::Primary(_)))
        .map(|(member_id, _slot)| member_id.0.clone())
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn materializes_plain_fixture_from_shared_assets_and_rendered_outputs() -> Result<()> {
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
            assert_eq!(compose.matches("NET_ADMIN").count(), 4);

            let runtime = fs::read_to_string(
                output_root.join(ClusterMember::NodeA.runtime_config_relative_path()),
            )
            .map_err(|source| HarnessError::Io {
                path: output_root.join(ClusterMember::NodeA.runtime_config_relative_path()),
                source,
            })?;
            assert!(runtime.contains(r#"username = "replicator""#));
            assert!(runtime.contains(r#"username = "rewinder""#));

            let observer = fs::read_to_string(
                output_root.join(ClusterMember::NodeA.observer_config_relative_path()),
            )
            .map_err(|source| HarnessError::Io {
                path: output_root.join(ClusterMember::NodeA.observer_config_relative_path()),
                source,
            })?;
            assert!(observer.contains(r#"api_url = "https://node-a:8443""#));
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
    fn materializes_custom_roles_without_observer_net_admin() -> Result<()> {
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
            assert_eq!(compose.matches("NET_ADMIN").count(), 3);

            let runtime = fs::read_to_string(
                output_root.join(ClusterMember::NodeB.runtime_config_relative_path()),
            )
            .map_err(|source| HarnessError::Io {
                path: output_root.join(ClusterMember::NodeB.runtime_config_relative_path()),
                source,
            })?;
            assert!(runtime.contains(r#"username = "mirrorbot""#));
            assert!(runtime.contains(r#"username = "rewindbot""#));

            let observer = fs::read_to_string(
                output_root.join(ClusterMember::NodeC.observer_config_relative_path()),
            )
            .map_err(|source| HarnessError::Io {
                path: output_root.join(ClusterMember::NodeC.observer_config_relative_path()),
                source,
            })?;
            assert!(observer.contains(r#"member_id = "observer-node-c""#));
            assert!(observer.contains(r#"username = "mirrorbot""#));
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

            let compose = fs::read_to_string(output_root.join("compose.yml")).map_err(|source| {
                HarnessError::Io {
                    path: output_root.join("compose.yml"),
                    source,
                }
            })?;
            assert!(compose.contains("etcd-a:"));
            assert!(compose.contains("etcd-b:"));
            assert!(compose.contains("etcd-c:"));
            assert!(compose.contains("etcd-a=http://etcd-a:2380"));
            assert!(compose.contains("etcd-b=http://etcd-b:2380"));
            assert!(compose.contains("etcd-c=http://etcd-c:2380"));
            assert!(compose.contains("etcd-a-data:/etcd-data"));
            assert!(compose.contains("etcd-b-data:/etcd-data"));
            assert!(compose.contains("etcd-c-data:/etcd-data"));

            let node_a_runtime =
                fs::read_to_string(output_root.join(ClusterMember::NodeA.runtime_config_relative_path()))
                    .map_err(|source| HarnessError::Io {
                        path: output_root.join(ClusterMember::NodeA.runtime_config_relative_path()),
                        source,
                    })?;
            assert!(node_a_runtime.contains(r#"endpoints = ["http://etcd-a:2379"]"#));

            let node_b_runtime =
                fs::read_to_string(output_root.join(ClusterMember::NodeB.runtime_config_relative_path()))
                    .map_err(|source| HarnessError::Io {
                        path: output_root.join(ClusterMember::NodeB.runtime_config_relative_path()),
                        source,
                    })?;
            assert!(node_b_runtime.contains(r#"endpoints = ["http://etcd-b:2379"]"#));

            let node_c_observer =
                fs::read_to_string(output_root.join(ClusterMember::NodeC.observer_config_relative_path()))
                    .map_err(|source| HarnessError::Io {
                        path: output_root.join(ClusterMember::NodeC.observer_config_relative_path()),
                        source,
                    })?;
            assert!(node_c_observer.contains(r#"endpoints = ["http://etcd-c:2379"]"#));
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
}
