use std::collections::BTreeMap;

use axum::Router;

use crate::{
    api::worker::{build_router, ApiObservedState},
    config_v2::RuntimeConfigV2,
    ha::state::HaState,
    logging::LogSender,
    pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
    process::state::ProcessState,
    state::{new_state_channel, NodeIdentity, WorkerStatus},
};

use super::HarnessError;

pub fn build_test_router(
    read_token: Option<&str>,
    admin_token: Option<&str>,
) -> Result<Router, HarnessError> {
    build_test_router_with_state(
        build_test_runtime_config(read_token, admin_token)?,
        ApiObservedState::Unavailable,
    )
}

pub fn build_test_router_with_live_state(
    read_token: Option<&str>,
    admin_token: Option<&str>,
) -> Result<Router, HarnessError> {
    let (_pg_publisher, pg) = new_state_channel(sample_pg_state());
    let (_process_publisher, process) = new_state_channel(sample_process_state());
    let (_dcs_publisher, dcs) = new_state_channel(crate::dcs::DcsSnapshot::starting());
    let (_ha_publisher, ha) = new_state_channel(HaState::initial(WorkerStatus::Running));

    build_test_router_with_state(
        build_test_runtime_config(read_token, admin_token)?,
        ApiObservedState::Live {
            pg,
            process,
            dcs,
            ha,
        },
    )
}

fn build_test_runtime_config(
    read_token: Option<&str>,
    admin_token: Option<&str>,
) -> Result<RuntimeConfigV2, HarnessError> {
    let auth = crate::dev_support::runtime_config::api_auth_from_optional_tokens(
        read_token,
        admin_token,
    )
    .map_err(HarnessError::InvalidInput)?;
    Ok(crate::dev_support::runtime_config::RuntimeConfigBuilder::new()
        .with_api_auth(auth)
        .build())
}

fn build_test_router_with_state(
    cfg: RuntimeConfigV2,
    observed: ApiObservedState,
) -> Result<Router, HarnessError> {
    let cfg = Box::leak(Box::new(cfg));
    let runtime = crate::api::startup::bootstrap(
        NodeIdentity {
            cluster_name: cfg.cluster_name.clone(),
            scope: cfg.scope.clone(),
            member_id: cfg.member_id.clone(),
        },
        cfg,
        crate::dcs::DcsHandle::closed(),
        observed,
        LogSender::disabled(),
    )
    .map_err(|err| HarnessError::InvalidInput(err.to_string()))?;
    build_router(runtime).map_err(|err| HarnessError::InvalidInput(err.to_string()))
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
