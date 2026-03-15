use std::collections::BTreeMap;
use std::{str, time::Duration};

use etcd_client::{
    Certificate, Client, Compare, CompareOp, ConnectOptions, EventType, GetOptions, Identity,
    LeaseKeepAliveStream, LeaseKeeper, PutOptions, TlsOptions, Txn, TxnOp, WatchOptions,
    WatchResponse, WatchStream, Watcher,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::mpsc;
use tokio::time::{Instant, MissedTickBehavior};

use crate::{
    config::{
        resolve_inline_or_path_bytes, resolve_secret_string, DcsAuthConfig, DcsClientConfig,
        DcsEndpoint, DcsTlsConfig,
    },
    logging::LogSender,
    pginfo::state::PgInfoState,
    state::{
        LeaseEpoch, MemberId, ObservedWalPosition, PgTcpTarget, StatePublisher, StateSubscriber,
        SwitchoverTarget, WorkerError,
    },
};

use super::{
    command::{dcs_command_channel, DcsCommand, DcsHandle},
    log_event::{DcsFailure, DcsLogEvent, DcsLogIdentity, DcsLogOrigin},
    view::{
        ClusterMemberView, ClusterView, DcsMode, DcsView, LeadershipObservation,
        MemberPostgresView, SwitchoverView,
    },
};

const ETCD_TIMEOUT: Duration = Duration::from_secs(2);
const RECONNECT_BACKOFF: Duration = Duration::from_secs(1);
const MIN_LEADER_LEASE_TTL_SECONDS: u64 = 1;

// ---------------------------------------------------------------------------
// Public (crate-level) bootstrap types
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Error, PartialEq, Eq)]
enum DcsError {
    #[error("path already exists: {0}")]
    AlreadyExists(String),
    #[error("decode failed for key `{key}`: {message}")]
    Decode { key: String, message: String },
    #[error("store I/O error: {0}")]
    Io(String),
}

// ---------------------------------------------------------------------------
// Internal worker context (fully private)
// ---------------------------------------------------------------------------

pub(super) struct WorkerCtx {
    self_id: MemberId,
    scope: String,
    endpoints: Vec<DcsEndpoint>,
    client_config: DcsClientConfig,
    poll_interval: Duration,
    member_ttl_ms: u64,
    advertised_postgres: PgTcpTarget,
    pg_subscriber: StateSubscriber<PgInfoState>,
    publisher: StatePublisher<DcsView>,
    cache: Cache,
    command_inbox: mpsc::UnboundedReceiver<DcsCommand>,
    log: LogSender,
    last_emitted_mode: Option<DcsMode>,
}

// ---------------------------------------------------------------------------
// Internal cache (etcd mirror)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct MemberRecord {
    owner: MemberId,
    ttl_ms: u64,
    postgres_target: PgTcpTarget,
    postgres: MemberPostgresView,
}

struct Cache {
    members: BTreeMap<MemberId, MemberRecord>,
    leader: Option<LeaseEpoch>,
    switchover: Option<SwitchoverTarget>,
}

impl Cache {
    fn empty() -> Self {
        Self {
            members: BTreeMap::new(),
            leader: None,
            switchover: None,
        }
    }

    fn clear(&mut self) {
        self.members.clear();
        self.leader = None;
        self.switchover = None;
    }
}

// ---------------------------------------------------------------------------
// Connected-session state
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Public entry points (for bootstrap / startup)
// ---------------------------------------------------------------------------

pub(super) struct WorkerConfig {
    pub(super) self_id: MemberId,
    pub(super) scope: String,
    pub(super) endpoints: Vec<DcsEndpoint>,
    pub(super) client_config: DcsClientConfig,
    pub(super) poll_interval: Duration,
    pub(super) member_ttl_ms: u64,
    pub(super) advertised_postgres: PgTcpTarget,
    pub(super) pg_subscriber: StateSubscriber<PgInfoState>,
    pub(super) publisher: StatePublisher<DcsView>,
    pub(super) log: LogSender,
}

pub(super) fn build_worker(config: WorkerConfig) -> (WorkerCtx, DcsHandle) {
    let (handle, command_inbox) = dcs_command_channel();
    let ctx = WorkerCtx {
        self_id: config.self_id,
        scope: config.scope,
        endpoints: config.endpoints,
        client_config: config.client_config,
        poll_interval: config.poll_interval,
        member_ttl_ms: config.member_ttl_ms,
        advertised_postgres: config.advertised_postgres,
        pg_subscriber: config.pg_subscriber,
        publisher: config.publisher,
        cache: Cache::empty(),
        command_inbox,
        log: config.log,
        last_emitted_mode: None,
    };
    (ctx, handle)
}

pub(super) async fn run(mut ctx: WorkerCtx) -> Result<(), WorkerError> {
    let mut reconnect_at = Instant::now();
    let mut session = None::<ConnectedSession>;
    let mut tick = tokio::time::interval(ctx.poll_interval);
    tick.set_missed_tick_behavior(MissedTickBehavior::Delay);
    publish_current_view(&mut ctx, false)?;

    loop {
        if let Some(connected) = session.as_mut() {
            let keepalive_deadline = connected
                .leader_lease
                .as_ref()
                .map(|lease| lease.next_keepalive_at);
            enum ConnectedStep {
                Tick,
                PgChanged,
                Command(DcsCommand),
                Watch(Result<Option<WatchResponse>, DcsError>),
                KeepAlive,
                Disconnected,
            }
            let step = tokio::select! {
                _ = tick.tick() => ConnectedStep::Tick,
                changed = ctx.pg_subscriber.changed() => {
                    changed.map_err(|err| WorkerError::Message(format!("dcs pg subscriber closed: {err}")))?;
                    ConnectedStep::PgChanged
                }
                command = ctx.command_inbox.recv() => {
                    match command {
                        Some(command) => ConnectedStep::Command(command),
                        None => ConnectedStep::Disconnected,
                    }
                }
                watch = connected.watch_stream.message() => ConnectedStep::Watch(
                    watch.map_err(|err| DcsError::Io(format!("dcs watch receive failed: {err}")))
                ),
                _ = async {
                    if let Some(deadline) = keepalive_deadline {
                        tokio::time::sleep_until(deadline).await;
                    }
                }, if keepalive_deadline.is_some() => ConnectedStep::KeepAlive,
            };

            let outcome = match step {
                ConnectedStep::Tick | ConnectedStep::PgChanged => {
                    sync_local_member(&ctx.self_id, &ctx.scope, &ctx.advertised_postgres, ctx.member_ttl_ms, &ctx.pg_subscriber.latest(), connected, &mut ctx.cache).await
                }
                ConnectedStep::Command(command) => {
                    handle_connected_command(
                        &ctx.self_id,
                        &ctx.scope,
                        ctx.member_ttl_ms,
                        connected,
                        &mut ctx.cache,
                        command,
                    )
                    .await
                }
                ConnectedStep::Watch(Ok(Some(response))) => {
                    apply_watch_response(&ctx.scope, &mut ctx.cache, response)
                }
                ConnectedStep::Watch(Ok(None)) => {
                    Err(DcsError::Io("etcd watch stream closed".to_string()))
                }
                ConnectedStep::Watch(Err(err)) => Err(err),
                ConnectedStep::KeepAlive => refresh_leader_keepalive(connected).await,
                ConnectedStep::Disconnected => {
                    return Err(WorkerError::Message(
                        "dcs command channel disconnected".to_string(),
                    ));
                }
            };

            match outcome {
                Ok(()) => publish_current_view(&mut ctx, true)?,
                Err(err) => {
                    handle_connected_failure(&mut ctx, connected, &err).await?;
                    session = None;
                    reconnect_at = Instant::now() + RECONNECT_BACKOFF;
                    publish_current_view(&mut ctx, false)?;
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
        let step = tokio::select! {
            _ = tokio::time::sleep_until(reconnect_at) => DisconnectedStep::Reconnect,
            _ = tick.tick() => DisconnectedStep::Tick,
            changed = ctx.pg_subscriber.changed() => {
                changed.map_err(|err| WorkerError::Message(format!("dcs pg subscriber closed: {err}")))?;
                DisconnectedStep::PgChanged
            }
            command = ctx.command_inbox.recv() => DisconnectedStep::Command(command),
        };

        match step {
            DisconnectedStep::Reconnect => match connect_session(&mut ctx).await {
                Ok(mut connected) => {
                    let pg_snapshot = ctx.pg_subscriber.latest();
                    if let Err(err) = sync_local_member(
                        &ctx.self_id,
                        &ctx.scope,
                        &ctx.advertised_postgres,
                        ctx.member_ttl_ms,
                        &pg_snapshot,
                        &mut connected,
                        &mut ctx.cache,
                    )
                    .await
                    {
                        handle_initial_connect_failure(&mut ctx, &err)?;
                        reconnect_at = Instant::now() + RECONNECT_BACKOFF;
                        publish_current_view(&mut ctx, false)?;
                    } else {
                        publish_current_view(&mut ctx, true)?;
                        session = Some(connected);
                    }
                }
                Err(err) => {
                    handle_initial_connect_failure(&mut ctx, &err)?;
                    reconnect_at = Instant::now() + RECONNECT_BACKOFF;
                    publish_current_view(&mut ctx, false)?;
                }
            },
            DisconnectedStep::Tick | DisconnectedStep::PgChanged => {}
            DisconnectedStep::Command(Some(_)) => {}
            DisconnectedStep::Command(None) => {
                return Err(WorkerError::Message(
                    "dcs command channel disconnected".to_string(),
                ));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// etcd connection
// ---------------------------------------------------------------------------

async fn connect_session(ctx: &mut WorkerCtx) -> Result<ConnectedSession, DcsError> {
    let prefix = scope_prefix(&ctx.scope);
    let mut client = connect_client(&ctx.endpoints, &ctx.client_config).await?;
    let revision = load_snapshot(&ctx.scope, &mut client, &mut ctx.cache).await?;
    let start_revision = revision.saturating_add(1);
    let (watcher, watch_stream) = timeout_etcd(
        "etcd watch",
        client.watch(
            prefix.as_str(),
            Some(
                WatchOptions::new()
                    .with_prefix()
                    .with_start_revision(start_revision),
            ),
        ),
    )
    .await?;
    Ok(ConnectedSession {
        client,
        _watcher: watcher,
        watch_stream,
        leader_lease: None,
    })
}

// ---------------------------------------------------------------------------
// Local member synchronization
// ---------------------------------------------------------------------------

async fn sync_local_member(
    self_id: &MemberId,
    scope: &str,
    advertised_postgres: &PgTcpTarget,
    member_ttl_ms: u64,
    pg_snapshot: &PgInfoState,
    session: &mut ConnectedSession,
    cache: &mut Cache,
) -> Result<(), DcsError> {
    let now = now_unix_millis().map_err(|err| DcsError::Io(err.to_string()))?;
    let path = member_path(scope, self_id);
    let pg_snapshot_stale = pg_snapshot.last_refresh_at().is_none_or(|last_refresh_at| {
        now.0.saturating_sub(last_refresh_at.0) > member_ttl_ms
    });

    if pg_snapshot_stale {
        timeout_etcd("etcd delete", session.client.delete(path.as_str(), None)).await?;
        cache.members.remove(self_id);
        release_local_leadership(session, scope, self_id, cache).await?;
        return Ok(());
    }

    let local_member = build_local_member_record(
        self_id,
        advertised_postgres,
        member_ttl_ms,
        pg_snapshot,
        cache.members.get(self_id),
    );
    let encoded = serde_json::to_string(&local_member).map_err(|err| DcsError::Decode {
        key: path.clone(),
        message: err.to_string(),
    })?;
    let ttl_seconds = ttl_seconds_from_ms(member_ttl_ms)?;
    let lease = timeout_etcd(
        "etcd lease grant",
        session.client.lease_grant(ttl_seconds, None),
    )
    .await?;
    let options = PutOptions::new().with_lease(lease.id());
    timeout_etcd(
        "etcd put",
        session.client.put(path.as_str(), encoded, Some(options)),
    )
    .await?;
    cache.members.insert(self_id.clone(), local_member);
    Ok(())
}

// ---------------------------------------------------------------------------
// Command handling
// ---------------------------------------------------------------------------

async fn handle_connected_command(
    self_id: &MemberId,
    scope: &str,
    member_ttl_ms: u64,
    session: &mut ConnectedSession,
    cache: &mut Cache,
    command: DcsCommand,
) -> Result<(), DcsError> {
    match command {
        DcsCommand::AcquireLeadership => {
            acquire_local_leadership(self_id, scope, member_ttl_ms, session, cache).await
        }
        DcsCommand::ReleaseLeadership => {
            release_local_leadership(session, scope, self_id, cache).await
        }
        DcsCommand::PublishSwitchover(target) => {
            publish_switchover(session, scope, cache, target).await
        }
        DcsCommand::ClearSwitchover => clear_switchover(session, scope, cache).await,
    }
}

// ---------------------------------------------------------------------------
// Leadership
// ---------------------------------------------------------------------------

async fn acquire_local_leadership(
    self_id: &MemberId,
    scope: &str,
    member_ttl_ms: u64,
    session: &mut ConnectedSession,
    cache: &mut Cache,
) -> Result<(), DcsError> {
    let path = leader_path(scope);
    if session
        .leader_lease
        .as_ref()
        .is_some_and(|lease| lease.leader_path == path && lease.member_id == *self_id)
    {
        return Ok(());
    }

    let epoch = LeaseEpoch {
        holder: self_id.clone(),
        generation: now_unix_millis()
            .map_err(|err| DcsError::Io(err.to_string()))?
            .0,
    };
    let encoded = serde_json::to_string(&epoch).map_err(|err| DcsError::Decode {
        key: path.clone(),
        message: err.to_string(),
    })?;
    let ttl_seconds = ttl_seconds_from_ms(member_ttl_ms)?;
    let lease = timeout_etcd(
        "etcd lease grant",
        session.client.lease_grant(ttl_seconds, None),
    )
    .await?;
    let lease_id = lease.id();
    let txn = Txn::new()
        .when(vec![Compare::version(path.as_str(), CompareOp::Equal, 0)])
        .and_then(vec![TxnOp::put(
            path.as_str(),
            encoded,
            Some(PutOptions::new().with_lease(lease_id)),
        )]);
    let response = timeout_etcd("etcd leader lease txn", session.client.txn(txn)).await?;
    if !response.succeeded() {
        timeout_etcd("etcd lease revoke", session.client.lease_revoke(lease_id)).await?;
        let existing = timeout_etcd("etcd get", session.client.get(path.as_str(), None)).await?;
        if existing
            .kvs()
            .iter()
            .find_map(|kv| {
                str::from_utf8(kv.value())
                    .ok()
                    .and_then(|raw| serde_json::from_str::<LeaseEpoch>(raw).ok())
            })
            .is_some_and(|existing_epoch| existing_epoch.holder == *self_id)
        {
            return Ok(());
        }
        return Err(DcsError::AlreadyExists(path));
    }

    let (keeper, stream) = timeout_etcd(
        "etcd lease keepalive create",
        session.client.lease_keep_alive(lease_id),
    )
    .await?;
    session.leader_lease = Some(OwnedLeaderLease {
        lease_id,
        leader_path: path,
        member_id: self_id.clone(),
        ttl_seconds,
        keeper,
        stream,
        next_keepalive_at: Instant::now() + leader_keepalive_interval(ttl_seconds),
    });
    cache.leader = Some(epoch);
    Ok(())
}

async fn release_local_leadership(
    session: &mut ConnectedSession,
    scope: &str,
    self_id: &MemberId,
    cache: &mut Cache,
) -> Result<(), DcsError> {
    let path = leader_path(scope);
    let Some(lease) = session.leader_lease.take() else {
        if cache
            .leader
            .as_ref()
            .is_some_and(|epoch| epoch.holder == *self_id)
        {
            cache.leader = None;
        }
        return Ok(());
    };
    if lease.leader_path != path || lease.member_id != *self_id {
        session.leader_lease = Some(lease);
        return Ok(());
    }

    timeout_etcd(
        "etcd lease revoke",
        session.client.lease_revoke(lease.lease_id),
    )
    .await?;
    cache.leader = None;
    Ok(())
}

// ---------------------------------------------------------------------------
// Switchover
// ---------------------------------------------------------------------------

async fn publish_switchover(
    session: &mut ConnectedSession,
    scope: &str,
    cache: &mut Cache,
    target: SwitchoverTarget,
) -> Result<(), DcsError> {
    if cache.switchover.as_ref().is_some_and(|t| *t == target) {
        return Ok(());
    }
    let path = switchover_path(scope);
    let encoded = serde_json::to_string(&target).map_err(|err| DcsError::Decode {
        key: path.clone(),
        message: err.to_string(),
    })?;
    timeout_etcd("etcd put", session.client.put(path.as_str(), encoded, None)).await?;
    cache.switchover = Some(target);
    Ok(())
}

async fn clear_switchover(
    session: &mut ConnectedSession,
    scope: &str,
    cache: &mut Cache,
) -> Result<(), DcsError> {
    if cache.switchover.is_none() {
        return Ok(());
    }
    let path = switchover_path(scope);
    timeout_etcd("etcd delete", session.client.delete(path.as_str(), None)).await?;
    cache.switchover = None;
    Ok(())
}

// ---------------------------------------------------------------------------
// Leader keepalive
// ---------------------------------------------------------------------------

async fn refresh_leader_keepalive(session: &mut ConnectedSession) -> Result<(), DcsError> {
    let Some(lease) = session.leader_lease.as_mut() else {
        return Ok(());
    };
    timeout_etcd("etcd lease keepalive send", lease.keeper.keep_alive()).await?;
    let response = timeout_etcd("etcd lease keepalive receive", lease.stream.message()).await?;
    match response {
        Some(message) if message.ttl() > 0 => {
            lease.next_keepalive_at = Instant::now() + leader_keepalive_interval(lease.ttl_seconds);
            Ok(())
        }
        Some(_) => Err(DcsError::Io(format!(
            "leader lease keepalive reported expired lease `{}`",
            lease.lease_id
        ))),
        None => Err(DcsError::Io(format!(
            "leader lease keepalive stream closed for lease `{}`",
            lease.lease_id
        ))),
    }
}

// ---------------------------------------------------------------------------
// Error / failure handling & logging
// ---------------------------------------------------------------------------

async fn handle_connected_failure(
    ctx: &mut WorkerCtx,
    session: &mut ConnectedSession,
    err: &DcsError,
) -> Result<(), WorkerError> {
    if session.leader_lease.is_some() {
        session.leader_lease = None;
    }
    ctx.log
        .send(DcsLogEvent::ConnectedStepFailed {
            origin: DcsLogOrigin::ConnectedFailure,
            identity: log_identity(ctx),
            failure: dcs_failure(err),
        })
        .map_err(|log_err| WorkerError::Message(format!("dcs watch failure log emit failed: {log_err}")))
}

fn handle_initial_connect_failure(ctx: &mut WorkerCtx, err: &DcsError) -> Result<(), WorkerError> {
    ctx.log
        .send(DcsLogEvent::InitialConnectFailed {
            origin: DcsLogOrigin::InitialConnectFailure,
            identity: log_identity(ctx),
            failure: dcs_failure(err),
        })
        .map_err(|log_err| WorkerError::Message(format!("dcs connect failure log emit failed: {log_err}")))
}

// ---------------------------------------------------------------------------
// State machine: cache -> DcsView
// ---------------------------------------------------------------------------

fn publish_current_view(ctx: &mut WorkerCtx, etcd_reachable: bool) -> Result<(), WorkerError> {
    let mode = evaluate_mode(etcd_reachable, &ctx.cache, &ctx.self_id);
    let next = build_dcs_view(mode, &ctx.cache);
    if ctx.last_emitted_mode != Some(next.mode()) {
        let previous = ctx.last_emitted_mode;
        let next_mode = next.mode();
        ctx.last_emitted_mode = Some(next_mode);
        ctx.log
            .send(DcsLogEvent::CoordinationModeTransition {
                origin: DcsLogOrigin::PublishCurrentView,
                identity: log_identity(ctx),
                previous,
                next: next_mode,
            })
            .map_err(|err| WorkerError::Message(format!("dcs coordination mode log emit failed: {err}")))?;
    }
    ctx.publisher
        .publish(next)
        .map_err(|err| WorkerError::Message(format!("dcs publish failed: {err}")))
}

fn evaluate_mode(etcd_reachable: bool, cache: &Cache, self_id: &MemberId) -> DcsMode {
    if !etcd_reachable {
        return DcsMode::NotTrusted;
    }
    if !cache.members.contains_key(self_id) {
        return DcsMode::Degraded;
    }
    let count = cache.members.len();
    let has_quorum = if count <= 1 { count == 1 } else { count >= 2 };
    if has_quorum {
        DcsMode::Coordinated
    } else {
        DcsMode::Degraded
    }
}

fn build_dcs_view(mode: DcsMode, cache: &Cache) -> DcsView {
    let authoritative_leader = cache
        .leader
        .as_ref()
        .map(|epoch| epoch.holder.clone());
    let cluster = ClusterView::from_parts(
        cache
            .members
            .iter()
            .map(|(member_id, record)| {
                let postgres = if authoritative_leader
                    .as_ref()
                    .is_some_and(|leader| leader != member_id)
                    && record.postgres.is_primary()
                {
                    record.postgres.downgrade_to_unknown()
                } else {
                    record.postgres.clone()
                };
                (
                    member_id.clone(),
                    ClusterMemberView::from_parts(postgres, record.postgres_target.clone()),
                )
            })
            .collect(),
        cache
            .leader
            .as_ref()
            .map(|epoch| LeadershipObservation::Held(epoch.clone()))
            .unwrap_or(LeadershipObservation::Open),
        cache
            .switchover
            .as_ref()
            .map(|target| SwitchoverView::Requested(target.clone()))
            .unwrap_or(SwitchoverView::None),
    );

    match mode {
        DcsMode::NotTrusted => DcsView::NotTrusted(cluster),
        DcsMode::Degraded => DcsView::Degraded(cluster),
        DcsMode::Coordinated => DcsView::Coordinated(cluster),
    }
}

fn build_local_member_record(
    self_id: &MemberId,
    postgres_target: &PgTcpTarget,
    ttl_ms: u64,
    pg_state: &PgInfoState,
    previous_record: Option<&MemberRecord>,
) -> MemberRecord {
    let postgres = match pg_state {
        PgInfoState::Unknown { common } => MemberPostgresView::Unknown {
            readiness: common.readiness.clone(),
            timeline: common
                .timeline
                .or_else(|| previous_record.and_then(|r| r.postgres.timeline())),
            system_identifier: common
                .system_identifier
                .or_else(|| previous_record.and_then(|r| r.postgres.system_identifier())),
        },
        PgInfoState::Primary {
            common, wal_lsn, ..
        } => MemberPostgresView::Primary {
            readiness: common.readiness.clone(),
            system_identifier: common.system_identifier,
            committed_wal: ObservedWalPosition {
                timeline: common.timeline,
                lsn: *wal_lsn,
            },
        },
        PgInfoState::Replica {
            common,
            replay_lsn,
            follow_lsn,
            upstream,
        } => MemberPostgresView::Replica {
            readiness: common.readiness.clone(),
            system_identifier: common.system_identifier,
            upstream: upstream.as_ref().map(|value| value.member_id.clone()),
            replay_wal: Some(ObservedWalPosition {
                timeline: common.timeline,
                lsn: *replay_lsn,
            }),
            follow_wal: follow_lsn.map(|lsn| ObservedWalPosition {
                timeline: common.timeline,
                lsn,
            }),
        },
    };

    MemberRecord {
        owner: self_id.clone(),
        ttl_ms,
        postgres_target: postgres_target.clone(),
        postgres,
    }
}

// ---------------------------------------------------------------------------
// etcd snapshot & watch
// ---------------------------------------------------------------------------

async fn load_snapshot(
    scope: &str,
    client: &mut Client,
    cache: &mut Cache,
) -> Result<i64, DcsError> {
    let prefix = scope_prefix(scope);
    let response = timeout_etcd(
        "etcd get",
        client.get(prefix.as_str(), Some(GetOptions::new().with_prefix())),
    )
    .await?;
    cache.clear();
    for kv in response.kvs() {
        let path = str::from_utf8(kv.key()).map_err(|err| DcsError::Decode {
            key: "watch-key".to_string(),
            message: err.to_string(),
        })?;
        let value = str::from_utf8(kv.value()).map_err(|err| DcsError::Decode {
            key: path.to_string(),
            message: err.to_string(),
        })?;
        apply_key_value(scope, cache, path, value)?;
    }
    Ok(response
        .header()
        .map(|header| header.revision())
        .unwrap_or_default())
}

fn apply_watch_response(
    scope: &str,
    cache: &mut Cache,
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
        match event.event_type() {
            EventType::Put => {
                let value = str::from_utf8(kv.value()).map_err(|err| DcsError::Decode {
                    key: path.to_string(),
                    message: err.to_string(),
                })?;
                apply_key_value(scope, cache, path, value)?;
            }
            EventType::Delete => {
                apply_delete(scope, cache, path);
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// etcd key parsing
// ---------------------------------------------------------------------------

enum KeyPath {
    Member(MemberId),
    Leader,
    Switchover,
}

fn apply_key_value(
    scope: &str,
    cache: &mut Cache,
    path: &str,
    raw: &str,
) -> Result<(), DcsError> {
    match parse_key(scope, path) {
        Some(KeyPath::Member(member_id)) => {
            let record = serde_json::from_str(raw).map_err(|err| DcsError::Decode {
                key: path.to_string(),
                message: err.to_string(),
            })?;
            cache.members.insert(member_id, record);
        }
        Some(KeyPath::Leader) => {
            let epoch = serde_json::from_str(raw).map_err(|err| DcsError::Decode {
                key: path.to_string(),
                message: err.to_string(),
            })?;
            cache.leader = Some(epoch);
        }
        Some(KeyPath::Switchover) => {
            let target = serde_json::from_str(raw).map_err(|err| DcsError::Decode {
                key: path.to_string(),
                message: err.to_string(),
            })?;
            cache.switchover = Some(target);
        }
        None => {}
    }
    Ok(())
}

fn apply_delete(scope: &str, cache: &mut Cache, path: &str) {
    match parse_key(scope, path) {
        Some(KeyPath::Member(member_id)) => {
            cache.members.remove(&member_id);
        }
        Some(KeyPath::Leader) => {
            cache.leader = None;
        }
        Some(KeyPath::Switchover) => {
            cache.switchover = None;
        }
        None => {}
    }
}

fn parse_key(scope: &str, full_path: &str) -> Option<KeyPath> {
    let scope = scope.trim_matches('/');
    let prefix = format!("/{scope}/");
    let suffix = full_path.strip_prefix(&prefix)?;
    match suffix.split('/').collect::<Vec<_>>().as_slice() {
        ["member", member_id] if !member_id.is_empty() => {
            Some(KeyPath::Member(MemberId((*member_id).to_string())))
        }
        ["leader"] => Some(KeyPath::Leader),
        ["switchover"] => Some(KeyPath::Switchover),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// etcd key paths
// ---------------------------------------------------------------------------

fn scope_prefix(scope: &str) -> String {
    format!("/{}/", scope.trim_matches('/'))
}

fn member_path(scope: &str, member_id: &MemberId) -> String {
    format!("/{}/member/{}", scope.trim_matches('/'), member_id.0)
}

fn leader_path(scope: &str) -> String {
    format!("/{}/leader", scope.trim_matches('/'))
}

fn switchover_path(scope: &str) -> String {
    format!("/{}/switchover", scope.trim_matches('/'))
}

// ---------------------------------------------------------------------------
// Logging helpers
// ---------------------------------------------------------------------------

fn log_identity(ctx: &WorkerCtx) -> DcsLogIdentity {
    DcsLogIdentity {
        scope: ctx.scope.clone(),
        member_id: ctx.self_id.0.clone(),
    }
}

fn dcs_failure(err: &DcsError) -> DcsFailure {
    match err {
        DcsError::AlreadyExists(error) => DcsFailure::AlreadyExists {
            error: error.clone(),
        },
        DcsError::Io(error) => DcsFailure::StoreIo {
            error: error.clone(),
        },
        DcsError::Decode { message, .. } => DcsFailure::Decode {
            error: message.clone(),
        },
    }
}

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

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

async fn connect_client(
    endpoints: &[DcsEndpoint],
    client_config: &DcsClientConfig,
) -> Result<Client, DcsError> {
    let addrs = endpoints
        .iter()
        .map(DcsEndpoint::to_client_string)
        .collect::<Vec<_>>();
    let options = build_connect_options(client_config)?;
    timeout_etcd("etcd connect", Client::connect(addrs, options)).await
}

fn build_connect_options(client: &DcsClientConfig) -> Result<Option<ConnectOptions>, DcsError> {
    let mut options = ConnectOptions::new();
    let mut configured = false;

    if let DcsAuthConfig::Basic { username, password } = &client.auth {
        let resolved = resolve_secret_string("dcs.client.auth.password", password)
            .map_err(|err| DcsError::Io(err.to_string()))?;
        options = options.with_user(username.clone(), resolved);
        configured = true;
    }

    if let DcsTlsConfig::Enabled {
        ca_cert,
        identity,
        server_name,
    } = &client.tls
    {
        let mut tls = TlsOptions::new();
        if let Some(ca_cert) = ca_cert.as_ref() {
            let pem = resolve_inline_or_path_bytes("dcs.client.tls.ca_cert", ca_cert)
                .map_err(|err| DcsError::Io(err.to_string()))?;
            tls = tls.ca_certificate(Certificate::from_pem(pem));
        }
        if let Some(identity) = identity.as_ref() {
            let cert_pem =
                resolve_inline_or_path_bytes("dcs.client.tls.identity.cert", &identity.cert)
                    .map_err(|err| DcsError::Io(err.to_string()))?;
            let key_pem = resolve_secret_string("dcs.client.tls.identity.key", &identity.key)
                .map_err(|err| DcsError::Io(err.to_string()))?;
            tls = tls.identity(Identity::from_pem(cert_pem, key_pem.into_bytes()));
        }
        if let Some(server_name) = server_name.as_ref() {
            tls = tls.domain_name(server_name.clone());
        }
        options = options.with_tls(tls);
        configured = true;
    }

    Ok(configured.then_some(options))
}

async fn timeout_etcd<T, F>(operation: &str, fut: F) -> Result<T, DcsError>
where
    F: std::future::Future<Output = Result<T, etcd_client::Error>>,
{
    match tokio::time::timeout(ETCD_TIMEOUT, fut).await {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(err)) => Err(DcsError::Io(format!("{operation} failed: {err}"))),
        Err(err) => Err(DcsError::Io(format!("{operation} timed out: {err}"))),
    }
}

fn now_unix_millis() -> Result<crate::state::UnixMillis, WorkerError> {
    let elapsed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|err| WorkerError::Message(format!("system clock before unix epoch: {err}")))?;
    let millis = u64::try_from(elapsed.as_millis())
        .map_err(|err| WorkerError::Message(format!("unix millis conversion failed: {err}")))?;
    Ok(crate::state::UnixMillis(millis))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        pginfo::state::{PgInfoState, Readiness},
        state::{LeaseEpoch, MemberId, PgTcpTarget, SystemIdentifier, TimelineId, WalLsn},
    };

    use super::{
        build_dcs_view, build_local_member_record, evaluate_mode, Cache, DcsMode,
        LeadershipObservation, MemberPostgresView, MemberRecord, ObservedWalPosition,
    };

    fn member_record(postgres: MemberPostgresView) -> Result<MemberRecord, String> {
        Ok(MemberRecord {
            owner: MemberId("owner".to_string()),
            ttl_ms: 5_000,
            postgres_target: PgTcpTarget::new("127.0.0.1".to_string(), 5432)?,
            postgres,
        })
    }

    #[test]
    fn build_dcs_view_hides_non_leader_primary_records() -> Result<(), String> {
        let mut members = BTreeMap::new();
        members.insert(
            MemberId("node-a".to_string()),
            member_record(MemberPostgresView::Primary {
                readiness: Readiness::Ready,
                system_identifier: None,
                committed_wal: ObservedWalPosition {
                    timeline: None,
                    lsn: WalLsn(42),
                },
            })?,
        );
        members.insert(
            MemberId("node-b".to_string()),
            member_record(MemberPostgresView::Primary {
                readiness: Readiness::Ready,
                system_identifier: None,
                committed_wal: ObservedWalPosition {
                    timeline: None,
                    lsn: WalLsn(41),
                },
            })?,
        );
        let cache = Cache {
            members,
            leader: Some(LeaseEpoch {
                holder: MemberId("node-a".to_string()),
                generation: 7,
            }),
            switchover: None,
        };

        let cluster = match build_dcs_view(DcsMode::Coordinated, &cache) {
            super::DcsView::Coordinated(cluster) => cluster,
            other => return Err(format!("expected coordinated view, got {other:?}")),
        };

        if cluster.leadership()
            != &LeadershipObservation::Held(LeaseEpoch {
                holder: MemberId("node-a".to_string()),
                generation: 7,
            })
        {
            return Err("expected node-a leadership to remain authoritative".to_string());
        }

        match cluster
            .member(&MemberId("node-a".to_string()))
            .ok_or_else(|| "missing node-a member".to_string())?
            .postgres()
        {
            MemberPostgresView::Primary { .. } => {}
            other => return Err(format!("expected node-a to remain primary, got {other:?}")),
        }

        match cluster
            .member(&MemberId("node-b".to_string()))
            .ok_or_else(|| "missing node-b member".to_string())?
            .postgres()
        {
            MemberPostgresView::Unknown { readiness, .. } if readiness == &Readiness::Ready => {}
            other => {
                return Err(format!(
                    "expected stale non-leader primary to be downgraded, got {other:?}"
                ))
            }
        }

        Ok(())
    }

    #[test]
    fn build_local_member_record_preserves_last_known_identity_when_pg_is_unknown(
    ) -> Result<(), String> {
        let previous = member_record(MemberPostgresView::Primary {
            readiness: Readiness::Ready,
            system_identifier: Some(SystemIdentifier(41)),
            committed_wal: ObservedWalPosition {
                timeline: Some(TimelineId(7)),
                lsn: WalLsn(42),
            },
        })?;
        let pg_state = PgInfoState::Unknown {
            common: crate::pginfo::state::PgInfoCommon {
                worker: crate::state::WorkerStatus::Running,
                sql: crate::pginfo::state::SqlStatus::Unreachable,
                readiness: Readiness::NotReady,
                timeline: None,
                system_identifier: None,
                pg_config: crate::pginfo::state::PgConfig {
                    port: None,
                    hot_standby: None,
                    primary_conninfo: None,
                    primary_slot_name: None,
                    extra: BTreeMap::new(),
                },
                last_refresh_at: None,
            },
        };

        let record = build_local_member_record(
            &MemberId("node-a".to_string()),
            &PgTcpTarget::new("127.0.0.1".to_string(), 5432)?,
            5_000,
            &pg_state,
            Some(&previous),
        );

        match record.postgres {
            MemberPostgresView::Unknown {
                timeline,
                system_identifier,
                ..
            } => {
                if timeline != Some(TimelineId(7)) {
                    return Err(format!("expected preserved timeline, got {timeline:?}"));
                }
                if system_identifier != Some(SystemIdentifier(41)) {
                    return Err(format!(
                        "expected preserved system identifier, got {system_identifier:?}"
                    ));
                }
            }
            other => return Err(format!("expected unknown member record, got {other:?}")),
        }

        Ok(())
    }

    #[test]
    fn evaluate_mode_not_trusted_when_etcd_unreachable() {
        let cache = Cache::empty();
        let mode = evaluate_mode(false, &cache, &MemberId("node-a".to_string()));
        assert_eq!(mode, DcsMode::NotTrusted);
    }

    #[test]
    fn evaluate_mode_degraded_when_self_not_in_members() -> Result<(), String> {
        let cache = Cache::empty();
        let mode = evaluate_mode(true, &cache, &MemberId("node-a".to_string()));
        assert_eq!(mode, DcsMode::Degraded);
        Ok(())
    }

    #[test]
    fn evaluate_mode_coordinated_with_self_in_members() -> Result<(), String> {
        let mut members = BTreeMap::new();
        members.insert(
            MemberId("node-a".to_string()),
            member_record(MemberPostgresView::Unknown {
                readiness: Readiness::NotReady,
                timeline: None,
                system_identifier: None,
            })?,
        );
        let cache = Cache {
            members,
            leader: None,
            switchover: None,
        };
        let mode = evaluate_mode(true, &cache, &MemberId("node-a".to_string()));
        assert_eq!(mode, DcsMode::Coordinated);
        Ok(())
    }
}
