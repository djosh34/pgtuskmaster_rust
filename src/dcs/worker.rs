use std::{collections::BTreeMap, str, time::Duration};

use etcd_client::{
    Certificate, Client, Compare, CompareOp, ConnectOptions, EventType, GetOptions, Identity,
    LeaseKeepAliveStream, LeaseKeeper, PutOptions, TlsOptions, Txn, TxnOp, WatchOptions,
    WatchResponse, WatchStream, Watcher,
};
use serde::Serialize;
use thiserror::Error;
use tokio::time::{Instant, MissedTickBehavior};

use crate::{
    config_v2::{
        types::{DcsEndpoint, TlsConfig},
        RuntimeConfigV2,
    },
    logging::LogSender,
    pginfo::state::PgInfoState,
    state::{
        new_state_channel, ApiRoute, LeaseEpoch, MemberId, PgRoute, StatePublisher,
        StateSubscriber, SwitchoverState, WorkerError,
    },
};

use super::{
    command::{DcsCommand, DcsCommandInbox},
    log_event::DcsLogEvent,
    state::{
        build_local_member_state, current_snapshot, DcsAuthority, DcsMemberState, DcsSnapshot,
    },
};

const ETCD_TIMEOUT: Duration = Duration::from_secs(2);
const RECONNECT_BACKOFF: Duration = Duration::from_secs(1);
const MIN_LEADER_LEASE_TTL_SECONDS: u64 = 1;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(crate) enum DcsError {
    #[error("path already exists: {0}")]
    AlreadyExists(String),
    #[error("decode failed for key `{key}`: {message}")]
    Decode { key: String, message: String },
    #[error("store I/O error: {0}")]
    Io(String),
    #[error("leader lease expired: {0}")]
    LeaderLeaseExpired(String),
}

pub(crate) struct DcsWorker<'a> {
    cfg: &'a RuntimeConfigV2,
    keys: DcsKeySpace,
    pg: StateSubscriber<PgInfoState>,
    publisher: StatePublisher<DcsSnapshot>,
    command_inbox: DcsCommandInbox,
    log: LogSender,
    cluster: DcsClusterState,
    session: Option<ConnectedSession>,
}

struct DcsClusterState {
    members: BTreeMap<MemberId, DcsMemberState>,
    leadership: Option<LeaseEpoch>,
    switchover: SwitchoverState,
    last_emitted_authority: Option<DcsAuthority>,
}

struct ConnectedSession {
    client: Client,
    _watcher: Watcher,
    watch_stream: WatchStream,
    leader_lease: Option<OwnedLeaderLease>,
}

struct OwnedLeaderLease {
    lease_id: i64,
    leader_path: String,
    member_id: MemberId,
    ttl_seconds: i64,
    keeper: LeaseKeeper,
    stream: LeaseKeepAliveStream,
    next_keepalive_at: Instant,
}

#[derive(Clone, Copy)]
enum FailurePhase {
    InitialConnect,
    ConnectedStep,
}

struct DcsKeySpace {
    prefix: String,
}

enum DcsKey {
    Member(MemberId),
    Leader,
    Switchover,
}

enum DcsChange {
    Member(MemberId, Option<Box<DcsMemberState>>),
    Leader(Option<LeaseEpoch>),
    Switchover(Option<SwitchoverState>),
}

struct KeyValueWrite {
    path: String,
    value: String,
}

struct EtcdRuntime;

pub(crate) async fn run(worker: DcsWorker<'_>) -> Result<(), WorkerError> {
    worker.run().await
}

pub(crate) fn bootstrap<'a>(
    cfg: &'a RuntimeConfigV2,
    pg: StateSubscriber<PgInfoState>,
    log: LogSender,
) -> Result<
    (
        StateSubscriber<DcsSnapshot>,
        super::DcsHandle,
        DcsWorker<'a>,
    ),
    DcsError,
> {
    let (publisher, state) = new_state_channel(DcsSnapshot::starting());
    let (handle, command_inbox) = super::command::dcs_command_channel();
    let worker = DcsWorker::new(cfg, pg, publisher, command_inbox, log);
    Ok((state, handle, worker))
}

impl<'a> DcsWorker<'a> {
    pub(crate) fn new(
        cfg: &'a RuntimeConfigV2,
        pg: StateSubscriber<PgInfoState>,
        publisher: StatePublisher<DcsSnapshot>,
        command_inbox: DcsCommandInbox,
        log: LogSender,
    ) -> Self {
        Self {
            cfg,
            keys: DcsKeySpace::new(cfg.scope.as_str()),
            pg,
            publisher,
            command_inbox,
            log,
            cluster: DcsClusterState::new(),
            session: None,
        }
    }

    async fn run(mut self) -> Result<(), WorkerError> {
        let mut reconnect_at = Instant::now();
        let mut tick = tokio::time::interval(self.cfg.ha.loop_interval);
        tick.set_missed_tick_behavior(MissedTickBehavior::Delay);
        self.publish_current_view(false)?;

        loop {
            if self.session.is_some() {
                enum ConnectedStep {
                    Tick,
                    PgChanged,
                    Command(DcsCommand),
                    Watch(Result<Option<WatchResponse>, DcsError>),
                    KeepAlive,
                    Disconnected,
                }

                let keepalive_deadline = self.session.as_ref().and_then(|session| {
                    session
                        .leader_lease
                        .as_ref()
                        .map(|lease| lease.next_keepalive_at)
                });
                let step = {
                    let pg = &mut self.pg;
                    let command_inbox = &mut self.command_inbox;
                    let Some(session) = self.session.as_mut() else {
                        return Err(WorkerError::Message(
                            "dcs session disappeared during connected step".to_string(),
                        ));
                    };
                    let watch_stream = &mut session.watch_stream;
                    tokio::select! {
                        _ = tick.tick() => ConnectedStep::Tick,
                        changed = pg.changed() => {
                            changed.map_err(|err| WorkerError::Message(format!("dcs pg subscriber closed: {err}")))?;
                            ConnectedStep::PgChanged
                        }
                        command = command_inbox.recv() => {
                            match command {
                                Some(command) => ConnectedStep::Command(command),
                                None => ConnectedStep::Disconnected,
                            }
                        }
                        watch = watch_stream.message() => ConnectedStep::Watch(
                            watch.map_err(|err| DcsError::Io(format!("dcs watch receive failed: {err}")))
                        ),
                        _ = async {
                            if let Some(deadline) = keepalive_deadline {
                                tokio::time::sleep_until(deadline).await;
                            }
                        }, if keepalive_deadline.is_some() => ConnectedStep::KeepAlive,
                    }
                };

                let outcome = match step {
                    ConnectedStep::Tick | ConnectedStep::PgChanged => {
                        let pg_snapshot = self.pg.latest();
                        let Some(session) = self.session.as_mut() else {
                            return Err(WorkerError::Message(
                                "dcs session disappeared during connected step".to_string(),
                            ));
                        };
                        session
                            .sync_local_member(
                                &self.cfg.member_id,
                                &self.keys,
                                self.cfg,
                                lease_ttl_ms(self.cfg),
                                &pg_snapshot,
                                &mut self.cluster,
                            )
                            .await
                    }
                    ConnectedStep::Command(command) => {
                        let Some(session) = self.session.as_mut() else {
                            return Err(WorkerError::Message(
                                "dcs session disappeared during connected step".to_string(),
                            ));
                        };
                        session
                            .apply_command(
                                &self.cfg.member_id,
                                &self.keys,
                                lease_ttl_ms(self.cfg),
                                command,
                                &mut self.cluster,
                            )
                            .await
                    }
                    ConnectedStep::Watch(Ok(Some(response))) => {
                        let Some(session) = self.session.as_mut() else {
                            return Err(WorkerError::Message(
                                "dcs session disappeared during connected step".to_string(),
                            ));
                        };
                        session.apply_watch(&self.keys, &mut self.cluster, response)
                    }
                    ConnectedStep::Watch(Ok(None)) => {
                        Err(DcsError::Io("etcd watch stream closed".to_string()))
                    }
                    ConnectedStep::Watch(Err(err)) => Err(err),
                    ConnectedStep::KeepAlive => {
                        let Some(session) = self.session.as_mut() else {
                            return Err(WorkerError::Message(
                                "dcs session disappeared during connected step".to_string(),
                            ));
                        };
                        session.refresh_leader_keepalive().await
                    }
                    ConnectedStep::Disconnected => {
                        return Err(WorkerError::Message(
                            "dcs command channel disconnected".to_string(),
                        ));
                    }
                };

                match outcome {
                    Ok(()) => self.publish_current_view(true)?,
                    Err(DcsError::LeaderLeaseExpired(cause)) => {
                        match self.recover_expired_leadership(cause).await {
                            Ok(()) => self.publish_current_view(true)?,
                            Err(err) => {
                                self.session = None;
                                self.log_failure(FailurePhase::ConnectedStep, &err)?;
                                reconnect_at = Instant::now() + RECONNECT_BACKOFF;
                                self.publish_current_view(false)?;
                            }
                        }
                    }
                    Err(err) => {
                        self.session = None;
                        self.log_failure(FailurePhase::ConnectedStep, &err)?;
                        reconnect_at = Instant::now() + RECONNECT_BACKOFF;
                        self.publish_current_view(false)?;
                    }
                }
                continue;
            }

            enum DisconnectedStep {
                Reconnect,
                Tick,
                PgChanged,
                Command(Option<DcsCommand>),
            }

            let step = {
                let pg = &mut self.pg;
                let command_inbox = &mut self.command_inbox;
                tokio::select! {
                    _ = tokio::time::sleep_until(reconnect_at) => DisconnectedStep::Reconnect,
                    _ = tick.tick() => DisconnectedStep::Tick,
                    changed = pg.changed() => {
                        changed.map_err(|err| WorkerError::Message(format!("dcs pg subscriber closed: {err}")))?;
                        DisconnectedStep::PgChanged
                    }
                    command = command_inbox.recv() => DisconnectedStep::Command(command),
                }
            };

            match step {
                DisconnectedStep::Reconnect => match self.connect_session().await {
                    Ok(session) => {
                        self.session = Some(session);
                        let pg_snapshot = self.pg.latest();
                        let Some(session) = self.session.as_mut() else {
                            return Err(WorkerError::Message(
                                "dcs session missing after successful connect".to_string(),
                            ));
                        };
                        let outcome = session
                            .sync_local_member(
                                &self.cfg.member_id,
                                &self.keys,
                                self.cfg,
                                lease_ttl_ms(self.cfg),
                                &pg_snapshot,
                                &mut self.cluster,
                            )
                            .await;
                        match outcome {
                            Ok(()) => self.publish_current_view(true)?,
                            Err(err) => {
                                self.session = None;
                                self.log_failure(FailurePhase::InitialConnect, &err)?;
                                reconnect_at = Instant::now() + RECONNECT_BACKOFF;
                                self.publish_current_view(false)?;
                            }
                        }
                    }
                    Err(err) => {
                        self.log_failure(FailurePhase::InitialConnect, &err)?;
                        reconnect_at = Instant::now() + RECONNECT_BACKOFF;
                        self.publish_current_view(false)?;
                    }
                },
                DisconnectedStep::Tick | DisconnectedStep::PgChanged => {}
                DisconnectedStep::Command(Some(_command)) => {}
                DisconnectedStep::Command(None) => {
                    return Err(WorkerError::Message(
                        "dcs command channel disconnected".to_string(),
                    ));
                }
            }
        }
    }

    async fn connect_session(&mut self) -> Result<ConnectedSession, DcsError> {
        ConnectedSession::connect(
            &self.cfg.dcs.endpoints,
            self.cfg,
            &self.keys,
            &mut self.cluster,
        )
        .await
    }

    async fn recover_expired_leadership(&mut self, cause: String) -> Result<(), DcsError> {
        let Some(session) = self.session.as_mut() else {
            return Ok(());
        };
        session.leader_lease = None;
        self.cluster.leadership = None;
        self.log
            .send(DcsLogEvent::LeaderLeaseExpired { cause })
            .map_err(|log_err| {
                DcsError::Io(format!("dcs lease-expiry log emit failed: {log_err}"))
            })?;
        session
            .restore_snapshot(&self.keys, &mut self.cluster)
            .await?;
        Ok(())
    }

    fn log_failure(&self, phase: FailurePhase, err: &DcsError) -> Result<(), WorkerError> {
        self.log.send(phase.event(err)).map_err(|log_err| {
            WorkerError::Message(format!("dcs failure log emit failed: {log_err}"))
        })
    }

    fn publish_current_view(&mut self, etcd_reachable: bool) -> Result<(), WorkerError> {
        let next = current_snapshot(
            etcd_reachable,
            &self.cfg.member_id,
            &self.cluster.leadership,
            &self.cluster.switchover,
            &self.cluster.members,
        );
        let next_authority = next.authority();
        if self.cluster.last_emitted_authority != Some(next_authority) {
            let previous = self.cluster.last_emitted_authority;
            self.cluster.last_emitted_authority = Some(next_authority);
            self.log
                .send(DcsLogEvent::CoordinationModeTransition {
                    previous: previous.map(|authority| authority.to_string()),
                    next: next_authority.to_string(),
                })
                .map_err(|err| {
                    WorkerError::Message(format!("dcs coordination mode log emit failed: {err}"))
                })?;
        }
        self.publisher
            .publish(next)
            .map_err(|err| WorkerError::Message(format!("dcs publish failed: {err}")))
    }
}

impl DcsClusterState {
    fn new() -> Self {
        Self {
            members: BTreeMap::new(),
            leadership: None,
            switchover: SwitchoverState::None,
            last_emitted_authority: None,
        }
    }

    fn reset(&mut self) {
        self.members.clear();
        self.leadership = None;
        self.switchover = SwitchoverState::None;
    }
}

impl ConnectedSession {
    async fn connect(
        endpoints: &[DcsEndpoint],
        cfg: &RuntimeConfigV2,
        keys: &DcsKeySpace,
        cluster: &mut DcsClusterState,
    ) -> Result<Self, DcsError> {
        let endpoints = endpoints
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        let options = EtcdRuntime::connect_options(cfg)?;
        let mut client =
            EtcdRuntime::timeout("etcd connect", Client::connect(endpoints, options)).await?;
        let revision = Self::restore_snapshot_from_client(&mut client, keys, cluster).await?;
        let (watcher, watch_stream) = EtcdRuntime::timeout(
            "etcd watch",
            client.watch(
                keys.prefix(),
                Some(
                    WatchOptions::new()
                        .with_prefix()
                        .with_start_revision(revision.saturating_add(1)),
                ),
            ),
        )
        .await?;
        Ok(Self {
            client,
            _watcher: watcher,
            watch_stream,
            leader_lease: None,
        })
    }

    async fn sync_local_member(
        &mut self,
        member_id: &MemberId,
        keys: &DcsKeySpace,
        cfg: &RuntimeConfigV2,
        member_ttl_ms: u64,
        pg_snapshot: &PgInfoState,
        cluster: &mut DcsClusterState,
    ) -> Result<(), DcsError> {
        let now = EtcdRuntime::unix_millis().map_err(|err| DcsError::Io(err.to_string()))?;
        let member_key = DcsKey::Member(member_id.clone());
        let member_path = keys.path(&member_key);
        let pg_snapshot_stale = pg_snapshot
            .last_refresh_at()
            .is_none_or(|last_refresh_at| now.0.saturating_sub(last_refresh_at.0) > member_ttl_ms);

        if pg_snapshot_stale {
            EtcdRuntime::timeout(
                "etcd delete",
                self.client.delete(member_path.as_str(), None),
            )
            .await?;
            cluster.members.remove(member_id);
            self.release_local_leadership(keys, member_id, cluster)
                .await?;
            return Ok(());
        }

        let cluster_advertisement = advertised_cluster_postgres(cfg);
        let operator_advertisement = advertised_operator_postgres(cfg);
        let operator_api_advertisement = advertised_operator_api(cfg);
        let local_member = build_local_member_state(
            &cluster_advertisement,
            operator_advertisement.as_ref(),
            operator_api_advertisement.as_ref(),
            pg_snapshot,
        );
        let write = keys.write(member_key, &local_member)?;
        let lease = EtcdRuntime::timeout(
            "etcd lease grant",
            self.client
                .lease_grant(EtcdRuntime::ttl_seconds_from_ms(member_ttl_ms)?, None),
        )
        .await?;
        let options = PutOptions::new().with_lease(lease.id());
        EtcdRuntime::timeout(
            "etcd put",
            self.client
                .put(write.path.as_str(), write.value, Some(options)),
        )
        .await?;
        cluster.members.insert(member_id.clone(), local_member);
        Ok(())
    }

    async fn apply_command(
        &mut self,
        member_id: &MemberId,
        keys: &DcsKeySpace,
        member_ttl_ms: u64,
        command: DcsCommand,
        cluster: &mut DcsClusterState,
    ) -> Result<(), DcsError> {
        match command {
            DcsCommand::AcquireLeadership => {
                self.acquire_local_leadership(member_id, keys, member_ttl_ms, cluster)
                    .await
            }
            DcsCommand::ReleaseLeadership => {
                self.release_local_leadership(keys, member_id, cluster)
                    .await
            }
            DcsCommand::PublishSwitchover(request) => {
                self.set_switchover(keys, SwitchoverState::Pending(request), cluster)
                    .await
            }
            DcsCommand::ClearSwitchover => {
                self.set_switchover(keys, SwitchoverState::None, cluster)
                    .await
            }
        }
    }

    async fn acquire_local_leadership(
        &mut self,
        member_id: &MemberId,
        keys: &DcsKeySpace,
        member_ttl_ms: u64,
        cluster: &mut DcsClusterState,
    ) -> Result<(), DcsError> {
        let leader_key = DcsKey::Leader;
        let leader_path = keys.path(&leader_key);
        if self
            .leader_lease
            .as_ref()
            .is_some_and(|lease| lease.leader_path == leader_path && lease.member_id == *member_id)
        {
            return Ok(());
        }

        let epoch = LeaseEpoch {
            holder: member_id.clone(),
            generation: EtcdRuntime::unix_millis()
                .map_err(|err| DcsError::Io(err.to_string()))?
                .0,
        };
        let write = keys.write(leader_key, &epoch)?;
        let ttl_seconds = EtcdRuntime::ttl_seconds_from_ms(member_ttl_ms)?;
        let lease = EtcdRuntime::timeout(
            "etcd lease grant",
            self.client.lease_grant(ttl_seconds, None),
        )
        .await?;
        let lease_id = lease.id();
        let txn = Txn::new()
            .when(vec![Compare::version(
                write.path.as_str(),
                CompareOp::Equal,
                0,
            )])
            .and_then(vec![TxnOp::put(
                write.path.as_str(),
                write.value,
                Some(PutOptions::new().with_lease(lease_id)),
            )]);
        let response = EtcdRuntime::timeout("etcd leader lease txn", self.client.txn(txn)).await?;
        if !response.succeeded() {
            EtcdRuntime::timeout("etcd lease revoke", self.client.lease_revoke(lease_id)).await?;
            let existing =
                EtcdRuntime::timeout("etcd get", self.client.get(write.path.as_str(), None))
                    .await?;
            if existing
                .kvs()
                .iter()
                .find_map(|kv| {
                    str::from_utf8(kv.value())
                        .ok()
                        .and_then(|raw| serde_json::from_str::<LeaseEpoch>(raw).ok())
                })
                .is_some_and(|existing_epoch| existing_epoch.holder == *member_id)
            {
                return Ok(());
            }
            return Err(DcsError::AlreadyExists(write.path));
        }

        let (keeper, stream) = EtcdRuntime::timeout(
            "etcd lease keepalive create",
            self.client.lease_keep_alive(lease_id),
        )
        .await?;
        self.leader_lease = Some(OwnedLeaderLease {
            lease_id,
            leader_path,
            member_id: member_id.clone(),
            ttl_seconds,
            keeper,
            stream,
            next_keepalive_at: Instant::now() + EtcdRuntime::leader_keepalive_interval(ttl_seconds),
        });
        cluster.leadership = Some(epoch);
        Ok(())
    }

    async fn release_local_leadership(
        &mut self,
        keys: &DcsKeySpace,
        self_id: &MemberId,
        cluster: &mut DcsClusterState,
    ) -> Result<(), DcsError> {
        let leader_path = keys.path(&DcsKey::Leader);
        let Some(lease) = self.leader_lease.take() else {
            if cluster
                .leadership
                .as_ref()
                .is_some_and(|epoch| epoch.holder == *self_id)
            {
                cluster.leadership = None;
            }
            return Ok(());
        };
        if lease.leader_path != leader_path || lease.member_id != *self_id {
            self.leader_lease = Some(lease);
            return Ok(());
        }

        EtcdRuntime::timeout(
            "etcd lease revoke",
            self.client.lease_revoke(lease.lease_id),
        )
        .await?;
        cluster.leadership = None;
        Ok(())
    }

    async fn set_switchover(
        &mut self,
        keys: &DcsKeySpace,
        next: SwitchoverState,
        cluster: &mut DcsClusterState,
    ) -> Result<(), DcsError> {
        if cluster.switchover == next {
            return Ok(());
        }
        let switchover_path = keys.path(&DcsKey::Switchover);
        if next == SwitchoverState::None {
            EtcdRuntime::timeout(
                "etcd delete",
                self.client.delete(switchover_path.as_str(), None),
            )
            .await?;
        } else {
            let write = keys.write(DcsKey::Switchover, &next)?;
            EtcdRuntime::timeout(
                "etcd put",
                self.client.put(write.path.as_str(), write.value, None),
            )
            .await?;
        }
        cluster.switchover = next;
        Ok(())
    }

    async fn refresh_leader_keepalive(&mut self) -> Result<(), DcsError> {
        let Some(lease) = self.leader_lease.as_mut() else {
            return Ok(());
        };
        EtcdRuntime::timeout("etcd lease keepalive send", lease.keeper.keep_alive()).await?;
        let response =
            EtcdRuntime::timeout("etcd lease keepalive receive", lease.stream.message()).await?;
        match response {
            Some(message) if message.ttl() > 0 => {
                lease.next_keepalive_at =
                    Instant::now() + EtcdRuntime::leader_keepalive_interval(lease.ttl_seconds);
                Ok(())
            }
            Some(_) => Err(DcsError::LeaderLeaseExpired(format!(
                "leader lease keepalive reported expired lease `{}`",
                lease.lease_id
            ))),
            None => Err(DcsError::Io(format!(
                "leader lease keepalive stream closed for lease `{}`",
                lease.lease_id
            ))),
        }
    }

    fn apply_watch(
        &mut self,
        keys: &DcsKeySpace,
        cluster: &mut DcsClusterState,
        response: WatchResponse,
    ) -> Result<(), DcsError> {
        if response.canceled() || response.compact_revision() > 0 {
            return Err(DcsError::Io(format!(
                "etcd watch canceled: reason='{}' compact_revision={}",
                response.cancel_reason(),
                response.compact_revision()
            )));
        }
        for event in response.events() {
            let Some(kv) = event.kv() else {
                return Err(DcsError::Io(
                    "etcd watch event missing key-value payload".to_string(),
                ));
            };
            let path = str::from_utf8(kv.key()).map_err(|err| DcsError::Decode {
                key: "watch-key".to_string(),
                message: err.to_string(),
            })?;
            let raw = match event.event_type() {
                EventType::Put => {
                    Some(str::from_utf8(kv.value()).map_err(|err| DcsError::Decode {
                        key: path.to_string(),
                        message: err.to_string(),
                    })?)
                }
                EventType::Delete => None,
            };
            keys.apply(cluster, path, raw)?;
        }
        Ok(())
    }

    async fn restore_snapshot(
        &mut self,
        keys: &DcsKeySpace,
        cluster: &mut DcsClusterState,
    ) -> Result<i64, DcsError> {
        Self::restore_snapshot_from_client(&mut self.client, keys, cluster).await
    }

    async fn restore_snapshot_from_client(
        client: &mut Client,
        keys: &DcsKeySpace,
        cluster: &mut DcsClusterState,
    ) -> Result<i64, DcsError> {
        let response = EtcdRuntime::timeout(
            "etcd get",
            client.get(keys.prefix(), Some(GetOptions::new().with_prefix())),
        )
        .await?;
        cluster.reset();
        for kv in response.kvs() {
            let path = str::from_utf8(kv.key()).map_err(|err| DcsError::Decode {
                key: "watch-key".to_string(),
                message: err.to_string(),
            })?;
            let raw = str::from_utf8(kv.value()).map_err(|err| DcsError::Decode {
                key: path.to_string(),
                message: err.to_string(),
            })?;
            keys.apply(cluster, path, Some(raw))?;
        }
        Ok(response
            .header()
            .map(|header| header.revision())
            .unwrap_or_default())
    }
}

impl FailurePhase {
    fn event(self, err: &DcsError) -> DcsLogEvent {
        match (self, err) {
            (_, DcsError::LeaderLeaseExpired(cause)) => DcsLogEvent::LeaderLeaseExpired {
                cause: cause.clone(),
            },
            (Self::ConnectedStep, DcsError::Io(cause)) => DcsLogEvent::ConnectedStepStoreIoFailed {
                cause: cause.clone(),
            },
            (Self::ConnectedStep, DcsError::Decode { key, message }) => {
                DcsLogEvent::ConnectedStepDecodeFailed {
                    cause: format!("key `{key}` decode failed: {message}"),
                }
            }
            (Self::ConnectedStep, DcsError::AlreadyExists(cause)) => {
                DcsLogEvent::ConnectedStepAlreadyExists {
                    cause: cause.clone(),
                }
            }
            (Self::InitialConnect, DcsError::Io(cause)) => {
                DcsLogEvent::InitialConnectStoreIoFailed {
                    cause: cause.clone(),
                }
            }
            (Self::InitialConnect, DcsError::Decode { key, message }) => {
                DcsLogEvent::InitialConnectDecodeFailed {
                    cause: format!("key `{key}` decode failed: {message}"),
                }
            }
            (Self::InitialConnect, DcsError::AlreadyExists(cause)) => {
                DcsLogEvent::InitialConnectAlreadyExists {
                    cause: cause.clone(),
                }
            }
        }
    }
}

impl DcsKeySpace {
    fn new(scope: &str) -> Self {
        Self {
            prefix: format!("/{}/", scope.trim_matches('/')),
        }
    }

    fn prefix(&self) -> &str {
        self.prefix.as_str()
    }

    fn path(&self, key: &DcsKey) -> String {
        match key {
            DcsKey::Member(member_id) => format!("{}member/{}", self.prefix, member_id.as_str()),
            DcsKey::Leader => format!("{}leader", self.prefix),
            DcsKey::Switchover => format!("{}switchover", self.prefix),
        }
    }

    fn write<T: Serialize>(&self, key: DcsKey, value: &T) -> Result<KeyValueWrite, DcsError> {
        let path = self.path(&key);
        let value = serde_json::to_string(value).map_err(|err| DcsError::Decode {
            key: path.clone(),
            message: err.to_string(),
        })?;
        Ok(KeyValueWrite { path, value })
    }

    fn apply(
        &self,
        cluster: &mut DcsClusterState,
        path: &str,
        raw: Option<&str>,
    ) -> Result<(), DcsError> {
        if let Some(change) = self.change(path, raw)? {
            change.apply(cluster);
        }
        Ok(())
    }

    fn change(&self, path: &str, raw: Option<&str>) -> Result<Option<DcsChange>, DcsError> {
        let Some(key) = self.parse(path) else {
            return Ok(None);
        };
        let decode_err = |err: serde_json::Error| DcsError::Decode {
            key: path.to_string(),
            message: err.to_string(),
        };
        match (key, raw) {
            (DcsKey::Member(member_id), Some(raw)) => Ok(Some(DcsChange::Member(
                member_id,
                Some(Box::new(serde_json::from_str(raw).map_err(decode_err)?)),
            ))),
            (DcsKey::Member(member_id), None) => Ok(Some(DcsChange::Member(member_id, None))),
            (DcsKey::Leader, Some(raw)) => Ok(Some(DcsChange::Leader(Some(
                serde_json::from_str(raw).map_err(decode_err)?,
            )))),
            (DcsKey::Leader, None) => Ok(Some(DcsChange::Leader(None))),
            (DcsKey::Switchover, Some(raw)) => Ok(Some(DcsChange::Switchover(Some(
                serde_json::from_str(raw).map_err(decode_err)?,
            )))),
            (DcsKey::Switchover, None) => Ok(Some(DcsChange::Switchover(None))),
        }
    }

    fn parse(&self, full_path: &str) -> Option<DcsKey> {
        let suffix = full_path.strip_prefix(self.prefix())?;
        let mut parts = suffix.split('/');
        match (parts.next(), parts.next(), parts.next()) {
            (Some("member"), Some(member_id), None) if !member_id.is_empty() => {
                Some(DcsKey::Member(MemberId(member_id.to_string())))
            }
            (Some("leader"), None, None) => Some(DcsKey::Leader),
            (Some("switchover"), None, None) => Some(DcsKey::Switchover),
            _ => None,
        }
    }
}

impl DcsChange {
    fn apply(self, cluster: &mut DcsClusterState) {
        match self {
            Self::Member(member_id, Some(member)) => {
                cluster.members.insert(member_id, *member);
            }
            Self::Member(member_id, None) => {
                cluster.members.remove(&member_id);
            }
            Self::Leader(leadership) => {
                cluster.leadership = leadership;
            }
            Self::Switchover(Some(switchover)) => {
                cluster.switchover = switchover;
            }
            Self::Switchover(None) => {
                cluster.switchover = SwitchoverState::None;
            }
        }
    }
}

impl EtcdRuntime {
    fn unix_millis() -> Result<crate::state::UnixMillis, WorkerError> {
        let elapsed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|err| {
                WorkerError::Message(format!("system clock before unix epoch: {err}"))
            })?;
        let millis = u64::try_from(elapsed.as_millis())
            .map_err(|err| WorkerError::Message(format!("unix millis conversion failed: {err}")))?;
        Ok(crate::state::UnixMillis(millis))
    }

    fn ttl_seconds_from_ms(lease_ttl_ms: u64) -> Result<i64, DcsError> {
        let rounded_seconds = lease_ttl_ms.saturating_add(999) / 1000;
        let clamped_seconds = rounded_seconds.max(MIN_LEADER_LEASE_TTL_SECONDS);
        i64::try_from(clamped_seconds).map_err(|_| {
            DcsError::Io(format!(
                "leader lease ttl `{lease_ttl_ms}`ms is too large to convert to etcd seconds"
            ))
        })
    }

    fn leader_keepalive_interval(ttl_seconds: i64) -> Duration {
        if ttl_seconds <= 1 {
            return Duration::from_millis(500);
        }
        Duration::from_secs(std::cmp::max(1, ttl_seconds as u64 / 3))
    }

    fn connect_options(cfg: &RuntimeConfigV2) -> Result<Option<ConnectOptions>, DcsError> {
        let mut options = ConnectOptions::new();
        let mut configured = false;

        if let Some(auth) = cfg.dcs.auth.as_ref() {
            options = options.with_user(auth.username.clone(), auth.password.as_str().to_string());
            configured = true;
        }

        if let Some(tls_cfg) = cfg.dcs.tls.as_ref() {
            let mut tls = TlsOptions::new();
            if let Some(ca_cert) = tls_cfg.ca_cert.as_ref() {
                let pem = read_tls_file(ca_cert)?;
                tls = tls.ca_certificate(Certificate::from_pem(pem));
            }
            if tls_identity_enabled(tls_cfg) {
                let cert_pem = read_tls_file(&tls_cfg.cert)?;
                let key_pem = read_tls_file(&tls_cfg.key)?;
                tls = tls.identity(Identity::from_pem(cert_pem, key_pem));
            }
            options = options.with_tls(tls);
            configured = true;
        }

        Ok(configured.then_some(options))
    }

    async fn timeout<T, F>(operation: &str, fut: F) -> Result<T, DcsError>
    where
        F: std::future::Future<Output = Result<T, etcd_client::Error>>,
    {
        match tokio::time::timeout(ETCD_TIMEOUT, fut).await {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(err)) => Err(DcsError::Io(format!("{operation} failed: {err}"))),
            Err(err) => Err(DcsError::Io(format!("{operation} timed out: {err}"))),
        }
    }
}

fn lease_ttl_ms(cfg: &RuntimeConfigV2) -> u64 {
    u64::try_from(cfg.ha.lease_ttl.as_millis()).unwrap_or(u64::MAX)
}

fn advertised_cluster_postgres(cfg: &RuntimeConfigV2) -> PgRoute {
    cfg.postgres.cluster_advertise.clone()
}

fn advertised_operator_postgres(cfg: &RuntimeConfigV2) -> Option<PgRoute> {
    cfg.postgres.operator_advertise.clone()
}

fn advertised_operator_api(cfg: &RuntimeConfigV2) -> Option<ApiRoute> {
    cfg.api.advertise.clone()
}

fn read_tls_file(path: &std::path::Path) -> Result<Vec<u8>, DcsError> {
    std::fs::read(path)
        .map_err(|err| DcsError::Io(format!("read tls file `{}` failed: {err}", path.display())))
}

fn tls_identity_enabled(tls: &TlsConfig) -> bool {
    !tls.cert.as_os_str().is_empty() && !tls.key.as_os_str().is_empty()
}
