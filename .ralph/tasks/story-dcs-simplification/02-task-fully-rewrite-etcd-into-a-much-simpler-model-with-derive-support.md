## Task: Fully Rewrite Etcd Into A Much Simpler Model With `#[]` Derive Support <status>not_started</status> <passes>false</passes>

<description>
**Goal**

The actual problem to solve is not "add a macro". The actual problem is that the DCS schema design, key formats, decode rules, watch interpretation, lease behavior, and business logic are currently smeared through long imperative etcd code. That is the mess.

This task must explicitly separate:

- the schema design
- the key formats
- the decode/encode rules for those keys
- the etcd transport implementation
- the DCS runtime/business behavior

That separation must be made explicit in code structure first. If a small etcd framework helps, that is allowed. It is not forbidden. What is forbidden is building generic infrastructure for its own sake while the schema is still scattered across `src/dcs/worker.rs`.

**Direct answers that must stay in this task**

- No, a proc macro is not inherently required for this project.
- Yes, a small extracted etcd framework is allowed if it directly serves the split between schema/key layout and etcd transport/runtime code.
- The first pass should prefer a handwritten schema boundary because that is the simplest and clearest way to fix the real problem.
- `#[]` derive support is optional follow-up. It is acceptable only if it reduces already-separated handwritten schema boilerplate. It must not be the thing that makes the design understandable.
- The design must stay fully inside this task. The implementer must not "figure it out later".

**What this task is really meant to solve**

Today the DCS implementation mixes all of these concerns in one place:

- path layout such as leader/member/switchover keys
- path parsing
- serde JSON encode/decode
- full-snapshot rebuild rules
- watch event interpretation
- lease policy rules
- command semantics
- DCS mode evaluation
- public view building
- retry/reconnect transport details

That is exactly backwards. The end state required by this task is:

- one explicit schema module that owns the DCS keyspace and record formats
- one explicit store module that owns etcd transport mechanics
- one explicit runtime module that owns DCS commands and view logic
- startup wiring that composes them
- no hidden schema logic sprinkled through watch loops or command handlers

**Why `repo` and `session` are needed**

These are not pattern-for-pattern's-sake names. They solve two real lifecycle splits that already exist whether we name them or not.

- `TypedEtcdRepo` is the long-lived transport/bootstrap owner.
  - It owns endpoints, auth, TLS, and the raw etcd client.
  - It knows how to establish a scope-bound session.
  - It does not know DCS business rules.
- `TypedEtcdSession` is the live scope/watch/lease owner.
  - It owns one scope prefix.
  - It owns one watch stream.
  - It owns any active ephemeral lease bookkeeping tied to that scope.
  - It owns one local typed cache of the decoded schema state.
  - It is the right place to reconnect, reload, and continue watching.

This split is required because the transport state is genuinely stateful:

- a session has watch stream state
- a session has revision/cached-state continuity
- a session may have an active leader lease that must be kept alive or dropped
- a disconnected session needs to be recreated cleanly without rebuilding the whole runtime architecture

So yes, `repo` and `session` are needed in this project. The exact names could change. The lifecycle split cannot.

**Macro caveat, clearly**

The macro exists only to compress already-correct schema declarations into less handwritten metadata.

What it would serve:

- generate field descriptors from one schema declaration
- generate key layout helpers
- generate `apply_put` / `apply_delete` dispatch
- generate repetitive encode/decode glue

What it must not serve:

- hiding business logic
- hiding lease semantics from review
- inventing a giant generic framework detached from this repo
- making the schema harder to read than a handwritten module

Conclusion for this repo:

- the macro is not required for the actual simplification
- the schema split is required
- the store/session split is required
- derive support is optional and should come only after the handwritten version is explicit and clean

If the handwritten schema layer is already small and clear, there is no obligation to add the macro at all.

**Will this plan work in the current project**

Yes, but only if the task explicitly includes the surrounding cleanup and does not pretend this is isolated to one file.

Why it will work:

- current runtime composition already treats DCS as a bootstrap returning state, handle, and worker
- current consumers in HA/API/CLI/process already mostly depend on `DcsView` and `DcsHandle`, not raw etcd details
- the workspace already uses an internal proc-macro crate pattern, so optional derive support is viable later if still wanted
- the current pain is concentrated in `src/dcs/worker.rs` and `src/dcs/state.rs`, which means the messy schema/transport coupling has a clear place to extract from

What else must change and should be cleaned aggressively:

- DCS config dead surface such as `dcs.init` if it is still not actually used
- test helpers and fixtures that preserve dead DCS config/state
- any remaining DCS code that reconstructs key layout outside the new schema module
- any remaining etcd code that decodes DCS records outside the new schema module

Greenfield rules apply here:

- no backward compatibility
- no compatibility shim for the old shape
- delete the old scattered implementation instead of preserving it under new wrappers

## Required final layout

The required end-state module split is:

```text
src/dcs/
  mod.rs
  schema.rs
  store.rs
  runtime.rs
  startup.rs
  command.rs
  log_event.rs
```

The responsibilities are:

- `schema.rs`
  - DCS records
  - public DCS view types
  - key layout functions
  - schema field descriptors
  - watch decode/apply rules
- `store.rs`
  - raw etcd client handling
  - watch stream handling
  - snapshot loading
  - generic singleton/map mutation helpers
  - lease attachment/keepalive mechanics
- `runtime.rs`
  - DCS commands
  - DCS actor loop
  - mode evaluation
  - view publication
  - translating PG info into member records
- `startup.rs`
  - wiring config into repo/runtime bootstrap
- `command.rs`
  - typed crate-private mutation handle

No other file is allowed to define DCS key formats.

No other file is allowed to define DCS etcd decode rules.

No other file is allowed to manually rebuild DCS paths.

## Actual design to implement

### 1. `src/dcs/mod.rs`

```rust
mod command;
pub(crate) mod log_event;
mod runtime;
mod schema;
mod store;
pub(crate) mod startup;

pub(crate) use command::DcsHandle;
pub(crate) use runtime::DcsWorker;
pub use schema::{
    ClusterMemberView, ClusterView, DcsMode, DcsView, LeadershipObservation, MemberPostgresView,
    NotTrustedView, SwitchoverView,
};
```

### 2. `src/dcs/schema.rs`

This file is the center of the task. It is the fix for the real problem. The schema and key layout are first-class here instead of being smeared into worker logic.

```rust
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    pginfo::state::{PgInfoState, Readiness},
    state::{
        LeaseEpoch, MemberId, ObservedWalPosition, PgTcpTarget, SwitchoverTarget,
        SystemIdentifier, TimelineId, UnixMillis,
    },
};

use super::store::{
    EtcdDecodeError, EtcdKeyCodec, EtcdSchema, LeaseAttachment, MapField, SingletonField,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DcsMode {
    NotTrusted,
    Degraded,
    Coordinated,
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct DcsState {
    pub(crate) leader: Option<LeaseEpoch>,
    pub(crate) switchover: Option<SwitchoverTarget>,
    pub(crate) members: BTreeMap<MemberId, MemberRecord>,
}

impl DcsState {
    pub(crate) fn empty() -> Self {
        Self {
            leader: None,
            switchover: None,
            members: BTreeMap::new(),
        }
    }
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

pub(crate) struct DcsKvSchema;
pub(crate) struct LeaderField;
pub(crate) struct SwitchoverField;
pub(crate) struct MembersField;

impl DcsKvSchema {
    pub(crate) fn prefix(scope: &str) -> String {
        format!("/{scope}")
    }

    pub(crate) fn leader_key(scope: &str) -> String {
        format!("{}/leader", Self::prefix(scope))
    }

    pub(crate) fn switchover_key(scope: &str) -> String {
        format!("{}/switchover", Self::prefix(scope))
    }

    pub(crate) fn member_prefix(scope: &str) -> String {
        format!("{}/members/", Self::prefix(scope))
    }

    pub(crate) fn member_key(scope: &str, member_id: &MemberId) -> String {
        format!("{}{member_id}", Self::member_prefix(scope))
    }

    pub(crate) fn parse_member_id(scope: &str, key: &str) -> Option<MemberId> {
        let prefix = Self::member_prefix(scope);
        key.strip_prefix(prefix.as_str())
            .map(|value| MemberId(value.to_string()))
            .filter(|member_id| !member_id.0.is_empty())
    }
}

impl EtcdSchema for DcsKvSchema {
    type State = DcsState;

    fn empty_state() -> Self::State {
        DcsState::empty()
    }

    fn prefix(scope: &str) -> String {
        Self::prefix(scope)
    }

    fn apply_put(
        state: &mut Self::State,
        scope: &str,
        key: &str,
        value: &[u8],
    ) -> Result<(), EtcdDecodeError> {
        if key == Self::leader_key(scope) {
            state.leader = Some(serde_json::from_slice(value).map_err(EtcdDecodeError::json)?);
            return Ok(());
        }

        if key == Self::switchover_key(scope) {
            state.switchover =
                Some(serde_json::from_slice(value).map_err(EtcdDecodeError::json)?);
            return Ok(());
        }

        if let Some(member_id) = Self::parse_member_id(scope, key) {
            let record = serde_json::from_slice(value).map_err(EtcdDecodeError::json)?;
            state.members.insert(member_id, record);
            return Ok(());
        }

        Err(EtcdDecodeError::unknown_key(key))
    }

    fn apply_delete(
        state: &mut Self::State,
        scope: &str,
        key: &str,
    ) -> Result<(), EtcdDecodeError> {
        if key == Self::leader_key(scope) {
            state.leader = None;
            return Ok(());
        }

        if key == Self::switchover_key(scope) {
            state.switchover = None;
            return Ok(());
        }

        if let Some(member_id) = Self::parse_member_id(scope, key) {
            state.members.remove(&member_id);
            return Ok(());
        }

        Err(EtcdDecodeError::unknown_key(key))
    }
}

impl SingletonField<DcsState> for LeaderField {
    type Value = LeaseEpoch;

    fn path(scope: &str) -> String {
        DcsKvSchema::leader_key(scope)
    }

    fn slot(state: &DcsState) -> Option<&Self::Value> {
        state.leader.as_ref()
    }

    fn slot_mut(state: &mut DcsState) -> &mut Option<Self::Value> {
        &mut state.leader
    }

    fn lease_attachment(_value: &Self::Value) -> LeaseAttachment {
        LeaseAttachment::SessionEphemeral
    }
}

impl SingletonField<DcsState> for SwitchoverField {
    type Value = SwitchoverTarget;

    fn path(scope: &str) -> String {
        DcsKvSchema::switchover_key(scope)
    }

    fn slot(state: &DcsState) -> Option<&Self::Value> {
        state.switchover.as_ref()
    }

    fn slot_mut(state: &mut DcsState) -> &mut Option<Self::Value> {
        &mut state.switchover
    }

    fn lease_attachment(_value: &Self::Value) -> LeaseAttachment {
        LeaseAttachment::None
    }
}

impl EtcdKeyCodec for MemberId {
    fn encode_key(&self) -> String {
        self.0.clone()
    }

    fn decode_key(value: &str) -> Result<Self, EtcdDecodeError> {
        if value.is_empty() {
            return Err(EtcdDecodeError::message("empty member id key".to_string()));
        }
        Ok(MemberId(value.to_string()))
    }
}

impl MapField<DcsState> for MembersField {
    type Key = MemberId;
    type Value = MemberRecord;

    fn prefix(scope: &str) -> String {
        DcsKvSchema::member_prefix(scope)
    }

    fn map(state: &DcsState) -> &BTreeMap<Self::Key, Self::Value> {
        &state.members
    }

    fn map_mut(state: &mut DcsState) -> &mut BTreeMap<Self::Key, Self::Value> {
        &mut state.members
    }

    fn lease_attachment(value: &Self::Value) -> LeaseAttachment {
        LeaseAttachment::TtlUntil(value.expires_at)
    }
}

pub(crate) fn evaluate_mode(
    etcd_reachable: bool,
    state: &DcsState,
    self_id: &MemberId,
    now: UnixMillis,
) -> DcsMode {
    if !etcd_reachable {
        return DcsMode::NotTrusted;
    }

    let local_is_live = state
        .members
        .get(self_id)
        .map(|member| member.expires_at.0 > now.0)
        .unwrap_or(false);
    if !local_is_live {
        return DcsMode::Degraded;
    }

    let live_members = state
        .members
        .values()
        .filter(|member| member.expires_at.0 > now.0)
        .count();
    if live_members < 1 {
        return DcsMode::Degraded;
    }

    DcsMode::Coordinated
}

pub(crate) fn build_dcs_view(mode: DcsMode, state: &DcsState, now: UnixMillis) -> DcsView {
    let authoritative_leader = state.leader.as_ref().map(|epoch| epoch.holder.clone());
    let cluster = ClusterView {
        members: state
            .members
            .iter()
            .filter(|(_, member)| member.expires_at.0 > now.0)
            .map(|(member_id, member)| {
                (
                    member_id.clone(),
                    ClusterMemberView {
                        postgres_target: member.postgres_target.clone(),
                        postgres: build_member_view(member_id, member, authoritative_leader.as_ref()),
                    },
                )
            })
            .collect(),
        leadership: state
            .leader
            .as_ref()
            .map(|epoch| LeadershipObservation::Held(epoch.clone()))
            .unwrap_or(LeadershipObservation::Open),
        switchover: state
            .switchover
            .as_ref()
            .map(|target| SwitchoverView::Requested(target.clone()))
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
        } if authoritative_leader.is_some_and(|leader| leader != member_id) => {
            MemberPostgresView::Unknown {
                readiness: readiness.clone(),
                timeline: committed_wal.timeline,
                system_identifier: *system_identifier,
            }
        }
        MemberPostgresRecord::Primary {
            readiness,
            system_identifier,
            committed_wal,
        } => MemberPostgresView::Primary {
            readiness: readiness.clone(),
            system_identifier: *system_identifier,
            committed_wal: committed_wal.clone(),
        },
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

pub(crate) fn build_local_member_record(
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
            timeline: common.timeline.or_else(|| previous_record.and_then(record_timeline)),
            system_identifier: common
                .system_identifier
                .or_else(|| previous_record.and_then(record_system_identifier)),
        },
        PgInfoState::Primary { common, wal_lsn, .. } => MemberPostgresRecord::Primary {
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
        } => MemberPostgresRecord::Replica {
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
        expires_at,
        postgres_target: postgres_target.clone(),
        postgres,
    }
}

fn record_timeline(record: &MemberRecord) -> Option<TimelineId> {
    match &record.postgres {
        MemberPostgresRecord::Unknown { timeline, .. } => *timeline,
        MemberPostgresRecord::Primary { committed_wal, .. } => committed_wal.timeline,
        MemberPostgresRecord::Replica {
            replay_wal,
            follow_wal,
            ..
        } => replay_wal
            .as_ref()
            .and_then(|position| position.timeline)
            .or_else(|| follow_wal.as_ref().and_then(|position| position.timeline)),
    }
}

fn record_system_identifier(record: &MemberRecord) -> Option<SystemIdentifier> {
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
```

### 3. `src/dcs/store.rs`

This is the allowed small etcd framework. It is allowed because it directly serves the extraction of schema/key layout from transport code. It is not "infra bullshit" if it stays this small and this explicit.

```rust
use std::{collections::BTreeMap, marker::PhantomData, str, time::Duration};

use etcd_client::{
    Certificate, Client, Compare, CompareOp, ConnectOptions, EventType, GetOptions, Identity,
    LeaseKeepAliveStream, LeaseKeeper, PutOptions, TlsOptions, Txn, TxnOp, WatchOptions,
    WatchResponse, WatchStream, Watcher,
};
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;
use tokio::time::Instant;

use crate::{
    config::{resolve_inline_or_path_bytes, resolve_secret_string, DcsClientConfig, DcsEndpoint},
    state::UnixMillis,
};

const ETCD_TIMEOUT: Duration = Duration::from_secs(2);
const KEEPALIVE_LEAD_TIME: Duration = Duration::from_millis(250);
const MIN_TTL_SECONDS: i64 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub(crate) enum EtcdStoreError {
    #[error("etcd io failed: {0}")]
    Io(String),
    #[error("etcd decode failed for key `{key}`: {message}")]
    Decode { key: String, message: String },
    #[error("invalid etcd event key `{0}`")]
    InvalidKey(String),
    #[error("missing lease while keepalive required")]
    MissingLease,
    #[error("optimistic claim failed")]
    ClaimFailed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct EtcdDecodeError {
    message: String,
}

impl EtcdDecodeError {
    pub(crate) fn json(err: serde_json::Error) -> Self {
        Self {
            message: err.to_string(),
        }
    }

    pub(crate) fn message(message: String) -> Self {
        Self { message }
    }

    pub(crate) fn unknown_key(key: &str) -> Self {
        Self {
            message: format!("unknown schema key `{key}`"),
        }
    }
}

pub(crate) trait EtcdKeyCodec: Clone + Ord + Send + Sync + 'static {
    fn encode_key(&self) -> String;
    fn decode_key(value: &str) -> Result<Self, EtcdDecodeError>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LeaseAttachment {
    None,
    SessionEphemeral,
    TtlUntil(UnixMillis),
}

pub(crate) trait EtcdSchema: Send + Sync + 'static {
    type State: Clone + Send + Sync + 'static;

    fn empty_state() -> Self::State;
    fn prefix(scope: &str) -> String;
    fn apply_put(
        state: &mut Self::State,
        scope: &str,
        key: &str,
        value: &[u8],
    ) -> Result<(), EtcdDecodeError>;
    fn apply_delete(
        state: &mut Self::State,
        scope: &str,
        key: &str,
    ) -> Result<(), EtcdDecodeError>;
}

pub(crate) trait SingletonField<S>: Send + Sync + 'static {
    type Value: Clone + Serialize + DeserializeOwned + Send + Sync + 'static;

    fn path(scope: &str) -> String;
    fn slot(state: &S) -> Option<&Self::Value>;
    fn slot_mut(state: &mut S) -> &mut Option<Self::Value>;
    fn lease_attachment(value: &Self::Value) -> LeaseAttachment;
}

pub(crate) trait MapField<S>: Send + Sync + 'static {
    type Key: EtcdKeyCodec;
    type Value: Clone + Serialize + DeserializeOwned + Send + Sync + 'static;

    fn prefix(scope: &str) -> String;
    fn map(state: &S) -> &BTreeMap<Self::Key, Self::Value>;
    fn map_mut(state: &mut S) -> &mut BTreeMap<Self::Key, Self::Value>;
    fn lease_attachment(value: &Self::Value) -> LeaseAttachment;

    fn path(scope: &str, key: &Self::Key) -> String {
        format!("{}{}", Self::prefix(scope), key.encode_key())
    }
}

pub(crate) struct TypedEtcdRepo<T: EtcdSchema> {
    client: Client,
    _schema: PhantomData<T>,
}

pub(crate) struct TypedEtcdSession<T: EtcdSchema> {
    client: Client,
    scope: String,
    cache: T::State,
    watcher: Watcher,
    watch_stream: WatchStream,
    session_lease: Option<SessionLease>,
    _schema: PhantomData<T>,
}

struct SessionLease {
    lease_id: i64,
    keeper: LeaseKeeper,
    stream: LeaseKeepAliveStream,
    next_keepalive_at: Instant,
}

pub(crate) struct SingletonHandle<'a, T: EtcdSchema, F: SingletonField<T::State>> {
    session: &'a mut TypedEtcdSession<T>,
    _field: PhantomData<F>,
}

pub(crate) struct MapHandle<'a, T: EtcdSchema, F: MapField<T::State>> {
    session: &'a mut TypedEtcdSession<T>,
    _field: PhantomData<F>,
}

impl<T: EtcdSchema> TypedEtcdRepo<T> {
    pub(crate) async fn connect(
        endpoints: Vec<DcsEndpoint>,
        client_cfg: DcsClientConfig,
    ) -> Result<Self, EtcdStoreError> {
        let endpoint_urls = endpoints
            .into_iter()
            .map(|endpoint| endpoint.to_url())
            .collect::<Vec<_>>();
        let options = build_connect_options(client_cfg).await?;
        let client = timeout_etcd(
            Client::connect(endpoint_urls, Some(options)),
            "client connect",
        )
        .await?;
        Ok(Self {
            client,
            _schema: PhantomData,
        })
    }

    pub(crate) async fn session(&self, scope: String) -> Result<TypedEtcdSession<T>, EtcdStoreError> {
        let prefix = T::prefix(scope.as_str());
        let (watcher, watch_stream) = timeout_etcd(
            self.client
                .clone()
                .watch(prefix.as_str(), Some(WatchOptions::new().with_prefix())),
            "watch start",
        )
        .await?;

        Ok(TypedEtcdSession {
            client: self.client.clone(),
            scope,
            cache: T::empty_state(),
            watcher,
            watch_stream,
            session_lease: None,
            _schema: PhantomData,
        })
    }
}

impl<T: EtcdSchema> TypedEtcdSession<T> {
    pub(crate) async fn load(&mut self) -> Result<T::State, EtcdStoreError> {
        let prefix = T::prefix(self.scope.as_str());
        let response = timeout_etcd(
            self.client
                .get(prefix.as_str(), Some(GetOptions::new().with_prefix())),
            "snapshot load",
        )
        .await?;

        let mut next = T::empty_state();
        for kv in response.kvs() {
            let key = str::from_utf8(kv.key())
                .map_err(|err| EtcdStoreError::Io(format!("utf8 key decode failed: {err}")))?;
            T::apply_put(&mut next, self.scope.as_str(), key, kv.value()).map_err(|err| {
                EtcdStoreError::Decode {
                    key: key.to_string(),
                    message: err.message,
                }
            })?;
        }

        self.cache = next.clone();
        Ok(next)
    }

    pub(crate) async fn next(&mut self) -> Result<Option<T::State>, EtcdStoreError> {
        self.keepalive_if_needed().await?;
        let response = self
            .watch_stream
            .message()
            .await
            .map_err(|err| EtcdStoreError::Io(format!("watch receive failed: {err}")))?;
        let Some(response) = response else {
            return Ok(None);
        };
        self.apply_watch_response(response)?;
        Ok(Some(self.cache.clone()))
    }

    pub(crate) fn singleton<F: SingletonField<T::State>>(&mut self) -> SingletonHandle<'_, T, F> {
        SingletonHandle {
            session: self,
            _field: PhantomData,
        }
    }

    pub(crate) fn map<F: MapField<T::State>>(&mut self) -> MapHandle<'_, T, F> {
        MapHandle {
            session: self,
            _field: PhantomData,
        }
    }

    fn apply_watch_response(&mut self, response: WatchResponse) -> Result<(), EtcdStoreError> {
        for event in response.events() {
            let kv = event
                .kv()
                .ok_or_else(|| EtcdStoreError::Io("watch event missing kv".to_string()))?;
            let key = str::from_utf8(kv.key())
                .map_err(|err| EtcdStoreError::Io(format!("utf8 key decode failed: {err}")))?;

            match event.event_type() {
                EventType::Put => {
                    T::apply_put(&mut self.cache, self.scope.as_str(), key, kv.value()).map_err(
                        |err| EtcdStoreError::Decode {
                            key: key.to_string(),
                            message: err.message,
                        },
                    )?;
                }
                EventType::Delete => {
                    T::apply_delete(&mut self.cache, self.scope.as_str(), key).map_err(|err| {
                        EtcdStoreError::Decode {
                            key: key.to_string(),
                            message: err.message,
                        }
                    })?;
                }
            }
        }
        Ok(())
    }

    async fn write_json(
        &mut self,
        path: &str,
        encoded: String,
        lease: LeaseAttachment,
    ) -> Result<(), EtcdStoreError> {
        let options = self.put_options_for_lease(lease).await?;
        timeout_etcd(
            self.client.put(path, encoded, options),
            "put key",
        )
        .await?;
        Ok(())
    }

    async fn delete_key(&mut self, path: &str) -> Result<(), EtcdStoreError> {
        timeout_etcd(self.client.delete(path, None), "delete key").await?;
        Ok(())
    }

    async fn claim_if_empty_json(
        &mut self,
        path: &str,
        encoded: String,
        lease: LeaseAttachment,
    ) -> Result<bool, EtcdStoreError> {
        let options = self.put_options_for_lease(lease).await?;
        let txn = Txn::new()
            .when(vec![Compare::version(path, CompareOp::Equal, 0)])
            .and_then(vec![TxnOp::put(path, encoded, options)]);
        let response = timeout_etcd(self.client.txn(txn), "claim key").await?;
        Ok(response.succeeded())
    }

    async fn keepalive_if_needed(&mut self) -> Result<(), EtcdStoreError> {
        let Some(lease) = self.session_lease.as_mut() else {
            return Ok(());
        };
        if Instant::now() < lease.next_keepalive_at {
            return Ok(());
        }

        lease
            .keeper
            .keep_alive()
            .await
            .map_err(|err| EtcdStoreError::Io(format!("lease keepalive send failed: {err}")))?;
        let response = lease
            .stream
            .message()
            .await
            .map_err(|err| EtcdStoreError::Io(format!("lease keepalive receive failed: {err}")))?;
        let granted_ttl = response
            .ok_or(EtcdStoreError::MissingLease)?
            .ttl();
        let ttl = granted_ttl.max(MIN_TTL_SECONDS);
        lease.next_keepalive_at =
            Instant::now() + Duration::from_secs(ttl as u64).saturating_sub(KEEPALIVE_LEAD_TIME);
        Ok(())
    }

    async fn put_options_for_lease(
        &mut self,
        lease: LeaseAttachment,
    ) -> Result<Option<PutOptions>, EtcdStoreError> {
        match lease {
            LeaseAttachment::None => Ok(None),
            LeaseAttachment::SessionEphemeral => {
                if self.session_lease.is_none() {
                    self.session_lease = Some(create_session_lease(&mut self.client).await?);
                }
                let lease_id = self
                    .session_lease
                    .as_ref()
                    .map(|lease_state| lease_state.lease_id)
                    .ok_or(EtcdStoreError::MissingLease)?;
                Ok(Some(PutOptions::new().with_lease(lease_id)))
            }
            LeaseAttachment::TtlUntil(deadline) => {
                let now = crate::logging::system_now_unix_millis();
                let ttl_ms = deadline.0.saturating_sub(now);
                let ttl_seconds =
                    i64::try_from((ttl_ms / 1000).max(1)).map_err(|err| EtcdStoreError::Io(err.to_string()))?;
                let lease = timeout_etcd(self.client.lease_grant(ttl_seconds, None), "ttl lease grant")
                    .await?;
                Ok(Some(PutOptions::new().with_lease(lease.id())))
            }
        }
    }
}

impl<'a, T, F> SingletonHandle<'a, T, F>
where
    T: EtcdSchema,
    F: SingletonField<T::State>,
{
    pub(crate) async fn set(
        self,
        value: Option<F::Value>,
    ) -> Result<(), EtcdStoreError> {
        let path = F::path(self.session.scope.as_str());
        match value {
            Some(value) => {
                let encoded = serde_json::to_string(&value)
                    .map_err(|err| EtcdStoreError::Io(format!("json encode failed: {err}")))?;
                self.session
                    .write_json(path.as_str(), encoded, F::lease_attachment(&value))
                    .await?;
                *F::slot_mut(&mut self.session.cache) = Some(value);
            }
            None => {
                self.session.delete_key(path.as_str()).await?;
                *F::slot_mut(&mut self.session.cache) = None;
            }
        }
        Ok(())
    }

    pub(crate) async fn claim_if_empty(
        self,
        value: F::Value,
    ) -> Result<bool, EtcdStoreError> {
        let path = F::path(self.session.scope.as_str());
        let encoded = serde_json::to_string(&value)
            .map_err(|err| EtcdStoreError::Io(format!("json encode failed: {err}")))?;
        let claimed = self
            .session
            .claim_if_empty_json(path.as_str(), encoded, F::lease_attachment(&value))
            .await?;
        if claimed {
            *F::slot_mut(&mut self.session.cache) = Some(value);
        }
        Ok(claimed)
    }

    pub(crate) async fn delete_if(
        self,
        predicate: impl FnOnce(&F::Value) -> bool,
    ) -> Result<(), EtcdStoreError> {
        let should_delete = F::slot(&self.session.cache).is_some_and(predicate);
        if !should_delete {
            return Ok(());
        }
        let path = F::path(self.session.scope.as_str());
        self.session.delete_key(path.as_str()).await?;
        *F::slot_mut(&mut self.session.cache) = None;
        Ok(())
    }
}

impl<'a, T, F> MapHandle<'a, T, F>
where
    T: EtcdSchema,
    F: MapField<T::State>,
{
    pub(crate) async fn put(
        self,
        key: F::Key,
        value: F::Value,
    ) -> Result<(), EtcdStoreError> {
        let path = F::path(self.session.scope.as_str(), &key);
        let encoded = serde_json::to_string(&value)
            .map_err(|err| EtcdStoreError::Io(format!("json encode failed: {err}")))?;
        self.session
            .write_json(path.as_str(), encoded, F::lease_attachment(&value))
            .await?;
        F::map_mut(&mut self.session.cache).insert(key, value);
        Ok(())
    }

    pub(crate) async fn delete(self, key: &F::Key) -> Result<(), EtcdStoreError> {
        let path = F::path(self.session.scope.as_str(), key);
        self.session.delete_key(path.as_str()).await?;
        F::map_mut(&mut self.session.cache).remove(key);
        Ok(())
    }
}

async fn build_connect_options(
    client_cfg: DcsClientConfig,
) -> Result<ConnectOptions, EtcdStoreError> {
    let mut options = ConnectOptions::new().with_timeout(ETCD_TIMEOUT);

    if let crate::config::DcsAuthConfig::Basic { username, password } = client_cfg.auth {
        let password = resolve_secret_string(&password)
            .map_err(|err| EtcdStoreError::Io(format!("dcs password resolve failed: {err}")))?;
        options = options.with_user(username, password);
    }

    if let crate::config::DcsTlsConfig::Enabled {
        ca_cert,
        identity,
        server_name,
    } = client_cfg.tls
    {
        let mut tls = TlsOptions::new();
        if let Some(ca_cert) = ca_cert {
            let bytes = resolve_inline_or_path_bytes(&ca_cert)
                .map_err(|err| EtcdStoreError::Io(format!("dcs ca resolve failed: {err}")))?;
            tls = tls.ca_certificate(Certificate::from_pem(bytes));
        }
        if let Some(identity_cfg) = identity {
            let cert = resolve_inline_or_path_bytes(&identity_cfg.cert)
                .map_err(|err| EtcdStoreError::Io(format!("dcs client cert resolve failed: {err}")))?;
            let key = resolve_inline_or_path_bytes(&identity_cfg.key)
                .map_err(|err| EtcdStoreError::Io(format!("dcs client key resolve failed: {err}")))?;
            tls = tls.identity(Identity::from_pem(cert, key));
        }
        if let Some(server_name) = server_name {
            tls = tls.domain_name(server_name);
        }
        options = options.with_tls(tls);
    }

    Ok(options)
}

async fn create_session_lease(client: &mut Client) -> Result<SessionLease, EtcdStoreError> {
    let granted = timeout_etcd(client.lease_grant(2, None), "session lease grant").await?;
    let lease_id = granted.id();
    let (keeper, stream) = client
        .lease_keep_alive(lease_id)
        .await
        .map_err(|err| EtcdStoreError::Io(format!("lease keepalive open failed: {err}")))?;
    Ok(SessionLease {
        lease_id,
        keeper,
        stream,
        next_keepalive_at: Instant::now(),
    })
}

async fn timeout_etcd<T>(
    future: impl std::future::Future<Output = Result<T, etcd_client::Error>>,
    label: &str,
) -> Result<T, EtcdStoreError> {
    tokio::time::timeout(ETCD_TIMEOUT, future)
        .await
        .map_err(|_| EtcdStoreError::Io(format!("etcd timeout while {label}")))?
        .map_err(|err| EtcdStoreError::Io(format!("{label}: {err}")))
}
```

### 4. `src/dcs/runtime.rs`

The runtime now consumes the schema/store layer. It no longer defines paths, parses keys, or manually decodes JSON.

```rust
use tokio::time::{Duration, MissedTickBehavior};

use crate::{
    logging::LogSender,
    pginfo::state::PgInfoState,
    state::{
        new_state_channel, LeaseEpoch, NodeIdentity, PgTcpTarget, StatePublisher, StateSubscriber,
        SwitchoverTarget, UnixMillis, WorkerError,
    },
};

use super::{
    command::DcsCommand,
    schema::{
        build_dcs_view, build_local_member_record, evaluate_mode, DcsKvSchema, DcsState,
        DcsView, LeaderField, MembersField, SwitchoverField,
    },
    store::{TypedEtcdRepo, TypedEtcdSession},
    DcsHandle,
};

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
    repo: TypedEtcdRepo<DcsKvSchema>,
    cache: DcsState,
    state_tx: StatePublisher<DcsView>,
    rx: tokio::sync::mpsc::UnboundedReceiver<DcsCommand>,
    _log: LogSender,
}

pub(crate) struct DcsRuntimeRequest {
    pub(crate) identity: NodeIdentity,
    pub(crate) advertised_postgres: PgTcpTarget,
    pub(crate) member_ttl_ms: u64,
    pub(crate) repo: TypedEtcdRepo<DcsKvSchema>,
    pub(crate) log: LogSender,
}

pub(crate) fn bootstrap_runtime(request: DcsRuntimeRequest) -> DcsRuntime {
    let (state_tx, state) = new_state_channel(DcsView::starting());
    let (handle, rx) = super::command::dcs_command_channel();

    DcsRuntime {
        state,
        handle,
        worker: DcsWorker {
            ctx: DcsWorkerCtx {
                identity: request.identity,
                advertised_postgres: request.advertised_postgres,
                member_ttl_ms: request.member_ttl_ms,
                repo: request.repo,
                cache: DcsState::empty(),
                state_tx,
                rx,
                _log: request.log,
            },
        },
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
            .map_err(to_worker_error)?;

        self.ctx.cache = session.load().await.map_err(to_worker_error)?;
        publish_view(&mut self.ctx, true)?;

        let mut reconnect = tokio::time::interval(Duration::from_secs(1));
        reconnect.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                maybe_command = self.ctx.rx.recv() => {
                    let Some(command) = maybe_command else {
                        return Err(WorkerError::Message("dcs command channel closed".to_string()));
                    };
                    apply_command(&mut self.ctx, &mut session, command).await?;
                    self.ctx.cache = session.load().await.map_err(to_worker_error)?;
                    publish_view(&mut self.ctx, true)?;
                }
                watched = session.next() => {
                    match watched.map_err(to_worker_error)? {
                        Some(state) => {
                            self.ctx.cache = state;
                            publish_view(&mut self.ctx, true)?;
                        }
                        None => {
                            publish_view(&mut self.ctx, false)?;
                            reconnect.tick().await;
                            self.ctx.cache = session.load().await.map_err(to_worker_error)?;
                            publish_view(&mut self.ctx, true)?;
                        }
                    }
                }
            }
        }
    }
}

async fn apply_command(
    ctx: &mut DcsWorkerCtx,
    session: &mut TypedEtcdSession<DcsKvSchema>,
    command: DcsCommand,
) -> Result<(), WorkerError> {
    match command {
        DcsCommand::AcquireLeadership => {
            let epoch = LeaseEpoch {
                holder: ctx.identity.member_id.clone(),
                generation: now_unix_millis()?.0,
            };
            let _ = session
                .singleton::<LeaderField>()
                .claim_if_empty(epoch)
                .await
                .map_err(to_worker_error)?;
        }
        DcsCommand::ReleaseLeadership => {
            session
                .singleton::<LeaderField>()
                .delete_if(|epoch| epoch.holder == ctx.identity.member_id)
                .await
                .map_err(to_worker_error)?;
        }
        DcsCommand::PublishSwitchover(target) => {
            session
                .singleton::<SwitchoverField>()
                .set(Some(target))
                .await
                .map_err(to_worker_error)?;
        }
        DcsCommand::ClearSwitchover => {
            session
                .singleton::<SwitchoverField>()
                .set(None)
                .await
                .map_err(to_worker_error)?;
        }
        DcsCommand::RefreshLocalMember(pg_state) => {
            let record = build_local_member_record(
                now_unix_millis()?,
                &ctx.advertised_postgres,
                ctx.member_ttl_ms,
                &pg_state,
                ctx.cache.members.get(&ctx.identity.member_id),
            );
            session
                .map::<MembersField>()
                .put(ctx.identity.member_id.clone(), record)
                .await
                .map_err(to_worker_error)?;
        }
        DcsCommand::RemoveLocalMember => {
            session
                .map::<MembersField>()
                .delete(&ctx.identity.member_id)
                .await
                .map_err(to_worker_error)?;
            session
                .singleton::<LeaderField>()
                .delete_if(|epoch| epoch.holder == ctx.identity.member_id)
                .await
                .map_err(to_worker_error)?;
        }
    }
    Ok(())
}

fn publish_view(ctx: &mut DcsWorkerCtx, etcd_reachable: bool) -> Result<(), WorkerError> {
    let now = now_unix_millis()?;
    let mode = evaluate_mode(etcd_reachable, &ctx.cache, &ctx.identity.member_id, now);
    let view = build_dcs_view(mode, &ctx.cache, now);
    ctx.state_tx
        .publish(view)
        .map_err(|err| WorkerError::Message(format!("dcs publish failed: {err}")))
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

### 5. `src/dcs/command.rs`

The current command handle design is already close to correct. Keep it typed and crate-private.

```rust
use tokio::sync::mpsc;

use crate::{pginfo::state::PgInfoState, state::SwitchoverTarget};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DcsCommand {
    RefreshLocalMember(PgInfoState),
    RemoveLocalMember,
    AcquireLeadership,
    ReleaseLeadership,
    PublishSwitchover(SwitchoverTarget),
    ClearSwitchover,
}

#[derive(Clone)]
pub(crate) struct DcsHandle {
    sender: mpsc::UnboundedSender<DcsCommand>,
}

pub(crate) type DcsCommandInbox = mpsc::UnboundedReceiver<DcsCommand>;

#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub(crate) enum DcsHandleError {
    #[error("dcs command channel closed")]
    ChannelClosed,
}

pub(crate) fn dcs_command_channel() -> (DcsHandle, DcsCommandInbox) {
    let (sender, receiver) = mpsc::unbounded_channel();
    (DcsHandle { sender }, receiver)
}

impl DcsHandle {
    pub(crate) fn refresh_local_member(&self, pg_state: PgInfoState) -> Result<(), DcsHandleError> {
        self.send(DcsCommand::RefreshLocalMember(pg_state))
    }

    pub(crate) fn remove_local_member(&self) -> Result<(), DcsHandleError> {
        self.send(DcsCommand::RemoveLocalMember)
    }

    pub(crate) fn acquire_leadership(&self) -> Result<(), DcsHandleError> {
        self.send(DcsCommand::AcquireLeadership)
    }

    pub(crate) fn release_leadership(&self) -> Result<(), DcsHandleError> {
        self.send(DcsCommand::ReleaseLeadership)
    }

    pub(crate) fn publish_switchover(
        &self,
        target: SwitchoverTarget,
    ) -> Result<(), DcsHandleError> {
        self.send(DcsCommand::PublishSwitchover(target))
    }

    pub(crate) fn clear_switchover(&self) -> Result<(), DcsHandleError> {
        self.send(DcsCommand::ClearSwitchover)
    }

    fn send(&self, command: DcsCommand) -> Result<(), DcsHandleError> {
        self.sender
            .send(command)
            .map_err(|_| DcsHandleError::ChannelClosed)
    }
}
```

### 6. `src/dcs/startup.rs`

```rust
use crate::{
    config::{DcsClientConfig, DcsEndpoint, RuntimeConfig},
    logging::LogSender,
    state::{NodeIdentity, PgTcpTarget},
};

use super::{
    runtime::{bootstrap_runtime, DcsRuntime, DcsRuntimeRequest},
    schema::DcsKvSchema,
    store::TypedEtcdRepo,
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
    let repo = TypedEtcdRepo::<DcsKvSchema>::connect(request.endpoints, request.client)
        .await
        .map_err(|err| err.to_string())?;

    Ok(bootstrap_runtime(DcsRuntimeRequest {
        identity: request.identity,
        advertised_postgres: request.advertised.postgres,
        member_ttl_ms: request.member_ttl_ms,
        repo,
        log: request.log,
    }))
}
```

## Optional appendix: derive support

This section must remain in the task because the question was asked explicitly. It is optional. It is not the main implementation requirement.

If, after the handwritten schema split is complete, the schema descriptors above still feel repetitive, then an internal proc-macro crate may be added. That macro is allowed only to generate the explicit schema metadata that is already understandable by hand.

The macro must generate:

- `DcsKvSchema`
- `LeaderField`
- `SwitchoverField`
- `MembersField`
- `impl EtcdSchema for ...`

It must not generate runtime logic.

### Optional file: `crates/pgtm_etcd_derive/Cargo.toml`

```toml
[package]
name = "pgtm_etcd_derive"
version = "0.1.0"
edition = "2021"

[lib]
proc-macro = true

[dependencies]
proc-macro2 = "1.0"
quote = "1.0"
syn = { version = "2.0", features = ["full", "extra-traits"] }
```

### Optional file: `crates/pgtm_etcd_derive/src/lib.rs`

This is the maximum acceptable scope. If the macro grows beyond this, stop and keep the handwritten schema.

```rust
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, Data, DeriveInput, Fields, LitStr, Meta, Type,
};

#[proc_macro_derive(EtcdSchema, attributes(etcd))]
pub fn derive_etcd_schema(input: TokenStream) -> TokenStream {
    derive_etcd_schema_impl(parse_macro_input!(input as DeriveInput))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn derive_etcd_schema_impl(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_ident = input.ident.clone();
    let data = match input.data {
        Data::Struct(data) => data,
        _ => {
            return Err(syn::Error::new(
                input.ident.span(),
                "EtcdSchema can only be derived for structs",
            ));
        }
    };

    let prefix = parse_container_prefix(&input.attrs)?;
    let fields = match data.fields {
        Fields::Named(fields) => fields.named,
        _ => {
            return Err(syn::Error::new(
                struct_ident.span(),
                "EtcdSchema requires named fields",
            ));
        }
    };

    let mut empty_fields = Vec::new();
    let mut apply_put_arms = Vec::new();
    let mut apply_delete_arms = Vec::new();
    let mut helper_items = Vec::new();

    for field in fields {
        let field_ident = field.ident.clone().ok_or_else(|| {
            syn::Error::new(struct_ident.span(), "missing field ident")
        })?;
        let meta = parse_field_meta(&field.attrs)?;

        empty_fields.push(quote! { #field_ident: Default::default() });

        match meta {
            FieldMeta::Singleton { path, lease } => {
                let helper_ident = format_ident!("{}", to_pascal(&field_ident.to_string()));
                let lease_tokens = lease_tokens_singleton(&lease);
                helper_items.push(quote! {
                    pub(crate) struct #helper_ident;

                    impl crate::dcs::store::SingletonField<#struct_ident> for #helper_ident {
                        type Value = extract_option_inner::<#field.ty>();

                        fn path(scope: &str) -> String {
                            let rendered = render_prefix(#prefix, scope);
                            format!("{}/{}", rendered, #path)
                        }

                        fn slot(state: &#struct_ident) -> Option<&Self::Value> {
                            state.#field_ident.as_ref()
                        }

                        fn slot_mut(state: &mut #struct_ident) -> &mut Option<Self::Value> {
                            &mut state.#field_ident
                        }

                        fn lease_attachment(value: &Self::Value) -> crate::dcs::store::LeaseAttachment {
                            #lease_tokens
                        }
                    }
                });

                apply_put_arms.push(quote! {
                    if key == #helper_ident::path(scope) {
                        state.#field_ident =
                            Some(serde_json::from_slice(value).map_err(crate::dcs::store::EtcdDecodeError::json)?);
                        return Ok(());
                    }
                });

                apply_delete_arms.push(quote! {
                    if key == #helper_ident::path(scope) {
                        state.#field_ident = None;
                        return Ok(());
                    }
                });
            }
            FieldMeta::Map { prefix_path, key_ty, lease } => {
                let helper_ident = format_ident!("{}", to_pascal(&field_ident.to_string()));
                let lease_tokens = lease_tokens_map(&lease);
                helper_items.push(quote! {
                    pub(crate) struct #helper_ident;

                    impl crate::dcs::store::MapField<#struct_ident> for #helper_ident {
                        type Key = #key_ty;
                        type Value = extract_btreemap_value::<#field.ty>();

                        fn prefix(scope: &str) -> String {
                            let rendered = render_prefix(#prefix, scope);
                            format!("{}/{}/", rendered, #prefix_path)
                        }

                        fn map(
                            state: &#struct_ident,
                        ) -> &std::collections::BTreeMap<Self::Key, Self::Value> {
                            &state.#field_ident
                        }

                        fn map_mut(
                            state: &mut #struct_ident,
                        ) -> &mut std::collections::BTreeMap<Self::Key, Self::Value> {
                            &mut state.#field_ident
                        }

                        fn lease_attachment(
                            value: &Self::Value,
                        ) -> crate::dcs::store::LeaseAttachment {
                            #lease_tokens
                        }
                    }
                });

                apply_put_arms.push(quote! {
                    if let Some(encoded_key) = key.strip_prefix(#helper_ident::prefix(scope).as_str()) {
                        let decoded_key = <#key_ty as crate::dcs::store::EtcdKeyCodec>::decode_key(encoded_key)?;
                        let decoded_value =
                            serde_json::from_slice(value).map_err(crate::dcs::store::EtcdDecodeError::json)?;
                        state.#field_ident.insert(decoded_key, decoded_value);
                        return Ok(());
                    }
                });

                apply_delete_arms.push(quote! {
                    if let Some(encoded_key) = key.strip_prefix(#helper_ident::prefix(scope).as_str()) {
                        let decoded_key = <#key_ty as crate::dcs::store::EtcdKeyCodec>::decode_key(encoded_key)?;
                        state.#field_ident.remove(&decoded_key);
                        return Ok(());
                    }
                });
            }
        }
    }

    Ok(quote! {
        pub(crate) struct DcsKvSchema;

        impl crate::dcs::store::EtcdSchema for DcsKvSchema {
            type State = #struct_ident;

            fn empty_state() -> Self::State {
                #struct_ident {
                    #(#empty_fields),*
                }
            }

            fn prefix(scope: &str) -> String {
                render_prefix(#prefix, scope)
            }

            fn apply_put(
                state: &mut Self::State,
                scope: &str,
                key: &str,
                value: &[u8],
            ) -> Result<(), crate::dcs::store::EtcdDecodeError> {
                #(#apply_put_arms)*
                Err(crate::dcs::store::EtcdDecodeError::unknown_key(key))
            }

            fn apply_delete(
                state: &mut Self::State,
                scope: &str,
                key: &str,
            ) -> Result<(), crate::dcs::store::EtcdDecodeError> {
                #(#apply_delete_arms)*
                Err(crate::dcs::store::EtcdDecodeError::unknown_key(key))
            }
        }

        #(#helper_items)*
    })
}

enum FieldMeta {
    Singleton {
        path: String,
        lease: LeaseMeta,
    },
    Map {
        prefix_path: String,
        key_ty: Type,
        lease: LeaseMeta,
    },
}

enum LeaseMeta {
    None,
    SessionEphemeral,
    TtlUntilField(syn::Ident),
}

fn parse_container_prefix(attrs: &[syn::Attribute]) -> syn::Result<String> {
    for attr in attrs {
        if !attr.path().is_ident("etcd") {
            continue;
        }
        let mut found = None::<String>;
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("prefix") {
                let value = meta.value()?.parse::<LitStr>()?;
                found = Some(value.value());
                return Ok(());
            }
            Err(meta.error("unsupported container attr"))
        })?;
        if let Some(found) = found {
            return Ok(found);
        }
    }
    Err(syn::Error::new(proc_macro2::Span::call_site(), "missing #[etcd(prefix = ...)]"))
}

fn parse_field_meta(attrs: &[syn::Attribute]) -> syn::Result<FieldMeta> {
    let mut singleton = None::<String>;
    let mut map = None::<String>;
    let mut key_ty = None::<Type>;
    let mut lease = LeaseMeta::None;

    for attr in attrs {
        if !attr.path().is_ident("etcd") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("singleton") {
                singleton = Some(meta.value()?.parse::<LitStr>()?.value());
                return Ok(());
            }
            if meta.path.is_ident("map") {
                map = Some(meta.value()?.parse::<LitStr>()?.value());
                return Ok(());
            }
            if meta.path.is_ident("key") {
                key_ty = Some(meta.value()?.parse()?);
                return Ok(());
            }
            if meta.path.is_ident("lease") {
                meta.parse_nested_meta(|lease_meta| {
                    if lease_meta.path.is_ident("kind") {
                        let kind = lease_meta.value()?.parse::<LitStr>()?.value();
                        if kind == "ephemeral" {
                            lease = LeaseMeta::SessionEphemeral;
                            return Ok(());
                        }
                        if kind == "ttl" {
                            return Ok(());
                        }
                        return Err(lease_meta.error("unsupported lease kind"));
                    }
                    if lease_meta.path.is_ident("from") {
                        lease = LeaseMeta::TtlUntilField(lease_meta.value()?.parse()?);
                        return Ok(());
                    }
                    Ok(())
                })?;
                return Ok(());
            }
            Err(meta.error("unsupported field attr"))
        })?;
    }

    match (singleton, map) {
        (Some(path), None) => Ok(FieldMeta::Singleton { path, lease }),
        (None, Some(prefix_path)) => Ok(FieldMeta::Map {
            prefix_path,
            key_ty: key_ty.ok_or_else(|| syn::Error::new(proc_macro2::Span::call_site(), "map field requires key = ..."))?,
            lease,
        }),
        _ => Err(syn::Error::new(proc_macro2::Span::call_site(), "field requires exactly one of singleton/map")),
    }
}

fn lease_tokens_singleton(lease: &LeaseMeta) -> proc_macro2::TokenStream {
    match lease {
        LeaseMeta::None => quote! { crate::dcs::store::LeaseAttachment::None },
        LeaseMeta::SessionEphemeral => quote! { crate::dcs::store::LeaseAttachment::SessionEphemeral },
        LeaseMeta::TtlUntilField(field) => quote! { crate::dcs::store::LeaseAttachment::TtlUntil(value.#field) },
    }
}

fn lease_tokens_map(lease: &LeaseMeta) -> proc_macro2::TokenStream {
    match lease {
        LeaseMeta::None => quote! { crate::dcs::store::LeaseAttachment::None },
        LeaseMeta::SessionEphemeral => quote! { crate::dcs::store::LeaseAttachment::SessionEphemeral },
        LeaseMeta::TtlUntilField(field) => quote! { crate::dcs::store::LeaseAttachment::TtlUntil(value.#field) },
    }
}

fn render_prefix(template: String, scope: &str) -> String {
    template.replace("{scope}", scope)
}

fn to_pascal(value: &str) -> String {
    let mut out = String::new();
    let mut upper = true;
    for ch in value.chars() {
        if ch == '_' {
            upper = true;
            continue;
        }
        if upper {
            out.extend(ch.to_uppercase());
            upper = false;
        } else {
            out.push(ch);
        }
    }
    out
}
```

That appendix is allowed. It is not required for task completion. The task is still successful if the handwritten schema/store split is completed cleanly and the macro is not added.

## Files that must be changed or explicitly audited

The implementation must include an explicit audit, not just "touch the obvious files".

- `Cargo.toml`
  - Only if optional derive support is added.
- `src/dcs/mod.rs`
- `src/dcs/command.rs`
- `src/dcs/state.rs`
  - Expected outcome: delete or replace by extracted `schema.rs`.
- `src/dcs/worker.rs`
  - Expected outcome: delete or replace by extracted `runtime.rs` plus `store.rs`.
- `src/dcs/startup.rs`
- `src/runtime/node.rs`
  - Retarget bootstrap request type if necessary.
- `src/ha/worker.rs`
- `src/ha/decide.rs`
- `src/api/controller.rs`
- `src/cli/status.rs`
- `src/process/source.rs`
- `src/process/worker.rs`
- `src/config/schema.rs`
  - Remove dead `DcsInitConfig` if still unused.
- `src/config/mod.rs`
  - Stop re-exporting dead DCS config types if removed.
- `src/dev_support/runtime_config.rs`
  - Remove dead builder support and tests for `dcs.init` if the feature is deleted.
- `tests/ha/support/...`
  - Update any fixture or observer logic that assumes old DCS data shape or stale config knobs.

## Search terms that must be driven to the new design

The implementer must explicitly grep these and either delete them or move them behind the new schema/store boundary:

- `Compare::`
- `Txn::new`
- `WatchOptions::new`
- `LeaseKeeper`
- `LeaseKeepAliveStream`
- `watch_stream`
- `leader_lease`
- `scope_prefix`
- `member_path`
- `leader_path`
- `switchover_path`
- `serde_json::from_slice`
- `serde_json::to_string`
- `EventType::Put`
- `EventType::Delete`
- `cfg.dcs.init`
- `DcsInitConfig`
- `dcs.init`
- `src/dcs/worker.rs`
- `src/dcs/state.rs`

The expected final rule is:

- DCS schema/key decode hits should land in `src/dcs/schema.rs`
- etcd transport hits should land in `src/dcs/store.rs`
- DCS runtime/business hits should land in `src/dcs/runtime.rs`

If those grep terms still appear scattered across unrelated DCS files after the rewrite, the task is not complete.
</description>

<acceptance_criteria>
- [ ] The task implementation extracts DCS schema design and key formats into a first-class boundary instead of leaving them spread across the worker/store logic.
- [ ] The final DCS layout is split into explicit `schema.rs`, `store.rs`, `runtime.rs`, and `startup.rs` responsibilities, with `command.rs` remaining the typed mutation entry point.
- [ ] No DCS path layout or key parsing remains hidden inside ad hoc command handlers or watch loops outside the schema module.
- [ ] No DCS JSON encode/decode remains hidden inside runtime/business logic outside the schema/store boundary.
- [ ] A small etcd framework is allowed and used only if it directly serves this split; the implementation does not build generic infra that is larger than the DCS problem it is fixing.
- [ ] `TypedEtcdRepo` and `TypedEtcdSession` or equivalent lifecycle-separated types exist and clearly isolate transport/bootstrap ownership from live scope/watch/lease ownership.
- [ ] The DCS runtime uses the schema/store layer and does not manually build keys, parse watch paths, or directly manipulate raw etcd transactions for DCS fields.
- [ ] The implementation explicitly audits and cleans adjacent config/test/consumer files that still depend on dead DCS shape or dead `dcs.init` support.
- [ ] If derive support is implemented, it is added only as a small follow-up over an already-clear handwritten schema split, and it does not hide business logic.
- [ ] If derive support is not implemented, that is still acceptable as long as the handwritten schema/store/runtime split is clean and the schema/key-format scattering problem is fully solved.
</acceptance_criteria>
