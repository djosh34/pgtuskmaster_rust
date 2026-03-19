use std::{io::Write, time::Duration};

use crate::{
    api::NodeState,
    cli::{
        args::StatusOptions, client::CliApiClient, config::OperatorContext, error::CliError, output,
    },
    command::{CommandOutputDto, StateCommandOutputDto, StateQueryOriginDto},
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
    Ok(StateCommandOutputDto::from_seed_state(
        state,
        queried_via,
        verbose,
    ))
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
