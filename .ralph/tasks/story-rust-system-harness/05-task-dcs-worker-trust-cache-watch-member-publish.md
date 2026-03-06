## Task: Implement DCS worker trust evaluation cache updates and member publishing <status>done</status> <passes>true</passes> <priority>high</priority>

<blocked_by>03-task-worker-state-models-and-context-contracts</blocked_by>

<description>
**Goal:** Implement DCS ownership rules: trust evaluation, typed key parsing, cache updates, and local member publishing.

**Scope:**
- Implement `src/dcs/keys.rs`, `src/dcs/state.rs`, `src/dcs/store.rs`, `src/dcs/worker.rs`, `src/dcs/mod.rs`.
- Implement `evaluate_trust`, `build_local_member_record`, `apply_watch_update`, `key_from_path`, `write_local_member`, and `refresh_from_etcd_watch`.
- Ensure DCS worker subscribes directly to pginfo watch and owns `/member/{self_id}` writes.

**Context from research:**
- Plan explicitly forbids external `upsert_member(...)` APIs.

**Expected outcome:**
- DCS state is authoritative and consistent with typed watch and etcd events.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] `DcsTrust` and `DcsState` exactly reflect plan semantics.
- [x] Key parsing rejects malformed paths with typed errors.
- [x] Tests cover quorum transitions (`FullQuorum`, `FailSafe`, `NotTrusted`).
- [x] Integration tests verify local member publish on pginfo version change.
- [x] Run `make check`.
- [x] Run `make test`.
- [x] Run `make lint`.
- [x] Run `make test`.
- [x] On failures, create `$add-bug` tasks for each distinct defect.
</acceptance_criteria>

<execution_plan>
## Detailed Implementation Plan

1. Baseline and task guardrails
- [x] Confirm blocker task `03-task-worker-state-models-and-context-contracts` is already done and keep existing worker contracts compiling while expanding DCS internals.
- [x] Capture a baseline with `cargo check --all-targets` before edits to isolate regressions caused by this task.
- [x] Keep the no-`unwrap` rule for all new runtime code and tests; convert failures to typed errors with clear messages.
- [x] Preserve the design constraint from task context: no external `upsert_member(...)` helper API surface.

2. Define typed DCS key model and parse errors in `src/dcs/keys.rs`
- [x] Extend `DcsKey` to carry typed member identifiers (`MemberId`) instead of raw strings.
- [x] Add a typed parse error enum (invalid prefix, missing member id, unknown key, malformed path segment count).
- [x] Implement `key_from_path(scope, full_path)` that:
- [x] Validates scope-prefixed paths.
- [x] Maps `.../member/{id}`, `.../leader`, `.../switchover`, `.../config`, and `.../init` into `DcsKey`.
- [x] Rejects malformed and unknown paths deterministically.
- [x] Add unit tests covering valid keys plus malformed paths (empty member id, extra segments, wrong scope, unknown leaf).

3. Expand DCS state contracts in `src/dcs/state.rs`
- [x] Keep `DcsTrust::{FullQuorum, FailSafe, NotTrusted}` and formalize evaluation semantics in code comments/tests.
- [x] Add a richer `MemberRecord` payload for local publish and cache projection:
- [x] `member_id`, `role`, `sql/readiness`, optional timeline, optional WAL positions, and last update timestamp/version metadata.
- [x] Keep `LeaderRecord`, `SwitchoverRequest`, and `InitLockRecord` typed and minimal for current HA consumers.
- [x] Expand `DcsWorkerCtx` to include:
- [x] `self_id: MemberId`
- [x] `scope: String`
- [x] `poll_interval: Duration`
- [x] `pg_subscriber: StateSubscriber<PgInfoState>`
- [x] `publisher: StatePublisher<DcsState>`
- [x] `store: DcsStore` (adapter wrapper from `store.rs`)
- [x] Update `worker_contract_tests.rs` sample constructors to satisfy new `DcsWorkerCtx` fields.

4. Implement trust and state transition helpers in `src/dcs/state.rs`
- [x] Implement `evaluate_trust(etcd_healthy, cache, self_id) -> DcsTrust` with explicit deterministic rules:
- [x] `NotTrusted` when etcd/store health is false.
- [x] `FailSafe` when etcd is healthy but cache is incomplete/inconsistent for safe leadership decisions.
- [x] `FullQuorum` only when etcd is healthy and cache satisfies minimal consistency (self member present and leader consistency checks pass).
- [x] Implement `build_local_member_record(self_id, pg_snapshot, now, pg_version)` mapping current `PgInfoState` into DCS member payload.
- [x] Add unit tests that force all three trust outcomes and assert stable local-member projection from `Unknown`, `Primary`, and `Replica` pg states.

5. Build DCS store adapter contracts in `src/dcs/store.rs`
- [x] Introduce a `DcsStore` trait first (watch + write interface) and typed watch event structs (put/delete + path/value + revision metadata) decoupled from worker logic.
- [x] Add store error enum for watch/write/decode failures, and keep connection concerns outside the trait so tests do not require a networked etcd process.
- [x] Implement an in-memory `TestDcsStore` in `store.rs` test module (or dedicated test helper) to drive deterministic worker tests without external binaries.
- [x] Implement `write_local_member(scope, member_record)`:
- [x] Serializes `MemberRecord` to JSON.
- [x] Writes only `/{scope}/member/{self_id}` (ownership rule).
- [x] No generic external member upsert API.
- [x] Implement `refresh_from_etcd_watch(scope, cache, events)`:
- [x] Applies typed parsed events into cache.
- [x] Handles delete/reset semantics for each key type.
- [x] Returns updated cache + health/diagnostic metadata for trust evaluation.
- [x] Keep adapter boundaries narrow so production etcd wiring can be added later without changing worker/state contracts.

6. Implement cache mutation logic in `src/dcs/worker.rs`
- [x] Add `apply_watch_update(cache, key, op, value)` as a pure helper.
- [x] Ensure per-key behavior:
- [x] `Member`: insert/update/remove member map entry.
- [x] `Leader`: set/clear leader record.
- [x] `Switchover`: set/clear switchover request.
- [x] `Config`: replace runtime-config cache field when valid payload is present.
- [x] `InitLock`: set/clear init-lock record.
- [x] Add deterministic handling for unknown/decode-failed values (skip with error propagation to caller, do not panic).
- [x] Unit-test each mutation branch including delete paths.

7. Wire DCS worker step and loop behavior in `src/dcs/worker.rs`
- [x] Implement `step_once(ctx)` to:
- [x] Read latest pg snapshot/version from `pg_subscriber`.
- [x] Write local member when pg version advanced since last published member payload (dedupe by last published pg version in worker context).
- [x] Pull and apply watch updates from store (`refresh_from_etcd_watch`).
- [x] Recompute trust via `evaluate_trust`.
- [x] Publish next `DcsState` via state publisher with monotonic versioning.
- [x] Implement `run(ctx)` as a timed loop invoking `step_once` and sleeping `poll_interval`.
- [x] Keep `step_once` resilient to transient store errors by publishing degraded trust (`NotTrusted`/`FailSafe`) instead of panicking.

8. Keep module exports strict in `src/dcs/mod.rs`
- [x] Export only modules/types used by current and already-planned tasks.
- [x] Avoid broad re-export fanout to keep clippy and dead-code noise low.

9. Test strategy: unit + integration behavior
- [x] `keys.rs` tests:
- [x] Path parsing happy paths and malformed rejection with exact typed errors.
- [x] `state.rs` tests:
- [x] Trust transition matrix (`FullQuorum`, `FailSafe`, `NotTrusted`) and local-member projection from pg variants.
- [x] `worker.rs` tests:
- [x] `apply_watch_update` coverage for each key and delete/update behavior.
- [x] `step_once` publishes expected DCS snapshot/trust and only writes self member path.
- [x] Integration-style test (in `dcs/worker.rs` test module or `tests/`):
- [x] Simulate pginfo version changes via real watch channel publisher/subscriber.
- [x] Assert local member publish occurs exactly on pginfo version change and writes to `/member/{self_id}` path under scope.

10. Verification command sequence (sequential, no parallel top-level cargo/make)
- [x] Run focused DCS tests first for quick feedback.
- [x] Run `make check`.
- [x] Run `make test`.
- [x] Run `make test`.
- [x] Run `make lint`.
- [x] If any required gate fails, create `$add-bug` task files per distinct defect before proceeding.

11. Completion bookkeeping (execution phase only, not during planning)
- [x] Mark task header/status and acceptance checkboxes only after all required gates pass.
- [x] Set `<passes>true</passes>` only after full required suite passes.
- [x] Run `/bin/bash .ralph/task_switch.sh`.
- [x] Commit all changes including `.ralph` updates with required format:
- [x] `task finished 05-task-dcs-worker-trust-cache-watch-member-publish: <summary + evidence + challenges>`
- [x] Append any new cross-task learning to `AGENTS.md`.
- [x] Append progress diary entry before exit.
</execution_plan>

DONE
