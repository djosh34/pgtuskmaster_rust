use serde::{Deserialize, Serialize};

use crate::{
    api::{
        AcceptedResponse, ApiError, ApiResult, DcsTrustResponse, HaDecisionResponse,
        HaPhaseResponse, HaStateResponse, LeaseReleaseReasonResponse, RecoveryStrategyResponse,
        StepDownReasonResponse,
    },
    dcs::{
        state::{DcsTrust, SwitchoverRequest},
        store::{DcsHaWriter, DcsStore},
    },
    debug_api::snapshot::SystemSnapshot,
    ha::{
        decision::{
            HaDecision, LeaseReleaseReason, RecoveryStrategy, StepDownPlan, StepDownReason,
        },
        state::HaPhase,
    },
    state::Versioned,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SwitchoverRequestInput {}

pub(crate) fn post_switchover(
    scope: &str,
    store: &mut dyn DcsStore,
    _input: SwitchoverRequestInput,
) -> ApiResult<AcceptedResponse> {
    let request = SwitchoverRequest {};
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
        member_count: snapshot.value.dcs.value.cache.members.len(),
        dcs_trust: map_dcs_trust(&snapshot.value.dcs.value.trust),
        ha_phase: map_ha_phase(&snapshot.value.ha.value.phase),
        ha_tick: snapshot.value.ha.value.tick,
        ha_decision: map_ha_decision(&snapshot.value.ha.value.decision),
        snapshot_sequence: snapshot.value.sequence,
    }
}

fn map_dcs_trust(value: &DcsTrust) -> DcsTrustResponse {
    match value {
        DcsTrust::FullQuorum => DcsTrustResponse::FullQuorum,
        DcsTrust::FailSafe => DcsTrustResponse::FailSafe,
        DcsTrust::NotTrusted => DcsTrustResponse::NotTrusted,
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

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use crate::{
        api::controller::{delete_switchover, post_switchover, SwitchoverRequestInput},
        dcs::{
            state::SwitchoverRequest,
            store::{DcsStore, DcsStoreError, WatchEvent},
        },
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

    #[test]
    fn switchover_input_denies_unknown_fields() {
        let raw = r#"{"extra":1}"#;
        let parsed = serde_json::from_str::<SwitchoverRequestInput>(raw);
        assert!(parsed.is_err());
    }

    #[test]
    fn post_switchover_writes_typed_record_to_expected_key() -> Result<(), crate::api::ApiError> {
        let mut store = RecordingStore::default();
        let response = post_switchover("scope-a", &mut store, SwitchoverRequestInput {})?;
        assert!(response.accepted);

        let (path, raw) = store
            .pop_write()
            .ok_or_else(|| crate::api::ApiError::internal("expected one DCS write".to_string()))?;
        assert_eq!(path, "/scope-a/switchover");
        let decoded = serde_json::from_str::<SwitchoverRequest>(&raw)
            .map_err(|err| crate::api::ApiError::internal(format!("decode failed: {err}")))?;
        assert_eq!(decoded, SwitchoverRequest {});
        Ok(())
    }

    #[test]
    fn switchover_input_accepts_empty_object() -> Result<(), crate::api::ApiError> {
        let parsed = serde_json::from_str::<SwitchoverRequestInput>("{}")
            .map_err(|err| crate::api::ApiError::internal(format!("decode failed: {err}")))?;
        let mut store = RecordingStore::default();
        let result = post_switchover("scope-a", &mut store, parsed)?;
        assert!(result.accepted);
        Ok(())
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
