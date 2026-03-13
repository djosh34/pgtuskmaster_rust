use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkerError {
    #[error("{0}")]
    Message(String),
}

impl From<crate::test_harness::HarnessError> for WorkerError {
    fn from(value: crate::test_harness::HarnessError) -> Self {
        Self::Message(format!("test harness error: {value}"))
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum StatePublishError {
    #[error("state channel is closed")]
    ChannelClosed,
    #[error("state version overflow")]
    VersionOverflow,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum StateRecvError {
    #[error("state channel is closed")]
    ChannelClosed,
}
