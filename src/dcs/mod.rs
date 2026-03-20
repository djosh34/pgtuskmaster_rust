use crate::{
    config_v2::RuntimeConfigV2,
    logging::LogSender,
    pginfo::state::PgInfoState,
    state::{new_state_channel, NodeIdentity, StateSubscriber},
};

mod command;
pub(crate) mod log_event;
mod state;
pub(crate) mod worker;

pub(crate) use command::DcsHandle;
pub use state::{DcsAuthority, DcsMemberState, DcsQuorumState, DcsSnapshot};
pub(crate) use worker::run;

pub(crate) fn bootstrap<'a>(
    identity: NodeIdentity,
    cfg: &'a RuntimeConfigV2,
    pg_subscriber: StateSubscriber<PgInfoState>,
    log: LogSender,
) -> Result<(StateSubscriber<DcsSnapshot>, DcsHandle, worker::DcsWorker<'a>), worker::DcsError> {
    let (publisher, state) = new_state_channel(DcsSnapshot::starting());
    let (handle, command_inbox) = command::dcs_command_channel();
    let worker = worker::DcsWorker::new(cfg, identity, pg_subscriber, publisher, command_inbox, log);
    Ok((state, handle, worker))
}
