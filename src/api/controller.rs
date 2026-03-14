use crate::{
    api::{AcceptedResponse, ApiError, ApiResult, NodeState},
    config::RuntimeConfig,
    dcs::{
        DcsHandle, DcsMemberPostgresView, DcsMemberView, DcsSwitchoverTargetView,
        DcsSwitchoverView, DcsTrust, DcsView,
    },
    ha::{
        state::HaState,
        types::{AuthorityProjection, PublicationState},
    },
    pginfo::state::{PgInfoState, Readiness},
    process::state::ProcessState,
    state::MemberId,
};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SwitchoverRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) switchover_to: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NodeStateSnapshot {
    pub(crate) cluster_name: String,
    pub(crate) scope: String,
    pub(crate) self_member_id: String,
    pub(crate) pg: PgInfoState,
    pub(crate) process: ProcessState,
    pub(crate) dcs: DcsView,
    pub(crate) ha: HaState,
}

pub(crate) async fn post_switchover(
    _scope: &str,
    self_id: &MemberId,
    handle: &DcsHandle,
    dcs: &DcsView,
    ha: &HaState,
    input: SwitchoverRequest,
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

pub(crate) fn build_node_state(_cfg: &RuntimeConfig, snapshot: NodeStateSnapshot) -> NodeState {
    NodeState {
        cluster_name: snapshot.cluster_name,
        scope: snapshot.scope,
        self_member_id: snapshot.self_member_id,
        pg: snapshot.pg,
        process: snapshot.process,
        dcs: snapshot.dcs,
        ha: snapshot.ha,
    }
}

fn validate_switchover_request(
    self_id: &MemberId,
    dcs: &DcsView,
    ha: &HaState,
    input: SwitchoverRequest,
) -> ApiResult<DcsSwitchoverView> {
    if dcs.trust != DcsTrust::FullQuorum {
        return Err(ApiError::bad_request(
            "switchover requests require full quorum DCS trust".to_string(),
        ));
    }

    match &ha.publication {
        PublicationState::Projected(AuthorityProjection::Primary(epoch))
            if epoch.holder == *self_id => {}
        _ => {
            return Err(ApiError::bad_request(
                "switchover requests must be sent to the authoritative primary".to_string(),
            ));
        }
    }

    let target = match input.switchover_to {
        None => {
            return Ok(DcsSwitchoverView {
                target: DcsSwitchoverTargetView::AnyHealthyReplica,
            });
        }
        Some(member_id) => member_id,
    };
    let target = target.trim();
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
