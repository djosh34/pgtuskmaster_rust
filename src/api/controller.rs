use serde::{Deserialize, Serialize};

use crate::{
    api::{
        AcceptedResponse, ApiError, ApiResult, DcsTrustResponse, HaClusterMemberResponse,
        HaDecisionResponse, HaPhaseResponse, HaStateResponse, LeaseReleaseReasonResponse,
        MemberRoleResponse, ReadinessResponse, RecoveryStrategyResponse, SqlStatusResponse,
        StepDownReasonResponse,
    },
    dcs::{
        state::{DcsTrust, MemberRecord, MemberRole, SwitchoverRequest},
        store::{DcsHaWriter, DcsStore},
    },
    debug_api::snapshot::SystemSnapshot,
    ha::{
        decision::{
            eligible_switchover_targets, HaDecision, LeaseReleaseReason, RecoveryStrategy,
            StepDownPlan, StepDownReason,
        },
        state::HaPhase,
    },
    state::Versioned,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SwitchoverRequestInput {
    #[serde(default)]
    pub(crate) switchover_to: Option<String>,
}

pub(crate) fn post_switchover(
    scope: &str,
    store: &mut dyn DcsStore,
    snapshot: Option<&SystemSnapshot>,
    input: SwitchoverRequestInput,
) -> ApiResult<AcceptedResponse> {
    let request = validate_switchover_request(snapshot, input)?;
    let encoded = serde_json::to_string(&request)
        .map_err(|err| ApiError::internal(format!("switchover encode failed: {err}")))?;

    let path = format!("/{}/switchover", scope.trim_matches('/'));
    store
        .write_path(&path, encoded)
        .map_err(|err| ApiError::DcsStore(err.to_string()))?;

    Ok(AcceptedResponse { accepted: true })
}

pub(crate) fn delete_switchover(
    scope: &str,
    store: &mut dyn DcsStore,
) -> ApiResult<AcceptedResponse> {
    DcsHaWriter::clear_switchover(store, scope)
        .map_err(|err| ApiError::DcsStore(err.to_string()))?;
    Ok(AcceptedResponse { accepted: true })
}

pub(crate) fn get_ha_state(snapshot: &Versioned<SystemSnapshot>) -> HaStateResponse {
    HaStateResponse {
        cluster_name: snapshot.value.config.value.cluster.name.clone(),
        scope: snapshot.value.config.value.dcs.scope.clone(),
        self_member_id: snapshot.value.config.value.cluster.member_id.clone(),
        leader: snapshot
            .value
            .dcs
            .value
            .cache
            .leader
            .as_ref()
            .map(|leader| leader.member_id.0.clone()),
        switchover_pending: snapshot.value.dcs.value.cache.switchover.is_some(),
        switchover_to: snapshot
            .value
            .dcs
            .value
            .cache
            .switchover
            .as_ref()
            .and_then(|request| {
                request
                    .switchover_to
                    .as_ref()
                    .map(|member_id| member_id.0.clone())
            }),
        member_count: snapshot.value.dcs.value.cache.members.len(),
        members: snapshot
            .value
            .dcs
            .value
            .cache
            .members
            .values()
            .map(map_member_record)
            .collect(),
        dcs_trust: map_dcs_trust(&snapshot.value.dcs.value.trust),
        ha_phase: map_ha_phase(&snapshot.value.ha.value.phase),
        ha_tick: snapshot.value.ha.value.tick,
        ha_decision: map_ha_decision(&snapshot.value.ha.value.decision),
        snapshot_sequence: snapshot.value.sequence,
    }
}

fn validate_switchover_request(
    snapshot: Option<&SystemSnapshot>,
    input: SwitchoverRequestInput,
) -> ApiResult<SwitchoverRequest> {
    let Some(raw_target) = input.switchover_to else {
        return Ok(SwitchoverRequest {
            switchover_to: None,
        });
    };
    let snapshot =
        snapshot.ok_or_else(|| ApiError::DcsStore("snapshot unavailable".to_string()))?;

    let target = raw_target.trim();
    if target.is_empty() {
        return Err(ApiError::bad_request(
            "switchover_to must not be empty".to_string(),
        ));
    }

    let target_member_id = crate::state::MemberId(target.to_string());
    let members = &snapshot.dcs.value.cache.members;
    if !members.contains_key(&target_member_id) {
        return Err(ApiError::bad_request(format!(
            "unknown switchover_to member `{target}`"
        )));
    }

    if snapshot
        .dcs
        .value
        .cache
        .leader
        .as_ref()
        .map(|leader| leader.member_id == target_member_id)
        .unwrap_or(false)
    {
        return Err(ApiError::bad_request(format!(
            "switchover_to member `{target}` is already the leader"
        )));
    }

    let self_member_id = crate::state::MemberId(snapshot.config.value.cluster.member_id.clone());
    let eligible_targets = eligible_switchover_targets(
        &crate::ha::state::WorldSnapshot {
            config: snapshot.config.clone(),
            pg: snapshot.pg.clone(),
            dcs: snapshot.dcs.clone(),
            process: snapshot.process.clone(),
        },
        &self_member_id,
    );
    if !eligible_targets.contains(&target_member_id) {
        return Err(ApiError::bad_request(format!(
            "switchover_to member `{target}` is not an eligible switchover target"
        )));
    }

    Ok(SwitchoverRequest {
        switchover_to: Some(target_member_id),
    })
}

fn map_dcs_trust(value: &DcsTrust) -> DcsTrustResponse {
    match value {
        DcsTrust::FullQuorum => DcsTrustResponse::FullQuorum,
        DcsTrust::FailSafe => DcsTrustResponse::FailSafe,
        DcsTrust::NotTrusted => DcsTrustResponse::NotTrusted,
    }
}

fn map_member_record(value: &MemberRecord) -> HaClusterMemberResponse {
    HaClusterMemberResponse {
        member_id: value.member_id.0.clone(),
        postgres_host: value.postgres_host.clone(),
        postgres_port: value.postgres_port,
        api_url: value.api_url.clone(),
        role: map_member_role(&value.role),
        sql: map_sql_status(&value.sql),
        readiness: map_readiness(&value.readiness),
        timeline: value.timeline.map(|timeline| u64::from(timeline.0)),
        write_lsn: value.write_lsn.map(|lsn| lsn.0),
        replay_lsn: value.replay_lsn.map(|lsn| lsn.0),
        updated_at_ms: value.updated_at.0,
        pg_version: value.pg_version.0,
    }
}

fn map_ha_phase(value: &HaPhase) -> HaPhaseResponse {
    match value {
        HaPhase::Init => HaPhaseResponse::Init,
        HaPhase::WaitingPostgresReachable => HaPhaseResponse::WaitingPostgresReachable,
        HaPhase::WaitingDcsTrusted => HaPhaseResponse::WaitingDcsTrusted,
        HaPhase::WaitingSwitchoverSuccessor => HaPhaseResponse::WaitingSwitchoverSuccessor,
        HaPhase::Replica => HaPhaseResponse::Replica,
        HaPhase::CandidateLeader => HaPhaseResponse::CandidateLeader,
        HaPhase::Primary => HaPhaseResponse::Primary,
        HaPhase::Rewinding => HaPhaseResponse::Rewinding,
        HaPhase::Bootstrapping => HaPhaseResponse::Bootstrapping,
        HaPhase::Fencing => HaPhaseResponse::Fencing,
        HaPhase::FailSafe => HaPhaseResponse::FailSafe,
    }
}

fn map_ha_decision(value: &HaDecision) -> HaDecisionResponse {
    match value {
        HaDecision::NoChange => HaDecisionResponse::NoChange,
        HaDecision::WaitForPostgres {
            start_requested,
            leader_member_id,
        } => HaDecisionResponse::WaitForPostgres {
            start_requested: *start_requested,
            leader_member_id: leader_member_id.as_ref().map(|leader| leader.0.clone()),
        },
        HaDecision::WaitForDcsTrust => HaDecisionResponse::WaitForDcsTrust,
        HaDecision::AttemptLeadership => HaDecisionResponse::AttemptLeadership,
        HaDecision::FollowLeader { leader_member_id } => HaDecisionResponse::FollowLeader {
            leader_member_id: leader_member_id.0.clone(),
        },
        HaDecision::BecomePrimary { promote } => {
            HaDecisionResponse::BecomePrimary { promote: *promote }
        }
        HaDecision::StepDown(plan) => map_step_down_plan(plan),
        HaDecision::RecoverReplica { strategy } => HaDecisionResponse::RecoverReplica {
            strategy: map_recovery_strategy(strategy),
        },
        HaDecision::FenceNode => HaDecisionResponse::FenceNode,
        HaDecision::ReleaseLeaderLease { reason } => HaDecisionResponse::ReleaseLeaderLease {
            reason: map_lease_release_reason(reason),
        },
        HaDecision::EnterFailSafe {
            release_leader_lease,
        } => HaDecisionResponse::EnterFailSafe {
            release_leader_lease: *release_leader_lease,
        },
    }
}

fn map_step_down_plan(value: &StepDownPlan) -> HaDecisionResponse {
    HaDecisionResponse::StepDown {
        reason: map_step_down_reason(&value.reason),
        release_leader_lease: value.release_leader_lease,
        clear_switchover: value.clear_switchover,
        fence: value.fence,
    }
}

fn map_step_down_reason(value: &StepDownReason) -> StepDownReasonResponse {
    match value {
        StepDownReason::Switchover => StepDownReasonResponse::Switchover,
        StepDownReason::ForeignLeaderDetected { leader_member_id } => {
            StepDownReasonResponse::ForeignLeaderDetected {
                leader_member_id: leader_member_id.0.clone(),
            }
        }
    }
}

fn map_recovery_strategy(value: &RecoveryStrategy) -> RecoveryStrategyResponse {
    match value {
        RecoveryStrategy::Rewind { leader_member_id } => RecoveryStrategyResponse::Rewind {
            leader_member_id: leader_member_id.0.clone(),
        },
        RecoveryStrategy::BaseBackup { leader_member_id } => RecoveryStrategyResponse::BaseBackup {
            leader_member_id: leader_member_id.0.clone(),
        },
        RecoveryStrategy::Bootstrap => RecoveryStrategyResponse::Bootstrap,
    }
}

fn map_lease_release_reason(value: &LeaseReleaseReason) -> LeaseReleaseReasonResponse {
    match value {
        LeaseReleaseReason::FencingComplete => LeaseReleaseReasonResponse::FencingComplete,
        LeaseReleaseReason::PostgresUnreachable => LeaseReleaseReasonResponse::PostgresUnreachable,
    }
}

fn map_member_role(value: &MemberRole) -> MemberRoleResponse {
    match value {
        MemberRole::Unknown => MemberRoleResponse::Unknown,
        MemberRole::Primary => MemberRoleResponse::Primary,
        MemberRole::Replica => MemberRoleResponse::Replica,
    }
}

fn map_sql_status(value: &crate::pginfo::state::SqlStatus) -> SqlStatusResponse {
    match value {
        crate::pginfo::state::SqlStatus::Unknown => SqlStatusResponse::Unknown,
        crate::pginfo::state::SqlStatus::Healthy => SqlStatusResponse::Healthy,
        crate::pginfo::state::SqlStatus::Unreachable => SqlStatusResponse::Unreachable,
    }
}

fn map_readiness(value: &crate::pginfo::state::Readiness) -> ReadinessResponse {
    match value {
        crate::pginfo::state::Readiness::Unknown => ReadinessResponse::Unknown,
        crate::pginfo::state::Readiness::Ready => ReadinessResponse::Ready,
        crate::pginfo::state::Readiness::NotReady => ReadinessResponse::NotReady,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, VecDeque};

    use crate::{
        api::controller::{delete_switchover, post_switchover, SwitchoverRequestInput},
        dcs::{
            state::{
                DcsCache, DcsState, DcsTrust, LeaderRecord, MemberRecord, MemberRole,
                SwitchoverRequest,
            },
            store::{DcsStore, DcsStoreError, WatchEvent},
        },
        debug_api::snapshot::{AppLifecycle, SystemSnapshot},
        ha::{
            decision::HaDecision,
            state::{HaPhase, HaState},
        },
        pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
        process::state::ProcessState,
        state::{MemberId, UnixMillis, Version, Versioned, WorkerStatus},
    };

    #[derive(Default)]
    struct RecordingStore {
        writes: VecDeque<(String, String)>,
        deletes: VecDeque<String>,
    }

    impl RecordingStore {
        fn pop_write(&mut self) -> Option<(String, String)> {
            self.writes.pop_front()
        }

        fn pop_delete(&mut self) -> Option<String> {
            self.deletes.pop_front()
        }
    }

    impl DcsStore for RecordingStore {
        fn healthy(&self) -> bool {
            true
        }

        fn read_path(&mut self, _path: &str) -> Result<Option<String>, DcsStoreError> {
            Ok(None)
        }

        fn write_path(&mut self, path: &str, value: String) -> Result<(), DcsStoreError> {
            self.writes.push_back((path.to_string(), value));
            Ok(())
        }

        fn put_path_if_absent(&mut self, path: &str, value: String) -> Result<bool, DcsStoreError> {
            self.writes.push_back((path.to_string(), value));
            Ok(true)
        }

        fn delete_path(&mut self, path: &str) -> Result<(), DcsStoreError> {
            self.deletes.push_back(path.to_string());
            Ok(())
        }

        fn drain_watch_events(&mut self) -> Result<Vec<WatchEvent>, DcsStoreError> {
            Ok(Vec::new())
        }
    }

    fn sample_snapshot() -> SystemSnapshot {
        let cfg = crate::test_harness::runtime_config::sample_runtime_config();
        let members = BTreeMap::from([
            (
                member_id("node-a"),
                member_record("node-a", MemberRole::Primary),
            ),
            (
                member_id("node-b"),
                member_record("node-b", MemberRole::Replica),
            ),
            (
                member_id("node-c"),
                member_record("node-c", MemberRole::Replica),
            ),
        ]);

        SystemSnapshot {
            app: AppLifecycle::Running,
            config: Versioned::new(Version(1), UnixMillis(1), cfg.clone()),
            pg: Versioned::new(
                Version(1),
                UnixMillis(1),
                PgInfoState::Primary {
                    common: pg_common(SqlStatus::Healthy),
                    wal_lsn: crate::state::WalLsn(10),
                    slots: Vec::new(),
                },
            ),
            dcs: Versioned::new(
                Version(1),
                UnixMillis(1),
                DcsState {
                    worker: WorkerStatus::Running,
                    trust: DcsTrust::FullQuorum,
                    cache: DcsCache {
                        members,
                        leader: Some(LeaderRecord {
                            member_id: member_id("node-a"),
                        }),
                        switchover: None,
                        config: cfg,
                        init_lock: None,
                    },
                    last_refresh_at: Some(UnixMillis(1)),
                },
            ),
            process: Versioned::new(
                Version(1),
                UnixMillis(1),
                ProcessState::Idle {
                    worker: WorkerStatus::Running,
                    last_outcome: None,
                },
            ),
            ha: Versioned::new(
                Version(1),
                UnixMillis(1),
                HaState {
                    worker: WorkerStatus::Running,
                    phase: HaPhase::Primary,
                    tick: 1,
                    decision: HaDecision::NoChange,
                },
            ),
            generated_at: UnixMillis(1),
            sequence: 1,
            changes: Vec::new(),
            timeline: Vec::new(),
        }
    }

    fn member_id(value: &str) -> MemberId {
        MemberId(value.to_string())
    }

    fn member_record(member_name: &str, role: MemberRole) -> MemberRecord {
        MemberRecord {
            member_id: member_id(member_name),
            postgres_host: "127.0.0.1".to_string(),
            postgres_port: 5432,
            api_url: None,
            role,
            sql: SqlStatus::Healthy,
            readiness: Readiness::Ready,
            timeline: None,
            write_lsn: None,
            replay_lsn: None,
            updated_at: UnixMillis(1),
            pg_version: Version(1),
        }
    }

    fn pg_common(sql: SqlStatus) -> PgInfoCommon {
        PgInfoCommon {
            worker: WorkerStatus::Running,
            sql,
            readiness: Readiness::Ready,
            timeline: None,
            pg_config: PgConfig {
                port: None,
                hot_standby: None,
                primary_conninfo: None,
                primary_slot_name: None,
                extra: BTreeMap::new(),
            },
            last_refresh_at: Some(UnixMillis(1)),
        }
    }

    #[test]
    fn switchover_input_denies_unknown_fields() {
        let raw = r#"{"extra":1}"#;
        let parsed = serde_json::from_str::<SwitchoverRequestInput>(raw);
        assert!(parsed.is_err());
    }

    #[test]
    fn post_switchover_writes_typed_record_to_expected_key() -> Result<(), crate::api::ApiError> {
        let mut store = RecordingStore::default();
        let snapshot = sample_snapshot();
        let response = post_switchover(
            "scope-a",
            &mut store,
            Some(&snapshot),
            SwitchoverRequestInput {
                switchover_to: None,
            },
        )?;
        assert!(response.accepted);

        let (path, raw) = store
            .pop_write()
            .ok_or_else(|| crate::api::ApiError::internal("expected one DCS write".to_string()))?;
        assert_eq!(path, "/scope-a/switchover");
        let decoded = serde_json::from_str::<SwitchoverRequest>(&raw)
            .map_err(|err| crate::api::ApiError::internal(format!("decode failed: {err}")))?;
        assert_eq!(
            decoded,
            SwitchoverRequest {
                switchover_to: None
            }
        );
        Ok(())
    }

    #[test]
    fn switchover_input_accepts_empty_object() -> Result<(), crate::api::ApiError> {
        let parsed = serde_json::from_str::<SwitchoverRequestInput>("{}")
            .map_err(|err| crate::api::ApiError::internal(format!("decode failed: {err}")))?;
        let mut store = RecordingStore::default();
        let snapshot = sample_snapshot();
        let result = post_switchover("scope-a", &mut store, Some(&snapshot), parsed)?;
        assert!(result.accepted);
        Ok(())
    }

    #[test]
    fn switchover_input_accepts_targeted_request() -> Result<(), crate::api::ApiError> {
        let parsed =
            serde_json::from_str::<SwitchoverRequestInput>(r#"{"switchover_to":"node-b"}"#)
                .map_err(|err| crate::api::ApiError::internal(format!("decode failed: {err}")))?;
        let snapshot = sample_snapshot();
        let mut store = RecordingStore::default();
        let result = post_switchover("scope-a", &mut store, Some(&snapshot), parsed)?;
        assert!(result.accepted);

        let (_path, raw) = store
            .pop_write()
            .ok_or_else(|| crate::api::ApiError::internal("expected one DCS write".to_string()))?;
        let decoded = serde_json::from_str::<SwitchoverRequest>(&raw)
            .map_err(|err| crate::api::ApiError::internal(format!("decode failed: {err}")))?;
        assert_eq!(
            decoded,
            SwitchoverRequest {
                switchover_to: Some(member_id("node-b"))
            }
        );
        Ok(())
    }

    #[test]
    fn switchover_input_rejects_unknown_target() {
        let parsed =
            serde_json::from_str::<SwitchoverRequestInput>(r#"{"switchover_to":"node-z"}"#);
        assert!(parsed.is_ok());

        let snapshot = sample_snapshot();
        let mut store = RecordingStore::default();
        let result = post_switchover(
            "scope-a",
            &mut store,
            Some(&snapshot),
            parsed.unwrap_or(SwitchoverRequestInput {
                switchover_to: None,
            }),
        );
        assert!(matches!(
            result,
            Err(crate::api::ApiError::BadRequest(message))
                if message.contains("unknown switchover_to member")
        ));
    }

    #[test]
    fn switchover_input_rejects_empty_target() {
        let snapshot = sample_snapshot();
        let mut store = RecordingStore::default();
        let result = post_switchover(
            "scope-a",
            &mut store,
            Some(&snapshot),
            SwitchoverRequestInput {
                switchover_to: Some("   ".to_string()),
            },
        );
        assert!(matches!(
            result,
            Err(crate::api::ApiError::BadRequest(message))
                if message.contains("must not be empty")
        ));
    }

    #[test]
    fn switchover_input_rejects_ineligible_target() {
        let mut snapshot = sample_snapshot();
        snapshot.dcs.value.cache.members.insert(
            member_id("node-z"),
            MemberRecord {
                member_id: member_id("node-z"),
                postgres_host: "127.0.0.1".to_string(),
                postgres_port: 5432,
                api_url: None,
                role: MemberRole::Unknown,
                sql: SqlStatus::Healthy,
                readiness: Readiness::Ready,
                timeline: None,
                write_lsn: None,
                replay_lsn: None,
                updated_at: UnixMillis(1),
                pg_version: Version(1),
            },
        );
        let mut store = RecordingStore::default();
        let result = post_switchover(
            "scope-a",
            &mut store,
            Some(&snapshot),
            SwitchoverRequestInput {
                switchover_to: Some("node-z".to_string()),
            },
        );
        assert!(matches!(
            result,
            Err(crate::api::ApiError::BadRequest(message))
                if message.contains("not an eligible switchover target")
        ));
    }

    #[test]
    fn delete_switchover_deletes_expected_key() -> Result<(), crate::api::ApiError> {
        let mut store = RecordingStore::default();
        let response = delete_switchover("scope-a", &mut store)?;
        assert!(response.accepted);
        assert_eq!(store.pop_delete().as_deref(), Some("/scope-a/switchover"));
        Ok(())
    }
}
