use serde::{Deserialize, Serialize};

use crate::{
    api::{AcceptedResponse, ApiError, ApiResult, NodeState},
    config::RuntimeConfig,
    dcs::{
        DcsHandle, DcsMemberPostgresView, DcsMemberView, DcsSwitchoverTargetView,
        DcsSwitchoverView, DcsTrust, DcsView,
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

pub(crate) async fn post_switchover(
    _scope: &str,
    self_id: &MemberId,
    handle: &DcsHandle,
    dcs: &DcsView,
    ha: &HaState,
    input: SwitchoverRequestInput,
) -> ApiResult<AcceptedResponse> {
    let request = validate_switchover_request(self_id, dcs, ha, input)?;
    handle
        .publish_switchover(request.target)
        .await
        .map_err(|err| ApiError::DcsCommand(err.to_string()))?;

    Ok(AcceptedResponse { accepted: true })
}

pub(crate) async fn delete_switchover(
    _scope: &str,
    handle: &DcsHandle,
) -> ApiResult<AcceptedResponse> {
    handle
        .clear_switchover()
        .await
        .map_err(|err| ApiError::DcsCommand(err.to_string()))?;
    Ok(AcceptedResponse { accepted: true })
}

pub(crate) fn build_node_state(
    cfg: &RuntimeConfig,
    pg: &PgInfoState,
    process: &ProcessState,
    dcs: &DcsView,
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
    dcs: &DcsView,
    ha: &HaState,
    input: SwitchoverRequestInput,
) -> ApiResult<DcsSwitchoverView> {
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
        return Ok(DcsSwitchoverView {
            target: DcsSwitchoverTargetView::AnyHealthyReplica,
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
        .members
        .get(&target_member_id)
        .ok_or_else(|| ApiError::bad_request(format!("unknown switchover_to member `{target}`")))?;
    if !member_slot_is_eligible_target(target_member) {
        return Err(ApiError::bad_request(format!(
            "switchover_to member `{target}` is not an eligible switchover target"
        )));
    }

    Ok(DcsSwitchoverView {
        target: DcsSwitchoverTargetView::Specific(target_member_id),
    })
}

fn member_slot_is_eligible_target(value: &DcsMemberView) -> bool {
    match &value.postgres {
        DcsMemberPostgresView::Unknown(observation) => observation.readiness == Readiness::Ready,
        DcsMemberPostgresView::Primary(_) => false,
        DcsMemberPostgresView::Replica(observation) => observation.readiness == Readiness::Ready,
    }
}
