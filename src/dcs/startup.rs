use std::{collections::BTreeMap, time::Duration};

use crate::{
    config::RuntimeConfig,
    logging::LogSender,
    pginfo::state::PgInfoState,
    state::{
        new_state_channel, NodeIdentity, PgEndpoint, StateSubscriber, SwitchoverState, WorkerError,
    },
};

use super::{
    command::dcs_command_channel,
    state::{DcsRuntimeCtx, DcsSnapshot},
    worker::DcsError,
    DcsHandle,
};

pub(crate) struct DcsRuntime {
    pub(crate) state: crate::state::StateSubscriber<DcsSnapshot>,
    pub(crate) handle: DcsHandle,
    pub(crate) worker: DcsWorker,
}

pub(crate) struct DcsWorker(pub(super) DcsRuntimeCtx);

impl DcsWorker {
    pub(crate) async fn run(self) -> Result<(), WorkerError> {
        super::worker::run(self.0).await
    }
}

pub(crate) fn bootstrap(
    identity: NodeIdentity,
    cfg: &RuntimeConfig,
    pg_subscriber: StateSubscriber<PgInfoState>,
    log: LogSender,
) -> Result<DcsRuntime, DcsError> {
    let (publisher, state) = new_state_channel(DcsSnapshot::starting());
    let (handle, command_inbox) = dcs_command_channel();
    let ctx = DcsRuntimeCtx {
        identity,
        endpoints: cfg.dcs.endpoints.clone(),
        client: cfg.dcs.client.clone(),
        poll_interval: Duration::from_millis(cfg.ha.loop_interval_ms),
        member_ttl_ms: cfg.ha.lease_ttl_ms,
        advertised_postgres: advertised_postgres_from_config(cfg)?,
        pg: pg_subscriber,
        publisher,
        members: BTreeMap::new(),
        leadership: None,
        switchover: SwitchoverState::None,
        command_inbox,
        log,
        last_emitted_authority: None,
    };

    Ok(DcsRuntime {
        state,
        handle,
        worker: DcsWorker(ctx),
    })
}

fn advertised_postgres_from_config(cfg: &RuntimeConfig) -> Result<PgEndpoint, DcsError> {
    let advertise_port = cfg
        .postgres
        .network
        .advertise_port
        .unwrap_or(cfg.postgres.network.listen_port);
    PgEndpoint::tcp(cfg.postgres.network.listen_host.clone(), advertise_port).map_err(DcsError::Io)
}
