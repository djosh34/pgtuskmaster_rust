pub mod args;
pub mod client;
pub mod config;
pub mod error;
pub mod output;
pub mod status;

use args::{Cli, Command, SwitchoverCommand};
use client::CliApiClient;
use config::resolve_operator_context;
use error::CliError;

pub async fn run(cli: Cli) -> Result<String, CliError> {
    let context = resolve_operator_context(&cli)?;
    let status_options = cli.status_options();
    let command = cli.command.clone().unwrap_or(Command::Status);
    if context.api_auth_enabled
        && matches!(command, Command::Switchover(_))
        && context.api_client.auth.admin_token.is_none()
    {
        return Err(CliError::Config(
            "admin token is required for switchover commands when API auth is enabled".to_string(),
        ));
    }

    match command {
        Command::Status => status::run_status(&context, status_options).await,
        Command::Switchover(switchover) => match switchover.command {
            SwitchoverCommand::Clear => {
                if status_options.watch || status_options.verbose {
                    return Err(CliError::Config(
                        "`--watch` and `--verbose` are only supported for `pgtm status`"
                            .to_string(),
                    ));
                }
                let client = CliApiClient::from_config(context.api_client)?;
                let response = client.delete_switchover().await?;
                output::render_accepted_output(&response, cli.json)
            }
            SwitchoverCommand::Request(request) => {
                if status_options.watch || status_options.verbose {
                    return Err(CliError::Config(
                        "`--watch` and `--verbose` are only supported for `pgtm status`"
                            .to_string(),
                    ));
                }
                let client = CliApiClient::from_config(context.api_client)?;
                let response = client.post_switchover(request.switchover_to).await?;
                output::render_accepted_output(&response, cli.json)
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::cli::error::CliError;

    #[test]
    fn exit_code_mapping_is_stable() {
        assert_eq!(CliError::Config("x".to_string()).exit_code(), 6.into());
        assert_eq!(CliError::Transport("x".to_string()).exit_code(), 3.into());
        assert_eq!(
            CliError::ApiStatus {
                status: 500,
                body: "x".to_string()
            }
            .exit_code(),
            4.into()
        );
        assert_eq!(CliError::Decode("x".to_string()).exit_code(), 5.into());
    }
}
