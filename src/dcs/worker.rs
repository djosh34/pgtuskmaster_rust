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
        DcsEndpoint, DcsTlsConfig,
    },
    state::{LeaseEpoch, MemberId, SwitchoverTarget, WorkerError},
};

use super::{
    command::{dcs_command_channel, DcsCommand, DcsHandle},
    log_event::{DcsFailure, DcsLogEvent, DcsLogIdentity, DcsLogOrigin},
    state::{
        build_dcs_view, build_local_member_record, evaluate_mode, DcsCadence, DcsControlPlane,
        DcsEtcdConfig, DcsLocalMemberAdvertisement, DcsNodeIdentity, DcsObservedState,
        DcsRuntime, DcsStateChannel, DcsWorkerCtx, LeadershipRecord, SwitchoverRecord,
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
}

pub(crate) struct DcsWorkerBootstrap {
    pub(crate) identity: DcsNodeIdentity,
    pub(crate) etcd: DcsEtcdConfig,
    pub(crate) cadence: DcsCadence,
    pub(crate) advertisement: super::state::DcsLocalMemberAdvertisement,
    pub(crate) observed: DcsObservedState,
    pub(crate) state_channel: DcsStateChannel,
    pub(crate) runtime: DcsRuntime,
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

pub(crate) fn build_worker_ctx(bootstrap: DcsWorkerBootstrap) -> (DcsWorkerCtx, DcsHandle) {
    let (handle, command_inbox) = dcs_command_channel();
    let DcsWorkerBootstrap {
        identity,
        etcd,
        cadence,
        advertisement,
        observed,
        state_channel,
        runtime,
    } = bootstrap;
    (
        DcsWorkerCtx {
            identity,
            etcd,
            cadence,
            advertisement,
            observed,
            state_channel,
            control: DcsControlPlane { command_inbox },
            runtime,
        },
        handle,
    )
}

pub(crate) async fn run(mut ctx: DcsWorkerCtx) -> Result<(), WorkerError> {
    let mut reconnect_at = Instant::now();
    let mut session = None::<ConnectedSession>;
    let mut tick = tokio::time::interval(ctx.cadence.poll_interval);
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
                changed = ctx.observed.pg.changed() => {
                    changed.map_err(|err| WorkerError::Message(format!("dcs pg subscriber closed: {err}")))?;
                    ConnectedStep::PgChanged
                }
                command = ctx.control.command_inbox.recv() => {
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
                    let advertisement = ctx.advertisement.clone();
                    let member_ttl_ms = ctx.cadence.member_ttl_ms;
                    let pg_snapshot = ctx.observed.pg.latest();
                    sync_local_member(
                        &identity,
                        &advertisement,
                        member_ttl_ms,
                        &pg_snapshot,
                        connected,
                        &mut ctx.state_channel.cache,
                    )
                    .await
                }
                ConnectedStep::Command(command) => {
                    let identity = ctx.identity.clone();
                    let member_ttl_ms = ctx.cadence.member_ttl_ms;
                    handle_connected_command(
                        &identity,
                        member_ttl_ms,
                        connected,
                        &mut ctx.state_channel.cache,
                        command,
                    )
                    .await
                    .map(|_| ())
                }
                ConnectedStep::Watch(Ok(Some(response))) => {
                    apply_watch_response(&ctx.identity.scope, &mut ctx.state_channel.cache, response)
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
            changed = ctx.observed.pg.changed() => {
                changed.map_err(|err| WorkerError::Message(format!("dcs pg subscriber closed: {err}")))?;
                DisconnectedStep::PgChanged
            }
            command = ctx.control.command_inbox.recv() => DisconnectedStep::Command(command),
        };

        match step {
            DisconnectedStep::Reconnect => match connect_session(&mut ctx).await {
                Ok(mut connected) => {
                    let identity = ctx.identity.clone();
                    let advertisement = ctx.advertisement.clone();
                    let member_ttl_ms = ctx.cadence.member_ttl_ms;
                    let pg_snapshot = ctx.observed.pg.latest();
                    if let Err(err) = sync_local_member(
                        &identity,
                        &advertisement,
                        member_ttl_ms,
                        &pg_snapshot,
                        &mut connected,
                        &mut ctx.state_channel.cache,
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

async fn connect_session(ctx: &mut DcsWorkerCtx) -> Result<ConnectedSession, DcsError> {
    let scope_prefix = scope_prefix(&ctx.identity.scope);
    let mut client = connect_client(&ctx.etcd).await?;
    let revision = load_snapshot(&ctx.identity.scope, &mut client, &mut ctx.state_channel.cache).await?;
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
    identity: &DcsNodeIdentity,
    advertisement: &DcsLocalMemberAdvertisement,
    member_ttl_ms: u64,
    pg_snapshot: &crate::pginfo::state::PgInfoState,
    session: &mut ConnectedSession,
    cache: &mut super::state::DcsCache,
) -> Result<(), DcsError> {
    let now = now_unix_millis().map_err(|err| DcsError::Io(err.to_string()))?;
    let local_member_path = member_path(&identity.scope, &identity.self_id);
    let pg_snapshot_stale = pg_snapshot.last_refresh_at().is_none_or(|last_refresh_at| {
        now.0.saturating_sub(last_refresh_at.0) > member_ttl_ms
    });

    if pg_snapshot_stale {
        timeout_etcd(
            "etcd delete",
            session.client.delete(local_member_path.as_str(), None),
        )
        .await?;
        cache.member_records.remove(&identity.self_id);
        release_local_leadership(session, &identity.scope, &identity.self_id, cache).await?;
        return Ok(());
    }

    let local_member = build_local_member_record(
        &identity.self_id,
        &advertisement.postgres,
        member_ttl_ms,
        pg_snapshot,
        cache.member_records.get(&identity.self_id),
    );
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
    cache
        .member_records
        .insert(identity.self_id.clone(), local_member);
    Ok(())
}

async fn handle_connected_command(
    identity: &DcsNodeIdentity,
    member_ttl_ms: u64,
    session: &mut ConnectedSession,
    cache: &mut super::state::DcsCache,
    command: DcsCommand,
) -> Result<CommandDisposition, DcsError> {
    match command {
        DcsCommand::AcquireLeadership => {
            acquire_local_leadership(identity, member_ttl_ms, session, cache).await?;
        }
        DcsCommand::ReleaseLeadership => {
            release_local_leadership(session, &identity.scope, &identity.self_id, cache)
                .await?;
        }
        DcsCommand::PublishSwitchoverAny => {
            publish_switchover(
                session,
                &identity.scope,
                cache,
                SwitchoverTarget::AnyHealthyReplica,
            )
            .await?;
        }
        DcsCommand::PublishSwitchoverTo(target) => {
            publish_switchover(
                session,
                &identity.scope,
                cache,
                SwitchoverTarget::Specific(target),
            )
            .await?;
        }
        DcsCommand::ClearSwitchover => {
            clear_switchover(session, &identity.scope, cache).await?;
        }
    }
    Ok(CommandDisposition::Applied)
}

fn handle_disconnected_command(_command: DcsCommand) -> CommandDisposition {
    CommandDisposition::IgnoredWhileDisconnected
}

async fn acquire_local_leadership(
    identity: &DcsNodeIdentity,
    member_ttl_ms: u64,
    session: &mut ConnectedSession,
    cache: &mut super::state::DcsCache,
) -> Result<(), DcsError> {
    let path = leader_path(&identity.scope);
    if session
        .leader_lease
        .as_ref()
        .map(|lease| lease.leader_path == path && lease.member_id == identity.self_id)
        .unwrap_or(false)
    {
        return Ok(());
    }

    let epoch = LeaseEpoch {
        holder: identity.self_id.clone(),
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
            .map(|existing_epoch| existing_epoch.holder == identity.self_id)
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
        member_id: identity.self_id.clone(),
        ttl_seconds,
        keeper,
        stream,
        next_keepalive_at: Instant::now() + leader_keepalive_interval(ttl_seconds),
    });
    cache.leader_record = Some(LeadershipRecord { epoch });
    Ok(())
}

async fn release_local_leadership(
    session: &mut ConnectedSession,
    scope: &str,
    self_id: &MemberId,
    cache: &mut super::state::DcsCache,
) -> Result<(), DcsError> {
    let path = leader_path(scope);
    let Some(lease) = session.leader_lease.take() else {
        if cache
            .leader_record
            .as_ref()
            .map(|record| record.epoch.holder == *self_id)
            .unwrap_or(false)
        {
            cache.leader_record = None;
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
    cache.leader_record = None;
    Ok(())
}

async fn publish_switchover(
    session: &mut ConnectedSession,
    scope: &str,
    cache: &mut super::state::DcsCache,
    target: SwitchoverTarget,
) -> Result<(), DcsError> {
    if cache
        .switchover_record
        .as_ref()
        .map(|record| record.target == target)
        .unwrap_or(false)
    {
        return Ok(());
    }
    let path = switchover_path(scope);
    let record = SwitchoverRecord {
        target: target.clone(),
    };
    let encoded = serde_json::to_string(&record).map_err(|err| DcsError::Decode {
        key: path.clone(),
        message: err.to_string(),
    })?;
    timeout_etcd("etcd put", session.client.put(path.as_str(), encoded, None)).await?;
    cache.switchover_record = Some(record);
    Ok(())
}

async fn clear_switchover(
    session: &mut ConnectedSession,
    scope: &str,
    cache: &mut super::state::DcsCache,
) -> Result<(), DcsError> {
    if cache.switchover_record.is_none() {
        return Ok(());
    }
    let path = switchover_path(scope);
    timeout_etcd("etcd delete", session.client.delete(path.as_str(), None)).await?;
    cache.switchover_record = None;
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

async fn handle_connected_failure(
    ctx: &mut DcsWorkerCtx,
    session: &mut ConnectedSession,
    err: &DcsError,
) -> Result<(), WorkerError> {
    if session.leader_lease.is_some() {
        session.leader_lease = None;
    }
    ctx.runtime
        .log
        .send(DcsLogEvent::ConnectedStepFailed {
            origin: DcsLogOrigin::ConnectedFailure,
            identity: dcs_event_identity(ctx),
            failure: dcs_failure(err),
        })
        .map_err(|log_err| WorkerError::Message(format!("dcs watch failure log emit failed: {log_err}")))
}

fn handle_initial_connect_failure(ctx: &mut DcsWorkerCtx, err: &DcsError) -> Result<(), WorkerError> {
    ctx.runtime
        .log
        .send(DcsLogEvent::InitialConnectFailed {
            origin: DcsLogOrigin::InitialConnectFailure,
            identity: dcs_event_identity(ctx),
            failure: dcs_failure(err),
        })
        .map_err(|log_err| WorkerError::Message(format!("dcs connect failure log emit failed: {log_err}")))
}

fn publish_current_view(ctx: &mut DcsWorkerCtx, etcd_reachable: bool) -> Result<(), WorkerError> {
    let mode = evaluate_mode(etcd_reachable, &ctx.state_channel.cache, &ctx.identity.self_id);
    let next = if etcd_reachable {
        build_dcs_view(mode, &ctx.state_channel.cache)
    } else {
        build_dcs_view(super::state::DcsMode::NotTrusted, &ctx.state_channel.cache)
    };
    if ctx.runtime.last_emitted_mode != Some(next.mode()) {
        let previous = ctx.runtime.last_emitted_mode;
        let next_mode = next.mode();
        ctx.runtime.last_emitted_mode = Some(next_mode);
        ctx.runtime
            .log
            .send(DcsLogEvent::CoordinationModeTransition {
                origin: DcsLogOrigin::PublishCurrentView,
                identity: dcs_event_identity(ctx),
                previous,
                next: next_mode,
            })
            .map_err(|err| WorkerError::Message(format!("dcs coordination mode log emit failed: {err}")))?;
    }
    ctx.state_channel
        .publisher
        .publish(next)
        .map_err(|err| WorkerError::Message(format!("dcs publish failed: {err}")))
}

async fn load_snapshot(
    scope: &str,
    client: &mut Client,
    cache: &mut super::state::DcsCache,
) -> Result<i64, DcsError> {
    let prefix = scope_prefix(scope);
    let response = timeout_etcd(
        "etcd get",
        client.get(prefix.as_str(), Some(GetOptions::new().with_prefix())),
    )
    .await?;
    cache.member_records.clear();
    cache.leader_record = None;
    cache.switchover_record = None;
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
    cache: &mut super::state::DcsCache,
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

fn apply_key_value(
    scope: &str,
    cache: &mut super::state::DcsCache,
    path: &str,
    raw: &str,
) -> Result<(), DcsError> {
    match parse_key(scope, path) {
        Some(KeyPath::Member(member_id)) => {
            let record = serde_json::from_str(raw).map_err(|err| DcsError::Decode {
                key: path.to_string(),
                message: err.to_string(),
            })?;
            cache.member_records.insert(member_id, record);
        }
        Some(KeyPath::Leader) => {
            let epoch = serde_json::from_str(raw).map_err(|err| DcsError::Decode {
                key: path.to_string(),
                message: err.to_string(),
            })?;
            cache.leader_record = Some(LeadershipRecord { epoch });
        }
        Some(KeyPath::Switchover) => {
            let record = serde_json::from_str(raw).map_err(|err| DcsError::Decode {
                key: path.to_string(),
                message: err.to_string(),
            })?;
            cache.switchover_record = Some(record);
        }
        None => {}
    }
    Ok(())
}

fn apply_delete(scope: &str, cache: &mut super::state::DcsCache, path: &str) {
    match parse_key(scope, path) {
        Some(KeyPath::Member(member_id)) => {
            cache.member_records.remove(&member_id);
        }
        Some(KeyPath::Leader) => {
            cache.leader_record = None;
        }
        Some(KeyPath::Switchover) => {
            cache.switchover_record = None;
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

fn dcs_event_identity(ctx: &DcsWorkerCtx) -> DcsLogIdentity {
    DcsLogIdentity {
        scope: ctx.identity.scope.clone(),
        member_id: ctx.identity.self_id.0.clone(),
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

async fn connect_client(config: &DcsEtcdConfig) -> Result<Client, DcsError> {
    let endpoints = config
        .endpoints
        .iter()
        .map(DcsEndpoint::to_client_string)
        .collect::<Vec<_>>();
    let options = build_connect_options(&config.client)?;
    timeout_etcd("etcd connect", Client::connect(endpoints, options)).await
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
