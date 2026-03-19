use std::{str, time::Duration};

use etcd_client::{
    Certificate, Client, Compare, CompareOp, ConnectOptions, EventType, GetOptions, Identity,
    LeaseKeepAliveStream, LeaseKeeper, PutOptions, TlsOptions, Txn, TxnOp, WatchOptions,
    WatchResponse, WatchStream, Watcher,
};
use thiserror::Error;
use tokio::time::{Instant, MissedTickBehavior};

use crate::{
    config::{
        resolve_inline_or_path_bytes, resolve_secret_string, DcsAuthConfig, DcsClientConfig,
        DcsTlsConfig,
    },
    state::{LeaseEpoch, MemberId, NodeIdentity, PgEndpoint, SwitchoverState, WorkerError},
};

use super::{
    command::DcsCommand,
    log_event::DcsLogEvent,
    state::{build_local_member_state, current_snapshot, DcsMemberState, DcsRuntimeCtx},
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

enum CommandDisposition {
    IgnoredWhileDisconnected,
    Applied,
}

pub(super) async fn run(mut ctx: DcsRuntimeCtx) -> Result<(), WorkerError> {
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
                changed = ctx.pg.changed() => {
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
                    let identity = ctx.identity.clone();
                    let advertisement = ctx.advertised_postgres.clone();
                    let member_ttl_ms = ctx.member_ttl_ms;
                    let pg_snapshot = ctx.pg.latest();
                    sync_local_member(
                        &identity,
                        &advertisement,
                        member_ttl_ms,
                        &pg_snapshot,
                        connected,
                        &mut ctx.members,
                        &mut ctx.leadership,
                    )
                    .await
                }
                ConnectedStep::Command(command) => {
                    let identity = ctx.identity.clone();
                    let member_ttl_ms = ctx.member_ttl_ms;
                    handle_connected_command(
                        &identity,
                        member_ttl_ms,
                        connected,
                        &mut ctx.switchover,
                        &mut ctx.leadership,
                        command,
                    )
                    .await
                    .map(|_| ())
                }
                ConnectedStep::Watch(Ok(Some(response))) => apply_watch_response(
                    ctx.identity.scope.as_str(),
                    &mut ctx.members,
                    &mut ctx.leadership,
                    &mut ctx.switchover,
                    response,
                ),
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
                Err(DcsError::LeaderLeaseExpired(cause)) => {
                    match handle_leader_lease_expired(&mut ctx, connected, cause.as_str()).await {
                        Ok(()) => publish_current_view(&mut ctx, true)?,
                        Err(err) => {
                            handle_connected_failure(&mut ctx, connected, &err).await?;
                            session = None;
                            reconnect_at = Instant::now() + RECONNECT_BACKOFF;
                            publish_current_view(&mut ctx, false)?;
                        }
                    }
                }
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
            changed = ctx.pg.changed() => {
                changed.map_err(|err| WorkerError::Message(format!("dcs pg subscriber closed: {err}")))?;
                DisconnectedStep::PgChanged
            }
            command = ctx.command_inbox.recv() => DisconnectedStep::Command(command),
        };

        match step {
            DisconnectedStep::Reconnect => match connect_session(&mut ctx).await {
                Ok(mut connected) => {
                    let identity = ctx.identity.clone();
                    let advertisement = ctx.advertised_postgres.clone();
                    let member_ttl_ms = ctx.member_ttl_ms;
                    let pg_snapshot = ctx.pg.latest();
                    if let Err(err) = sync_local_member(
                        &identity,
                        &advertisement,
                        member_ttl_ms,
                        &pg_snapshot,
                        &mut connected,
                        &mut ctx.members,
                        &mut ctx.leadership,
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
            DisconnectedStep::Command(Some(command)) => {
                handle_disconnected_command(command);
            }
            DisconnectedStep::Command(None) => {
                return Err(WorkerError::Message(
                    "dcs command channel disconnected".to_string(),
                ));
            }
        }
    }
}

async fn connect_session(ctx: &mut DcsRuntimeCtx) -> Result<ConnectedSession, DcsError> {
    let scope_prefix = scope_prefix(ctx.identity.scope.as_str());
    let endpoints = ctx
        .endpoints
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let options = build_connect_options(&ctx.client)?;
    let mut client = timeout_etcd("etcd connect", Client::connect(endpoints, options)).await?;
    let revision = load_snapshot(
        ctx.identity.scope.as_str(),
        &mut client,
        &mut ctx.members,
        &mut ctx.leadership,
        &mut ctx.switchover,
    )
    .await?;
    let start_revision = revision.saturating_add(1);
    let (watcher, watch_stream) = timeout_etcd(
        "etcd watch",
        client.watch(
            scope_prefix.as_str(),
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

async fn sync_local_member(
    identity: &NodeIdentity,
    advertisement: &PgEndpoint,
    member_ttl_ms: u64,
    pg_snapshot: &crate::pginfo::state::PgInfoState,
    session: &mut ConnectedSession,
    members: &mut std::collections::BTreeMap<MemberId, DcsMemberState>,
    leadership: &mut Option<LeaseEpoch>,
) -> Result<(), DcsError> {
    let now = now_unix_millis().map_err(|err| DcsError::Io(err.to_string()))?;
    let local_member_path = member_path(identity.scope.as_str(), &identity.member_id);
    let pg_snapshot_stale = pg_snapshot
        .last_refresh_at()
        .is_none_or(|last_refresh_at| now.0.saturating_sub(last_refresh_at.0) > member_ttl_ms);

    if pg_snapshot_stale {
        timeout_etcd(
            "etcd delete",
            session.client.delete(local_member_path.as_str(), None),
        )
        .await?;
        members.remove(&identity.member_id);
        release_local_leadership(
            session,
            identity.scope.as_str(),
            &identity.member_id,
            leadership,
        )
        .await?;
        return Ok(());
    }

    let local_member = build_local_member_state(advertisement, pg_snapshot);
    let encoded = serde_json::to_string(&local_member).map_err(|err| DcsError::Decode {
        key: local_member_path.clone(),
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
        session
            .client
            .put(local_member_path.as_str(), encoded, Some(options)),
    )
    .await?;
    members.insert(identity.member_id.clone(), local_member);
    Ok(())
}

async fn handle_connected_command(
    identity: &NodeIdentity,
    member_ttl_ms: u64,
    session: &mut ConnectedSession,
    switchover: &mut SwitchoverState,
    leadership: &mut Option<LeaseEpoch>,
    command: DcsCommand,
) -> Result<CommandDisposition, DcsError> {
    match command {
        DcsCommand::AcquireLeadership => {
            acquire_local_leadership(identity, member_ttl_ms, session, leadership).await?;
        }
        DcsCommand::ReleaseLeadership => {
            release_local_leadership(
                session,
                identity.scope.as_str(),
                &identity.member_id,
                leadership,
            )
            .await?;
        }
        DcsCommand::PublishSwitchoverAny => {
            publish_switchover(
                session,
                identity.scope.as_str(),
                switchover,
                SwitchoverState::AnyHealthyReplica,
            )
            .await?;
        }
        DcsCommand::PublishSwitchoverTo(target) => {
            publish_switchover(
                session,
                identity.scope.as_str(),
                switchover,
                SwitchoverState::Specific(target),
            )
            .await?;
        }
        DcsCommand::ClearSwitchover => {
            clear_switchover(session, identity.scope.as_str(), switchover).await?;
        }
    }
    Ok(CommandDisposition::Applied)
}

fn handle_disconnected_command(_command: DcsCommand) -> CommandDisposition {
    CommandDisposition::IgnoredWhileDisconnected
}

async fn acquire_local_leadership(
    identity: &NodeIdentity,
    member_ttl_ms: u64,
    session: &mut ConnectedSession,
    leadership: &mut Option<LeaseEpoch>,
) -> Result<(), DcsError> {
    let path = leader_path(identity.scope.as_str());
    if session
        .leader_lease
        .as_ref()
        .map(|lease| lease.leader_path == path && lease.member_id == identity.member_id)
        .unwrap_or(false)
    {
        return Ok(());
    }

    let epoch = LeaseEpoch {
        holder: identity.member_id.clone(),
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
            .map(|existing_epoch| existing_epoch.holder == identity.member_id)
            .unwrap_or(false)
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
        member_id: identity.member_id.clone(),
        ttl_seconds,
        keeper,
        stream,
        next_keepalive_at: Instant::now() + leader_keepalive_interval(ttl_seconds),
    });
    *leadership = Some(epoch);
    Ok(())
}

async fn release_local_leadership(
    session: &mut ConnectedSession,
    scope: &str,
    self_id: &MemberId,
    leadership: &mut Option<LeaseEpoch>,
) -> Result<(), DcsError> {
    let path = leader_path(scope);
    let Some(lease) = session.leader_lease.take() else {
        if leadership
            .as_ref()
            .map(|epoch| epoch.holder == *self_id)
            .unwrap_or(false)
        {
            *leadership = None;
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
    *leadership = None;
    Ok(())
}

async fn publish_switchover(
    session: &mut ConnectedSession,
    scope: &str,
    switchover: &mut SwitchoverState,
    target: SwitchoverState,
) -> Result<(), DcsError> {
    if switchover == &target {
        return Ok(());
    }
    let path = switchover_path(scope);
    let encoded = serde_json::to_string(&target).map_err(|err| DcsError::Decode {
        key: path.clone(),
        message: err.to_string(),
    })?;
    timeout_etcd("etcd put", session.client.put(path.as_str(), encoded, None)).await?;
    *switchover = target;
    Ok(())
}

async fn clear_switchover(
    session: &mut ConnectedSession,
    scope: &str,
    switchover: &mut SwitchoverState,
) -> Result<(), DcsError> {
    if *switchover == SwitchoverState::None {
        return Ok(());
    }
    let path = switchover_path(scope);
    timeout_etcd("etcd delete", session.client.delete(path.as_str(), None)).await?;
    *switchover = SwitchoverState::None;
    Ok(())
}

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

async fn handle_connected_failure(
    ctx: &mut DcsRuntimeCtx,
    session: &mut ConnectedSession,
    err: &DcsError,
) -> Result<(), WorkerError> {
    if session.leader_lease.is_some() {
        session.leader_lease = None;
    }
    ctx.log
        .send(connected_failure_event(err))
        .map_err(|log_err| {
            WorkerError::Message(format!("dcs watch failure log emit failed: {log_err}"))
        })
}

async fn handle_leader_lease_expired(
    ctx: &mut DcsRuntimeCtx,
    session: &mut ConnectedSession,
    cause: &str,
) -> Result<(), DcsError> {
    session.leader_lease = None;
    ctx.leadership = None;
    ctx.log
        .send(DcsLogEvent::LeaderLeaseExpired {
            cause: cause.to_string(),
        })
        .map_err(|log_err| DcsError::Io(format!("dcs lease-expiry log emit failed: {log_err}")))?;
    load_snapshot(
        ctx.identity.scope.as_str(),
        &mut session.client,
        &mut ctx.members,
        &mut ctx.leadership,
        &mut ctx.switchover,
    )
    .await?;
    Ok(())
}

fn handle_initial_connect_failure(
    ctx: &mut DcsRuntimeCtx,
    err: &DcsError,
) -> Result<(), WorkerError> {
    ctx.log
        .send(initial_connect_failure_event(err))
        .map_err(|log_err| {
            WorkerError::Message(format!("dcs connect failure log emit failed: {log_err}"))
        })
}

fn publish_current_view(ctx: &mut DcsRuntimeCtx, etcd_reachable: bool) -> Result<(), WorkerError> {
    let next = current_snapshot(
        etcd_reachable,
        &ctx.identity.member_id,
        &ctx.leadership,
        &ctx.switchover,
        &ctx.members,
    );
    let next_authority = next.authority();
    if ctx.last_emitted_authority != Some(next_authority) {
        let previous = ctx.last_emitted_authority;
        ctx.last_emitted_authority = Some(next_authority);
        ctx.log
            .send(DcsLogEvent::CoordinationModeTransition {
                previous: previous.map(|authority| authority.to_string()),
                next: next_authority.to_string(),
            })
            .map_err(|err| {
                WorkerError::Message(format!("dcs coordination mode log emit failed: {err}"))
            })?;
    }
    ctx.publisher
        .publish(next)
        .map_err(|err| WorkerError::Message(format!("dcs publish failed: {err}")))
}

fn connected_failure_event(err: &DcsError) -> DcsLogEvent {
    match err {
        DcsError::Io(cause) => DcsLogEvent::ConnectedStepStoreIoFailed {
            cause: cause.clone(),
        },
        DcsError::LeaderLeaseExpired(cause) => DcsLogEvent::LeaderLeaseExpired {
            cause: cause.clone(),
        },
        DcsError::Decode { key, message } => DcsLogEvent::ConnectedStepDecodeFailed {
            cause: format!("key `{key}` decode failed: {message}"),
        },
        DcsError::AlreadyExists(cause) => DcsLogEvent::ConnectedStepAlreadyExists {
            cause: cause.clone(),
        },
    }
}

fn initial_connect_failure_event(err: &DcsError) -> DcsLogEvent {
    match err {
        DcsError::Io(cause) => DcsLogEvent::InitialConnectStoreIoFailed {
            cause: cause.clone(),
        },
        DcsError::LeaderLeaseExpired(cause) => DcsLogEvent::LeaderLeaseExpired {
            cause: cause.clone(),
        },
        DcsError::Decode { key, message } => DcsLogEvent::InitialConnectDecodeFailed {
            cause: format!("key `{key}` decode failed: {message}"),
        },
        DcsError::AlreadyExists(cause) => DcsLogEvent::InitialConnectAlreadyExists {
            cause: cause.clone(),
        },
    }
}

async fn load_snapshot(
    scope: &str,
    client: &mut Client,
    members: &mut std::collections::BTreeMap<MemberId, DcsMemberState>,
    leadership: &mut Option<LeaseEpoch>,
    switchover: &mut SwitchoverState,
) -> Result<i64, DcsError> {
    let prefix = scope_prefix(scope);
    let response = timeout_etcd(
        "etcd get",
        client.get(prefix.as_str(), Some(GetOptions::new().with_prefix())),
    )
    .await?;
    members.clear();
    *leadership = None;
    *switchover = SwitchoverState::None;
    for kv in response.kvs() {
        let path = str::from_utf8(kv.key()).map_err(|err| DcsError::Decode {
            key: "watch-key".to_string(),
            message: err.to_string(),
        })?;
        let value = str::from_utf8(kv.value()).map_err(|err| DcsError::Decode {
            key: path.to_string(),
            message: err.to_string(),
        })?;
        apply_key_value(scope, members, leadership, switchover, path, value)?;
    }
    Ok(response
        .header()
        .map(|header| header.revision())
        .unwrap_or_default())
}

fn apply_watch_response(
    scope: &str,
    members: &mut std::collections::BTreeMap<MemberId, DcsMemberState>,
    leadership: &mut Option<LeaseEpoch>,
    switchover: &mut SwitchoverState,
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
                apply_key_value(scope, members, leadership, switchover, path, value)?;
            }
            EventType::Delete => {
                apply_delete(scope, members, leadership, switchover, path);
            }
        }
    }
    Ok(())
}

fn apply_key_value(
    scope: &str,
    members: &mut std::collections::BTreeMap<MemberId, DcsMemberState>,
    leadership: &mut Option<LeaseEpoch>,
    switchover: &mut SwitchoverState,
    path: &str,
    raw: &str,
) -> Result<(), DcsError> {
    match parse_key(scope, path) {
        Some(KeyPath::Member(member_id)) => {
            let record: DcsMemberState =
                serde_json::from_str(raw).map_err(|err| DcsError::Decode {
                    key: path.to_string(),
                    message: err.to_string(),
                })?;
            members.insert(member_id, record);
        }
        Some(KeyPath::Leader) => {
            let epoch: LeaseEpoch = serde_json::from_str(raw).map_err(|err| DcsError::Decode {
                key: path.to_string(),
                message: err.to_string(),
            })?;
            *leadership = Some(epoch);
        }
        Some(KeyPath::Switchover) => {
            let record: SwitchoverState =
                serde_json::from_str(raw).map_err(|err| DcsError::Decode {
                    key: path.to_string(),
                    message: err.to_string(),
                })?;
            *switchover = record;
        }
        None => {}
    }
    Ok(())
}

fn apply_delete(
    scope: &str,
    members: &mut std::collections::BTreeMap<MemberId, DcsMemberState>,
    leadership: &mut Option<LeaseEpoch>,
    switchover: &mut SwitchoverState,
    path: &str,
) {
    match parse_key(scope, path) {
        Some(KeyPath::Member(member_id)) => {
            members.remove(&member_id);
        }
        Some(KeyPath::Leader) => {
            *leadership = None;
        }
        Some(KeyPath::Switchover) => {
            *switchover = SwitchoverState::None;
        }
        None => {}
    }
}

enum KeyPath {
    Member(MemberId),
    Leader,
    Switchover,
}

fn parse_key(scope: &str, full_path: &str) -> Option<KeyPath> {
    let scope = scope.trim_matches('/');
    let prefix = format!("/{scope}/");
    if !full_path.starts_with(&prefix) {
        return None;
    }
    let suffix = &full_path[prefix.len()..];
    match suffix.split('/').collect::<Vec<_>>().as_slice() {
        ["member", member_id] if !member_id.is_empty() => {
            Some(KeyPath::Member(MemberId((*member_id).to_string())))
        }
        ["leader"] => Some(KeyPath::Leader),
        ["switchover"] => Some(KeyPath::Switchover),
        _ => None,
    }
}

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
