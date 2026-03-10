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
    bootstrap_path, cluster_identity_path, cluster_initialized_path, encode_leader_record,
    leader_path, DcsLeaderStore, DcsStore, DcsStoreError, WatchEvent, WatchOp,
};
use crate::config::DcsEndpoint;
use crate::{
    dcs::state::{BootstrapLockRecord, ClusterIdentityRecord, ClusterInitializedRecord},
    state::MemberId,
};

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

    #[cfg(test)]
    fn connect_with_worker_bootstrap_timeout(
        endpoints: Vec<DcsEndpoint>,
        scope: &str,
        worker_bootstrap_timeout: Duration,
    ) -> Result<Self, DcsStoreError> {
        Self::connect_with_options(endpoints, scope, worker_bootstrap_timeout, None)
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
    let (path, encoded) = encode_leader_record(scope, member_id)?;

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

fn worker_command_label(command: &WorkerCommand) -> &'static str {
    match command {
        WorkerCommand::Read { .. } => "read",
        WorkerCommand::Write { .. } => "write",
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

    fn acquire_bootstrap_lease(
        &mut self,
        scope: &str,
        member_id: &MemberId,
    ) -> Result<(), DcsStoreError> {
        let path = bootstrap_path(scope);
        let encoded = serde_json::to_string(&BootstrapLockRecord {
            holder: member_id.clone(),
        })
        .map_err(|err| DcsStoreError::Decode {
            key: path.clone(),
            message: err.to_string(),
        })?;
        if self.put_path_if_absent(path.as_str(), encoded)? {
            Ok(())
        } else {
            Err(DcsStoreError::AlreadyExists(path))
        }
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

    fn release_bootstrap_lease(
        &mut self,
        scope: &str,
        _member_id: &MemberId,
    ) -> Result<(), DcsStoreError> {
        self.delete_path(bootstrap_path(scope).as_str())
    }

    fn write_cluster_initialized(
        &mut self,
        scope: &str,
        record: &ClusterInitializedRecord,
    ) -> Result<(), DcsStoreError> {
        let path = cluster_initialized_path(scope);
        let encoded = serde_json::to_string(record).map_err(|err| DcsStoreError::Decode {
            key: path.clone(),
            message: err.to_string(),
        })?;
        self.write_path(path.as_str(), encoded)
    }

    fn write_cluster_identity(
        &mut self,
        scope: &str,
        record: &ClusterIdentityRecord,
    ) -> Result<(), DcsStoreError> {
        let path = cluster_identity_path(scope);
        let encoded = serde_json::to_string(record).map_err(|err| DcsStoreError::Decode {
            key: path.clone(),
            message: err.to_string(),
        })?;
        self.write_path(path.as_str(), encoded)
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
                evaluate_trust, BootstrapLockRecord, DcsView, DcsState, DcsTrust, DcsWorkerCtx,
                LeaderRecord, MemberRecord, MemberRole, SwitchoverRequest,
            },
            store::{
                refresh_from_etcd_watch, DcsLeaderStore, DcsStore, DcsStoreError, WatchEvent,
                WatchOp,
            },
            worker::step_once,
        },
        ha::{
            decide::decide,
            state::{
                ClusterMode, DecideInput, DesiredNodeState, HaState, LeadershipTransferState,
                QuiescentReason, WorldSnapshot,
            },
        },
        pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
        process::state::ProcessState,
        state::{new_state_channel, MemberId, UnixMillis, Version, WorkerError, WorkerStatus},
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

    fn sample_cache(scope: &str) -> DcsView {
        DcsView {
            members: BTreeMap::new(),
            leader: None,
            switchover: None,
            config: sample_runtime_config(scope),
            cluster_initialized: None,
            cluster_identity: None,
            bootstrap_lock: None,
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
                local_api_url: Some("http://127.0.0.1:8080".to_string()),
                local_data_dir: PathBuf::from("/tmp/pgtm/data"),
                local_postgres_binary: PathBuf::from("/usr/bin/postgres"),
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
                    api_url: None,
                    role: MemberRole::Primary,
                    sql: SqlStatus::Healthy,
                    readiness: Readiness::Ready,
                    timeline: None,
                    write_lsn: None,
                    replay_lsn: None,
            system_identifier: None,
            durable_end_lsn: None,
            state_class: None,
            postgres_runtime_class: None,
                    updated_at: UnixMillis(1),
                    pg_version: crate::state::Version(1),
                },
            );
            cache.switchover = Some(SwitchoverRequest {
                switchover_to: None,
            });
            cache.bootstrap_lock = Some(BootstrapLockRecord {
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
            if cache.bootstrap_lock.is_some() {
                return Err(boxed_error(
                    "expected bootstrap lock record to be cleared by reconnect reset",
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

            let seeded =
                store_a.put_path_if_absent(path_config.as_str(), "config-a".to_string())?;
            if !seeded {
                return Err(boxed_error("expected config seed to succeed on first write"));
            }
            let seeded_again =
                store_b.put_path_if_absent(path_config.as_str(), "config-b".to_string())?;
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
            if value != "config-a" {
                return Err(boxed_error(format!(
                    "expected config to remain 'config-a', got: {value:?}"
                )));
            }

            Ok(())
        }
        .await;

        shutdown_with_result(fixture, result).await
    }

    #[tokio::test(flavor = "current_thread")]
    async fn etcd_store_leader_lease_expires_after_owner_stops_renewing() -> TestResult {
        let fixture =
            RealEtcdFixture::spawn("dcs-etcd-store-leader-lease-expiry", "scope-lease-expiry")
                .await?;

        let fixture = fixture;
        let result: TestResult = async {
            let leader_key = format!("/{}/leader", fixture.scope);
            let owner_member = MemberId("node-a".to_string());
            let mut observer =
                EtcdDcsStore::connect(vec![fixture.endpoint_model()?], &fixture.scope)?;

            {
                let mut owner = EtcdDcsStore::connect_with_leader_lease(
                    vec![fixture.endpoint_model()?],
                    &fixture.scope,
                    1_000,
                )?;
                owner.acquire_leader_lease(&fixture.scope, &owner_member)?;
                wait_for_event(
                    &mut observer,
                    WatchOp::Put,
                    leader_key.as_str(),
                    Duration::from_secs(5),
                )?;
            }

            wait_for_event(
                &mut observer,
                WatchOp::Delete,
                leader_key.as_str(),
                Duration::from_secs(5),
            )?;

            let mut client = Client::connect(vec![fixture.endpoint.clone()], None)
                .await
                .map_err(|err| boxed_error(format!("etcd client connect failed: {err}")))?;
            let response = client
                .get(leader_key.as_str(), None)
                .await
                .map_err(|err| boxed_error(format!("etcd get leader failed: {err}")))?;
            if !response.kvs().is_empty() {
                return Err(boxed_error(
                    "expected leader key to expire after keepalive stopped",
                ));
            }

            Ok(())
        }
        .await;

        shutdown_with_result(fixture, result).await
    }

    #[tokio::test(flavor = "current_thread")]
    async fn etcd_store_releases_only_locally_owned_leader_lease() -> TestResult {
        let fixture = RealEtcdFixture::spawn(
            "dcs-etcd-store-owner-only-leader-release",
            "scope-owner-release",
        )
        .await?;

        let fixture = fixture;
        let result: TestResult = async {
            let leader_key = format!("/{}/leader", fixture.scope);
            let owner_member = MemberId("node-a".to_string());
            let other_member = MemberId("node-b".to_string());
            let mut observer =
                EtcdDcsStore::connect(vec![fixture.endpoint_model()?], &fixture.scope)?;
            let mut owner = EtcdDcsStore::connect_with_leader_lease(
                vec![fixture.endpoint_model()?],
                &fixture.scope,
                3_000,
            )?;
            let mut other = EtcdDcsStore::connect_with_leader_lease(
                vec![fixture.endpoint_model()?],
                &fixture.scope,
                3_000,
            )?;

            owner.acquire_leader_lease(&fixture.scope, &owner_member)?;
            wait_for_event(
                &mut observer,
                WatchOp::Put,
                leader_key.as_str(),
                Duration::from_secs(5),
            )?;

            other.release_leader_lease(&fixture.scope, &other_member)?;

            let leader_after_foreign_release = other.read_path(leader_key.as_str())?;
            if leader_after_foreign_release.is_none() {
                return Err(boxed_error(
                    "expected foreign release attempt to leave owner leader key intact",
                ));
            }

            owner.release_leader_lease(&fixture.scope, &owner_member)?;
            wait_for_event(
                &mut observer,
                WatchOp::Delete,
                leader_key.as_str(),
                Duration::from_secs(5),
            )?;

            Ok(())
        }
        .await;

        shutdown_with_result(fixture, result).await
    }

    #[tokio::test(flavor = "current_thread")]
    async fn leader_expiry_flows_through_watch_cache_trust_and_desired_state() -> TestResult {
        let fixture = RealEtcdFixture::spawn(
            "dcs-etcd-store-leader-expiry-ha-chain",
            "scope-leader-expiry-ha-chain",
        )
        .await?;

        let fixture = fixture;
        let result: TestResult = async {
            let leader_key = format!("/{}/leader", fixture.scope);
            let owner_member = MemberId("node-a".to_string());
            let mut observer = EtcdDcsStore::connect(vec![fixture.endpoint_model()?], &fixture.scope)?;
            let mut writer = EtcdDcsStore::connect(vec![fixture.endpoint_model()?], &fixture.scope)?;

            for record in [
                MemberRecord {
                    member_id: MemberId("node-a".to_string()),
                    postgres_host: "127.0.0.1".to_string(),
                    postgres_port: 5432,
                    api_url: None,
                    role: MemberRole::Primary,
                    sql: SqlStatus::Healthy,
                    readiness: Readiness::Ready,
                    timeline: None,
                    write_lsn: None,
                    replay_lsn: None,
            system_identifier: None,
            durable_end_lsn: None,
            state_class: None,
            postgres_runtime_class: None,
                    updated_at: UnixMillis(1),
                    pg_version: Version(1),
                },
                MemberRecord {
                    member_id: MemberId("node-b".to_string()),
                    postgres_host: "127.0.0.2".to_string(),
                    postgres_port: 5432,
                    api_url: None,
                    role: MemberRole::Primary,
                    sql: SqlStatus::Healthy,
                    readiness: Readiness::Ready,
                    timeline: None,
                    write_lsn: None,
                    replay_lsn: None,
            system_identifier: None,
            durable_end_lsn: None,
            state_class: None,
            postgres_runtime_class: None,
                    updated_at: UnixMillis(15_000),
                    pg_version: Version(1),
                },
                MemberRecord {
                    member_id: MemberId("node-c".to_string()),
                    postgres_host: "127.0.0.3".to_string(),
                    postgres_port: 5432,
                    api_url: None,
                    role: MemberRole::Replica,
                    sql: SqlStatus::Healthy,
                    readiness: Readiness::Ready,
                    timeline: None,
                    write_lsn: None,
                    replay_lsn: None,
            system_identifier: None,
            durable_end_lsn: None,
            state_class: None,
            postgres_runtime_class: None,
                    updated_at: UnixMillis(15_000),
                    pg_version: Version(1),
                },
            ] {
                let encoded = serde_json::to_string(&record)
                    .map_err(|err| boxed_error(format!("encode member record failed: {err}")))?;
                writer.write_path(
                    format!("/{}/member/{}", fixture.scope, record.member_id.0).as_str(),
                    encoded,
                )?;
            }

            {
                let mut owner = EtcdDcsStore::connect_with_leader_lease(
                    vec![fixture.endpoint_model()?],
                    &fixture.scope,
                    1_000,
                )?;
                owner.acquire_leader_lease(&fixture.scope, &owner_member)?;
                wait_for_event(
                    &mut observer,
                    WatchOp::Put,
                    leader_key.as_str(),
                    Duration::from_secs(5),
                )?;
            }

            wait_for_event(
                &mut observer,
                WatchOp::Delete,
                leader_key.as_str(),
                Duration::from_secs(5),
            )?;

            let mut snapshot_store =
                EtcdDcsStore::connect(vec![fixture.endpoint_model()?], &fixture.scope)?;
            let mut cache = DcsView {
                members: BTreeMap::new(),
                leader: None,
                switchover: None,
                config: sample_runtime_config(&fixture.scope),
                cluster_initialized: None,
            cluster_identity: None,
            bootstrap_lock: None,
            };
            let events = snapshot_store.drain_watch_events()?;
            refresh_from_etcd_watch(&fixture.scope, &mut cache, events)?;

            if cache.leader.is_some() {
                return Err(boxed_error(
                    "expected DCS-visible cache to drop leader record after lease expiry",
                ));
            }

            let now = UnixMillis(20_000);
            let trust = evaluate_trust(true, &cache, &MemberId("node-b".to_string()), now);
            if trust != DcsTrust::FreshQuorum {
                return Err(boxed_error(format!(
                    "expected healthy majority to remain full quorum after leader expiry, got {trust:?}"
                )));
            }

            let mut world_config = cache.config.clone();
            world_config.cluster.member_id = "node-b".to_string();
            let world = WorldSnapshot {
                config: crate::state::Versioned::new(Version(1), now, world_config.clone()),
                pg: crate::state::Versioned::new(
                    Version(1),
                    now,
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
                            last_refresh_at: Some(now),
                        },
                        wal_lsn: crate::state::WalLsn(10),
                        slots: Vec::new(),
                    },
                ),
                dcs: crate::state::Versioned::new(
                    Version(1),
                    now,
                    DcsState {
                        worker: WorkerStatus::Running,
                        trust,
                        cache,
                        last_refresh_at: Some(now),
                    },
                ),
                process: crate::state::Versioned::new(
                    Version(1),
                    now,
                    ProcessState::Idle {
                        worker: WorkerStatus::Running,
                        last_outcome: None,
                    },
                ),
            };

            let decision = decide(DecideInput {
                current: HaState {
                    worker: WorkerStatus::Running,
                    cluster_mode: ClusterMode::InitializedNoLeaderFreshQuorum,
                    desired_state: DesiredNodeState::Quiescent {
                        reason: QuiescentReason::WaitingForAuthoritativeLeader,
                    },
                    leadership_transfer: LeadershipTransferState::None,
                    tick: 3,
                },
                world,
            });

            if !matches!(
                decision.next.desired_state,
                DesiredNodeState::Primary { .. }
            ) {
                return Err(boxed_error(format!(
                    "expected lease-expiry chain to advance into a primary acquisition plan, got {:?}",
                    decision.next.desired_state
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
            vec![crate::config::DcsEndpoint::from_socket_addr(
                std::net::SocketAddr::from(([127, 0, 0, 1], 1)),
            )],
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
