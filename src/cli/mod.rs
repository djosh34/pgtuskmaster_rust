pub mod args;
pub mod client;
pub mod config;
pub mod error;
pub mod output;

use args::{Cli, Command, SwitchoverCommand};
use client::{AcceptedResponse, CliApiClient, HaStateResponse};
use config::resolve_operator_context;
use error::CliError;

pub enum CommandOutput {
    HaState(Box<HaStateResponse>),
    Accepted(AcceptedResponse),
}

pub async fn run(cli: Cli) -> Result<String, CliError> {
    let context = resolve_operator_context(&cli)?;
    let output_format = cli.output;
    let command = cli.command;
    if context.api_auth_enabled
        && matches!(command, Command::Switchover(_))
        && context.api_client.auth.admin_token.is_none()
    {
        return Err(CliError::Config(
            "admin token is required for switchover commands when API auth is enabled".to_string(),
        ));
    }
    let client = CliApiClient::from_config(context.api_client)?;

    let command_output = match command {
        Command::Status => CommandOutput::HaState(Box::new(client.get_ha_state().await?)),
        Command::Switchover(switchover) => match switchover.command {
            SwitchoverCommand::Clear => CommandOutput::Accepted(client.delete_switchover().await?),
            SwitchoverCommand::Request(request) => {
                CommandOutput::Accepted(client.post_switchover(request.switchover_to).await?)
            }
        },
    };

    output::render_output(&command_output, output_format)
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
