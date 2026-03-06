use std::collections::BTreeMap;
use std::time::Duration;

use crate::{
    config::{
        ApiAuthConfig, ApiConfig, ApiSecurityConfig, ApiTlsMode, BackupConfig, BinaryPaths,
        DcsConfig, HaConfig, InlineOrPath, LogCleanupConfig, LogLevel, LoggingConfig, PgHbaConfig,
        PgIdentConfig, PostgresConnIdentityConfig, PostgresConfig, PostgresLoggingConfig,
        PostgresRoleConfig, PostgresRolesConfig, ProcessConfig, RoleAuthConfig, RuntimeConfig,
        StderrSinkConfig, TlsServerConfig,
    },
    dcs::state::{DcsCache, DcsState, DcsTrust, DcsWorkerCtx},
    dcs::store::{DcsStore, DcsStoreError, WatchEvent},
    debug_api::{
        snapshot::{AppLifecycle, SystemSnapshot},
        worker::{DebugApiContractStubInputs, DebugApiCtx},
    },
    ha::{
        actions::HaAction,
        state::{HaPhase, HaState, HaWorkerContractStubInputs, HaWorkerCtx, WorldSnapshot},
    },
    pginfo::state::{
        PgConfig, PgInfoCommon, PgInfoState, PgInfoWorkerCtx, PgSslMode, Readiness, SqlStatus,
    },
    process::{
        state::{JobOutcome, ProcessJobKind, ProcessState, ProcessWorkerCtx},
        worker as process_worker,
    },
    state::{
        new_state_channel, ClusterName, JobId, MemberId, UnixMillis, Version, Versioned,
        WorkerError, WorkerStatus,
    },
};

#[derive(Default)]
struct ContractStore;

impl DcsStore for ContractStore {
    fn healthy(&self) -> bool {
        true
    }

    fn read_path(&mut self, _path: &str) -> Result<Option<String>, DcsStoreError> {
        Ok(None)
    }

    fn write_path(&mut self, _path: &str, _value: String) -> Result<(), DcsStoreError> {
        Ok(())
    }

    fn put_path_if_absent(&mut self, _path: &str, _value: String) -> Result<bool, DcsStoreError> {
        Ok(true)
    }

    fn delete_path(&mut self, _path: &str) -> Result<(), DcsStoreError> {
        Ok(())
    }

    fn drain_watch_events(&mut self) -> Result<Vec<WatchEvent>, DcsStoreError> {
        Ok(Vec::new())
    }
}

fn sample_runtime_config() -> RuntimeConfig {
    RuntimeConfig {
        cluster: crate::config::schema::ClusterConfig {
            name: "cluster-a".to_string(),
            member_id: "node-a".to_string(),
        },
        postgres: PostgresConfig {
            data_dir: "/tmp/pgdata".into(),
            connect_timeout_s: 5,
            listen_host: "127.0.0.1".to_string(),
            listen_port: 5432,
            socket_dir: "/tmp/pgtuskmaster/socket".into(),
            log_file: "/tmp/pgtuskmaster/postgres.log".into(),
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
        debug: crate::config::schema::DebugConfig { enabled: true },
    }
}

fn sample_pg_state() -> PgInfoState {
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

fn sample_dcs_state(cfg: RuntimeConfig) -> DcsState {
    DcsState {
        worker: WorkerStatus::Starting,
        trust: DcsTrust::NotTrusted,
        cache: DcsCache {
            members: BTreeMap::new(),
            leader: None,
            switchover: None,
            config: cfg,
            init_lock: None,
        },
        last_refresh_at: None,
    }
}

fn sample_process_state() -> ProcessState {
    ProcessState::Idle {
        worker: WorkerStatus::Starting,
        last_outcome: None,
    }
}

fn sample_ha_state() -> HaState {
    HaState {
        worker: WorkerStatus::Starting,
        phase: HaPhase::Init,
        tick: 0,
        pending: vec![HaAction::SignalFailSafe],
    }
}

#[test]
fn required_state_types_exist() {
    let _process_state: Option<ProcessState> = None;
    let _process_job_kind: Option<ProcessJobKind> = None;
    let _job_outcome: Option<JobOutcome> = None;

    let _ha_phase: Option<HaPhase> = None;
    let _ha_state: Option<HaState> = None;
    let _world_snapshot: Option<WorldSnapshot> = None;

    let _system_snapshot: Option<SystemSnapshot> = None;
}

#[test]
fn worker_contract_symbols_exist() {
    let _ = crate::pginfo::worker::run;
    let _ = crate::pginfo::worker::step_once;

    let _ = crate::dcs::worker::run;
    let _ = crate::dcs::worker::step_once;

    let _ = process_worker::run;
    let _ = process_worker::step_once;

    let _ = crate::ha::worker::run;
    let _ = crate::ha::worker::step_once;

    let _ = crate::api::worker::run;
    let _ = crate::api::worker::step_once;

    let _ = crate::debug_api::worker::run;
    let _ = crate::debug_api::worker::step_once;
}

#[tokio::test(flavor = "current_thread")]
async fn step_once_contracts_are_callable() -> Result<(), WorkerError> {
    let self_member_id = MemberId("node-a".to_string());

    let initial_pg = sample_pg_state();
    let (publisher, pg_subscriber) = new_state_channel(initial_pg.clone(), UnixMillis(1));
    let mut pg_ctx = PgInfoWorkerCtx {
        self_id: self_member_id.clone(),
        postgres_dsn: "host=127.0.0.1 port=1 user=postgres dbname=postgres".to_string(),
        poll_interval: Duration::from_millis(10),
        publisher,
        log: crate::logging::LogHandle::null(),
        last_emitted_sql_status: None,
    };
    crate::pginfo::worker::step_once(&mut pg_ctx).await?;
    let pg_latest = pg_subscriber.latest();
    assert_eq!(pg_latest.version, Version(1));
    assert!(matches!(
        &pg_latest.value,
        PgInfoState::Unknown { common }
            if common.worker == WorkerStatus::Running && common.sql == SqlStatus::Unreachable
    ));

    let initial_dcs = sample_dcs_state(sample_runtime_config());
    let (dcs_publisher, dcs_subscriber) = new_state_channel(initial_dcs, UnixMillis(1));
    let dcs_pg_subscriber = pg_subscriber.clone();
    let mut dcs_ctx = DcsWorkerCtx {
        self_id: self_member_id.clone(),
        scope: "scope-a".to_string(),
        poll_interval: Duration::from_millis(10),
        pg_subscriber: dcs_pg_subscriber,
        publisher: dcs_publisher,
        store: Box::new(ContractStore),
        log: crate::logging::LogHandle::null(),
        cache: DcsCache {
            members: BTreeMap::new(),
            leader: None,
            switchover: None,
            config: sample_runtime_config(),
            init_lock: None,
        },
        last_published_pg_version: None,
        last_emitted_store_healthy: None,
        last_emitted_trust: None,
    };
    crate::dcs::worker::step_once(&mut dcs_ctx).await?;
    let dcs_latest = dcs_subscriber.latest();
    assert_eq!(dcs_latest.version, Version(1));
    assert!(dcs_latest.value.last_refresh_at.is_some());
    assert_eq!(dcs_ctx.last_published_pg_version, Some(pg_latest.version));
    assert!(dcs_ctx.cache.members.contains_key(&self_member_id));

    let initial_process = sample_process_state();
    let (process_publisher, process_subscriber) = new_state_channel(initial_process, UnixMillis(1));
    let (_process_tx, process_rx) = tokio::sync::mpsc::unbounded_channel();
    let mut process_ctx = ProcessWorkerCtx::contract_stub(
        sample_runtime_config().process.clone(),
        process_publisher,
        process_rx,
    );
    process_worker::step_once(&mut process_ctx).await?;
    assert!(matches!(&process_ctx.state, ProcessState::Idle { .. }));
    assert!(process_ctx.state.running_job_id().is_none());
    assert!(matches!(
        &process_ctx.state,
        ProcessState::Idle {
            last_outcome: None,
            ..
        }
    ));
    let process_latest = process_subscriber.latest();
    assert_eq!(process_latest.version, Version(0));
    assert!(matches!(
        &process_latest.value,
        ProcessState::Idle {
            worker: WorkerStatus::Starting,
            last_outcome: None
        }
    ));

    let runtime_cfg = sample_runtime_config();
    let initial_ha = sample_ha_state();
    let (ha_publisher, ha_subscriber) = new_state_channel(initial_ha, UnixMillis(1));
    let (_cfg_publisher, cfg_subscriber) = new_state_channel(runtime_cfg.clone(), UnixMillis(1));
    let api_cfg_subscriber = cfg_subscriber.clone();
    let debug_cfg_subscriber = cfg_subscriber.clone();
    let (_ha_pg_publisher, ha_pg_subscriber) = new_state_channel(sample_pg_state(), UnixMillis(1));
    let debug_pg_subscriber = ha_pg_subscriber.clone();
    let (_ha_dcs_publisher, ha_dcs_subscriber) =
        new_state_channel(sample_dcs_state(runtime_cfg.clone()), UnixMillis(1));
    let debug_dcs_subscriber = ha_dcs_subscriber.clone();
    let (_ha_process_publisher, ha_process_subscriber) =
        new_state_channel(sample_process_state(), UnixMillis(1));
    let debug_process_subscriber = ha_process_subscriber.clone();
    let (ha_process_tx, _ha_process_rx) = tokio::sync::mpsc::unbounded_channel();
    let mut ha_ctx = HaWorkerCtx::contract_stub(HaWorkerContractStubInputs {
        publisher: ha_publisher,
        config_subscriber: cfg_subscriber,
        pg_subscriber: ha_pg_subscriber,
        dcs_subscriber: ha_dcs_subscriber,
        process_subscriber: ha_process_subscriber,
        process_inbox: ha_process_tx,
        dcs_store: Box::new(ContractStore),
        scope: "scope-a".to_string(),
        self_id: self_member_id.clone(),
    });
    crate::ha::worker::step_once(&mut ha_ctx).await?;
    assert_eq!(ha_ctx.state.phase, HaPhase::FailSafe);
    assert_eq!(ha_ctx.state.tick, 1);
    assert_eq!(ha_ctx.state.worker, WorkerStatus::Running);
    let ha_latest = ha_subscriber.latest();
    assert_eq!(ha_latest.version, Version(1));
    assert_eq!(ha_latest.value, ha_ctx.state);
    let debug_ha_subscriber = ha_subscriber.clone();

    let api_listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|err| WorkerError::Message(format!("api bind failed: {err}")))?;
    let mut api_ctx = crate::api::worker::ApiWorkerCtx::contract_stub(
        api_listener,
        api_cfg_subscriber,
        Box::new(ContractStore),
    );
    let api_addr_before = api_ctx.local_addr()?;
    crate::api::worker::step_once(&mut api_ctx).await?;
    let api_addr_after = api_ctx.local_addr()?;
    assert_eq!(api_addr_before, api_addr_after);

    let initial_debug_snapshot = SystemSnapshot {
        app: AppLifecycle::Starting,
        config: debug_cfg_subscriber.latest(),
        pg: debug_pg_subscriber.latest(),
        dcs: debug_dcs_subscriber.latest(),
        process: debug_process_subscriber.latest(),
        ha: debug_ha_subscriber.latest(),
        generated_at: UnixMillis(1),
        sequence: 0,
        changes: Vec::new(),
        timeline: Vec::new(),
    };
    let (debug_publisher, debug_subscriber) =
        new_state_channel(initial_debug_snapshot, UnixMillis(1));
    let mut debug_ctx = DebugApiCtx::contract_stub(DebugApiContractStubInputs {
        publisher: debug_publisher,
        config_subscriber: debug_cfg_subscriber,
        pg_subscriber: debug_pg_subscriber,
        dcs_subscriber: debug_dcs_subscriber,
        process_subscriber: debug_process_subscriber,
        ha_subscriber: debug_ha_subscriber,
    });
    crate::debug_api::worker::step_once(&mut debug_ctx).await?;
    let debug_latest = debug_subscriber.latest();
    assert_eq!(debug_latest.version, Version(1));
    assert_eq!(debug_latest.value.app, AppLifecycle::Starting);
    assert_eq!(debug_latest.value.config.version, Version(0));
    Ok(())
}

#[test]
fn snapshot_contract_type_compiles() {
    let cfg = sample_runtime_config();
    let pg = sample_pg_state();
    let dcs = sample_dcs_state(cfg.clone());
    let process = sample_process_state();
    let ha = sample_ha_state();

    let world = WorldSnapshot {
        config: Versioned::new(Version(1), UnixMillis(1), cfg.clone()),
        pg: Versioned::new(Version(1), UnixMillis(1), pg),
        dcs: Versioned::new(Version(1), UnixMillis(1), dcs),
        process: Versioned::new(Version(1), UnixMillis(1), process),
    };
    assert_eq!(world.config.version, Version(1));

    let debug_ctx = crate::debug_api::snapshot::DebugSnapshotCtx {
        app: crate::debug_api::snapshot::AppLifecycle::Running,
        config: Versioned::new(Version(2), UnixMillis(2), cfg),
        pg: Versioned::new(Version(2), UnixMillis(2), sample_pg_state()),
        dcs: Versioned::new(
            Version(2),
            UnixMillis(2),
            sample_dcs_state(sample_runtime_config()),
        ),
        process: Versioned::new(Version(2), UnixMillis(2), sample_process_state()),
        ha: Versioned::new(Version(2), UnixMillis(2), ha),
    };

    let system = crate::debug_api::snapshot::build_snapshot(&debug_ctx, UnixMillis(2), 0, &[], &[]);
    assert_eq!(system.config.version, Version(2));
    let _unused = ClusterName("cluster-a".to_string());
    let _job_id = JobId("job-1".to_string());
}
