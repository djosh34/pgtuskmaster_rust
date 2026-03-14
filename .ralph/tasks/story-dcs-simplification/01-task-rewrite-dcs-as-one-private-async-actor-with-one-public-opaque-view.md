## Task: Rewrite DCS As One Private Async Actor With One Public `DcsView` <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Rewrite the DCS subsystem so it has exactly one owning async loop, exactly one etcd client/session owner, zero `Arc`/`Mutex` inside the production DCS path, one public read-only serde `DcsView`, and one crate-private typed command handle (`DcsHandle`) for mutation. The higher-order goal is to turn DCS into a small, private coordination domain instead of a collection of storage-shaped types, bridge layers, and leaked implementation details. This is a deliberate simplification task, not a privacy-only wrapper task: the end state must remove code, remove representations, and remove synchronization primitives that only exist because the current design split ownership badly.

This task must not be placed under `story-ctl-operator-experience`. There is an older completed task there, `.ralph/tasks/story-ctl-operator-experience/10-task-collapse-dcs-behind-a-single-private-component-and-read-only-dcs-view.md`, but this new task is a fresh follow-up story focused specifically on aggressive DCS simplification and public-surface collapse.

**Complete redesign decisions already made from research and user discussion:**
- The requirements are intentionally strict and must be treated as hard constraints, not preferences:
  - one loop
  - zero `Arc`
  - zero `Mutex`
  - zero separate “is healthy” / store-health functions in the DCS public model
  - one single public DCS view enum
  - writes happen only through crate-private typed functions on the DCS command handle
  - all old superseded code must be cleaned up, not left behind
  - all DCS code must be private by default first; visibility should only be widened if it is absolutely necessary to satisfy the final boundary
- A published read-only state channel such as the existing watch-based state pattern is still allowed. The hard ban is specifically on `Arc`/`Mutex` in the production DCS implementation, not on all forms of state publication or subscriptions.
- The current naming `FullQuorum` should be removed. The code today does not compute real quorum mathematics; it only distinguishes store reachability plus minimal observed-member conditions. The replacement naming should use something like `Coordinated`, `Trusted`, or another simpler term that matches what the code actually proves.
- The DCS coordination state must collapse into one single public enum named `DcsView`.
- `DcsView` should be a public enum, not a struct with a separate `trust` field, because this rewrite wants one total read-only view for both internal logic and public/API serialization.
- The required internal shape is:
  - `NotTrusted(NotTrustedView)`
  - `Degraded(ClusterView)`
  - `Coordinated(ClusterView)`
- Do **not** implement the user’s literal recursive idea `NotTrusted | Degraded(DcsView) | FullQuorum(DcsView)`; that is structurally recursive and not the right representation.
- `NotTrusted` should **not** carry the full cluster snapshot by default. Current code only needs stale observed leadership information in the not-trusted path for conservative fencing/publication, not the whole member/routing/switchover payload.
- `Degraded` and `Coordinated` should carry one shared private cluster payload type so their data surface is identical while the mode remains impossible to ignore.
- The public read surface should be:
  - `pub enum DcsView`
  - the nested public read-only payload/view types required to inspect and serialize `DcsView`
- The crate-private mutation surface should be:
  - `pub(crate) struct DcsHandle`
  - `pub(crate) enum DcsHandleError`
- All other DCS types must be private unless the crate runtime composition root truly needs `pub(crate)` visibility.
- Visibility must start at private. The implementer must not default to `pub(crate)` for internal sharing. The desired end state is:
  - public: `DcsView` and its nested read-only payload/view types
  - crate-private: `DcsHandle`, `DcsHandleError`, and the smallest bootstrap/runtime wiring needed to construct and run the DCS actor
  - everything else: private to `src/dcs`, and preferably private to the smallest possible module
- The current suggestion of a public `reason` payload should **not** be added by default. Research found no current HA/API/process/CLI logic that depends on a public reason classification for `NotTrusted` or `Degraded`. If the implementation still wants reason classification for logging or internal branching, keep that classification private to `src/dcs`. Only promote a reason enum to the public API if a real consumer emerges during implementation and there is no cleaner method-based alternative.
- Prefer variant semantics over bolting on separate public mode/health/reason fields. “Health” as a separate concept should disappear from the public model and from internal bookkeeping where it only duplicates enum state.
- DCS should be rewritten around one async owner loop and direct etcd ownership. The current sync trait + dedicated thread + shared queue bridge must be removed. `Arc` and `Mutex` are not allowed to remain in the production DCS implementation after this rewrite.
- This repo is greenfield. Do not preserve backward compatibility for the old DCS public shape, old type names, or old API wire format if a cleaner boundary requires changing them.

**Scope:**
- Rewrite `src/dcs` around one async actor/loop that directly owns etcd connection state, watch state, leader lease ownership state, and the private in-memory snapshot used to publish the public read-only `DcsView`.
- Delete the current bridge architecture that exists only to present a synchronous `DcsStore` interface over an async etcd client.
- Reduce the DCS surface to one public read-only `DcsView` enum and one crate-private typed `DcsHandle`.
- Remove public and crate-public DCS record/cache/store/helper types that leak implementation structure outside `src/dcs`.
- Update HA, process, API, CLI, runtime wiring, and tests to consume the new `DcsView` directly instead of today’s public nested DCS structs/fields.
- Remove dead DCS legacy such as `/scope/config` decoding if it still exists and is not used by the new design.
- Keep all real etcd watch/key/lease mechanics private inside `src/dcs`.
- Audit visibility aggressively. `pub(crate)` must not remain as a convenience default inside DCS.

**Out of scope:**
- Do not redesign HA policy beyond the minimum adaptation required to consume the new DCS interface.
- Do not add new operator-experience features.
- Do not preserve the old API shape just to avoid migrations.

**Context from research: current implementation facts that motivate the rewrite:**
- `src/dcs/mod.rs` currently already tries to present a small public face, but the actual implementation is spread across `command`, `startup`, `state`, `store`, `worker`, `etcd_store`, and `keys`. This creates many `pub(crate)` internal types solely because sibling modules need access.
- `src/dcs/command.rs` already provides the right high-level idea for the write boundary: a typed `DcsHandle` with typed commands such as acquire leadership, release leadership, publish switchover, and clear switchover.
- `src/runtime/node.rs` currently wires DCS to the rest of the system as one state subscriber and one command handle. That boundary direction is good and should remain, but the state type should collapse to one public `DcsView` instead of today’s spread-out public DCS struct family.
- `src/state/watch_state.rs` already provides the right publishing primitive for read-mostly state with `tokio::sync::watch`; keep this pattern.
- The major complexity sits in `src/dcs/etcd_store.rs`. It currently introduces:
  - a dedicated OS thread for the etcd worker
  - a second Tokio runtime built inside that thread
  - `Arc<AtomicBool>` to track store health
  - `Arc<Mutex<VecDeque<WatchEvent>>>` to shuttle watch updates
  - a `tokio::sync::mpsc` command channel plus blocking `std::sync::mpsc` response channels
  - per-request blocking timeouts to emulate a synchronous store API
- Those synchronization primitives are not domain-essential. They are artifacts of the current “sync `DcsStore` trait in front of an async etcd client” design.
- `src/dcs/store.rs` currently defines `DcsStore` and low-level operations such as `snapshot_prefix`, `write_path`, `write_path_with_lease`, `delete_path`, `drain_watch_events`, `acquire_leader_lease`, `release_leader_lease`, and `clear_switchover`. This is too low-level and should disappear from the external design entirely.
- `src/dcs/state.rs` currently splits public view and public trust into separate types (`DcsView`, `DcsTrust`), while current consumers mostly treat trust as a mode gate. `evaluate_trust` currently implements:
  - unhealthy store -> `NotTrusted`
  - self member missing -> `Degraded`
  - otherwise insufficient members -> `Degraded`
  - else `FullQuorum`
  This appears at `src/dcs/state.rs` around lines 312-325. The helper `has_member_quorum` currently means “1 member for a singleton view, 2 members for any multi-member view”, not true majority math.
- `src/dcs/state.rs` also carries `last_emitted_store_healthy: Option<bool>` and `last_emitted_trust: Option<DcsTrust>` for log deduplication. The new design should remove the separate “store healthy” modeling. If transition dedup is still needed, it should key off the enum mode itself and remain internal.
- `src/dcs/keys.rs` still recognizes `/scope/config`, and `src/dcs/store.rs` still decodes that key into a full `RuntimeConfig`. `src/dcs/worker.rs` ignores the resulting config value. This looks like dead coupling/legacy and should be removed unless the rewrite finds a real current use.
- `dcs.init` appears in config schema and docs, and the DCS internals still parse `/scope/init` and `/scope/config`, but this investigation did not find a real production runtime path that consumes `cfg.dcs.init` or writes those keys. Treat this as stale/legacy. This task must delete `dcs.init`, `/scope/init`, and `/scope/config` support entirely rather than carry them forward as “maybe needed” bootstrap features.

**Context from research: what current consumers actually need from DCS:**
- HA is the broadest consumer, but it still only uses DCS as input and then immediately builds its own `WorldView`.
- `src/ha/decide.rs` only distinguishes `FullQuorum` from “not `FullQuorum`”; it does not currently branch differently between `Degraded` and `NotTrusted`.
- `src/api/controller.rs` rejects switchover unless trust is `FullQuorum`.
- `src/process/worker.rs` only uses DCS to resolve a source member by leader id and then read that member’s endpoint/postgres-role shape.
- CLI status/connect/switchover use DCS mostly for member discovery, switchover visibility, and simple coordination status text.
- Across non-DCS consumers, the truly needed observations are:
  - whether DCS is coordinated, degraded, or not trusted
  - which members exist
  - how to inspect one member’s API endpoint and postgres endpoint
  - whether a member is primary / ready replica / ready unknown
  - the current leader epoch if any
  - whether a switchover exists and whether it targets a specific member or any ready replica
  - timeline/system identifier/WAL position for leader and replica selection logic
- Consumers do **not** need public access to:
  - internal cache types
  - raw record types
  - raw key parsers
  - watch events
  - revision bookkeeping
  - lease IDs
  - a separate public `health` bool
  - `DcsMemberLeaseView`
  - public `worker`/`last_observed_at` metadata unless the rewrite finds a current consumer during implementation

**Required shared primitives to move into `src/state` as part of this task:**
- This task must not introduce fresh duplicate structs/enums for concepts that are already duplicated elsewhere in the repository. The rewrite must move the real shared primitives into `src/state` and make DCS/HA/API/process reuse them.
- Do **not** create a vague generic `types` module. Add focused shared modules under `src/state` instead, for example `coordination.rs` and `net.rs`.
- At minimum this task must introduce and use these shared types:

```rust
// src/state/coordination.rs

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LeaseEpoch {
    pub holder: MemberId,
    pub generation: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SwitchoverTarget {
    AnyHealthyReplica,
    Specific(MemberId),
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ObservedWalPosition {
    pub timeline: Option<TimelineId>,
    pub lsn: WalLsn,
}
```

```rust
// src/state/net.rs

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PgTcpTarget {
    pub host: String,
    pub port: u16,
}

impl PgTcpTarget {
    pub fn new(host: String, port: u16) -> Result<Self, String>;
    pub fn host(&self) -> &str;
    pub fn port(&self) -> u16;
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PgUnixTarget {
    pub socket_dir: std::path::PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PgConnectTarget {
    Tcp(PgTcpTarget),
    Unix(PgUnixTarget),
}
```

- `PgTcpTarget` exists because DCS member advertisement is always a PostgreSQL TCP target, not a generic “any TCP thing”.
- `PgConnectTarget` exists because the repo already overloads PostgreSQL connection “host” to sometimes mean a Unix socket directory in `PgConnInfo`; this task should stop carrying that overload as raw stringly data.
- This task must move existing duplicates onto these shared types and delete the copies rather than leaving transitional parallel definitions behind.
- Concretely, this task should eliminate or fold the duplicated concepts currently living in:
  - `src/dcs/state.rs`: `DcsMemberEndpointView`, `DcsMemberApiView`, `WalVector`, `DcsSwitchoverTargetView`
  - `src/ha/types.rs`: `LeaseEpoch`, `SwitchoverTarget`
  - `src/pginfo/conninfo.rs`: raw `host: String` plus `port: u16` as an overloaded transport target
  - the new public `DcsView` payloads introduced by this rewrite: they must reuse the shared types rather than define `ApiObservedLeadership`, `ApiWalVector`, or raw `postgres_host` / `postgres_port` pairs again
- This task must remove member API URL advertisement from DCS entirely. It is currently used only for:
  - HA failover/switchover eligibility in `src/ha/worker.rs`
  - CLI status rendering in `src/cli/status.rs`
- After the rewrite, DCS member state should no longer carry an API URL or API-presence bit at all. HA must stop deriving `ApiVisibility` from DCS member records, and CLI/API output must stop rendering per-member API URLs from DCS.

**Required `DcsView` boundary:**
- The single read-only DCS state exposed both internally and through public API serialization must be:

```rust
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DcsView {
    NotTrusted(NotTrustedView),
    Degraded(ClusterView),
    Coordinated(ClusterView),
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NotTrustedView {
    observed_leadership: Option<LeaseEpoch>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ClusterView {
    members: BTreeMap<MemberId, ClusterMemberView>,
    leadership: LeadershipObservation,
    switchover: SwitchoverView,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ClusterMemberView {
    postgres: MemberPostgresView,
    postgres_target: PgTcpTarget,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LeadershipObservation {
    Open,
    Held(LeaseEpoch),
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "state", content = "target")]
pub enum SwitchoverView {
    None,
    Requested(SwitchoverTarget),
}
```

- `NotTrustedView` is intentionally small. It should expose only the stale observed leadership data needed by current fail-safe/fencing publication logic.
- `ClusterView` is the shared payload for `Degraded` and `Coordinated`. That keeps their data surface identical while forcing callers to branch on coordination mode explicitly.
- `DcsView` should be matched directly inside crate code where that is the clearest expression of impossible states.
- `DcsView` and its nested payload/view types should be public so `NodeState` can expose exactly one total DCS view, but their fields should stay private and they must not expose public constructors. The only public way to obtain a fresh `DcsView` remains subscription to the DCS actor output.
- The public read/query surface should be limited to what current call sites actually need:

```rust
impl DcsView {
    pub fn mode(&self) -> DcsMode;
    pub fn observed_leadership(&self) -> Option<&LeaseEpoch>;
    pub fn cluster(&self) -> Option<&ClusterView>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DcsMode {
    NotTrusted,
    Degraded,
    Coordinated,
}

impl NotTrustedView {
    pub fn observed_leadership(&self) -> Option<&LeaseEpoch>;
}

impl ClusterView {
    pub fn member_ids(&self) -> impl Iterator<Item = &MemberId>;
    pub fn member_count(&self) -> usize;
    pub fn member(&self, member_id: &MemberId) -> Option<&ClusterMemberView>;
    pub fn leadership(&self) -> &LeadershipObservation;
    pub fn switchover(&self) -> &SwitchoverView;
}

impl ClusterMemberView {
    pub fn postgres_target(&self) -> &PgTcpTarget;
    pub fn postgres(&self) -> &MemberPostgresView;
}

impl MemberPostgresView {
    pub fn readiness(&self) -> Readiness;
    pub fn system_identifier(&self) -> Option<SystemIdentifier>;
    pub fn timeline(&self) -> Option<TimelineId>;
    pub fn is_primary(&self) -> bool;
    pub fn is_ready_replica(&self) -> bool;
    pub fn is_ready_non_primary(&self) -> bool;
    pub fn committed_wal(&self) -> Option<&ObservedWalPosition>;
    pub fn replay_wal(&self) -> Option<&ObservedWalPosition>;
    pub fn follow_wal(&self) -> Option<&ObservedWalPosition>;
    pub fn upstream(&self) -> Option<&MemberId>;
}
```

- `DcsHandle` must remain the typed write boundary.
- There should be no second public DCS DTO type. `NodeState` and any other public API structs should expose `DcsView` directly.

**Required typed command surface on `DcsHandle`:**
- The command handle is the only allowed mutation path into DCS from outside `src/dcs`.
- At minimum it must expose typed operations equivalent to:
  - acquire local leadership
  - release local leadership
  - publish a switchover request targeting any healthy replica
  - publish a switchover request targeting a specific member
  - clear the current switchover request
- There must be no typed init/bootstrap command, because current investigation did not find a real production consumer for `dcs.init`, `/scope/init`, or `/scope/config`, and this task explicitly removes that dead feature.
- No raw path/string mutation methods may remain on the command boundary.

**Required crate-private `DcsHandle` signatures:**

```rust
pub(crate) enum DcsHandleError {
    ChannelClosed,
}

impl DcsHandle {
    pub(crate) fn acquire_leadership(&self) -> Result<(), DcsHandleError>;
    pub(crate) fn release_leadership(&self) -> Result<(), DcsHandleError>;
    pub(crate) fn publish_switchover_any(&self) -> Result<(), DcsHandleError>;
    pub(crate) fn publish_switchover_to(
        &self,
        target: MemberId,
    ) -> Result<(), DcsHandleError>;
    pub(crate) fn clear_switchover(&self) -> Result<(), DcsHandleError>;
}
```

Do not expose a public raw `DcsCommand` enum unless implementation proves it is necessary. The preferred boundary is typed methods on the handle, because the design goal is one crate-private state ADT plus one crate-private mutation handle, not another exposed command ADT.
- `DcsHandle` fields must stay private.
- `DcsHandle` constructors/factory helpers must stay private to `src/dcs` or at most `pub(crate)` to bootstrap/test-support wiring. The current public `DcsHandle::closed()` escape hatch must be removed from the runtime surface.

**Required crate-private command error semantics:**
- The handle should be one-way enqueue only, not request/response.
- The failure mode should therefore be minimal: failure means the command could not be queued because the DCS actor/handle channel is closed.
- Do not preserve the current request/response error split unless implementation finds a concrete case that truly needs it and the task result documents that case.
- The current code exposes:
  - `ChannelClosed`
  - `Rejected(String)`
  - `Transport(String)`
  in [`src/dcs/command.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dcs/command.rs#L26)
- This task should simplify that model down to a typed non-string crate-private enum:
  - `DcsHandleError::ChannelClosed`
- Mutation outcomes after enqueue should be observed through the published `DcsView` and logging, not through a reply channel.
- Because HA can tick again before a fresh `DcsView` reflecting the mutation is published, the DCS actor must treat duplicate identical commands as safe/idempotent. At minimum:
  - repeated `acquire_leadership` from the same node must not create semantic failure
  - repeated `release_leadership` must collapse to a no-op once local leadership is already gone
  - repeated `clear_switchover` must collapse to a no-op once switchover state is already absent
  - repeated `publish_switchover_any` or `publish_switchover_to(same_target)` must be equivalent to one publish

**Explicit design sketch that this task should implement unless code-level constraints force a small, justified variation:**

This is not a second competing `DcsView` definition. The canonical required type signatures are the serde-enabled public definitions above under **Required `DcsView` boundary**. This sketch exists only to restate the ownership/privacy intent:
- there is exactly one public `DcsView`
- callers match on that one enum directly
- payload fields stay private
- constructors stay non-public
- `DcsHandle` stays crate-private

In other words, the implementation should use the same public serde `DcsView` type for both internal read logic and HTTP/CLI serialization. There must not be one serde `DcsView` and another non-serde `DcsView`.

The task should actually prefer renaming `FullQuorum` to `Coordinated` or `Trusted`. With the current algorithm, `FullQuorum` is misleading. If someone insists on the name `FullQuorum`, then the implementation would also need to add explicit expected voter count and compute real majority, which is not the desired simplification path here.

The critical part is this: `NotTrusted` should carry only what current code really uses under lost trust. Research found that the not-trusted path currently uses stale observed leadership for conservative fencing/publication, but does **not** require the full cluster member/switchover/routing snapshot to keep making normal coordinated decisions.

That gives the codebase one total public read-only DCS view and one crate-private mutation handle. Everything else becomes private implementation detail.

**Required internal design after the rewrite:**
- One DCS-owned async actor/task, likely in `src/dcs/worker.rs` or a renamed private module.
- That actor owns:
  - etcd client connection/session
  - watch stream
  - reconnect/backoff state
  - leader lease ownership state
  - the single private in-memory snapshot that backs `DcsView`
  - the command inbox
  - publication of the latest `DcsView`
- Remove the dedicated store thread, shared queues, and shared atomics.
- The loop should use `tokio::select!` over:
  - watch stream messages
  - local PG state changes
  - typed commands from `DcsHandle`
  - reconnect/republish timers as needed
- Private snapshot state should be one coherent internal model, not split between “cache structs” plus separately drained watch queues plus extra health state.

**Explicit internal rewrite sketch that this task must implement unless there is a repository-wide architectural change outside DCS that the task documents and completes:**

The big simplification is not the enum itself. The big simplification is deleting the sync store bridge.

Right now the `Arc`/`Mutex` exist because DCS is split into:
- a DCS worker loop
- a synchronous `DcsStore` trait in [`src/dcs/store.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dcs/store.rs)
- an async etcd client hidden behind a dedicated OS thread and a private Tokio runtime in [`src/dcs/etcd_store.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dcs/etcd_store.rs#L229)

That bridge forces:
- `Arc<AtomicBool>` for store health
- `Arc<Mutex<VecDeque<WatchEvent>>>` for cross-thread watch events
- command channels and blocking timeouts
- duplicate “store healthy” bookkeeping

The replacement should be one async actor, one loop, one owner:

```rust
struct DcsActor {
    pg_rx: StateSubscriber<PgInfoState>,
    cmd_rx: mpsc::UnboundedReceiver<DcsCommandRequest>,
    view_tx: StatePublisher<DcsView>,

    session: Session,
    snapshot: Snapshot,
    advertised: AdvertisedEndpoints,
    self_id: MemberId,
    scope: String,
    ttl_ms: u64,
}

enum Session {
    Disconnected { last_error: Option<String>, retry_at: Instant },
    Connected {
        client: etcd_client::Client,
        watch: etcd_client::WatchStream,
        owned_leader: Option<OwnedLeaderLease>,
    },
}
```

Then the loop is just `tokio::select!` over:
- `pg_rx.changed()`
- `cmd_rx.recv()`
- `watch.message()`
- a timer tick for reconnect / TTL / republish

Behavior:
1. connect to etcd
2. fetch one initial snapshot for `/{scope}/...`
3. start watch from the returned revision
4. mutate private snapshot directly on incoming watch events
5. on local PG change, rewrite only the local member key
6. on handle command, write leader or switchover key directly
7. on any etcd failure, drop session, derive the minimal `NotTrustedView` payload from the last observed leadership state, publish `DcsView::NotTrusted(...)`, reconnect with backoff

The command path should be “just a channel” internally:
- crate-private typed method on `DcsHandle`
- internal send into the DCS actor inbox
- no internal response path back to the caller
- no reply/ack waiting in HA/API/process
- mutation success/failure after enqueue is reflected by subsequent `DcsView` state and logs
- duplicate identical commands must be safe/idempotent because HA may enqueue again before DCS publication catches up

That must remove:
- the separate worker thread
- the `DcsStore` trait
- `EtcdDcsStore`
- `Arc`
- `Mutex`
- the watch event queue
- the `store_healthy` bool as a modeled concept
- the `last_emitted_store_healthy: Option<bool>` log state in [`src/dcs/state.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dcs/state.rs#L214)

“Health” stops being its own thing. Session state maps directly to the enum variant:
- disconnected / watch broken / bootstrap failed -> `NotTrusted`
- connected but coordination prerequisites not met -> `Degraded`
- connected and coordination prerequisites met -> `Coordinated`

**Concrete implementation directions and file targets:**
- `src/dcs/mod.rs`
  - Re-export only the minimal public surface.
  - Keep submodules private by default.
  - Eliminate re-exports of storage-shaped/helper types.
- `src/dcs/mod.rs` should become the only intentional DCS module surface, with only `DcsView` and its nested read-only payload/view types remaining public.
- `src/dcs/command.rs`
  - Keep the typed command handle concept.
  - Update command payloads if needed for the new model.
  - Keep crate API tiny and typed.
  - Ensure the final crate-private command surface covers only the allowed mutations: local leadership acquire/release, switchover publish-any, switchover publish-to-member, and switchover clear.
  - Prefer typed non-async enqueue methods on `DcsHandle` over exposing a public raw command enum.
  - Prefer a minimal enqueue-only error model such as closed-channel/actor-gone.
- `src/dcs/startup.rs`
  - Likely shrink heavily or fold into `mod.rs`.
  - Bootstrap should construct the single actor, one `watch` publisher/subscriber pair, and one `DcsHandle`.
- `src/state/mod.rs`
  - Re-export the new shared DCS/HA/API primitives from focused new submodules.
- `src/state/coordination.rs`
  - Add the shared `LeaseEpoch`, `SwitchoverTarget`, and `ObservedWalPosition` types used by DCS, HA, and the public `DcsView` payloads.
- `src/state/net.rs`
  - Add the shared `PgTcpTarget`, `PgUnixTarget`, and `PgConnectTarget` types.
- `src/dcs/state.rs`
  - Replace the current public struct/enum matrix with the new public `DcsView` enum plus private-field payload representation.
  - Use the shared `LeaseEpoch`, `SwitchoverTarget`, `ObservedWalPosition`, and `PgTcpTarget` types rather than redefining copies locally.
  - Remove public `DcsTrust`.
  - Remove public record/cache structs.
  - Remove public worker metadata fields if no longer needed.
  - Remove member API URL/state entirely from the internal DCS payload.
- `src/dcs/worker.rs`
  - Rewrite around one async actor with direct etcd ownership.
  - Eliminate the current bridge assumptions and store-health bool propagation.
  - Derive crate-private `DcsView` variants from session + private snapshot state.
- `src/dcs/etcd_store.rs`
  - Either delete entirely or replace with a much smaller private async etcd helper module that does not own cross-thread state.
  - No `Arc`, no `Mutex`, and no OS-thread worker loop may remain.
- `src/dcs/store.rs`
  - Delete. If tiny private helper functions remain useful, move them into a private etcd adapter module without keeping a public/store-like boundary.
- `src/dcs/keys.rs`
  - Keep only if still necessary and make fully private.
  - Remove dead `/config` support unless the rewrite introduces an explicit current use.
- `src/config/schema.rs`, docs, and any tests/builders touching `DcsInitConfig`
  - Remove `dcs.init` entirely.
  - Remove all associated docs/tests/builders and stale references.
- `src/runtime/node.rs`
  - Keep runtime as a composition root that wires the new DCS actor and passes only crate-private `DcsHandle` and `StateSubscriber<DcsView>` to internal workers.
- `src/ha/worker.rs`
  - Stop direct field-walking through today’s public DCS structs.
  - Match on `DcsView` directly and use payload methods.
  - Reuse the moved shared `LeaseEpoch`, `SwitchoverTarget`, and `ObservedWalPosition` types instead of local copies.
  - Remove the current coupling where peer API reachability is inferred from `member.routing.api.is_some()`.
  - Adjust naming from `FullQuorum` to the new mode vocabulary.
- `src/ha/decide.rs`
  - Replace `DcsTrust::FullQuorum` checks with the new `DcsView` mode or a derived HA coordination-mode representation.
- `src/ha/types.rs`
  - Delete the local `LeaseEpoch` and `SwitchoverTarget` definitions after moving them into `src/state`.
  - Keep HA-only derived decision types such as `LeadershipView`, `SwitchoverState`, and `WalPosition` local to HA.
- `src/process/worker.rs` and `src/process/source.rs`
  - Replace `DcsMemberView` dependency with `ClusterView`/`ClusterMemberView` payload methods reached through `DcsView` matching.
- `src/pginfo/conninfo.rs`
  - Replace the overloaded raw `host: String` + `port: u16` transport representation with `PgConnectTarget`.
  - Rendering/parsing of PostgreSQL conninfo should continue to work, but the in-memory type should stop conflating TCP hosts with Unix socket directories.
- `src/api/controller.rs`
  - Validate switchovers by matching on `DcsView`; only `Coordinated`/`Degraded` should expose cluster payload, and `NotTrusted` should reject immediately.
- `src/api/mod.rs`, `src/api/worker.rs`, `src/cli/status.rs`, `src/cli/connect.rs`, `src/cli/switchover.rs`
  - Use `DcsView` directly in `NodeState` and any other HTTP-facing DTOs.
  - Reuse the shared `PgTcpTarget`, `LeaseEpoch`, `SwitchoverTarget`, and `ObservedWalPosition` types in `DcsView` instead of duplicating fields.
  - Remove per-member API URL rendering from DCS-derived CLI/API output.
- `src/logging/event.rs` and any log serialization/tests
  - Update labels/serialization if `DcsTrust` is removed or renamed.
- DCS-related tests in `src/dcs/worker.rs`, HA tests, API tests, CLI tests, and documentation that mentions `FullQuorum`
  - Update to the new model and names.

**Explicit deletion/folding targets from the design discussion:**
- keep `src/dcs/mod.rs` as the only DCS module surface, with only `DcsView` and its nested read-only payload/view types public
- fold most of `src/dcs/startup.rs` into `mod.rs` if that reduces surface and indirection
- delete `src/dcs/store.rs`
- replace `src/dcs/etcd_store.rs` with a much smaller private `etcd.rs` that contains only async helper functions, if a separate file is still useful
- fold `src/dcs/state.rs` into one private-field payload family plus the public `DcsView` enum if that reduces duplication and visibility noise
- add focused shared modules under `src/state` instead of a generic `types` bucket
- move `LeaseEpoch`, `SwitchoverTarget`, and `ObservedWalPosition` into `src/state/coordination.rs`
- move `PgTcpTarget`, `PgUnixTarget`, and `PgConnectTarget` into `src/state/net.rs`
- delete the old duplicate endpoint/url/wal/switchover/epoch types after migration; do not leave parallel old/new definitions behind
- delete DCS member API URL advertisement and its old types after migration; do not leave a parallel “optional API endpoint in DCS” path behind
- inline or fully privatize `src/dcs/keys.rs`
- remove `/scope/config` decoding entirely; it currently decodes a full `RuntimeConfig` from DCS and then ignores it in [`src/dcs/store.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dcs/store.rs#L191)
- remove `/scope/init` support entirely; it currently survives as stale parsing/internal cache state via:
  - [`src/dcs/keys.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dcs/keys.rs)
  - [`src/dcs/state.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dcs/state.rs)
  - [`src/dcs/worker.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dcs/worker.rs)
  - [`src/dcs/store.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dcs/store.rs)
- remove `dcs.init` config support entirely; it currently survives as stale schema/docs/helper state via:
  - [`src/config/schema.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/config/schema.rs#L337)
  - [`src/config/mod.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/config/mod.rs)
  - [`src/dev_support/runtime_config.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dev_support/runtime_config.rs)
  - [`docs/src/reference/runtime-configuration.md`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/src/reference/runtime-configuration.md#L242)
  - [`docs/src/how-to/bootstrap-cluster.md`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/src/how-to/bootstrap-cluster.md#L18)
  - [`docs/src/reference/dcs-state-model.md`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/src/reference/dcs-state-model.md)

**Patterns to follow:**
- Use ownership to eliminate synchronization, not synchronization to patch over split ownership.
- Prefer one public read-only DCS ADT over multiple parallel public enums/structs/DTOs.
- Prefer private fields and private constructors over duplicate public DTO layers when one read-only view is sufficient.
- Prefer focused shared domain primitives in `src/state` over ad hoc copies in DCS/HA/API/process.
- Prefer typed PostgreSQL targets over raw strings and loose `host`/`port` pairs.
- Prefer removing non-essential member metadata from DCS entirely over typing and preserving it “just in case”.
- Prefer compiler-enforced privacy over conventions.
- Prefer deleting dead code to preserving compatibility shims.

**Expected outcome:**
- The repository has one DCS owner loop, one etcd session owner, and no DCS `Arc`/`Mutex`/cross-thread event queue machinery.
- `DcsView` is the one total public read-only DCS view.
- `DcsHandle` is a crate-private write path into DCS.
- Non-DCS code cannot import or manipulate DCS record/cache/store/key types because those types no longer exist publicly.
- DCS mode is represented directly by the `DcsView` enum itself; there is no separate public `health` or `trust` field.
- `FullQuorum` is gone.
- Public reasons for degraded/not-trusted are not added unless a real consumer proves they are necessary. Any such classification should stay private by default.
- The package is materially smaller, easier to read, and closer to one coherent DCS domain boundary instead of several stacked representations.

**What downstream code should look like after the rewrite:**
- HA/API/process stop digging through DCS structs directly.
- Instead of patterns such as:
  - `dcs.trust != DcsTrust::FullQuorum` in [`src/api/controller.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/api/controller.rs#L81)
  - `dcs.members.get(...)` in [`src/process/worker.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/process/worker.rs#L1485)
  - direct leader/switchover matching in [`src/ha/worker.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/worker.rs#L375)
- they should call `DcsView` methods such as:
  - `dcs.is_coordinated()`
  - `dcs.leader_epoch()`
  - `dcs.member_is_ready_replica(id)`
  - `dcs.postgres_endpoint(id)`
  - possibly convenience helpers like `dcs.iter_ready_replicas()` if implementation finds them useful

This method-driven downstream shape is the real public-surface reduction.

</description>

<acceptance_criteria>
- [ ] The strict rewrite constraints are satisfied explicitly: one loop, zero `Arc`, zero `Mutex`, zero separate “health” functions/booleans in the public HTTP DCS model, one public serde `DcsView`, and one crate-private typed write handle.
- [ ] `src/dcs` is rewritten so the final public surface is only `DcsView` and its nested read-only payload/view types, while the final crate-private internal surface is only `DcsHandle`, `DcsHandleError`, and the minimum required runtime wiring.
- [ ] Focused shared modules are added under `src/state` for the actually shared DCS/HA/API/process primitives; this task must not solve the overlap by creating a generic catch-all `types` module.
- [ ] `LeaseEpoch`, `SwitchoverTarget`, and `ObservedWalPosition` are moved into `src/state` and reused by DCS, HA, and the public `DcsView` payloads.
- [ ] `PgTcpTarget`, `PgUnixTarget`, and `PgConnectTarget` are added under `src/state` and reused by DCS/API/pginfo where appropriate.
- [ ] `DcsView` is a public enum whose variants express coordination mode directly; `DcsTrust`/`FullQuorum` no longer exist publicly.
- [ ] The final naming replaces `FullQuorum` with a term that matches the actual semantics, such as `Coordinated`/`Trusted`, and docs/tests/logging are updated consistently.
- [ ] There is no second public DCS DTO. HTTP/API consumers, CLI code, and internal crate code all use the same read-only `DcsView`.
- [ ] All writing into DCS happens only via exposed typed command-handle functions. No other write path remains.
- [ ] The final `DcsHandle` exposes only typed one-way enqueue mutations for:
  - acquiring local leadership
  - releasing local leadership
  - publishing switchover to any healthy replica
  - publishing switchover to a specific member
  - clearing switchover
- [ ] The final crate-private `DcsHandle` surface is method-based, with signatures equivalent to:
  - `pub(crate) enum DcsHandleError { ChannelClosed }`
  - `acquire_leadership(&self) -> Result<(), DcsHandleError>`
  - `release_leadership(&self) -> Result<(), DcsHandleError>`
  - `publish_switchover_any(&self) -> Result<(), DcsHandleError>`
  - `publish_switchover_to(&self, MemberId) -> Result<(), DcsHandleError>`
  - `clear_switchover(&self) -> Result<(), DcsHandleError>`
- [ ] The final crate-private command path is one-way enqueue only. No request/response reply channel remains in the mutation boundary.
- [ ] The final crate-private command error model is minimal and enqueue-oriented. It should not preserve “rejected” / “transport” failure classes unless implementation proves a concrete need and documents it.
- [ ] Duplicate identical commands are explicitly safe. If HA/API sends the same mutation again before `DcsView` catches up, the DCS actor handles that as an idempotent no-op/equivalent operation rather than a semantic error.
- [ ] All non-DCS consumers inside this crate (`src/ha`, `src/process`, `src/api`, runtime wiring, CLI code, and relevant tests) are updated to match on `DcsView` and use only the public read-only payload methods they actually need.
- [ ] Public API/CLI surfaces (`src/api`, `src/cli`, integration tests, serialized `NodeState`) expose `DcsView` directly. No `ApiDcsView` or equivalent parallel wire-only DCS type remains.
- [ ] DCS member state no longer carries an API URL or API-presence marker. HA must no longer derive `ApiVisibility` from DCS member records, and CLI/API output must no longer render per-member API URLs from DCS.
- [ ] `PgConnInfo` no longer overloads raw `host: String` to mean both TCP host and Unix socket directory; the in-memory type uses the shared `PgConnectTarget`.
- [ ] `src/dcs/store.rs` is deleted, or any tiny unavoidable remnants are moved private inside `src/dcs` with no store-like boundary exposed outside the owning actor.
- [ ] The old `src/dcs/etcd_store.rs` thread/bridge architecture is deleted or replaced by a substantially smaller private async etcd helper with no `Arc`, no `Mutex`, and no cross-thread watch event queue in the DCS path.
- [ ] The final DCS runtime path has one async owner loop and one etcd client/session owner only.
- [ ] No `Arc<...>`, `Mutex<...>`, or equivalent shared-state synchronization remains anywhere in the production DCS implementation. This is a hard requirement, not a best-effort target.
- [ ] The separate modeled `store_healthy` / “health bool” concept is removed from the DCS design. Coordination mode is represented directly by the `DcsView` enum itself.
- [ ] Public degraded/not-trusted reason types are not introduced unless a real external consumer proves they are necessary. If internal reason classification remains, it stays private to `src/dcs`.
- [ ] `/scope/config` decoding and any similar dead DCS legacy are removed unless the rewrite finds a concrete current use and converts it into an intentional typed design.
- [ ] `dcs.init`, `/scope/init`, and `/scope/config` are removed entirely as dead feature surface.
- [ ] All old superseded DCS code is cleaned up. No dead compatibility code, leftover adapter scaffolding, or unused old DCS type family remains in the tree.
- [ ] All old duplicated shared-concept types are cleaned up after the migration. In particular, no parallel old/new copies of epoch, switchover target, observed WAL vector, or PostgreSQL target types remain, and the old DCS member API URL type/path is deleted rather than carried forward.
- [ ] `src/dcs/mod.rs` and the rest of `src/dcs` are audited so unnecessary `pub` and `pub(crate)` visibility is removed aggressively.
- [ ] All DCS code is private by default first. Public visibility is used only for `DcsView` and the nested read-only payload/view types needed to inspect and serialize it; `pub(crate)` is used only where absolutely necessary to realize the crate-private mutation boundary.
- [ ] There are no raw DCS key/path manipulations, raw etcd CRUD calls, or internal DCS record/cache imports outside `src/dcs`.
- [ ] API/CLI output and validation continue to provide the product behavior they currently need, adapted to the public `DcsView`.
- [ ] DCS, HA, process, API, CLI, runtime, logging, and affected docs/tests are updated coherently for the new model.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
