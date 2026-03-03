use std::process::ExitCode;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("transport error: {0}")]
    Transport(String),
    #[error("api request failed with status {status}: {body}")]
    ApiStatus { status: u16, body: String },
    #[error("response decode failed: {0}")]
    Decode(String),
    #[error("request build failed: {0}")]
    RequestBuild(String),
    #[error("output serialization failed: {0}")]
    Output(String),
}

impl CliError {
    pub fn exit_code(&self) -> ExitCode {
        match self {
            Self::Transport(_) | Self::RequestBuild(_) => ExitCode::from(3),
            Self::ApiStatus { .. } => ExitCode::from(4),
            Self::Decode(_) | Self::Output(_) => ExitCode::from(5),
        }
    }
}
