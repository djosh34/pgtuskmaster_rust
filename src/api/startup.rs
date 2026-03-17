use crate::{
    dcs::DcsHandle,
    logging::LogSender,
    state::{NodeIdentity, StateSubscriber, WorkerError},
};

use super::worker::{
    build_router, ApiBindConfig, ApiObservedState, ApiReloadCertificatesHandle, ApiRuntimeCtx,
};

pub(crate) struct ApiRuntimeRequest {
    pub(crate) identity: NodeIdentity,
    pub(crate) runtime_config: StateSubscriber<crate::config::RuntimeConfig>,
    pub(crate) dcs_handle: DcsHandle,
    pub(crate) observed_state: ApiObservedState,
    pub(crate) log: LogSender,
}

pub(crate) struct ApiRuntime {
    pub(crate) worker: ApiWorker,
}

pub(crate) struct ApiWorker(ApiRuntimeCtx);

impl ApiWorker {
    pub(crate) async fn run(self) -> Result<(), WorkerError> {
        super::worker::run(self.0).await
    }
}

impl ApiRuntime {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn into_router(self) -> Result<axum::Router, WorkerError> {
        build_router(self.worker.0)
    }
}

pub(crate) fn bootstrap(request: ApiRuntimeRequest) -> Result<ApiRuntime, WorkerError> {
    let cfg = request.runtime_config.latest();
    let transport = crate::tls::build_api_server_transport(&cfg.api.transport)
        .map_err(|err| WorkerError::Message(format!("api tls config build failed: {err}")))?;

    Ok(ApiRuntime {
        worker: ApiWorker(ApiRuntimeCtx {
            identity: request.identity,
            observed: request.observed_state,
            runtime_config: request.runtime_config,
            dcs_handle: request.dcs_handle,
            bind: ApiBindConfig::listen(cfg.api.listen_addr),
            auth: crate::config::TokenAuth::Disabled,
            reload_certificates: ApiReloadCertificatesHandle::from_transport(&transport),
            transport,
            _log: request.log,
        }),
    })
}
