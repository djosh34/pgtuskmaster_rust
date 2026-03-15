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

- `EtcdRepo` is the long-lived transport/bootstrap owner.
  - It owns endpoints, auth, TLS, and the raw etcd client.
  - It knows how to establish a scope-bound session.
  - It does not know DCS business rules.
- `EtcdSession` is the live scope/watch/lease owner.
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

**Why one prefix watch is the right model**

The intended model here is not "watch each field one by one". The intended model is:

- one snapshot load of the schema prefix
- one watch on that same schema prefix
- one schema-owned apply layer that updates typed state from those events

For DCS that means watching one prefix such as `/{scope}` or `/{scope}/...`, not separate watches for leader, switchover, and members.

That makes sense because:

- the DCS schema is one keyspace
- the runtime wants one coherent typed cache
- reconnect/reload logic becomes much simpler if there is one scope watch
- one watch avoids field-by-field watch bookkeeping noise

Watching all keys in the entire etcd cluster usually does **not** make sense for this task because:

- DCS should only care about its own schema prefix
- watching the entire keyspace would force DCS schema code to ignore unrelated keys
- it would couple DCS to data it does not own
- it weakens the whole point of explicit keyspace separation

The etcd API itself is range/prefix oriented. That is why the client surface is built around key, range, prefix, and all-keys modes instead of "watch this arbitrary set of disjoint keys as one typed object". If the schema keys are meant to belong together, the right fix is to put them under one prefix and watch that prefix.

So the answer is:

- no, the design should not watch each key one by one
- yes, one prefix watch is the correct approach
- yes, watching all keys exists, but it is usually the wrong boundary for DCS

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

**Stringly timeout helper is rejected**

The task must **not** reintroduce helpers like:

```rust
timeout_etcd(future, "put key")
timeout_etcd(future, "snapshot load")
timeout_etcd(future, "watch start")
```

That is ugly, stringly, and weakly typed.

If timeout/context wrapping is needed, it must use a typed operation enum instead:

```rust
enum EtcdOp {
    Connect,
    StartPrefixWatch,
    LoadSnapshot,
    PutKey,
    DeleteKey,
    CompareAndSwapAbsent,
    GrantLease,
    KeepLeaseAlive,
}
```

and errors should carry that typed operation:

```rust
enum EtcdStoreError {
    Timeout { op: EtcdOp },
    Io { op: EtcdOp, message: String },
    Decode { key: String, message: String },
    Mutation(String),
}
```

The point is:

- no random strings
- no ad hoc timeout labels
- no "framework feeling" helper that hides weak typing

**Preferred mutation model**

Yes, the model can be much simpler than per-field imperative helpers.

The preferred model for this task is:

- runtime mutates typed state
- schema diff logic infers which etcd keys changed
- store applies the minimal required etcd operations

Example:

```rust
session
    .mutate(|state| {
        state.switchover = Some(target);
        Ok(())
    })
    .await?;
```

and:

```rust
session
    .mutate(|state| {
        state.switchover = None;
        Ok(())
    })
    .await?;
```

The schema/store layer should infer from the before/after typed state whether that means:

- `put /{scope}/switchover`
- `delete /{scope}/switchover`

That is not impossible. It is a valid and preferred simplification.

The same applies to:

- member upsert/delete
- leader acquire/release

The only caveat is that leader acquisition is not just "set value"; it also needs a guard such as "only create if absent" plus lease attachment. But that still fits the same model if the schema diff produces a guarded write plan instead of raw per-field API calls.

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
mod log_event;
mod runtime;
mod schema;
mod store;
mod startup;

pub(crate) use command::DcsHandle;
pub(crate) use startup::{bootstrap, BootstrapRequest, DcsAdvertisedEndpoints, DcsRuntime};
pub use schema::{
    ClusterMemberView, ClusterView, DcsMode, DcsView, LeadershipObservation, MemberPostgresView,
    NotTrustedView, SwitchoverView,
};
```

### Visibility rule for the whole task

`pub(crate)` must **not** be sprayed across internal DCS types.

The default rule is:

- private first
- `pub(super)` only when one sibling DCS module must use it
- `pub(crate)` only at the root DCS boundary when non-DCS crate code must use it
- `pub` only for the intentionally public read-only `DcsView` surface

The only acceptable `pub(crate)` exceptions in the final design are:

- `dcs::DcsHandle`
  - needed by HA and API so they can send typed DCS commands
- `dcs::bootstrap`
  - needed by `src/runtime/node.rs` as the crate composition root
- `dcs::BootstrapRequest`
  - needed by `src/runtime/node.rs` to construct DCS bootstrap input
- `dcs::DcsAdvertisedEndpoints`
  - needed by `src/runtime/node.rs` when deriving advertised endpoints from config
- `dcs::DcsRuntime`
  - needed by `src/runtime/node.rs` because bootstrap returns the DCS state/handle/worker bundle

Everything else in the DCS implementation should stay private or `pub(super)`.

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

    pub(super) fn starting() -> Self {
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
pub(super) struct DcsState {
    pub(super) leader: Option<LeaseEpoch>,
    pub(super) switchover: Option<SwitchoverTarget>,
    pub(super) members: BTreeMap<MemberId, MemberRecord>,
}

impl DcsState {
    pub(super) fn empty() -> Self {
        Self {
            leader: None,
            switchover: None,
            members: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct MemberRecord {
    pub(super) expires_at: UnixMillis,
    pub(super) postgres_target: PgTcpTarget,
    pub(super) postgres: MemberPostgresRecord,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(super) enum MemberPostgresRecord {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum LeaseAttachment {
    None,
    SessionEphemeral,
    TtlUntil(UnixMillis),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum SchemaWrite {
    Put {
        key: String,
        json: String,
        lease: LeaseAttachment,
    },
    Delete {
        key: String,
    },
    ClaimIfAbsent {
        key: String,
        json: String,
        lease: LeaseAttachment,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct EtcdDecodeError(String);

impl EtcdDecodeError {
    fn json(err: serde_json::Error) -> Self {
        Self(err.to_string())
    }

    fn unknown_key(key: &str) -> Self {
        Self(format!("unknown schema key `{key}`"))
    }
}

impl std::fmt::Display for EtcdDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.as_str())
    }
}

pub(super) struct DcsKvSchema;

impl DcsKvSchema {
    pub(super) fn prefix(scope: &str) -> String {
        format!("/{scope}")
    }

    pub(super) fn leader_key(scope: &str) -> String {
        format!("{}/leader", Self::prefix(scope))
    }

    pub(super) fn switchover_key(scope: &str) -> String {
        format!("{}/switchover", Self::prefix(scope))
    }

    pub(super) fn member_prefix(scope: &str) -> String {
        format!("{}/members/", Self::prefix(scope))
    }

    pub(super) fn member_key(scope: &str, member_id: &MemberId) -> String {
        format!("{}{member_id}", Self::member_prefix(scope))
    }

    pub(super) fn parse_member_id(scope: &str, key: &str) -> Option<MemberId> {
        let prefix = Self::member_prefix(scope);
        key.strip_prefix(prefix.as_str())
            .map(|value| MemberId(value.to_string()))
            .filter(|member_id| !member_id.0.is_empty())
    }

    pub(super) fn apply_put(
        state: &mut DcsState,
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

    pub(super) fn apply_delete(
        state: &mut DcsState,
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

    pub(super) fn diff(
        scope: &str,
        before: &DcsState,
        after: &DcsState,
    ) -> Result<Vec<SchemaWrite>, String> {
        let mut writes = Vec::new();

        if before.switchover != after.switchover {
            match &after.switchover {
                Some(target) => writes.push(SchemaWrite::Put {
                    key: Self::switchover_key(scope),
                    json: serde_json::to_string(target).map_err(|err| err.to_string())?,
                    lease: LeaseAttachment::None,
                }),
                None => writes.push(SchemaWrite::Delete {
                    key: Self::switchover_key(scope),
                }),
            }
        }

        if before.leader != after.leader {
            match (&before.leader, &after.leader) {
                (None, Some(epoch)) => writes.push(SchemaWrite::ClaimIfAbsent {
                    key: Self::leader_key(scope),
                    json: serde_json::to_string(epoch).map_err(|err| err.to_string())?,
                    lease: LeaseAttachment::SessionEphemeral,
                }),
                (Some(_), None) => writes.push(SchemaWrite::Delete {
                    key: Self::leader_key(scope),
                }),
                (Some(_), Some(epoch)) => writes.push(SchemaWrite::Put {
                    key: Self::leader_key(scope),
                    json: serde_json::to_string(epoch).map_err(|err| err.to_string())?,
                    lease: LeaseAttachment::SessionEphemeral,
                }),
                (None, None) => {}
            }
        }

        for member_id in before
            .members
            .keys()
            .chain(after.members.keys())
            .collect::<std::collections::BTreeSet<_>>()
        {
            match (before.members.get(member_id), after.members.get(member_id)) {
                (Some(_), None) => writes.push(SchemaWrite::Delete {
                    key: Self::member_key(scope, member_id),
                }),
                (None, Some(record)) | (Some(_), Some(record))
                    if before.members.get(member_id) != Some(record) =>
                {
                    writes.push(SchemaWrite::Put {
                        key: Self::member_key(scope, member_id),
                        json: serde_json::to_string(record).map_err(|err| err.to_string())?,
                        lease: LeaseAttachment::TtlUntil(record.expires_at),
                    });
                }
                _ => {}
            }
        }

        Ok(writes)
    }
}

pub(super) fn evaluate_mode(
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

pub(super) fn build_dcs_view(mode: DcsMode, state: &DcsState, now: UnixMillis) -> DcsView {
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

pub(super) fn build_local_member_record(
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

This layer must stay thin. The earlier handle-heavy sketch with `SingletonHandle`, `MapHandle`, `TypedEtcdRepo`, and `TypedEtcdSession` is **not** the preferred end state anymore.

For this task, the simpler preferred shape is:

- `EtcdRepo`
- `EtcdSession`
- `EtcdSession::load()`
- `EtcdSession::next()`
- `EtcdSession::mutate(...)`

No field-handle boilerplate.

No stringly timeout helper.

No generic-looking framework unless the handwritten version proves too repetitive.

```rust
use std::{str, time::Duration};

use etcd_client::{
    Certificate, Client, Compare, CompareOp, ConnectOptions, EventType, GetOptions, Identity,
    LeaseKeepAliveStream, LeaseKeeper, PutOptions, TlsOptions, Txn, TxnOp, WatchOptions,
    WatchResponse, WatchStream,
};
use thiserror::Error;
use tokio::time::Instant;

use crate::{
    config::{resolve_inline_or_path_bytes, resolve_secret_string, DcsClientConfig, DcsEndpoint},
};

use super::schema::{DcsKvSchema, DcsState, SchemaWrite};

const ETCD_TIMEOUT: Duration = Duration::from_secs(2);
const KEEPALIVE_LEAD_TIME: Duration = Duration::from_millis(250);
const MIN_TTL_SECONDS: i64 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EtcdOp {
    Connect,
    StartPrefixWatch,
    LoadSnapshot,
    PutKey,
    DeleteKey,
    CompareAndSwapAbsent,
    GrantLease,
    KeepLeaseAlive,
}

#[derive(Debug, Error)]
pub(super) enum EtcdStoreError {
    #[error("etcd timeout during {op:?}")]
    Timeout { op: EtcdOp },
    #[error("etcd io during {op:?}: {message}")]
    Io { op: EtcdOp, message: String },
    #[error("etcd decode failed for key `{key}`: {message}")]
    Decode { key: String, message: String },
    #[error("mutation rejected: {0}")]
    Mutation(String),
    #[error("missing session lease")]
    MissingSessionLease,
}

pub(super) struct EtcdRepo {
    client: Client,
}

pub(super) struct EtcdSession {
    client: Client,
    scope: String,
    cache: DcsState,
    watch_stream: WatchStream,
    session_lease: Option<SessionLease>,
}

struct SessionLease {
    lease_id: i64,
    keeper: LeaseKeeper,
    stream: LeaseKeepAliveStream,
    next_keepalive_at: Instant,
}

impl EtcdRepo {
    pub(super) async fn connect(
        endpoints: Vec<DcsEndpoint>,
        client_cfg: DcsClientConfig,
    ) -> Result<Self, EtcdStoreError> {
        let endpoint_urls = endpoints
            .into_iter()
            .map(|endpoint| endpoint.to_url())
            .collect::<Vec<_>>();
        let options = build_connect_options(client_cfg).await?;
        let client = run_with_timeout(
            EtcdOp::Connect,
            Client::connect(endpoint_urls, Some(options)),
        )
        .await?;
        Ok(Self { client })
    }

    pub(super) async fn session(&self, scope: String) -> Result<EtcdSession, EtcdStoreError> {
        let prefix = DcsKvSchema::prefix(scope.as_str());
        let watch_stream = run_with_timeout(
            EtcdOp::StartPrefixWatch,
            self.client
                .clone()
                .watch(prefix.as_str(), Some(WatchOptions::new().with_prefix())),
        )
        .await?;

        Ok(EtcdSession {
            client: self.client.clone(),
            scope,
            cache: DcsState::empty(),
            watch_stream,
            session_lease: None,
        })
    }
}

impl EtcdSession {
    pub(super) async fn load(&mut self) -> Result<DcsState, EtcdStoreError> {
        let prefix = DcsKvSchema::prefix(self.scope.as_str());
        let response = run_with_timeout(
            EtcdOp::LoadSnapshot,
            self.client
                .get(prefix.as_str(), Some(GetOptions::new().with_prefix())),
        )
        .await?;

        let mut next = DcsState::empty();
        for kv in response.kvs() {
            let key = str::from_utf8(kv.key())
                .map_err(|err| EtcdStoreError::Io {
                    op: EtcdOp::LoadSnapshot,
                    message: format!("utf8 key decode failed: {err}"),
                })?;
            DcsKvSchema::apply_put(&mut next, self.scope.as_str(), key, kv.value())
                .map_err(|err| EtcdStoreError::Decode {
                    key: key.to_string(),
                    message: err.to_string(),
                })?;
        }

        self.cache = next.clone();
        Ok(next)
    }

    pub(super) async fn next(&mut self) -> Result<Option<DcsState>, EtcdStoreError> {
        self.keepalive_if_needed().await?;
        let response = self
            .watch_stream
            .message()
            .await
            .map_err(|err| EtcdStoreError::Io {
                op: EtcdOp::StartPrefixWatch,
                message: format!("watch receive failed: {err}"),
            })?;
        let Some(response) = response else {
            return Ok(None);
        };
        self.apply_watch_response(response)?;
        Ok(Some(self.cache.clone()))
    }

    pub(super) async fn mutate(
        &mut self,
        mutate: impl FnOnce(&mut DcsState) -> Result<(), String>,
    ) -> Result<(), EtcdStoreError> {
        let before = self.cache.clone();
        let mut after = before.clone();
        mutate(&mut after).map_err(EtcdStoreError::Mutation)?;
        let plan = DcsKvSchema::diff(self.scope.as_str(), &before, &after)
            .map_err(EtcdStoreError::Mutation)?;
        self.apply_plan(plan).await?;
        self.cache = after;
        Ok(())
    }

    fn apply_watch_response(&mut self, response: WatchResponse) -> Result<(), EtcdStoreError> {
        for event in response.events() {
            let kv = event
                .kv()
                .ok_or_else(|| EtcdStoreError::Io {
                    op: EtcdOp::StartPrefixWatch,
                    message: "watch event missing kv".to_string(),
                })?;
            let key = str::from_utf8(kv.key())
                .map_err(|err| EtcdStoreError::Io {
                    op: EtcdOp::StartPrefixWatch,
                    message: format!("utf8 key decode failed: {err}"),
                })?;

            match event.event_type() {
                EventType::Put => {
                    DcsKvSchema::apply_put(&mut self.cache, self.scope.as_str(), key, kv.value())
                        .map_err(|err| EtcdStoreError::Decode {
                            key: key.to_string(),
                            message: err.to_string(),
                        })?;
                }
                EventType::Delete => {
                    DcsKvSchema::apply_delete(&mut self.cache, self.scope.as_str(), key)
                        .map_err(|err| EtcdStoreError::Decode {
                            key: key.to_string(),
                            message: err.to_string(),
                        })?;
                }
            }
        }
        Ok(())
    }

    async fn apply_plan(&mut self, plan: Vec<SchemaWrite>) -> Result<(), EtcdStoreError> {
        for write in plan {
            match write {
                SchemaWrite::Put {
                    key,
                    json,
                    lease,
                } => {
                    let options = self.put_options_for_lease(lease).await?;
                    run_with_timeout(
                        EtcdOp::PutKey,
                        self.client.put(key, json, options),
                    )
                    .await?;
                }
                SchemaWrite::Delete { key } => {
                    run_with_timeout(EtcdOp::DeleteKey, self.client.delete(key, None)).await?;
                }
                SchemaWrite::ClaimIfAbsent {
                    key,
                    json,
                    lease,
                } => {
                    let options = self.put_options_for_lease(lease).await?;
                    let txn = Txn::new()
                        .when(vec![Compare::version(key.as_str(), CompareOp::Equal, 0)])
                        .and_then(vec![TxnOp::put(key.as_str(), json, options)]);
                    let response =
                        run_with_timeout(EtcdOp::CompareAndSwapAbsent, self.client.txn(txn)).await?;
                    if !response.succeeded() {
                        return Err(EtcdStoreError::Mutation(
                            "claim-if-absent guard failed".to_string(),
                        ));
                    }
                }
            }
        }
        Ok(())
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
            .map_err(|err| EtcdStoreError::Io {
                op: EtcdOp::KeepLeaseAlive,
                message: format!("lease keepalive send failed: {err}"),
            })?;
        let response = lease
            .stream
            .message()
            .await
            .map_err(|err| EtcdStoreError::Io {
                op: EtcdOp::KeepLeaseAlive,
                message: format!("lease keepalive receive failed: {err}"),
            })?;
        let granted_ttl = response
            .ok_or(EtcdStoreError::MissingSessionLease)?
            .ttl();
        let ttl = granted_ttl.max(MIN_TTL_SECONDS);
        lease.next_keepalive_at =
            Instant::now() + Duration::from_secs(ttl as u64).saturating_sub(KEEPALIVE_LEAD_TIME);
        Ok(())
    }

    async fn put_options_for_lease(
        &mut self,
        lease: super::schema::LeaseAttachment,
    ) -> Result<Option<PutOptions>, EtcdStoreError> {
        match lease {
            super::schema::LeaseAttachment::None => Ok(None),
            super::schema::LeaseAttachment::SessionEphemeral => {
                if self.session_lease.is_none() {
                    self.session_lease = Some(create_session_lease(&mut self.client).await?);
                }
                let lease_id = self
                    .session_lease
                    .as_ref()
                    .map(|lease_state| lease_state.lease_id)
                    .ok_or(EtcdStoreError::MissingSessionLease)?;
                Ok(Some(PutOptions::new().with_lease(lease_id)))
            }
            super::schema::LeaseAttachment::TtlUntil(deadline) => {
                let now = crate::logging::system_now_unix_millis();
                let ttl_ms = deadline.0.saturating_sub(now);
                let ttl_seconds =
                    i64::try_from((ttl_ms / 1000).max(1)).map_err(|err| EtcdStoreError::Io {
                        op: EtcdOp::GrantLease,
                        message: err.to_string(),
                    })?;
                let lease = run_with_timeout(
                    EtcdOp::GrantLease,
                    self.client.lease_grant(ttl_seconds, None),
                )
                .await?;
                Ok(Some(PutOptions::new().with_lease(lease.id())))
            }
        }
    }
}

async fn build_connect_options(
    client_cfg: DcsClientConfig,
) -> Result<ConnectOptions, EtcdStoreError> {
    let mut options = ConnectOptions::new().with_timeout(ETCD_TIMEOUT);

    if let crate::config::DcsAuthConfig::Basic { username, password } = client_cfg.auth {
        let password = resolve_secret_string(&password).map_err(|err| EtcdStoreError::Io {
            op: EtcdOp::Connect,
            message: format!("dcs password resolve failed: {err}"),
        })?;
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
            let bytes = resolve_inline_or_path_bytes(&ca_cert).map_err(|err| EtcdStoreError::Io {
                op: EtcdOp::Connect,
                message: format!("dcs ca resolve failed: {err}"),
            })?;
            tls = tls.ca_certificate(Certificate::from_pem(bytes));
        }
        if let Some(identity_cfg) = identity {
            let cert = resolve_inline_or_path_bytes(&identity_cfg.cert).map_err(|err| EtcdStoreError::Io {
                op: EtcdOp::Connect,
                message: format!("dcs client cert resolve failed: {err}"),
            })?;
            let key = resolve_inline_or_path_bytes(&identity_cfg.key).map_err(|err| EtcdStoreError::Io {
                op: EtcdOp::Connect,
                message: format!("dcs client key resolve failed: {err}"),
            })?;
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
    let granted = run_with_timeout(EtcdOp::GrantLease, client.lease_grant(2, None)).await?;
    let lease_id = granted.id();
    let (keeper, stream) = client
        .lease_keep_alive(lease_id)
        .await
        .map_err(|err| EtcdStoreError::Io {
            op: EtcdOp::KeepLeaseAlive,
            message: format!("lease keepalive open failed: {err}"),
        })?;
    Ok(SessionLease {
        lease_id,
        keeper,
        stream,
        next_keepalive_at: Instant::now(),
    })
}

async fn run_with_timeout<T>(
    op: EtcdOp,
    future: impl std::future::Future<Output = Result<T, etcd_client::Error>>,
) -> Result<T, EtcdStoreError> {
    tokio::time::timeout(ETCD_TIMEOUT, future)
        .await
        .map_err(|_| EtcdStoreError::Timeout { op })?
        .map_err(|err| EtcdStoreError::Io {
            op,
            message: err.to_string(),
        })
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
    schema::{build_dcs_view, build_local_member_record, evaluate_mode, DcsState, DcsView},
    store::{EtcdRepo, EtcdSession},
    DcsHandle,
};

pub(super) struct DcsRuntime {
    pub(super) state: StateSubscriber<DcsView>,
    pub(super) handle: DcsHandle,
    pub(super) worker: DcsWorker,
}

pub(super) struct DcsWorker {
    ctx: DcsWorkerCtx,
}

struct DcsWorkerCtx {
    identity: NodeIdentity,
    advertised_postgres: PgTcpTarget,
    member_ttl_ms: u64,
    repo: EtcdRepo,
    cache: DcsState,
    state_tx: StatePublisher<DcsView>,
    rx: tokio::sync::mpsc::UnboundedReceiver<DcsCommand>,
    _log: LogSender,
}

pub(super) struct DcsRuntimeRequest {
    pub(super) identity: NodeIdentity,
    pub(super) advertised_postgres: PgTcpTarget,
    pub(super) member_ttl_ms: u64,
    pub(super) repo: EtcdRepo,
    pub(super) log: LogSender,
}

pub(super) fn bootstrap_runtime(request: DcsRuntimeRequest) -> DcsRuntime {
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
    pub(super) async fn run(mut self) -> Result<(), WorkerError> {
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
    session: &mut EtcdSession,
    command: DcsCommand,
) -> Result<(), WorkerError> {
    match command {
        DcsCommand::AcquireLeadership => {
            let epoch = LeaseEpoch {
                holder: ctx.identity.member_id.clone(),
                generation: now_unix_millis()?.0,
            };
            let self_id = ctx.identity.member_id.clone();
            session
                .mutate(|state| {
                    if state.leader.is_none() {
                        state.leader = Some(epoch);
                    }
                    if state
                        .leader
                        .as_ref()
                        .is_some_and(|leader| leader.holder == self_id)
                    {
                        return Ok(());
                    }
                    Ok(())
                })
                .await
                .map_err(to_worker_error)?;
        }
        DcsCommand::ReleaseLeadership => {
            session
                .mutate(|state| {
                    if state
                        .leader
                        .as_ref()
                        .is_some_and(|epoch| epoch.holder == ctx.identity.member_id)
                    {
                        state.leader = None;
                    }
                    Ok(())
                })
                .await
                .map_err(to_worker_error)?;
        }
        DcsCommand::PublishSwitchover(target) => {
            session
                .mutate(|state| {
                    state.switchover = Some(target);
                    Ok(())
                })
                .await
                .map_err(to_worker_error)?;
        }
        DcsCommand::ClearSwitchover => {
            session
                .mutate(|state| {
                    state.switchover = None;
                    Ok(())
                })
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
            let member_id = ctx.identity.member_id.clone();
            session
                .mutate(|state| {
                    state.members.insert(member_id, record);
                    Ok(())
                })
                .await
                .map_err(to_worker_error)?;
        }
        DcsCommand::RemoveLocalMember => {
            session
                .mutate(|state| {
                    state.members.remove(&ctx.identity.member_id);
                    if state
                        .leader
                        .as_ref()
                        .is_some_and(|epoch| epoch.holder == ctx.identity.member_id)
                    {
                        state.leader = None;
                    }
                    Ok(())
                })
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
pub(super) enum DcsCommand {
    RefreshLocalMember(PgInfoState),
    RemoveLocalMember,
    AcquireLeadership,
    ReleaseLeadership,
    PublishSwitchover(SwitchoverTarget),
    ClearSwitchover,
}

#[derive(Clone)]
pub(super) struct DcsHandle {
    sender: mpsc::UnboundedSender<DcsCommand>,
}

pub(super) type DcsCommandInbox = mpsc::UnboundedReceiver<DcsCommand>;

#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub(super) enum DcsHandleError {
    #[error("dcs command channel closed")]
    ChannelClosed,
}

pub(super) fn dcs_command_channel() -> (DcsHandle, DcsCommandInbox) {
    let (sender, receiver) = mpsc::unbounded_channel();
    (DcsHandle { sender }, receiver)
}

impl DcsHandle {
    pub(super) fn refresh_local_member(&self, pg_state: PgInfoState) -> Result<(), DcsHandleError> {
        self.send(DcsCommand::RefreshLocalMember(pg_state))
    }

    pub(super) fn remove_local_member(&self) -> Result<(), DcsHandleError> {
        self.send(DcsCommand::RemoveLocalMember)
    }

    pub(super) fn acquire_leadership(&self) -> Result<(), DcsHandleError> {
        self.send(DcsCommand::AcquireLeadership)
    }

    pub(super) fn release_leadership(&self) -> Result<(), DcsHandleError> {
        self.send(DcsCommand::ReleaseLeadership)
    }

    pub(super) fn publish_switchover(
        &self,
        target: SwitchoverTarget,
    ) -> Result<(), DcsHandleError> {
        self.send(DcsCommand::PublishSwitchover(target))
    }

    pub(super) fn clear_switchover(&self) -> Result<(), DcsHandleError> {
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
    store::EtcdRepo,
};

pub(super) struct DcsAdvertisedEndpoints {
    pub(super) postgres: PgTcpTarget,
}

pub(super) struct BootstrapRequest {
    pub(super) identity: NodeIdentity,
    pub(super) endpoints: Vec<DcsEndpoint>,
    pub(super) client: DcsClientConfig,
    pub(super) member_ttl_ms: u64,
    pub(super) advertised: DcsAdvertisedEndpoints,
    pub(super) log: LogSender,
}

impl DcsAdvertisedEndpoints {
    pub(super) fn from_config(cfg: &RuntimeConfig) -> Result<Self, String> {
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

pub(super) async fn bootstrap(request: BootstrapRequest) -> Result<DcsRuntime, String> {
    let repo = EtcdRepo::connect(request.endpoints, request.client)
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

## Alternative derive branch

Calling this "optional files" was the wrong framing.

The correct framing is:

- the default branch of this task is the handwritten schema/store split above
- there is a second deliberate branch where the team chooses to add derive support

If the derive branch is chosen, the files below are required for that branch.

If the derive branch is not chosen, they should not exist at all.

The derive branch is allowed only after the handwritten schema split is already explicit and understandable.

Because the main design was simplified away from field handles toward `diff(...)` plus `SchemaWrite`, the derive branch must follow that same simpler direction. It must not drag the task back toward the older handle-heavy store design.

The macro must generate:

- `DcsKvSchema`
- `DcsKvSchema::apply_put(...)`
- `DcsKvSchema::apply_delete(...)`
- `DcsKvSchema::diff(...)`
- `SchemaWrite` emission metadata

It must not generate runtime logic.

### Derive-branch file: `crates/pgtm_etcd_derive/Cargo.toml`

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

### Derive-branch file: `crates/pgtm_etcd_derive/src/lib.rs`

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
                    pub(super) struct #helper_ident;

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
                    pub(super) struct #helper_ident;

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
        pub(super) struct DcsKvSchema;

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
- `pub(crate)`
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
- [ ] `pub(crate)` is not sprayed across internal DCS implementation types; the final code uses private-by-default visibility, `pub(super)` for sibling DCS sharing, and only the explicitly justified root DCS crate-boundary exceptions remain `pub(crate)`.
- [ ] A small etcd framework is allowed and used only if it directly serves this split; the implementation does not build generic infra that is larger than the DCS problem it is fixing.
- [ ] `EtcdRepo` and `EtcdSession` or equivalent lifecycle-separated types exist and clearly isolate transport/bootstrap ownership from live scope/watch/lease ownership.
- [ ] The live watch model is one schema-prefix watch plus snapshot reloads, not a pile of field-by-field watches.
- [ ] The DCS runtime uses the schema/store layer and does not manually build keys, parse watch paths, or directly manipulate raw etcd transactions for DCS fields.
- [ ] The implementation explicitly audits and cleans adjacent config/test/consumer files that still depend on dead DCS shape or dead `dcs.init` support.
- [ ] If derive support is implemented, it is added only as a small follow-up over an already-clear handwritten schema split, and it does not hide business logic.
- [ ] If derive support is not implemented, that is still acceptable as long as the handwritten schema/store/runtime split is clean and the schema/key-format scattering problem is fully solved.
</acceptance_criteria>
