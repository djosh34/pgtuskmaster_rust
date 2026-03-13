use crate::{
    api::NodeState,
    cli::{
        client::CliApiClient,
        config::OperatorContext,
        error::CliError,
        output,
        status::{authority_primary_member, fetch_seed_state, member_is_ready_replica},
    },
    dcs::state::DcsTrust,
    ha::types::AuthorityView,
    state::MemberId,
};

pub(crate) async fn run_clear(context: &OperatorContext, json: bool) -> Result<String, CliError> {
    let client = CliApiClient::from_config(context.api_client.clone())?;
    let response = client.delete_switchover().await?;
    output::render_accepted_output(&response, json)
}

pub(crate) async fn run_request(
    context: &OperatorContext,
    json: bool,
    switchover_to: Option<String>,
) -> Result<String, CliError> {
    let (state, _queried_via) = fetch_seed_state(context).await?;
    validate_switchover_request(&state, switchover_to.as_deref())?;

    let client = CliApiClient::from_config(context.api_client.clone())?;
    let response = client.post_switchover(switchover_to).await?;
    output::render_accepted_output(&response, json)
}

fn validate_switchover_request(
    state: &NodeState,
    switchover_to: Option<&str>,
) -> Result<(), CliError> {
    if state.dcs.trust != DcsTrust::FullQuorum {
        return Err(CliError::Resolution(format!(
            "cannot request switchover via `{}`: seed node does not currently report full quorum trust",
            state.self_member_id
        )));
    }

    match &state.ha.publication.authority {
        AuthorityView::Primary { member, .. } if member.0 == state.self_member_id => {}
        _ => {
            return Err(CliError::Resolution(format!(
                "cannot request switchover via `{}`: seed node is not the authoritative primary",
                state.self_member_id
            )));
        }
    }

    let Some(target_member_id) = switchover_to else {
        return Ok(());
    };

    if authority_primary_member(state).as_deref() == Some(target_member_id) {
        return Err(CliError::Resolution(format!(
            "cannot target member `{target_member_id}` for switchover: it is already the leader"
        )));
    }

    let target_member = state
        .dcs
        .cache
        .member_slots
        .get(&MemberId(target_member_id.to_string()))
        .ok_or_else(|| {
            CliError::Resolution(format!(
                "cannot target member `{target_member_id}` for switchover: it is not present in the seed node DCS member slots"
            ))
        })?;

    if !member_is_ready_replica(target_member) {
        return Err(CliError::Resolution(format!(
            "cannot target member `{target_member_id}` for switchover: it is not a ready replica in the seed node DCS view"
        )));
    }

    Ok(())
}
