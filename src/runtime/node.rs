use std::{
    collections::BTreeMap,
    fs,
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use thiserror::Error;
use tokio::{net::TcpListener, sync::mpsc};

use crate::{
    api::worker::ApiWorkerCtx,
    config::{load_runtime_config, validate_runtime_config, ConfigError, RuntimeConfig},
    dcs::{
        etcd_store::EtcdDcsStore,
        state::{DcsCache, DcsState, DcsTrust, DcsWorkerCtx, MemberRole},
        store::{refresh_from_etcd_watch, DcsStore},
    },
    debug_api::{
        snapshot::{build_snapshot, AppLifecycle, DebugSnapshotCtx},
        worker::{DebugApiContractStubInputs, DebugApiCtx},
    },
    ha::state::{
        HaPhase, HaState, HaWorkerContractStubInputs, HaWorkerCtx, ProcessDispatchDefaults,
    },
    logging::{
        EventMeta, LogParser, LogProducer, LogRecord, LogSource, LogTransport, SeverityText,
    },
    pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
    process::{
        jobs::{
            BaseBackupSpec, BootstrapSpec, ProcessCommandRunner, ProcessExit, ReplicatorSourceConn,
            RewinderSourceConn, StartPostgresSpec,
        },
        state::{ProcessJobKind, ProcessState, ProcessWorkerCtx},
        worker::{build_command, system_now_unix_millis, timeout_for_kind, TokioCommandRunner},
    },
    state::{new_state_channel, MemberId, UnixMillis, WorkerStatus},
};

#[derive(Clone, Debug)]
enum StartupAction {
    ClaimInitLockAndSeedConfig,
    RunJob(Box<ProcessJobKind>),
    TakeoverRestoredDataDir,
    StartPostgres,
}

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
        listen_addr: String,
        message: String,
    },
    #[error("worker failed: {0}")]
    Worker(String),
    #[error("time error: {0}")]
    Time(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum StartupMode {
    InitializePrimary,
    RestoreBootstrap,
    CloneReplica {
        leader_member_id: MemberId,
        source: ReplicatorSourceConn,
    },
    ResumeExisting,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DataDirState {
    Missing,
    Empty,
    Existing,
}

pub async fn run_node_from_config_path(path: &Path) -> Result<(), RuntimeError> {
    let cfg = load_runtime_config(path)?;
    run_node_from_config(cfg).await
}

pub async fn run_node_from_config(cfg: RuntimeConfig) -> Result<(), RuntimeError> {
    validate_runtime_config(&cfg)?;
    crate::self_exe::init_from_current_exe().map_err(|err| {
        RuntimeError::StartupExecution(format!("self executable path init failed: {err}"))
    })?;

    if cfg.backup.enabled {
        match cfg.backup.provider {
            crate::config::BackupProvider::Pgbackrest => {
                crate::backup::worker::validate_pgbackrest_enabled_config(&cfg).map_err(|err| {
                    RuntimeError::StartupExecution(format!(
                        "backup provider configuration failed validation: {err}"
                    ))
                })?;
            }
        }
    }

    let logging = crate::logging::bootstrap(&cfg).map_err(|err| {
        RuntimeError::StartupExecution(format!("logging bootstrap failed: {err}"))
    })?;
    let log = logging.handle.clone();
    let startup_run_id = format!(
        "{}-{}",
        cfg.cluster.member_id,
        crate::logging::system_now_unix_millis()
    );
    let mut start_attrs = BTreeMap::new();
    start_attrs.insert(
        "scope".to_string(),
        serde_json::Value::String(cfg.dcs.scope.clone()),
    );
    start_attrs.insert(
        "member_id".to_string(),
        serde_json::Value::String(cfg.cluster.member_id.clone()),
    );
    start_attrs.insert(
        "startup_run_id".to_string(),
        serde_json::Value::String(startup_run_id.clone()),
    );
    start_attrs.insert(
        "logging.level".to_string(),
        serde_json::Value::String(format!("{:?}", cfg.logging.level).to_lowercase()),
    );
    log.emit_event(
        SeverityText::Info,
        "runtime starting",
        "runtime::run_node_from_config",
        EventMeta::new("runtime.startup.entered", "runtime", "ok"),
        start_attrs,
    )
    .map_err(|err| {
        RuntimeError::StartupExecution(format!("runtime start log emit failed: {err}"))
    })?;

    let process_defaults = process_defaults_from_config(&cfg);
    let startup_mode = plan_startup(&cfg, &process_defaults, &log, startup_run_id.as_str())?;
    execute_startup(
        &cfg,
        &process_defaults,
        &startup_mode,
        &log,
        startup_run_id.as_str(),
    )
    .await?;

    run_workers(cfg, process_defaults, log).await
}

// ?!?!?! WHY LIKE THIS?
fn process_defaults_from_config(cfg: &RuntimeConfig) -> ProcessDispatchDefaults {
    let basebackup_source = ReplicatorSourceConn {
        conninfo: crate::pginfo::state::PgConnInfo {
            host: cfg.postgres.rewind_source_host.clone(),
            port: cfg.postgres.rewind_source_port,
            user: cfg.postgres.roles.replicator.username.clone(),
            dbname: cfg.postgres.rewind_conn_identity.dbname.clone(),
            application_name: None,
            connect_timeout_s: Some(cfg.postgres.connect_timeout_s),
            ssl_mode: cfg.postgres.rewind_conn_identity.ssl_mode,
            options: None,
        },
        auth: cfg.postgres.roles.replicator.auth.clone(),
    };

    let rewind_source = RewinderSourceConn {
        conninfo: crate::pginfo::state::PgConnInfo {
            host: cfg.postgres.rewind_source_host.clone(),
            port: cfg.postgres.rewind_source_port,
            user: cfg.postgres.roles.rewinder.username.clone(),
            dbname: cfg.postgres.rewind_conn_identity.dbname.clone(),
            application_name: None,
            connect_timeout_s: Some(cfg.postgres.connect_timeout_s),
            ssl_mode: cfg.postgres.rewind_conn_identity.ssl_mode,
            options: None,
        },
        auth: cfg.postgres.roles.rewinder.auth.clone(),
    };

    ProcessDispatchDefaults {
        postgres_host: cfg.postgres.listen_host.clone(),
        postgres_port: cfg.postgres.listen_port,
        socket_dir: cfg.postgres.socket_dir.clone(),
        log_file: cfg.postgres.log_file.clone(),
        basebackup_source,
        rewind_source,
        shutdown_mode: crate::process::jobs::ShutdownMode::Fast,
    }
}

fn plan_startup(
    cfg: &RuntimeConfig,
    process_defaults: &ProcessDispatchDefaults,
    log: &crate::logging::LogHandle,
    startup_run_id: &str,
) -> Result<StartupMode, RuntimeError> {
    plan_startup_with_probe(cfg, process_defaults, log, startup_run_id, probe_dcs_cache)
}

fn plan_startup_with_probe(
    cfg: &RuntimeConfig,
    process_defaults: &ProcessDispatchDefaults,
    log: &crate::logging::LogHandle,
    startup_run_id: &str,
    probe: impl Fn(&RuntimeConfig) -> Result<DcsCache, RuntimeError>,
) -> Result<StartupMode, RuntimeError> {
    let data_dir_state = match inspect_data_dir(&cfg.postgres.data_dir) {
        Ok(value) => {
            let mut attrs = BTreeMap::new();
            attrs.insert(
                "scope".to_string(),
                serde_json::Value::String(cfg.dcs.scope.clone()),
            );
            attrs.insert(
                "member_id".to_string(),
                serde_json::Value::String(cfg.cluster.member_id.clone()),
            );
            attrs.insert(
                "startup_run_id".to_string(),
                serde_json::Value::String(startup_run_id.to_string()),
            );
            attrs.insert(
                "postgres.data_dir".to_string(),
                serde_json::Value::String(cfg.postgres.data_dir.display().to_string()),
            );
            attrs.insert(
                "data_dir_state".to_string(),
                serde_json::Value::String(format!("{value:?}").to_lowercase()),
            );
            log.emit_event(
                SeverityText::Debug,
                "data dir inspected",
                "runtime::plan_startup",
                EventMeta::new("runtime.startup.data_dir.inspected", "runtime", "ok"),
                attrs,
            )
            .map_err(|err| {
                RuntimeError::StartupPlanning(format!("data dir inspection log emit failed: {err}"))
            })?;
            value
        }
        Err(err) => {
            let mut attrs = BTreeMap::new();
            attrs.insert(
                "scope".to_string(),
                serde_json::Value::String(cfg.dcs.scope.clone()),
            );
            attrs.insert(
                "member_id".to_string(),
                serde_json::Value::String(cfg.cluster.member_id.clone()),
            );
            attrs.insert(
                "startup_run_id".to_string(),
                serde_json::Value::String(startup_run_id.to_string()),
            );
            attrs.insert(
                "postgres.data_dir".to_string(),
                serde_json::Value::String(cfg.postgres.data_dir.display().to_string()),
            );
            attrs.insert(
                "error".to_string(),
                serde_json::Value::String(err.to_string()),
            );
            log.emit_event(
                SeverityText::Error,
                "data dir inspection failed",
                "runtime::plan_startup",
                EventMeta::new("runtime.startup.data_dir.inspected", "runtime", "failed"),
                attrs,
            )
            .map_err(|emit_err| {
                RuntimeError::StartupPlanning(format!(
                    "data dir inspection log emit failed: {emit_err}"
                ))
            })?;
            return Err(err);
        }
    };

    let cache = match probe(cfg) {
        Ok(cache) => {
            let mut attrs = BTreeMap::new();
            attrs.insert(
                "scope".to_string(),
                serde_json::Value::String(cfg.dcs.scope.clone()),
            );
            attrs.insert(
                "member_id".to_string(),
                serde_json::Value::String(cfg.cluster.member_id.clone()),
            );
            attrs.insert(
                "startup_run_id".to_string(),
                serde_json::Value::String(startup_run_id.to_string()),
            );
            attrs.insert(
                "dcs_probe_status".to_string(),
                serde_json::Value::String("ok".to_string()),
            );
            log.emit_event(
                SeverityText::Info,
                "startup dcs cache probe ok",
                "runtime::plan_startup",
                EventMeta::new("runtime.startup.dcs_cache_probe", "runtime", "ok"),
                attrs,
            )
            .map_err(|err| {
                RuntimeError::StartupPlanning(format!("dcs cache probe log emit failed: {err}"))
            })?;
            Some(cache)
        }
        Err(err) => {
            let mut attrs = BTreeMap::new();
            attrs.insert(
                "scope".to_string(),
                serde_json::Value::String(cfg.dcs.scope.clone()),
            );
            attrs.insert(
                "member_id".to_string(),
                serde_json::Value::String(cfg.cluster.member_id.clone()),
            );
            attrs.insert(
                "startup_run_id".to_string(),
                serde_json::Value::String(startup_run_id.to_string()),
            );
            attrs.insert(
                "error".to_string(),
                serde_json::Value::String(err.to_string()),
            );
            attrs.insert(
                "dcs_probe_status".to_string(),
                serde_json::Value::String("failed".to_string()),
            );
            log.emit_event(
                SeverityText::Warn,
                "startup dcs cache probe failed; continuing without cache",
                "runtime::plan_startup",
                EventMeta::new("runtime.startup.dcs_cache_probe", "runtime", "failed"),
                attrs,
            )
            .map_err(|emit_err| {
                RuntimeError::StartupPlanning(format!(
                    "dcs cache probe log emit failed: {emit_err}"
                ))
            })?;
            None
        }
    };

    let startup_mode = select_startup_mode(
        data_dir_state,
        cache.as_ref(),
        &cfg.cluster.member_id,
        process_defaults,
        cfg.postgres.connect_timeout_s,
        cfg.backup.bootstrap.enabled,
    )?;

    let mut attrs = BTreeMap::new();
    attrs.insert(
        "scope".to_string(),
        serde_json::Value::String(cfg.dcs.scope.clone()),
    );
    attrs.insert(
        "member_id".to_string(),
        serde_json::Value::String(cfg.cluster.member_id.clone()),
    );
    attrs.insert(
        "startup_run_id".to_string(),
        serde_json::Value::String(startup_run_id.to_string()),
    );
    attrs.insert(
        "startup_mode".to_string(),
        serde_json::Value::String(format!("{startup_mode:?}").to_lowercase()),
    );
    log.emit_event(
        SeverityText::Info,
        "startup mode selected",
        "runtime::plan_startup",
        EventMeta::new("runtime.startup.mode_selected", "runtime", "ok"),
        attrs,
    )
    .map_err(|err| RuntimeError::StartupPlanning(format!("startup mode log emit failed: {err}")))?;

    Ok(startup_mode)
}

fn inspect_data_dir(data_dir: &Path) -> Result<DataDirState, RuntimeError> {
    match fs::metadata(data_dir) {
        Ok(meta) => {
            if !meta.is_dir() {
                return Err(RuntimeError::StartupPlanning(format!(
                    "postgres.data_dir is not a directory: {}",
                    data_dir.display()
                )));
            }
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Ok(DataDirState::Missing);
        }
        Err(err) => {
            return Err(RuntimeError::StartupPlanning(format!(
                "failed to inspect data dir {}: {err}",
                data_dir.display()
            )));
        }
    }

    if data_dir.join("PG_VERSION").exists() {
        return Ok(DataDirState::Existing);
    }

    let mut entries = fs::read_dir(data_dir).map_err(|err| {
        RuntimeError::StartupPlanning(format!(
            "failed to read data dir {}: {err}",
            data_dir.display()
        ))
    })?;

    if entries.next().is_none() {
        Ok(DataDirState::Empty)
    } else {
        Err(RuntimeError::StartupPlanning(format!(
            "ambiguous data dir state: `{}` is non-empty but has no PG_VERSION",
            data_dir.display()
        )))
    }
}

fn probe_dcs_cache(cfg: &RuntimeConfig) -> Result<DcsCache, RuntimeError> {
    let mut store =
        EtcdDcsStore::connect(cfg.dcs.endpoints.clone(), &cfg.dcs.scope).map_err(|err| {
            RuntimeError::StartupPlanning(format!("failed to connect dcs for startup probe: {err}"))
        })?;

    let events = store.drain_watch_events().map_err(|err| {
        RuntimeError::StartupPlanning(format!("failed to read startup dcs events: {err}"))
    })?;

    let mut cache = DcsCache {
        members: BTreeMap::new(),
        leader: None,
        switchover: None,
        config: cfg.clone(),
        init_lock: None,
    };

    refresh_from_etcd_watch(&cfg.dcs.scope, &mut cache, events).map_err(|err| {
        RuntimeError::StartupPlanning(format!("failed to decode startup dcs snapshot: {err}"))
    })?;

    Ok(cache)
}

fn select_startup_mode(
    data_dir_state: DataDirState,
    cache: Option<&DcsCache>,
    self_member_id: &str,
    process_defaults: &ProcessDispatchDefaults,
    connect_timeout_s: u32,
    restore_bootstrap_enabled: bool,
) -> Result<StartupMode, RuntimeError> {
    match data_dir_state {
        DataDirState::Existing => Ok(StartupMode::ResumeExisting),
        DataDirState::Missing | DataDirState::Empty => {
            let init_lock_present = cache
                .and_then(|snapshot| snapshot.init_lock.as_ref())
                .is_some();

            let leader_from_leader_key = cache.and_then(|snapshot| {
                let leader_record = snapshot.leader.as_ref()?;
                if leader_record.member_id.0 == self_member_id {
                    return None;
                }
                let member = snapshot.members.get(&leader_record.member_id)?;
                let eligible =
                    member.role == MemberRole::Primary && member.sql == SqlStatus::Healthy;
                if eligible {
                    Some(leader_record.member_id.clone())
                } else {
                    None
                }
            });

            let leader_from_members = cache.and_then(|snapshot| {
                if !init_lock_present {
                    return None;
                }
                snapshot
                    .members
                    .values()
                    .find(|member| {
                        member.member_id.0 != self_member_id
                            && member.role == MemberRole::Primary
                            && member.sql == SqlStatus::Healthy
                    })
                    .map(|member| member.member_id.clone())
            });

            let leader = leader_from_leader_key.or(leader_from_members);

            match leader {
                Some(leader_member_id) => Ok(StartupMode::CloneReplica {
                    leader_member_id,
                    source: default_leader_source(process_defaults, connect_timeout_s),
                }),
                None => {
                    if init_lock_present {
                        Err(RuntimeError::StartupPlanning(
                            "cluster is already initialized (dcs init lock present) but no healthy primary is available for basebackup"
                                .to_string(),
                        ))
                    } else if restore_bootstrap_enabled {
                        Ok(StartupMode::RestoreBootstrap)
                    } else {
                        Ok(StartupMode::InitializePrimary)
                    }
                }
            }
        }
    }
}

fn default_leader_source(
    process_defaults: &ProcessDispatchDefaults,
    connect_timeout_s: u32,
) -> ReplicatorSourceConn {
    let mut source = process_defaults.basebackup_source.clone();
    source.conninfo.connect_timeout_s = Some(connect_timeout_s);
    source
}

fn claim_dcs_init_lock_and_seed_config(cfg: &RuntimeConfig) -> Result<(), String> {
    let init_path = format!("/{}/init", cfg.dcs.scope.trim_matches('/'));
    let config_path = format!("/{}/config", cfg.dcs.scope.trim_matches('/'));

    let mut store = EtcdDcsStore::connect(cfg.dcs.endpoints.clone(), &cfg.dcs.scope)
        .map_err(|err| format!("connect failed: {err}"))?;

    let encoded_init = serde_json::to_string(&crate::dcs::state::InitLockRecord {
        holder: MemberId(cfg.cluster.member_id.clone()),
    })
    .map_err(|err| format!("encode init lock record failed: {err}"))?;

    let claimed = store
        .put_path_if_absent(init_path.as_str(), encoded_init)
        .map_err(|err| format!("init lock write failed at `{init_path}`: {err}"))?;
    if !claimed {
        return Err(format!(
            "cluster already initialized (init lock exists at `{init_path}`)"
        ));
    }

    if let Some(init_cfg) = cfg.dcs.init.as_ref() {
        if init_cfg.write_on_bootstrap {
            let _seeded = store
                .put_path_if_absent(config_path.as_str(), init_cfg.payload_json.clone())
                .map_err(|err| format!("seed config failed at `{config_path}`: {err}"))?;
        }
    }

    Ok(())
}

async fn execute_startup(
    cfg: &RuntimeConfig,
    process_defaults: &ProcessDispatchDefaults,
    startup_mode: &StartupMode,
    log: &crate::logging::LogHandle,
    startup_run_id: &str,
) -> Result<(), RuntimeError> {
    ensure_start_paths(process_defaults, &cfg.postgres.data_dir)?;

    let actions = build_startup_actions(cfg, startup_mode)?;

    let mut planned_attrs = BTreeMap::new();
    planned_attrs.insert(
        "scope".to_string(),
        serde_json::Value::String(cfg.dcs.scope.clone()),
    );
    planned_attrs.insert(
        "member_id".to_string(),
        serde_json::Value::String(cfg.cluster.member_id.clone()),
    );
    planned_attrs.insert(
        "startup_run_id".to_string(),
        serde_json::Value::String(startup_run_id.to_string()),
    );
    planned_attrs.insert(
        "startup_mode".to_string(),
        serde_json::Value::String(format!("{startup_mode:?}").to_lowercase()),
    );
    planned_attrs.insert(
        "startup_actions_total".to_string(),
        serde_json::Value::Number(serde_json::Number::from(actions.len() as u64)),
    );
    log.emit_event(
        SeverityText::Debug,
        "startup actions planned",
        "runtime::execute_startup",
        EventMeta::new("runtime.startup.actions_planned", "runtime", "ok"),
        planned_attrs,
    )
    .map_err(|err| {
        RuntimeError::StartupExecution(format!("startup actions log emit failed: {err}"))
    })?;

    for (action_index, action) in actions.into_iter().enumerate() {
        let action_kind = match &action {
            StartupAction::ClaimInitLockAndSeedConfig => "claim_init_lock_and_seed_config",
            StartupAction::RunJob(_) => "run_job",
            StartupAction::TakeoverRestoredDataDir => "takeover_restored_data_dir",
            StartupAction::StartPostgres => "start_postgres",
        };
        let mut step_attrs = BTreeMap::new();
        step_attrs.insert(
            "scope".to_string(),
            serde_json::Value::String(cfg.dcs.scope.clone()),
        );
        step_attrs.insert(
            "member_id".to_string(),
            serde_json::Value::String(cfg.cluster.member_id.clone()),
        );
        step_attrs.insert(
            "startup_run_id".to_string(),
            serde_json::Value::String(startup_run_id.to_string()),
        );
        step_attrs.insert(
            "startup_mode".to_string(),
            serde_json::Value::String(format!("{startup_mode:?}").to_lowercase()),
        );
        step_attrs.insert(
            "startup_action_index".to_string(),
            serde_json::Value::Number(serde_json::Number::from(action_index as u64)),
        );
        step_attrs.insert(
            "startup_action_kind".to_string(),
            serde_json::Value::String(action_kind.to_string()),
        );
        log.emit_event(
            SeverityText::Info,
            "startup action started",
            "runtime::execute_startup",
            EventMeta::new("runtime.startup.action", "runtime", "started"),
            step_attrs.clone(),
        )
        .map_err(|err| {
            RuntimeError::StartupExecution(format!("startup action log emit failed: {err}"))
        })?;

        match &action {
            StartupAction::RunJob(job)
                if matches!(job.as_ref(), ProcessJobKind::PgBackRestRestore(_)) =>
            {
                emit_startup_phase(log, "restore", "pgbackrest restore").map_err(|err| {
                    RuntimeError::StartupExecution(format!("startup phase log emit failed: {err}"))
                })?;
            }
            StartupAction::TakeoverRestoredDataDir => {
                emit_startup_phase(log, "takeover", "managed pre-recovery takeover").map_err(
                    |err| {
                        RuntimeError::StartupExecution(format!(
                            "startup phase log emit failed: {err}"
                        ))
                    },
                )?;
            }
            StartupAction::StartPostgres => {
                emit_startup_phase(log, "start", "start postgres with managed config").map_err(
                    |err| {
                        RuntimeError::StartupExecution(format!(
                            "startup phase log emit failed: {err}"
                        ))
                    },
                )?;
            }
            _ => {}
        }

        let result = match action {
            StartupAction::ClaimInitLockAndSeedConfig => {
                claim_dcs_init_lock_and_seed_config(cfg).map_err(|err| {
                    RuntimeError::StartupExecution(format!("dcs init lock claim failed: {err}"))
                })?;
                Ok(())
            }
            StartupAction::RunJob(job) => run_startup_job(cfg, *job, log).await,
            StartupAction::TakeoverRestoredDataDir => {
                crate::postgres_managed::takeover_restored_data_dir(
                    cfg,
                    cfg.backup.bootstrap.takeover_policy,
                    true,
                )
                .map_err(|err| {
                    RuntimeError::StartupExecution(format!(
                        "takeover restored data dir failed: {err}"
                    ))
                })?;
                Ok(())
            }
            StartupAction::StartPostgres => run_start_job(cfg, process_defaults, log).await,
        };

        match result {
            Ok(()) => {
                let done_attrs = step_attrs;
                log.emit_event(
                    SeverityText::Info,
                    "startup action completed",
                    "runtime::execute_startup",
                    EventMeta::new("runtime.startup.action", "runtime", "ok"),
                    done_attrs,
                )
                .map_err(|err| {
                    RuntimeError::StartupExecution(format!("startup action log emit failed: {err}"))
                })?;
            }
            Err(err) => {
                let mut failed_attrs = step_attrs;
                failed_attrs.insert(
                    "error".to_string(),
                    serde_json::Value::String(err.to_string()),
                );
                log.emit_event(
                    SeverityText::Error,
                    "startup action failed",
                    "runtime::execute_startup",
                    EventMeta::new("runtime.startup.action", "runtime", "failed"),
                    failed_attrs,
                )
                .map_err(|emit_err| {
                    RuntimeError::StartupExecution(format!(
                        "startup action failure log emit failed: {emit_err}"
                    ))
                })?;
                return Err(err);
            }
        };
    }

    Ok(())
}

fn emit_startup_phase(
    log: &crate::logging::LogHandle,
    phase: &str,
    detail: &str,
) -> Result<(), crate::logging::LogError> {
    log.emit(
        SeverityText::Info,
        format!("startup phase={phase} ({detail})"),
        LogSource {
            producer: LogProducer::App,
            transport: LogTransport::Internal,
            parser: LogParser::App,
            origin: "startup".to_string(),
        },
    )
}

fn build_startup_actions(
    cfg: &RuntimeConfig,
    startup_mode: &StartupMode,
) -> Result<Vec<StartupAction>, RuntimeError> {
    match startup_mode {
        StartupMode::InitializePrimary => Ok(vec![
            StartupAction::ClaimInitLockAndSeedConfig,
            StartupAction::RunJob(Box::new(ProcessJobKind::Bootstrap(BootstrapSpec {
                data_dir: cfg.postgres.data_dir.clone(),
                superuser_username: cfg.postgres.roles.superuser.username.clone(),
                timeout_ms: Some(cfg.process.bootstrap_timeout_ms),
            }))),
            StartupAction::StartPostgres,
        ]),
        StartupMode::RestoreBootstrap => {
            let restore_job = crate::backup::worker::pgbackrest_restore_job(
                cfg,
                crate::state::JobId("startup-restore".to_string()),
            )
            .map_err(|err| {
                RuntimeError::StartupPlanning(format!("build pgbackrest restore job failed: {err}"))
            })?;
            Ok(vec![
                StartupAction::ClaimInitLockAndSeedConfig,
                StartupAction::RunJob(Box::new(restore_job.kind)),
                StartupAction::TakeoverRestoredDataDir,
                StartupAction::StartPostgres,
            ])
        }
        StartupMode::CloneReplica { source, .. } => Ok(vec![
            StartupAction::RunJob(Box::new(ProcessJobKind::BaseBackup(BaseBackupSpec {
                data_dir: cfg.postgres.data_dir.clone(),
                source: source.clone(),
                timeout_ms: Some(cfg.process.bootstrap_timeout_ms),
            }))),
            StartupAction::StartPostgres,
        ]),
        StartupMode::ResumeExisting => {
            if has_postmaster_pid(&cfg.postgres.data_dir) {
                Ok(Vec::new())
            } else {
                Ok(vec![StartupAction::StartPostgres])
            }
        }
    }
}

fn ensure_start_paths(
    process_defaults: &ProcessDispatchDefaults,
    data_dir: &Path,
) -> Result<(), RuntimeError> {
    if let Some(parent) = data_dir.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            RuntimeError::StartupExecution(format!(
                "failed to create postgres data dir parent `{}`: {err}",
                parent.display()
            ))
        })?;
    }

    fs::create_dir_all(&process_defaults.socket_dir).map_err(|err| {
        RuntimeError::StartupExecution(format!(
            "failed to create postgres socket dir `{}`: {err}",
            process_defaults.socket_dir.display()
        ))
    })?;

    if let Some(log_parent) = process_defaults.log_file.parent() {
        fs::create_dir_all(log_parent).map_err(|err| {
            RuntimeError::StartupExecution(format!(
                "failed to create postgres log dir `{}`: {err}",
                log_parent.display()
            ))
        })?;
    }

    Ok(())
}

fn has_postmaster_pid(data_dir: &Path) -> bool {
    data_dir.join("postmaster.pid").exists()
}

async fn run_start_job(
    cfg: &RuntimeConfig,
    process_defaults: &ProcessDispatchDefaults,
    log: &crate::logging::LogHandle,
) -> Result<(), RuntimeError> {
    let managed =
        crate::postgres_managed::materialize_managed_postgres_config(cfg).map_err(|err| {
            RuntimeError::StartupExecution(format!(
                "materialize managed postgres config failed: {err}"
            ))
        })?;
    run_startup_job(
        cfg,
        ProcessJobKind::StartPostgres(StartPostgresSpec {
            data_dir: cfg.postgres.data_dir.clone(),
            host: process_defaults.postgres_host.clone(),
            port: process_defaults.postgres_port,
            socket_dir: process_defaults.socket_dir.clone(),
            log_file: process_defaults.log_file.clone(),
            extra_postgres_settings: managed.extra_settings,
            wait_seconds: Some(30),
            timeout_ms: Some(cfg.process.bootstrap_timeout_ms),
        }),
        log,
    )
    .await
}

async fn run_startup_job(
    cfg: &RuntimeConfig,
    job: ProcessJobKind,
    log: &crate::logging::LogHandle,
) -> Result<(), RuntimeError> {
    let mut runner = TokioCommandRunner;
    let timeout_ms = timeout_for_kind(&job, &cfg.process);
    let job_id = crate::state::JobId(format!("startup-{}", std::process::id()));
    let command = build_command(
        &cfg.process,
        &job_id,
        &job,
        cfg.logging.capture_subprocess_output,
    )
    .map_err(|err| {
        RuntimeError::StartupExecution(format!("startup command build failed: {err}"))
    })?;
    let log_identity = command.log_identity.clone();
    let command_display = format!("{} {}", command.program.display(), command.args.join(" "));

    let mut handle = runner.spawn(command).map_err(|err| {
        RuntimeError::StartupExecution(format!(
            "startup command spawn failed for `{command_display}`: {err}"
        ))
    })?;

    let started = system_now_unix_millis().map_err(|err| RuntimeError::Time(err.to_string()))?;
    let deadline = started.0.saturating_add(timeout_ms);

    loop {
        let lines = handle.drain_output(256 * 1024).await.map_err(|err| {
            RuntimeError::StartupExecution(format!("startup process output drain failed: {err}"))
        })?;
        for line in lines {
            if let Err(err) = emit_startup_subprocess_line(log, &log_identity, line.clone()) {
                let mut attrs = BTreeMap::new();
                attrs.insert(
                    "job_id".to_string(),
                    serde_json::Value::String(log_identity.job_id.0.clone()),
                );
                attrs.insert(
                    "job_kind".to_string(),
                    serde_json::Value::String(log_identity.job_kind.clone()),
                );
                attrs.insert(
                    "binary".to_string(),
                    serde_json::Value::String(log_identity.binary.clone()),
                );
                attrs.insert(
                    "stream".to_string(),
                    serde_json::Value::String(match line.stream {
                        crate::process::jobs::ProcessOutputStream::Stdout => "stdout".to_string(),
                        crate::process::jobs::ProcessOutputStream::Stderr => "stderr".to_string(),
                    }),
                );
                attrs.insert(
                    "bytes_len".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(line.bytes.len() as u64)),
                );
                attrs.insert(
                    "error".to_string(),
                    serde_json::Value::String(err.to_string()),
                );
                log.emit_event(
                    SeverityText::Warn,
                    "startup subprocess line emit failed",
                    "runtime::run_startup_job",
                    EventMeta::new(
                        "runtime.startup.subprocess_log_emit_failed",
                        "runtime",
                        "failed",
                    ),
                    attrs,
                )
                .map_err(|emit_err| {
                    RuntimeError::StartupExecution(format!(
                        "startup subprocess emit failure log emit failed: {emit_err}"
                    ))
                })?;
            }
        }

        match handle.poll_exit().map_err(|err| {
            RuntimeError::StartupExecution(format!("startup process poll failed: {err}"))
        })? {
            Some(ProcessExit::Success) => return Ok(()),
            Some(ProcessExit::Failure { code }) => {
                let lines = handle.drain_output(256 * 1024).await.map_err(|err| {
                    RuntimeError::StartupExecution(format!(
                        "startup process output drain failed: {err}"
                    ))
                })?;
                for line in lines {
                    emit_startup_subprocess_line(log, &log_identity, line).map_err(|err| {
                        RuntimeError::StartupExecution(format!(
                            "startup subprocess line emit failed: {err}"
                        ))
                    })?;
                }
                return Err(RuntimeError::StartupExecution(format!(
                    "startup command `{command_display}` exited unsuccessfully (code: {code:?})"
                )));
            }
            None => {}
        }

        let now = system_now_unix_millis().map_err(|err| RuntimeError::Time(err.to_string()))?;
        if now.0 >= deadline {
            handle.cancel().await.map_err(|err| {
                RuntimeError::StartupExecution(format!(
                    "startup command `{command_display}` timeout cancellation failed: {err}"
                ))
            })?;
            let lines = handle.drain_output(256 * 1024).await.map_err(|err| {
                RuntimeError::StartupExecution(format!(
                    "startup process output drain failed: {err}"
                ))
            })?;
            for line in lines {
                emit_startup_subprocess_line(log, &log_identity, line).map_err(|err| {
                    RuntimeError::StartupExecution(format!(
                        "startup subprocess line emit failed: {err}"
                    ))
                })?;
            }
            return Err(RuntimeError::StartupExecution(format!(
                "startup command `{command_display}` timed out after {timeout_ms} ms"
            )));
        }

        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

fn emit_startup_subprocess_line(
    log: &crate::logging::LogHandle,
    identity: &crate::process::jobs::ProcessLogIdentity,
    line: crate::process::jobs::ProcessOutputLine,
) -> Result<(), crate::logging::LogError> {
    let (transport, severity) = match line.stream {
        crate::process::jobs::ProcessOutputStream::Stdout => (
            crate::logging::LogTransport::ChildStdout,
            SeverityText::Info,
        ),
        crate::process::jobs::ProcessOutputStream::Stderr => (
            crate::logging::LogTransport::ChildStderr,
            SeverityText::Warn,
        ),
    };

    let (message, raw_bytes_hex) = match String::from_utf8(line.bytes) {
        Ok(message) => (message, None),
        Err(err) => {
            let bytes = err.into_bytes();
            let hex = emit_hex_encode(bytes.as_slice());
            (format!("non_utf8_bytes_hex={hex}"), Some(hex))
        }
    };

    let mut record = LogRecord::new(
        crate::logging::system_now_unix_millis(),
        log.hostname().to_string(),
        severity,
        message,
        crate::logging::LogSource {
            producer: crate::logging::LogProducer::PgTool,
            transport,
            parser: crate::logging::LogParser::Raw,
            origin: "startup".to_string(),
        },
    );

    record.attributes.insert(
        "job_id".to_string(),
        serde_json::Value::String(identity.job_id.0.clone()),
    );
    record.attributes.insert(
        "job_kind".to_string(),
        serde_json::Value::String(identity.job_kind.clone()),
    );
    record.attributes.insert(
        "binary".to_string(),
        serde_json::Value::String(identity.binary.clone()),
    );
    record.attributes.insert(
        "stream".to_string(),
        serde_json::Value::String(match line.stream {
            crate::process::jobs::ProcessOutputStream::Stdout => "stdout".to_string(),
            crate::process::jobs::ProcessOutputStream::Stderr => "stderr".to_string(),
        }),
    );
    if let Some(hex) = raw_bytes_hex {
        record
            .attributes
            .insert("raw_bytes_hex".to_string(), serde_json::Value::String(hex));
    }

    log.emit_record(&record)
}

fn emit_hex_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len().saturating_mul(2));
    for b in bytes {
        out.push(TABLE[(b >> 4) as usize] as char);
        out.push(TABLE[(b & 0x0f) as usize] as char);
    }
    out
}

async fn run_workers(
    cfg: RuntimeConfig,
    process_defaults: ProcessDispatchDefaults,
    log: crate::logging::LogHandle,
) -> Result<(), RuntimeError> {
    let now = now_unix_millis()?;

    let (_cfg_publisher, cfg_subscriber) = new_state_channel(cfg.clone(), now);
    let (pg_publisher, pg_subscriber) = new_state_channel(initial_pg_state(), now);

    let initial_dcs = DcsState {
        worker: WorkerStatus::Starting,
        trust: DcsTrust::NotTrusted,
        cache: DcsCache {
            members: BTreeMap::new(),
            leader: None,
            switchover: None,
            config: cfg.clone(),
            init_lock: None,
        },
        last_refresh_at: None,
    };
    let (dcs_publisher, dcs_subscriber) = new_state_channel(initial_dcs, now);

    let initial_process = ProcessState::Idle {
        worker: WorkerStatus::Starting,
        last_outcome: None,
    };
    let (process_publisher, process_subscriber) = new_state_channel(initial_process.clone(), now);

    let initial_ha = HaState {
        worker: WorkerStatus::Starting,
        phase: HaPhase::Init,
        tick: 0,
        decision: crate::ha::decision::HaDecision::NoChange,
    };
    let (ha_publisher, ha_subscriber) = new_state_channel(initial_ha, now);

    let initial_debug_snapshot = build_snapshot(
        &DebugSnapshotCtx {
            app: AppLifecycle::Running,
            config: cfg_subscriber.latest(),
            pg: pg_subscriber.latest(),
            dcs: dcs_subscriber.latest(),
            process: process_subscriber.latest(),
            ha: ha_subscriber.latest(),
        },
        now,
        0,
        &[],
        &[],
    );
    let (debug_publisher, debug_subscriber) = new_state_channel(initial_debug_snapshot, now);

    let self_id = MemberId(cfg.cluster.member_id.clone());
    let scope = cfg.dcs.scope.clone();

    let pg_ctx = crate::pginfo::state::PgInfoWorkerCtx {
        self_id: self_id.clone(),
        postgres_dsn: local_postgres_dsn(
            &process_defaults,
            &cfg.postgres.local_conn_identity,
            cfg.postgres.roles.superuser.username.as_str(),
            cfg.postgres.connect_timeout_s,
        ),
        poll_interval: Duration::from_millis(cfg.ha.loop_interval_ms),
        publisher: pg_publisher,
        log: log.clone(),
        last_emitted_sql_status: None,
    };

    let dcs_store = EtcdDcsStore::connect(cfg.dcs.endpoints.clone(), &scope)
        .map_err(|err| RuntimeError::Worker(format!("dcs store connect failed: {err}")))?;
    let dcs_ctx = DcsWorkerCtx {
        self_id: self_id.clone(),
        scope: scope.clone(),
        poll_interval: Duration::from_millis(cfg.ha.loop_interval_ms),
        pg_subscriber: pg_subscriber.clone(),
        publisher: dcs_publisher,
        store: Box::new(dcs_store),
        log: log.clone(),
        cache: DcsCache {
            members: BTreeMap::new(),
            leader: None,
            switchover: None,
            config: cfg.clone(),
            init_lock: None,
        },
        last_published_pg_version: None,
        last_emitted_store_healthy: None,
        last_emitted_trust: None,
    };

    let (process_inbox_tx, process_inbox_rx) = mpsc::unbounded_channel();
    let process_ctx = ProcessWorkerCtx {
        poll_interval: Duration::from_millis(10),
        config: cfg.process.clone(),
        log: log.clone(),
        capture_subprocess_output: cfg.logging.capture_subprocess_output,
        state: initial_process,
        publisher: process_publisher,
        inbox: process_inbox_rx,
        inbox_disconnected_logged: false,
        command_runner: Box::new(TokioCommandRunner),
        active_runtime: None,
        last_rejection: None,
        now: Box::new(system_now_unix_millis),
    };

    let ha_store = EtcdDcsStore::connect(cfg.dcs.endpoints.clone(), &scope)
        .map_err(|err| RuntimeError::Worker(format!("ha store connect failed: {err}")))?;
    let mut ha_ctx = HaWorkerCtx::contract_stub(HaWorkerContractStubInputs {
        publisher: ha_publisher,
        config_subscriber: cfg_subscriber.clone(),
        pg_subscriber: pg_subscriber.clone(),
        dcs_subscriber: dcs_subscriber.clone(),
        process_subscriber: process_subscriber.clone(),
        process_inbox: process_inbox_tx,
        dcs_store: Box::new(ha_store),
        scope: scope.clone(),
        self_id: self_id.clone(),
    });
    ha_ctx.poll_interval = Duration::from_millis(cfg.ha.loop_interval_ms);
    ha_ctx.now = Box::new(system_now_unix_millis);
    ha_ctx.process_defaults = process_defaults;
    ha_ctx.log = log.clone();

    let mut debug_ctx = DebugApiCtx::contract_stub(DebugApiContractStubInputs {
        publisher: debug_publisher,
        config_subscriber: cfg_subscriber.clone(),
        pg_subscriber: pg_subscriber.clone(),
        dcs_subscriber: dcs_subscriber.clone(),
        process_subscriber: process_subscriber.clone(),
        ha_subscriber: ha_subscriber.clone(),
    });
    debug_ctx.app = AppLifecycle::Running;
    debug_ctx.poll_interval = Duration::from_millis(cfg.ha.loop_interval_ms);
    debug_ctx.now = Box::new(system_now_unix_millis);

    let api_store = EtcdDcsStore::connect(cfg.dcs.endpoints.clone(), &scope)
        .map_err(|err| RuntimeError::Worker(format!("api store connect failed: {err}")))?;
    let listener = TcpListener::bind(cfg.api.listen_addr.as_str())
        .await
        .map_err(|err| RuntimeError::ApiBind {
            listen_addr: cfg.api.listen_addr.clone(),
            message: err.to_string(),
        })?;
    let mut api_ctx = ApiWorkerCtx::new(listener, cfg_subscriber, Box::new(api_store), log.clone());
    api_ctx.set_ha_snapshot_subscriber(debug_subscriber);
    let server_tls = crate::tls::build_rustls_server_config(&cfg.api.security.tls)
        .map_err(|err| RuntimeError::Worker(format!("api tls config build failed: {err}")))?;
    api_ctx
        .configure_tls(cfg.api.security.tls.mode, server_tls)
        .map_err(|err| RuntimeError::Worker(format!("api tls configure failed: {err}")))?;
    let require_client_cert = match cfg.api.security.tls.client_auth.as_ref() {
        Some(auth) => auth.require_client_cert,
        None => false,
    };
    api_ctx.set_require_client_cert(require_client_cert);

    tokio::try_join!(
        crate::pginfo::worker::run(pg_ctx),
        crate::dcs::worker::run(dcs_ctx),
        crate::process::worker::run(process_ctx),
        crate::logging::postgres_ingest::run(crate::logging::postgres_ingest::build_ctx(
            cfg.clone(),
            log.clone(),
        )),
        crate::ha::worker::run(ha_ctx),
        crate::debug_api::worker::run(debug_ctx),
        crate::api::worker::run(api_ctx),
    )
    .map_err(|err| RuntimeError::Worker(err.to_string()))?;

    Ok(())
}

fn local_postgres_dsn(
    process_defaults: &ProcessDispatchDefaults,
    identity: &crate::config::PostgresConnIdentityConfig,
    superuser_username: &str,
    connect_timeout_s: u32,
) -> String {
    format!(
        "host={} port={} user={} dbname={} connect_timeout={} sslmode={}",
        process_defaults.postgres_host,
        process_defaults.postgres_port,
        superuser_username,
        identity.dbname,
        connect_timeout_s,
        identity.ssl_mode.as_str()
    )
}

fn initial_pg_state() -> PgInfoState {
    PgInfoState::Unknown {
        common: PgInfoCommon {
            worker: WorkerStatus::Starting,
            sql: SqlStatus::Unknown,
            readiness: Readiness::Unknown,
            timeline: None,
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

fn now_unix_millis() -> Result<UnixMillis, RuntimeError> {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| RuntimeError::Time(format!("system time before epoch: {err}")))?;
    let millis = u64::try_from(elapsed.as_millis())
        .map_err(|err| RuntimeError::Time(format!("millis conversion failed: {err}")))?;
    Ok(UnixMillis(millis))
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        fs, io,
        path::PathBuf,
        sync::Arc,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::pginfo::conninfo::PgSslMode;
    use crate::{
        config::{
            ApiAuthConfig, ApiConfig, ApiSecurityConfig, ApiTlsMode, BackupConfig, BinaryPaths,
            ClusterConfig, DcsConfig, DebugConfig, HaConfig, InlineOrPath, LogCleanupConfig,
            LogLevel, LoggingConfig, PgHbaConfig, PgIdentConfig, PostgresConfig,
            PostgresConnIdentityConfig, PostgresLoggingConfig, PostgresRoleConfig,
            PostgresRolesConfig, ProcessConfig, RoleAuthConfig, RuntimeConfig, StderrSinkConfig,
            TlsServerConfig,
        },
        dcs::state::{DcsCache, LeaderRecord, MemberRecord, MemberRole},
        logging::{LogHandle, LogSink, SeverityText, TestSink},
        pginfo::state::{Readiness, SqlStatus},
        state::{MemberId, UnixMillis, Version},
    };

    use super::{build_startup_actions, StartupAction};
    use super::{
        default_leader_source, inspect_data_dir, select_startup_mode, DataDirState, StartupMode,
    };
    use super::{plan_startup_with_probe, process_defaults_from_config};

    fn sample_runtime_config() -> RuntimeConfig {
        RuntimeConfig {
            cluster: ClusterConfig {
                name: "cluster-a".to_string(),
                member_id: "node-a".to_string(),
            },
            postgres: PostgresConfig {
                data_dir: PathBuf::from("/tmp/pgtuskmaster-test-data"),
                connect_timeout_s: 5,
                listen_host: "127.0.0.1".to_string(),
                listen_port: 5432,
                socket_dir: PathBuf::from("/tmp/pgtuskmaster/socket"),
                log_file: PathBuf::from("/tmp/pgtuskmaster/postgres.log"),
                rewind_source_host: "127.0.0.1".to_string(),
                rewind_source_port: 5432,
                local_conn_identity: PostgresConnIdentityConfig {
                    user: "postgres".to_string(),
                    dbname: "postgres".to_string(),
                    ssl_mode: PgSslMode::Prefer,
                },
                rewind_conn_identity: PostgresConnIdentityConfig {
                    user: "rewinder".to_string(),
                    dbname: "postgres".to_string(),
                    ssl_mode: PgSslMode::Prefer,
                },
                tls: TlsServerConfig {
                    mode: ApiTlsMode::Disabled,
                    identity: None,
                    client_auth: None,
                },
                roles: PostgresRolesConfig {
                    superuser: PostgresRoleConfig {
                        username: "postgres".to_string(),
                        auth: RoleAuthConfig::Tls,
                    },
                    replicator: PostgresRoleConfig {
                        username: "replicator".to_string(),
                        auth: RoleAuthConfig::Tls,
                    },
                    rewinder: PostgresRoleConfig {
                        username: "rewinder".to_string(),
                        auth: RoleAuthConfig::Tls,
                    },
                },
                pg_hba: PgHbaConfig {
                    source: InlineOrPath::Inline {
                        content: "local all all trust\n".to_string(),
                    },
                },
                pg_ident: PgIdentConfig {
                    source: InlineOrPath::Inline {
                        content: "# empty\n".to_string(),
                    },
                },
            },
            dcs: DcsConfig {
                endpoints: vec!["http://127.0.0.1:2379".to_string()],
                scope: "scope-a".to_string(),
                init: None,
            },
            ha: HaConfig {
                loop_interval_ms: 1000,
                lease_ttl_ms: 10_000,
            },
            process: ProcessConfig {
                pg_rewind_timeout_ms: 1000,
                bootstrap_timeout_ms: 1000,
                fencing_timeout_ms: 1000,
                backup_timeout_ms: 1000,
                binaries: BinaryPaths {
                    postgres: "/usr/bin/postgres".into(),
                    pg_ctl: "/usr/bin/pg_ctl".into(),
                    pg_rewind: "/usr/bin/pg_rewind".into(),
                    initdb: "/usr/bin/initdb".into(),
                    pg_basebackup: "/usr/bin/pg_basebackup".into(),
                    psql: "/usr/bin/psql".into(),
                    pgbackrest: None,
                },
            },
            backup: BackupConfig::default(),
            logging: LoggingConfig {
                level: LogLevel::Info,
                capture_subprocess_output: true,
                postgres: PostgresLoggingConfig {
                    enabled: true,
                    pg_ctl_log_file: None,
                    log_dir: None,
                    poll_interval_ms: 200,
                    cleanup: LogCleanupConfig {
                        enabled: true,
                        max_files: 10,
                        max_age_seconds: 60,
                        protect_recent_seconds: 300,
                    },
                },
                sinks: crate::config::LoggingSinksConfig {
                    stderr: StderrSinkConfig { enabled: true },
                    file: crate::config::FileSinkConfig {
                        enabled: false,
                        path: None,
                        mode: crate::config::FileSinkMode::Append,
                    },
                },
            },
            api: ApiConfig {
                listen_addr: "127.0.0.1:8080".to_string(),
                security: ApiSecurityConfig {
                    tls: TlsServerConfig {
                        mode: ApiTlsMode::Disabled,
                        identity: None,
                        client_auth: None,
                    },
                    auth: ApiAuthConfig::Disabled,
                },
            },
            debug: DebugConfig { enabled: true },
        }
    }

    fn temp_path(label: &str) -> PathBuf {
        let millis = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_millis(),
            Err(_) => 0,
        };
        std::env::temp_dir().join(format!(
            "pgtuskmaster-runtime-{label}-{millis}-{}",
            std::process::id()
        ))
    }

    fn remove_if_exists(path: &PathBuf) -> Result<(), io::Error> {
        if path.exists() {
            fs::remove_dir_all(path)?;
        }
        Ok(())
    }

    fn test_log_handle() -> (LogHandle, Arc<TestSink>) {
        let sink = Arc::new(TestSink::default());
        let sink_dyn: Arc<dyn LogSink> = sink.clone();
        (
            LogHandle::new("host-a".to_string(), sink_dyn, SeverityText::Trace),
            sink,
        )
    }

    #[test]
    fn inspect_data_dir_classifies_missing_empty_and_existing(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let missing = temp_path("missing");
        remove_if_exists(&missing)?;
        assert_eq!(inspect_data_dir(&missing)?, DataDirState::Missing);

        let empty = temp_path("empty");
        remove_if_exists(&empty)?;
        fs::create_dir_all(&empty)?;
        assert_eq!(inspect_data_dir(&empty)?, DataDirState::Empty);

        let existing = temp_path("existing");
        remove_if_exists(&existing)?;
        fs::create_dir_all(&existing)?;
        fs::write(existing.join("PG_VERSION"), b"16\n")?;
        assert_eq!(inspect_data_dir(&existing)?, DataDirState::Existing);

        remove_if_exists(&empty)?;
        remove_if_exists(&existing)?;
        Ok(())
    }

    #[test]
    fn plan_startup_emits_data_dir_and_mode_events_without_network_probe(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut cfg = sample_runtime_config();
        let dir = temp_path("plan-startup-log");
        remove_if_exists(&dir)?;
        cfg.postgres.data_dir = dir.clone();

        let process_defaults = process_defaults_from_config(&cfg);
        let (log, sink) = test_log_handle();

        let _startup_mode =
            plan_startup_with_probe(&cfg, &process_defaults, &log, "run-1", |_cfg| {
                Ok(DcsCache {
                    members: BTreeMap::new(),
                    leader: None,
                    switchover: None,
                    config: cfg.clone(),
                    init_lock: None,
                })
            })?;

        let inspected = sink.collect_matching(|record| {
            matches!(
                record.attributes.get("event.name"),
                Some(serde_json::Value::String(name))
                    if name == "runtime.startup.data_dir.inspected"
            )
        })?;
        if inspected.is_empty() {
            return Err(Box::new(io::Error::other(
                "expected runtime.startup.data_dir.inspected event",
            )));
        }

        let probe = sink.collect_matching(|record| {
            matches!(
                record.attributes.get("event.name"),
                Some(serde_json::Value::String(name))
                    if name == "runtime.startup.dcs_cache_probe"
            )
        })?;
        if probe.is_empty() {
            return Err(Box::new(io::Error::other(
                "expected runtime.startup.dcs_cache_probe event",
            )));
        }

        let mode_selected = sink.collect_matching(|record| {
            matches!(
                record.attributes.get("event.name"),
                Some(serde_json::Value::String(name))
                    if name == "runtime.startup.mode_selected"
            )
        })?;
        if mode_selected.is_empty() {
            return Err(Box::new(io::Error::other(
                "expected runtime.startup.mode_selected event",
            )));
        }

        remove_if_exists(&dir)?;
        Ok(())
    }

    #[test]
    fn inspect_data_dir_rejects_ambiguous_partial_state() -> Result<(), Box<dyn std::error::Error>>
    {
        let ambiguous = temp_path("ambiguous");
        remove_if_exists(&ambiguous)?;
        fs::create_dir_all(&ambiguous)?;
        fs::write(ambiguous.join("postgresql.conf"), b"# test\n")?;

        let result = inspect_data_dir(&ambiguous);
        assert!(result.is_err());

        remove_if_exists(&ambiguous)?;
        Ok(())
    }

    #[test]
    fn select_startup_mode_prefers_clone_when_foreign_healthy_leader_exists(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cfg = sample_runtime_config();
        let defaults = crate::ha::state::ProcessDispatchDefaults::contract_stub();

        let leader_id = MemberId("node-b".to_string());
        let mut members = BTreeMap::new();
        members.insert(
            leader_id.clone(),
            MemberRecord {
                member_id: leader_id.clone(),
                role: MemberRole::Primary,
                sql: SqlStatus::Healthy,
                readiness: Readiness::Ready,
                timeline: None,
                write_lsn: None,
                replay_lsn: None,
                updated_at: UnixMillis(1),
                pg_version: Version(1),
            },
        );

        let cache = DcsCache {
            members,
            leader: Some(LeaderRecord {
                member_id: leader_id.clone(),
            }),
            switchover: None,
            config: cfg.clone(),
            init_lock: None,
        };

        let mode = select_startup_mode(
            DataDirState::Empty,
            Some(&cache),
            "node-a",
            &defaults,
            cfg.postgres.connect_timeout_s,
            cfg.backup.bootstrap.enabled,
        )?;

        assert!(matches!(mode, StartupMode::CloneReplica { .. }));
        if let StartupMode::CloneReplica {
            leader_member_id,
            source,
        } = mode
        {
            assert_eq!(leader_member_id, leader_id);
            assert_eq!(
                source,
                default_leader_source(&defaults, cfg.postgres.connect_timeout_s)
            );
        }
        Ok(())
    }

    #[test]
    fn select_startup_mode_uses_initialize_when_no_leader_evidence(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cfg = sample_runtime_config();
        let defaults = crate::ha::state::ProcessDispatchDefaults::contract_stub();

        let mode = select_startup_mode(
            DataDirState::Empty,
            None,
            "node-a",
            &defaults,
            cfg.postgres.connect_timeout_s,
            cfg.backup.bootstrap.enabled,
        )?;

        assert_eq!(mode, StartupMode::InitializePrimary);
        Ok(())
    }

    #[test]
    fn select_startup_mode_uses_restore_bootstrap_when_enabled_and_uninitialized(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut cfg = sample_runtime_config();
        cfg.backup.enabled = true;
        cfg.backup.bootstrap.enabled = true;
        let defaults = crate::ha::state::ProcessDispatchDefaults::contract_stub();

        let mode = select_startup_mode(
            DataDirState::Empty,
            None,
            "node-a",
            &defaults,
            cfg.postgres.connect_timeout_s,
            cfg.backup.bootstrap.enabled,
        )?;

        assert_eq!(mode, StartupMode::RestoreBootstrap);
        Ok(())
    }

    #[test]
    fn restore_bootstrap_action_order_is_restore_takeover_start(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut cfg = sample_runtime_config();
        cfg.backup.enabled = true;
        cfg.backup.bootstrap.enabled = true;
        if let Some(pg_cfg) = cfg.backup.pgbackrest.as_mut() {
            pg_cfg.stanza = Some("stanza-a".to_string());
            pg_cfg.repo = Some("1".to_string());
        }

        let actions = build_startup_actions(&cfg, &StartupMode::RestoreBootstrap)?;
        if actions.len() != 4 {
            return Err(Box::new(std::io::Error::other(format!(
                "expected 4 actions, got {}",
                actions.len()
            ))));
        }
        if !matches!(
            actions.first(),
            Some(StartupAction::ClaimInitLockAndSeedConfig)
        ) {
            return Err(Box::new(std::io::Error::other(
                "expected claim init lock action first",
            )));
        }
        let second_is_restore = match actions.get(1) {
            Some(StartupAction::RunJob(job)) => {
                matches!(
                    job.as_ref(),
                    crate::process::state::ProcessJobKind::PgBackRestRestore(_)
                )
            }
            _ => false,
        };
        if !second_is_restore {
            return Err(Box::new(std::io::Error::other(
                "expected pgbackrest restore job second",
            )));
        }
        if !matches!(actions.get(2), Some(StartupAction::TakeoverRestoredDataDir)) {
            return Err(Box::new(std::io::Error::other(
                "expected takeover action third",
            )));
        }
        if !matches!(actions.get(3), Some(StartupAction::StartPostgres)) {
            return Err(Box::new(std::io::Error::other(
                "expected start postgres action last",
            )));
        }
        Ok(())
    }

    #[test]
    fn select_startup_mode_uses_resume_when_pgdata_exists() -> Result<(), Box<dyn std::error::Error>>
    {
        let cfg = sample_runtime_config();
        let defaults = crate::ha::state::ProcessDispatchDefaults::contract_stub();
        let mode = select_startup_mode(
            DataDirState::Existing,
            None,
            "node-a",
            &defaults,
            cfg.postgres.connect_timeout_s,
            cfg.backup.bootstrap.enabled,
        )?;
        assert_eq!(mode, StartupMode::ResumeExisting);
        Ok(())
    }

    #[test]
    fn select_startup_mode_rejects_initialize_when_init_lock_present(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cfg = sample_runtime_config();
        let defaults = crate::ha::state::ProcessDispatchDefaults::contract_stub();

        let cache = DcsCache {
            members: BTreeMap::new(),
            leader: None,
            switchover: None,
            config: cfg.clone(),
            init_lock: Some(crate::dcs::state::InitLockRecord {
                holder: MemberId("node-other".to_string()),
            }),
        };

        let result = select_startup_mode(
            DataDirState::Empty,
            Some(&cache),
            "node-a",
            &defaults,
            cfg.postgres.connect_timeout_s,
            cfg.backup.bootstrap.enabled,
        );

        assert!(matches!(
            result,
            Err(super::RuntimeError::StartupPlanning(_))
        ));
        Ok(())
    }

    #[test]
    fn select_startup_mode_uses_member_records_when_init_lock_present_and_leader_missing(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cfg = sample_runtime_config();
        let defaults = crate::ha::state::ProcessDispatchDefaults::contract_stub();

        let primary_id = MemberId("node-b".to_string());
        let mut members = BTreeMap::new();
        members.insert(
            primary_id.clone(),
            MemberRecord {
                member_id: primary_id.clone(),
                role: MemberRole::Primary,
                sql: SqlStatus::Healthy,
                readiness: Readiness::Ready,
                timeline: None,
                write_lsn: None,
                replay_lsn: None,
                updated_at: UnixMillis(1),
                pg_version: Version(1),
            },
        );

        let cache = DcsCache {
            members,
            leader: None,
            switchover: None,
            config: cfg.clone(),
            init_lock: Some(crate::dcs::state::InitLockRecord {
                holder: MemberId("node-init".to_string()),
            }),
        };

        let mode = select_startup_mode(
            DataDirState::Empty,
            Some(&cache),
            "node-a",
            &defaults,
            cfg.postgres.connect_timeout_s,
            cfg.backup.bootstrap.enabled,
        )?;

        assert!(matches!(mode, StartupMode::CloneReplica { .. }));
        Ok(())
    }

    #[test]
    fn runtime_uses_role_specific_users_for_dsn_clone_and_rewind() {
        let mut cfg = sample_runtime_config();
        cfg.postgres.roles.superuser.username = "su_admin".to_string();
        cfg.postgres.roles.replicator.username = "repl_user".to_string();
        cfg.postgres.roles.rewinder.username = "rewind_user".to_string();
        cfg.postgres.local_conn_identity.user = "su_admin".to_string();
        cfg.postgres.rewind_conn_identity.user = "rewind_user".to_string();

        let defaults = super::process_defaults_from_config(&cfg);
        assert_eq!(defaults.basebackup_source.conninfo.user, "repl_user");
        assert_eq!(defaults.rewind_source.conninfo.user, "rewind_user");

        let local_dsn = super::local_postgres_dsn(
            &defaults,
            &cfg.postgres.local_conn_identity,
            cfg.postgres.roles.superuser.username.as_str(),
            cfg.postgres.connect_timeout_s,
        );
        assert!(local_dsn.contains("user=su_admin"));

        let leader_source = default_leader_source(&defaults, cfg.postgres.connect_timeout_s);
        assert_eq!(leader_source.conninfo.user, "repl_user");
    }
}
