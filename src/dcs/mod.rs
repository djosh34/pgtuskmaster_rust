use crate::{
    config::RuntimeConfig,
    logging::LogSender,
    pginfo::state::PgInfoState,
    state::{new_state_channel, NodeIdentity, PgEndpoint, StateSubscriber},
};

mod command;
pub(crate) mod log_event;
mod state;
pub(crate) mod worker;

pub(crate) use command::DcsHandle;
pub use state::{DcsAuthority, DcsMemberState, DcsQuorumState, DcsSnapshot};
pub(crate) use worker::run;

pub(crate) fn bootstrap(
    identity: NodeIdentity,
    cfg: &RuntimeConfig,
    pg_subscriber: StateSubscriber<PgInfoState>,
    log: LogSender,
) -> Result<(StateSubscriber<DcsSnapshot>, DcsHandle, worker::DcsWorker), worker::DcsError> {
    let (publisher, state) = new_state_channel(DcsSnapshot::starting());
    let (handle, command_inbox) = command::dcs_command_channel();
    let advertise_port = cfg
        .postgres
        .network
        .advertise_port
        .unwrap_or(cfg.postgres.network.listen_port);
    let advertised_postgres =
        PgEndpoint::tcp(cfg.postgres.network.listen_host.clone(), advertise_port)
            .map_err(worker::DcsError::Io)?;
    let worker = worker::DcsWorker::new(
        identity,
        cfg.dcs.endpoints.clone(),
        cfg.dcs.client.clone(),
        std::time::Duration::from_millis(cfg.ha.loop_interval_ms),
        cfg.ha.lease_ttl_ms,
        advertised_postgres,
        pg_subscriber,
        publisher,
        command_inbox,
        log,
    );
    Ok((state, handle, worker))
}
