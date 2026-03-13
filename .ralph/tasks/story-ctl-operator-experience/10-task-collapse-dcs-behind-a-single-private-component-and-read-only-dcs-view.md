## Task: Collapse DCS Behind A Single Private Component And A Read-Only `DcsView` <status>not_started</status> <passes>false</passes>

<priority>high</priority>

<description>
**Goal:** Refactor the DCS implementation so the repository has one DCS owner component total, one etcd client total, no raw DCS store access outside `src/dcs`, and one public read-only typed view of DCS state for the rest of the system. The rest of the codebase must stop treating DCS as a shared raw key-value space with leaked record types and leaked path-based mutation primitives. Instead, all DCS writes and lease operations must flow through the DCS component itself, and all non-DCS code must interact with DCS only through a narrow command/query boundary and a read-only typed state view.

**Higher-order goal:** The current implementation is architecturally spaghetti because there is no real DCS boundary. Runtime currently constructs three etcd-facing stores: one for the DCS worker, one separate leader-lease store for HA, and one separate plain store for API. The DCS worker writes member slots and also performs stale `/leader` cleanup; HA directly acquires/releases leader lease through its own store; API directly writes and deletes the switchover key. At the same time, many non-DCS modules import raw DCS storage types such as `DcsState`, `DcsCache`, `MemberSlot`, and `SwitchoverIntentRecord` and derive their own semantics from those internals. The result is split write ownership, duplicated meaning, and bugs such as a non-leader node deleting `/leader` based only on its cache view. This task must replace that architecture with a real single-owner DCS subsystem.

**Decisions already made from user discussion and research:**
- There must be exactly one etcd client / DCS owner component in the running node.
- DCS internals must stay internal to `src/dcs`. Other modules must not directly access DCS keys, raw etcd paths, raw key parsing, raw DCS store traits, or internal record/cache types.
- Non-DCS code must not mutate DCS by calling `write_path`, `delete_path`, `acquire_leader_lease`, `release_leader_lease`, or similar low-level operations. All DCS mutations must go through DCS-owned commands.
- `NodeState` must still expose DCS information, but only as one public read-only typed view. It is acceptable and preferred to introduce a new `DcsView` type for that public surface, while keeping the field name `dcs` if that makes the API clearer.
- Public DCS exposure should be reduced to the smallest useful surface. A good target is one public `DcsView` plus any tiny helper enums needed by public API DTOs; everything else should be crate-private or module-private.
- The repo should end with verified cleanup of old code, methods, and tests related to the leaked / duplicated DCS boundary. Dead or disabled legacy code should be removed rather than preserved.
- Verification must explicitly prove there are no other DCS / etcd clients left anywhere outside the single DCS owner.
- The task is implementation work, not a design-only note. It must carry the refactor through code changes, cleanup, and validation.

**Current implementation facts from research that motivate this task:**
- `src/runtime/node.rs` currently creates three etcd-facing stores:
  - a DCS worker store via `EtcdDcsStore::connect(...)`
  - a separate HA lease store via `EtcdDcsStore::connect_with_leader_lease(...)`
  - a separate API store via `EtcdDcsStore::connect(...)`
- `src/dcs/mod.rs` currently exports `pub mod keys`, `pub mod state`, and `pub mod store`, which leaks DCS internals across the crate and to external users of the library.
- `src/dcs/store.rs` currently exposes low-level path-based operations such as `read_path`, `write_path`, `write_path_with_lease`, `put_path_if_absent`, `delete_path`, and `snapshot_prefix`. These are adapter-level etcd/KV primitives, not a coherent domain boundary.
- `src/api/controller.rs` currently mutates DCS directly by writing and deleting `/{scope}/switchover`.
- `src/ha/worker.rs` currently mutates DCS directly via `DcsLeaderStore::acquire_leader_lease`, `release_leader_lease`, and `clear_switchover`.
- `src/dcs/worker.rs` currently mutates DCS directly for local member slots and also performs stale leader cleanup by deleting `/{scope}/leader` when its cache says the holder is missing. This is what currently allows a non-leader node to delete `/leader`.
- `src/ha/worker.rs`, `src/ha/source_conn.rs`, `src/ha/process_dispatch.rs`, `src/api/controller.rs`, `src/api/mod.rs`, `src/api/worker.rs`, `src/cli/status.rs`, `src/cli/connect.rs`, `src/cli/switchover.rs`, and `src/runtime/node.rs` currently import raw DCS state types.
- There is semantic duplication today:
  - raw storage/cache types in `src/dcs/state.rs` such as `LeaderLeaseRecord`, `MemberSlot`, and `SwitchoverIntentRecord`
  - HA-domain types in `src/ha/types.rs` such as `LeaseEpoch`, `LeaseState`, and `SwitchoverRequest`
  Some duplication between storage and domain can be legitimate, but the current code leaks storage types outward and then rebuilds overlapping higher-level meaning elsewhere.
- `src/runtime/node.rs` contains dead disabled legacy test code under `#[cfg(all(test, any()))]` with old DCS type names and shapes. That dead code should be removed during this cleanup instead of left around as archaeological noise.
- `src/dcs/etcd_store.rs` does use the real `etcd-client` crate directly, but it currently wraps that library in a large amount of extra machinery:
  - a dedicated OS thread for the store worker
  - a second dedicated keepalive thread for leader lease maintenance
  - an `Arc<AtomicBool>` health flag
  - an `Arc<Mutex<VecDeque<WatchEvent>>>` event queue
  - command channels and per-request response channels to emulate a synchronous store trait over an async etcd client
  This task must not assume all of that machinery is inherently necessary just because it exists today.

**Explicit simplification requirement, not optional cleanup:**
- This task must aggressively reduce DCS complexity, not merely move the same complexity behind a private wall.
- The implementer must actively challenge whether each of the following is still needed in the final design:
  - private store traits that only exist to emulate raw KV access
  - dedicated OS threads for DCS worker and lease keepalive when a simpler async task model would work
  - `Arc`, `Mutex`, `AtomicBool`, and queued watch-event plumbing used only because the current design splits ownership awkwardly
  - path-string CRUD command wrappers that exist only to preserve the old store API
  - duplicate snapshot + event-drain + cache-refresh plumbing that can be collapsed once DCS is a single owner
- The desired outcome is less code, fewer moving parts, fewer synchronization primitives, and fewer representations of the same facts.
- If some internal cache or channel remains necessary after the refactor, that is acceptable only when it has a clear architectural reason and a much smaller footprint than today.

**Explicit guidance on the internal cache question:**
- The task must reconsider why DCS has an internal cache at all and document the final answer in code structure.
- A small internal cache/read model is acceptable if it is the single DCS-owned source for publishing the latest `DcsView` to HA/API/CLI consumers and for handling watch-driven updates coherently.
- What is not acceptable is the current combination of:
  - one internal cache shape leaked outside DCS
  - separate watch event queues
  - separate direct RPC success health semantics
  - multiple external store clients doing their own reads/writes around that cache
- The final design should prefer one coherent internal read model if a cache is still needed, or remove internal layers entirely if they are not justified. The task must not preserve the current cache/event/store layering out of inertia.

**Required architectural target:**
- DCS becomes a real component with:
  - one etcd client / one watch session owner
  - one internal cache / watch application path
  - one internal command inbox for DCS mutations
  - one published read-only `DcsView`
- HA and API stop holding etcd-facing DCS stores.
- HA and API communicate with DCS through narrow commands only.
- DCS keys, key parsing, etcd path formats, lease IDs, watch resets, and store traits are internal implementation details.
- DCS publishes a read-only view that is sufficient for:
  - HA observation and decision input
  - API `/state`
  - CLI status / connect / switchover validation
  without exposing internal cache mutation or raw etcd keyspace mechanics.

**Exact boundary contract that must exist after the refactor:**

The boundary must be explicit and small. After this task, all non-DCS code must interact with DCS through exactly two concepts only:

1. A public read-only DCS state surface
- Name: `DcsView` or an equivalently clear final name
- Purpose: this is the only DCS data shape that may appear in `NodeState` or be consumed directly by HA/API/CLI/runtime consumers outside `src/dcs`
- Properties:
  - typed
  - read-only
  - serializable for API output if needed
  - contains only product/domain-facing coordination information
  - does not expose internal watch, cache, lease-ownership, revision, or raw key/path details

2. A public typed DCS command surface
- Name: `DcsHandle`, `DcsClient`, or equivalently clear final name
- Purpose: this is the only way non-DCS code may ask DCS to mutate coordination state
- Properties:
  - exposes typed operations only
  - does not expose raw etcd CRUD/path APIs
  - does not expose raw lease IDs or watcher/session objects
  - can be implemented with an inbox/sender plus internal worker processing, but that mechanism itself remains internal

There must not be any third boundary. In particular, the final architecture must not leave behind a second "private but still externally passed around" low-level DCS store trait or etcd adapter used by HA/API/runtime. The public boundary is only:
- read-only `DcsView`
- typed command handle for DCS-owned mutations

**Exact public DCS operations that the rest of the codebase may request after the refactor:**
- request local leader acquisition
- request local leader release
- publish switchover request
- clear switchover request
- if startup/init still needs DCS persistence, one explicit typed init-lock/config-seeding command rather than raw KV operations

The implementation may choose final method names, sync/async shape, and concrete transport, but the important boundary rule is fixed: the rest of the codebase may request DCS actions only through typed commands like the above, never through raw store/path methods.

**Exact internal-only DCS layers after the refactor:**
- etcd adapter/client code
- watch session establishment and watch stream management
- bootstrap snapshot and reconnect logic
- lease grant/keepalive/revoke internals
- key/path encoding and decoding
- persisted/wire/internal record structs
- internal mutable cache structs
- raw store traits/helpers if any still exist for the adapter implementation

These layers must remain inside `src/dcs` only. They are implementation details, not part of the architectural boundary.

**Exact naming/shape guidance for types so the boundary stays understandable:**
- Use `*View` for public read-only DCS surfaces consumed outside `src/dcs`
- Use `*Record` for persisted/wire/internal DCS storage types
- Use `*Cache` only for internal mutable DCS in-memory mirrors
- Keep worker lifecycle / command plumbing separate from public read models

This task must not leave the codebase in a state where a storage-shaped type is publicly exposed and then reinterpreted elsewhere as if it were the domain boundary.

**Exact things that must become impossible outside `src/dcs`:**
- importing or constructing `EtcdDcsStore`
- importing DCS keys or key parsers
- computing DCS raw paths such as `/{scope}/leader` or `/{scope}/switchover`
- calling raw path methods like `read_path`, `write_path`, `write_path_with_lease`, `put_path_if_absent`, `delete_path`, or `snapshot_prefix`
- calling raw lease methods like `acquire_leader_lease`, `release_leader_lease`, or any future equivalent low-level lease primitive
- constructing or mutating internal DCS cache structs directly in non-DCS code
- depending on raw persisted DCS record types in HA/API/CLI/runtime code

The final code should enforce this through Rust module privacy, not just through convention.

**Exact way `NodeState` must expose DCS after the refactor:**
- `NodeState` must keep a `dcs` field because DCS state remains part of the product surface
- that field must contain the public read-only `DcsView`
- it must not contain internal worker state, raw cache state, raw record structs, or low-level transport/storage details

**Exact module-boundary expectation after the refactor:**
- `src/dcs/mod.rs` should re-export only the intentionally public DCS surface
- likely public/re-exported items:
  - `DcsView`
  - tiny helper enums/views needed by `DcsView`
  - typed DCS command handle
  - any minimal error type required by that command handle
- likely private modules/items:
  - `keys`
  - `etcd_store`
  - internal `store`
  - internal `record`
  - internal `cache`
  - internal `worker`
  - any other etcd/watch/lease implementation detail

The exact final file names may differ, but the privacy outcome is required. If introducing dedicated `view.rs`, `command.rs`, `record.rs`, `cache.rs`, or `component.rs` makes the boundary clearer, do that.

**Recommended shape of the new boundary:**
- Introduce one DCS-owned command interface, for example a sender/inbox plus typed commands such as:
  - request or attempt leadership acquisition for `self_id`
  - release locally owned leadership
  - publish switchover request
  - clear switchover request
  - any init-lock command that startup may need if that logic is still DCS-backed
- Introduce one public read-only `DcsView` type for consumption outside `src/dcs`.
- Keep internal record/cache/storage types inside `src/dcs`, with `*Record` naming where they are persisted/wire/internal.
- If HA still needs richer derived coordination facts than `DcsView` alone, derive those in one place from `DcsView` rather than from raw internal records and maps scattered around the codebase.

**Expected ownership model after the refactor:**
- DCS owns:
  - member slot publication / expiry
  - leader lease key acquisition / keepalive / release
  - switchover key persistence
  - DCS watch health and snapshot refresh
  - any DCS init-lock persistence
  - all raw etcd keys and path encodings
- HA owns:
  - deciding desired authority / role
  - asking DCS to acquire or release leadership when local policy requires it
  - consuming the read-only `DcsView`
- API owns:
  - validating user requests using `DcsView` and HA state
  - asking DCS to store or clear switchover state
- CLI owns:
  - consuming the public `NodeState` / `DcsView` surface only

**Explicit implementation-quality expectations for the DCS rewrite:**
- Prefer one async DCS component/task model over thread-plus-sync-bridge machinery unless the code can justify a specific exception.
- Prefer direct typed command handling over generic raw store command enums carrying path strings.
- Prefer one internal source of truth for the current DCS view rather than separate mutable event buffers and later cache replay layers.
- Prefer removing synchronization primitives entirely where ownership can make them unnecessary.
- If `Arc`, `Mutex`, atomics, extra channels, or extra worker threads remain in the final DCS implementation, they must exist for a clear reason tied to the final single-owner architecture, not as leftovers from the current multi-client bridge design.
- The refactor should measurably reduce DCS code size and conceptual surface area. Do not accept a result that merely privatizes the current complexity without shrinking it.

**Important behavior note to preserve while simplifying architecture:**
- The current HA loop releases leader lease for more than switchovers. Research showed release is currently requested in `src/ha/reconcile.rs` not only during switchover demotion but also when the node is fenced or has already demoted/offlined after detecting foreign leadership or storage-stall fencing. The intent is to withdraw authority immediately instead of waiting only for TTL expiry. That behavioral intent may still be valid, but the actual release operation must become a DCS-owned internal action triggered by one explicit DCS command from the local node, not a direct external store call and never a foreign-node raw key delete.

**Scope:**
- Refactor `src/dcs/` into a private implementation module with a deliberately tiny public surface.
- Remove multiple etcd-store instances from runtime wiring and replace them with one DCS owner component.
- Replace external `DcsStore` / `DcsLeaderStore` usage with DCS-owned commands and subscriptions/views.
- Replace public/internal DCS type leakage with one public read-only `DcsView` and private internal record/cache/store types.
- Update HA, API, CLI, and runtime to consume `DcsView` and/or DCS commands instead of raw DCS internals.
- Remove dead legacy DCS code and tests that no longer fit the new boundary.
- Add explicit verification that there is exactly one etcd/DCS client left and no stray path-based DCS mutations remain outside `src/dcs`.

**Out of scope:**
- Do not redesign HA policy itself beyond what is necessary to route DCS actions through the new boundary.
- Do not preserve backward compatibility for the old leaked DCS module surface. This is greenfield; remove the old surface.
- Do not leave temporary compatibility shims such as duplicate old/new DCS APIs or `pub` re-exports of internal DCS record types. Finish the boundary cleanup in one pass.

**Concrete implementation plan and files to inspect/edit:**
- `src/lib.rs`
  - Stop exporting the full DCS module publicly if possible. Prefer a private `dcs` module plus explicit public re-exports only for the new read-only `DcsView` surface if needed.
- `src/dcs/mod.rs`
  - Make internal submodules private by default.
  - Re-export only the minimal intended surface.
- `src/dcs/keys.rs`
  - Make fully private to DCS. No external module should import or depend on DCS keys directly.
- `src/dcs/store.rs`
  - Remove low-level public path-based store traits from external use.
  - Convert to private/internal adapter interfaces if still needed inside DCS.
  - Replace raw mutation APIs with internal command handling or typed internal helpers.
- `src/dcs/etcd_store.rs`
  - Keep etcd-specific code private to DCS.
  - Ensure the one etcd client / watch owner lives only here, behind DCS.
  - Remove any assumptions needed only because HA/API held separate stores.
  - Simplify the current thread/channel/queue/atomic structure aggressively; do not preserve it by default.
- `src/dcs/state.rs`
  - Split internal cache/storage/worker state from public read-only state.
  - Introduce or move the public read-only `DcsView` here or in a dedicated `view.rs`.
  - Make internal cache structures and internal records private or crate-private.
  - Rename leaked persisted/internal types to `*Record` where clarity is needed.
  - Minimize the number of state representations; do not keep parallel shapes unless they are materially different boundary models.
- `src/dcs/worker.rs`
  - Turn this into the single owner of DCS mutation side effects.
  - Consume typed DCS commands from HA/API/startup instead of assuming all writes originate locally in raw store callers.
  - Remove foreign raw `/leader` deletion behavior. Leader-key changes must follow the DCS-owned lease semantics, not cache-based foreign cleanup.
  - Re-evaluate whether stale leader cleanup should become lease-expiry-driven only, locally owned release only, or some stricter DCS-owned reconciliation rule.
  - Re-evaluate whether the current watch-event draining and refresh layering should exist at all in the final structure.
- `src/runtime/node.rs`
  - Replace the three-store wiring with one DCS component instance total.
  - HA and API should receive DCS command handles plus a subscriber/read-only view, not separate etcd stores.
  - Remove disabled dead legacy DCS test code under `#[cfg(all(test, any()))]`.
- `src/ha/state.rs`
  - Replace `Box<dyn DcsLeaderStore>` with a DCS command handle type.
  - Replace `StateSubscriber<DcsState>` with the new public read-only DCS view subscriber if needed.
- `src/ha/worker.rs`
  - Stop importing raw internal DCS storage types.
  - Observe through `DcsView`.
  - Send leadership acquire/release and switchover-clear requests through the DCS command interface.
- `src/ha/reconcile.rs`
  - Keep policy if still correct, but route resulting DCS actions through the new boundary.
  - Validate the reasons for lease release and make them explicit in terms of local-authority withdrawal rather than raw store mechanics.
- `src/ha/source_conn.rs`
  - Stop depending on raw internal `MemberSlot` if that is storage-internal. Either consume an exposed read-only member view type or use helper selectors derived from `DcsView`.
- `src/ha/process_dispatch.rs`
  - Stop pulling source members directly from raw internal DCS cache types. Use `DcsView` or DCS-provided helper translation.
- `src/ha/types.rs`
  - Reduce duplication where possible after the DCS boundary is narrowed.
  - Keep HA-domain types that are genuinely higher-level, but avoid carrying duplicated trust/lease shapes if they can be normalized more cleanly.
- `src/api/controller.rs`
  - Stop taking `&mut dyn DcsStore`.
  - Validate using `DcsView`.
  - Send switchover write/delete requests through DCS commands only.
- `src/api/worker.rs`
  - Replace the DCS store dependency with DCS command handle(s).
  - Keep exposing `NodeState`, but its `dcs` field should become the new public read-only DCS view type.
- `src/api/mod.rs`
  - Update `NodeState` so `dcs` contains the public read-only `DcsView` instead of internal worker/cache state.
- `src/cli/status.rs`
  - Update to read only the new public DCS view shape.
  - Avoid depending on raw internal storage types.
- `src/cli/connect.rs`
  - Resolve targets from the public DCS view only.
- `src/cli/switchover.rs`
  - Validate from `NodeState.dcs` public view only.
- Search the whole repo for current DCS leaks and remove them:
  - imports of `dcs::state::*`
  - imports of `dcs::store::*`
  - direct construction of internal DCS cache structs in non-DCS modules
  - direct raw store calls such as `write_path`, `delete_path`, `acquire_leader_lease`, `release_leader_lease`, `clear_switchover`
  - direct `EtcdDcsStore::connect` or `connect_with_leader_lease` outside the single runtime-owned DCS component bootstrap path

**Concrete verification instructions that must be performed during the task:**
- Verify there is one etcd/DCS client owner only. A grep such as `rg -n "EtcdDcsStore::connect|EtcdDcsStore::connect_with_leader_lease" src` should show only the final single DCS bootstrap site.
- Verify no external raw DCS mutation remains. Greps for `write_path(`, `delete_path(`, `acquire_leader_lease(`, `release_leader_lease(`, `clear_switchover(` outside `src/dcs/` should either return nothing or only DCS-internal command plumbing.
- Verify no non-DCS module imports internal DCS storage/cache modules. Greps for `dcs::state::`, `dcs::store::`, and `dcs::keys::` outside `src/dcs/` must be reduced to the intentional minimal public `DcsView` surface only.
- Verify the old disabled DCS test/code paths are removed, not just ignored.
- Verify `NodeState` still exposes DCS information through the public read-only view.

**Expected outcome:**
- The codebase has one real DCS component boundary instead of three separate etcd clients and leaked raw stores.
- Only `src/dcs` knows about etcd keys, etcd leases, watch streams, and raw DCS records.
- HA and API no longer mutate etcd directly.
- `NodeState` still exposes DCS state, but only through one read-only typed `DcsView`.
- The `/leader` key cannot be deleted by arbitrary non-owner code paths outside the DCS-owned boundary.
- The repo is easier to reason about because DCS has one owner, one command surface, one read model, and much less leaked storage detail.
- The DCS implementation itself is substantially smaller and simpler than before, with unnecessary `Arc`/`Mutex`/thread/channel/store-bridge machinery removed rather than merely hidden.

</description>

<acceptance_criteria>
- [ ] `src/runtime/node.rs` is refactored so the running node constructs exactly one etcd/DCS client owner component total; the previous three-store split is removed
- [ ] `src/dcs/mod.rs` no longer publicly exposes internal modules such as `keys`, `store`, and internal state/cache plumbing
- [ ] `src/dcs/keys.rs` is internal-only; no code outside `src/dcs/` accesses DCS keys or path parsing directly
- [ ] `src/dcs/store.rs` low-level raw path-based mutation APIs are no longer part of the external DCS boundary
- [ ] `src/dcs/etcd_store.rs` remains an internal adapter and is not used directly by HA, API, CLI, or other non-DCS modules
- [ ] `src/dcs/state.rs` and/or a new DCS view module define one public read-only typed `DcsView` surface while keeping internal cache/storage/worker types private or crate-private
- [ ] The final DCS implementation explicitly simplifies the current concurrency and plumbing model instead of preserving it wholesale behind a private boundary
- [ ] Any remaining internal cache, channels, worker threads, `Arc`, `Mutex`, or atomics in DCS are justified by the final single-owner architecture and are materially fewer/smaller than today
- [ ] `src/api/mod.rs` `NodeState` keeps a `dcs` field, but that field contains the public read-only DCS view type rather than leaked internal DCS worker/cache state
- [ ] `src/ha/state.rs` no longer holds `Box<dyn DcsLeaderStore>`; HA uses a DCS command handle instead
- [ ] `src/api/worker.rs` no longer holds `Box<dyn DcsStore>`; API uses DCS command handle(s) instead
- [ ] `src/api/controller.rs` no longer performs raw `write_path` / `delete_path` operations against DCS; switchover writes and clears go through DCS commands only
- [ ] `src/ha/worker.rs` no longer performs direct leader lease store operations; leadership acquire/release and switchover clear go through DCS commands only
- [ ] `src/dcs/worker.rs` becomes the single owner of DCS writes and lease side effects
- [ ] Foreign raw deletion of `/{scope}/leader` is removed; `/leader` lifecycle is owned by DCS lease semantics and locally initiated DCS commands only
- [ ] `src/ha/worker.rs`, `src/ha/source_conn.rs`, `src/ha/process_dispatch.rs`, `src/api/controller.rs`, `src/api/mod.rs`, `src/api/worker.rs`, `src/cli/status.rs`, `src/cli/connect.rs`, `src/cli/switchover.rs`, and `src/runtime/node.rs` are updated to depend only on the public read-only DCS view / DCS command boundary, not internal DCS cache/store/key types
- [ ] Type cleanup is completed so internal persisted/wire/cache types are clearly separated from the public read-only DCS view and from HA-domain types
- [ ] Redundant or dead old DCS code/tests are removed, including the disabled legacy DCS test block in `src/runtime/node.rs`
- [ ] A repo-wide verification confirms there is exactly one DCS/etcd client owner left and no other `EtcdDcsStore::connect` / `connect_with_leader_lease` call sites outside that single bootstrap path
- [ ] A repo-wide verification confirms there are no raw DCS mutation calls outside `src/dcs/`
- [ ] A repo-wide verification confirms there are no non-DCS imports of internal DCS modules/types beyond the intentional public read-only view surface
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
