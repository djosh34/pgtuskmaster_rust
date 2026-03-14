use std::{path::Path, time::Duration};

use thiserror::Error;

use crate::{
    config::{load_runtime_config, validate_runtime_config, ConfigError, RuntimeConfig},
    logging::{AppEvent, AppEventHeader, SeverityText, StructuredFields},
    process::state::ProcessRuntimePlan,
    state::{new_state_channel, ClusterName, MemberId, NodeIdentity, ScopeName},
};

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

#[derive(Clone, Copy, Debug)]
enum RuntimeEventKind {
    StartupEntered,
}

impl RuntimeEventKind {
    fn name(self) -> &'static str {
        match self {
            Self::StartupEntered => "runtime.startup.entered",
        }
    }
}

fn runtime_event(
    kind: RuntimeEventKind,
    result: &str,
    severity: SeverityText,
    message: impl Into<String>,
) -> AppEvent {
    AppEvent::new(
        severity,
        message,
        AppEventHeader::new(kind.name(), "runtime", result),
    )
}

fn runtime_base_fields(cfg: &RuntimeConfig, startup_run_id: &str) -> StructuredFields {
    let mut fields = StructuredFields::new();
    fields.insert("scope", cfg.cluster.scope.clone());
    fields.insert("member_id", cfg.cluster.member_id.clone());
    fields.insert("startup_run_id", startup_run_id.to_string());
    fields
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
    let log = logging.handle.clone();
    let startup_run_id = format!(
        "{}-{}",
        cfg.cluster.member_id,
        crate::logging::system_now_unix_millis()
    );
    let mut event = runtime_event(
        RuntimeEventKind::StartupEntered,
        "ok",
        SeverityText::Info,
        "runtime starting",
    );
    let fields = event.fields_mut();
    fields.append_json_map(runtime_base_fields(&cfg, startup_run_id.as_str()).into_attributes());
    fields.insert(
        "logging.level",
        format!("{:?}", cfg.logging.level).to_lowercase(),
    );
    log.emit_app_event("runtime::run_node_from_config", event)
        .map_err(|err| {
            RuntimeError::StartupExecution(format!("runtime start log emit failed: {err}"))
        })?;

    let process_plan = ProcessRuntimePlan::from_config(&cfg);
    process_plan.ensure_start_paths().map_err(|err| {
        RuntimeError::StartupExecution(format!("process start path preparation failed: {err}"))
    })?;

    run_workers(cfg, process_plan, log).await
}

async fn run_workers(
    cfg: RuntimeConfig,
    process_plan: ProcessRuntimePlan,
    log: crate::logging::LogHandle,
) -> Result<(), RuntimeError> {
    let (_cfg_publisher, cfg_subscriber) = new_state_channel(cfg.clone());
    let identity = NodeIdentity {
        cluster_name: ClusterName(cfg.cluster.name.clone()),
        scope: ScopeName(cfg.cluster.scope.clone()),
        member_id: MemberId(cfg.cluster.member_id.clone()),
    };
    let worker_poll_interval = Duration::from_millis(cfg.ha.loop_interval_ms);

    let pginfo = crate::pginfo::startup::bootstrap(crate::pginfo::startup::PgInfoRuntimeRequest {
        self_id: identity.member_id.clone(),
        probe: crate::pginfo::state::PgProbeTarget::local_from_config(&cfg, &process_plan),
        poll_interval: worker_poll_interval,
        log: log.clone(),
    });

    let dcs = crate::dcs::startup::bootstrap(crate::dcs::startup::DcsRuntimeRequest {
        identity: identity.clone(),
        endpoints: cfg.dcs.endpoints.clone(),
        client: cfg.dcs.client.clone(),
        poll_interval: worker_poll_interval,
        member_ttl_ms: cfg.ha.lease_ttl_ms,
        advertised: crate::dcs::startup::DcsAdvertisedEndpoints::from_config(&cfg),
        pg_subscriber: pginfo.state.clone(),
        log: log.clone(),
    })
    .map_err(|err| RuntimeError::Worker(format!("dcs store connect failed: {err}")))?;

    let process =
        crate::process::startup::bootstrap(crate::process::startup::ProcessRuntimeRequest {
            identity: identity.clone(),
            runtime_config: cfg_subscriber.clone(),
            dcs_subscriber: dcs.state.clone(),
            plan: process_plan,
            config: cfg.process.clone(),
            capture_subprocess_output: cfg.logging.capture_subprocess_output,
            log: log.clone(),
        });

    let ha = crate::ha::startup::bootstrap(crate::ha::startup::HaRuntimeRequest {
        identity: identity.clone(),
        poll_interval: worker_poll_interval,
        config_subscriber: cfg_subscriber.clone(),
        pg_subscriber: pginfo.state.clone(),
        dcs_subscriber: dcs.state.clone(),
        process_subscriber: process.state.clone(),
        process_control: process.control.clone(),
        dcs_handle: dcs.handle.clone(),
    });

    let api = crate::api::startup::bootstrap(crate::api::startup::ApiRuntimeRequest {
        identity,
        runtime_config: cfg_subscriber,
        dcs_handle: dcs.handle.clone(),
        observed_state: crate::api::worker::ApiObservedState::Live {
            pg: pginfo.state.clone(),
            process: process.state.clone(),
            dcs: dcs.state.clone(),
            ha: ha.state.clone(),
        },
        log: log.clone(),
    })
    .map_err(|err| RuntimeError::Worker(err.to_string()))?;

    tokio::try_join!(
        pginfo.worker.run(),
        dcs.worker.run(),
        process.worker.run(),
        crate::logging::postgres_ingest::run(crate::logging::postgres_ingest::build_ctx(
            cfg.clone(),
            log.clone(),
        )),
        ha.worker.run(),
        api.worker.run(),
    )
    .map_err(|err| RuntimeError::Worker(err.to_string()))?;

    Ok(())
}
