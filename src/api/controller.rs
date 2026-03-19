use crate::{
    api::{AcceptedResponse, ApiError, ApiResult},
    dcs::{DcsHandle, DcsSnapshot},
    ha::{
        state::HaState,
        types::{AuthorityProjection, PublicationState},
    },
    pginfo::state::Readiness,
    state::{MemberId, SwitchoverState},
};

pub(crate) type SwitchoverRequest = SwitchoverState;

pub(crate) async fn post_switchover(
    _scope: &str,
    self_id: &MemberId,
    handle: &DcsHandle,
    dcs: &DcsSnapshot,
    ha: &HaState,
    input: SwitchoverRequest,
) -> ApiResult<AcceptedResponse> {
    if !dcs.is_quorum() {
        return Err(ApiError::bad_request(
            "switchover requests require coordinated DCS state".to_string(),
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

    let target = match input {
        SwitchoverState::None => {
            return Err(ApiError::bad_request(
                "switchover request must choose a target state".to_string(),
            ));
        }
        SwitchoverState::AnyHealthyReplica => SwitchoverState::AnyHealthyReplica,
        SwitchoverState::Specific(target_member_id) => {
            let target = target_member_id.0.trim().to_string();
            if &target_member_id == self_id {
                return Err(ApiError::bad_request(format!(
                    "switchover_to member `{target}` is already the leader"
                )));
            }

            let target_member = dcs.member(&target_member_id).ok_or_else(|| {
                ApiError::bad_request(format!("unknown switchover_to member `{target}`"))
            })?;
            let postgres = target_member.postgres();
            if postgres.readiness() != Readiness::Ready || postgres.is_primary() {
                return Err(ApiError::bad_request(format!(
                    "switchover_to member `{target}` is not an eligible switchover target"
                )));
            }

            SwitchoverState::Specific(target_member_id)
        }
    };
    handle
        .publish_switchover(target)
        .map_err(|err| ApiError::DcsCommand(err.to_string()))?;

    Ok(AcceptedResponse { accepted: true })
}

pub(crate) async fn delete_switchover(
    _scope: &str,
    handle: &DcsHandle,
) -> ApiResult<AcceptedResponse> {
    handle
        .clear_switchover()
        .map_err(|err| ApiError::DcsCommand(err.to_string()))?;
    Ok(AcceptedResponse { accepted: true })
}
