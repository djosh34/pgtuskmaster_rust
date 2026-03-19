use crate::{cli::error::CliError, command::CommandOutputDto};

pub fn render_command_output(value: &CommandOutputDto, json: bool) -> Result<String, CliError> {
    if json {
        Ok(serde_json::to_string_pretty(value)?)
    } else {
        Ok(value.to_string())
    }
}
