pub mod args;
pub mod client;
pub mod config;
pub mod connect;
pub mod error;
pub mod output;
pub mod status;
pub mod switchover;

use args::{Cli, Command, SwitchoverCommand};
use config::resolve_operator_context;
use connect::{run_primary, run_replicas};
use error::CliError;
use switchover::{run_clear as run_switchover_clear, run_request as run_switchover_request};

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
        Command::Primary(connection_args) => {
            if status_options.watch || status_options.verbose {
                return Err(CliError::Config(
                    "`--watch` and `--verbose` are only supported for `pgtm status`".to_string(),
                ));
            }
            run_primary(&context, cli.connection_options(&connection_args)).await
        }
        Command::Replicas(connection_args) => {
            if status_options.watch || status_options.verbose {
                return Err(CliError::Config(
                    "`--watch` and `--verbose` are only supported for `pgtm status`".to_string(),
                ));
            }
            run_replicas(&context, cli.connection_options(&connection_args)).await
        }
        Command::Switchover(switchover) => match switchover.command {
            SwitchoverCommand::Clear => {
                if status_options.watch || status_options.verbose {
                    return Err(CliError::Config(
                        "`--watch` and `--verbose` are only supported for `pgtm status`"
                            .to_string(),
                    ));
                }
                run_switchover_clear(&context, cli.json).await
            }
            SwitchoverCommand::Request(request) => {
                if status_options.watch || status_options.verbose {
                    return Err(CliError::Config(
                        "`--watch` and `--verbose` are only supported for `pgtm status`"
                            .to_string(),
                    ));
                }
                run_switchover_request(&context, cli.json, request.switchover_to).await
            }
        },
    }
}
