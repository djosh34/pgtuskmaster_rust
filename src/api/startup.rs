use crate::{
    dcs::DcsHandle,
    logging::LogSender,
    state::{NodeIdentity, StateSubscriber, WorkerError},
};

use super::worker::{
    build_router, ApiBindConfig, ApiObservedState, ApiReloadCertificatesHandle, ApiRuntimeCtx,
};

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn into_router(ctx: ApiRuntimeCtx) -> Result<axum::Router, WorkerError> {
    build_router(ctx)
}

pub(crate) async fn run(ctx: ApiRuntimeCtx) -> Result<(), WorkerError> {
    super::worker::run(ctx).await
}

pub(crate) fn bootstrap(
    identity: NodeIdentity,
    runtime_config: StateSubscriber<crate::config::RuntimeConfig>,
    dcs_handle: DcsHandle,
    observed: ApiObservedState,
    log: LogSender,
) -> Result<ApiRuntimeCtx, WorkerError> {
    let cfg = runtime_config.latest();
    let transport = crate::tls::build_api_server_transport(&cfg.api.transport)
        .map_err(|err| WorkerError::Message(format!("api tls config build failed: {err}")))?;

    Ok(ApiRuntimeCtx {
        identity,
        observed,
        runtime_config,
        dcs_handle,
        bind: ApiBindConfig::listen(cfg.api.listen_addr),
        auth: crate::config::TokenAuth::Disabled,
        reload_certificates: ApiReloadCertificatesHandle::from_transport(&transport),
        transport,
        _log: log,
    })
}
