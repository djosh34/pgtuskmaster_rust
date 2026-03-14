## Task: Rewrite DCS As One Private Async Actor With One Public Opaque `DcsView` <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Rewrite the DCS subsystem so it has exactly one owning async loop, exactly one etcd client/session owner, zero `Arc`/`Mutex` inside the production DCS path, and exactly two public concepts for the rest of the codebase: a typed command handle and one public opaque `DcsView`. The higher-order goal is to turn DCS into a small, private coordination domain instead of a collection of storage-shaped types, bridge layers, and leaked implementation details. This is a deliberate simplification task, not a privacy-only wrapper task: the end state must remove code, remove representations, and remove synchronization primitives that only exist because the current design split ownership badly.

This task must not be placed under `story-ctl-operator-experience`. There is an older completed task there, `.ralph/tasks/story-ctl-operator-experience/10-task-collapse-dcs-behind-a-single-private-component-and-read-only-dcs-view.md`, but this new task is a fresh follow-up story focused specifically on aggressive DCS simplification and public-surface collapse.

**Complete redesign decisions already made from research and user discussion:**
- The requirements are intentionally strict and must be treated as hard constraints, not preferences:
  - one loop
  - zero `Arc`
  - zero `Mutex`
  - zero separate “is healthy” / store-health functions in the DCS public model
  - one single exposed DCS view enum
  - writes happen only through exposed typed functions on the DCS command handle
  - all old superseded code must be cleaned up, not left behind
  - all DCS code must be private by default first; visibility should only be widened if it is absolutely necessary to satisfy the final boundary
- A published read-only state channel such as the existing watch-based state pattern is still allowed. The hard ban is specifically on `Arc`/`Mutex` in the production DCS implementation, not on all forms of state publication or subscriptions.
- The current naming `FullQuorum` should be removed. The code today does not compute real quorum mathematics; it only distinguishes store reachability plus minimal observed-member conditions. The replacement naming should use something like `Coordinated`, `Trusted`, or another simpler term that matches what the code actually proves.
- The public DCS state must collapse into one single public type named `DcsView`.
- `DcsView` should be an enum, not a struct with a separate public `trust` field.
- The recommended public shape is an opaque enum with variants that represent the externally meaningful coordination mode while still retaining the last known snapshot internally, for example:
  - `NotTrusted { ... }`
  - `Degraded { ... }`
  - `Coordinated { ... }`
- Do **not** implement the user’s literal recursive idea `NotTrusted | Degraded(DcsView) | FullQuorum(DcsView)`; that is structurally recursive and not the right representation. Use a private inner snapshot payload instead.
- `NotTrusted` must still retain the last known snapshot internally. Current HA behavior still benefits from last known leader/member information even when trust is lost, especially for conservative fencing/following behavior. A payload-less `NotTrusted` would throw away information the current runtime still uses.
- The enum must be **opaque** to non-DCS code. The rest of the repo should interact with it through `impl DcsView` methods rather than by reaching into public nested structs.
- The public surface should be:
  - `DcsView`
  - `impl DcsView { ... }` query methods
  - `DcsHandle` for typed incoming commands
  - the smallest possible public command error type if needed
- All other DCS types must be private unless a public helper type is proven unavoidable.
- Visibility must start at private. The implementer must not default to `pub(crate)` for internal sharing. The desired end state is:
  - public: only `DcsView`, `impl DcsView`, `DcsHandle`, and the smallest required public command error type
  - crate-private: only the smallest bootstrap/runtime wiring needed to construct and run the DCS actor if that cannot be kept inside `src/dcs`
  - everything else: private to `src/dcs`, and preferably private to the smallest possible module
- The current suggestion of a public `reason` payload should **not** be added by default. Research found no current HA/API/process/CLI logic that depends on a public reason classification for `NotTrusted` or `Degraded`. If the implementation still wants reason classification for logging or internal branching, keep that classification private to `src/dcs`. Only promote a reason enum to the public API if a real consumer emerges during implementation and there is no cleaner method-based alternative.
- Prefer variant semantics over bolting on separate public mode/health/reason fields. “Health” as a separate concept should disappear from the public model and from internal bookkeeping where it only duplicates enum state.
- DCS should be rewritten around one async owner loop and direct etcd ownership. The current sync trait + dedicated thread + shared queue bridge must be removed. `Arc` and `Mutex` are not allowed to remain in the production DCS implementation after this rewrite.
- This repo is greenfield. Do not preserve backward compatibility for the old DCS public shape, old type names, or old API wire format if a cleaner boundary requires changing them.

**Scope:**
- Rewrite `src/dcs` around one async actor/loop that directly owns etcd connection state, watch state, leader lease ownership state, and the private in-memory snapshot used to publish `DcsView`.
- Delete the current bridge architecture that exists only to present a synchronous `DcsStore` interface over an async etcd client.
- Reduce the public DCS surface to one opaque `DcsView` enum plus `impl DcsView` methods and the typed `DcsHandle`.
- Remove public and crate-public DCS record/cache/store/helper types that leak implementation structure outside `src/dcs`.
- Update HA, process, API, CLI, runtime wiring, and tests to consume the new `DcsView` methods instead of public nested DCS structs/fields.
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
- `src/runtime/node.rs` currently wires DCS to the rest of the system as one state subscriber and one command handle. That boundary direction is good and should remain, but the state type should become one opaque public enum and the internals should shrink heavily.
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
- Across non-DCS consumers, the truly needed public observations are:
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

**Required new public boundary:**
- `pub enum DcsView` with coordination-mode variants only.
- `impl DcsView` with method-based accessors and query helpers. The exact final method list may evolve, but the boundary should look roughly like:
  - mode queries: `is_not_trusted`, `is_degraded`, `is_coordinated`
  - membership queries: `member_count`, `has_member`, `member_ids`
  - leader queries: `leader_epoch`, `leader_member_id`
  - switchover queries: `has_switchover`, `switchover_target_member`, `switchover_targets_any_ready_replica`
  - endpoint queries: `postgres_endpoint`, `api_url`
  - member-role/readiness queries: `member_is_primary`, `member_is_ready_replica`, `member_readiness`
  - WAL/system identity queries used by HA: `member_system_identifier`, `member_wal_position`, and any similar minimal helpers
- `DcsHandle` must remain the typed write boundary.
- No public nested DCS structs/record enums unless implementation proves a method-based boundary is impossible or dramatically worse.

**Required typed command surface on `DcsHandle`:**
- The command handle is the only allowed mutation path into DCS from outside `src/dcs`.
- At minimum it must expose typed operations equivalent to:
  - acquire local leadership
  - release local leadership
  - publish a switchover request targeting any healthy replica
  - publish a switchover request targeting a specific member
  - clear the current switchover request
- There must be no typed init/bootstrap command, because current investigation did not find a real production consumer for `dcs.init`, `/scope/init`, or `/scope/config`, and this task explicitly removes that dead feature.
- No raw path/string mutation methods may remain on the public boundary.

**Required public `DcsHandle` method signatures:**

```rust
pub enum DcsHandleError {
    ChannelClosed,
}

impl DcsHandle {
    pub fn acquire_leadership(&self) -> Result<(), DcsHandleError>;
    pub fn release_leadership(&self) -> Result<(), DcsHandleError>;
    pub fn publish_switchover_any(&self) -> Result<(), DcsHandleError>;
    pub fn publish_switchover_to(
        &self,
        target: MemberId,
    ) -> Result<(), DcsHandleError>;
    pub fn clear_switchover(&self) -> Result<(), DcsHandleError>;
}
```

Do not expose a public raw `DcsCommand` enum unless implementation proves it is necessary. The preferred boundary is typed methods on the handle, because the design goal is one public state type plus one public mutation handle, not another exposed command ADT.

**Required public command error semantics:**
- The handle should be one-way enqueue only, not request/response.
- The public failure mode should therefore be minimal: failure means the command could not be queued because the DCS actor/handle channel is closed.
- Do not preserve the current request/response error split unless implementation finds a concrete case that truly needs it and the task result documents that case.
- The current code exposes:
  - `ChannelClosed`
  - `Rejected(String)`
  - `Transport(String)`
  in [`src/dcs/command.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dcs/command.rs#L26)
- This task should simplify that model down to a typed non-string public enum:
  - `DcsHandleError::ChannelClosed`
- Mutation outcomes after enqueue should be observed through the published `DcsView` and logging, not through a reply channel.
- Because HA can tick again before a fresh `DcsView` reflecting the mutation is published, the DCS actor must treat duplicate identical commands as safe/idempotent. At minimum:
  - repeated `acquire_leadership` from the same node must not create semantic failure
  - repeated `release_leadership` must collapse to a no-op once local leadership is already gone
  - repeated `clear_switchover` must collapse to a no-op once switchover state is already absent
  - repeated `publish_switchover_any` or `publish_switchover_to(same_target)` must be equivalent to one publish

**Explicit design sketch that this task should implement unless code-level constraints force a small, justified variation:**

If the goal is “one public DCS state type, everything else private”, make `DcsView` an opaque enum and force consumers to use methods instead of public fields.

```rust
pub enum DcsView {
    NotTrusted { snapshot: Box<Snapshot>, reason: NotTrustedReason },
    Degraded { snapshot: Box<Snapshot>, reason: DegradedReason },
    Coordinated { snapshot: Box<Snapshot> },
}

pub struct DcsHandle { ... }
```

The task should actually prefer renaming `FullQuorum` to `Coordinated` or `Trusted`. With the current algorithm, `FullQuorum` is misleading. If someone insists on the name `FullQuorum`, then the implementation would also need to add explicit expected voter count and compute real majority, which is not the desired simplification path here.

The critical part is this: `NotTrusted` still needs to carry a snapshot. HA currently still uses the last known leader/member information when trust is lost, for example to fence safely or keep following conservatively in [`src/ha/decide.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/decide.rs#L13). A payload-less `NotTrusted` would throw away information the current system still relies on.

If the final public API really exposes only one public DCS state type, then `Snapshot`, `Member`, `Leader`, `Switchover`, and any internal reason enums should stay private. `DcsView` should become method-driven:

```rust
impl DcsView {
    pub fn is_not_trusted(&self) -> bool;
    pub fn is_degraded(&self) -> bool;
    pub fn is_coordinated(&self) -> bool;

    pub fn member_ids(&self) -> Vec<MemberId>;
    pub fn member_count(&self) -> usize;
    pub fn has_member(&self, id: &MemberId) -> bool;

    pub fn leader_epoch(&self) -> Option<(MemberId, u64)>;
    pub fn switchover_requested(&self) -> bool;
    pub fn switchover_target_member(&self) -> Option<MemberId>;
    pub fn switchover_targets_any_ready_replica(&self) -> bool;

    pub fn postgres_endpoint(&self, id: &MemberId) -> Option<(&str, u16)>;
    pub fn api_url(&self, id: &MemberId) -> Option<&str>;

    pub fn member_is_primary(&self, id: &MemberId) -> bool;
    pub fn member_is_ready_replica(&self, id: &MemberId) -> bool;
    pub fn member_readiness(&self, id: &MemberId) -> Option<Readiness>;
    pub fn member_system_identifier(&self, id: &MemberId) -> Option<SystemIdentifier>;
    pub fn member_wal_position(&self, id: &MemberId) -> Option<(Option<TimelineId>, WalLsn)>;
}
```

That gives the codebase one public state type and one public command handle. Everything else becomes private implementation detail.

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
7. on any etcd failure, drop session, keep last known snapshot, publish `DcsView::NotTrusted { ... }`, reconnect with backoff

The command path should be “just a channel” internally:
- public typed method on `DcsHandle`
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
- `src/dcs/mod.rs` should become the only intentional public DCS module surface.
- `src/dcs/command.rs`
  - Keep the typed command handle concept.
  - Update command payloads if needed for the new model.
  - Keep external API tiny and typed.
  - Ensure the final public command surface covers only the allowed mutations: local leadership acquire/release, switchover publish-any, switchover publish-to-member, and switchover clear.
  - Prefer typed non-async enqueue methods on `DcsHandle` over exposing a public raw command enum.
  - Prefer a minimal enqueue-only error model such as closed-channel/actor-gone.
- `src/dcs/startup.rs`
  - Likely shrink heavily or fold into `mod.rs`.
  - Bootstrap should construct the single actor, one `watch` publisher/subscriber pair, and one `DcsHandle`.
- `src/dcs/state.rs`
  - Replace the current public struct/enum matrix with the new opaque public `DcsView` enum plus private snapshot representation.
  - Remove public `DcsTrust`.
  - Remove public record/cache structs.
  - Remove public worker metadata fields if no longer needed.
- `src/dcs/worker.rs`
  - Rewrite around one async actor with direct etcd ownership.
  - Eliminate the current bridge assumptions and store-health bool propagation.
  - Derive public `DcsView` variants from session + private snapshot state.
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
  - Keep runtime as a composition root that wires the new DCS actor and passes only `DcsHandle` and `StateSubscriber<DcsView>` outward.
- `src/ha/worker.rs`
  - Stop direct field-walking through public DCS structs.
  - Build `WorldView` via `DcsView` methods.
  - Adjust naming from `FullQuorum` to the new mode vocabulary.
- `src/ha/decide.rs`
  - Replace `DcsTrust::FullQuorum` checks with the new `DcsView` mode or a derived HA coordination-mode representation.
- `src/process/worker.rs` and `src/process/source.rs`
  - Replace `DcsMemberView` dependency with `DcsView` methods for source-member lookup and endpoint extraction.
- `src/api/controller.rs`
  - Validate switchovers through `DcsView` methods instead of direct struct access and `DcsTrust`.
- `src/api/mod.rs`, `src/api/worker.rs`, `src/cli/status.rs`, `src/cli/connect.rs`, `src/cli/switchover.rs`
  - Update API/CLI rendering and validation to use the new public boundary.
- `src/logging/event.rs` and any log serialization/tests
  - Update labels/serialization if `DcsTrust` is removed or renamed.
- DCS-related tests in `src/dcs/worker.rs`, HA tests, API tests, CLI tests, and documentation that mentions `FullQuorum`
  - Update to the new model and names.

**Explicit deletion/folding targets from the design discussion:**
- keep `src/dcs/mod.rs` as the only public DCS module surface
- fold most of `src/dcs/startup.rs` into `mod.rs` if that reduces surface and indirection
- delete `src/dcs/store.rs`
- replace `src/dcs/etcd_store.rs` with a much smaller private `etcd.rs` that contains only async helper functions, if a separate file is still useful
- fold `src/dcs/state.rs` into one private snapshot type plus the public `DcsView` enum if that reduces duplication and visibility noise
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
- Prefer one ADT that represents externally visible coordination state over multiple parallel public enums/structs.
- Prefer method-based opaque boundaries over public field bags.
- Prefer compiler-enforced privacy over conventions.
- Prefer deleting dead code to preserving compatibility shims.

**Expected outcome:**
- The repository has one DCS owner loop, one etcd session owner, and no DCS `Arc`/`Mutex`/cross-thread event queue machinery.
- `DcsView` is the only public DCS state type.
- `DcsHandle` is the only public write path into DCS.
- Non-DCS code cannot import or manipulate DCS record/cache/store/key types because those types no longer exist publicly.
- DCS mode is represented by the `DcsView` enum itself; there is no separate public `health` or `trust` field.
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
- [ ] The strict rewrite constraints are satisfied explicitly: one loop, zero `Arc`, zero `Mutex`, zero separate “health” functions/booleans in the DCS public model, one public DCS view enum, and one typed write handle.
- [ ] `src/dcs` is rewritten so the final public surface is only `DcsView`, `impl DcsView` query methods, `DcsHandle`, and the minimum required public command error type.
- [ ] `DcsView` is a public enum whose variants express coordination mode directly; `DcsTrust`/`FullQuorum` no longer exist publicly.
- [ ] The final naming replaces `FullQuorum` with a term that matches the actual semantics, such as `Coordinated`/`Trusted`, and docs/tests/logging are updated consistently.
- [ ] `DcsView` remains opaque to outside code: non-DCS modules do not rely on public nested DCS record structs or public field bags to inspect DCS state.
- [ ] All writing into DCS happens only via exposed typed command-handle functions. No other write path remains.
- [ ] The final `DcsHandle` exposes only typed one-way enqueue mutations for:
  - acquiring local leadership
  - releasing local leadership
  - publishing switchover to any healthy replica
  - publishing switchover to a specific member
  - clearing switchover
- [ ] The final public `DcsHandle` surface is method-based, with signatures equivalent to:
  - `pub enum DcsHandleError { ChannelClosed }`
  - `acquire_leadership(&self) -> Result<(), DcsHandleError>`
  - `release_leadership(&self) -> Result<(), DcsHandleError>`
  - `publish_switchover_any(&self) -> Result<(), DcsHandleError>`
  - `publish_switchover_to(&self, MemberId) -> Result<(), DcsHandleError>`
  - `clear_switchover(&self) -> Result<(), DcsHandleError>`
- [ ] The final public command path is one-way enqueue only. No request/response reply channel remains in the public mutation boundary.
- [ ] The final public command error model is minimal and enqueue-oriented. It should not preserve “rejected” / “transport” failure classes unless implementation proves a concrete need and documents it.
- [ ] Duplicate identical commands are explicitly safe. If HA/API sends the same mutation again before `DcsView` catches up, the DCS actor handles that as an idempotent no-op/equivalent operation rather than a semantic error.
- [ ] All non-DCS consumers (`src/ha`, `src/process`, `src/api`, `src/cli`, runtime wiring, and relevant tests) are updated to use `DcsView` methods and `DcsHandle` only.
- [ ] `src/dcs/store.rs` is deleted, or any tiny unavoidable remnants are moved private inside `src/dcs` with no store-like boundary exposed outside the owning actor.
- [ ] The old `src/dcs/etcd_store.rs` thread/bridge architecture is deleted or replaced by a substantially smaller private async etcd helper with no `Arc`, no `Mutex`, and no cross-thread watch event queue in the DCS path.
- [ ] The final DCS runtime path has one async owner loop and one etcd client/session owner only.
- [ ] No `Arc<...>`, `Mutex<...>`, or equivalent shared-state synchronization remains anywhere in the production DCS implementation. This is a hard requirement, not a best-effort target.
- [ ] The separate modeled `store_healthy` / “health bool” concept is removed from the DCS design. Coordination mode is represented by the `DcsView` enum itself.
- [ ] Public degraded/not-trusted reason types are not introduced unless a real external consumer proves they are necessary. If internal reason classification remains, it stays private to `src/dcs`.
- [ ] `/scope/config` decoding and any similar dead DCS legacy are removed unless the rewrite finds a concrete current use and converts it into an intentional typed design.
- [ ] `dcs.init`, `/scope/init`, and `/scope/config` are removed entirely as dead feature surface.
- [ ] All old superseded DCS code is cleaned up. No dead compatibility code, leftover adapter scaffolding, or unused old DCS type family remains in the tree.
- [ ] `src/dcs/mod.rs` and the rest of `src/dcs` are audited so unnecessary `pub` and `pub(crate)` visibility is removed aggressively.
- [ ] All DCS code is private by default first. `pub(crate)` is used only where absolutely necessary to realize the final boundary. The expected public surface is essentially only the view/handle boundary.
- [ ] There are no raw DCS key/path manipulations, raw etcd CRUD calls, or internal DCS record/cache imports outside `src/dcs`.
- [ ] API/CLI output and validation continue to provide the product behavior they currently need, adapted to the new DCS public boundary.
- [ ] DCS, HA, process, API, CLI, runtime, logging, and affected docs/tests are updated coherently for the new model.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
