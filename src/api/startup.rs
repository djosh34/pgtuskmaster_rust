use crate::{
    config_v2::RuntimeConfigV2,
    dcs::DcsHandle,
    logging::LogSender,
    state::{NodeIdentity, WorkerError},
};

use super::worker::{ApiObservedState, ApiReloadCertificatesHandle, ApiRuntimeCtx};

pub(crate) async fn run(ctx: ApiRuntimeCtx<'static>) -> Result<(), WorkerError> {
    super::worker::run(ctx).await
}

pub(crate) fn bootstrap<'a>(
    identity: NodeIdentity,
    cfg: &'a RuntimeConfigV2,
    dcs_handle: DcsHandle,
    observed: ApiObservedState,
    log: LogSender,
) -> Result<ApiRuntimeCtx<'a>, WorkerError> {
    let transport = crate::tls::build_api_server_transport_v2(&cfg.api.transport)
        .map_err(|err| WorkerError::Message(format!("api tls config build failed: {err}")))?;

    Ok(ApiRuntimeCtx {
        cfg,
        identity,
        observed,
        dcs_handle,
        reload_certificates: ApiReloadCertificatesHandle::from_transport(&transport),
        transport,
        _log: log,
    })
}
