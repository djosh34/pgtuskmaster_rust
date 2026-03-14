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
    ha::types::AuthorityView,
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
    givens::given_root,
    observer::{
        pgtm::PgtmObserver,
        sql::SqlObserver,
    },
    timeouts::TimeoutModel,
    topology::{ClusterMember, ComposeService, SupportService},
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

    pub fn remember_alias(
        &mut self,
        alias: impl Into<MemberAlias>,
        member: ClusterMember,
    ) {
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
        self.scenario.workload.proof.convergence_blocked_members.clear();
    }

    pub fn clear_primary_history(&mut self) {
        self.scenario.invariants.observed_authoritative_primaries.clear();
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
pub struct HarnessShared {
    pub run_id: String,
    pub feature_name: String,
    pub given_name: String,
    pub run_dir: PathBuf,
    pub source_copy_dir: PathBuf,
    pub artifacts_dir: PathBuf,
    pub compose_file: PathBuf,
    pub compose_project: String,
    pub cucumber_test_image_run_id: String,
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
        let cucumber_test_image_run_id = required_env("PGTM_CUCUMBER_TEST_RUN_ID")?;
        let run_dir = repo_root
            .join("tests/ha/runs")
            .join(feature.feature_name.as_str())
            .join(run_id.as_str());
        let source_copy_dir = run_dir.join("source-copy");
        let artifacts_dir = run_dir.join("artifacts");
        create_dir_all(run_dir.as_path())?;
        create_dir_all(source_copy_dir.as_path())?;
        create_dir_all(artifacts_dir.as_path())?;
        copy_directory(given_root.as_path(), source_copy_dir.as_path())?;
        create_fault_directories(source_copy_dir.as_path())?;

        let compose_file = source_copy_dir.join("compose.yml");
        let timeouts = TimeoutModel::from_runtime_config(
            source_copy_dir
                .join(ClusterMember::SEED_PRIMARY.runtime_config_relative_path())
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
            self.compose_file.as_path(),
            self.compose_project.as_str(),
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
        let path = self.artifacts_dir.join(artifact_name);
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
            format!(
                "member={member} path={} peer={peer_service}",
                path.label()
            ),
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
            format!(
                "member={member} path={} peer={peer_service}",
                path.label()
            ),
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
        self.block_member_path_to_host(
            member,
            TrafficPath::Api,
            SupportService::Observer.into(),
        )
    }

    pub fn cut_member_off_from_dcs(&self, member: ClusterMember) -> Result<()> {
        self.block_member_path_to_host(member, TrafficPath::Dcs, SupportService::Etcd.into())
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
            .chain([SupportService::Observer.into()].into_iter())
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
        self.source_copy_dir.join("faults").join(member.service_name())
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
        self.wait_for_service_health(SupportService::Etcd.into()).await?;
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
            .compose_down(self.compose_file.as_path(), self.compose_project.as_str());
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
        match self.observer().state() {
            Ok(state) => write_text_file(
                self.artifacts_dir.join("observer-state.json").as_path(),
                serde_json::to_string_pretty(&state)
                    .map_err(|source| HarnessError::Json {
                        context: "serializing observer state payload".to_string(),
                        source,
                    })?
                    .as_str(),
            )?,
            Err(err) => failures.push(format!("observer state capture failed: {err}")),
        }

        for service in [
            SupportService::Observer.into(),
            ClusterMember::NodeA.into(),
            ClusterMember::NodeB.into(),
            ClusterMember::NodeC.into(),
            SupportService::Etcd.into(),
        ] {
            match self.service_container_id(service) {
                Ok(container_id) => match self.docker.inspect_container(container_id.as_str()) {
                    Ok(inspect) => {
                        let artifact = self
                            .artifacts_dir
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
    match &status.ha.publication.authority {
        AuthorityView::Primary { member, .. } => Some(member.0.clone()),
        AuthorityView::NoPrimary(_) | AuthorityView::Unknown => None,
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
