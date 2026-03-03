pub mod args;
pub mod client;
pub mod error;
pub mod output;

use args::{Cli, Command, HaCommand, LeaderCommand, SwitchoverCommand};
use client::{AcceptedResponse, CliApiClient, HaStateResponse};
use error::CliError;

pub enum CommandOutput {
    HaState(HaStateResponse),
    Accepted(AcceptedResponse),
}

pub async fn run(cli: Cli) -> Result<String, CliError> {
    let output_format = cli.output;
    let command = cli.command;
    let client = CliApiClient::new(
        cli.base_url,
        cli.timeout_ms,
        cli.read_token,
        cli.admin_token,
    )?;

    let command_output = match command {
        Command::Ha(ha) => match ha.command {
            HaCommand::State => CommandOutput::HaState(client.get_ha_state().await?),
            HaCommand::Leader(leader) => match leader.command {
                LeaderCommand::Set(input) => {
                    CommandOutput::Accepted(client.post_set_leader(input.member_id).await?)
                }
                LeaderCommand::Clear => CommandOutput::Accepted(client.delete_leader().await?),
            },
            HaCommand::Switchover(switchover) => match switchover.command {
                SwitchoverCommand::Clear => {
                    CommandOutput::Accepted(client.delete_switchover().await?)
                }
                SwitchoverCommand::Request(input) => {
                    CommandOutput::Accepted(client.post_switchover(input.requested_by).await?)
                }
            },
        },
    };

    output::render_output(&command_output, output_format)
}

#[cfg(test)]
mod tests {
    use crate::cli::error::CliError;

    #[test]
    fn exit_code_mapping_is_stable() {
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
