---
## Bug: Pginfo standby polling test fails during primary configure with db error <status>done</status> <passes>true</passes>

<description>
`make test` failed in `pginfo::worker::tests::step_once_maps_replica_when_polling_standby` with a runtime panic while preparing the primary postgres fixture.

Repro:
- `make test`
- Failing test:
- `pginfo::worker::tests::step_once_maps_replica_when_polling_standby`

Observed trace excerpt:
- `configure primary failed: db error`

This appears in the test setup path for standby polling, before the HA/worker assertions complete. Please explore and research the codebase first, then implement a robust fix with deterministic setup/teardown and clear error propagation for fixture configuration.

Reality check from execution at 2026-03-02T23:11:59Z:
- Current `src/pginfo/worker.rs` standby test path has no `configure primary` SQL step and no matching error string.
- The reported failure text appears stale relative to current code and could not be reproduced in this run.
</description>

<acceptance_criteria>
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] BDD features pass (covered by `make test`).
</acceptance_criteria>

## Execution Plan (Verified)

### 0) Scope reconciliation and deterministic repro
- [x] Create a task-specific evidence directory under `.ralph/evidence/pginfo-standby-polling-test-configure-primary-db-error/` and capture all command outputs there.
- [x] Reproduce from the same top-level gate first: run `make test` once and capture the full log under the task-specific evidence path.
- [x] Re-run the exact target test directly:
  - `cargo test pginfo::worker::tests::step_once_maps_replica_when_polling_standby -- --nocapture`
- [x] Run a stress repeat loop (>=30 iterations) of the target test to classify behavior as deterministic vs flaky.
- [x] If current failure text differs from the task description (for example `connect to primary failed: db error` vs `configure primary failed: db error`), record the exact observed string and stack location in this task file before coding.

### 1) Root-cause deep dive (skeptical)
- [x] Inspect `src/pginfo/worker.rs` standby fixture setup end-to-end and explicitly verify whether a "configure primary" SQL step still exists:
  - primary spawn/readiness timing
  - primary SQL/configuration path (if present)
  - `pg_basebackup` invocation and error surfaces
  - replica spawn/readiness and first poll timing
- [x] Inspect `src/test_harness/pg16.rs` startup semantics and identify whether port-open readiness can race against SQL readiness.
- [x] Correlate failure location with log evidence (postgres stdout/stderr logs under namespace paths) to determine whether root cause is:
  - startup/connectivity race,
  - fixture configuration ordering,
  - replication warmup timing,
  - or stale task narrative already fixed by prior changes.

### 2) Fix strategy (choose strictest valid path)
- [x] If failure reproduces in current code: not applicable in this run (failure did not reproduce).
  - implement deterministic fixture hardening in `src/pginfo/worker.rs` test module:
    - keep bounded readiness retries before first primary SQL use,
    - avoid brittle one-shot SQL assumptions during startup,
    - use explicit `Result`-based error propagation with detailed context messages,
    - preserve deterministic cleanup on both success and failure paths.
- [x] If failure does **not** reproduce and prior code already fixed the issue:
  - document evidence proving closure (gate logs + repeat-loop stability),
  - add/adjust focused regression assertion(s) only if there is still an uncovered race window,
  - update bug description text in this file to reflect current reality (stale report vs active failure), with timestamped evidence references.
- [x] Do not introduce skips/optional test behavior; real-binary execution remains mandatory.

### 3) Verification gates and evidence capture
- [x] Run targeted tests for the touched area first:
  - `cargo test pginfo::worker::tests::step_once_maps_replica_when_polling_standby -- --nocapture`
  - `cargo test pginfo::worker::tests::step_once_transitions_unreachable_to_primary_and_tracks_wal_and_slots -- --nocapture`
- [x] Re-run flaky-prone target loop (>=10 passes) to verify stability after fix.
- [x] Run required repository gates sequentially:
  - [x] `make check`
  - [x] `make test`
  - [x] `make test`
  - [x] `make lint`
- [x] For `make test` and `make lint`, persist logs and grep for `congratulations` / `evaluation failed` per task rule.

### 4) Task closeout
- [x] Update this task file with:
  - root cause summary,
  - exact code changes,
  - evidence file paths and final gate results.
- [x] Mark acceptance criteria checkboxes only after evidence is present.
- [x] Set status/passes tags when all gates are green.
- [x] Run `/bin/bash .ralph/task_switch.sh`.
- [x] Commit all changes including `.ralph` artifacts with required message format.
- [x] Append any new cross-task learning to `AGENTS.md`.

## Execution Notes

### Root cause summary
- Reported failure text (`configure primary failed: db error`) is stale against the current codebase.
- Current standby test setup in `src/pginfo/worker.rs` does not include the old fragile primary configuration step that could emit that message.
- In this run the standby test was stable: one direct run + 30-iteration stress run + two full `make test` passes without failure.

### Exact code changes
- No production or test Rust source changes were required.
- Task execution/closeout updates are contained to Ralph task/evidence bookkeeping files.

### Evidence paths and results
- Evidence directory: `.ralph/evidence/pginfo-standby-polling-test-configure-primary-db-error/`
- Repro/full test logs:
  - `make-test.repro.log` (pass)
  - `make-test.log` (pass)
  - `make-check.log` (pass)
  - `make-test.log` (pass)
  - `make-lint.log` (pass)
- Targeted logs:
  - `cargo-test-step_once_maps_replica_when_polling_standby.log` (pass)
  - `cargo-test-step_once_transitions_unreachable_to_primary_and_tracks_wal_and_slots.log` (pass)
  - `repeat30-step_once_maps_replica_when_polling_standby.log` (30/30 pass)
- Required grep artifacts:
  - `make-test-congratulations.grep` (0 matches)
  - `make-test-evaluation-failed.grep` (0 matches)
  - `make-lint-congratulations.grep` (0 matches)
  - `make-lint-evaluation-failed.grep` (0 matches)

NOW EXECUTE
