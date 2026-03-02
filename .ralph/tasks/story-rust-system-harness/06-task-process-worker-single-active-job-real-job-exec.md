---
## Task: Implement process worker single-active-job execution with real job tests <status>done</status> <passes>true</passes> <passing>true</passing> <priority>high</priority>

<blocked_by>03-task-worker-state-models-and-context-contracts</blocked_by>

<description>
**Goal:** Implement process worker to run exactly one long-running job at a time and publish deterministic outcomes.

**Scope:**
- Implement `src/process/jobs.rs`, `src/process/state.rs`, `src/process/worker.rs`, `src/process/mod.rs`.
- Implement `can_accept_job`, `start_job`, `tick_active_job`, `cancel_active_job`, `run`, and `step_once`.
- Add real execution tests for `Bootstrap`, `PgRewind`, `Promote`, `Demote`, `StartPostgres`, `StopPostgres`, `RestartPostgres`, and `Fencing`.

**Context from research:**
- Plan requires no queue/history, only single active job + latest outcome.

**Expected outcome:**
- Process worker executes HA-dispatched jobs safely and exposes accurate state for HA decisions.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] Worker rejects new jobs while one is active.
- [x] `JobOutcome` states are emitted for success/failure/timeout/cancelled.
- [x] Real job execution tests run against actual binaries and harness resources.
- [x] Run `make check`.
- [x] Run `make test`.
- [x] Run `make lint`.
- [x] Run `make test-bdd`.
- [x] Failures trigger `$add-bug` tasks with failing job kind and repro.
</acceptance_criteria>

<execution_plan>
## Detailed Implementation Plan

1. Baseline and contract guardrails
- [x] Confirm blocker task `03-task-worker-state-models-and-context-contracts` remains done/passing before changing process contracts.
- [x] Capture a baseline with `cargo check --all-targets` so regressions are attributable to this task.
- [x] Keep runtime code free of `unwrap`/`expect`/`panic` and propagate typed errors via `ProcessError`/`WorkerError`.
- [x] Preserve the design invariant from plan: single active job, no queue history, only latest outcome.

2. Expand process job contracts in `src/process/jobs.rs`
- [x] Replace placeholder job specs with typed fields required for real execution (paths, connection inputs, runtime options, timeout overrides where applicable).
- [x] Extend `ActiveJob` to carry job identity and immutable execution metadata needed by `ProcessState::Running`.
- [x] Extend `ProcessError` with concrete failure categories used by execution logic (spawn failure, early exit, timeout, cancel failure, unsupported input), while preserving compatibility with existing HA tests expecting operation-failure semantics.
- [x] Add helper methods for translating exit status/errors into deterministic `ProcessError` values.

3. Expand process state/context in `src/process/state.rs`
- [x] Keep `ProcessState` and `JobOutcome` enum shapes stable, but add helper accessors for active id and idle outcome updates.
- [x] Replace `ProcessWorkerCtx { _private: () }` with real worker context fields:
- [x] process config/binary paths and poll interval.
- [x] typed state publisher/subscriber handles used by worker and tests.
- [x] inbound job request receiver (or equivalent typed mailbox) and single active runtime slot.
- [x] deterministic clock hook for tests (`fn now() -> UnixMillis`) so timeout/outcome assertions are stable.
- [x] Update `worker_contract_tests.rs` and any constructor sites to match the new context shape.

4. Implement command construction and execution plumbing in `src/process/worker.rs`
- [x] Add command builders per `ProcessJobKind` using configured binaries and validated arguments.
- [x] Add an internal command-runner seam (runtime `tokio::process` implementation + test double) so unit tests can assert lifecycle and outcome mapping without spawning real binaries.
- [x] Implement `can_accept_job(state: &ProcessState) -> bool`.
- [x] Implement `start_job(ctx, job)` to transition idle -> running and spawn/process command ownership.
- [x] Implement `tick_active_job(ctx)` to poll active command completion, emit `Success`/`Failure`/`Timeout`, and transition back to idle with `last_outcome`.
- [x] Implement `cancel_active_job(ctx, reason)` to terminate active work and emit deterministic `Cancelled` outcome.
- [x] Keep `step_once` as deterministic orchestration: ingest at most one pending request, reject new work while running, tick active work, publish state.
- [x] Keep `run` loop as `step_once + sleep(ctx.poll_interval)` with error propagation.

5. Define process-job ingress contract without coupling to HA worker internals
- [x] Introduce a typed process job request envelope (`JobId` + `ProcessJobKind`) consumed directly by process worker context.
- [x] Keep HA action -> process job translation explicitly out of this task (owned by task 08) to avoid cross-task API churn.
- [x] Ensure worker semantics for concurrent requests are explicit:
- [x] Accept when idle.
- [x] Reject when running and preserve current active job.
- [x] Return/record a deterministic rejection error path that can be asserted in unit tests.
- [x] Avoid global queues/history; only current active + latest outcome remains visible.

6. Implement real job behavior per job kind
- [x] `Bootstrap`: run `initdb` against test harness data dir.
- [x] `PgRewind`: run `pg_rewind` with supplied source conninfo and target data dir.
- [x] `Promote`: run `pg_ctl promote`.
- [x] `Demote`: implement deterministic demotion flow (stop postgres with controlled mode and prepare replica-ready state according to current constraints).
- [x] `StartPostgres`: run `pg_ctl start` with explicit host/port/socket/log options.
- [x] `StopPostgres`: run `pg_ctl stop` with explicit mode.
- [x] `RestartPostgres`: run `pg_ctl restart` with explicit options.
- [x] `Fencing`: implement concrete fence operation (at minimum immediate stop/lease-safe barrier command) and map failures to typed outcomes.

7. Unit tests for single-active-job and outcomes
- [x] Add focused tests in `src/process/worker.rs` (or dedicated process test module) for:
- [x] `can_accept_job` behavior for idle vs running.
- [x] starting a job when idle transitions to running and publishes state.
- [x] rejecting a new request while active.
- [x] success/failure/timeout/cancelled outcome mapping.
- [x] `cancel_active_job` behavior on running and idle states.
- [x] Ensure all new tests avoid unwrap-style runtime shortcuts in non-test code.

8. Real execution integration tests with harness resources
- [x] Add integration-style tests that use `test_harness::namespace`, `ports`, and `pg16` resources plus real PG16 binaries when available.
- [x] For each required job kind (`Bootstrap`, `PgRewind`, `Promote`, `Demote`, `StartPostgres`, `StopPostgres`, `RestartPostgres`, `Fencing`), add a dedicated scenario that exercises the real binary invocation path.
- [x] Keep tests self-skipping when required binaries are absent, matching existing repository pattern.
- [x] Ensure cleanup is robust (namespace guard + process shutdown) to avoid cross-test contamination.

9. Compatibility and module export wiring
- [x] Update `src/process/mod.rs` exports only as needed for crate-internal consumers.
- [x] Keep visibility narrow (`pub(crate)`) and avoid unnecessary re-export fanout.
- [x] Re-run and adjust HA decide/unit tests if stricter `ProcessError` variants require updated assertions.

10. Targeted validation before full gates
- [x] Run focused process tests first (`cargo test process::` and any new integration target names) for fast feedback.
- [x] Fix flaky timing behavior by stabilizing timeout/cancel thresholds and deterministic polling intervals.
- [x] Confirm no clippy regressions for large enum variants or unused fields/imports.

11. Required full gate sequence (sequential)
- [x] Run `make check`.
- [x] Run `make test`.
- [x] Run `make test-bdd`.
- [x] Run `make lint`.
- [x] If any command fails, stop and create `$add-bug` task(s) with failing job kind, repro steps, and key logs before resuming.

12. Acceptance and task bookkeeping
- [x] Tick each acceptance checkbox only when directly evidenced by passing tests/commands.
- [x] Update task header tags to done/passing only after all four required `make` targets pass.
- [x] Set `<passing>true</passing>` only after final verification.

13. Completion protocol
- [x] Run `/bin/bash .ralph/task_switch.sh` after successful completion.
- [x] Commit all changes (including `.ralph` updates) with:
- [x] `task finished 06-task-process-worker-single-active-job-real-job-exec: <summary + evidence + challenges>`
- [x] Append new learnings/surprises to `AGENTS.md`.
- [x] Append progress diary entry via `.ralph/progress_append.sh`.

14. Skeptical verification amendments (applied in TO BE VERIFIED pass)
- [x] Removed scope coupling that made task 06 responsible for HA action dispatch concerns intended for task 08.
- [x] Added explicit command-runner test seam to keep unit tests deterministic while still keeping separate real-binary integration tests.
</execution_plan>

NOW EXECUTE
