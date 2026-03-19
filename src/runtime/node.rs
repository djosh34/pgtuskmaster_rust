use std::{path::Path, time::Duration};

use thiserror::Error;

use crate::{
    config::{load_runtime_config, validate_runtime_config, ConfigError, RuntimeConfig},
    process::state::ProcessRuntimePlan,
    state::{new_state_channel, ClusterName, MemberId, NodeIdentity, ScopeName},
};

use super::log_event::RuntimeLogEvent;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("config error: {0}")]
    Config(#[from] ConfigError),
    #[error("startup planning failed: {0}")]
    StartupPlanning(String),
    #[error("startup execution failed: {0}")]
    StartupExecution(String),
    #[error("api bind failed at `{listen_addr}`: {message}")]
    ApiBind {
        listen_addr: std::net::SocketAddr,
        message: String,
    },
    #[error("worker failed: {0}")]
    Worker(String),
    #[error("time error: {0}")]
    Time(String),
}

fn runtime_startup_event(
    startup_run_id: &str,
    logging_level: crate::config::LogLevel,
) -> RuntimeLogEvent {
    RuntimeLogEvent::StartupEntered {
        startup_run_id: startup_run_id.to_string(),
        logging_level,
    }
}

pub async fn run_node_from_config_path(path: &Path) -> Result<(), RuntimeError> {
    let cfg = load_runtime_config(path)?;
    run_node_from_config(cfg).await
}

pub async fn run_node_from_config(cfg: RuntimeConfig) -> Result<(), RuntimeError> {
    validate_runtime_config(&cfg)?;

    let logging = crate::logging::bootstrap(&cfg).map_err(|err| {
        RuntimeError::StartupExecution(format!("logging bootstrap failed: {err}"))
    })?;
    let log = logging.sender.clone();
    let worker = logging.worker;
    let startup_run_id = format!(
        "{}-{}",
        cfg.cluster.member_id,
        crate::logging::system_now_unix_millis()
    );
    log.send(runtime_startup_event(
        startup_run_id.as_str(),
        cfg.logging.level,
    ))
    .map_err(|err| {
        RuntimeError::StartupExecution(format!("runtime start log emit failed: {err}"))
    })?;

    let process_plan = ProcessRuntimePlan::from_config(&cfg);
    process_plan.ensure_start_paths().map_err(|err| {
        RuntimeError::StartupExecution(format!("process start path preparation failed: {err}"))
    })?;

    run_workers(cfg, process_plan, log, worker).await
}

async fn run_workers(
    cfg: RuntimeConfig,
    process_plan: ProcessRuntimePlan,
    log: crate::logging::LogSender,
    log_worker: crate::logging::LogWorker,
) -> Result<(), RuntimeError> {
    let (_cfg_publisher, cfg_subscriber) = new_state_channel(cfg.clone());
    let identity = NodeIdentity {
        cluster_name: ClusterName(cfg.cluster.name.clone()),
        scope: ScopeName(cfg.cluster.scope.clone()),
        member_id: MemberId(cfg.cluster.member_id.clone()),
    };
    let worker_poll_interval = Duration::from_millis(cfg.ha.loop_interval_ms);

    let pginfo =
        crate::pginfo::startup::bootstrap(identity.clone(), &cfg, &process_plan, log.clone());

    let dcs =
        crate::dcs::startup::bootstrap(identity.clone(), &cfg, pginfo.state.clone(), log.clone())
            .map_err(|err| RuntimeError::Worker(format!("dcs store connect failed: {err}")))?;

    let process = crate::process::startup::bootstrap(
        identity.clone(),
        &cfg,
        crate::process::state::ProcessObservedState {
            runtime_config: cfg_subscriber.clone(),
            dcs: dcs.state.clone(),
        },
        process_plan,
        log.clone(),
    );

    let ha = crate::ha::startup::bootstrap(
        identity.clone(),
        worker_poll_interval,
        crate::ha::state::HaObservedState {
            config: cfg_subscriber.clone(),
            pg: pginfo.state.clone(),
            dcs: dcs.state.clone(),
            process: process.state.clone(),
        },
        crate::ha::state::HaControlPlane {
            process_intent_inbox: process.control.intents.clone(),
            dcs_handle: dcs.handle.clone(),
        },
    );

    let api = crate::api::startup::bootstrap(
        identity,
        cfg_subscriber,
        dcs.handle.clone(),
        crate::api::worker::ApiObservedState::Live {
            pg: pginfo.state.clone(),
            process: process.state.clone(),
            dcs: dcs.state.clone(),
            ha: ha.state.clone(),
        },
        log.clone(),
    )
    .map_err(|err| RuntimeError::Worker(err.to_string()))?;

    let ((), pginfo_result, dcs_result, process_result, ingest_result, ha_result, api_result) = tokio::join!(
        log_worker.run(),
        pginfo.worker.run(),
        dcs.worker.run(),
        process.worker.run(),
        crate::logging::postgres_ingest::run(crate::logging::postgres_ingest::build_ctx(
            cfg.clone(),
            log.clone(),
        )),
        crate::ha::worker::run(ha.worker),
        crate::api::startup::run(api),
    );

    pginfo_result.map_err(|err| RuntimeError::Worker(err.to_string()))?;
    dcs_result.map_err(|err| RuntimeError::Worker(err.to_string()))?;
    process_result.map_err(|err| RuntimeError::Worker(err.to_string()))?;
    ingest_result.map_err(|err| RuntimeError::Worker(err.to_string()))?;
    ha_result.map_err(|err| RuntimeError::Worker(err.to_string()))?;
    api_result.map_err(|err| RuntimeError::Worker(err.to_string()))?;

    Ok(())
}
