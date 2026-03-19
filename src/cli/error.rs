use std::{io, process::ExitCode};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("config error: {0}")]
    Config(String),
    #[error("transport error: {0}")]
    Transport(String),
    #[error("api request failed with status {status}: {body}")]
    ApiStatus { status: u16, body: String },
    #[error("response decode failed: {0}")]
    Decode(String),
    #[error("request build failed: {0}")]
    RequestBuild(String),
    #[error("resolution error: {0}")]
    Resolution(String),
    #[error("output serialization failed: {0}")]
    OutputSerialize(#[from] serde_json::Error),
    #[error("watch write failed: {0}")]
    OutputWrite(#[source] io::Error),
    #[error("watch flush failed: {0}")]
    OutputFlush(#[source] io::Error),
}

impl CliError {
    pub fn exit_code(&self) -> ExitCode {
        match self {
            Self::Config(_) => ExitCode::from(6),
            Self::Transport(_) | Self::RequestBuild(_) => ExitCode::from(3),
            Self::ApiStatus { .. } | Self::Resolution(_) => ExitCode::from(4),
            Self::Decode(_)
            | Self::OutputSerialize(_)
            | Self::OutputWrite(_)
            | Self::OutputFlush(_) => ExitCode::from(5),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{error::Error as _, io, process::ExitCode};

    use super::CliError;

    #[test]
    fn output_serialize_errors_keep_source_and_exit_code() {
        let err = CliError::from(serde_json::Error::io(io::Error::other("json sink failed")));

        assert_eq!(err.exit_code(), ExitCode::from(5));
        assert_eq!(
            err.to_string(),
            "output serialization failed: json sink failed"
        );
        assert_eq!(
            err.source().map(ToString::to_string).as_deref(),
            Some("json sink failed")
        );
    }
}
