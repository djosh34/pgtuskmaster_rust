use std::{path::PathBuf, string::FromUtf8Error};

#[derive(Debug, thiserror::Error)]
pub enum HarnessError {
    #[error("{0}")]
    Message(String),

    #[error("io failed at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("command `{executable}` failed while {context}: status={status}\nstdout:\n{stdout}\nstderr:\n{stderr}")]
    CommandFailed {
        executable: PathBuf,
        context: String,
        status: String,
        stdout: String,
        stderr: String,
    },

    #[error("utf-8 decode failed for {context}: {source}")]
    Utf8 {
        context: String,
        #[source]
        source: FromUtf8Error,
    },

    #[error("json decode failed for {context}: {source}")]
    Json {
        context: String,
        #[source]
        source: serde_json::Error,
    },
}

pub type Result<T> = std::result::Result<T, HarnessError>;

impl HarnessError {
    pub fn message(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}
