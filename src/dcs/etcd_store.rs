use std::{
    collections::VecDeque,
    future::Future,
    str,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use etcd_client::{
    Client, Compare, CompareOp, EventType, GetOptions, Txn, TxnOp, WatchOptions, WatchResponse,
    WatchStream, Watcher,
};
use tokio::sync::mpsc as tokio_mpsc;

use super::store::{DcsStore, DcsStoreError, WatchEvent, WatchOp};
use crate::config::DcsEndpoint;

const COMMAND_TIMEOUT: Duration = Duration::from_secs(2);
const WORKER_BOOTSTRAP_TIMEOUT: Duration = Duration::from_secs(8);
const WATCH_IDLE_INTERVAL: Duration = Duration::from_millis(100);

enum WorkerCommand {
    Read {
        path: String,
        response_tx: mpsc::Sender<Result<Option<String>, DcsStoreError>>,
    },
    Write {
        path: String,
        value: String,
        response_tx: mpsc::Sender<Result<(), DcsStoreError>>,
    },
    PutIfAbsent {
        path: String,
        value: String,
        response_tx: mpsc::Sender<Result<bool, DcsStoreError>>,
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
    command_tx: tokio_mpsc::UnboundedSender<WorkerCommand>,
    worker_handle: Option<JoinHandle<()>>,
}

impl EtcdDcsStore {
    pub(crate) fn connect(endpoints: Vec<DcsEndpoint>, scope: &str) -> Result<Self, DcsStoreError> {
        Self::connect_with_worker_bootstrap_timeout(endpoints, scope, WORKER_BOOTSTRAP_TIMEOUT)
    }

    fn connect_with_worker_bootstrap_timeout(
        endpoints: Vec<DcsEndpoint>,
        scope: &str,
        worker_bootstrap_timeout: Duration,
    ) -> Result<Self, DcsStoreError> {
        if endpoints.is_empty() {
            return Err(DcsStoreError::Io(
                "at least one etcd endpoint is required".to_string(),
            ));
        }

        let scope_prefix = format!("/{}/", scope.trim_matches('/'));
        let healthy = Arc::new(AtomicBool::new(false));
        let events = Arc::new(Mutex::new(VecDeque::new()));
        let (command_tx, command_rx) = tokio_mpsc::unbounded_channel::<WorkerCommand>();
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

        match startup_rx.recv_timeout(worker_bootstrap_timeout) {
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
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // The worker might still be performing its bootstrap (connect + get + watch).
                // Request shutdown and close the command channel, but do not join here: joining
                // would turn this bounded startup timeout into an unbounded connect() call.
                let _ = command_tx.send(WorkerCommand::Shutdown);
                drop(command_tx);
                drop(worker_handle);
                Err(DcsStoreError::Io(format!(
                    "timed out waiting for etcd worker startup after {worker_bootstrap_timeout:?}"
                )))
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                let worker_panicked = worker_handle.join().is_err();
                let suffix = if worker_panicked {
                    " (worker panicked)"
                } else {
                    ""
                };
                Err(DcsStoreError::Io(format!(
                    "etcd worker exited before signaling startup{suffix}"
                )))
            }
        }
    }

    pub(crate) fn put_path_if_absent(
        &mut self,
        path: &str,
        value: String,
    ) -> Result<bool, DcsStoreError> {
        let (response_tx, response_rx) = mpsc::channel::<Result<bool, DcsStoreError>>();
        self.command_tx
            .send(WorkerCommand::PutIfAbsent {
                path: path.to_string(),
                value,
                response_tx,
            })
            .map_err(|err| {
                self.mark_unhealthy();
                DcsStoreError::Io(format!("send put-if-absent command failed: {err}"))
            })?;

        response_rx.recv_timeout(COMMAND_TIMEOUT).map_err(|err| {
            self.mark_unhealthy();
            DcsStoreError::Io(format!(
                "timed out waiting for put-if-absent response: {err}"
            ))
        })?
    }

    fn mark_unhealthy(&self) {
        self.healthy.store(false, Ordering::SeqCst);
    }
}

fn run_worker_loop(
    endpoints: Vec<DcsEndpoint>,
    scope_prefix: String,
    healthy: Arc<AtomicBool>,
    events: Arc<Mutex<VecDeque<WatchEvent>>>,
    mut command_rx: tokio_mpsc::UnboundedReceiver<WorkerCommand>,
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
        let mut had_successful_session = false;

        let (mut client, mut _watcher, mut watch_stream): (
            Option<Client>,
            Option<Watcher>,
            Option<WatchStream>,
        ) = match establish_watch_session(
            &endpoints,
            &scope_prefix,
            &events,
            had_successful_session,
        )
        .await
        {
            Ok((next_client, next_watcher, next_stream)) => {
                had_successful_session = true;
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
            if client.is_none() || watch_stream.is_none() {
                tokio::select! {
                    maybe_command = command_rx.recv() => {
                        let Some(command) = maybe_command else {
                            return;
                        };
                        if !handle_worker_command(
                            command,
                            &endpoints,
                            &healthy,
                            &events,
                            &mut client,
                            &mut _watcher,
                            &mut watch_stream,
                        ).await {
                            return;
                        }
                    }
                    _ = tokio::time::sleep(WATCH_IDLE_INTERVAL) => {
                        match establish_watch_session(
                            &endpoints,
                            &scope_prefix,
                            &events,
                            had_successful_session,
                        )
                        .await
                        {
                            Ok((next_client, next_watcher, next_stream)) => {
                                had_successful_session = true;
                                client = Some(next_client);
                                _watcher = Some(next_watcher);
                                watch_stream = Some(next_stream);
                                healthy.store(true, Ordering::SeqCst);
                            }
                            Err(_) => {
                                healthy.store(false, Ordering::SeqCst);
                            }
                        }
                    }
                }
                continue;
            }

            let Some(active_stream) = watch_stream.as_mut() else {
                tokio::time::sleep(WATCH_IDLE_INTERVAL).await;
                continue;
            };

            tokio::select! {
                maybe_command = command_rx.recv() => {
                    let Some(command) = maybe_command else {
                        return;
                    };
                    if !handle_worker_command(
                        command,
                        &endpoints,
                        &healthy,
                        &events,
                        &mut client,
                        &mut _watcher,
                        &mut watch_stream,
                    ).await {
                        return;
                    }
                }
                response = active_stream.message() => {
                    match response {
                        Ok(Some(response)) => {
                            if apply_watch_response(response, &events).is_err() {
                                if invalidate_watch_session(
                                    &healthy,
                                    &events,
                                    &mut client,
                                    &mut _watcher,
                                    &mut watch_stream,
                                )
                                .is_err()
                                {
                                    return;
                                }
                            } else {
                                healthy.store(true, Ordering::SeqCst);
                            }
                        }
                        Ok(None) | Err(_) => {
                            if invalidate_watch_session(
                                &healthy,
                                &events,
                                &mut client,
                                &mut _watcher,
                                &mut watch_stream,
                            )
                            .is_err()
                            {
                                return;
                            }
                        }
                    }
                }
                _ = tokio::time::sleep(WATCH_IDLE_INTERVAL) => {}
            }
        }
    });
}

async fn handle_worker_command(
    command: WorkerCommand,
    endpoints: &[DcsEndpoint],
    healthy: &Arc<AtomicBool>,
    events: &Arc<Mutex<VecDeque<WatchEvent>>>,
    client: &mut Option<Client>,
    watcher: &mut Option<Watcher>,
    watch_stream: &mut Option<WatchStream>,
) -> bool {
    match command {
        WorkerCommand::Write {
            path,
            value,
            response_tx,
        } => {
            let result = execute_write(endpoints, client, healthy, &path, value).await;
            let invalidate_result = if result.is_err() {
                invalidate_watch_session(healthy, events, client, watcher, watch_stream)
            } else {
                Ok(())
            };
            let _ = response_tx.send(result);
            invalidate_result.is_ok()
        }
        WorkerCommand::Read { path, response_tx } => {
            let result = execute_read(endpoints, client, healthy, &path).await;
            let invalidate_result = if result.is_err() {
                invalidate_watch_session(healthy, events, client, watcher, watch_stream)
            } else {
                Ok(())
            };
            let _ = response_tx.send(result);
            invalidate_result.is_ok()
        }
        WorkerCommand::PutIfAbsent {
            path,
            value,
            response_tx,
        } => {
            let result = execute_put_if_absent(endpoints, client, healthy, &path, value).await;
            let invalidate_result = if result.is_err() {
                invalidate_watch_session(healthy, events, client, watcher, watch_stream)
            } else {
                Ok(())
            };
            let _ = response_tx.send(result);
            invalidate_result.is_ok()
        }
        WorkerCommand::Delete { path, response_tx } => {
            let result = execute_delete(endpoints, client, healthy, &path).await;
            let invalidate_result = if result.is_err() {
                invalidate_watch_session(healthy, events, client, watcher, watch_stream)
            } else {
                Ok(())
            };
            let _ = response_tx.send(result);
            invalidate_result.is_ok()
        }
        WorkerCommand::Shutdown => false,
    }
}

fn invalidate_watch_session(
    healthy: &Arc<AtomicBool>,
    events: &Arc<Mutex<VecDeque<WatchEvent>>>,
    client: &mut Option<Client>,
    watcher: &mut Option<Watcher>,
    watch_stream: &mut Option<WatchStream>,
) -> Result<(), DcsStoreError> {
    healthy.store(false, Ordering::SeqCst);
    *client = None;
    *watcher = None;
    *watch_stream = None;
    clear_watch_events(events)
}

async fn establish_watch_session(
    endpoints: &[DcsEndpoint],
    scope_prefix: &str,
    events: &Arc<Mutex<VecDeque<WatchEvent>>>,
    is_reconnect: bool,
) -> Result<(Client, Watcher, WatchStream), DcsStoreError> {
    #[cfg(test)]
    apply_test_establish_delay().await;

    let mut client = connect_client(endpoints).await?;
    let snapshot_revision =
        bootstrap_snapshot(&mut client, scope_prefix, events, is_reconnect).await?;
    let start_revision = snapshot_revision.saturating_add(1);
    let (watcher, watch_stream) =
        create_watch_stream(&mut client, scope_prefix, start_revision).await?;
    Ok((client, watcher, watch_stream))
}

async fn connect_client(endpoints: &[DcsEndpoint]) -> Result<Client, DcsStoreError> {
    let client_endpoints = endpoints
        .iter()
        .map(DcsEndpoint::to_client_string)
        .collect::<Vec<_>>();
    timeout_etcd("etcd connect", Client::connect(client_endpoints, None)).await
}

async fn execute_write(
    endpoints: &[DcsEndpoint],
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
    endpoints: &[DcsEndpoint],
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

async fn execute_put_if_absent(
    endpoints: &[DcsEndpoint],
    client: &mut Option<Client>,
    healthy: &Arc<AtomicBool>,
    path: &str,
    value: String,
) -> Result<bool, DcsStoreError> {
    if client.is_none() {
        *client = Some(connect_client(endpoints).await?);
    }

    let Some(active_client) = client.as_mut() else {
        healthy.store(false, Ordering::SeqCst);
        return Err(DcsStoreError::Io(
            "etcd client unavailable for put-if-absent".to_string(),
        ));
    };

    let compare = Compare::version(path, CompareOp::Equal, 0);
    let then_put = TxnOp::put(path, value, None);
    let txn = Txn::new().when(vec![compare]).and_then(vec![then_put]);

    match timeout_etcd("etcd txn", active_client.txn(txn)).await {
        Ok(response) => {
            healthy.store(true, Ordering::SeqCst);
            Ok(response.succeeded())
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
    is_reconnect: bool,
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
    if is_reconnect {
        queue.push_back(WatchEvent {
            op: WatchOp::Reset,
            path: scope_prefix.to_string(),
            value: None,
            revision,
        });
    }
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

    if is_reconnect {
        replace_watch_events(events, queue)?;
    } else {
        enqueue_watch_events(events, queue)?;
    }
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

fn replace_watch_events(
    events: &Arc<Mutex<VecDeque<WatchEvent>>>,
    queue: VecDeque<WatchEvent>,
) -> Result<(), DcsStoreError> {
    let mut guard = events
        .lock()
        .map_err(|_| DcsStoreError::Io("events lock poisoned".to_string()))?;
    guard.clear();
    guard.extend(queue);
    Ok(())
}

fn clear_watch_events(events: &Arc<Mutex<VecDeque<WatchEvent>>>) -> Result<(), DcsStoreError> {
    let mut guard = events
        .lock()
        .map_err(|_| DcsStoreError::Io("events lock poisoned".to_string()))?;
    guard.clear();
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

#[cfg(test)]
use std::sync::atomic::AtomicU64;

#[cfg(test)]
static TEST_ESTABLISH_DELAY_MS: AtomicU64 = AtomicU64::new(0);

#[cfg(test)]
async fn apply_test_establish_delay() {
    let delay_ms = TEST_ESTABLISH_DELAY_MS.load(Ordering::SeqCst);
    if delay_ms == 0 {
        return;
    }
    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
}

async fn execute_read(
    endpoints: &[DcsEndpoint],
    client: &mut Option<Client>,
    healthy: &Arc<AtomicBool>,
    path: &str,
) -> Result<Option<String>, DcsStoreError> {
    if client.is_none() {
        *client = Some(connect_client(endpoints).await?);
    }

    let Some(active_client) = client.as_mut() else {
        healthy.store(false, Ordering::SeqCst);
        return Err(DcsStoreError::Io(
            "etcd client unavailable for read".to_string(),
        ));
    };

    match timeout_etcd("etcd get", active_client.get(path, None)).await {
        Ok(response) => {
            healthy.store(true, Ordering::SeqCst);
            let Some(kv) = response.kvs().first() else {
                return Ok(None);
            };
            let raw = kv.value();
            let decoded = String::from_utf8(raw.to_vec()).map_err(|err| {
                DcsStoreError::Io(format!("etcd read value not utf8 for `{path}`: {err}"))
            })?;
            Ok(Some(decoded))
        }
        Err(err) => {
            healthy.store(false, Ordering::SeqCst);
            *client = None;
            Err(err)
        }
    }
}

impl DcsStore for EtcdDcsStore {
    fn healthy(&self) -> bool {
        self.healthy.load(Ordering::SeqCst)
    }

    fn read_path(&mut self, path: &str) -> Result<Option<String>, DcsStoreError> {
        let (response_tx, response_rx) = mpsc::channel();
        self.command_tx
            .send(WorkerCommand::Read {
                path: path.to_string(),
                response_tx,
            })
            .map_err(|err| {
                self.mark_unhealthy();
                DcsStoreError::Io(format!("send read command failed: {err}"))
            })?;

        response_rx.recv_timeout(COMMAND_TIMEOUT).map_err(|err| {
            self.mark_unhealthy();
            DcsStoreError::Io(format!("timed out waiting for read command: {err}"))
        })?
    }

    fn write_path(&mut self, path: &str, value: String) -> Result<(), DcsStoreError> {
        let (response_tx, response_rx) = mpsc::channel();
        self.command_tx
            .send(WorkerCommand::Write {
                path: path.to_string(),
                value,
                response_tx,
            })
            .map_err(|err| {
                self.mark_unhealthy();
                DcsStoreError::Io(format!("send write command failed: {err}"))
            })?;

        response_rx.recv_timeout(COMMAND_TIMEOUT).map_err(|err| {
            self.mark_unhealthy();
            DcsStoreError::Io(format!("timed out waiting for write command: {err}"))
        })?
    }

    fn put_path_if_absent(&mut self, path: &str, value: String) -> Result<bool, DcsStoreError> {
        EtcdDcsStore::put_path_if_absent(self, path, value)
    }

    fn delete_path(&mut self, path: &str) -> Result<(), DcsStoreError> {
        let (response_tx, response_rx) = mpsc::channel();
        self.command_tx
            .send(WorkerCommand::Delete {
                path: path.to_string(),
                response_tx,
            })
            .map_err(|err| {
                self.mark_unhealthy();
                DcsStoreError::Io(format!("send delete command failed: {err}"))
            })?;

        response_rx.recv_timeout(COMMAND_TIMEOUT).map_err(|err| {
            self.mark_unhealthy();
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
        fs,
        path::PathBuf,
        time::{Duration, Instant},
    };

    use etcd_client::Client;

    use crate::{
        config::RuntimeConfig,
        dcs::{
            etcd_store::EtcdDcsStore,
            state::{
                DcsCache, DcsState, DcsTrust, DcsWorkerCtx, InitLockRecord, LeaderRecord,
                MemberRecord, MemberRole, SwitchoverRequest,
            },
            store::{refresh_from_etcd_watch, DcsStore, DcsStoreError, WatchEvent, WatchOp},
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

    type BoxError = Box<dyn std::error::Error + Send + Sync>;
    type TestResult = Result<(), BoxError>;

    fn boxed_error(message: impl Into<String>) -> BoxError {
        Box::new(std::io::Error::other(message.into()))
    }

    struct RealEtcdFixture {
        _guard: NamespaceGuard,
        handle: EtcdHandle,
        etcd_bin: PathBuf,
        namespace_id: String,
        log_dir: PathBuf,
        peer_port: u16,
        endpoint: String,
        scope: String,
    }

    impl RealEtcdFixture {
        async fn spawn(test_name: &str, scope: &str) -> Result<Self, HarnessError> {
            let etcd_bin = require_etcd_bin_for_real_tests()?;

            let guard = NamespaceGuard::new(test_name)?;
            let namespace = guard.namespace()?;
            let namespace_id = namespace.id.clone();
            let log_dir = namespace.child_dir("logs/etcd-store");
            let data_dir = prepare_etcd_data_dir(namespace)?;

            let reservation = allocate_ports(2)?;
            let ports = reservation.as_slice();
            let client_port = ports[0];
            let peer_port = ports[1];
            drop(reservation);

            let handle = spawn_etcd3(EtcdInstanceSpec {
                etcd_bin: etcd_bin.clone(),
                namespace_id: namespace_id.clone(),
                member_name: "node-a".to_string(),
                data_dir,
                log_dir: log_dir.clone(),
                client_port,
                peer_port,
                startup_timeout: Duration::from_secs(10),
            })
            .await?;

            Ok(Self {
                _guard: guard,
                handle,
                etcd_bin,
                namespace_id,
                log_dir,
                peer_port,
                endpoint: format!("http://127.0.0.1:{client_port}"),
                scope: scope.to_string(),
            })
        }

        async fn shutdown(&mut self) -> Result<(), HarnessError> {
            self.handle.shutdown().await
        }

        async fn restart_clean(&mut self) -> Result<(), HarnessError> {
            self.handle.shutdown().await?;

            if self.handle.data_dir.exists() {
                fs::remove_dir_all(&self.handle.data_dir)?;
            }
            fs::create_dir_all(&self.handle.data_dir)?;

            let client_port = self.handle.client_port;
            let data_dir = self.handle.data_dir.clone();
            let handle = spawn_etcd3(EtcdInstanceSpec {
                etcd_bin: self.etcd_bin.clone(),
                namespace_id: self.namespace_id.clone(),
                member_name: self.handle.member_name().to_string(),
                data_dir,
                log_dir: self.log_dir.clone(),
                client_port,
                peer_port: self.peer_port,
                startup_timeout: Duration::from_secs(10),
            })
            .await?;
            self.handle = handle;
            Ok(())
        }

        fn endpoint_model(&self) -> Result<crate::config::DcsEndpoint, BoxError> {
            crate::config::DcsEndpoint::parse(self.endpoint.as_str())
                .map_err(|err| boxed_error(format!("parse fixture endpoint failed: {err}")))
        }
    }

    struct EstablishDelayGuard {
        previous_ms: u64,
    }

    impl EstablishDelayGuard {
        fn new(delay_ms: u64) -> Self {
            let previous_ms =
                super::TEST_ESTABLISH_DELAY_MS.swap(delay_ms, std::sync::atomic::Ordering::SeqCst);
            Self { previous_ms }
        }
    }

    impl Drop for EstablishDelayGuard {
        fn drop(&mut self) {
            super::TEST_ESTABLISH_DELAY_MS
                .store(self.previous_ms, std::sync::atomic::Ordering::SeqCst);
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
        crate::test_harness::runtime_config::RuntimeConfigBuilder::new()
            .with_dcs_scope(scope)
            .build()
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
                local_postgres_host: "127.0.0.1".to_string(),
                local_postgres_port: 5432,
                pg_subscriber,
                publisher: dcs_publisher,
                store: Box::new(store),
                log: crate::logging::LogHandle::null(),
                cache: sample_cache(scope),
                last_published_pg_version: None,
                last_emitted_store_healthy: None,
                last_emitted_trust: None,
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
    async fn etcd_store_connect_timeout_returns_and_does_not_hang() -> TestResult {
        let fixture =
            RealEtcdFixture::spawn("dcs-etcd-store-connect-timeout", "scope-timeout").await?;

        let fixture = fixture;
        let result: TestResult = async {
            let _delay_guard = EstablishDelayGuard::new(2_500);
            let endpoint = fixture.endpoint_model()?;
            let scope = fixture.scope.clone();

            let handle = tokio::task::spawn_blocking(move || {
                let started_at = Instant::now();
                let store_result = EtcdDcsStore::connect_with_worker_bootstrap_timeout(
                    vec![endpoint],
                    scope.as_str(),
                    Duration::from_millis(50),
                );
                (started_at.elapsed(), store_result)
            });

            let outcome = tokio::time::timeout(Duration::from_secs(2), handle).await;
            let (elapsed, store_result) = match outcome {
                Ok(joined) => match joined {
                    Ok(out) => out,
                    Err(err) => {
                        return Err(boxed_error(format!(
                            "connect spawn_blocking join failed: {err}"
                        )));
                    }
                },
                Err(_) => {
                    return Err(boxed_error(
                        "timed out waiting for connect() to return after startup timeout",
                    ));
                }
            };

            if elapsed >= Duration::from_secs(1) {
                return Err(boxed_error(format!(
                    "expected connect() to return promptly after worker bootstrap timeout, elapsed={elapsed:?}",
                )));
            }

            match store_result {
                Ok(_) => Err(boxed_error(
                    "expected connect() to fail when worker bootstrap timeout is too small",
                )),
                Err(DcsStoreError::Io(message)) => {
                    if !message.contains("timed out waiting for etcd worker startup") {
                        return Err(boxed_error(format!(
                            "expected startup-timeout io error, got: {message}"
                        )));
                    }
                    Ok(())
                }
                Err(other) => Err(boxed_error(format!(
                    "expected io error for startup timeout, got: {other}"
                ))),
            }
        }
        .await;

        shutdown_with_result(fixture, result).await
    }

    #[tokio::test(flavor = "current_thread")]
    async fn etcd_store_reconnect_resets_cache_when_snapshot_is_empty() -> TestResult {
        let fixture =
            RealEtcdFixture::spawn("dcs-etcd-store-reconnect-reset", "scope-reconnect").await?;

        let mut fixture = fixture;
        let result: TestResult = async {
            let mut store = EtcdDcsStore::connect(vec![fixture.endpoint_model()?], &fixture.scope)?;
            let mut cache = sample_cache(&fixture.scope);

            cache.members.insert(
                MemberId("node-stale".to_string()),
                MemberRecord {
                    member_id: MemberId("node-stale".to_string()),
                    postgres_host: "10.0.0.10".to_string(),
                    postgres_port: 5432,
                    role: MemberRole::Primary,
                    sql: SqlStatus::Healthy,
                    readiness: Readiness::Ready,
                    timeline: None,
                    write_lsn: None,
                    replay_lsn: None,
                    updated_at: UnixMillis(1),
                    pg_version: crate::state::Version(1),
                },
            );
            cache.switchover = Some(SwitchoverRequest {
                switchover_to: None,
            });
            cache.init_lock = Some(InitLockRecord {
                holder: MemberId("node-stale".to_string()),
            });

            cache.leader = Some(LeaderRecord {
                member_id: MemberId("node-stale".to_string()),
            });

            let stale_leader = serde_json::to_string(&LeaderRecord {
                member_id: MemberId("node-stale".to_string()),
            })
            .map_err(|err| boxed_error(format!("encode leader json failed: {err}")))?;

            {
                let mut guard = store
                    .events
                    .lock()
                    .map_err(|_| boxed_error("events lock poisoned"))?;
                guard.push_back(WatchEvent {
                    op: WatchOp::Put,
                    path: format!("/{}/leader", fixture.scope),
                    value: Some(stale_leader),
                    revision: 1,
                });
            }

            fixture.restart_clean().await?;

            let deadline = Instant::now() + Duration::from_secs(10);
            let mut observed_reset = false;
            while Instant::now() < deadline {
                let events = store.drain_watch_events()?;
                if events.is_empty() {
                    tokio::time::sleep(Duration::from_millis(50)).await;
                    continue;
                }

                if events.iter().any(|event| event.op == WatchOp::Reset) {
                    if events.iter().any(|event| {
                        event.op == WatchOp::Put
                            && event.path == format!("/{}/leader", fixture.scope)
                    }) {
                        return Err(boxed_error(
                            "expected reconnect to replace the watch queue (dropping stale leader PUT)",
                        ));
                    }
                    refresh_from_etcd_watch(&fixture.scope, &mut cache, events)?;
                    observed_reset = true;
                    break;
                }

                if events.iter().any(|event| {
                    event.op == WatchOp::Put && event.path == format!("/{}/leader", fixture.scope)
                }) {
                    return Err(boxed_error(
                        "observed leader PUT before reconnect Reset marker; stale events must be cleared during disconnect window",
                    ));
                }
                return Err(boxed_error(format!(
                    "observed watch events before reconnect Reset marker: {events:?}"
                )));
            }

            if !observed_reset {
                return Err(boxed_error(
                    "timed out waiting for reconnect snapshot reset marker",
                ));
            }

            if cache.leader.is_some() {
                return Err(boxed_error(
                    "expected leader record to be cleared by reconnect reset",
                ));
            }
            if !cache.members.is_empty() {
                return Err(boxed_error(
                    "expected members to be cleared by reconnect reset",
                ));
            }
            if cache.switchover.is_some() {
                return Err(boxed_error(
                    "expected switchover record to be cleared by reconnect reset",
                ));
            }
            if cache.init_lock.is_some() {
                return Err(boxed_error(
                    "expected init lock record to be cleared by reconnect reset",
                ));
            }

            Ok(())
        }
        .await;

        shutdown_with_result(fixture, result).await
    }

    #[tokio::test(flavor = "current_thread")]
    async fn etcd_store_disconnect_clears_pending_queue_before_reconnect_snapshot() -> TestResult {
        let fixture = RealEtcdFixture::spawn(
            "dcs-etcd-store-disconnect-clears-queue",
            "scope-disconnect-clears-queue",
        )
        .await?;

        let mut fixture = fixture;
        let result: TestResult = async {
            let mut store = EtcdDcsStore::connect(vec![fixture.endpoint_model()?], &fixture.scope)?;

            let stale_leader = serde_json::to_string(&LeaderRecord {
                member_id: MemberId("node-stale".to_string()),
            })
            .map_err(|err| boxed_error(format!("encode leader json failed: {err}")))?;

            {
                let mut guard = store
                    .events
                    .lock()
                    .map_err(|_| boxed_error("events lock poisoned"))?;
                guard.push_back(WatchEvent {
                    op: WatchOp::Put,
                    path: format!("/{}/leader", fixture.scope),
                    value: Some(stale_leader),
                    revision: 1,
                });
            }

            {
                let _delay_guard = EstablishDelayGuard::new(1000);
                fixture.restart_clean().await?;

                let events = store.drain_watch_events()?;
                if events.iter().any(|event| event.op != WatchOp::Reset) {
                    return Err(boxed_error(format!(
                        "expected disconnect to clear queued watch events before reconnect Reset (allowing only Reset markers); observed={events:?}"
                    )));
                }
                if events.iter().any(|event| event.op == WatchOp::Reset) {
                    return Ok(());
                }
            }

            let reset_deadline = Instant::now() + Duration::from_secs(10);
            while Instant::now() < reset_deadline {
                let events = store.drain_watch_events()?;
                if events.iter().any(|event| event.op == WatchOp::Reset) {
                    return Ok(());
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
            }

            Err(boxed_error(
                "timed out waiting for reconnect Reset marker after etcd restart",
            ))
        }
        .await;

        shutdown_with_result(fixture, result).await
    }

    #[tokio::test(flavor = "current_thread")]
    async fn etcd_store_round_trips_write_delete_and_events() -> TestResult {
        let fixture = RealEtcdFixture::spawn("dcs-etcd-store-roundtrip", "scope-a").await?;

        let fixture = fixture;
        let result: TestResult = async {
            let mut store = EtcdDcsStore::connect(vec![fixture.endpoint_model()?], &fixture.scope)?;
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
    async fn etcd_store_put_if_absent_claims_only_once_and_does_not_overwrite() -> TestResult {
        let fixture = RealEtcdFixture::spawn("dcs-etcd-store-put-if-absent", "scope-put").await?;

        let fixture = fixture;
        let result: TestResult = async {
            let path_init = format!("/{}/init", fixture.scope);
            let path_config = format!("/{}/config", fixture.scope);

            let mut store_a = EtcdDcsStore::connect(vec![fixture.endpoint_model()?], &fixture.scope)?;
            let mut store_b = EtcdDcsStore::connect(vec![fixture.endpoint_model()?], &fixture.scope)?;

            let claimed_a = store_a.put_path_if_absent(path_init.as_str(), "init-a".to_string())?;
            let claimed_b = store_b.put_path_if_absent(path_init.as_str(), "init-b".to_string())?;
            if claimed_a == claimed_b {
                return Err(boxed_error(format!(
                    "expected exactly one init claim to succeed, got claimed_a={claimed_a} claimed_b={claimed_b}"
                )));
            }

            let seeded = store_a.put_path_if_absent(path_config.as_str(), "config-v1".to_string())?;
            if !seeded {
                return Err(boxed_error("expected config seed to succeed on first write"));
            }
            let seeded_again =
                store_b.put_path_if_absent(path_config.as_str(), "config-v2".to_string())?;
            if seeded_again {
                return Err(boxed_error(
                    "expected config seed to be rejected when key already exists",
                ));
            }

            let mut client = Client::connect(vec![fixture.endpoint.clone()], None)
                .await
                .map_err(|err| boxed_error(format!("etcd client connect failed: {err}")))?;
            let response = client
                .get(path_config.as_str(), None)
                .await
                .map_err(|err| boxed_error(format!("etcd get config failed: {err}")))?;
            let Some(kv) = response.kvs().first() else {
                return Err(boxed_error("expected config key to exist"));
            };
            let value = std::str::from_utf8(kv.value())
                .map_err(|err| boxed_error(format!("config value not utf8: {err}")))?;
            if value != "config-v1" {
                return Err(boxed_error(format!(
                    "expected config to remain 'config-v1', got: {value:?}"
                )));
            }

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
            let store = EtcdDcsStore::connect(vec![fixture.endpoint_model()?], &fixture.scope)?;
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
            let store = EtcdDcsStore::connect(vec![fixture.endpoint_model()?], &fixture.scope)?;
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
        match EtcdDcsStore::connect(
            vec![crate::config::DcsEndpoint::from_socket_addr(std::net::SocketAddr::from((
                [127, 0, 0, 1],
                1,
            )))],
            "scope-a",
        ) {
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
