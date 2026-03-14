use tokio::sync::mpsc;

use crate::state::{MemberId, SwitchoverTarget};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DcsCommand {
    AcquireLeadership,
    ReleaseLeadership,
    PublishSwitchoverAny,
    PublishSwitchoverTo(MemberId),
    ClearSwitchover,
}

#[derive(Clone)]
pub(crate) struct DcsHandle {
    sender: mpsc::UnboundedSender<DcsCommand>,
}

pub(crate) type DcsCommandInbox = mpsc::UnboundedReceiver<DcsCommand>;

#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub(crate) enum DcsHandleError {
    #[error("dcs command channel closed")]
    ChannelClosed,
}

pub(crate) fn dcs_command_channel() -> (DcsHandle, DcsCommandInbox) {
    let (sender, receiver) = mpsc::unbounded_channel();
    (DcsHandle { sender }, receiver)
}

impl DcsHandle {
    #[cfg(any(test, feature = "internal-test-support"))]
    pub(crate) fn closed() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        drop(receiver);
        Self { sender }
    }

    pub(crate) fn acquire_leadership(&self) -> Result<(), DcsHandleError> {
        self.send(DcsCommand::AcquireLeadership)
    }

    pub(crate) fn release_leadership(&self) -> Result<(), DcsHandleError> {
        self.send(DcsCommand::ReleaseLeadership)
    }

    pub(crate) fn publish_switchover_any(&self) -> Result<(), DcsHandleError> {
        self.send(DcsCommand::PublishSwitchoverAny)
    }

    pub(crate) fn publish_switchover_to(&self, target: MemberId) -> Result<(), DcsHandleError> {
        self.send(DcsCommand::PublishSwitchoverTo(target))
    }

    pub(crate) fn publish_switchover(
        &self,
        target: SwitchoverTarget,
    ) -> Result<(), DcsHandleError> {
        match target {
            SwitchoverTarget::AnyHealthyReplica => self.publish_switchover_any(),
            SwitchoverTarget::Specific(member_id) => self.publish_switchover_to(member_id),
        }
    }

    pub(crate) fn clear_switchover(&self) -> Result<(), DcsHandleError> {
        self.send(DcsCommand::ClearSwitchover)
    }

    fn send(&self, command: DcsCommand) -> Result<(), DcsHandleError> {
        self.sender
            .send(command)
            .map_err(|_| DcsHandleError::ChannelClosed)
    }
}
