use std::{io::Write, time::Duration};

use crate::{
    api::NodeState,
    cli::{
        args::StatusOptions, client::CliApiClient, config::OperatorContext, error::CliError, output,
    },
    command::{
        CommandOutputDto, StateCommandOutputDto, StateHealthDto, StateProjectionDto,
        StateQueryOriginDto, StateWarningDto, switchover_projection,
    },
    dcs::ClusterMemberView,
    ha::types::{AuthorityProjection, PublicationState},
};

pub(crate) async fn run_status(
    context: &OperatorContext,
    options: StatusOptions,
) -> Result<String, CliError> {
    if options.watch {
        return run_watch(context, options).await;
    }

    let output = fetch_state_command_output(context, options.verbose).await?;
    output::render_command_output(
        &CommandOutputDto::State {
            output: Box::new(output),
        },
        options.json,
    )
}

pub(crate) async fn fetch_seed_state(
    context: &OperatorContext,
) -> Result<(NodeState, StateQueryOriginDto), CliError> {
    let client = CliApiClient::from_config(context.api_client.clone())?;
    let state = client.get_state().await?;
    let queried_via = StateQueryOriginDto {
        member_id: state.identity.member_id.0.clone(),
        api_url: client.base_url().to_string(),
    };
    Ok((state, queried_via))
}

pub(crate) async fn fetch_state_command_output(
    context: &OperatorContext,
    verbose: bool,
) -> Result<StateCommandOutputDto, CliError> {
    let (state, queried_via) = fetch_seed_state(context).await?;
    Ok(build_state_command_output(state, queried_via, verbose))
}

pub(crate) fn build_state_command_output(
    state: NodeState,
    queried_via: StateQueryOriginDto,
    verbose: bool,
) -> StateCommandOutputDto {
    let projection = build_state_projection(&state, queried_via, verbose);
    StateCommandOutputDto { projection, state }
}

pub(crate) fn build_state_projection(
    state: &NodeState,
    queried_via: StateQueryOriginDto,
    verbose: bool,
) -> StateProjectionDto {
    let warnings = collect_warnings(state);
    let health = if warnings.is_empty() {
        StateHealthDto::Healthy
    } else {
        StateHealthDto::Degraded
    };
    StateProjectionDto {
        cluster_name: state.identity.cluster_name.0.clone(),
        scope: state.identity.scope.0.clone(),
        queried_via,
        health,
        verbose,
        discovered_member_count: state.dcs.member_count(),
        warnings,
        switchover: switchover_projection(&state.dcs),
    }
}

async fn run_watch(context: &OperatorContext, options: StatusOptions) -> Result<String, CliError> {
    let mut stdout = std::io::stdout();
    let interval = Duration::from_secs(2);

    loop {
        let rendered = fetch_state_command_output(context, options.verbose)
            .await
            .and_then(|output| {
                output::render_command_output(
                    &CommandOutputDto::State {
                        output: Box::new(output),
                    },
                    options.json,
                )
            })?;
        if options.json {
            writeln!(stdout, "{rendered}")
                .map_err(|err| CliError::Output(format!("watch write failed: {err}")))?;
        } else {
            writeln!(stdout, "\x1B[2J\x1B[H{rendered}")
                .map_err(|err| CliError::Output(format!("watch write failed: {err}")))?;
        }
        stdout
            .flush()
            .map_err(|err| CliError::Output(format!("watch flush failed: {err}")))?;

        tokio::select! {
            _ = tokio::signal::ctrl_c() => return Ok(String::new()),
            _ = tokio::time::sleep(interval) => {}
        }
    }
}

pub(crate) fn collect_warnings(state: &NodeState) -> Vec<StateWarningDto> {
    let degraded_mode_warning = (!state.dcs.is_quorum()).then(|| StateWarningDto {
        code: "degraded_dcs_mode".to_string(),
        message: format!(
            "seed node reports {} DCS mode",
            dcs_mode_label(&state.dcs)
        ),
    });
    let no_primary_warning = authority_primary_member(state).is_none().then(|| StateWarningDto {
        code: "no_primary".to_string(),
        message: "seed node does not currently project an authoritative primary".to_string(),
    });
    let no_members_warning = (state.dcs.member_count() == 0).then(|| StateWarningDto {
            code: "no_members".to_string(),
            message: "seed node does not currently expose any DCS member slots".to_string(),
        });

    [
        degraded_mode_warning,
        no_primary_warning,
        no_members_warning,
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
}

pub(crate) fn authority_primary_member(state: &NodeState) -> Option<String> {
    match &state.ha.publication {
        PublicationState::Projected(AuthorityProjection::Primary(epoch)) => {
            Some(epoch.holder.0.clone())
        }
        PublicationState::Unknown
        | PublicationState::Projected(AuthorityProjection::NoPrimary(_)) => None,
    }
}

pub(crate) fn member_is_ready_replica(member: &ClusterMemberView) -> bool {
    member.postgres().is_ready_replica()
}

fn dcs_mode_label(snapshot: &crate::dcs::DcsView) -> &'static str {
    snapshot.mode_label()
}
