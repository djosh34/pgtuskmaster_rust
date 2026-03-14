use std::time::Duration;

use crate::{
    config::{DcsClientConfig, DcsEndpoint, RuntimeConfig},
    logging::LogSender,
    pginfo::state::PgInfoState,
    state::{new_state_channel, NodeIdentity, PgTcpTarget, StateSubscriber, WorkerError},
};

use super::{
    state::{
        DcsCadence, DcsEtcdConfig, DcsLocalMemberAdvertisement, DcsNodeIdentity,
        DcsObservedState, DcsRuntime as DcsWorkerRuntime, DcsStateChannel, DcsView,
    },
    worker::{DcsError, DcsWorkerBootstrap},
    DcsHandle,
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

pub(crate) struct DcsWorker(super::state::DcsWorkerCtx);

impl DcsAdvertisedEndpoints {
    pub(crate) fn from_config(cfg: &RuntimeConfig) -> Result<Self, DcsError> {
        let advertise_port = cfg
            .postgres
            .network
            .advertise_port
            .unwrap_or(cfg.postgres.network.listen_port);
        let postgres = PgTcpTarget::new(cfg.postgres.network.listen_host.clone(), advertise_port)
            .map_err(DcsError::Io)?;
        Ok(Self { postgres })
    }
}

impl DcsWorker {
    pub(crate) async fn run(self) -> Result<(), WorkerError> {
        super::worker::run(self.0).await
    }
}

pub(crate) fn bootstrap(request: DcsRuntimeRequest) -> Result<DcsRuntime, DcsError> {
    let (publisher, state) = new_state_channel(DcsView::starting());
    let (ctx, handle) = super::worker::build_worker_ctx(DcsWorkerBootstrap {
        identity: DcsNodeIdentity {
            self_id: request.identity.member_id,
            scope: request.identity.scope.0,
        },
        etcd: DcsEtcdConfig {
            endpoints: request.endpoints,
            client: request.client,
        },
        cadence: DcsCadence {
            poll_interval: request.poll_interval,
            member_ttl_ms: request.member_ttl_ms,
        },
        advertisement: DcsLocalMemberAdvertisement {
            postgres: request.advertised.postgres,
        },
        observed: DcsObservedState {
            pg: request.pg_subscriber,
        },
        state_channel: DcsStateChannel::new(publisher),
        runtime: DcsWorkerRuntime {
            log: request.log,
            last_emitted_mode: None,
        },
    });

    Ok(DcsRuntime {
        state,
        handle,
        worker: DcsWorker(ctx),
    })
}
