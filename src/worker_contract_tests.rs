use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::sync::{
    mpsc::{self, RecvTimeoutError},
    Arc, Mutex,
};
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{
    api::{HaPhaseResponse, HaStateResponse},
    config::RuntimeConfig,
    dcs::state::{DcsCache, DcsState, DcsTrust, DcsWorkerCtx},
    dcs::store::{DcsStore, DcsStoreError, WatchEvent},
    debug_api::{
        snapshot::{build_snapshot, AppLifecycle, DebugSnapshotCtx, SystemSnapshot},
        worker::{DebugApiContractStubInputs, DebugApiCtx},
    },
    ha::{
        decision::HaDecision,
        state::{HaPhase, HaState, HaWorkerContractStubInputs, HaWorkerCtx, WorldSnapshot},
    },
    pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, PgInfoWorkerCtx, Readiness, SqlStatus},
    process::{
        state::{JobOutcome, ProcessJobKind, ProcessState, ProcessWorkerCtx},
        worker as process_worker,
    },
    state::{
        new_state_channel, ClusterName, JobId, MemberId, UnixMillis, Version, Versioned,
        WorkerError, WorkerStatus,
    },
};

const CONTRACT_STORE_RELEASE_TIMEOUT: Duration = Duration::from_secs(5);
const CONTRACT_WORKER_POLL_INTERVAL: Duration = Duration::from_millis(10);
const DEBUG_API_TEST_POLL_INTERVAL: Duration = Duration::from_millis(5);
const CONTRACT_BLOCKING_START_TIMEOUT: Duration = Duration::from_secs(1);
const CONTRACT_API_RESPONSIVE_DEADLINE: Duration = Duration::from_millis(500);

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

struct BlockingAcquireStore {
    acquire_started: Arc<Mutex<Option<mpsc::Sender<()>>>>,
    acquire_release: Arc<Mutex<mpsc::Receiver<()>>>,
}

impl BlockingAcquireStore {
    fn new() -> (Self, mpsc::Receiver<()>, mpsc::Sender<()>) {
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        (
            Self {
                acquire_started: Arc::new(Mutex::new(Some(started_tx))),
                acquire_release: Arc::new(Mutex::new(release_rx)),
            },
            started_rx,
            release_tx,
        )
    }
}

impl DcsStore for BlockingAcquireStore {
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
        let mut started_guard = self
            .acquire_started
            .lock()
            .map_err(|_| DcsStoreError::Io("acquire started lock poisoned".to_string()))?;
        if let Some(tx) = started_guard.take() {
            tx.send(())
                .map_err(|_| DcsStoreError::Io("acquire started signal failed".to_string()))?;
        }
        drop(started_guard);

        let release_guard = self
            .acquire_release
            .lock()
            .map_err(|_| DcsStoreError::Io("acquire release lock poisoned".to_string()))?;
        match release_guard.recv_timeout(CONTRACT_STORE_RELEASE_TIMEOUT) {
            Ok(()) => Ok(true),
            Err(RecvTimeoutError::Timeout) => Err(DcsStoreError::Io(
                "acquire release unblock timed out".to_string(),
            )),
            Err(RecvTimeoutError::Disconnected) => Err(DcsStoreError::Io(
                "acquire release unblock disconnected".to_string(),
            )),
        }
    }

    fn delete_path(&mut self, _path: &str) -> Result<(), DcsStoreError> {
        Ok(())
    }

    fn drain_watch_events(&mut self) -> Result<Vec<WatchEvent>, DcsStoreError> {
        Ok(Vec::new())
    }
}

fn sample_runtime_config() -> RuntimeConfig {
    crate::test_harness::runtime_config::sample_runtime_config()
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

fn sample_primary_pg_state() -> PgInfoState {
    PgInfoState::Primary {
        common: PgInfoCommon {
            worker: WorkerStatus::Running,
            sql: SqlStatus::Healthy,
            readiness: Readiness::Ready,
            timeline: None,
            pg_config: PgConfig {
                port: None,
                hot_standby: None,
                primary_conninfo: None,
                primary_slot_name: None,
                extra: BTreeMap::new(),
            },
            last_refresh_at: Some(UnixMillis(1)),
        },
        wal_lsn: crate::state::WalLsn(1),
        slots: Vec::new(),
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

fn sample_dcs_state_with_trust(cfg: RuntimeConfig, trust: DcsTrust) -> DcsState {
    DcsState {
        trust,
        ..sample_dcs_state(cfg)
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
        decision: HaDecision::EnterFailSafe {
            release_leader_lease: false,
        },
    }
}

async fn get_ha_state_via_tcp(addr: SocketAddr) -> Result<HaStateResponse, WorkerError> {
    let mut stream = tokio::net::TcpStream::connect(addr)
        .await
        .map_err(|err| WorkerError::Message(format!("api connect failed: {err}")))?;
    stream
        .write_all(b"GET /ha/state HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .await
        .map_err(|err| WorkerError::Message(format!("api request write failed: {err}")))?;
    let mut raw = Vec::new();
    stream
        .read_to_end(&mut raw)
        .await
        .map_err(|err| WorkerError::Message(format!("api response read failed: {err}")))?;
    let text = String::from_utf8(raw)
        .map_err(|err| WorkerError::Message(format!("api response utf8 failed: {err}")))?;
    let (head, body) = text
        .split_once("\r\n\r\n")
        .ok_or_else(|| WorkerError::Message("api response missing header separator".to_string()))?;
    if !head.starts_with("HTTP/1.1 200") {
        return Err(WorkerError::Message(format!(
            "api returned unexpected response: {head}"
        )));
    }
    serde_json::from_str(body)
        .map_err(|err| WorkerError::Message(format!("api response decode failed: {err}")))
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
        postgres_conninfo: crate::pginfo::state::PgConnInfo {
            host: "127.0.0.1".to_string(),
            port: 1,
            user: "postgres".to_string(),
            dbname: "postgres".to_string(),
            application_name: None,
            connect_timeout_s: None,
            ssl_mode: crate::pginfo::state::PgSslMode::Prefer,
            options: None,
        },
        poll_interval: CONTRACT_WORKER_POLL_INTERVAL,
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
        poll_interval: CONTRACT_WORKER_POLL_INTERVAL,
        local_postgres_host: sample_runtime_config().postgres.listen_host.clone(),
        local_postgres_port: sample_runtime_config().postgres.listen_port,
        local_api_url: Some("http://127.0.0.1:8080".to_string()),
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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ha_state_api_stays_responsive_while_ha_attempt_leadership_blocks(
) -> Result<(), WorkerError> {
    let runtime_cfg = sample_runtime_config();
    let (_cfg_publisher, cfg_subscriber) = new_state_channel(runtime_cfg.clone(), UnixMillis(1));
    let (_pg_publisher, pg_subscriber) =
        new_state_channel(sample_primary_pg_state(), UnixMillis(1));
    let (_dcs_publisher, dcs_subscriber) = new_state_channel(
        sample_dcs_state_with_trust(runtime_cfg.clone(), DcsTrust::FullQuorum),
        UnixMillis(1),
    );
    let (_process_publisher, process_subscriber) =
        new_state_channel(sample_process_state(), UnixMillis(1));
    let (ha_publisher, ha_subscriber) = new_state_channel(sample_ha_state(), UnixMillis(1));

    let initial_snapshot = build_snapshot(
        &DebugSnapshotCtx {
            app: AppLifecycle::Running,
            config: cfg_subscriber.latest(),
            pg: pg_subscriber.latest(),
            dcs: dcs_subscriber.latest(),
            process: process_subscriber.latest(),
            ha: ha_subscriber.latest(),
        },
        UnixMillis(1),
        0,
        &[],
        &[],
    );
    let (debug_publisher, debug_subscriber) = new_state_channel(initial_snapshot, UnixMillis(1));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|err| WorkerError::Message(format!("api bind failed: {err}")))?;
    let mut api_ctx = crate::api::worker::ApiWorkerCtx::contract_stub(
        listener,
        cfg_subscriber.clone(),
        Box::new(ContractStore),
    );
    api_ctx.set_ha_snapshot_subscriber(debug_subscriber);
    let api_addr = api_ctx.local_addr()?;

    let mut debug_ctx = DebugApiCtx::contract_stub(DebugApiContractStubInputs {
        publisher: debug_publisher,
        config_subscriber: cfg_subscriber.clone(),
        pg_subscriber: pg_subscriber.clone(),
        dcs_subscriber: dcs_subscriber.clone(),
        process_subscriber: process_subscriber.clone(),
        ha_subscriber: ha_subscriber.clone(),
    });
    debug_ctx.app = AppLifecycle::Running;
    debug_ctx.poll_interval = DEBUG_API_TEST_POLL_INTERVAL;

    let (process_tx, _process_rx) = tokio::sync::mpsc::unbounded_channel();
    let (store, acquire_started_rx, acquire_release_tx) = BlockingAcquireStore::new();
    let mut ha_ctx = HaWorkerCtx::contract_stub(HaWorkerContractStubInputs {
        publisher: ha_publisher,
        config_subscriber: cfg_subscriber,
        pg_subscriber,
        dcs_subscriber,
        process_subscriber,
        process_inbox: process_tx,
        dcs_store: Box::new(store),
        scope: "scope-a".to_string(),
        self_id: MemberId("node-a".to_string()),
    });
    ha_ctx.state = HaState {
        worker: WorkerStatus::Running,
        phase: HaPhase::FailSafe,
        tick: 0,
        decision: HaDecision::NoChange,
    };

    let local = tokio::task::LocalSet::new();
    local
        .run_until(async move {
            let api_handle =
                tokio::task::spawn_local(async move { crate::api::worker::run(api_ctx).await });
            let debug_handle =
                tokio::task::spawn_local(
                    async move { crate::debug_api::worker::run(debug_ctx).await },
                );
            let ha_handle = tokio::task::spawn_local(async move {
                let result = crate::ha::worker::step_once(&mut ha_ctx).await;
                (ha_ctx, result)
            });

            let started_result = tokio::task::spawn_blocking(move || {
                acquire_started_rx.recv_timeout(CONTRACT_BLOCKING_START_TIMEOUT)
            })
            .await
            .map_err(|err| WorkerError::Message(format!("blocking wait join failed: {err}")))?;
            match started_result {
                Ok(()) => {}
                Err(err) => {
                    api_handle.abort();
                    debug_handle.abort();
                    ha_handle.abort();
                    return Err(WorkerError::Message(format!(
                        "blocking acquire-leader path did not start: {err}"
                    )));
                }
            }

            let deadline = tokio::time::Instant::now() + CONTRACT_API_RESPONSIVE_DEADLINE;
            let observed = loop {
                match get_ha_state_via_tcp(api_addr).await {
                    Ok(state)
                        if state.ha_phase == HaPhaseResponse::Primary && state.ha_tick == 1 =>
                    {
                        break state;
                    }
                    Ok(_state) => {}
                    Err(_err) => {}
                }
                if tokio::time::Instant::now() >= deadline {
                    api_handle.abort();
                    debug_handle.abort();
                    ha_handle.abort();
                    return Err(WorkerError::Message(
                        "timed out waiting for responsive /ha/state".to_string(),
                    ));
                }
                tokio::time::sleep(CONTRACT_WORKER_POLL_INTERVAL).await;
            };

            acquire_release_tx
                .send(())
                .map_err(|_| WorkerError::Message("acquire release signal failed".to_string()))?;
            let (ha_ctx, ha_result) = ha_handle
                .await
                .map_err(|err| WorkerError::Message(format!("ha step join failed: {err}")))?;
            ha_result?;

            api_handle.abort();
            debug_handle.abort();
            let _ = api_handle.await;
            let _ = debug_handle.await;

            assert_eq!(observed.ha_phase, HaPhaseResponse::Primary);
            assert!(observed.snapshot_sequence > 0);
            assert_eq!(ha_ctx.state.phase, HaPhase::Primary);
            Ok(())
        })
        .await
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
