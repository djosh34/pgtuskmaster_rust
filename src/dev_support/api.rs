use std::{collections::BTreeMap, path::Path};

use axum::Router;

use crate::{
    api::worker::{build_router, ApiObservedState},
    config_v2::types::{ApiAuth, Secret},
    config_v2::{runtime_test_config_with_data_dir, RuntimeConfigV2},
    ha::state::HaState,
    logging::LogSender,
    pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
    process::state::ProcessState,
    state::{new_state_channel, WorkerStatus},
};

use super::HarnessError;

pub fn build_test_router(
    read_token: Option<&str>,
    admin_token: Option<&str>,
) -> Result<Router, HarnessError> {
    let auth = api_auth_from_optional_tokens(read_token, admin_token)
        .map_err(HarnessError::InvalidInput)?;
    let config = runtime_test_config_with_data_dir(Path::new("/tmp/pgdata"))
        .map_err(|err| HarnessError::InvalidInput(err.to_string()))?;
    build_test_router_with_state(
        RuntimeConfigV2 {
            api: crate::config_v2::types::ApiConfig { auth, ..config.api },
            ..config
        },
        ApiObservedState::Unavailable,
    )
}

pub fn build_test_router_with_live_state(
    read_token: Option<&str>,
    admin_token: Option<&str>,
) -> Result<Router, HarnessError> {
    let auth = api_auth_from_optional_tokens(read_token, admin_token)
        .map_err(HarnessError::InvalidInput)?;
    let config = runtime_test_config_with_data_dir(Path::new("/tmp/pgdata"))
        .map_err(|err| HarnessError::InvalidInput(err.to_string()))?;
    let (_pg_publisher, pg) = new_state_channel(sample_pg_state());
    let (_process_publisher, process) = new_state_channel(sample_process_state());
    let (_dcs_publisher, dcs) = new_state_channel(crate::dcs::DcsSnapshot::starting());
    let (_ha_publisher, ha) = new_state_channel(HaState::initial(WorkerStatus::Running));

    build_test_router_with_state(
        RuntimeConfigV2 {
            api: crate::config_v2::types::ApiConfig { auth, ..config.api },
            ..config
        },
        ApiObservedState::Live {
            pg,
            process,
            dcs,
            ha,
        },
    )
}

fn build_test_router_with_state(
    cfg: RuntimeConfigV2,
    observed: ApiObservedState,
) -> Result<Router, HarnessError> {
    let cfg = Box::leak(Box::new(cfg));
    let runtime = crate::api::worker::ApiRuntimeCtx::new(
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

pub(crate) fn api_auth_from_optional_tokens(
    read_token: Option<&str>,
    admin_token: Option<&str>,
) -> Result<ApiAuth, String> {
    match (read_token, admin_token) {
        (None, None) => Ok(ApiAuth::Disabled),
        (Some(read_token), Some(admin_token)) => {
            let read_token = read_token.trim();
            let admin_token = admin_token.trim();
            if read_token.is_empty() {
                return Err("read token must not be empty".to_string());
            }
            if admin_token.is_empty() {
                return Err("admin token must not be empty".to_string());
            }
            Ok(ApiAuth::Tokens {
                read_token: Secret::new(read_token.to_string()),
                admin_token: Secret::new(admin_token.to_string()),
            })
        }
        _ => Err("read and admin tokens must either both be set or both be absent".to_string()),
    }
}
