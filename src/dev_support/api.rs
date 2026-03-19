use std::collections::BTreeMap;

use axum::Router;

use crate::{
    api::{startup::ApiRuntimeRequest, worker::ApiObservedState},
    config::RuntimeConfig,
    ha::state::HaState,
    logging::LogSender,
    pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
    process::state::ProcessState,
    state::{new_state_channel, ClusterName, MemberId, NodeIdentity, ScopeName, WorkerStatus},
};

use super::HarnessError;

pub fn build_test_router(cfg: RuntimeConfig) -> Result<Router, HarnessError> {
    build_test_router_with_state(cfg, ApiObservedState::Unavailable)
}

pub fn build_test_router_with_live_state(cfg: RuntimeConfig) -> Result<Router, HarnessError> {
    let (_pg_publisher, pg) = new_state_channel(sample_pg_state());
    let (_process_publisher, process) = new_state_channel(sample_process_state());
    let (_dcs_publisher, dcs) = new_state_channel(crate::dcs::DcsSnapshot::starting());
    let (_ha_publisher, ha) = new_state_channel(HaState::initial(WorkerStatus::Running));

    build_test_router_with_state(
        cfg,
        ApiObservedState::Live {
            pg,
            process,
            dcs,
            ha,
        },
    )
}

fn build_test_router_with_state(
    cfg: RuntimeConfig,
    observed: ApiObservedState,
) -> Result<Router, HarnessError> {
    let (_cfg_publisher, runtime_config) = new_state_channel(cfg.clone());
    let runtime = crate::api::startup::bootstrap(ApiRuntimeRequest {
        identity: NodeIdentity {
            cluster_name: ClusterName(cfg.cluster.name.clone()),
            scope: ScopeName(cfg.cluster.scope.clone()),
            member_id: MemberId(cfg.cluster.member_id.clone()),
        },
        runtime_config,
        dcs_handle: crate::dcs::DcsHandle::closed(),
        observed_state: observed,
        log: LogSender::disabled(),
    })
    .map_err(|err| HarnessError::InvalidInput(err.to_string()))?;
    runtime
        .into_router()
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
