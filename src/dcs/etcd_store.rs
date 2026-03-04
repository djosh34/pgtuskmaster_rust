use std::{
    collections::VecDeque,
    future::Future,
    str,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, TryRecvError},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use etcd_client::{
    Client, EventType, GetOptions, WatchOptions, WatchResponse, WatchStream, Watcher,
};

use super::store::{DcsStore, DcsStoreError, WatchEvent, WatchOp};

const COMMAND_TIMEOUT: Duration = Duration::from_secs(2);
const WATCH_IDLE_INTERVAL: Duration = Duration::from_millis(100);

enum WorkerCommand {
    Write {
        path: String,
        value: String,
        response_tx: mpsc::Sender<Result<(), DcsStoreError>>,
    },
    Delete {
        path: String,
        response_tx: mpsc::Sender<Result<(), DcsStoreError>>,
    },
    Shutdown,
}

pub(crate) struct EtcdDcsStore {
    healthy: Arc<AtomicBool>,
    events: Arc<Mutex<VecDeque<WatchEvent>>>,
    command_tx: mpsc::Sender<WorkerCommand>,
    worker_handle: Option<JoinHandle<()>>,
}

impl EtcdDcsStore {
    pub(crate) fn connect(endpoints: Vec<String>, scope: &str) -> Result<Self, DcsStoreError> {
        if endpoints.is_empty() {
            return Err(DcsStoreError::Io(
                "at least one etcd endpoint is required".to_string(),
            ));
        }

        let scope_prefix = format!("/{}/", scope.trim_matches('/'));
        let healthy = Arc::new(AtomicBool::new(false));
        let events = Arc::new(Mutex::new(VecDeque::new()));
        let (command_tx, command_rx) = mpsc::channel::<WorkerCommand>();
        let (startup_tx, startup_rx) = mpsc::channel::<Result<(), DcsStoreError>>();

        let worker_healthy = Arc::clone(&healthy);
        let worker_events = Arc::clone(&events);
        let worker_endpoints = endpoints;
        let worker_scope = scope_prefix;

        let worker_handle = thread::Builder::new()
            .name("etcd-dcs-store".to_string())
            .spawn(move || {
                run_worker_loop(
                    worker_endpoints,
                    worker_scope,
                    worker_healthy,
                    worker_events,
                    command_rx,
                    startup_tx,
                );
            })
            .map_err(|err| DcsStoreError::Io(format!("spawn etcd worker failed: {err}")))?;

        match startup_rx.recv_timeout(COMMAND_TIMEOUT) {
            Ok(Ok(())) => Ok(Self {
                healthy,
                events,
                command_tx,
                worker_handle: Some(worker_handle),
            }),
            Ok(Err(err)) => {
                let _ = worker_handle.join();
                Err(err)
            }
            Err(err) => {
                let _ = worker_handle.join();
                Err(DcsStoreError::Io(format!(
                    "timed out waiting for etcd worker startup: {err}"
                )))
            }
        }
    }
}

fn run_worker_loop(
    endpoints: Vec<String>,
    scope_prefix: String,
    healthy: Arc<AtomicBool>,
    events: Arc<Mutex<VecDeque<WatchEvent>>>,
    command_rx: mpsc::Receiver<WorkerCommand>,
    startup_tx: mpsc::Sender<Result<(), DcsStoreError>>,
) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build();

    let Ok(runtime) = runtime else {
        let _ = startup_tx.send(Err(DcsStoreError::Io(
            "failed to build tokio runtime for etcd store worker".to_string(),
        )));
        return;
    };

    runtime.block_on(async move {
        let (mut client, mut _watcher, mut watch_stream): (
            Option<Client>,
            Option<Watcher>,
            Option<WatchStream>,
        ) = match establish_watch_session(&endpoints, &scope_prefix, &events).await {
            Ok((next_client, next_watcher, next_stream)) => {
                healthy.store(true, Ordering::SeqCst);
                let _ = startup_tx.send(Ok(()));
                (Some(next_client), Some(next_watcher), Some(next_stream))
            }
            Err(err) => {
                healthy.store(false, Ordering::SeqCst);
                let _ = startup_tx.send(Err(err));
                return;
            }
        };

        loop {
            loop {
                match command_rx.try_recv() {
                    Ok(command) => match command {
                        WorkerCommand::Write {
                            path,
                            value,
                            response_tx,
                        } => {
                            let result =
                                execute_write(&endpoints, &mut client, &healthy, &path, value)
                                    .await;
                            if result.is_err() {
                                _watcher = None;
                                watch_stream = None;
                            }
                            let _ = response_tx.send(result);
                        }
                        WorkerCommand::Delete { path, response_tx } => {
                            let result =
                                execute_delete(&endpoints, &mut client, &healthy, &path).await;
                            if result.is_err() {
                                _watcher = None;
                                watch_stream = None;
                            }
                            let _ = response_tx.send(result);
                        }
                        WorkerCommand::Shutdown => return,
                    },
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => return,
                }
            }

            if client.is_none() || watch_stream.is_none() {
                match establish_watch_session(&endpoints, &scope_prefix, &events).await {
                    Ok((next_client, next_watcher, next_stream)) => {
                        client = Some(next_client);
                        _watcher = Some(next_watcher);
                        watch_stream = Some(next_stream);
                        healthy.store(true, Ordering::SeqCst);
                    }
                    Err(_) => {
                        healthy.store(false, Ordering::SeqCst);
                        tokio::time::sleep(WATCH_IDLE_INTERVAL).await;
                    }
                }
                continue;
            }

            let Some(active_stream) = watch_stream.as_mut() else {
                tokio::time::sleep(WATCH_IDLE_INTERVAL).await;
                continue;
            };

            match tokio::time::timeout(WATCH_IDLE_INTERVAL, active_stream.message()).await {
                Ok(Ok(Some(response))) => {
                    if apply_watch_response(response, &events).is_err() {
                        healthy.store(false, Ordering::SeqCst);
                        client = None;
                        _watcher = None;
                        watch_stream = None;
                    } else {
                        healthy.store(true, Ordering::SeqCst);
                    }
                }
                Ok(Ok(None)) => {
                    healthy.store(false, Ordering::SeqCst);
                    client = None;
                    _watcher = None;
                    watch_stream = None;
                }
                Ok(Err(_)) => {
                    healthy.store(false, Ordering::SeqCst);
                    client = None;
                    _watcher = None;
                    watch_stream = None;
                }
                Err(_) => {}
            }
        }
    });
}

async fn establish_watch_session(
    endpoints: &[String],
    scope_prefix: &str,
    events: &Arc<Mutex<VecDeque<WatchEvent>>>,
) -> Result<(Client, Watcher, WatchStream), DcsStoreError> {
    let mut client = connect_client(endpoints).await?;
    let snapshot_revision = bootstrap_snapshot(&mut client, scope_prefix, events).await?;
    let start_revision = snapshot_revision.saturating_add(1);
    let (watcher, watch_stream) =
        create_watch_stream(&mut client, scope_prefix, start_revision).await?;
    Ok((client, watcher, watch_stream))
}

async fn connect_client(endpoints: &[String]) -> Result<Client, DcsStoreError> {
    timeout_etcd("etcd connect", Client::connect(endpoints.to_vec(), None)).await
}

async fn execute_write(
    endpoints: &[String],
    client: &mut Option<Client>,
    healthy: &Arc<AtomicBool>,
    path: &str,
    value: String,
) -> Result<(), DcsStoreError> {
    if client.is_none() {
        *client = Some(connect_client(endpoints).await?);
    }

    let Some(active_client) = client.as_mut() else {
        healthy.store(false, Ordering::SeqCst);
        return Err(DcsStoreError::Io(
            "etcd client unavailable for write".to_string(),
        ));
    };

    match timeout_etcd("etcd put", active_client.put(path, value, None)).await {
        Ok(_) => {
            healthy.store(true, Ordering::SeqCst);
            Ok(())
        }
        Err(err) => {
            healthy.store(false, Ordering::SeqCst);
            *client = None;
            Err(err)
        }
    }
}

async fn execute_delete(
    endpoints: &[String],
    client: &mut Option<Client>,
    healthy: &Arc<AtomicBool>,
    path: &str,
) -> Result<(), DcsStoreError> {
    if client.is_none() {
        *client = Some(connect_client(endpoints).await?);
    }

    let Some(active_client) = client.as_mut() else {
        healthy.store(false, Ordering::SeqCst);
        return Err(DcsStoreError::Io(
            "etcd client unavailable for delete".to_string(),
        ));
    };

    match timeout_etcd("etcd delete", active_client.delete(path, None)).await {
        Ok(_) => {
            healthy.store(true, Ordering::SeqCst);
            Ok(())
        }
        Err(err) => {
            healthy.store(false, Ordering::SeqCst);
            *client = None;
            Err(err)
        }
    }
}

async fn bootstrap_snapshot(
    client: &mut Client,
    scope_prefix: &str,
    events: &Arc<Mutex<VecDeque<WatchEvent>>>,
) -> Result<i64, DcsStoreError> {
    let response = timeout_etcd(
        "etcd get",
        client.get(scope_prefix, Some(GetOptions::new().with_prefix())),
    )
    .await?;

    let revision = response
        .header()
        .map(|header| header.revision())
        .unwrap_or(0);

    let mut queue = VecDeque::new();
    for kv in response.kvs() {
        let path = str::from_utf8(kv.key()).map_err(|err| DcsStoreError::Decode {
            key: "watch-key".to_string(),
            message: err.to_string(),
        })?;
        let value = str::from_utf8(kv.value()).map_err(|err| DcsStoreError::Decode {
            key: path.to_string(),
            message: err.to_string(),
        })?;

        queue.push_back(WatchEvent {
            op: WatchOp::Put,
            path: path.to_string(),
            value: Some(value.to_string()),
            revision: kv.mod_revision(),
        });
    }

    enqueue_watch_events(events, queue)?;
    Ok(revision)
}

async fn create_watch_stream(
    client: &mut Client,
    scope_prefix: &str,
    start_revision: i64,
) -> Result<(Watcher, WatchStream), DcsStoreError> {
    let watch_options = WatchOptions::new()
        .with_prefix()
        .with_start_revision(start_revision);
    timeout_etcd(
        "etcd watch",
        client.watch(scope_prefix, Some(watch_options)),
    )
    .await
}

fn apply_watch_response(
    response: WatchResponse,
    events: &Arc<Mutex<VecDeque<WatchEvent>>>,
) -> Result<(), DcsStoreError> {
    if response.canceled() || response.compact_revision() > 0 {
        return Err(DcsStoreError::Io(format!(
            "etcd watch canceled: reason='{}' compact_revision={}",
            response.cancel_reason(),
            response.compact_revision()
        )));
    }

    let mut queue = VecDeque::new();
    for event in response.events() {
        let Some(kv) = event.kv() else {
            return Err(DcsStoreError::Io(
                "etcd watch event missing key-value payload".to_string(),
            ));
        };

        let path = str::from_utf8(kv.key()).map_err(|err| DcsStoreError::Decode {
            key: "watch-key".to_string(),
            message: err.to_string(),
        })?;

        match event.event_type() {
            EventType::Put => {
                let value = str::from_utf8(kv.value()).map_err(|err| DcsStoreError::Decode {
                    key: path.to_string(),
                    message: err.to_string(),
                })?;
                queue.push_back(WatchEvent {
                    op: WatchOp::Put,
                    path: path.to_string(),
                    value: Some(value.to_string()),
                    revision: kv.mod_revision(),
                });
            }
            EventType::Delete => {
                queue.push_back(WatchEvent {
                    op: WatchOp::Delete,
                    path: path.to_string(),
                    value: None,
                    revision: kv.mod_revision(),
                });
            }
        }
    }

    enqueue_watch_events(events, queue)
}

fn enqueue_watch_events(
    events: &Arc<Mutex<VecDeque<WatchEvent>>>,
    queue: VecDeque<WatchEvent>,
) -> Result<(), DcsStoreError> {
    let mut guard = events
        .lock()
        .map_err(|_| DcsStoreError::Io("events lock poisoned".to_string()))?;
    guard.extend(queue);
    Ok(())
}

async fn timeout_etcd<T, F>(operation: &str, fut: F) -> Result<T, DcsStoreError>
where
    F: Future<Output = Result<T, etcd_client::Error>>,
{
    match tokio::time::timeout(COMMAND_TIMEOUT, fut).await {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(err)) => Err(DcsStoreError::Io(format!("{operation} failed: {err}"))),
        Err(err) => Err(DcsStoreError::Io(format!("{operation} timed out: {err}"))),
    }
}

impl DcsStore for EtcdDcsStore {
    fn healthy(&self) -> bool {
        self.healthy.load(Ordering::SeqCst)
    }

    fn write_path(&mut self, path: &str, value: String) -> Result<(), DcsStoreError> {
        let (response_tx, response_rx) = mpsc::channel();
        self.command_tx
            .send(WorkerCommand::Write {
                path: path.to_string(),
                value,
                response_tx,
            })
            .map_err(|err| DcsStoreError::Io(format!("send write command failed: {err}")))?;

        response_rx.recv_timeout(COMMAND_TIMEOUT).map_err(|err| {
            DcsStoreError::Io(format!("timed out waiting for write command: {err}"))
        })?
    }

    fn delete_path(&mut self, path: &str) -> Result<(), DcsStoreError> {
        let (response_tx, response_rx) = mpsc::channel();
        self.command_tx
            .send(WorkerCommand::Delete {
                path: path.to_string(),
                response_tx,
            })
            .map_err(|err| DcsStoreError::Io(format!("send delete command failed: {err}")))?;

        response_rx.recv_timeout(COMMAND_TIMEOUT).map_err(|err| {
            DcsStoreError::Io(format!("timed out waiting for delete command: {err}"))
        })?
    }

    fn drain_watch_events(&mut self) -> Result<Vec<WatchEvent>, DcsStoreError> {
        let mut guard = self
            .events
            .lock()
            .map_err(|_| DcsStoreError::Io("events lock poisoned".to_string()))?;
        Ok(guard.drain(..).collect())
    }
}

impl Drop for EtcdDcsStore {
    fn drop(&mut self) {
        let _ = self.command_tx.send(WorkerCommand::Shutdown);
        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        time::{Duration, Instant},
    };

    use etcd_client::Client;

    use crate::{
        config::{
            schema::{ClusterConfig, DebugConfig, HaConfig, PostgresConfig},
            ApiAuthConfig, ApiConfig, ApiSecurityConfig, ApiTlsMode, BinaryPaths, DcsConfig,
            InlineOrPath, LogCleanupConfig, LogLevel, LoggingConfig, PgHbaConfig, PgIdentConfig,
            PostgresConnIdentityConfig, PostgresLoggingConfig, PostgresRoleConfig,
            PostgresRolesConfig, ProcessConfig, RoleAuthConfig, RuntimeConfig, StderrSinkConfig,
            TlsServerConfig,
        },
        dcs::{
            etcd_store::EtcdDcsStore,
            state::{DcsCache, DcsState, DcsTrust, DcsWorkerCtx, LeaderRecord},
            store::{DcsStore, DcsStoreError, WatchOp},
            worker::step_once,
        },
        pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
        state::{new_state_channel, MemberId, UnixMillis, WorkerError, WorkerStatus},
        test_harness::{
            binaries::require_etcd_bin_for_real_tests,
            etcd3::{prepare_etcd_data_dir, spawn_etcd3, EtcdHandle, EtcdInstanceSpec},
            namespace::NamespaceGuard,
            ports::allocate_ports,
            HarnessError,
        },
    };
    use crate::pginfo::conninfo::PgSslMode;

    type BoxError = Box<dyn std::error::Error + Send + Sync>;
    type TestResult = Result<(), BoxError>;

    fn boxed_error(message: impl Into<String>) -> BoxError {
        Box::new(std::io::Error::other(message.into()))
    }

    struct RealEtcdFixture {
        _guard: NamespaceGuard,
        handle: EtcdHandle,
        endpoint: String,
        scope: String,
    }

    impl RealEtcdFixture {
        async fn spawn(test_name: &str, scope: &str) -> Result<Self, HarnessError> {
            let etcd_bin = require_etcd_bin_for_real_tests()?;

            let guard = NamespaceGuard::new(test_name)?;
            let namespace = guard.namespace()?;
            let data_dir = prepare_etcd_data_dir(namespace)?;

            let reservation = allocate_ports(2)?;
            let ports = reservation.as_slice();
            let client_port = ports[0];
            let peer_port = ports[1];
            drop(reservation);

            let log_dir = namespace.child_dir("logs/etcd-store");
            let handle = spawn_etcd3(EtcdInstanceSpec {
                etcd_bin,
                namespace_id: namespace.id.clone(),
                member_name: "node-a".to_string(),
                data_dir,
                log_dir,
                client_port,
                peer_port,
                startup_timeout: Duration::from_secs(10),
            })
            .await?;

            Ok(Self {
                _guard: guard,
                handle,
                endpoint: format!("http://127.0.0.1:{client_port}"),
                scope: scope.to_string(),
            })
        }

        async fn shutdown(&mut self) -> Result<(), HarnessError> {
            self.handle.shutdown().await
        }
    }

    fn wait_for_event(
        store: &mut dyn DcsStore,
        op: WatchOp,
        path: &str,
        timeout: Duration,
    ) -> Result<(), DcsStoreError> {
        let deadline = Instant::now() + timeout;
        loop {
            for event in store.drain_watch_events()? {
                if event.op == op && event.path == path {
                    return Ok(());
                }
            }
            if Instant::now() >= deadline {
                return Err(DcsStoreError::Io(format!(
                    "timed out waiting for event {:?} at {}",
                    op, path
                )));
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    fn sample_runtime_config(scope: &str) -> RuntimeConfig {
        RuntimeConfig {
            cluster: ClusterConfig {
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
                    user: "postgres".to_string(),
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
                        content: String::new(),
                    },
                },
                pg_ident: PgIdentConfig {
                    source: InlineOrPath::Inline {
                        content: String::new(),
                    },
                },
            },
            dcs: DcsConfig {
                endpoints: vec!["http://127.0.0.1:2379".to_string()],
                scope: scope.to_string(),
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
                binaries: BinaryPaths {
                    postgres: "/usr/bin/postgres".into(),
                    pg_ctl: "/usr/bin/pg_ctl".into(),
                    pg_rewind: "/usr/bin/pg_rewind".into(),
                    initdb: "/usr/bin/initdb".into(),
                    pg_basebackup: "/usr/bin/pg_basebackup".into(),
                    psql: "/usr/bin/psql".into(),
                },
            },
            logging: LoggingConfig {
                level: LogLevel::Info,
                capture_subprocess_output: true,
                postgres: PostgresLoggingConfig {
                    enabled: true,
                    pg_ctl_log_file: None,
                    log_dir: None,
                    archive_command_log_file: None,
                    poll_interval_ms: 200,
                    cleanup: LogCleanupConfig {
                        enabled: true,
                        max_files: 10,
                        max_age_seconds: 60,
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

    fn sample_cache(scope: &str) -> DcsCache {
        DcsCache {
            members: BTreeMap::new(),
            leader: None,
            switchover: None,
            config: sample_runtime_config(scope),
            init_lock: None,
        }
    }

    fn sample_pg() -> PgInfoState {
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
            wal_lsn: crate::state::WalLsn(42),
            slots: Vec::new(),
        }
    }

    fn build_worker_ctx(
        scope: &str,
        store: EtcdDcsStore,
    ) -> (DcsWorkerCtx, crate::state::StateSubscriber<DcsState>) {
        let self_id = MemberId("node-a".to_string());
        let initial_pg = sample_pg();
        let (_pg_publisher, pg_subscriber) = new_state_channel(initial_pg, UnixMillis(1));

        let initial_dcs = DcsState {
            worker: WorkerStatus::Starting,
            trust: DcsTrust::NotTrusted,
            cache: sample_cache(scope),
            last_refresh_at: Some(UnixMillis(1)),
        };
        let (dcs_publisher, dcs_subscriber) = new_state_channel(initial_dcs, UnixMillis(1));

        (
            DcsWorkerCtx {
                self_id,
                scope: scope.to_string(),
                poll_interval: Duration::from_millis(50),
                pg_subscriber,
                publisher: dcs_publisher,
                store: Box::new(store),
                cache: sample_cache(scope),
                last_published_pg_version: None,
            },
            dcs_subscriber,
        )
    }

    async fn shutdown_with_result(mut fixture: RealEtcdFixture, result: TestResult) -> TestResult {
        let shutdown_result = fixture.shutdown().await;
        match result {
            Err(err) => Err(err),
            Ok(()) => match shutdown_result {
                Ok(()) => Ok(()),
                Err(err) => Err(Box::new(err)),
            },
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn etcd_store_round_trips_write_delete_and_events() -> TestResult {
        let fixture = RealEtcdFixture::spawn("dcs-etcd-store-roundtrip", "scope-a").await?;

        let fixture = fixture;
        let result: TestResult = async {
            let mut store = EtcdDcsStore::connect(vec![fixture.endpoint.clone()], &fixture.scope)?;
            let path = format!("/{}/member/node-a", fixture.scope);
            let value = r#"{"member_id":"node-a","role":"Primary"}"#.to_string();

            store.write_path(path.as_str(), value)?;
            wait_for_event(
                &mut store,
                WatchOp::Put,
                path.as_str(),
                Duration::from_secs(5),
            )?;

            store.delete_path(path.as_str())?;
            wait_for_event(
                &mut store,
                WatchOp::Delete,
                path.as_str(),
                Duration::from_secs(5),
            )?;

            Ok(())
        }
        .await;

        shutdown_with_result(fixture, result).await
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_consumes_real_etcd_watch_path_without_mocking() -> TestResult {
        let fixture = RealEtcdFixture::spawn("dcs-etcd-store-step-once", "scope-b").await?;

        let fixture = fixture;
        let result: TestResult = async {
            let store = EtcdDcsStore::connect(vec![fixture.endpoint.clone()], &fixture.scope)?;
            let mut client = Client::connect(vec![fixture.endpoint.clone()], None)
                .await
                .map_err(|err| boxed_error(format!("etcd client connect failed: {err}")))?;

            let leader_path = format!("/{}/leader", fixture.scope);
            let leader_json = serde_json::to_string(&LeaderRecord {
                member_id: MemberId("node-b".to_string()),
            })
            .map_err(|err| boxed_error(format!("encode leader json failed: {err}")))?;

            client
                .put(leader_path.as_str(), leader_json, None)
                .await
                .map_err(|err| boxed_error(format!("put leader key failed: {err}")))?;

            let (mut ctx, dcs_subscriber) = build_worker_ctx(&fixture.scope, store);
            let self_member = MemberId("node-a".to_string());
            let expected_leader = MemberId("node-b".to_string());

            let deadline = Instant::now() + Duration::from_secs(5);
            let mut observed = false;
            while Instant::now() < deadline {
                step_once(&mut ctx)
                    .await
                    .map_err(|err| boxed_error(format!("dcs step_once failed: {err}")))?;

                let latest = dcs_subscriber.latest();
                let leader_matches = latest
                    .value
                    .cache
                    .leader
                    .as_ref()
                    .map(|leader| leader.member_id.clone())
                    == Some(expected_leader.clone());
                let self_member_written = latest.value.cache.members.contains_key(&self_member);
                if leader_matches && self_member_written {
                    observed = true;
                    break;
                }

                tokio::time::sleep(Duration::from_millis(50)).await;
            }

            if !observed {
                return Err(boxed_error(
                    "timed out waiting for step_once to publish real-etcd leader/member refresh",
                ));
            }

            let member_path = format!("/{}/member/node-a", fixture.scope);
            let member_response = client
                .get(member_path.as_str(), None)
                .await
                .map_err(|err| boxed_error(format!("get member key failed: {err}")))?;
            if member_response.kvs().is_empty() {
                return Err(boxed_error(
                    "expected member key to be persisted at /{scope}/member/{id}",
                ));
            }

            Ok(())
        }
        .await;

        shutdown_with_result(fixture, result).await
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_marks_store_unhealthy_on_real_decode_failure() -> TestResult {
        let fixture = RealEtcdFixture::spawn("dcs-etcd-store-decode-failure", "scope-c").await?;

        let fixture = fixture;
        let result: TestResult = async {
            let store = EtcdDcsStore::connect(vec![fixture.endpoint.clone()], &fixture.scope)?;
            let mut client = Client::connect(vec![fixture.endpoint.clone()], None)
                .await
                .map_err(|err| boxed_error(format!("etcd client connect failed: {err}")))?;

            let leader_path = format!("/{}/leader", fixture.scope);
            client
                .put(leader_path.as_str(), "not-json", None)
                .await
                .map_err(|err| boxed_error(format!("put malformed leader key failed: {err}")))?;

            let (mut ctx, dcs_subscriber) = build_worker_ctx(&fixture.scope, store);
            let expected_worker =
                WorkerStatus::Faulted(WorkerError::Message("dcs store unhealthy".to_string()));

            let deadline = Instant::now() + Duration::from_secs(5);
            let mut observed_fault = false;
            while Instant::now() < deadline {
                step_once(&mut ctx)
                    .await
                    .map_err(|err| boxed_error(format!("dcs step_once failed: {err}")))?;

                let latest = dcs_subscriber.latest();
                if latest.value.worker == expected_worker
                    && latest.value.trust == DcsTrust::NotTrusted
                {
                    observed_fault = true;
                    break;
                }

                tokio::time::sleep(Duration::from_millis(50)).await;
            }

            if !observed_fault {
                return Err(boxed_error(
                    "timed out waiting for decode failure to fault dcs worker state",
                ));
            }

            Ok(())
        }
        .await;

        shutdown_with_result(fixture, result).await
    }

    #[tokio::test(flavor = "current_thread")]
    async fn etcd_store_write_reports_unreachable_endpoint() -> TestResult {
        match EtcdDcsStore::connect(vec!["http://127.0.0.1:1".to_string()], "scope-a") {
            Ok(mut store) => match store.write_path("/scope-a/member/node-a", "{}".to_string()) {
                Ok(_) => Err(boxed_error(
                    "expected write against unreachable endpoint to fail",
                )),
                Err(DcsStoreError::Io(_)) => Ok(()),
                Err(other) => Err(boxed_error(format!(
                    "expected io error for unreachable endpoint write, got {other}"
                ))),
            },
            Err(DcsStoreError::Io(_)) => Ok(()),
            Err(other) => Err(boxed_error(format!(
                "expected io error for unreachable endpoint connect, got {other}"
            ))),
        }
    }
}
