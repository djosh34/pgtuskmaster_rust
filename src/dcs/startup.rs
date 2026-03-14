use std::time::Duration;

use crate::{
    config::{DcsEndpoint, RuntimeConfig},
    logging::LogHandle,
    pginfo::state::PgInfoState,
    state::{new_state_channel, NodeIdentity, StateSubscriber, WorkerError},
};

use super::{
    command::DcsHandle,
    state::{
        DcsApiAdvertisement, DcsCadence, DcsLocalMemberAdvertisement, DcsNodeIdentity,
        DcsObservedState, DcsRuntime as DcsWorkerRuntime, DcsStateChannel, DcsView,
    },
    store::DcsStoreError,
    worker::{DcsStoreBootstrap, DcsWorkerBootstrap},
};

pub(crate) struct DcsAdvertisedEndpoints {
    pub(crate) postgres: crate::dcs::DcsMemberEndpointView,
    pub(crate) api: Option<crate::dcs::DcsMemberApiView>,
}

pub(crate) struct DcsRuntimeRequest {
    pub(crate) identity: NodeIdentity,
    pub(crate) endpoints: Vec<DcsEndpoint>,
    pub(crate) poll_interval: Duration,
    pub(crate) member_ttl_ms: u64,
    pub(crate) advertised: DcsAdvertisedEndpoints,
    pub(crate) pg_subscriber: StateSubscriber<PgInfoState>,
    pub(crate) log: LogHandle,
}

pub(crate) struct DcsRuntime {
    pub(crate) state: crate::state::StateSubscriber<DcsView>,
    pub(crate) handle: DcsHandle,
    pub(crate) worker: DcsWorker,
}

pub(crate) struct DcsWorker(super::state::DcsWorkerCtx);

impl DcsAdvertisedEndpoints {
    pub(crate) fn from_config(cfg: &RuntimeConfig) -> Self {
        let api = if let Some(api_url) = cfg.pgtm.as_ref().and_then(|pgtm| pgtm.api_url.clone()) {
            Some(crate::dcs::DcsMemberApiView { url: api_url })
        } else if cfg.api.listen_addr.ip().is_unspecified() {
            None
        } else {
            let scheme = match cfg.api.security.transport {
                crate::config::ApiTransportConfig::Http => "http",
                crate::config::ApiTransportConfig::Https { .. } => "https",
            };
            Some(crate::dcs::DcsMemberApiView {
                url: format!("{scheme}://{}", cfg.api.listen_addr),
            })
        };

        Self {
            postgres: crate::dcs::DcsMemberEndpointView {
                host: cfg.postgres.listen_host.clone(),
                port: cfg
                    .postgres
                    .advertise_port
                    .unwrap_or(cfg.postgres.listen_port),
            },
            api,
        }
    }
}

impl DcsWorker {
    pub(crate) async fn run(self) -> Result<(), WorkerError> {
        super::worker::run(self.0).await
    }
}

pub(crate) fn bootstrap(request: DcsRuntimeRequest) -> Result<DcsRuntime, DcsStoreError> {
    let (publisher, state) = new_state_channel(DcsView::starting());
    let (ctx, handle) = super::worker::build_worker_ctx(DcsWorkerBootstrap {
        identity: DcsNodeIdentity {
            self_id: request.identity.member_id,
            scope: request.identity.scope.0,
        },
        store: DcsStoreBootstrap {
            endpoints: request.endpoints,
        },
        cadence: DcsCadence {
            poll_interval: request.poll_interval,
            member_ttl_ms: request.member_ttl_ms,
        },
        advertisement: DcsLocalMemberAdvertisement {
            postgres: request.advertised.postgres,
            api: request
                .advertised
                .api
                .map(DcsApiAdvertisement::Advertised)
                .unwrap_or(DcsApiAdvertisement::NotAdvertised),
        },
        observed: DcsObservedState {
            pg: request.pg_subscriber,
        },
        state_channel: DcsStateChannel::new(publisher),
        runtime: DcsWorkerRuntime {
            log: request.log,
            last_emitted_store_healthy: None,
            last_emitted_trust: None,
        },
    })?;

    Ok(DcsRuntime {
        state,
        handle,
        worker: DcsWorker(ctx),
    })
}
