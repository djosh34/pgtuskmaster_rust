use thiserror::Error;
use tokio::sync::{mpsc, oneshot};

use super::state::DcsSwitchoverTargetView;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DcsCommand {
    AcquireLeadership,
    ReleaseLeadership,
    PublishSwitchover {
        target: DcsSwitchoverTargetView,
    },
    ClearSwitchover,
}

#[derive(Clone)]
pub struct DcsHandle {
    sender: mpsc::UnboundedSender<DcsCommandRequest>,
}

pub(crate) struct DcsCommandRequest {
    pub(crate) command: DcsCommand,
    pub(crate) response_tx: oneshot::Sender<Result<(), DcsCommandError>>,
}

pub(crate) type DcsCommandInbox = mpsc::UnboundedReceiver<DcsCommandRequest>;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum DcsCommandError {
    #[error("dcs command channel closed")]
    ChannelClosed,
    #[error("dcs command rejected: {0}")]
    Rejected(String),
    #[error("dcs command transport failed: {0}")]
    Transport(String),
}

pub(crate) fn dcs_command_channel() -> (DcsHandle, DcsCommandInbox) {
    let (sender, receiver) = mpsc::unbounded_channel();
    (DcsHandle { sender }, receiver)
}

impl DcsHandle {
    pub fn closed() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        drop(receiver);
        Self { sender }
    }

    pub async fn acquire_leadership(&self) -> Result<(), DcsCommandError> {
        self.send(DcsCommand::AcquireLeadership).await
    }

    pub async fn release_leadership(&self) -> Result<(), DcsCommandError> {
        self.send(DcsCommand::ReleaseLeadership).await
    }

    pub async fn publish_switchover(
        &self,
        target: DcsSwitchoverTargetView,
    ) -> Result<(), DcsCommandError> {
        self.send(DcsCommand::PublishSwitchover { target }).await
    }

    pub async fn clear_switchover(&self) -> Result<(), DcsCommandError> {
        self.send(DcsCommand::ClearSwitchover).await
    }

    async fn send(&self, command: DcsCommand) -> Result<(), DcsCommandError> {
        let (response_tx, response_rx) = oneshot::channel();
        self.sender
            .send(DcsCommandRequest {
                command,
                response_tx,
            })
            .map_err(|_| DcsCommandError::ChannelClosed)?;
        response_rx
            .await
            .map_err(|err| DcsCommandError::Transport(err.to_string()))?
    }
}
