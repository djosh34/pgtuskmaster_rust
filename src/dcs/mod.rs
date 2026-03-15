mod command;
mod log_event;
mod view;
mod worker;

pub use view::{
    ClusterMemberView, ClusterView, DcsMode, DcsView, LeadershipObservation, MemberPostgresView,
    SwitchoverView,
};

pub(crate) use command::DcsHandle;

use std::time::Duration;

use crate::{
    config::{DcsClientConfig, DcsEndpoint, RuntimeConfig},
    logging::LogSender,
    pginfo::state::PgInfoState,
    state::{new_state_channel, NodeIdentity, PgTcpTarget, StateSubscriber, WorkerError},
};

pub(crate) struct DcsAdvertisedEndpoints {
    pub(crate) postgres: PgTcpTarget,
}

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
    pub(crate) state: crate::state::StateSubscriber<DcsView>,
    pub(crate) handle: DcsHandle,
    pub(crate) worker: DcsWorker,
}

pub(crate) struct DcsWorker(worker::WorkerCtx);

impl DcsAdvertisedEndpoints {
    pub(crate) fn from_config(cfg: &RuntimeConfig) -> Result<Self, String> {
        let advertise_port = cfg
            .postgres
            .network
            .advertise_port
            .unwrap_or(cfg.postgres.network.listen_port);
        let postgres = PgTcpTarget::new(cfg.postgres.network.listen_host.clone(), advertise_port)?;
        Ok(Self { postgres })
    }
}

impl DcsWorker {
    pub(crate) async fn run(self) -> Result<(), WorkerError> {
        worker::run(self.0).await
    }
}

pub(crate) fn bootstrap(request: DcsRuntimeRequest) -> Result<DcsRuntime, String> {
    let (publisher, state) = new_state_channel(DcsView::starting());
    let (ctx, handle) = worker::build_worker(worker::WorkerConfig {
        self_id: request.identity.member_id,
        scope: request.identity.scope.0,
        endpoints: request.endpoints,
        client_config: request.client,
        poll_interval: request.poll_interval,
        member_ttl_ms: request.member_ttl_ms,
        advertised_postgres: request.advertised.postgres,
        pg_subscriber: request.pg_subscriber,
        publisher,
        log: request.log,
    });

    Ok(DcsRuntime {
        state,
        handle,
        worker: DcsWorker(ctx),
    })
}
