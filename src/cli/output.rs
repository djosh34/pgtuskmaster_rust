use crate::{cli::error::CliError, command::CommandOutputDto};

pub fn render_command_output(value: &CommandOutputDto, json: bool) -> Result<String, CliError> {
    if json {
        serde_json::to_string_pretty(value)
            .map_err(|err| CliError::Output(format!("json encode failed: {err}")))
    } else {
        Ok(value.to_string())
    }
}
