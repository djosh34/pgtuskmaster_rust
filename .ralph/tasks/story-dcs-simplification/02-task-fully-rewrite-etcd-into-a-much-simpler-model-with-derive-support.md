## Task: Fully Rewrite Etcd Into A Much Simpler Model With `#[]` Derive Support <status>not_started</status> <passes>false</passes>

Yes. You are right.

`etcd.rs` is still too ugly because I kept too much transport detail visible instead of pushing it behind the generated schema boundary.

If this is *your* `#[]` syntax and *your* conventions, then you do not need generic low-level plumbing all over the crate. You can absolutely make this mostly disappear.

The right model is:

you declare the schema once,
the macro generates the repository metadata,
the runtime uses a single generic typed store,
`Some(x)` means put JSON,
`None` means delete key,
`lease(...)` on a field means attach lease policy,
`watch` is derived from the schema prefix and key layout.

That means the handwritten DCS code should shrink to roughly:

`schema.rs`
`runtime.rs`
`startup.rs`
maybe one tiny generic `store.rs` shared by all schemas

That is it.

Here is the version you actually want.

---

## `mod.rs`

```rust
mod runtime;
mod schema;
pub(crate) mod startup;

pub(crate) use runtime::{DcsHandle, DcsWorker};
pub use schema::{
    ClusterMemberView, ClusterView, DcsMode, DcsView, LeadershipObservation, MemberPostgresView,
    NotTrustedView, SwitchoverView,
};
```

---

## `schema.rs`

This is the only handwritten schema declaration.

```rust
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    pginfo::state::Readiness,
    state::{
        LeaseEpoch, MemberId, ObservedWalPosition, PgTcpTarget, SwitchoverTarget,
        SystemIdentifier, TimelineId, UnixMillis,
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DcsMode {
    NotTrusted,
    Degraded,
    Coordinated,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EtcdSchema)]
#[etcd(prefix = "/{scope}")]
pub(crate) struct DcsState {
    #[etcd(singleton = "leader", lease(kind = "ephemeral", holder = "self"))]
    pub(crate) leader: Option<LeaseEpoch>,

    #[etcd(singleton = "switchover")]
    pub(crate) switchover: Option<SwitchoverTarget>,

    #[etcd(map = "members", key = "MemberId", lease(kind = "ttl", from = "expires_at"))]
    pub(crate) members: BTreeMap<MemberId, MemberRecord>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct MemberRecord {
    pub(crate) expires_at: UnixMillis,
    pub(crate) postgres_target: PgTcpTarget,
    pub(crate) postgres: MemberPostgresRecord,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum MemberPostgresRecord {
    Unknown {
        readiness: Readiness,
        timeline: Option<TimelineId>,
        system_identifier: Option<SystemIdentifier>,
    },
    Primary {
        readiness: Readiness,
        system_identifier: Option<SystemIdentifier>,
        committed_wal: ObservedWalPosition,
    },
    Replica {
        readiness: Readiness,
        system_identifier: Option<SystemIdentifier>,
        upstream: Option<MemberId>,
        replay_wal: Option<ObservedWalPosition>,
        follow_wal: Option<ObservedWalPosition>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DcsView {
    NotTrusted(NotTrustedView),
    Degraded(ClusterView),
    Coordinated(ClusterView),
}

impl DcsView {
    pub fn mode(&self) -> DcsMode {
        match self {
            Self::NotTrusted(_) => DcsMode::NotTrusted,
            Self::Degraded(_) => DcsMode::Degraded,
            Self::Coordinated(_) => DcsMode::Coordinated,
        }
    }

    pub fn observed_leadership(&self) -> Option<&LeaseEpoch> {
        match self {
            Self::NotTrusted(view) => view.observed_leadership(),
            Self::Degraded(view) | Self::Coordinated(view) => view.leadership().held(),
        }
    }

    pub fn cluster(&self) -> &ClusterView {
        match self {
            Self::NotTrusted(view) => view.cluster(),
            Self::Degraded(view) | Self::Coordinated(view) => view,
        }
    }

    pub(crate) fn starting() -> Self {
        Self::NotTrusted(NotTrustedView {
            observed_leadership: None,
            cluster: ClusterView {
                members: BTreeMap::new(),
                leadership: LeadershipObservation::Open,
                switchover: SwitchoverView::None,
            },
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotTrustedView {
    observed_leadership: Option<LeaseEpoch>,
    cluster: ClusterView,
}

impl NotTrustedView {
    pub fn observed_leadership(&self) -> Option<&LeaseEpoch> {
        self.observed_leadership.as_ref()
    }

    pub fn cluster(&self) -> &ClusterView {
        &self.cluster
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterView {
    members: BTreeMap<MemberId, ClusterMemberView>,
    leadership: LeadershipObservation,
    switchover: SwitchoverView,
}

impl ClusterView {
    pub fn members(&self) -> impl Iterator<Item = (&MemberId, &ClusterMemberView)> {
        self.members.iter()
    }

    pub fn member(&self, member_id: &MemberId) -> Option<&ClusterMemberView> {
        self.members.get(member_id)
    }

    pub fn leadership(&self) -> &LeadershipObservation {
        &self.leadership
    }

    pub fn switchover(&self) -> &SwitchoverView {
        &self.switchover
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterMemberView {
    postgres: MemberPostgresView,
    postgres_target: PgTcpTarget,
}

impl ClusterMemberView {
    pub fn postgres_target(&self) -> &PgTcpTarget {
        &self.postgres_target
    }

    pub fn postgres(&self) -> &MemberPostgresView {
        &self.postgres
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MemberPostgresView {
    Unknown {
        readiness: Readiness,
        timeline: Option<TimelineId>,
        system_identifier: Option<SystemIdentifier>,
    },
    Primary {
        readiness: Readiness,
        system_identifier: Option<SystemIdentifier>,
        committed_wal: ObservedWalPosition,
    },
    Replica {
        readiness: Readiness,
        system_identifier: Option<SystemIdentifier>,
        upstream: Option<MemberId>,
        replay_wal: Option<ObservedWalPosition>,
        follow_wal: Option<ObservedWalPosition>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeadershipObservation {
    Open,
    Held(LeaseEpoch),
}

impl LeadershipObservation {
    pub fn held(&self) -> Option<&LeaseEpoch> {
        match self {
            Self::Open => None,
            Self::Held(epoch) => Some(epoch),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "state", content = "target")]
pub enum SwitchoverView {
    None,
    Requested(SwitchoverTarget),
}
```

---

## `runtime.rs`

This is the small handwritten runtime. No path code. No JSON code. No timeout labels. No raw etcd txn code.

That all lives in the generic store generated from `EtcdSchema`.

```rust
use tokio::time::{Duration, MissedTickBehavior};

use crate::{
    logging::LogSender,
    pginfo::state::PgInfoState,
    state::{
        new_state_channel, LeaseEpoch, MemberId, NodeIdentity, PgTcpTarget, StatePublisher,
        StateSubscriber, SwitchoverTarget, UnixMillis, WorkerError,
    },
};

use super::schema::{
    ClusterMemberView, ClusterView, DcsMode, DcsState, DcsView, LeadershipObservation,
    MemberPostgresRecord, MemberPostgresView, MemberRecord, NotTrustedView, SwitchoverView,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DcsCommand {
    RefreshLocalMember(PgInfoState),
    RemoveLocalMember,
    AcquireLeadership,
    ReleaseLeadership,
    PublishSwitchover(SwitchoverTarget),
    ClearSwitchover,
    Reload,
}

#[derive(Clone)]
pub(crate) struct DcsHandle {
    tx: tokio::sync::mpsc::UnboundedSender<DcsCommand>,
}

pub(crate) struct DcsRuntime {
    pub(crate) state: StateSubscriber<DcsView>,
    pub(crate) handle: DcsHandle,
    pub(crate) worker: DcsWorker,
}

pub(crate) struct DcsWorker {
    ctx: DcsWorkerCtx,
}

struct DcsWorkerCtx {
    identity: NodeIdentity,
    advertised_postgres: PgTcpTarget,
    member_ttl_ms: u64,
    repo: TypedEtcdRepo<DcsState>,
    cache: DcsState,
    state_tx: StatePublisher<DcsView>,
    rx: tokio::sync::mpsc::UnboundedReceiver<DcsCommand>,
    _log: LogSender,
}

pub(crate) struct DcsRuntimeRequest {
    pub(crate) identity: NodeIdentity,
    pub(crate) advertised_postgres: PgTcpTarget,
    pub(crate) member_ttl_ms: u64,
    pub(crate) repo: TypedEtcdRepo<DcsState>,
    pub(crate) log: LogSender,
}

pub(crate) fn bootstrap_runtime(request: DcsRuntimeRequest) -> DcsRuntime {
    let (state_tx, state) = new_state_channel(DcsView::starting());
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    DcsRuntime {
        state,
        handle: DcsHandle { tx },
        worker: DcsWorker {
            ctx: DcsWorkerCtx {
                identity: request.identity,
                advertised_postgres: request.advertised_postgres,
                member_ttl_ms: request.member_ttl_ms,
                repo: request.repo,
                cache: DcsState {
                    leader: None,
                    switchover: None,
                    members: Default::default(),
                },
                state_tx,
                rx,
                _log: request.log,
            },
        },
    }
}

impl DcsHandle {
    pub(crate) fn refresh_local_member(&self, pg: PgInfoState) -> Result<(), ()> {
        self.tx.send(DcsCommand::RefreshLocalMember(pg)).map_err(|_| ())
    }

    pub(crate) fn remove_local_member(&self) -> Result<(), ()> {
        self.tx.send(DcsCommand::RemoveLocalMember).map_err(|_| ())
    }

    pub(crate) fn acquire_leadership(&self) -> Result<(), ()> {
        self.tx.send(DcsCommand::AcquireLeadership).map_err(|_| ())
    }

    pub(crate) fn release_leadership(&self) -> Result<(), ()> {
        self.tx.send(DcsCommand::ReleaseLeadership).map_err(|_| ())
    }

    pub(crate) fn publish_switchover(&self, target: SwitchoverTarget) -> Result<(), ()> {
        self.tx.send(DcsCommand::PublishSwitchover(target)).map_err(|_| ())
    }

    pub(crate) fn clear_switchover(&self) -> Result<(), ()> {
        self.tx.send(DcsCommand::ClearSwitchover).map_err(|_| ())
    }

    pub(crate) fn reload(&self) -> Result<(), ()> {
        self.tx.send(DcsCommand::Reload).map_err(|_| ())
    }
}

impl DcsWorker {
    pub(crate) async fn run(mut self) -> Result<(), WorkerError> {
        let scope = self.ctx.identity.scope.0.clone();
        let mut session = self
            .ctx
            .repo
            .session(scope)
            .await
            .map_err(|e| WorkerError::Message(e.to_string()))?;

        self.ctx.cache = session.load().await.map_err(to_worker_error)?;
        publish_view(&mut self.ctx, true)?;

        let mut reconnect = tokio::time::interval(Duration::from_secs(1));
        reconnect.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                maybe_cmd = self.ctx.rx.recv() => {
                    let Some(cmd) = maybe_cmd else {
                        return Err(WorkerError::Message("dcs command channel closed".to_string()));
                    };

                    apply_command(&mut self.ctx, &mut session, cmd).await?;
                    self.ctx.cache = session.load().await.map_err(to_worker_error)?;
                    publish_view(&mut self.ctx, true)?;
                }

                watch = session.next() => {
                    match watch {
                        Ok(Some(state)) => {
                            self.ctx.cache = state;
                            publish_view(&mut self.ctx, true)?;
                        }
                        Ok(None) => {
                            publish_view(&mut self.ctx, false)?;
                            reconnect.tick().await;
                            self.ctx.cache = session.load().await.map_err(to_worker_error)?;
                            publish_view(&mut self.ctx, true)?;
                        }
                        Err(err) => {
                            publish_view(&mut self.ctx, false)?;
                            reconnect.tick().await;
                            return Err(to_worker_error(err));
                        }
                    }
                }
            }
        }
    }
}

async fn apply_command(
    ctx: &mut DcsWorkerCtx,
    session: &mut TypedEtcdSession<DcsState>,
    cmd: DcsCommand,
) -> Result<(), WorkerError> {
    match cmd {
        DcsCommand::RefreshLocalMember(pg_state) => {
            let record = build_local_member_record(
                now_unix_millis()?,
                &ctx.advertised_postgres,
                ctx.member_ttl_ms,
                &pg_state,
                ctx.cache.members.get(&ctx.identity.member_id),
            );
            session
                .field_mut(|s| &mut s.members)
                .put(ctx.identity.member_id.clone(), record)
                .await
                .map_err(to_worker_error)?;
        }
        DcsCommand::RemoveLocalMember => {
            session
                .field_mut(|s| &mut s.members)
                .delete(&ctx.identity.member_id)
                .await
                .map_err(to_worker_error)?;

            session
                .field_mut(|s| &mut s.leader)
                .delete_if(|epoch| epoch.holder == ctx.identity.member_id)
                .await
                .map_err(to_worker_error)?;
        }
        DcsCommand::AcquireLeadership => {
            let epoch = LeaseEpoch {
                holder: ctx.identity.member_id.clone(),
                generation: now_unix_millis()?.0,
            };

            let _ = session
                .field_mut(|s| &mut s.leader)
                .claim_if_empty(epoch)
                .await
                .map_err(to_worker_error)?;
        }
        DcsCommand::ReleaseLeadership => {
            session
                .field_mut(|s| &mut s.leader)
                .delete_if(|epoch| epoch.holder == ctx.identity.member_id)
                .await
                .map_err(to_worker_error)?;
        }
        DcsCommand::PublishSwitchover(target) => {
            session
                .field_mut(|s| &mut s.switchover)
                .set(Some(target))
                .await
                .map_err(to_worker_error)?;
        }
        DcsCommand::ClearSwitchover => {
            session
                .field_mut(|s| &mut s.switchover)
                .set(None)
                .await
                .map_err(to_worker_error)?;
        }
        DcsCommand::Reload => {}
    }

    Ok(())
}

fn publish_view(ctx: &mut DcsWorkerCtx, etcd_reachable: bool) -> Result<(), WorkerError> {
    let now = now_unix_millis()?;
    let mode = evaluate_mode(etcd_reachable, &ctx.cache, &ctx.identity.member_id, now);
    let view = build_dcs_view(
        if etcd_reachable { mode } else { DcsMode::NotTrusted },
        &ctx.cache,
        now,
    );

    ctx.state_tx
        .publish(view)
        .map_err(|err| WorkerError::Message(format!("dcs publish failed: {err}")))
}

fn evaluate_mode(
    etcd_reachable: bool,
    state: &DcsState,
    self_id: &MemberId,
    now: UnixMillis,
) -> DcsMode {
    if !etcd_reachable {
        return DcsMode::NotTrusted;
    }

    let active_members = state
        .members
        .iter()
        .filter(|(_, m)| m.expires_at.0 > now.0)
        .count();

    if !state
        .members
        .get(self_id)
        .map(|m| m.expires_at.0 > now.0)
        .unwrap_or(false)
    {
        return DcsMode::Degraded;
    }

    if active_members < 1 {
        return DcsMode::Degraded;
    }

    DcsMode::Coordinated
}

fn build_dcs_view(mode: DcsMode, state: &DcsState, now: UnixMillis) -> DcsView {
    let authoritative_leader = state.leader.as_ref().map(|x| x.holder.clone());

    let cluster = ClusterView {
        members: state
            .members
            .iter()
            .filter(|(_, record)| record.expires_at.0 > now.0)
            .map(|(member_id, record)| {
                (
                    member_id.clone(),
                    ClusterMemberView {
                        postgres: build_member_view(member_id, record, authoritative_leader.as_ref()),
                        postgres_target: record.postgres_target.clone(),
                    },
                )
            })
            .collect(),
        leadership: state
            .leader
            .as_ref()
            .map(|x| LeadershipObservation::Held(x.clone()))
            .unwrap_or(LeadershipObservation::Open),
        switchover: state
            .switchover
            .as_ref()
            .map(|x| SwitchoverView::Requested(x.clone()))
            .unwrap_or(SwitchoverView::None),
    };

    match mode {
        DcsMode::NotTrusted => DcsView::NotTrusted(NotTrustedView {
            observed_leadership: state.leader.clone(),
            cluster,
        }),
        DcsMode::Degraded => DcsView::Degraded(cluster),
        DcsMode::Coordinated => DcsView::Coordinated(cluster),
    }
}

fn build_member_view(
    member_id: &MemberId,
    record: &MemberRecord,
    authoritative_leader: Option<&MemberId>,
) -> MemberPostgresView {
    match &record.postgres {
        MemberPostgresRecord::Unknown {
            readiness,
            timeline,
            system_identifier,
        } => MemberPostgresView::Unknown {
            readiness: readiness.clone(),
            timeline: *timeline,
            system_identifier: *system_identifier,
        },
        MemberPostgresRecord::Primary {
            readiness,
            system_identifier,
            committed_wal,
        } => {
            if authoritative_leader.is_some_and(|leader| leader != member_id) {
                MemberPostgresView::Unknown {
                    readiness: readiness.clone(),
                    timeline: committed_wal.timeline,
                    system_identifier: *system_identifier,
                }
            } else {
                MemberPostgresView::Primary {
                    readiness: readiness.clone(),
                    system_identifier: *system_identifier,
                    committed_wal: committed_wal.clone(),
                }
            }
        }
        MemberPostgresRecord::Replica {
            readiness,
            system_identifier,
            upstream,
            replay_wal,
            follow_wal,
        } => MemberPostgresView::Replica {
            readiness: readiness.clone(),
            system_identifier: *system_identifier,
            upstream: upstream.clone(),
            replay_wal: replay_wal.clone(),
            follow_wal: follow_wal.clone(),
        },
    }
}

fn build_local_member_record(
    now: UnixMillis,
    postgres_target: &PgTcpTarget,
    ttl_ms: u64,
    pg_state: &PgInfoState,
    previous_record: Option<&MemberRecord>,
) -> MemberRecord {
    let expires_at = UnixMillis(now.0.saturating_add(ttl_ms));

    let postgres = match pg_state {
        PgInfoState::Unknown { common } => MemberPostgresRecord::Unknown {
            readiness: common.readiness.clone(),
            timeline: common.timeline.or_else(|| previous_record.and_then(member_record_timeline)),
            system_identifier: common.system_identifier.or_else(|| {
                previous_record.and_then(member_record_system_identifier)
            }),
        },
        PgInfoState::Primary { common, wal_lsn, .. } => MemberPostgresRecord::Primary {
            readiness: common.readiness.clone(),
            system_identifier: common.system_identifier,
            committed_wal: crate::state::ObservedWalPosition {
                timeline: common.timeline,
                lsn: *wal_lsn,
            },
        },
        PgInfoState::Replica {
            common,
            replay_lsn,
            follow_lsn,
            upstream,
        } => MemberPostgresRecord::Replica {
            readiness: common.readiness.clone(),
            system_identifier: common.system_identifier,
            upstream: upstream.as_ref().map(|u| u.member_id.clone()),
            replay_wal: Some(crate::state::ObservedWalPosition {
                timeline: common.timeline,
                lsn: *replay_lsn,
            }),
            follow_wal: follow_lsn.map(|lsn| crate::state::ObservedWalPosition {
                timeline: common.timeline,
                lsn,
            }),
        },
    };

    MemberRecord {
        expires_at,
        postgres_target: postgres_target.clone(),
        postgres,
    }
}

fn member_record_timeline(record: &MemberRecord) -> Option<crate::state::TimelineId> {
    match &record.postgres {
        MemberPostgresRecord::Unknown { timeline, .. } => *timeline,
        MemberPostgresRecord::Primary { committed_wal, .. } => committed_wal.timeline,
        MemberPostgresRecord::Replica {
            replay_wal,
            follow_wal,
            ..
        } => replay_wal
            .as_ref()
            .and_then(|x| x.timeline)
            .or_else(|| follow_wal.as_ref().and_then(|x| x.timeline)),
    }
}

fn member_record_system_identifier(
    record: &MemberRecord,
) -> Option<crate::state::SystemIdentifier> {
    match &record.postgres {
        MemberPostgresRecord::Unknown {
            system_identifier, ..
        }
        | MemberPostgresRecord::Primary {
            system_identifier, ..
        }
        | MemberPostgresRecord::Replica {
            system_identifier, ..
        } => *system_identifier,
    }
}

fn now_unix_millis() -> Result<UnixMillis, WorkerError> {
    let elapsed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|err| WorkerError::Message(format!("system clock before unix epoch: {err}")))?;
    let millis = u64::try_from(elapsed.as_millis())
        .map_err(|err| WorkerError::Message(format!("unix millis conversion failed: {err}")))?;
    Ok(UnixMillis(millis))
}

fn to_worker_error(err: impl std::fmt::Display) -> WorkerError {
    WorkerError::Message(err.to_string())
}
```

---

## `startup.rs`

Also tiny.

```rust
use crate::{
    config::{DcsClientConfig, DcsEndpoint, RuntimeConfig},
    logging::LogSender,
    state::{NodeIdentity, PgTcpTarget},
};

use super::{
    runtime::{bootstrap_runtime, DcsRuntime, DcsRuntimeRequest},
    schema::DcsState,
};

pub(crate) struct DcsAdvertisedEndpoints {
    pub(crate) postgres: PgTcpTarget,
}

pub(crate) struct BootstrapRequest {
    pub(crate) identity: NodeIdentity,
    pub(crate) endpoints: Vec<DcsEndpoint>,
    pub(crate) client: DcsClientConfig,
    pub(crate) member_ttl_ms: u64,
    pub(crate) advertised: DcsAdvertisedEndpoints,
    pub(crate) log: LogSender,
}

impl DcsAdvertisedEndpoints {
    pub(crate) fn from_config(cfg: &RuntimeConfig) -> Result<Self, String> {
        let advertise_port = cfg
            .postgres
            .network
            .advertise_port
            .unwrap_or(cfg.postgres.network.listen_port);

        let postgres =
            PgTcpTarget::new(cfg.postgres.network.listen_host.clone(), advertise_port)?;
        Ok(Self { postgres })
    }
}

pub(crate) async fn bootstrap(request: BootstrapRequest) -> Result<DcsRuntime, String> {
    let repo = TypedEtcdRepo::<DcsState>::connect(
        request.endpoints,
        request.client,
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(bootstrap_runtime(DcsRuntimeRequest {
        identity: request.identity,
        advertised_postgres: request.advertised.postgres,
        member_ttl_ms: request.member_ttl_ms,
        repo,
        log: request.log,
    }))
}
```

---

## What disappeared

All of this disappears from DCS code:

path helpers
manual UTF-8 decode
manual serde encode/decode
manual watch key parsing
manual delete-vs-put branching per field
manual lease plumbing per field
manual “timeout_etcd("etcd xyz", …)” strings
manual prefix watch construction

That becomes generic store machinery generated from the schema.

So yes, the answer is: this should be closer to 20% of the previous code.

The handwritten DCS crate should just express:

the schema
the view-building rules
the command semantics

Everything else should be generic infra.

---

## What the generated API is assumed to provide

This is the missing piece. Your custom derive should generate something roughly equivalent to:

```rust
TypedEtcdRepo::<DcsState>::connect(...)
repo.session(scope).await?

session.load().await?
session.next().await?

session.field_mut(|s| &mut s.leader).claim_if_empty(epoch).await?
session.field_mut(|s| &mut s.leader).delete_if(...).await?

session.field_mut(|s| &mut s.switchover).set(Some(target)).await?
session.field_mut(|s| &mut s.switchover).set(None).await?

session.field_mut(|s| &mut s.members).put(member_id, record).await?
session.field_mut(|s| &mut s.members).delete(&member_id).await?
```

That is the real simplification. Not “write a smaller ugly etcd adapter by hand.” The real simplification is “stop handwriting the adapter at all.”

---

## The one honest caveat

I did not include the full implementation of `TypedEtcdRepo`, `TypedEtcdSession`, and the `EtcdSchema` proc macro here, because that is the framework layer, not the DCS layer.

But that is exactly where the missing boilerplate belongs.

So yes: your instinct is correct. The right end state is not a prettier `etcd.rs`. The right end state is that DCS has no meaningful `etcd.rs` at all.

The next useful step is for me to write the minimal framework API for:

`#[derive(EtcdSchema)]`
`TypedEtcdRepo<T>`
`TypedEtcdSession<T>`
field accessors for `Option<T>` singletons and `BTreeMap<K, V>` maps
watch -> typed state reload

That is the part that collapses the remaining 80%.


<acceptance_criteria>
</acceptance_criteria>
