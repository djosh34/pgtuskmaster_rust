use serde::{Deserialize, Serialize};

use crate::{
    api::{AcceptedResponse, ApiError, ApiResult, NodeState},
    config::RuntimeConfig,
    dcs::{
        state::{
            DcsState, DcsTrust, MemberPostgresView, MemberSlot, SwitchoverIntentRecord,
            SwitchoverTargetRecord,
        },
        store::DcsStore,
    },
    ha::{state::HaState, types::AuthorityView},
    pginfo::state::{PgInfoState, Readiness},
    process::state::ProcessState,
    state::MemberId,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SwitchoverRequestInput {
    #[serde(default)]
    pub(crate) switchover_to: Option<String>,
}

pub(crate) fn post_switchover(
    scope: &str,
    self_id: &MemberId,
    store: &mut dyn DcsStore,
    dcs: &DcsState,
    ha: &HaState,
    input: SwitchoverRequestInput,
) -> ApiResult<AcceptedResponse> {
    let request = validate_switchover_request(self_id, dcs, ha, input)?;
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
    store
        .delete_path(format!("/{}/switchover", scope.trim_matches('/')).as_str())
        .map_err(|err| ApiError::DcsStore(err.to_string()))?;
    Ok(AcceptedResponse { accepted: true })
}

pub(crate) fn build_node_state(
    cfg: &RuntimeConfig,
    pg: &PgInfoState,
    process: &ProcessState,
    dcs: &DcsState,
    ha: &HaState,
) -> NodeState {
    NodeState {
        cluster_name: cfg.cluster.name.clone(),
        scope: cfg.dcs.scope.clone(),
        self_member_id: cfg.cluster.member_id.clone(),
        pg: pg.clone(),
        process: process.clone(),
        dcs: dcs.clone(),
        ha: ha.clone(),
    }
}

fn validate_switchover_request(
    self_id: &MemberId,
    dcs: &DcsState,
    ha: &HaState,
    input: SwitchoverRequestInput,
) -> ApiResult<SwitchoverIntentRecord> {
    if dcs.trust != DcsTrust::FullQuorum {
        return Err(ApiError::bad_request(
            "switchover requests require full quorum DCS trust".to_string(),
        ));
    }

    match &ha.publication.authority {
        AuthorityView::Primary { member, .. } if member == self_id => {}
        _ => {
            return Err(ApiError::bad_request(
                "switchover requests must be sent to the authoritative primary".to_string(),
            ));
        }
    }

    let Some(raw_target) = input.switchover_to else {
        let target_member_id = select_generic_switchover_target(self_id, dcs).ok_or_else(|| {
            ApiError::bad_request(
                "no eligible switchover target is currently available".to_string(),
            )
        })?;
        return Ok(SwitchoverIntentRecord {
            target: SwitchoverTargetRecord::Specific(target_member_id),
        });
    };

    let target = raw_target.trim();
    if target.is_empty() {
        return Err(ApiError::bad_request(
            "switchover_to must not be empty".to_string(),
        ));
    }

    let target_member_id = MemberId(target.to_string());
    if &target_member_id == self_id {
        return Err(ApiError::bad_request(format!(
            "switchover_to member `{target}` is already the leader"
        )));
    }

    let target_member = dcs
        .cache
        .member_slots
        .get(&target_member_id)
        .ok_or_else(|| ApiError::bad_request(format!("unknown switchover_to member `{target}`")))?;
    if !member_slot_is_eligible_target(target_member) {
        return Err(ApiError::bad_request(format!(
            "switchover_to member `{target}` is not an eligible switchover target"
        )));
    }

    Ok(SwitchoverIntentRecord {
        target: SwitchoverTargetRecord::Specific(target_member_id),
    })
}

fn member_slot_is_eligible_target(value: &MemberSlot) -> bool {
    match &value.postgres {
        MemberPostgresView::Unknown(observation) => observation.readiness == Readiness::Ready,
        MemberPostgresView::Primary(_) => false,
        MemberPostgresView::Replica(observation) => observation.readiness == Readiness::Ready,
    }
}

fn select_generic_switchover_target(self_id: &MemberId, dcs: &DcsState) -> Option<MemberId> {
    dcs.cache
        .member_slots
        .iter()
        .filter(|(member_id, member)| {
            *member_id != self_id && member_slot_is_eligible_target(member)
        })
        .max_by(|(left_id, left_member), (right_id, right_member)| {
            compare_switchover_target_slots(left_id, left_member, right_id, right_member)
        })
        .map(|(member_id, _)| member_id.clone())
}

fn compare_switchover_target_slots(
    left_id: &MemberId,
    left_member: &MemberSlot,
    right_id: &MemberId,
    right_member: &MemberSlot,
) -> std::cmp::Ordering {
    target_rank(left_member)
        .cmp(&target_rank(right_member))
        .then_with(|| right_id.cmp(left_id))
}

fn target_rank(member: &MemberSlot) -> (u8, u64, u64) {
    match &member.postgres {
        MemberPostgresView::Replica(observation) => observation
            .replay_wal
            .as_ref()
            .or(observation.follow_wal.as_ref())
            .map(|wal| {
                (
                    1,
                    wal.timeline.map_or(0, |value| u64::from(value.0)),
                    wal.lsn.0,
                )
            })
            .unwrap_or((0, 0, 0)),
        MemberPostgresView::Unknown(observation) => (
            0,
            observation.timeline.map_or(0, |value| u64::from(value.0)),
            0,
        ),
        MemberPostgresView::Primary(_) => (0, 0, 0),
    }
}
