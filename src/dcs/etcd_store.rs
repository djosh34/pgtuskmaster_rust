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
    Client, Compare, CompareOp, EventType, GetOptions, PutOptions, Txn, TxnOp, WatchOptions,
    WatchResponse, WatchStream, Watcher,
};
use tokio::sync::mpsc as tokio_mpsc;

use super::store::{
    encode_leader_record, leader_path, DcsLeaderStore, DcsStore, DcsStoreError, WatchEvent, WatchOp,
};
use crate::config::DcsEndpoint;
use crate::state::MemberId;

const COMMAND_TIMEOUT: Duration = Duration::from_secs(2);
const WORKER_BOOTSTRAP_TIMEOUT: Duration = Duration::from_secs(8);
const WATCH_IDLE_INTERVAL: Duration = Duration::from_millis(100);
const MIN_LEADER_LEASE_TTL_SECONDS: u64 = 1;

#[derive(Clone, Copy, Debug)]
struct LeaderLeaseConfig {
    ttl_seconds: i64,
}

#[derive(Debug)]
struct OwnedLeaderLease {
    lease_id: i64,
    leader_path: String,
    member_id: MemberId,
    stop_tx: mpsc::Sender<()>,
    failure_rx: mpsc::Receiver<DcsStoreError>,
    keepalive_handle: JoinHandle<()>,
}

enum WorkerCommand {
    Read {
        path: String,
        response_tx: mpsc::Sender<Result<Option<String>, DcsStoreError>>,
    },
    SnapshotPrefix {
        path_prefix: String,
        response_tx: mpsc::Sender<Result<Vec<WatchEvent>, DcsStoreError>>,
    },
    Write {
        path: String,
        value: String,
        response_tx: mpsc::Sender<Result<(), DcsStoreError>>,
    },
    WriteWithLease {
        path: String,
        value: String,
        lease_ttl_ms: u64,
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
    AcquireLeaderLease {
        scope: String,
        member_id: MemberId,
        response_tx: mpsc::Sender<Result<(), DcsStoreError>>,
    },
    ReleaseLeaderLease {
        scope: String,
        member_id: MemberId,
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
        Self::connect_with_options(endpoints, scope, WORKER_BOOTSTRAP_TIMEOUT, None)
    }

    pub(crate) fn connect_with_leader_lease(
        endpoints: Vec<DcsEndpoint>,
        scope: &str,
        lease_ttl_ms: u64,
    ) -> Result<Self, DcsStoreError> {
        let leader_lease = Some(leader_lease_config_from_ttl_ms(lease_ttl_ms)?);
        Self::connect_with_options(endpoints, scope, WORKER_BOOTSTRAP_TIMEOUT, leader_lease)
    }

    fn connect_with_options(
        endpoints: Vec<DcsEndpoint>,
        scope: &str,
        worker_bootstrap_timeout: Duration,
        leader_lease_config: Option<LeaderLeaseConfig>,
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
                    leader_lease_config,
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

    fn request_unit_command(
        &mut self,
        command: WorkerCommand,
        operation: &str,
    ) -> Result<(), DcsStoreError> {
        let (response_tx, response_rx) = mpsc::channel::<Result<(), DcsStoreError>>();
        let command = match command {
            WorkerCommand::AcquireLeaderLease {
                scope, member_id, ..
            } => WorkerCommand::AcquireLeaderLease {
                scope,
                member_id,
                response_tx,
            },
            WorkerCommand::ReleaseLeaderLease {
                scope, member_id, ..
            } => WorkerCommand::ReleaseLeaderLease {
                scope,
                member_id,
                response_tx,
            },
            other => {
                return Err(DcsStoreError::Io(format!(
                    "unexpected worker command for unit request: {other_label}",
                    other_label = worker_command_label(&other)
                )));
            }
        };

        self.command_tx.send(command).map_err(|err| {
            self.mark_unhealthy();
            DcsStoreError::Io(format!("send {operation} command failed: {err}"))
        })?;

        response_rx.recv_timeout(COMMAND_TIMEOUT).map_err(|err| {
            self.mark_unhealthy();
            DcsStoreError::Io(format!("timed out waiting for {operation} command: {err}"))
        })?
    }
}

fn run_worker_loop(
    endpoints: Vec<DcsEndpoint>,
    scope_prefix: String,
    leader_lease_config: Option<LeaderLeaseConfig>,
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
        let mut owned_leader = None;

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
            if poll_owned_leader(&healthy, &mut owned_leader).is_err() {
                let _ = stop_owned_leader(&mut owned_leader);
            }

            if client.is_none() || watch_stream.is_none() {
                tokio::select! {
                    maybe_command = command_rx.recv() => {
                        let Some(command) = maybe_command else {
                            let _ = stop_owned_leader(&mut owned_leader);
                            return;
                        };
                        let mut command_ctx = WorkerCommandCtx {
                            endpoints: &endpoints,
                            leader_lease_config: &leader_lease_config,
                            healthy: &healthy,
                            events: &events,
                            client: &mut client,
                            watcher: &mut _watcher,
                            watch_stream: &mut watch_stream,
                            owned_leader: &mut owned_leader,
                        };
                        if !command_ctx.handle(command).await {
                            let _ = stop_owned_leader(&mut owned_leader);
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
                        let _ = stop_owned_leader(&mut owned_leader);
                        return;
                    };
                    let mut command_ctx = WorkerCommandCtx {
                        endpoints: &endpoints,
                        leader_lease_config: &leader_lease_config,
                        healthy: &healthy,
                        events: &events,
                        client: &mut client,
                        watcher: &mut _watcher,
                        watch_stream: &mut watch_stream,
                        owned_leader: &mut owned_leader,
                    };
                    if !command_ctx.handle(command).await {
                        let _ = stop_owned_leader(&mut owned_leader);
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

struct WorkerCommandCtx<'a> {
    endpoints: &'a [DcsEndpoint],
    leader_lease_config: &'a Option<LeaderLeaseConfig>,
    healthy: &'a Arc<AtomicBool>,
    events: &'a Arc<Mutex<VecDeque<WatchEvent>>>,
    client: &'a mut Option<Client>,
    watcher: &'a mut Option<Watcher>,
    watch_stream: &'a mut Option<WatchStream>,
    owned_leader: &'a mut Option<OwnedLeaderLease>,
}

impl WorkerCommandCtx<'_> {
    async fn handle(&mut self, command: WorkerCommand) -> bool {
        match command {
            WorkerCommand::Write {
                path,
                value,
                response_tx,
            } => {
                let result =
                    execute_write(self.endpoints, self.client, self.healthy, &path, value).await;
                let invalidate_result = if should_invalidate_on_error(&result) {
                    invalidate_watch_session(
                        self.healthy,
                        self.events,
                        self.client,
                        self.watcher,
                        self.watch_stream,
                    )
                } else {
                    Ok(())
                };
                let _ = response_tx.send(result);
                invalidate_result.is_ok()
            }
            WorkerCommand::WriteWithLease {
                path,
                value,
                lease_ttl_ms,
                response_tx,
            } => {
                let result = execute_write_with_lease(
                    self.endpoints,
                    self.client,
                    self.healthy,
                    &path,
                    value,
                    lease_ttl_ms,
                )
                .await;
                let invalidate_result = if should_invalidate_on_error(&result) {
                    invalidate_watch_session(
                        self.healthy,
                        self.events,
                        self.client,
                        self.watcher,
                        self.watch_stream,
                    )
                } else {
                    Ok(())
                };
                let _ = response_tx.send(result);
                invalidate_result.is_ok()
            }
            WorkerCommand::Read { path, response_tx } => {
                let result = execute_read(self.endpoints, self.client, self.healthy, &path).await;
                let invalidate_result = if should_invalidate_on_error(&result) {
                    invalidate_watch_session(
                        self.healthy,
                        self.events,
                        self.client,
                        self.watcher,
                        self.watch_stream,
                    )
                } else {
                    Ok(())
                };
                let _ = response_tx.send(result);
                invalidate_result.is_ok()
            }
            WorkerCommand::SnapshotPrefix {
                path_prefix,
                response_tx,
            } => {
                let result = execute_snapshot_prefix(
                    self.endpoints,
                    self.client,
                    self.healthy,
                    &path_prefix,
                )
                .await;
                let invalidate_result = if should_invalidate_on_error(&result) {
                    invalidate_watch_session(
                        self.healthy,
                        self.events,
                        self.client,
                        self.watcher,
                        self.watch_stream,
                    )
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
                let result =
                    execute_put_if_absent(self.endpoints, self.client, self.healthy, &path, value)
                        .await;
                let invalidate_result = if should_invalidate_on_error(&result) {
                    invalidate_watch_session(
                        self.healthy,
                        self.events,
                        self.client,
                        self.watcher,
                        self.watch_stream,
                    )
                } else {
                    Ok(())
                };
                let _ = response_tx.send(result);
                invalidate_result.is_ok()
            }
            WorkerCommand::Delete { path, response_tx } => {
                let result = execute_delete(self.endpoints, self.client, self.healthy, &path).await;
                let invalidate_result = if should_invalidate_on_error(&result) {
                    invalidate_watch_session(
                        self.healthy,
                        self.events,
                        self.client,
                        self.watcher,
                        self.watch_stream,
                    )
                } else {
                    Ok(())
                };
                let _ = response_tx.send(result);
                invalidate_result.is_ok()
            }
            WorkerCommand::AcquireLeaderLease {
                scope,
                member_id,
                response_tx,
            } => {
                let result = execute_acquire_leader_lease(
                    self.endpoints,
                    self.leader_lease_config,
                    self.healthy,
                    &scope,
                    &member_id,
                    self.owned_leader,
                )
                .await;
                let _ = response_tx.send(result);
                true
            }
            WorkerCommand::ReleaseLeaderLease {
                scope,
                member_id,
                response_tx,
            } => {
                let result = execute_release_leader_lease(
                    self.endpoints,
                    self.healthy,
                    &scope,
                    &member_id,
                    self.owned_leader,
                )
                .await;
                let _ = response_tx.send(result);
                true
            }
            WorkerCommand::Shutdown => {
                let _ = stop_owned_leader(self.owned_leader);
                false
            }
        }
    }
}

fn should_invalidate_on_error<T>(result: &Result<T, DcsStoreError>) -> bool {
    matches!(result, Err(DcsStoreError::Io(_)))
}

fn poll_owned_leader(
    healthy: &Arc<AtomicBool>,
    owned_leader: &mut Option<OwnedLeaderLease>,
) -> Result<(), DcsStoreError> {
    let failure = match owned_leader.as_mut() {
        Some(state) => match state.failure_rx.try_recv() {
            Ok(err) => Some(err),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => Some(DcsStoreError::Io(format!(
                "leader lease keepalive stopped unexpectedly for `{}`",
                state.leader_path
            ))),
        },
        None => None,
    };

    if let Some(err) = failure {
        healthy.store(false, Ordering::SeqCst);
        return Err(err);
    }

    Ok(())
}

fn stop_owned_leader(owned_leader: &mut Option<OwnedLeaderLease>) -> Result<(), DcsStoreError> {
    let Some(state) = owned_leader.take() else {
        return Ok(());
    };

    let _ = state.stop_tx.send(());
    match state.keepalive_handle.join() {
        Ok(()) => Ok(()),
        Err(_) => Err(DcsStoreError::Io(format!(
            "leader lease keepalive thread panicked for `{}`",
            state.leader_path
        ))),
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

fn leader_lease_config_from_ttl_ms(lease_ttl_ms: u64) -> Result<LeaderLeaseConfig, DcsStoreError> {
    let rounded_seconds = lease_ttl_ms.saturating_add(999) / 1000;
    let clamped_seconds = rounded_seconds.max(MIN_LEADER_LEASE_TTL_SECONDS);
    let ttl_seconds = i64::try_from(clamped_seconds).map_err(|_| {
        DcsStoreError::Io(format!(
            "leader lease ttl `{lease_ttl_ms}`ms is too large to convert to etcd seconds"
        ))
    })?;

    Ok(LeaderLeaseConfig { ttl_seconds })
}

fn leader_keepalive_interval(ttl_seconds: i64) -> Duration {
    if ttl_seconds <= 1 {
        return Duration::from_millis(500);
    }

    let ttl_seconds = ttl_seconds as u64;
    Duration::from_secs(std::cmp::max(1, ttl_seconds / 3))
}

fn spawn_leader_keepalive(
    endpoints: &[DcsEndpoint],
    lease_id: i64,
    leader_path_value: &str,
    member_id: &MemberId,
    ttl_seconds: i64,
) -> Result<OwnedLeaderLease, DcsStoreError> {
    let (stop_tx, stop_rx) = mpsc::channel::<()>();
    let (failure_tx, failure_rx) = mpsc::channel::<DcsStoreError>();
    let keepalive_interval = leader_keepalive_interval(ttl_seconds);
    let keepalive_endpoints = endpoints.to_vec();
    let keepalive_handle = thread::Builder::new()
        .name("etcd-leader-keepalive".to_string())
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();

            let result = match runtime {
                Ok(runtime) => run_leader_keepalive(
                    &runtime,
                    &keepalive_endpoints,
                    lease_id,
                    keepalive_interval,
                    stop_rx,
                ),
                Err(err) => Err(DcsStoreError::Io(format!(
                    "failed to build leader keepalive runtime: {err}"
                ))),
            };

            if let Err(err) = result {
                let _ = failure_tx.send(err);
            }
        })
        .map_err(|err| DcsStoreError::Io(format!("spawn leader keepalive failed: {err}")))?;

    Ok(OwnedLeaderLease {
        lease_id,
        leader_path: leader_path_value.to_string(),
        member_id: member_id.clone(),
        stop_tx,
        failure_rx,
        keepalive_handle,
    })
}

fn run_leader_keepalive(
    runtime: &tokio::runtime::Runtime,
    endpoints: &[DcsEndpoint],
    lease_id: i64,
    keepalive_interval: Duration,
    stop_rx: mpsc::Receiver<()>,
) -> Result<(), DcsStoreError> {
    let mut client = runtime.block_on(connect_client(endpoints))?;
    let (mut keeper, mut stream) = runtime.block_on(timeout_etcd(
        "etcd lease keepalive create",
        client.lease_keep_alive(lease_id),
    ))?;

    loop {
        match stop_rx.recv_timeout(keepalive_interval) {
            Ok(()) => return Ok(()),
            Err(mpsc::RecvTimeoutError::Timeout) => {
                runtime.block_on(timeout_etcd(
                    "etcd lease keepalive send",
                    keeper.keep_alive(),
                ))?;
                let response = runtime.block_on(timeout_etcd(
                    "etcd lease keepalive receive",
                    stream.message(),
                ))?;
                match response {
                    Some(message) if message.ttl() > 0 => {}
                    Some(_) => {
                        return Err(DcsStoreError::Io(format!(
                            "leader lease keepalive reported expired lease `{lease_id}`"
                        )));
                    }
                    None => {
                        return Err(DcsStoreError::Io(format!(
                            "leader lease keepalive stream closed for lease `{lease_id}`"
                        )));
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => return Ok(()),
        }
    }
}

async fn execute_acquire_leader_lease(
    endpoints: &[DcsEndpoint],
    leader_lease_config: &Option<LeaderLeaseConfig>,
    healthy: &Arc<AtomicBool>,
    scope: &str,
    member_id: &MemberId,
    owned_leader: &mut Option<OwnedLeaderLease>,
) -> Result<(), DcsStoreError> {
    let generation = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis() as u64);
    let (path, encoded) = encode_leader_record(scope, member_id, generation)?;

    if owned_leader
        .as_ref()
        .map(|state| state.leader_path == path && state.member_id == *member_id)
        .unwrap_or(false)
    {
        return Ok(());
    }

    let Some(lease_config) = leader_lease_config else {
        return Err(DcsStoreError::LeaderLeaseNotConfigured(path));
    };

    let mut client = connect_client(endpoints).await?;
    let lease_response = timeout_etcd(
        "etcd lease grant",
        client.lease_grant(lease_config.ttl_seconds, None),
    )
    .await?;
    let lease_id = lease_response.id();
    let compare = Compare::version(path.as_str(), CompareOp::Equal, 0);
    let put = TxnOp::put(
        path.as_str(),
        encoded,
        Some(PutOptions::new().with_lease(lease_id)),
    );
    let txn = Txn::new().when(vec![compare]).and_then(vec![put]);
    let txn_result = timeout_etcd("etcd leader lease txn", client.txn(txn)).await;

    match txn_result {
        Ok(response) if response.succeeded() => {
            stop_owned_leader(owned_leader)?;
            let owned = spawn_leader_keepalive(
                endpoints,
                lease_id,
                path.as_str(),
                member_id,
                lease_config.ttl_seconds,
            )?;
            *owned_leader = Some(owned);
            healthy.store(true, Ordering::SeqCst);
            Ok(())
        }
        Ok(_) => {
            let _ = timeout_etcd("etcd lease revoke", client.lease_revoke(lease_id)).await;
            Err(DcsStoreError::AlreadyExists(path))
        }
        Err(err) => {
            let _ = timeout_etcd("etcd lease revoke", client.lease_revoke(lease_id)).await;
            healthy.store(false, Ordering::SeqCst);
            Err(err)
        }
    }
}

async fn execute_release_leader_lease(
    endpoints: &[DcsEndpoint],
    healthy: &Arc<AtomicBool>,
    scope: &str,
    member_id: &MemberId,
    owned_leader: &mut Option<OwnedLeaderLease>,
) -> Result<(), DcsStoreError> {
    let path = leader_path(scope);
    let should_release = owned_leader
        .as_ref()
        .map(|state| state.leader_path == path && state.member_id == *member_id)
        .unwrap_or(false);

    if !should_release {
        return Ok(());
    }

    let lease_id = owned_leader
        .as_ref()
        .map(|state| state.lease_id)
        .unwrap_or_default();
    stop_owned_leader(owned_leader)?;

    let mut client = connect_client(endpoints).await?;
    match timeout_etcd("etcd lease revoke", client.lease_revoke(lease_id)).await {
        Ok(_) => {
            healthy.store(true, Ordering::SeqCst);
            Ok(())
        }
        Err(err) => {
            healthy.store(false, Ordering::SeqCst);
            Err(err)
        }
    }
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

async fn execute_write_with_lease(
    endpoints: &[DcsEndpoint],
    client: &mut Option<Client>,
    healthy: &Arc<AtomicBool>,
    path: &str,
    value: String,
    lease_ttl_ms: u64,
) -> Result<(), DcsStoreError> {
    if client.is_none() {
        *client = Some(connect_client(endpoints).await?);
    }

    let Some(active_client) = client.as_mut() else {
        healthy.store(false, Ordering::SeqCst);
        return Err(DcsStoreError::Io(
            "etcd client unavailable for leased write".to_string(),
        ));
    };

    let lease_config = leader_lease_config_from_ttl_ms(lease_ttl_ms)?;
    let lease_response = timeout_etcd(
        "etcd lease grant",
        active_client.lease_grant(lease_config.ttl_seconds, None),
    )
    .await?;
    let options = PutOptions::new().with_lease(lease_response.id());
    match timeout_etcd("etcd put", active_client.put(path, value, Some(options))).await {
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

async fn execute_snapshot_prefix(
    endpoints: &[DcsEndpoint],
    client: &mut Option<Client>,
    healthy: &Arc<AtomicBool>,
    path_prefix: &str,
) -> Result<Vec<WatchEvent>, DcsStoreError> {
    if client.is_none() {
        *client = Some(connect_client(endpoints).await?);
    }

    let Some(active_client) = client.as_mut() else {
        healthy.store(false, Ordering::SeqCst);
        return Err(DcsStoreError::Io(
            "etcd client unavailable for prefix snapshot".to_string(),
        ));
    };

    match timeout_etcd(
        "etcd get",
        active_client.get(path_prefix, Some(GetOptions::new().with_prefix())),
    )
    .await
    {
        Ok(response) => {
            healthy.store(true, Ordering::SeqCst);
            let revision = response
                .header()
                .map(|header| header.revision())
                .unwrap_or(0);
            let mut events = vec![WatchEvent {
                op: WatchOp::Reset,
                path: path_prefix.to_string(),
                value: None,
                revision,
            }];
            for kv in response.kvs() {
                let path = str::from_utf8(kv.key()).map_err(|err| DcsStoreError::Decode {
                    key: "watch-key".to_string(),
                    message: err.to_string(),
                })?;
                let value = str::from_utf8(kv.value()).map_err(|err| DcsStoreError::Decode {
                    key: path.to_string(),
                    message: err.to_string(),
                })?;
                events.push(WatchEvent {
                    op: WatchOp::Put,
                    path: path.to_string(),
                    value: Some(value.to_string()),
                    revision: kv.mod_revision(),
                });
            }
            Ok(events)
        }
        Err(err) => {
            healthy.store(false, Ordering::SeqCst);
            *client = None;
            Err(err)
        }
    }
}

fn worker_command_label(command: &WorkerCommand) -> &'static str {
    match command {
        WorkerCommand::Read { .. } => "read",
        WorkerCommand::SnapshotPrefix { .. } => "snapshot_prefix",
        WorkerCommand::Write { .. } => "write",
        WorkerCommand::WriteWithLease { .. } => "write_with_lease",
        WorkerCommand::PutIfAbsent { .. } => "put_if_absent",
        WorkerCommand::Delete { .. } => "delete",
        WorkerCommand::AcquireLeaderLease { .. } => "acquire_leader_lease",
        WorkerCommand::ReleaseLeaderLease { .. } => "release_leader_lease",
        WorkerCommand::Shutdown => "shutdown",
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

    fn snapshot_prefix(&mut self, path_prefix: &str) -> Result<Vec<WatchEvent>, DcsStoreError> {
        let (response_tx, response_rx) = mpsc::channel();
        self.command_tx
            .send(WorkerCommand::SnapshotPrefix {
                path_prefix: path_prefix.to_string(),
                response_tx,
            })
            .map_err(|err| {
                self.mark_unhealthy();
                DcsStoreError::Io(format!("send snapshot-prefix command failed: {err}"))
            })?;

        response_rx.recv_timeout(COMMAND_TIMEOUT).map_err(|err| {
            self.mark_unhealthy();
            DcsStoreError::Io(format!(
                "timed out waiting for snapshot-prefix command: {err}"
            ))
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

    fn write_path_with_lease(
        &mut self,
        path: &str,
        value: String,
        lease_ttl_ms: u64,
    ) -> Result<(), DcsStoreError> {
        let (response_tx, response_rx) = mpsc::channel();
        self.command_tx
            .send(WorkerCommand::WriteWithLease {
                path: path.to_string(),
                value,
                lease_ttl_ms,
                response_tx,
            })
            .map_err(|err| {
                self.mark_unhealthy();
                DcsStoreError::Io(format!("send leased-write command failed: {err}"))
            })?;

        response_rx.recv_timeout(COMMAND_TIMEOUT).map_err(|err| {
            self.mark_unhealthy();
            DcsStoreError::Io(format!("timed out waiting for leased-write command: {err}"))
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

impl DcsLeaderStore for EtcdDcsStore {
    fn acquire_leader_lease(
        &mut self,
        scope: &str,
        member_id: &MemberId,
    ) -> Result<(), DcsStoreError> {
        self.request_unit_command(
            WorkerCommand::AcquireLeaderLease {
                scope: scope.to_string(),
                member_id: member_id.clone(),
                response_tx: mpsc::channel::<Result<(), DcsStoreError>>().0,
            },
            "acquire leader lease",
        )
    }

    fn release_leader_lease(
        &mut self,
        scope: &str,
        member_id: &MemberId,
    ) -> Result<(), DcsStoreError> {
        self.request_unit_command(
            WorkerCommand::ReleaseLeaderLease {
                scope: scope.to_string(),
                member_id: member_id.clone(),
                response_tx: mpsc::channel::<Result<(), DcsStoreError>>().0,
            },
            "release leader lease",
        )
    }

    fn clear_switchover(&mut self, scope: &str) -> Result<(), DcsStoreError> {
        self.delete_path(format!("/{}/switchover", scope.trim_matches('/')).as_str())
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
