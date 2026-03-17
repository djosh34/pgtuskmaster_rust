use std::{collections::BTreeMap, time::Duration};

use crate::{
    config::{DcsClientConfig, DcsEndpoint, RuntimeConfig},
    logging::LogSender,
    pginfo::state::PgInfoState,
    state::{
        NodeIdentity, PgEndpoint, StateSubscriber, SwitchoverState, WorkerError, new_state_channel,
    },
};

use super::{
    DcsHandle,
    command::dcs_command_channel,
    state::{DcsRuntimeCtx, DcsSnapshot},
    worker::DcsError,
};

pub(crate) type DcsAdvertisedEndpoints = PgEndpoint;

pub(crate) struct DcsRuntimeRequest {
    pub(crate) identity: NodeIdentity,
    pub(crate) endpoints: Vec<DcsEndpoint>,
    pub(crate) client: DcsClientConfig,
    pub(crate) poll_interval: Duration,
    pub(crate) member_ttl_ms: u64,
    pub(crate) advertised: DcsAdvertisedEndpoints,
    pub(crate) pg_subscriber: StateSubscriber<PgInfoState>,
    pub(crate) log: LogSender,
}

pub(crate) struct DcsRuntime {
    pub(crate) state: crate::state::StateSubscriber<DcsSnapshot>,
    pub(crate) handle: DcsHandle,
    pub(crate) worker: DcsWorker,
}

pub(crate) struct DcsWorker(pub(super) DcsRuntimeCtx);

impl DcsAdvertisedEndpoints {
    pub(crate) fn from_config(cfg: &RuntimeConfig) -> Result<Self, DcsError> {
        let advertise_port = cfg
            .postgres
            .network
            .advertise_port
            .unwrap_or(cfg.postgres.network.listen_port);
        PgEndpoint::tcp(cfg.postgres.network.listen_host.clone(), advertise_port)
            .map_err(DcsError::Io)
    }
}

impl DcsWorker {
    pub(crate) async fn run(self) -> Result<(), WorkerError> {
        super::worker::run(self.0).await
    }
}

pub(crate) fn bootstrap(request: DcsRuntimeRequest) -> Result<DcsRuntime, DcsError> {
    let (publisher, state) = new_state_channel(DcsSnapshot::starting());
    let (handle, command_inbox) = dcs_command_channel();
    let ctx = DcsRuntimeCtx {
        identity: request.identity,
        endpoints: request.endpoints,
        client: request.client,
        poll_interval: request.poll_interval,
        member_ttl_ms: request.member_ttl_ms,
        advertised_postgres: request.advertised,
        pg: request.pg_subscriber,
        publisher,
        members: BTreeMap::new(),
        leadership: None,
        switchover: SwitchoverState::None,
        command_inbox,
        log: request.log,
        last_emitted_authority: None,
    };

    Ok(DcsRuntime {
        state,
        handle,
        worker: DcsWorker(ctx),
    })
}
