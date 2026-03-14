use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkerError {
    #[error("{0}")]
    Message(String),
}

#[cfg(any(test, feature = "internal-test-support"))]
impl From<crate::dev_support::HarnessError> for WorkerError {
    fn from(value: crate::dev_support::HarnessError) -> Self {
        Self::Message(format!("test harness error: {value}"))
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum StateRecvError {
    #[error("state channel is closed")]
    ChannelClosed,
}
