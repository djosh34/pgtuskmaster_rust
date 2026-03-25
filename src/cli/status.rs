use std::{io::Write, time::Duration};

use crate::{
    api::NodeState,
    cli::{
        args::StatusOptions, client::CliApiClient, config::OperatorContext, error::CliError,
        render::render_state_output,
    },
};

pub(crate) async fn run_status(
    context: &OperatorContext,
    options: StatusOptions,
) -> Result<String, CliError> {
    if options.watch {
        return run_watch(context, options).await;
    }

    let (state, api_url) = fetch_seed_state(context).await?;
    render_state_output(&state, api_url.as_str(), options.verbose, options.json)
}

pub(crate) async fn fetch_seed_state(
    context: &OperatorContext,
) -> Result<(NodeState, String), CliError> {
    let client = CliApiClient::from_config(context.api_client.clone())?;
    let state = client.get_state().await?;
    Ok((state, client.base_url().to_string()))
}

async fn run_watch(context: &OperatorContext, options: StatusOptions) -> Result<String, CliError> {
    let mut stdout = std::io::stdout();
    let interval = Duration::from_secs(2);

    loop {
        let (state, api_url) = fetch_seed_state(context).await?;
        let rendered =
            render_state_output(&state, api_url.as_str(), options.verbose, options.json)?;
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
