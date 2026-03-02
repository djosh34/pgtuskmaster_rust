use std::{
    collections::{BTreeMap, VecDeque},
    str,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use etcd_client::{Client, GetOptions};

use super::store::{DcsStore, DcsStoreError, WatchEvent, WatchOp};

const COMMAND_TIMEOUT: Duration = Duration::from_secs(5);
const WATCH_POLL_INTERVAL: Duration = Duration::from_millis(100);

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
        let mut client = match connect_client(&endpoints).await {
            Ok(client) => {
                healthy.store(true, Ordering::SeqCst);
                let _ = startup_tx.send(Ok(()));
                Some(client)
            }
            Err(err) => {
                healthy.store(false, Ordering::SeqCst);
                let _ = startup_tx.send(Err(err));
                None
            }
        };

        let mut cache = BTreeMap::<String, String>::new();

        if let Some(active_client) = client.as_mut() {
            if refresh_events(active_client, &scope_prefix, &mut cache, &events).await.is_err() {
                healthy.store(false, Ordering::SeqCst);
                client = None;
            }
        }

        let mut ticker = tokio::time::interval(WATCH_POLL_INTERVAL);

        loop {
            while let Ok(command) = command_rx.try_recv() {
                match command {
                    WorkerCommand::Write {
                        path,
                        value,
                        response_tx,
                    } => {
                        let result =
                            execute_write(&endpoints, &mut client, &healthy, &path, value).await;
                        let _ = response_tx.send(result);
                    }
                    WorkerCommand::Delete { path, response_tx } => {
                        let result = execute_delete(&endpoints, &mut client, &healthy, &path).await;
                        let _ = response_tx.send(result);
                    }
                    WorkerCommand::Shutdown => return,
                }
            }

            ticker.tick().await;

            if client.is_none() {
                client = match connect_client(&endpoints).await {
                    Ok(next_client) => {
                        healthy.store(true, Ordering::SeqCst);
                        Some(next_client)
                    }
                    Err(_) => {
                        healthy.store(false, Ordering::SeqCst);
                        None
                    }
                };
            }

            let Some(active_client) = client.as_mut() else {
                continue;
            };

            if refresh_events(active_client, &scope_prefix, &mut cache, &events)
                .await
                .is_err()
            {
                healthy.store(false, Ordering::SeqCst);
                client = None;
            } else {
                healthy.store(true, Ordering::SeqCst);
            }
        }
    });
}

async fn connect_client(endpoints: &[String]) -> Result<Client, DcsStoreError> {
    Client::connect(endpoints.to_vec(), None)
        .await
        .map_err(|err| DcsStoreError::Io(format!("etcd connect failed: {err}")))
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

    match active_client.put(path, value, None).await {
        Ok(_) => {
            healthy.store(true, Ordering::SeqCst);
            Ok(())
        }
        Err(err) => {
            healthy.store(false, Ordering::SeqCst);
            *client = None;
            Err(DcsStoreError::Io(format!("etcd put failed: {err}")))
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

    match active_client.delete(path, None).await {
        Ok(_) => {
            healthy.store(true, Ordering::SeqCst);
            Ok(())
        }
        Err(err) => {
            healthy.store(false, Ordering::SeqCst);
            *client = None;
            Err(DcsStoreError::Io(format!("etcd delete failed: {err}")))
        }
    }
}

async fn refresh_events(
    client: &mut Client,
    scope_prefix: &str,
    cache: &mut BTreeMap<String, String>,
    events: &Arc<Mutex<VecDeque<WatchEvent>>>,
) -> Result<(), DcsStoreError> {
    let response = client
        .get(scope_prefix, Some(GetOptions::new().with_prefix()))
        .await
        .map_err(|err| DcsStoreError::Io(format!("etcd get failed: {err}")))?;

    let default_revision = response
        .header()
        .map(|header| header.revision())
        .unwrap_or_default();

    let mut next = BTreeMap::<String, String>::new();
    let mut queue = VecDeque::<WatchEvent>::new();

    for kv in response.kvs() {
        let path = str::from_utf8(kv.key()).map_err(|err| DcsStoreError::Decode {
            key: "watch-key".to_string(),
            message: err.to_string(),
        })?;
        let value = str::from_utf8(kv.value()).map_err(|err| DcsStoreError::Decode {
            key: path.to_string(),
            message: err.to_string(),
        })?;

        let path_owned = path.to_string();
        let value_owned = value.to_string();
        let changed = cache.get(&path_owned) != Some(&value_owned);
        if changed {
            queue.push_back(WatchEvent {
                op: WatchOp::Put,
                path: path_owned.clone(),
                value: Some(value_owned.clone()),
                revision: kv.mod_revision(),
            });
        }
        next.insert(path_owned, value_owned);
    }

    for missing_key in cache.keys() {
        if !next.contains_key(missing_key) {
            queue.push_back(WatchEvent {
                op: WatchOp::Delete,
                path: missing_key.clone(),
                value: None,
                revision: default_revision,
            });
        }
    }

    *cache = next;

    let mut guard = events
        .lock()
        .map_err(|_| DcsStoreError::Io("events lock poisoned".to_string()))?;
    guard.extend(queue);
    Ok(())
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
    use std::time::{Duration, Instant};

    use crate::dcs::{
        etcd_store::EtcdDcsStore,
        store::{DcsStore, DcsStoreError, WatchOp},
    };
    use crate::test_harness::{
        binaries::require_etcd_bin,
        etcd3::{prepare_etcd_data_dir, spawn_etcd3, EtcdInstanceSpec},
        namespace::NamespaceGuard,
        ports::allocate_ports,
        HarnessError,
    };

    type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

    fn boxed_error(message: impl Into<String>) -> Box<dyn std::error::Error + Send + Sync> {
        Box::new(std::io::Error::other(message.into()))
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

    #[tokio::test(flavor = "current_thread")]
    async fn etcd_store_round_trips_write_delete_and_events() -> TestResult {
        let etcd_bin = require_etcd_bin()?;
        let guard = NamespaceGuard::new("dcs-etcd-store-roundtrip")?;
        let namespace = guard.namespace()?;
        let data_dir = prepare_etcd_data_dir(namespace)?;

        let reservation = allocate_ports(2)?;
        let ports = reservation.as_slice();
        let client_port = ports[0];
        let peer_port = ports[1];
        drop(reservation);

        let log_dir = namespace.child_dir("logs/etcd-store");
        let mut handle = spawn_etcd3(EtcdInstanceSpec {
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

        let endpoint = format!("http://127.0.0.1:{client_port}");
        let mut store = EtcdDcsStore::connect(vec![endpoint], "scope-a")?;
        let path = "/scope-a/member/node-a";
        let value = r#"{"member_id":"node-a","role":"Primary"}"#.to_string();

        store.write_path(path, value)?;
        wait_for_event(&mut store, WatchOp::Put, path, Duration::from_secs(5))?;

        store.delete_path(path)?;
        wait_for_event(&mut store, WatchOp::Delete, path, Duration::from_secs(5))?;

        handle.shutdown().await?;
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn etcd_store_write_reports_unreachable_endpoint() -> TestResult {
        let mut store =
            EtcdDcsStore::connect(vec!["http://127.0.0.1:1".to_string()], "scope-a")?;
        match store.write_path("/scope-a/member/node-a", "{}".to_string()) {
            Ok(_) => Err(boxed_error(
                "expected write against unreachable endpoint to fail",
            )),
            Err(DcsStoreError::Io(_)) => Ok(()),
            Err(other) => Err(boxed_error(format!(
                "expected io error for unreachable endpoint write, got {other}"
            ))),
        }
    }

    #[test]
    fn harness_error_type_is_used() {
        let _: Option<HarnessError> = None;
    }
}
