use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;

use crate::{
    config::{BinaryPaths, ProcessConfig, RuntimeConfig},
    dcs::state::{DcsCache, DcsState, DcsTrust, DcsWorkerCtx},
    dcs::store::{DcsStore, DcsStoreError, WatchEvent},
    debug_api::{snapshot::SystemSnapshot, worker::DebugApiCtx},
    ha::{
        actions::HaAction,
        state::{HaPhase, HaState, HaWorkerCtx, WorldSnapshot},
    },
    pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, PgInfoWorkerCtx, Readiness, SqlStatus},
    process::{
        state::{JobOutcome, ProcessJobKind, ProcessState, ProcessWorkerCtx},
        worker as process_worker,
    },
    state::{
        new_state_channel, ClusterName, JobId, MemberId, UnixMillis, Version, Versioned,
        WorkerStatus,
    },
};

#[derive(Default)]
struct ContractStore;

impl DcsStore for ContractStore {
    fn healthy(&self) -> bool {
        true
    }

    fn write_path(&mut self, _path: &str, _value: String) -> Result<(), DcsStoreError> {
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
        postgres: crate::config::schema::PostgresConfig {
            data_dir: "/tmp/pgdata".into(),
            connect_timeout_s: 5,
        },
        dcs: crate::config::schema::DcsConfig {
            endpoints: vec!["http://127.0.0.1:2379".to_string()],
            scope: "scope-a".to_string(),
        },
        ha: crate::config::schema::HaConfig {
            loop_interval_ms: 1000,
            lease_ttl_ms: 10_000,
        },
        process: ProcessConfig {
            pg_rewind_timeout_ms: 1000,
            bootstrap_timeout_ms: 1000,
            fencing_timeout_ms: 1000,
            binaries: BinaryPaths {
                postgres: "/usr/bin/postgres".into(),
                pg_ctl: "/usr/bin/pg_ctl".into(),
                pg_rewind: "/usr/bin/pg_rewind".into(),
                initdb: "/usr/bin/initdb".into(),
                psql: "/usr/bin/psql".into(),
            },
        },
        api: crate::config::schema::ApiConfig {
            listen_addr: "127.0.0.1:8080".to_string(),
        },
        debug: crate::config::schema::DebugConfig { enabled: true },
        security: crate::config::schema::SecurityConfig {
            tls_enabled: false,
            auth_token: None,
        },
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
        recent_action_ids: BTreeSet::new(),
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
async fn step_once_contracts_are_callable() {
    let initial_pg = sample_pg_state();
    let (publisher, pg_subscriber) = new_state_channel(initial_pg.clone(), UnixMillis(1));
    let mut pg_ctx = PgInfoWorkerCtx {
        self_id: MemberId("node-a".to_string()),
        postgres_dsn: "host=127.0.0.1 port=1 user=postgres dbname=postgres".to_string(),
        poll_interval: Duration::from_millis(10),
        publisher,
    };
    crate::pginfo::worker::step_once(&mut pg_ctx)
        .await
        .expect("pginfo step_once should be callable");

    let initial_dcs = sample_dcs_state(sample_runtime_config());
    let (dcs_publisher, _dcs_subscriber) = new_state_channel(initial_dcs, UnixMillis(1));
    let mut dcs_ctx = DcsWorkerCtx {
        self_id: MemberId("node-a".to_string()),
        scope: "scope-a".to_string(),
        poll_interval: Duration::from_millis(10),
        pg_subscriber,
        publisher: dcs_publisher,
        store: Box::new(ContractStore),
        cache: DcsCache {
            members: BTreeMap::new(),
            leader: None,
            switchover: None,
            config: sample_runtime_config(),
            init_lock: None,
        },
        last_published_pg_version: None,
    };
    crate::dcs::worker::step_once(&mut dcs_ctx)
        .await
        .expect("dcs step_once should be callable");

    let initial_process = sample_process_state();
    let (process_publisher, _process_subscriber) =
        new_state_channel(initial_process, UnixMillis(1));
    let (_process_tx, process_rx) = tokio::sync::mpsc::unbounded_channel();
    let mut process_ctx = ProcessWorkerCtx::contract_stub(
        sample_runtime_config().process.clone(),
        process_publisher,
        process_rx,
    );
    process_worker::step_once(&mut process_ctx)
        .await
        .expect("process step_once should be callable");

    let mut ha_ctx = HaWorkerCtx { _private: () };
    crate::ha::worker::step_once(&mut ha_ctx)
        .await
        .expect("ha step_once should be callable");

    let mut api_ctx = crate::api::worker::ApiWorkerCtx;
    crate::api::worker::step_once(&mut api_ctx)
        .await
        .expect("api step_once should be callable");

    let mut debug_ctx = DebugApiCtx;
    crate::debug_api::worker::step_once(&mut debug_ctx)
        .await
        .expect("debug_api step_once should be callable");
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

    let system = crate::debug_api::snapshot::build_snapshot(&debug_ctx, UnixMillis(2));
    assert_eq!(system.config.version, Version(2));
    let _unused = ClusterName("cluster-a".to_string());
    let _job_id = JobId("job-1".to_string());
}
