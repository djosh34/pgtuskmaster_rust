use std::{io::Write, time::Duration};

use crate::{
    api::NodeState,
    cli::{args::StatusOptions, client::CliApiClient, config::OperatorContext, error::CliError},
    command::{CommandOutputDto, StateCommandOutputDto},
};

pub(crate) async fn run_status(
    context: &OperatorContext,
    options: StatusOptions,
) -> Result<String, CliError> {
    if options.watch {
        return run_watch(context, options).await;
    }

    let output = fetch_state_command_output(context, options.verbose).await?;
    CommandOutputDto::State {
        output: Box::new(output),
    }
    .render(options.json)
}

pub(crate) async fn fetch_seed_state(
    context: &OperatorContext,
) -> Result<(NodeState, String), CliError> {
    let client = CliApiClient::from_config(context.api_client.clone())?;
    let state = client.get_state().await?;
    Ok((state, client.base_url().to_string()))
}

pub(crate) async fn fetch_state_command_output(
    context: &OperatorContext,
    verbose: bool,
) -> Result<StateCommandOutputDto, CliError> {
    let (state, api_url) = fetch_seed_state(context).await?;
    Ok(StateCommandOutputDto::from_seed_state(
        state, api_url, verbose,
    ))
}

async fn run_watch(context: &OperatorContext, options: StatusOptions) -> Result<String, CliError> {
    let mut stdout = std::io::stdout();
    let interval = Duration::from_secs(2);

    loop {
        let rendered = fetch_state_command_output(context, options.verbose)
            .await
            .and_then(|output| {
                CommandOutputDto::State {
                    output: Box::new(output),
                }
                .render(options.json)
            })?;
        if options.json {
            writeln!(stdout, "{rendered}").map_err(CliError::OutputWrite)?;
        } else {
            writeln!(stdout, "\x1B[2J\x1B[H{rendered}").map_err(CliError::OutputWrite)?;
        }
        stdout.flush().map_err(CliError::OutputFlush)?;

        tokio::select! {
            _ = tokio::signal::ctrl_c() => return Ok(String::new()),
            _ = tokio::time::sleep(interval) => {}
        }
    }
}
