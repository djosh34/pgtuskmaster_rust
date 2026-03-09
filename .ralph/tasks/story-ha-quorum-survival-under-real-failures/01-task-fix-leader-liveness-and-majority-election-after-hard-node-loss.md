## Task: Fix Leader Liveness And Majority Election After Hard Node Loss <status>not_started</status> <passes>false</passes>

<priority>high</priority>

<description>
**Goal:** Change the HA coordination model so a surviving quorum can elect a new leader after a hard leader or whole-node death, even when the dead node cannot run cleanup code. The higher-order goal is to make pgtuskmaster behave like a real quorum-based HA system under abrupt node loss, not only under the narrower case where PostgreSQL dies but the HA runtime remains alive long enough to release leadership deliberately.

**This task is the model fix that must land before the new real-world E2E tests can be meaningful.** The current end-to-end tests mainly stop PostgreSQL while leaving the runtime alive, which masks the stale-leader deadlock. This task must remove that deadlock at the source.

**Files and areas that must be audited and are expected to change:**
- `src/dcs/store.rs`
- `src/dcs/etcd_store.rs`
- `src/dcs/state.rs`
- `src/dcs/worker.rs`
- `src/ha/decision.rs`
- `src/ha/decide.rs`
- `src/ha/apply.rs`
- any runtime wiring touched by the new DCS store interface in `src/runtime/node.rs`
- directly related unit/integration tests under `src/dcs/`, `src/ha/`, and any worker-contract tests that depend on the store interface
- relevant docs in `docs/src/reference/`, `docs/src/explanation/`, and `docs/src/how-to/handle-primary-failure.md`

**Current source-backed behavior that is broken:**
- `/scope/leader` is currently written as a plain key through `put_path_if_absent(...)` in `src/dcs/store.rs`. It is not attached to an etcd lease and therefore does not expire automatically when the owning node dies.
- The etcd-backed store does provide a watch-fed cache and resnapshot behavior. It bootstraps from a full snapshot and then applies watch events. When etcd emits a delete, the DCS cache will eventually drop that key. This part is basically correct.
- However, the store does not itself guarantee that leadership data is live rather than stale. The DCS state can still contain a leader record for a dead node if the key remains in etcd, because it is a plain key today.
- `evaluate_trust(...)` in `src/dcs/state.rs` currently downgrades trust to `FailSafe` whenever a leader record exists but that leader member record is missing or stale.
- `decide_phase(...)` in `src/ha/decide.rs` blocks all ordinary election logic unless `DcsTrust` is `FullQuorum`.
- `is_available_primary_leader(...)` in `src/ha/decision.rs` also treats missing leader member metadata too leniently, which is only hidden today because the trust gate trips first.
- Combined, those rules mean a hard leader death can leave a stale leader record behind and strand a healthy 2-of-3 majority in `FailSafe` forever.

**Clarification about how the DCS store works today, because this is easy to misread from the code:**
- The store/watch path already behaves roughly like “present what etcd currently contains” once etcd itself changes. The startup snapshot plus watch stream are the source of truth for the DCS cache, and `refresh_from_etcd_watch(...)` correctly removes keys when it receives delete events.
- That means the bug is **not** that the worker ignores leader-key deletes. If etcd deletes `/scope/leader`, the DCS cache should stop showing it.
- The actual bug is earlier in the chain: etcd is not instructed to delete `/scope/leader` on hard leader death, because leader ownership is not lease-backed today.
- So the current store does **not** guarantee live leadership data. It only reflects current etcd data, and current etcd data can itself remain stale forever for `/scope/leader`.

**Current function-level map of the broken path:**
- `src/dcs/store.rs`
- `DcsHaWriter::write_leader_lease(...)` encodes `LeaderRecord` and calls `put_path_if_absent(...)`.
- `DcsHaWriter::delete_leader(...)` blindly deletes `/{scope}/leader`.
- `src/dcs/etcd_store.rs`
- `put_path_if_absent(...)` is plain conditional key creation, not lease-backed leadership.
- watch/snapshot helpers (`create_watch_stream(...)`, `apply_watch_response(...)`, `drain_watch_events(...)`) correctly propagate etcd changes into the DCS cache, but they can only remove `/leader` after etcd itself removes it.
- `src/dcs/state.rs`
- `evaluate_trust(...)` treats stale/missing leader metadata as an automatic `FailSafe`.
- `src/ha/decision.rs`
- `is_available_primary_leader(...)` currently returns `true` when the leader member record is missing, preserving a phantom “available” leader fact.
- `src/ha/decide.rs`
- `decide_phase(...)` gates normal election behavior on `DcsTrust::FullQuorum`.
- This means the stale-leader bug is a composition problem across the store, DCS trust, and leader-availability layers, not a single isolated branch.

**The intended architecture after this task:**
- `decide.rs` stays pure and remains the only HA decision function. Do not turn it into an etcd/lease manager.
- The DCS worker in `src/dcs/worker.rs` remains responsible for:
- publishing the local member heartbeat,
- consuming watch updates,
- building the DCS cache,
- evaluating trust from the observed cache and configuration.
- The HA layer remains responsible for deciding when leadership should be acquired or released.
- The etcd-specific store implementation in `src/dcs/etcd_store.rs` becomes responsible for the mechanics of leader lease ownership:
- creating an etcd lease with TTL,
- attaching `/scope/leader` to that lease,
- keeping that lease alive while this store instance still owns leadership,
- revoking or releasing only its own lease when HA asks to step down,
- letting etcd expire the key automatically on hard process death.

**Exact architectural rule this task must preserve:**
- The HA loop decides **whether** leadership should be acquired or released.
- The HA apply/dispatch layer requests acquire/release from the DCS layer.
- The etcd store implements **how** a leader lease is materially acquired, kept alive, and released.
- The DCS worker consumes the resulting snapshot/watch-fed etcd state and republishes a cache for HA.
- `decide.rs` must never become the place that manually tracks etcd lease IDs, keepalive timers, or expiry timers.

**This task should use the existing HA TTL config rather than inventing a new one.**
- The TTL for leader liveness must come from the existing runtime configuration, specifically the existing HA lease TTL config already used elsewhere for freshness semantics.
- Do not hard-code a new TTL and do not invent a second operator-facing leader-timeout field unless there is an unavoidable source-backed reason. If a new field becomes unavoidable, document why the existing `ha.lease_ttl_ms` could not serve both purposes, but the default intent is to reuse the existing config.

**Required behavior after the fix:**
- In a 3-node cluster, if the current leader dies hard and stops updating anything, the surviving 2-of-3 quorum must be able to elect exactly one new primary before the dead node is healed.
- In that same case, the healthy majority side must not become stuck in `FailSafe`.
- In a true no-quorum case, such as only 1-of-3 remaining, `FailSafe` should still happen. `FailSafe` is still a valid safety mode; it just must not be the steady-state outcome for a healthy majority with a dead old leader.
- In a minority partition, the minority still must not self-promote.
- When the old primary later returns, it must rejoin safely rather than reclaim leadership automatically.

**Explicit implementation plan and constraints:**
- Change the DCS store interface in `src/dcs/store.rs` so the concept of “leader lease” means a real expiring ownership token, not a plain conditional key write.
- The store API should become explicit enough that `src/ha/apply.rs` can call acquire/release semantics without needing to know etcd lease IDs.
- The store API change should be explicit at the trait level. The assignee should not keep the current misleading names if they still imply plain-key behavior. Rename as needed so the trait communicates “lease-backed leadership ownership”.
- The etcd store implementation in `src/dcs/etcd_store.rs` must:
- acquire an etcd lease using the configured TTL,
- write `/scope/leader` attached to that lease,
- keep the lease alive in the background while owned,
- stop keepalive and let expiry happen automatically on hard death,
- ensure explicit release only releases the lease owned by that node/store instance,
- not allow a node to blindly delete a foreign leader key.
- The etcd lease TTL must come from the existing runtime configuration field `ha.lease_ttl_ms`. This must be wired through explicitly; do not silently choose a fixed seconds value inside the etcd store.
- The assignee must think through how the etcd store instance keeps ownership state. If ownership state needs to be held in the store instance, that is acceptable, but it must remain encapsulated inside the etcd-specific implementation and not leak into the pure HA decision layer.
- Keep the DCS watch model authoritative for current visible state:
- bootstrap snapshot plus watch stream should continue to define the cache,
- if etcd deletes `/scope/leader` because the lease expired, the watch/update path should remove it from the cache and therefore from the DCS state seen by HA,
- on reconnect/resnapshot, stale local cache state must be replaced by the latest etcd snapshot.
- The assignee must not replace the watch-fed cache model with ad hoc synchronous reads from `decide.rs`. The current snapshot-plus-watch model is correct in principle and should be preserved.
- Update `src/dcs/state.rs` so trust no longer answers the wrong question. Trust should represent whether the node has a trustworthy quorum view, not whether an old leader key still exists.
- Specifically: stale or missing leader metadata must not automatically force `FailSafe` if a healthy quorum still exists.
- Keep self-freshness and quorum-freshness strict. It is still correct to degrade trust when this node cannot trust its own presence or when there are not enough fresh members for quorum.
- Update quorum math if needed so it reflects actual majority rather than the current “at least two fresh members in multi-node clusters” shortcut.
- Update `src/ha/decision.rs` so stale or missing leader metadata becomes “no active leader”, not “leader is still effectively available”.
- `is_available_primary_leader(...)` must stop returning `true` when the leader member metadata is missing.
- A missing, stale, unhealthy, or non-primary leader record should result in “no active leader”, which then allows the normal election path to run when trust is otherwise good enough.
- Keep `src/ha/decide.rs` mostly intact and pure. The point is to feed it correct DCS semantics, not to rewrite the phase machine.
- Keep lease ownership mechanics out of `decide.rs` and out of the DCS poller loop.

**Non-goals and forbidden shortcuts:**
- Do not “fix” this by weakening safety into best effort.
- Do not special-case “if leader stale then just ignore it” in `decide.rs` while leaving the DCS/store model inconsistent.
- Do not move leadership heartbeat/lease semantics into ad hoc HA-loop timers inside `decide.rs`.
- Do not keep plain-key leader ownership and merely relax tests; that would preserve the bug.
- Do not add a solution that only works for the current 3-node tests but has no coherent quorum semantics.
- Do not leave the source-level tests implicit. This task is not complete if only E2E tests are added or updated.
- Do not treat “watch cache eventually updates” as sufficient proof. The assignee must prove the full chain: lease-backed deletion in etcd -> watch/delete/reset propagation -> correct DCS trust/election behavior.

**Expected outcome:**
- A 3-node cluster that loses its leader abruptly can still elect one new primary from the surviving 2-of-3 quorum before any manual heal of the dead node.
- `FailSafe` remains a valid mode for no-quorum and minority conditions, but it is no longer the healthy-majority outcome of a dead old leader.
- The HA logic has an explicit, defensible rule for dead-leader expiry that matches quorum semantics and remains compatible with the existing pure decision loop.
- The DCS store presents current visible etcd state to HA via snapshot plus watch updates, including automatic removal of `/leader` when the backing lease expires.
- The code and docs no longer claim “primary failure auto-recovers” while still deadlocking on a stale leader record.

</description>

<acceptance_criteria>
- [ ] Update [src/dcs/store.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dcs/store.rs) so the leader-ownership API is no longer plain-key semantics. The API must clearly express acquire and release of expiring leadership rather than generic put/delete of `/leader`.
- [ ] Update [src/dcs/etcd_store.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dcs/etcd_store.rs) to implement leader ownership with a real etcd lease using the existing configured HA TTL.
- [ ] The etcd store implementation must keep the leader lease alive while owned and let it expire automatically on hard death. This must be source-backed and tested, not left as an implied future improvement.
- [ ] The etcd store implementation must not allow arbitrary foreign-leader deletion as the normal release path; release must correspond to the local owner’s lease/session.
- [ ] The etcd store implementation must wire the lease TTL from the existing runtime config field `ha.lease_ttl_ms`. This must be explicit in code; do not hide a duplicate timeout constant inside the etcd layer.
- [ ] The etcd store must continue to present current visible etcd state to the DCS worker by snapshot plus watch updates. If etcd deletes `/scope/leader` due to lease expiry, the watched DCS cache must drop that leader record and the HA loop must see that removal through normal DCS state publication.
- [ ] The assignee must verify, in tests, that reconnect/resnapshot behavior still replaces stale cached state after this refactor. Lease-backed leadership must not regress the existing reset/snapshot correctness guarantees.
- [ ] Update [src/dcs/state.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dcs/state.rs) so trust evaluation no longer deadlocks a healthy majority behind a stale dead-leader record.
- [ ] `evaluate_trust(...)` must remain strict about local-self freshness and quorum freshness.
- [ ] `evaluate_trust(...)` must stop treating stale or missing leader metadata as an automatic `FailSafe` trigger when a healthy quorum view still exists.
- [ ] If quorum math is changed, it must be changed explicitly and tested explicitly rather than left as an accidental side effect.
- [ ] Update [src/ha/decision.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/decision.rs) so stale, missing, unhealthy, or non-primary leader metadata yields “no active leader” instead of preserving a phantom available leader.
- [ ] In particular, `is_available_primary_leader(...)` must no longer treat missing leader member metadata as available.
- [ ] Keep [src/ha/decide.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/decide.rs) as the pure HA decision engine. The fix must not move etcd lease mechanics into the decision loop.
- [ ] Update [src/ha/apply.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/apply.rs) so HA lease actions call the new acquire/release semantics cleanly.
- [ ] Update any required runtime wiring in [src/runtime/node.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/runtime/node.rs) so the HA store receives the configured TTL and the new store interface cleanly.
- [ ] Add or update focused unit and integration tests in the source tree, not only E2E tests. At minimum:
- [ ] store-level tests for lease-backed leader acquisition and expiry in `src/dcs/etcd_store.rs`,
- [ ] store-level tests for “only the owner can release its leader lease/session” behavior in `src/dcs/etcd_store.rs`,
- [ ] store/watch tests proving `/scope/leader` disappears from the cache after actual etcd lease expiry and after reconnect/resnapshot,
- [ ] DCS trust tests in `src/dcs/state.rs`,
- [ ] leader-availability fact tests in `src/ha/decision.rs`,
- [ ] HA decision tests in `src/ha/decide.rs`,
- [ ] any worker/apply contract tests needed because the store API changed.
- [ ] Add at least one source-level test that models this exact sequence: leader disappears without releasing leadership, the configured TTL elapses, two healthy members still retain quorum, `/scope/leader` disappears from the DCS-visible state, and one healthy member can become the only primary without manual DCS cleanup.
- [ ] Add at least one negative source-level test that models the minority case under the same dead-leader conditions and proves 1-of-3 still cannot elect itself primary.
- [ ] Add at least one source-level test that proves `FailSafe` still happens for no-quorum or minority conditions after this refactor, so the fix does not erase safety mode entirely.
- [ ] Add at least one source-level test that proves a stale/missing leader member record now becomes “no active leader” instead of “leader still available”.
- [ ] Make the final model explicit in code comments where necessary and in docs: what data is live in DCS state, what invalidates dead leadership, when `FailSafe` still applies, and why the new behavior does not open a split-brain path.
- [ ] Update the relevant docs pages in `docs/src/reference/`, `docs/src/explanation/`, and [docs/src/how-to/handle-primary-failure.md](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/src/how-to/handle-primary-failure.md) so the fail-safe and primary-failure descriptions match the implemented model exactly.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
