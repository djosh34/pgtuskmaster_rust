use std::collections::BTreeMap;

use axum::Router;

use crate::{
    api::worker::{
        build_router, ApiAuthState, ApiBindConfig, ApiCertificateReloadHandle, ApiNodeIdentity,
        ApiServerCtx, ApiStateSubscribers,
    },
    config::RuntimeConfig,
    dcs::DcsHandle,
    ha::state::HaState,
    logging::LogHandle,
    pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
    process::state::ProcessState,
    state::{new_state_channel, WorkerStatus},
};

use super::HarnessError;

pub fn build_test_router(
    cfg: RuntimeConfig,
    dcs_handle: DcsHandle,
) -> Result<Router, HarnessError> {
    build_test_router_with_state(cfg, dcs_handle, None)
}

pub fn build_test_router_with_live_state(
    cfg: RuntimeConfig,
    dcs_handle: DcsHandle,
) -> Result<Router, HarnessError> {
    let (_pg_publisher, pg) = new_state_channel(sample_pg_state());
    let (_process_publisher, process) = new_state_channel(sample_process_state());
    let (_dcs_publisher, dcs) = new_state_channel(crate::dcs::DcsView::empty(WorkerStatus::Running));
    let (_ha_publisher, ha) = new_state_channel(HaState::initial(WorkerStatus::Running));

    build_test_router_with_state(
        cfg,
        dcs_handle,
        Some(ApiStateSubscribers {
            pg,
            process,
            dcs,
            ha,
        }),
    )
}

fn build_test_router_with_state(
    cfg: RuntimeConfig,
    dcs_handle: DcsHandle,
    state: Option<ApiStateSubscribers>,
) -> Result<Router, HarnessError> {
    let (_cfg_publisher, runtime_config) = new_state_channel(cfg.clone());
    let transport = crate::tls::build_api_server_transport(&cfg.api.security.transport)
        .map_err(|err| HarnessError::InvalidInput(err.to_string()))?;
    build_router(ApiServerCtx {
        bind: ApiBindConfig::listen(cfg.api.listen_addr),
        identity: ApiNodeIdentity {
            cluster_name: cfg.cluster.name.clone(),
            scope: cfg.dcs.scope.clone(),
            member_id: cfg.cluster.member_id.clone(),
        },
        runtime_config,
        dcs_handle,
        state,
        auth: ApiAuthState::Disabled,
        transport: transport.clone(),
        cert_reloader: ApiCertificateReloadHandle::from_transport(&transport),
        log: LogHandle::disabled(),
    })
    .map_err(|err| HarnessError::InvalidInput(err.to_string()))
}

fn sample_pg_state() -> PgInfoState {
    PgInfoState::Unknown {
        common: PgInfoCommon {
            worker: WorkerStatus::Running,
            sql: SqlStatus::Healthy,
            readiness: Readiness::Ready,
            timeline: None,
            system_identifier: None,
            pg_config: PgConfig {
                port: None,
                hot_standby: None,
                primary_conninfo: None,
                primary_slot_name: None,
                extra: BTreeMap::new(),
            },
            last_refresh_at: None,
        },
    }
}

fn sample_process_state() -> ProcessState {
    ProcessState::Idle {
        worker: WorkerStatus::Running,
        last_outcome: None,
    }
}
