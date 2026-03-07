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
    ha::source_conn::basebackup_source_from_member,
    ha::state::{
        HaPhase, HaState, HaWorkerContractStubInputs, HaWorkerCtx, ProcessDispatchDefaults,
    },
    logging::{
        EventMeta, LogParser, LogProducer, LogSource, LogTransport, SeverityText,
        SubprocessLineRecord, SubprocessStream,
    },
    pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
    postgres_managed_conf::ManagedPostgresStartIntent,
    process::{
        jobs::{
            BaseBackupSpec, BootstrapSpec, ProcessCommandRunner, ProcessExit, ReplicatorSourceConn,
            StartPostgresSpec,
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
    StartPostgres(ManagedPostgresStartIntent),
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
    InitializePrimary {
        start_intent: ManagedPostgresStartIntent,
    },
    CloneReplica {
        leader_member_id: MemberId,
        source: ReplicatorSourceConn,
        start_intent: ManagedPostgresStartIntent,
    },
    ResumeExisting {
        start_intent: ManagedPostgresStartIntent,
    },
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
    ProcessDispatchDefaults {
        postgres_host: cfg.postgres.listen_host.clone(),
        postgres_port: cfg.postgres.listen_port,
        socket_dir: cfg.postgres.socket_dir.clone(),
        log_file: cfg.postgres.log_file.clone(),
        replicator_username: cfg.postgres.roles.replicator.username.clone(),
        replicator_auth: cfg.postgres.roles.replicator.auth.clone(),
        rewinder_username: cfg.postgres.roles.rewinder.username.clone(),
        rewinder_auth: cfg.postgres.roles.rewinder.auth.clone(),
        remote_dbname: cfg.postgres.rewind_conn_identity.dbname.clone(),
        remote_ssl_mode: cfg.postgres.rewind_conn_identity.ssl_mode,
        connect_timeout_s: cfg.postgres.connect_timeout_s,
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
        cfg.postgres.data_dir.as_path(),
        cache.as_ref(),
        &cfg.cluster.member_id,
        process_defaults,
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
    data_dir: &Path,
    cache: Option<&DcsCache>,
    self_member_id: &str,
    process_defaults: &ProcessDispatchDefaults,
) -> Result<StartupMode, RuntimeError> {
    match data_dir_state {
        DataDirState::Existing => Ok(StartupMode::ResumeExisting {
            start_intent: select_resume_start_intent(
                data_dir,
                cache,
                self_member_id,
                process_defaults,
            )?,
        }),
        DataDirState::Missing | DataDirState::Empty => {
            let init_lock_present = cache
                .and_then(|snapshot| snapshot.init_lock.as_ref())
                .is_some();
            let self_member_id = MemberId(self_member_id.to_string());

            let leader = leader_from_leader_key(cache, &self_member_id).or_else(|| {
                if init_lock_present {
                    foreign_healthy_primary_member(cache, &self_member_id)
                } else {
                    None
                }
            });

            match leader {
                Some(leader_member) => {
                    let source = basebackup_source_from_member(
                        &self_member_id,
                        &leader_member,
                        process_defaults,
                    )
                    .map_err(|err| RuntimeError::StartupPlanning(err.to_string()))?;
                    Ok(StartupMode::CloneReplica {
                        leader_member_id: leader_member.member_id.clone(),
                        start_intent: replica_start_intent_from_source(&source),
                        source,
                    })
                }
                None => {
                    if init_lock_present {
                        Err(RuntimeError::StartupPlanning(
                            "cluster is already initialized (dcs init lock present) but no healthy primary is available for basebackup"
                                .to_string(),
                        ))
                    } else {
                        Ok(StartupMode::InitializePrimary {
                            start_intent: ManagedPostgresStartIntent::primary(),
                        })
                    }
                }
            }
        }
    }
}

fn select_resume_start_intent(
    data_dir: &Path,
    cache: Option<&DcsCache>,
    self_member_id: &str,
    process_defaults: &ProcessDispatchDefaults,
) -> Result<ManagedPostgresStartIntent, RuntimeError> {
    let self_member_id = MemberId(self_member_id.to_string());
    let existing_managed_replica =
        crate::postgres_managed::read_existing_replica_start_intent(data_dir)
            .map_err(|err| RuntimeError::StartupPlanning(err.to_string()))?;

    let Some(cache) = cache else {
        if existing_managed_replica.is_some() {
            return Err(RuntimeError::StartupPlanning(
                "existing postgres data dir contains managed replica recovery state but startup dcs cache probe was unavailable; cannot rebuild authoritative startup intent"
                    .to_string(),
            ));
        }
        return Ok(ManagedPostgresStartIntent::primary());
    };

    if cache
        .leader
        .as_ref()
        .map(|record| record.member_id == self_member_id)
        .unwrap_or(false)
    {
        return Ok(ManagedPostgresStartIntent::primary());
    }

    if let Some(leader_member) = leader_from_leader_key(Some(cache), &self_member_id)
        .or_else(|| foreign_healthy_primary_member(Some(cache), &self_member_id))
    {
        let source =
            basebackup_source_from_member(&self_member_id, &leader_member, process_defaults)
                .map_err(|err| RuntimeError::StartupPlanning(err.to_string()))?;
        return Ok(replica_start_intent_from_source(&source));
    }

    if local_primary_member(cache, &self_member_id).is_some() {
        return Ok(ManagedPostgresStartIntent::primary());
    }

    if existing_managed_replica.is_some() {
        return Err(RuntimeError::StartupPlanning(
            "existing postgres data dir contains managed replica recovery state but no healthy primary is available in DCS to rebuild authoritative managed config"
                .to_string(),
        ));
    }

    Ok(ManagedPostgresStartIntent::primary())
}

fn leader_from_leader_key(
    cache: Option<&DcsCache>,
    self_member_id: &MemberId,
) -> Option<crate::dcs::state::MemberRecord> {
    let snapshot = cache?;
    let leader_record = snapshot.leader.as_ref()?;
    if leader_record.member_id == *self_member_id {
        return None;
    }
    let member = snapshot.members.get(&leader_record.member_id)?;
    let eligible = member.role == MemberRole::Primary && member.sql == SqlStatus::Healthy;
    if eligible {
        Some(member.clone())
    } else {
        None
    }
}

fn foreign_healthy_primary_member(
    cache: Option<&DcsCache>,
    self_member_id: &MemberId,
) -> Option<crate::dcs::state::MemberRecord> {
    cache?
        .members
        .values()
        .find(|member| {
            member.member_id != *self_member_id
                && member.role == MemberRole::Primary
                && member.sql == SqlStatus::Healthy
        })
        .cloned()
}

fn local_primary_member<'a>(
    cache: &'a DcsCache,
    self_member_id: &MemberId,
) -> Option<&'a crate::dcs::state::MemberRecord> {
    cache
        .members
        .get(self_member_id)
        .filter(|member| member.role == MemberRole::Primary && member.sql == SqlStatus::Healthy)
}

fn replica_start_intent_from_source(source: &ReplicatorSourceConn) -> ManagedPostgresStartIntent {
    ManagedPostgresStartIntent::replica(source.conninfo.clone(), None)
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
            StartupAction::StartPostgres(_) => "start_postgres",
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

        if let StartupAction::StartPostgres(_) = &action {
            emit_startup_phase(log, "start", "start postgres with managed config").map_err(
                |err| {
                    RuntimeError::StartupExecution(format!("startup phase log emit failed: {err}"))
                },
            )?;
        }

        let result = match action {
            StartupAction::ClaimInitLockAndSeedConfig => {
                claim_dcs_init_lock_and_seed_config(cfg).map_err(|err| {
                    RuntimeError::StartupExecution(format!("dcs init lock claim failed: {err}"))
                })?;
                Ok(())
            }
            StartupAction::RunJob(job) => run_startup_job(cfg, *job, log).await,
            StartupAction::StartPostgres(start_intent) => {
                run_start_job(cfg, process_defaults, &start_intent, log).await
            }
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
        StartupMode::InitializePrimary { start_intent } => Ok(vec![
            StartupAction::ClaimInitLockAndSeedConfig,
            StartupAction::RunJob(Box::new(ProcessJobKind::Bootstrap(BootstrapSpec {
                data_dir: cfg.postgres.data_dir.clone(),
                superuser_username: cfg.postgres.roles.superuser.username.clone(),
                timeout_ms: Some(cfg.process.bootstrap_timeout_ms),
            }))),
            StartupAction::StartPostgres(start_intent.clone()),
        ]),
        StartupMode::CloneReplica {
            source,
            start_intent,
            ..
        } => Ok(vec![
            StartupAction::RunJob(Box::new(ProcessJobKind::BaseBackup(BaseBackupSpec {
                data_dir: cfg.postgres.data_dir.clone(),
                source: source.clone(),
                timeout_ms: Some(cfg.process.bootstrap_timeout_ms),
            }))),
            StartupAction::StartPostgres(start_intent.clone()),
        ]),
        StartupMode::ResumeExisting { start_intent } => {
            if has_postmaster_pid(&cfg.postgres.data_dir) {
                Ok(Vec::new())
            } else {
                Ok(vec![StartupAction::StartPostgres(start_intent.clone())])
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
    start_intent: &ManagedPostgresStartIntent,
    log: &crate::logging::LogHandle,
) -> Result<(), RuntimeError> {
    let managed = crate::postgres_managed::materialize_managed_postgres_config(cfg, start_intent)
        .map_err(|err| {
        RuntimeError::StartupExecution(format!("materialize managed postgres config failed: {err}"))
    })?;
    run_startup_job(
        cfg,
        ProcessJobKind::StartPostgres(StartPostgresSpec {
            data_dir: cfg.postgres.data_dir.clone(),
            config_file: managed.postgresql_conf_path,
            log_file: process_defaults.log_file.clone(),
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
    let stream = match line.stream {
        crate::process::jobs::ProcessOutputStream::Stdout => SubprocessStream::Stdout,
        crate::process::jobs::ProcessOutputStream::Stderr => SubprocessStream::Stderr,
    };

    log.emit_raw_record(
        SubprocessLineRecord::new(
            crate::logging::LogProducer::PgTool,
            "startup",
            identity.job_id.0.clone(),
            identity.job_kind.clone(),
            identity.binary.clone(),
            stream,
            line.bytes,
        )
        .into_raw_record()?,
    )
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
        local_postgres_host: cfg.postgres.listen_host.clone(),
        local_postgres_port: cfg.postgres.listen_port,
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
        config::{PostgresConfig, RuntimeConfig},
        dcs::state::{DcsCache, LeaderRecord, MemberRecord, MemberRole},
        logging::{LogHandle, LogSink, SeverityText, TestSink},
        pginfo::state::{Readiness, SqlStatus},
        state::{MemberId, UnixMillis, Version},
    };

    use super::{
        inspect_data_dir, plan_startup_with_probe, process_defaults_from_config,
        select_resume_start_intent, select_startup_mode, DataDirState, StartupMode,
    };
    use crate::postgres_managed_conf::ManagedPostgresStartIntent;

    fn sample_runtime_config() -> RuntimeConfig {
        crate::test_harness::runtime_config::RuntimeConfigBuilder::new()
            .with_postgres_data_dir(PathBuf::from("/tmp/pgtuskmaster-test-data"))
            .build()
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
                postgres_host: "10.0.0.20".to_string(),
                postgres_port: 5440,
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

        let data_dir = temp_path("startup-mode-clone");
        remove_if_exists(&data_dir)?;
        let mode = select_startup_mode(
            DataDirState::Empty,
            &data_dir,
            Some(&cache),
            "node-a",
            &defaults,
        )?;

        assert!(matches!(mode, StartupMode::CloneReplica { .. }));
        if let StartupMode::CloneReplica {
            leader_member_id,
            source,
            ..
        } = mode
        {
            assert_eq!(leader_member_id, leader_id);
            assert_eq!(
                source,
                crate::ha::source_conn::basebackup_source_from_member(
                    &MemberId("node-a".to_string()),
                    cache.members.get(&leader_id).ok_or_else(|| {
                        io::Error::other("leader member missing from startup test cache")
                    })?,
                    &defaults,
                )?
            );
        }
        Ok(())
    }

    #[test]
    fn select_startup_mode_uses_initialize_when_no_leader_evidence(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let defaults = crate::ha::state::ProcessDispatchDefaults::contract_stub();
        let data_dir = temp_path("startup-mode-init");
        remove_if_exists(&data_dir)?;

        let mode = select_startup_mode(DataDirState::Empty, &data_dir, None, "node-a", &defaults)?;

        assert_eq!(
            mode,
            StartupMode::InitializePrimary {
                start_intent: ManagedPostgresStartIntent::primary(),
            }
        );
        Ok(())
    }

    #[test]
    fn select_startup_mode_uses_resume_when_pgdata_exists() -> Result<(), Box<dyn std::error::Error>>
    {
        let defaults = crate::ha::state::ProcessDispatchDefaults::contract_stub();
        let data_dir = temp_path("startup-mode-resume");
        remove_if_exists(&data_dir)?;
        let mode =
            select_startup_mode(DataDirState::Existing, &data_dir, None, "node-a", &defaults)?;
        assert_eq!(
            mode,
            StartupMode::ResumeExisting {
                start_intent: ManagedPostgresStartIntent::primary(),
            }
        );
        Ok(())
    }

    #[test]
    fn select_resume_start_intent_prefers_dcs_leader_over_local_auto_conf(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cfg = sample_runtime_config();
        let defaults = process_defaults_from_config(&cfg);
        let data_dir = temp_path("resume-dcs-authority");
        remove_if_exists(&data_dir)?;
        fs::create_dir_all(&data_dir)?;

        let runtime_config = RuntimeConfig {
            postgres: PostgresConfig {
                data_dir: data_dir.clone(),
                ..cfg.postgres.clone()
            },
            ..cfg.clone()
        };
        crate::postgres_managed::materialize_managed_postgres_config(
            &runtime_config,
            &ManagedPostgresStartIntent::replica(
                crate::pginfo::state::PgConnInfo {
                    host: "10.0.0.30".to_string(),
                    port: 5439,
                    user: "replicator".to_string(),
                    dbname: "postgres".to_string(),
                    application_name: None,
                    connect_timeout_s: Some(2),
                    ssl_mode: PgSslMode::Prefer,
                    options: None,
                },
                Some("slot_local".to_string()),
            ),
        )?;
        fs::write(
            data_dir.join("postgresql.auto.conf"),
            "primary_conninfo = 'host=192.0.2.99 port=6543 user=bad dbname=postgres'\n",
        )?;

        let leader_id = MemberId("node-b".to_string());
        let mut members = BTreeMap::new();
        members.insert(
            leader_id.clone(),
            MemberRecord {
                member_id: leader_id.clone(),
                postgres_host: "10.0.0.20".to_string(),
                postgres_port: 5440,
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
            config: runtime_config.clone(),
            init_lock: None,
        };

        let intent = select_resume_start_intent(&data_dir, Some(&cache), "node-a", &defaults)?;
        let expected_source = crate::ha::source_conn::basebackup_source_from_member(
            &MemberId("node-a".to_string()),
            cache
                .members
                .get(&leader_id)
                .ok_or_else(|| io::Error::other("leader missing from test cache"))?,
            &defaults,
        )?;
        assert_eq!(
            intent,
            ManagedPostgresStartIntent::replica(expected_source.conninfo, None)
        );

        remove_if_exists(&data_dir)?;
        Ok(())
    }

    #[test]
    fn select_resume_start_intent_rejects_local_replica_state_without_dcs_authority(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cfg = sample_runtime_config();
        let defaults = process_defaults_from_config(&cfg);
        let data_dir = temp_path("resume-without-dcs");
        remove_if_exists(&data_dir)?;
        fs::create_dir_all(&data_dir)?;

        let runtime_config = RuntimeConfig {
            postgres: PostgresConfig {
                data_dir: data_dir.clone(),
                ..cfg.postgres.clone()
            },
            ..cfg.clone()
        };
        crate::postgres_managed::materialize_managed_postgres_config(
            &runtime_config,
            &ManagedPostgresStartIntent::replica(
                crate::pginfo::state::PgConnInfo {
                    host: "10.0.0.30".to_string(),
                    port: 5439,
                    user: "replicator".to_string(),
                    dbname: "postgres".to_string(),
                    application_name: None,
                    connect_timeout_s: Some(2),
                    ssl_mode: PgSslMode::Prefer,
                    options: None,
                },
                Some("slot_local".to_string()),
            ),
        )?;

        let result = select_resume_start_intent(&data_dir, None, "node-a", &defaults);
        assert!(matches!(
            result,
            Err(super::RuntimeError::StartupPlanning(_))
        ));

        remove_if_exists(&data_dir)?;
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

        let data_dir = temp_path("startup-mode-init-lock");
        remove_if_exists(&data_dir)?;
        let result = select_startup_mode(
            DataDirState::Empty,
            &data_dir,
            Some(&cache),
            "node-a",
            &defaults,
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
                postgres_host: "10.0.0.21".to_string(),
                postgres_port: 5441,
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

        let data_dir = temp_path("startup-mode-member-fallback");
        remove_if_exists(&data_dir)?;
        let mode = select_startup_mode(
            DataDirState::Empty,
            &data_dir,
            Some(&cache),
            "node-a",
            &defaults,
        )?;

        assert!(matches!(mode, StartupMode::CloneReplica { .. }));
        Ok(())
    }

    #[test]
    fn runtime_uses_role_specific_users_for_dsn_clone_and_rewind(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut cfg = sample_runtime_config();
        cfg.postgres.roles.superuser.username = "su_admin".to_string();
        cfg.postgres.roles.replicator.username = "repl_user".to_string();
        cfg.postgres.roles.rewinder.username = "rewind_user".to_string();
        cfg.postgres.local_conn_identity.user = "su_admin".to_string();
        cfg.postgres.rewind_conn_identity.user = "rewind_user".to_string();

        let defaults = super::process_defaults_from_config(&cfg);
        assert_eq!(defaults.replicator_username, "repl_user");
        assert_eq!(defaults.rewinder_username, "rewind_user");

        let local_dsn = super::local_postgres_dsn(
            &defaults,
            &cfg.postgres.local_conn_identity,
            cfg.postgres.roles.superuser.username.as_str(),
            cfg.postgres.connect_timeout_s,
        );
        assert!(local_dsn.contains("user=su_admin"));

        let leader_source = crate::ha::source_conn::basebackup_source_from_member(
            &MemberId("node-a".to_string()),
            &MemberRecord {
                member_id: MemberId("node-b".to_string()),
                postgres_host: "10.0.0.30".to_string(),
                postgres_port: 5442,
                role: MemberRole::Primary,
                sql: SqlStatus::Healthy,
                readiness: Readiness::Ready,
                timeline: None,
                write_lsn: None,
                replay_lsn: None,
                updated_at: UnixMillis(1),
                pg_version: Version(1),
            },
            &defaults,
        )?;
        assert_eq!(leader_source.conninfo.user, "repl_user");
        Ok(())
    }
}
