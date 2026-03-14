use crate::{
    config::RuntimeConfig,
    dcs::DcsHandle,
    logging::LogHandle,
    state::{NodeIdentity, StateSubscriber, WorkerError},
};

use super::worker::{
    ApiAuthState, ApiBindConfig, ApiClusterIdentity, ApiControlPlane, ApiObservedState,
    ApiReloadCertificatesHandle, ApiServerCtx, ApiServingPlan,
};

#[derive(Clone)]
pub(crate) struct ApiRuntimeRequest {
    pub(crate) identity: NodeIdentity,
    pub(crate) runtime_config: StateSubscriber<RuntimeConfig>,
    pub(crate) dcs_handle: DcsHandle,
    pub(crate) observed_state: ApiObservedState,
    pub(crate) log: LogHandle,
}

pub(crate) struct ApiRuntime {
    pub(crate) worker: ApiServer,
}

pub(crate) struct ApiServer(ApiServerCtx);

impl ApiServer {
    pub(crate) async fn run(self) -> Result<(), WorkerError> {
        super::worker::run(self.0).await
    }
}

pub(crate) fn bootstrap(request: ApiRuntimeRequest) -> Result<ApiRuntime, WorkerError> {
    let cfg = request.runtime_config.latest();
    let transport = crate::tls::build_api_server_transport(&cfg.api.transport)
        .map_err(|err| WorkerError::Message(format!("api tls config build failed: {err}")))?;

    Ok(ApiRuntime {
        worker: ApiServer(ApiServerCtx {
            identity: ApiClusterIdentity {
                cluster_name: request.identity.cluster_name.0,
                scope: request.identity.scope.0,
                member_id: request.identity.member_id.0,
            },
            observed: request.observed_state,
            control: ApiControlPlane {
                runtime_config: request.runtime_config,
                dcs_handle: request.dcs_handle,
            },
            serving: ApiServingPlan {
                bind: ApiBindConfig::listen(cfg.api.listen_addr),
                auth: ApiAuthState::Disabled,
                reload_certificates: ApiReloadCertificatesHandle::from_transport(&transport),
                transport,
            },
            log: request.log,
        }),
    })
}
