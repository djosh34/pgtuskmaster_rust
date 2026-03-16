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

## Deep Verification Results

This section documents a verified code audit of the current DCS implementation against the proposed design. Every claim below is backed by inspection of the actual source.

### Feasibility: Confirmed

The proposed schema/store/runtime split is fully viable against the current codebase. Evidence:

1. **Clean consumer boundary.** All external consumers (ha, process, api, cli, runtime, dev_support) depend only on `DcsView`, `DcsHandle`, `DcsMode`, `ClusterView`, `ClusterMemberView`, `MemberPostgresView`, `LeadershipObservation`, `SwitchoverView`, and `NotTrustedView`. No external module imports etcd types, etcd client, or raw DCS key paths. (Verified: `src/ha/`, `src/process/`, `src/api/`, `src/cli/`, `src/runtime/node.rs`, `src/dev_support/`)

2. **Isolated etcd dependency.** All `etcd_client::` imports are confined to `src/dcs/worker.rs`. No other file in the crate imports etcd types. The split into store.rs (etcd transport) is a clean extraction. (Verified: `grep -r 'etcd_client' src/`)

3. **Existing proc-macro pattern.** The workspace already has `crates/pgtm_log_derive/` as a proc-macro crate, so optional derive support follows an established pattern. (Verified: `Cargo.toml` workspace members)

4. **Bootstrap boundary is already correct.** `src/runtime/node.rs` calls `crate::dcs::startup::bootstrap(DcsRuntimeRequest { ... })` and receives `DcsRuntime { state, handle, worker }`. The proposed design preserves this exact interface shape. (Verified: `src/runtime/node.rs:94`)

5. **State channel pattern is reusable.** The `StatePublisher<DcsView>` / `StateSubscriber<DcsView>` pattern from `src/state/` is already used. The new runtime.rs just calls `new_state_channel(DcsView::starting())` exactly as today. (Verified: `src/dcs/startup.rs:62`)

### Additional Simplifications Found (Beyond Original Task)

The following simplifications reduce code further and improve type safety. They should be incorporated into the implementation.

#### 1. Merge `MemberPostgresRecord` into `MemberPostgresView` (eliminates ~90 lines)

**Current state:** `MemberPostgresRecord` (internal, stored in etcd) and `MemberPostgresView` (public) are structurally identical enums with identical serde attributes (`#[serde(tag = "kind", rename_all = "snake_case")]`). The `build_member_view` function manually maps every field between them, with one special case: downgrading non-leader Primary members to Unknown.

**Fix:** Use `MemberPostgresView` as the single canonical type for both etcd storage and public display. The "downgrade non-leader Primary" transformation — which protects consumers from split-brain confusion by marking members that claim Primary without holding the authoritative leader record — becomes a simple `sanitize_for_view()` method during view building, not a full parallel type hierarchy.

This eliminates:
- The entire `MemberPostgresRecord` enum definition (~20 lines)
- The `build_member_view` function's field-by-field mapping (~50 lines)
- The `record_timeline()` and `record_system_identifier()` helper functions (~25 lines)

And replaces them with one small sanitize method (~15 lines).

#### 2. Remove `expires_at` from `MemberRecord` — trust etcd leases (eliminates clock dependency)

**Current state:** The task proposes `MemberRecord { expires_at: UnixMillis, ... }` and uses `member.expires_at.0 > now.0` in `evaluate_mode()` and `build_dcs_view()` to filter live members.

**Problem:** This is redundant with etcd's lease mechanism. When a member's etcd lease expires, etcd automatically deletes the key and fires a watch delete event. The current codebase already relies on this — `evaluate_mode` just checks `cache.member_records.contains_key(self_id)` without any clock-based filtering. Adding application-level expiry checking on top of etcd leases introduces clock skew sensitivity and unnecessary complexity.

**Fix:** Remove `expires_at` from `MemberRecord`. The simplified record becomes:

```rust
pub(super) struct MemberRecord {
    pub(super) postgres_target: PgTcpTarget,
    pub(super) postgres: MemberPostgresView,  // reuses the public type
}
```

And `evaluate_mode` stays clock-free:

```rust
fn evaluate_mode(etcd_reachable: bool, state: &DcsState, self_id: &MemberId) -> DcsMode {
    if !etcd_reachable { return DcsMode::NotTrusted; }
    if !state.members.contains_key(self_id) { return DcsMode::Degraded; }
    if state.members.is_empty() { return DcsMode::Degraded; }
    DcsMode::Coordinated
}
```

This also removes the `UnixMillis` import from schema.rs entirely.

#### 3. Simplify `LeaseAttachment::TtlUntil(UnixMillis)` → `LeaseAttachment::Ttl(u64)`

Since `expires_at` is removed from records, the `TtlUntil` variant is unnecessary. Replace with `Ttl(u64)` which holds the TTL in milliseconds. The `diff()` function takes `member_ttl_ms: u64` as a parameter. This is cleaner because the lease TTL is a config-level concern, not a record-level concern.

```rust
enum LeaseAttachment {
    None,              // switchover
    SessionEphemeral,  // leader
    Ttl(u64),          // members (TTL in ms, from config)
}
```

The store layer translates `Ttl(ms)` to an etcd lease grant.

#### 4. Remove wrapper newtypes `LeadershipRecord` and `SwitchoverRecord`

**Current state (in state.rs):** `LeadershipRecord { epoch: LeaseEpoch }` and `SwitchoverRecord { target: SwitchoverTarget }` are single-field wrapper structs. They add indirection without safety.

**The task already does this correctly** by storing `Option<LeaseEpoch>` and `Option<SwitchoverTarget>` directly in `DcsState`. Confirmed this is the right call.

#### 5. Remove `MemberLeaseRecord` entirely

**Current state (in state.rs):** `MemberLeaseRecord { owner: MemberId, ttl_ms: u64 }` is stored inside each `MemberRecord`. The `owner` field is redundant with the BTreeMap key (the member ID). The `ttl_ms` is only used for creating the etcd lease, which is a transport concern, not a schema concern.

**Fix:** Remove `MemberLeaseRecord`. The TTL comes from config (passed to `diff()`), and the owner is the map key. This is consistent with simplification #2.

### Summary of type count reduction

| Concern | Current types | Task proposed | After simplifications |
|---------|--------------|--------------|----------------------|
| Postgres state | `MemberPostgresRecord` + `MemberPostgresView` | `MemberPostgresRecord` + `MemberPostgresView` | **`MemberPostgresView` only** |
| Member record | `MemberRecord` + `MemberLeaseRecord` | `MemberRecord` (with `expires_at`) | **`MemberRecord` (2 fields, no lease/expiry)** |
| Leadership | `LeadershipRecord` | `Option<LeaseEpoch>` | `Option<LeaseEpoch>` |
| Switchover | `SwitchoverRecord` | `Option<SwitchoverTarget>` | `Option<SwitchoverTarget>` |
| Lease variant | N/A | `TtlUntil(UnixMillis)` | **`Ttl(u64)`** |

Net eliminated types: `MemberPostgresRecord`, `MemberLeaseRecord`, `LeadershipRecord`, `SwitchoverRecord` (4 types removed from current code). The task already eliminates 2 of those; these additional simplifications eliminate 2 more plus remove the clock dependency from the schema layer.

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
        SystemIdentifier, TimelineId,
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

impl MemberPostgresView {
    /// Extract the timeline from any variant, for carry-forward during Unknown states.
    pub fn timeline(&self) -> Option<TimelineId> {
        match self {
            Self::Unknown { timeline, .. } => *timeline,
            Self::Primary { committed_wal, .. } => committed_wal.timeline,
            Self::Replica {
                replay_wal,
                follow_wal,
                ..
            } => replay_wal
                .as_ref()
                .and_then(|pos| pos.timeline)
                .or_else(|| follow_wal.as_ref().and_then(|pos| pos.timeline)),
        }
    }

    /// Extract system_identifier from any variant, for carry-forward during Unknown states.
    pub fn system_identifier(&self) -> Option<SystemIdentifier> {
        match self {
            Self::Unknown { system_identifier, .. }
            | Self::Primary { system_identifier, .. }
            | Self::Replica { system_identifier, .. } => *system_identifier,
        }
    }
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

// ── Internal schema state (stored in etcd, visible only within dcs module) ──

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

// MemberRecord stores only the domain data.
// Lease TTL is a transport concern (etcd handles expiry via lease grant),
// and the member ID is the BTreeMap key — neither belongs in the record.
// MemberPostgresView is reused directly (no separate MemberPostgresRecord).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct MemberRecord {
    pub(super) postgres_target: PgTcpTarget,
    pub(super) postgres: MemberPostgresView,
}

// Lease metadata for schema writes. This is a transport-level concern.
// The schema `diff()` emits these; the store layer translates them to etcd leases.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum LeaseAttachment {
    /// No lease (e.g. switchover key).
    None,
    /// Attached to the session-scoped ephemeral lease (e.g. leader key).
    SessionEphemeral,
    /// Attached to a TTL-based lease with the given duration in milliseconds.
    /// The TTL comes from config, not from the record. Etcd handles actual expiry.
    Ttl(u64),
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
        member_ttl_ms: u64,
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
                        lease: LeaseAttachment::Ttl(member_ttl_ms),
                    });
                }
                _ => {}
            }
        }

        Ok(writes)
    }
}

// evaluate_mode has no clock dependency. Etcd leases handle member expiry.
// If etcd deleted the key (lease expired), the member won't be in state.members.
pub(super) fn evaluate_mode(
    etcd_reachable: bool,
    state: &DcsState,
    self_id: &MemberId,
) -> DcsMode {
    if !etcd_reachable {
        return DcsMode::NotTrusted;
    }

    if !state.members.contains_key(self_id) {
        return DcsMode::Degraded;
    }

    DcsMode::Coordinated
}

// build_dcs_view doesn't filter members by time — etcd lease expiry already
// removed dead members from state. It uses MemberPostgresView directly with
// a sanitize step to downgrade non-leader primaries.
pub(super) fn build_dcs_view(mode: DcsMode, state: &DcsState) -> DcsView {
    let authoritative_leader = state.leader.as_ref().map(|epoch| &epoch.holder);
    let cluster = ClusterView {
        members: state
            .members
            .iter()
            .map(|(member_id, member)| {
                (
                    member_id.clone(),
                    ClusterMemberView {
                        postgres_target: member.postgres_target.clone(),
                        postgres: sanitize_member_postgres(
                            member_id,
                            &member.postgres,
                            authoritative_leader,
                        ),
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

// Sanitizes a member's postgres state for public view.
// If a member claims to be Primary but is NOT the authoritative leader,
// downgrade it to Unknown to prevent split-brain confusion in consumers.
fn sanitize_member_postgres(
    member_id: &MemberId,
    postgres: &MemberPostgresView,
    authoritative_leader: Option<&MemberId>,
) -> MemberPostgresView {
    match postgres {
        MemberPostgresView::Primary {
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
        other => other.clone(),
    }
}

pub(super) fn build_local_member_record(
    postgres_target: &PgTcpTarget,
    pg_state: &PgInfoState,
    previous_record: Option<&MemberRecord>,
) -> MemberRecord {
    let postgres = match pg_state {
        PgInfoState::Unknown { common } => MemberPostgresView::Unknown {
            readiness: common.readiness.clone(),
            timeline: common.timeline.or_else(|| previous_record.and_then(|r| r.postgres.timeline())),
            system_identifier: common
                .system_identifier
                .or_else(|| previous_record.and_then(|r| r.postgres.system_identifier())),
        },
        PgInfoState::Primary { common, wal_lsn, .. } => MemberPostgresView::Primary {
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
        postgres_target: postgres_target.clone(),
        postgres,
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
        member_ttl_ms: u64,
        mutate: impl FnOnce(&mut DcsState) -> Result<(), String>,
    ) -> Result<(), EtcdStoreError> {
        let before = self.cache.clone();
        let mut after = before.clone();
        mutate(&mut after).map_err(EtcdStoreError::Mutation)?;
        let plan = DcsKvSchema::diff(self.scope.as_str(), &before, &after, member_ttl_ms)
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
            super::schema::LeaseAttachment::Ttl(ttl_ms) => {
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
        SwitchoverTarget, WorkerError,
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
                generation: now_generation()?,
            };
            let self_id = ctx.identity.member_id.clone();
            session
                .mutate(ctx.member_ttl_ms, |state| {
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
                .mutate(ctx.member_ttl_ms, |state| {
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
                .mutate(ctx.member_ttl_ms, |state| {
                    state.switchover = Some(target);
                    Ok(())
                })
                .await
                .map_err(to_worker_error)?;
        }
        DcsCommand::ClearSwitchover => {
            session
                .mutate(ctx.member_ttl_ms, |state| {
                    state.switchover = None;
                    Ok(())
                })
                .await
                .map_err(to_worker_error)?;
        }
        DcsCommand::RefreshLocalMember(pg_state) => {
            let record = build_local_member_record(
                &ctx.advertised_postgres,
                &pg_state,
                ctx.cache.members.get(&ctx.identity.member_id),
            );
            let member_id = ctx.identity.member_id.clone();
            session
                .mutate(ctx.member_ttl_ms, |state| {
                    state.members.insert(member_id, record);
                    Ok(())
                })
                .await
                .map_err(to_worker_error)?;
        }
        DcsCommand::RemoveLocalMember => {
            session
                .mutate(ctx.member_ttl_ms, |state| {
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
    let mode = evaluate_mode(etcd_reachable, &ctx.cache, &ctx.identity.member_id);
    let view = build_dcs_view(mode, &ctx.cache);
    ctx.state_tx
        .publish(view)
        .map_err(|err| WorkerError::Message(format!("dcs publish failed: {err}")))
}

/// Used only for leader epoch generation — not for record expiry.
fn now_generation() -> Result<u64, WorkerError> {
    let elapsed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|err| WorkerError::Message(format!("system clock before unix epoch: {err}")))?;
    let millis = u64::try_from(elapsed.as_millis())
        .map_err(|err| WorkerError::Message(format!("unix millis conversion failed: {err}")))?;
    Ok(millis)
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

This is where derive support now makes real sense.

With the `diff(...)` design, the repetitive parts are now obvious:

- field path rendering
- `apply_put(...)` routing
- `apply_delete(...)` routing
- `diff(...)` emission of `SchemaWrite`
- lease attachment metadata

That is exactly the kind of repetition a proc macro can remove cleanly.

So the answer is:

- before the redesign, macro-first looked wrong because it was trying to hide a confused model
- after the redesign, macro support looks reasonable because the handwritten model is now explicit and the remaining duplication is mostly schema glue

The macro is still only good if it generates obvious code from a very small attribute language. It becomes bad again if it starts generating runtime logic or turning the schema into a hidden DSL.

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, Data, DeriveInput, Fields, LitStr, Type};

#[proc_macro_derive(EtcdSchema, attributes(etcd))]
pub fn derive_etcd_schema(input: TokenStream) -> TokenStream {
    derive_etcd_schema_impl(parse_macro_input!(input as DeriveInput))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn derive_etcd_schema_impl(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let state_ident = input.ident.clone();
    let data = match &input.data {
        Data::Struct(data) => data,
        _ => {
            return Err(syn::Error::new(
                input.ident.span(),
                "EtcdSchema can only be derived for structs",
            ));
        }
    };

    let prefix = parse_container_prefix(&input.attrs)?;
    let fields = match &data.fields {
        Fields::Named(fields) => fields.named.iter().cloned().collect::<Vec<_>>(),
        _ => {
            return Err(syn::Error::new(
                state_ident.span(),
                "EtcdSchema requires named fields",
            ));
        }
    };

    let mut put_arms = Vec::new();
    let mut delete_arms = Vec::new();
    let mut apply_put_arms = Vec::new();
    let mut apply_delete_arms = Vec::new();
    let mut diff_arms = Vec::new();

    for field in fields {
        let field_ident = field
            .ident
            .clone()
            .ok_or_else(|| syn::Error::new(state_ident.span(), "missing field ident"))?;
        let meta = parse_field_meta(&field.attrs)?;

        match meta {
            FieldMeta::Singleton { path, lease } => {
                let value_ty = option_inner_type(&field.ty)?;
                let lease_expr = singleton_lease_expr(&lease, &field_ident);
                let key_expr = quote! {
                    format!("{}/{}", render_prefix(#prefix, scope), #path)
                };

                apply_put_arms.push(quote! {
                    if key == #key_expr {
                        state.#field_ident = Some(
                            serde_json::from_slice::<#value_ty>(value)
                                .map_err(crate::dcs::schema::EtcdDecodeError::json)?
                        );
                        return Ok(());
                    }
                });

                apply_delete_arms.push(quote! {
                    if key == #key_expr {
                        state.#field_ident = None;
                        return Ok(());
                    }
                });

                diff_arms.push(quote! {
                    if before.#field_ident != after.#field_ident {
                        match (&before.#field_ident, &after.#field_ident) {
                            (None, Some(value)) => writes.push(crate::dcs::schema::SchemaWrite::Put {
                                key: #key_expr,
                                json: serde_json::to_string(value).map_err(|err| err.to_string())?,
                                lease: #lease_expr,
                            }),
                            (Some(_), None) => writes.push(crate::dcs::schema::SchemaWrite::Delete {
                                key: #key_expr,
                            }),
                            (Some(_), Some(value)) => writes.push(crate::dcs::schema::SchemaWrite::Put {
                                key: #key_expr,
                                json: serde_json::to_string(value).map_err(|err| err.to_string())?,
                                lease: #lease_expr,
                            }),
                            (None, None) => {}
                        }
                    }
                });
            }
            FieldMeta::Map {
                prefix_path,
                key_ty,
                lease,
            } => {
                let value_ty = btreemap_value_type(&field.ty)?;
                let lease_expr = map_lease_expr(&lease);
                let map_prefix_expr = quote! {
                    format!("{}/{}/", render_prefix(#prefix, scope), #prefix_path)
                };

                apply_put_arms.push(quote! {
                    if let Some(encoded_key) = key.strip_prefix(#map_prefix_expr.as_str()) {
                        let decoded_key = <#key_ty as crate::dcs::schema::EtcdKeyCodec>::decode_key(encoded_key)?;
                        let decoded_value = serde_json::from_slice::<#value_ty>(value)
                            .map_err(crate::dcs::schema::EtcdDecodeError::json)?;
                        state.#field_ident.insert(decoded_key, decoded_value);
                        return Ok(());
                    }
                });

                apply_delete_arms.push(quote! {
                    if let Some(encoded_key) = key.strip_prefix(#map_prefix_expr.as_str()) {
                        let decoded_key = <#key_ty as crate::dcs::schema::EtcdKeyCodec>::decode_key(encoded_key)?;
                        state.#field_ident.remove(&decoded_key);
                        return Ok(());
                    }
                });

                diff_arms.push(quote! {
                    for key in before
                        .#field_ident
                        .keys()
                        .chain(after.#field_ident.keys())
                        .collect::<std::collections::BTreeSet<_>>()
                    {
                        match (before.#field_ident.get(key), after.#field_ident.get(key)) {
                            (Some(_), None) => writes.push(crate::dcs::schema::SchemaWrite::Delete {
                                key: format!("{}{}", #map_prefix_expr, key.encode_key()),
                            }),
                            (None, Some(value)) | (Some(_), Some(value))
                                if before.#field_ident.get(key) != Some(value) =>
                            {
                                writes.push(crate::dcs::schema::SchemaWrite::Put {
                                    key: format!("{}{}", #map_prefix_expr, key.encode_key()),
                                    json: serde_json::to_string(value).map_err(|err| err.to_string())?,
                                    lease: #lease_expr(value),
                                });
                            }
                            _ => {}
                        }
                    }
                });
            }
        }
    }

    Ok(quote! {
        pub(super) struct DcsKvSchema;

        impl DcsKvSchema {
            pub(super) fn prefix(scope: &str) -> String {
                render_prefix(#prefix, scope)
            }

            pub(super) fn apply_put(
                state: &mut #state_ident,
                scope: &str,
                key: &str,
                value: &[u8],
            ) -> Result<(), crate::dcs::schema::EtcdDecodeError> {
                #(#apply_put_arms)*
                Err(crate::dcs::schema::EtcdDecodeError::unknown_key(key))
            }

            pub(super) fn apply_delete(
                state: &mut #state_ident,
                scope: &str,
                key: &str,
            ) -> Result<(), crate::dcs::schema::EtcdDecodeError> {
                #(#apply_delete_arms)*
                Err(crate::dcs::schema::EtcdDecodeError::unknown_key(key))
            }

            pub(super) fn diff(
                scope: &str,
                before: &#state_ident,
                after: &#state_ident,
            ) -> Result<Vec<crate::dcs::schema::SchemaWrite>, String> {
                let mut writes = Vec::new();
                #(#diff_arms)*
                Ok(writes)
            }
        }
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
    TtlMsField(syn::Ident),
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
                        lease = LeaseMeta::TtlMsField(lease_meta.value()?.parse()?);
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
            key_ty: key_ty.ok_or_else(|| {
                syn::Error::new(proc_macro2::Span::call_site(), "map field requires key = ...")
            })?,
            lease,
        }),
        _ => Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "field requires exactly one of singleton/map",
        )),
    }
}

fn singleton_lease_expr(
    lease: &LeaseMeta,
    _field_ident: &syn::Ident,
) -> proc_macro2::TokenStream {
    match lease {
        LeaseMeta::None => quote! { crate::dcs::schema::LeaseAttachment::None },
        LeaseMeta::SessionEphemeral => {
            quote! { crate::dcs::schema::LeaseAttachment::SessionEphemeral }
        }
        LeaseMeta::TtlMsField(field) => {
            quote! { crate::dcs::schema::LeaseAttachment::Ttl(value.#field) }
        }
    }
}

fn map_lease_expr(lease: &LeaseMeta) -> proc_macro2::TokenStream {
    match lease {
        LeaseMeta::None => quote! { |_| crate::dcs::schema::LeaseAttachment::None },
        LeaseMeta::SessionEphemeral => {
            quote! { |_| crate::dcs::schema::LeaseAttachment::SessionEphemeral }
        }
        LeaseMeta::TtlMsField(field) => {
            quote! { |value| crate::dcs::schema::LeaseAttachment::Ttl(value.#field) }
        }
    }
}

fn render_prefix(template: &str, scope: &str) -> String {
    template.replace("{scope}", scope)
}

fn option_inner_type(ty: &Type) -> syn::Result<Type> {
    extract_generic_type(ty, "Option")
}

fn btreemap_value_type(ty: &Type) -> syn::Result<Type> {
    let syn::Type::Path(path) = ty else {
        return Err(syn::Error::new(ty.span(), "expected BTreeMap type"));
    };
    let segment = path
        .path
        .segments
        .last()
        .ok_or_else(|| syn::Error::new(ty.span(), "missing path segment"))?;
    if segment.ident != "BTreeMap" {
        return Err(syn::Error::new(ty.span(), "expected BTreeMap type"));
    }
    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
        return Err(syn::Error::new(ty.span(), "expected generic BTreeMap type"));
    };
    let mut iter = args.args.iter();
    let _key = iter.next();
    let Some(syn::GenericArgument::Type(value_ty)) = iter.next() else {
        return Err(syn::Error::new(ty.span(), "missing BTreeMap value type"));
    };
    Ok(value_ty.clone())
}

fn extract_generic_type(ty: &Type, expected: &str) -> syn::Result<Type> {
    let syn::Type::Path(path) = ty else {
        return Err(syn::Error::new(ty.span(), "expected generic type"));
    };
    let segment = path
        .path
        .segments
        .last()
        .ok_or_else(|| syn::Error::new(ty.span(), "missing path segment"))?;
    if segment.ident != expected {
        return Err(syn::Error::new(
            ty.span(),
            format!("expected {expected}<...>"),
        ));
    }
    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
        return Err(syn::Error::new(ty.span(), "expected generic args"));
    };
    let Some(syn::GenericArgument::Type(inner)) = args.args.first() else {
        return Err(syn::Error::new(ty.span(), "missing inner type"));
    };
    Ok(inner.clone())
}
```

This appendix now represents a legitimate branch of the design, not a bad smell by default. It is still not mandatory. The task is still successful if the handwritten schema/store split is completed cleanly and the macro is not added.

## Files that must be changed or explicitly audited

The implementation must include an explicit audit, not just "touch the obvious files".

### Files to DELETE

These files must be deleted entirely. Their content moves to the new schema/store/runtime structure.

- **`src/dcs/state.rs`** — all types, business logic, and key layout functions move to `schema.rs`. There is no reason for this file to exist after the split.
- **`src/dcs/worker.rs`** — etcd transport moves to `store.rs`, runtime loop and command handling moves to `runtime.rs`. This file is the primary source of the schema/transport/business coupling that the task fixes.

### Files to CREATE

- **`src/dcs/schema.rs`** — owns all DCS record types, public view types, key layout, apply_put/apply_delete, diff logic, evaluate_mode, build_dcs_view.
- **`src/dcs/store.rs`** — owns EtcdRepo, EtcdSession, etcd client connect, watch stream, lease management, snapshot load, mutate-via-diff. All `etcd_client::` imports must be confined here.
- **`src/dcs/runtime.rs`** — owns the DCS worker loop, command handling, view publication. Consumes schema+store; does not directly import `etcd_client::`.

### Files to MODIFY

- **`src/dcs/mod.rs`**
  - Replace `mod state;` and `mod worker;` with `mod schema; mod store; mod runtime;`.
  - Update re-exports. Keep only `DcsHandle`, `DcsView`, `ClusterView`, etc. as public.
  - Remove any re-export of dead types.

- **`src/dcs/command.rs`**
  - Already nearly correct. Verify visibility is `pub(super)`.
  - Must not change functionality.

- **`src/dcs/startup.rs`**
  - Update bootstrap to use new `EtcdRepo` and `bootstrap_runtime` from `runtime.rs`.
  - Update import paths.

- **`src/dcs/log_event.rs`**
  - Verify no structural changes needed (event types are decoupled from schema).

- **`src/config/schema.rs`**
  - **DELETE `DcsInitConfig` struct** (line 379-382). It is dead code — never used at runtime.
  - **DELETE `init: Option<DcsInitConfig>` field** from `DcsConfig` (line 342). This field is never consumed.

- **`src/config/mod.rs`**
  - **Remove `DcsInitConfig` from re-export list** (line 19).

- **`src/dev_support/runtime_config.rs`**
  - **DELETE `with_dcs_init()` method** (line 280).
  - **DELETE `builder_can_override_auth_and_dcs_init` test** (or rewrite as `builder_can_override_auth` without the dead `DcsInitConfig` references) (line 625).

- **`src/runtime/node.rs`**
  - Update bootstrap call if type names change (currently `DcsRuntimeRequest` → keep or rename).
  - Verify import paths still match new module structure.

### Files to AUDIT (may need minor updates)

These files consume DCS public types. If the public API surface changes (e.g. method signatures), they need updates:

- **`src/ha/worker.rs`** — uses `DcsView`, `DcsHandle`, `DcsMode`.
- **`src/ha/decide.rs`** — uses `DcsView`, `ClusterView`, `LeadershipObservation`.
- **`src/api/controller.rs`** — uses `DcsView`, `DcsHandle`.
- **`src/cli/status.rs`** — uses `DcsView`, `ClusterView`, `ClusterMemberView`, `MemberPostgresView`.
- **`src/process/source.rs`** — uses `DcsView`.
- **`src/process/worker.rs`** — uses `DcsView`.
- **`tests/ha/support/...`** — Update fixture/observer logic that may reference old DCS data shape or dead config knobs.

### Dead code that must be removed

This is an explicit checklist. Every item must be verified deleted:

- [ ] `DcsInitConfig` struct in `src/config/schema.rs`
- [ ] `init: Option<DcsInitConfig>` field in `DcsConfig`
- [ ] `DcsInitConfig` re-export in `src/config/mod.rs`
- [ ] `with_dcs_init()` method in `src/dev_support/runtime_config.rs`
- [ ] `DcsInitConfig` import/usage in `src/dev_support/runtime_config.rs` test
- [ ] `MemberPostgresRecord` enum (replaced by `MemberPostgresView` reuse)
- [ ] `MemberLeaseRecord` struct (lease is a transport concern, not a record concern)
- [ ] `LeadershipRecord` wrapper (use `LeaseEpoch` directly)
- [ ] `SwitchoverRecord` wrapper (use `SwitchoverTarget` directly)
- [ ] `record_timeline()` free function (replaced by `MemberPostgresView::timeline()` method)
- [ ] `record_system_identifier()` free function (replaced by `MemberPostgresView::system_identifier()` method)
- [ ] Any `timeout_etcd("string label", ...)` helper (replaced by `run_with_timeout(EtcdOp::Variant, ...)`)
- [ ] The old `src/dcs/state.rs` file itself
- [ ] The old `src/dcs/worker.rs` file itself

## Search terms that must be driven to the new design

The implementer must explicitly grep these and either delete them or move them behind the new schema/store boundary:

- `Compare::` → must exist only in `store.rs`
- `Txn::new` → must exist only in `store.rs`
- `WatchOptions::new` → must exist only in `store.rs`
- `LeaseKeeper` → must exist only in `store.rs`
- `LeaseKeepAliveStream` → must exist only in `store.rs`
- `watch_stream` → must exist only in `store.rs`
- `leader_lease` → must exist only in `store.rs`
- `scope_prefix` → must exist only in `schema.rs` (as `DcsKvSchema::prefix()`)
- `member_path` → must exist only in `schema.rs` (as `DcsKvSchema::member_key()`)
- `leader_path` → must exist only in `schema.rs` (as `DcsKvSchema::leader_key()`)
- `switchover_path` → must exist only in `schema.rs` (as `DcsKvSchema::switchover_key()`)
- `serde_json::from_slice` → must exist only in `schema.rs` (`apply_put`)
- `serde_json::to_string` → must exist only in `schema.rs` (`diff`)
- `EventType::Put` → must exist only in `store.rs` (delegating to `schema.rs`)
- `EventType::Delete` → must exist only in `store.rs` (delegating to `schema.rs`)
- `cfg.dcs.init` → must be deleted entirely (dead code)
- `DcsInitConfig` → must be deleted entirely (dead code)
- `dcs.init` → must be deleted entirely (dead config key)
- `pub(crate)` in `src/dcs/` → only on the 5 explicitly justified root-boundary items (DcsHandle, bootstrap, BootstrapRequest, DcsAdvertisedEndpoints, DcsRuntime)
- `src/dcs/worker.rs` → must not exist
- `src/dcs/state.rs` → must not exist
- `MemberPostgresRecord` → must not exist (merged into MemberPostgresView)
- `MemberLeaseRecord` → must not exist (lease is transport concern)
- `LeadershipRecord` → must not exist (use LeaseEpoch directly)
- `SwitchoverRecord` → must not exist (use SwitchoverTarget directly)
- `timeout_etcd` → must not exist (replaced by run_with_timeout with EtcdOp)
- `expires_at` → must not exist in schema (trust etcd lease expiry)
- `UnixMillis` → must not be imported by schema.rs (no clock in schema layer)

The expected final rule is:

- DCS schema/key decode hits should land in `src/dcs/schema.rs`
- etcd transport hits should land in `src/dcs/store.rs`
- DCS runtime/business hits should land in `src/dcs/runtime.rs`

If those grep terms still appear scattered across unrelated DCS files after the rewrite, the task is not complete.

## Simpler Alternative to Proc Macro: `macro_rules!`

If the team decides derive support is worth adding, consider a `macro_rules!` declarative macro as a simpler alternative to a full proc-macro crate. Benefits:

- No separate crate needed (no `crates/pgtm_etcd_derive/`)
- Faster compilation (no proc-macro compile step)
- Easier to read and maintain (no syn/quote dependencies)
- The schema declaration becomes ultra-compact

Example usage:

```rust
dcs_schema! {
    prefix = "/{scope}";
    singleton leader: LeaseEpoch => lease::ephemeral;
    singleton switchover: SwitchoverTarget => lease::none;
    map members[MemberId]: MemberRecord => lease::ttl;
}
```

This would generate `DcsKvSchema` with `prefix()`, `apply_put()`, `apply_delete()`, and `diff()` — the exact same output as the proc-macro, but defined inline in the crate.

The `macro_rules!` body generates the same pattern as the handwritten code:
- For singletons: match key == `{prefix}/{path}`, deserialize value type, emit Put/Delete in diff
- For maps: match key starts with `{prefix}/{path}/`, parse key suffix, deserialize value type, emit Put/Delete in diff

The proc-macro alternative in the previous section remains valid if the team prefers attribute-based declaration over `macro_rules!` syntax. Both produce the same output. The choice is a team preference, not a correctness issue.
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
- [ ] `MemberPostgresRecord` is eliminated; `MemberPostgresView` is reused as the single type for both etcd storage and public display, with a `sanitize_member_postgres()` transformation during view building.
- [ ] `MemberRecord` does not contain `expires_at` or any lease metadata. The record stores only domain data (`postgres_target`, `postgres`). Etcd lease expiry is handled by the store layer.
- [ ] `evaluate_mode` and `build_dcs_view` have no clock/time dependency. They trust etcd to remove expired member keys via lease expiry.
- [ ] All dead code items from the explicit cleanup checklist are verified deleted: `DcsInitConfig`, `MemberLeaseRecord`, `LeadershipRecord`, `SwitchoverRecord`, `timeout_etcd`, old `state.rs` and `worker.rs` files.
- [ ] `LeaseAttachment` uses `Ttl(u64)` (milliseconds from config), not `TtlUntil(UnixMillis)` (record-level deadline).
- [ ] If derive support is implemented, it is added only as a small follow-up over an already-clear handwritten schema split, and it does not hide business logic.
- [ ] If derive support is not implemented, that is still acceptable as long as the handwritten schema/store/runtime split is clean and the schema/key-format scattering problem is fully solved.
</acceptance_criteria>
