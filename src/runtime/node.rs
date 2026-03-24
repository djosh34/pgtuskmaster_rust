use std::{fs, path::Path};

use thiserror::Error;

use crate::config_v2::{load_runtime_config, ConfigErrorV2, RuntimeConfigV2};

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

    prepare_runtime_start_paths(cfg)?;

    run_workers(cfg, log, worker).await
}

fn prepare_runtime_start_paths(cfg: &RuntimeConfigV2) -> Result<(), RuntimeError> {
    let data_dir = &cfg.postgres.data_dir;
    if let Some(parent) = data_dir.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            RuntimeError::StartupExecution(format!(
                "failed to create postgres data dir parent `{}`: {err}",
                parent.display()
            ))
        })?;
    }

    fs::create_dir_all(data_dir).map_err(|err| {
        RuntimeError::StartupExecution(format!(
            "failed to create postgres data dir `{}`: {err}",
            data_dir.display()
        ))
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(data_dir, fs::Permissions::from_mode(0o700)).map_err(|err| {
            RuntimeError::StartupExecution(format!(
                "failed to set postgres data dir permissions on `{}`: {err}",
                data_dir.display()
            ))
        })?;
    }

    fs::create_dir_all(&cfg.postgres.socket_dir).map_err(|err| {
        RuntimeError::StartupExecution(format!(
            "failed to create postgres socket dir `{}`: {err}",
            cfg.postgres.socket_dir.display()
        ))
    })?;

    if let Some(log_parent) = cfg.postgres.log_file.parent() {
        fs::create_dir_all(log_parent).map_err(|err| {
            RuntimeError::StartupExecution(format!(
                "failed to create postgres log dir `{}`: {err}",
                log_parent.display()
            ))
        })?;
    }

    Ok(())
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

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::{
        config_v2::runtime_test_config_with_data_dir, dev_support::test_fs::unique_test_dir,
    };

    use super::{prepare_runtime_start_paths, RuntimeError};

    #[test]
    fn prepare_runtime_start_paths_creates_required_directories() -> Result<(), String> {
        let root = unique_test_dir("runtime-node", "prepare-start-paths")?;
        let data_dir = root.join("pg").join("data");
        let socket_dir = root.join("run").join("socket");
        let log_file = root.join("logs").join("postgres.log");
        let cfg =
            runtime_test_config_with_data_dir(data_dir.clone()).map_err(|err| err.to_string())?;
        let cfg = crate::config_v2::RuntimeConfigV2 {
            postgres: crate::config_v2::types::PostgresConfig {
                socket_dir: socket_dir.clone(),
                log_file: log_file.clone(),
                ..cfg.postgres
            },
            ..cfg
        };

        prepare_runtime_start_paths(&cfg).map_err(|err| err.to_string())?;

        if !data_dir.is_dir() {
            return Err(format!(
                "expected data dir to exist at {}",
                data_dir.display()
            ));
        }
        if !socket_dir.is_dir() {
            return Err(format!(
                "expected socket dir to exist at {}",
                socket_dir.display()
            ));
        }
        let log_parent = log_file
            .parent()
            .ok_or_else(|| format!("expected log parent for {}", log_file.display()))?;
        if !log_parent.is_dir() {
            return Err(format!(
                "expected log dir to exist at {}",
                log_parent.display()
            ));
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let mode = fs::metadata(&data_dir)
                .map_err(|err| format!("metadata {} failed: {err}", data_dir.display()))?
                .permissions()
                .mode()
                & 0o777;
            if mode != 0o700 {
                return Err(format!(
                    "expected {} permissions 0o700, observed {mode:o}",
                    data_dir.display()
                ));
            }
        }

        Ok(())
    }

    #[test]
    fn prepare_runtime_start_paths_reports_socket_dir_failures() -> Result<(), String> {
        let root = unique_test_dir("runtime-node", "prepare-start-paths-error")?;
        let data_dir = root.join("pg").join("data");
        let socket_dir = root.join("socket-file");
        fs::write(&socket_dir, "occupied")
            .map_err(|err| format!("write {} failed: {err}", socket_dir.display()))?;
        let cfg = runtime_test_config_with_data_dir(data_dir).map_err(|err| err.to_string())?;
        let cfg = crate::config_v2::RuntimeConfigV2 {
            postgres: crate::config_v2::types::PostgresConfig {
                socket_dir: socket_dir.clone(),
                ..cfg.postgres
            },
            ..cfg
        };

        let error = prepare_runtime_start_paths(&cfg).err().ok_or_else(|| {
            format!(
                "expected startup path preparation to fail for {}",
                socket_dir.display()
            )
        })?;
        match error {
            RuntimeError::StartupExecution(message) => {
                if !message.contains("failed to create postgres socket dir") {
                    return Err(format!("unexpected startup error: {message}"));
                }
            }
            other => return Err(format!("unexpected error variant: {other}")),
        }

        Ok(())
    }
}
