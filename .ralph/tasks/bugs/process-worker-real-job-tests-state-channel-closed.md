---
## Bug: Process worker real job tests fail with state channel closed <status>done</status> <passes>true</passes>

<description>
`make test` failed while running real process worker job tests. Multiple tests panic because process state publish fails with `state channel is closed`.

Repro:
- `make test`
- Failing tests:
- `process::worker::tests::real_demote_job_executes_binary_path`
- `process::worker::tests::real_fencing_job_executes_binary_path`
- `process::worker::tests::real_promote_job_executes_binary_path`
- `process::worker::tests::real_restart_job_executes_binary_path`
- `process::worker::tests::real_start_and_stop_jobs_execute_binary_paths`

Observed trace excerpts:
- `demote job failed: process publish failed: state channel is closed`
- `fencing job failed: process publish failed: state channel is closed`
- `promote job failed: process publish failed: state channel is closed`
- `restart job failed: process publish failed: state channel is closed`
- `stop job failed: process publish failed: state channel is closed`

Please explore and research the codebase first, then implement a fix. Focus on subscriber lifetime and test harness ownership so real job tests keep required watch subscribers alive through all publish calls.
</description>

<acceptance_criteria>
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] BDD features pass (covered by `make test`).
</acceptance_criteria>

## Implementation Plan (Draft)

### 0) Reproduce and pin failure shape
- [x] Run `make test` once and capture exact failing tests + first failure stack traces into a new evidence folder.
- [x] Re-run the five reported process worker real-job tests individually with `--nocapture` (current run: all five passed; no immediate isolated repro).
- [x] Verify whether failures are deterministic by repeating at least one representative test (`real_start_and_stop_jobs_execute_binary_paths`) in a short loop.
- [x] If isolated reruns pass, treat this as potentially order/threading-sensitive and re-run representative commands with `-- --test-threads=1` plus a bounded loop to surface scheduling-dependent drops.

### 1) Root cause confirmation (subscriber lifetime)
- [x] Audit `src/process/worker.rs` real-test helpers (`real_ctx`, `bootstrap_and_start`, `wait_for_outcome`, `submit_job_and_wait`) to map where the only `StateSubscriber<ProcessState>` is created and how long it is guaranteed to live.
- [x] Validate watch-channel semantics in `src/state/watch_state.rs`: `StatePublisher::publish` fails when all receivers are dropped.
- [x] Confirm whether `_sub` bindings provide only implicit lifetime and can be dropped before all publish calls in async flows; require an explicit ownership path tied to helper/fixture methods instead of underscore-only bindings.

### 2) Fix design and implementation
- [x] Refactor real-process test setup to enforce subscriber lifetime across the full job lifecycle:
- [x] Replace tuple-based setup return with a fixture struct that owns `ProcessWorkerCtx`, job sender, `StateSubscriber<ProcessState>`, and `NamespaceGuard` together.
- [x] Move job submission/wait helpers onto the fixture (`impl` methods) so they always run while the subscriber owner is alive.
- [x] Keep at least one explicit read from subscriber snapshots during waits/assertions and fail with contextual messages if state publication stops progressing.
- [x] Update affected real-job tests to use the fixture methods and remove fragile unused-binding lifetime assumptions.
- [x] Keep changes scoped to tests/harness unless runtime behavior needs adjustment for correctness.

### 3) Add regression coverage
- [x] Add/adjust test assertions so each real-job test proves publish-to-idle state transitions complete without channel-close errors.
- [x] Add a focused unit/integration test around helper lifecycle showing that dropping all subscribers reproduces `ChannelClosed`, while fixture-owned subscriber avoids it.
- [x] Ensure all newly touched tests use proper error propagation in touched code paths (no new unwrap/expect/panic additions).

### 4) Validate and close
- [x] Run targeted tests for the five failing real-job cases first.
- [x] Run required full gates sequentially:
- [x] `make check`
- [x] `make test`
- [x] `make test`
- [x] `make lint`
- [x] Capture gate outputs under a task-specific evidence directory and record pass/fail markers used by Ralph (`congratulations` / `evaluation failed` where applicable).
- [x] Update this bug file with final root cause and fix summary.
- [x] Mark checkboxes and tags only after all gates pass.
- [x] Run `/bin/bash .ralph/task_switch.sh`.
- [x] Commit all changes (including `.ralph` artifacts) with message:
- [x] `task finished process-worker-real-job-tests-state-channel-closed: <summary with test evidence and lifecycle fix details>`

## Result Summary

- Root cause: Real process-worker tests relied on underscore subscriber bindings (`_sub`) plus tuple-based helpers, which made subscriber ownership implicit and brittle for async lifecycle reasoning. `StatePublisher::publish` fails by design when all receivers are dropped.
- Fix: Introduced `RealProcessFixture` in `src/process/worker.rs` test module to own `ProcessWorkerCtx`, job sender, `StateSubscriber<ProcessState>`, and `NamespaceGuard` together; moved submit/wait logic onto fixture methods and added explicit subscriber snapshot reads during wait loops.
- Regression coverage: Added `start_job_returns_channel_closed_when_all_subscribers_are_dropped` to prove channel-close behavior when no subscriber remains.
- Evidence/logs: `.ralph/evidence/process-worker-real-job-tests-state-channel-closed-20260302/` (`make-check.log`, `make-test.log`, `make-lint.log`, `gate-status.txt`).
