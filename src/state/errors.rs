use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum WorkerError {
    #[error("{0}")]
    Message(String),
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
