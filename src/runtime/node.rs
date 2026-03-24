use std::path::Path;

use thiserror::Error;

use crate::{
    config_v2::{load_runtime_config, ConfigErrorV2, RuntimeConfigV2},
    process::state::ensure_start_paths,
};

use super::log_event::RuntimeLogEvent;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("config error: {0}")]
    Config(#[from] ConfigErrorV2),
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

pub async fn run_node_from_config_path(path: &Path) -> Result<(), RuntimeError> {
    let cfg = load_runtime_config(path)?;
    run_node_from_config(cfg).await
}

pub(crate) async fn run_node_from_config(cfg: RuntimeConfigV2) -> Result<(), RuntimeError> {
    let cfg = Box::leak(Box::new(cfg));
    let logging = crate::logging::bootstrap(cfg).map_err(|err| {
        RuntimeError::StartupExecution(format!("logging bootstrap failed: {err}"))
    })?;
    let log = logging.sender.clone();
    let worker = logging.worker;
    let startup_run_id = format!(
        "{}-{}",
        cfg.member_id.as_str(),
        crate::logging::system_now_unix_millis()
    );
    log.send(RuntimeLogEvent::StartupEntered {
        startup_run_id: startup_run_id.to_string(),
        logging_level: runtime_log_level(&cfg.logging.level).to_string(),
    })
    .map_err(|err| {
        RuntimeError::StartupExecution(format!("runtime start log emit failed: {err}"))
    })?;

    ensure_start_paths(cfg).map_err(|err| {
        RuntimeError::StartupExecution(format!("process start path preparation failed: {err}"))
    })?;

    run_workers(cfg, log, worker).await
}

async fn run_workers(
    cfg: &'static RuntimeConfigV2,
    log: crate::logging::LogSender,
    log_worker: crate::logging::LogWorker,
) -> Result<(), RuntimeError> {
    let (pginfo_worker, pginfo_state) = crate::pginfo::worker::bootstrap(cfg, log.clone());

    let (dcs_state, dcs_handle, dcs_worker) =
        crate::dcs::worker::bootstrap(cfg, pginfo_state.clone(), log.clone())
            .map_err(|err| RuntimeError::Worker(format!("dcs store connect failed: {err}")))?;

    let (process_worker, process_state, process_intents) = crate::process::worker::bootstrap(
        cfg,
        crate::process::state::ProcessObservedState {
            dcs: dcs_state.clone(),
        },
        log.clone(),
    );

    let (ha_worker, ha_state) = crate::ha::worker::bootstrap(
        cfg,
        crate::ha::state::HaObservedState {
            pg: pginfo_state.clone(),
            dcs: dcs_state.clone(),
            process: process_state.clone(),
        },
        crate::ha::state::HaControlPlane {
            process_intent_inbox: process_intents.clone(),
            dcs_handle: dcs_handle.clone(),
        },
    );

    let api = crate::api::worker::ApiRuntimeCtx::new(
        cfg,
        dcs_handle.clone(),
        crate::api::worker::ApiObservedState::Live {
            pg: pginfo_state.clone(),
            process: process_state.clone(),
            dcs: dcs_state.clone(),
            ha: ha_state.clone(),
        },
        log.clone(),
    )
    .map_err(|err| RuntimeError::Worker(err.to_string()))?;

    let ((), pginfo_result, dcs_result, process_result, ingest_result, ha_result, api_result) = tokio::join!(
        log_worker.run(),
        crate::pginfo::worker::run(pginfo_worker),
        crate::dcs::run(dcs_worker),
        crate::process::worker::run(process_worker),
        crate::logging::postgres_ingest::run(crate::logging::postgres_ingest::build_ctx(
            cfg,
            log.clone()
        )),
        crate::ha::worker::run(ha_worker),
        crate::api::worker::run(api),
    );

    pginfo_result.map_err(|err| RuntimeError::Worker(err.to_string()))?;
    dcs_result.map_err(|err| RuntimeError::Worker(err.to_string()))?;
    process_result.map_err(|err| RuntimeError::Worker(err.to_string()))?;
    ingest_result.map_err(|err| RuntimeError::Worker(err.to_string()))?;
    ha_result.map_err(|err| RuntimeError::Worker(err.to_string()))?;
    api_result.map_err(|err| RuntimeError::Worker(err.to_string()))?;

    Ok(())
}

fn runtime_log_level(level: &crate::config_v2::types::LogLevel) -> &'static str {
    match level {
        crate::config_v2::types::LogLevel::Trace => "trace",
        crate::config_v2::types::LogLevel::Debug => "debug",
        crate::config_v2::types::LogLevel::Info => "info",
        crate::config_v2::types::LogLevel::Warn => "warn",
        crate::config_v2::types::LogLevel::Error => "error",
        crate::config_v2::types::LogLevel::Fatal => "fatal",
    }
}
