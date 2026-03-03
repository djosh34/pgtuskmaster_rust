---
## Task: Implement HA worker select loop and action dispatch wiring <status>done</status> <passes>true</passes> <passing>true</passing> <priority>high</priority>

<blocked_by>04-task-pginfo-worker-single-query-and-real-pg-tests,05-task-dcs-worker-trust-cache-watch-member-publish,06-task-process-worker-single-active-job-real-job-exec,07-task-ha-decide-pure-matrix-idempotency-tests</blocked_by>

<description>
**Goal:** Wire the HA runtime loop that reacts to typed watcher changes and periodic ticks, then dispatches actions.

**Scope:**
- Implement `src/ha/worker.rs` with `run`, `step_once`, and `dispatch_actions`.
- Use `tokio::select!` over `pg`, `dcs`, `process`, and `config` `changed()` plus interval tick.
- Ensure no wake-bus/event-enum side channels are introduced.

**Context from research:**
- Plan explicitly removes wake-reason buses in favor of direct watcher changes.

**Expected outcome:**
- HA loop advances from real system state changes and dispatches process/DCS actions safely.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] HA worker consumes `WorldSnapshot` from typed subscribers only.
- [x] `dispatch_actions` is robust to transient errors and produces typed action errors.
- [x] Integration tests assert `step_once` and full loop parity for the same inputs.
- [x] Run `make check`.
- [x] Run `make test`.
- [x] Run `make lint`.
- [x] Run `make test`.
- [x] Failures must generate `$add-bug` tasks with action traces.
</acceptance_criteria>

<execution_plan>
## Detailed Implementation Plan

1. Baseline and constraints
- [x] Confirm blockers `04`, `05`, `06`, and `07` remain in done/passing state before wiring HA runtime.
- [x] Capture a baseline build with `cargo check --all-targets` prior to edits so regressions are attributable to this task.
- [x] Keep runtime code free of `unwrap`/`expect`/`panic`; use typed error propagation through `WorkerError` + HA-specific dispatch errors.
- [x] Preserve the architecture constraint from task description: no wake-bus/event-enum side channel, only typed watcher subscriptions + interval tick.
- [x] Use actual module paths from this repo (`src/state/mod.rs`, `src/pginfo/state.rs`, `src/config/mod.rs`) when updating imports/tests; earlier draft referenced non-existent `src/state.rs` and `src/pg/state.rs`.

2. Expand HA worker context contract in `src/ha/state.rs`
- [x] Replace `HaWorkerCtx { _private: () }` with concrete runtime fields:
- [x] `poll_interval: Duration`
- [x] `state: HaState`
- [x] `publisher: StatePublisher<HaState>`
- [x] `config_subscriber: StateSubscriber<RuntimeConfig>`
- [x] `pg_subscriber: StateSubscriber<PgInfoState>`
- [x] `dcs_subscriber: StateSubscriber<DcsState>`
- [x] `process_subscriber: StateSubscriber<ProcessState>`
- [x] `process_inbox: tokio::sync::mpsc::UnboundedSender<ProcessJobRequest>`
- [x] `dcs_store: Box<dyn DcsStore>`
- [x] `scope: String` and `self_id: MemberId` for leader-lease paths
- [x] `process_defaults` bundle for required job specs (host, port, socket_dir, log_file, rewind source conninfo, shutdown mode), because `ProcessJobKind` variants are spec-carrying and cannot be constructed from action enum alone.
- [x] `now: Box<dyn FnMut() -> Result<UnixMillis, WorkerError> + Send>` for deterministic tests/job ids
- [x] Add a `contract_stub(...)` constructor so `worker_contract_tests.rs` can keep validating `ha::worker::step_once` callability with minimal setup.

3. Add HA dispatch error model in `src/ha/worker.rs` (or `src/ha/state.rs` if shared)
- [x] Define typed action dispatch errors with enough detail for bug reports and traces, e.g.:
- [x] `ActionDispatchError::ProcessSend { action, message }`
- [x] `ActionDispatchError::DcsWrite { action, path, message }`
- [x] `ActionDispatchError::DcsDelete { action, path, message }`
- [x] `ActionDispatchError::UnsupportedAction { action, reason }` (only if truly unavoidable)
- [x] Keep errors action-scoped so one failed action does not mask others in the same dispatch batch.

4. Fill DCS primitives needed by HA dispatch in `src/dcs/store.rs`
- [x] Extend `DcsStore` with `delete_path(&mut self, path: &str) -> Result<(), DcsStoreError>` so `ReleaseLeaderLease` has a first-class operation.
- [x] Implement helper(s) for leader lease writes/deletes using existing key conventions:
- [x] leader path: `/{scope}/leader`
- [x] value payload: serialized `LeaderRecord { member_id: self_id }`
- [x] Update `TestDcsStore` and any in-file fake stores (including worker tests) for the new trait method.
- [x] Keep backward compatibility for current DCS worker behavior and tests.

5. Implement `WorldSnapshot` materialization from typed subscribers only
- [x] Add helper in HA worker to read `latest()` from `config`, `pg`, `dcs`, and `process` subscribers and construct `WorldSnapshot`.
- [x] Ensure `step_once` uses this helper exclusively (no ad hoc reads outside typed subscribers).
- [x] Include current `HaState` plus `WorldSnapshot` as input to `decide(...)`.

6. Implement `dispatch_actions` in `src/ha/worker.rs` with best-effort semantics
- [x] Signature target: dispatches all actions, returns aggregated typed errors rather than failing fast on first error.
- [x] DCS action mapping:
- [x] `AcquireLeaderLease` -> write leader key for `self_id`.
- [x] `ReleaseLeaderLease` -> delete leader key.
- [x] Process action mapping via `process_inbox.send(ProcessJobRequest { id, kind })` with concrete spec builders:
- [x] `StartPostgres` -> `ProcessJobKind::StartPostgres`
- [x] `PromoteToPrimary` -> `ProcessJobKind::Promote`
- [x] `DemoteToReplica` -> `ProcessJobKind::Demote`
- [x] `StartRewind` -> `ProcessJobKind::PgRewind`
- [x] `RunBootstrap` -> `ProcessJobKind::Bootstrap`
- [x] `FenceNode` -> `ProcessJobKind::Fencing`
- [x] Build each spec from runtime config + `process_defaults`; do not leave placeholder empty paths/conninfo because process worker validates these as hard errors.
- [x] Explicitly define handling for `FollowLeader` and `SignalFailSafe` for current codebase capabilities (either concrete mapping or explicit no-op/typed unsupported), and cover this with tests.
- [x] Build deterministic `JobId` generation from `(tick, action index, action kind, now)` to avoid collisions in fast loops.

7. Implement `step_once` in `src/ha/worker.rs`
- [x] Gather latest world snapshot from typed subscribers.
- [x] Run `decide(DecideInput { current: ctx.state.clone(), world })`.
- [x] Run `dispatch_actions` on decide output actions.
- [x] Publish next HA state through `ctx.publisher` with current time.
- [x] On dispatch errors, keep behavior resilient:
- [x] continue attempting remaining actions
- [x] surface aggregated typed failure details in returned error and/or worker status update so transient issues are visible.
- [x] Ensure `step_once` remains directly callable/deterministic for integration tests.

8. Implement `run` with `tokio::select!` in `src/ha/worker.rs`
- [x] Create `tokio::time::interval(ctx.poll_interval)` for periodic ticks.
- [x] Use `tokio::select!` over:
- [x] `ctx.pg_subscriber.changed()`
- [x] `ctx.dcs_subscriber.changed()`
- [x] `ctx.process_subscriber.changed()`
- [x] `ctx.config_subscriber.changed()`
- [x] interval `tick()`
- [x] For every wake reason, call `step_once(&mut ctx).await?`.
- [x] Handle closed watch channels as typed `WorkerError` with explicit source (which subscriber closed).
- [x] Avoid introducing any synthetic wake reason enums/buses.

9. Update contract tests and HA unit coverage
- [x] Update `src/worker_contract_tests.rs` for new `HaWorkerCtx` constructor and dependencies (publisher/subscribers/inbox/dcs store/clock).
- [x] Add HA worker unit tests in `src/ha/worker.rs` covering:
- [x] `step_once` consumes typed subscribers and publishes HA state.
- [x] dispatch action mapping for each actionable variant (DCS and process).
- [x] dispatch best-effort behavior when one action fails and later actions still attempt.
- [x] typed error contents include action id/path/message.
- [x] `run` reacts to watcher changes and interval ticks via `tokio::select!`.

10. Add parity-style integration tests for step and loop behavior
- [x] Add/extend tests so the same initial world + updates produce equivalent observable HA state/action dispatch under:
- [x] direct `step_once` invocation
- [x] spawned `run` loop with real state channels and triggered `publish(...)` changes
- [x] Assert parity on phase transitions and action side effects (process inbox submissions, leader key writes/deletes).

11. Targeted local validation before full gates
- [x] Run focused tests first (`cargo test ha::worker`, `cargo test worker_contract_tests`, and any new module-specific tests).
- [x] Fix deterministic timing issues in run-loop tests with bounded waits and controlled clocks.
- [x] Confirm clippy cleanliness for new enums/traits and no dead code leaks.

12. Required full gate sequence (sequential and complete)
- [x] Run `make check`.
- [x] Run `make test`.
- [x] Run `make test`.
- [x] Run `make lint`.
- [x] If any gate fails, stop and create `$add-bug` task(s) with action traces and repro steps before continuing.

13. Acceptance and bookkeeping
- [x] Tick acceptance checkboxes only with direct evidence from tests and gate outputs.
- [x] Update task header tags only after all required gates pass.
- [x] Set `<passing>true</passing>` only at full completion.
- [x] Run `/bin/bash .ralph/task_switch.sh` once the task is truly complete.
- [x] Commit all files (including `.ralph` artifacts) with message:
- [x] `task finished 08-task-ha-worker-select-loop-and-action-dispatch: <summary + gate evidence + challenges>`
- [x] Append learnings/surprises to `AGENTS.md` after completion.
</execution_plan>

NOW EXECUTE
