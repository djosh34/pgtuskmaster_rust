---
## Task: Implement HA worker select loop and action dispatch wiring <status>not_started</status> <passes>false</passes> <priority>high</priority>

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
- [ ] HA worker consumes `WorldSnapshot` from typed subscribers only.
- [ ] `dispatch_actions` is robust to transient errors and produces typed action errors.
- [ ] Integration tests assert `step_once` and full loop parity for the same inputs.
- [ ] Run `make check`.
- [ ] Run `make test`.
- [ ] Run `make lint`.
- [ ] Run `make test-bdd`.
- [ ] Failures must generate `$add-bug` tasks with action traces.
</acceptance_criteria>

<execution_plan>
## Detailed Implementation Plan

1. Baseline and constraints
- [ ] Confirm blockers `04`, `05`, `06`, and `07` remain in done/passing state before wiring HA runtime.
- [ ] Capture a baseline build with `cargo check --all-targets` prior to edits so regressions are attributable to this task.
- [ ] Keep runtime code free of `unwrap`/`expect`/`panic`; use typed error propagation through `WorkerError` + HA-specific dispatch errors.
- [ ] Preserve the architecture constraint from task description: no wake-bus/event-enum side channel, only typed watcher subscriptions + interval tick.
- [ ] Use actual module paths from this repo (`src/state/mod.rs`, `src/pginfo/state.rs`, `src/config/mod.rs`) when updating imports/tests; earlier draft referenced non-existent `src/state.rs` and `src/pg/state.rs`.

2. Expand HA worker context contract in `src/ha/state.rs`
- [ ] Replace `HaWorkerCtx { _private: () }` with concrete runtime fields:
- [ ] `poll_interval: Duration`
- [ ] `state: HaState`
- [ ] `publisher: StatePublisher<HaState>`
- [ ] `config_subscriber: StateSubscriber<RuntimeConfig>`
- [ ] `pg_subscriber: StateSubscriber<PgInfoState>`
- [ ] `dcs_subscriber: StateSubscriber<DcsState>`
- [ ] `process_subscriber: StateSubscriber<ProcessState>`
- [ ] `process_inbox: tokio::sync::mpsc::UnboundedSender<ProcessJobRequest>`
- [ ] `dcs_store: Box<dyn DcsStore>`
- [ ] `scope: String` and `self_id: MemberId` for leader-lease paths
- [ ] `process_defaults` bundle for required job specs (host, port, socket_dir, log_file, rewind source conninfo, shutdown mode), because `ProcessJobKind` variants are spec-carrying and cannot be constructed from action enum alone.
- [ ] `now: Box<dyn FnMut() -> Result<UnixMillis, WorkerError> + Send>` for deterministic tests/job ids
- [ ] Add a `contract_stub(...)` constructor so `worker_contract_tests.rs` can keep validating `ha::worker::step_once` callability with minimal setup.

3. Add HA dispatch error model in `src/ha/worker.rs` (or `src/ha/state.rs` if shared)
- [ ] Define typed action dispatch errors with enough detail for bug reports and traces, e.g.:
- [ ] `ActionDispatchError::ProcessSend { action, message }`
- [ ] `ActionDispatchError::DcsWrite { action, path, message }`
- [ ] `ActionDispatchError::DcsDelete { action, path, message }`
- [ ] `ActionDispatchError::UnsupportedAction { action, reason }` (only if truly unavoidable)
- [ ] Keep errors action-scoped so one failed action does not mask others in the same dispatch batch.

4. Fill DCS primitives needed by HA dispatch in `src/dcs/store.rs`
- [ ] Extend `DcsStore` with `delete_path(&mut self, path: &str) -> Result<(), DcsStoreError>` so `ReleaseLeaderLease` has a first-class operation.
- [ ] Implement helper(s) for leader lease writes/deletes using existing key conventions:
- [ ] leader path: `/{scope}/leader`
- [ ] value payload: serialized `LeaderRecord { member_id: self_id }`
- [ ] Update `TestDcsStore` and any in-file fake stores (including worker tests) for the new trait method.
- [ ] Keep backward compatibility for current DCS worker behavior and tests.

5. Implement `WorldSnapshot` materialization from typed subscribers only
- [ ] Add helper in HA worker to read `latest()` from `config`, `pg`, `dcs`, and `process` subscribers and construct `WorldSnapshot`.
- [ ] Ensure `step_once` uses this helper exclusively (no ad hoc reads outside typed subscribers).
- [ ] Include current `HaState` plus `WorldSnapshot` as input to `decide(...)`.

6. Implement `dispatch_actions` in `src/ha/worker.rs` with best-effort semantics
- [ ] Signature target: dispatches all actions, returns aggregated typed errors rather than failing fast on first error.
- [ ] DCS action mapping:
- [ ] `AcquireLeaderLease` -> write leader key for `self_id`.
- [ ] `ReleaseLeaderLease` -> delete leader key.
- [ ] Process action mapping via `process_inbox.send(ProcessJobRequest { id, kind })` with concrete spec builders:
- [ ] `StartPostgres` -> `ProcessJobKind::StartPostgres`
- [ ] `PromoteToPrimary` -> `ProcessJobKind::Promote`
- [ ] `DemoteToReplica` -> `ProcessJobKind::Demote`
- [ ] `StartRewind` -> `ProcessJobKind::PgRewind`
- [ ] `RunBootstrap` -> `ProcessJobKind::Bootstrap`
- [ ] `FenceNode` -> `ProcessJobKind::Fencing`
- [ ] Build each spec from runtime config + `process_defaults`; do not leave placeholder empty paths/conninfo because process worker validates these as hard errors.
- [ ] Explicitly define handling for `FollowLeader` and `SignalFailSafe` for current codebase capabilities (either concrete mapping or explicit no-op/typed unsupported), and cover this with tests.
- [ ] Build deterministic `JobId` generation from `(tick, action index, action kind, now)` to avoid collisions in fast loops.

7. Implement `step_once` in `src/ha/worker.rs`
- [ ] Gather latest world snapshot from typed subscribers.
- [ ] Run `decide(DecideInput { current: ctx.state.clone(), world })`.
- [ ] Run `dispatch_actions` on decide output actions.
- [ ] Publish next HA state through `ctx.publisher` with current time.
- [ ] On dispatch errors, keep behavior resilient:
- [ ] continue attempting remaining actions
- [ ] surface aggregated typed failure details in returned error and/or worker status update so transient issues are visible.
- [ ] Ensure `step_once` remains directly callable/deterministic for integration tests.

8. Implement `run` with `tokio::select!` in `src/ha/worker.rs`
- [ ] Create `tokio::time::interval(ctx.poll_interval)` for periodic ticks.
- [ ] Use `tokio::select!` over:
- [ ] `ctx.pg_subscriber.changed()`
- [ ] `ctx.dcs_subscriber.changed()`
- [ ] `ctx.process_subscriber.changed()`
- [ ] `ctx.config_subscriber.changed()`
- [ ] interval `tick()`
- [ ] For every wake reason, call `step_once(&mut ctx).await?`.
- [ ] Handle closed watch channels as typed `WorkerError` with explicit source (which subscriber closed).
- [ ] Avoid introducing any synthetic wake reason enums/buses.

9. Update contract tests and HA unit coverage
- [ ] Update `src/worker_contract_tests.rs` for new `HaWorkerCtx` constructor and dependencies (publisher/subscribers/inbox/dcs store/clock).
- [ ] Add HA worker unit tests in `src/ha/worker.rs` covering:
- [ ] `step_once` consumes typed subscribers and publishes HA state.
- [ ] dispatch action mapping for each actionable variant (DCS and process).
- [ ] dispatch best-effort behavior when one action fails and later actions still attempt.
- [ ] typed error contents include action id/path/message.
- [ ] `run` reacts to watcher changes and interval ticks via `tokio::select!`.

10. Add parity-style integration tests for step and loop behavior
- [ ] Add/extend tests so the same initial world + updates produce equivalent observable HA state/action dispatch under:
- [ ] direct `step_once` invocation
- [ ] spawned `run` loop with real state channels and triggered `publish(...)` changes
- [ ] Assert parity on phase transitions and action side effects (process inbox submissions, leader key writes/deletes).

11. Targeted local validation before full gates
- [ ] Run focused tests first (`cargo test ha::worker`, `cargo test worker_contract_tests`, and any new module-specific tests).
- [ ] Fix deterministic timing issues in run-loop tests with bounded waits and controlled clocks.
- [ ] Confirm clippy cleanliness for new enums/traits and no dead code leaks.

12. Required full gate sequence (sequential and complete)
- [ ] Run `make check`.
- [ ] Run `make test`.
- [ ] Run `make test-bdd`.
- [ ] Run `make lint`.
- [ ] If any gate fails, stop and create `$add-bug` task(s) with action traces and repro steps before continuing.

13. Acceptance and bookkeeping
- [ ] Tick acceptance checkboxes only with direct evidence from tests and gate outputs.
- [ ] Update task header tags only after all required gates pass.
- [ ] Set `<passing>true</passing>` only at full completion.
- [ ] Run `/bin/bash .ralph/task_switch.sh` once the task is truly complete.
- [ ] Commit all files (including `.ralph` artifacts) with message:
- [ ] `task finished 08-task-ha-worker-select-loop-and-action-dispatch: <summary + gate evidence + challenges>`
- [ ] Append learnings/surprises to `AGENTS.md` after completion.
</execution_plan>

NOW EXECUTE
