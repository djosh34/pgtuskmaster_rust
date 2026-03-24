## Plan: Collapse Process Job Kind Taxonomy Onto ProcessJobKind

### Why this reduction target

`src/process/jobs.rs`, `src/process/state.rs`, and `src/process/worker.rs` still restate the same process classification table through parallel enums and match ladders:

- `ActiveJobKind` and `ProcessJobKind` encode the same bootstrap/basebackup/pg_rewind/promote/demote/start split, with `ProcessJobKind` only adding the broad `StartPostgres` command bucket.
- `ProcessIntent`, `PostgresStartIntent`, `PostgresStartMode`, and `ProcessExecutionKind` each convert back into one or both enums through separate `match` trees.
- `worker.rs` logs request-time failures with one enum, tracks active/running state with another enum, and converts again on exit and timeout.

That is one job-kind taxonomy scattered across too many type layers.

### Current overlap already verified

- `src/process/jobs.rs` defines `ProcessIntent::active_job_kind`, `ProcessIntent::process_job_kind`, `PostgresStartIntent::active_job_kind`, `PostgresStartIntent::process_job_kind`, `PostgresStartMode::active_job_kind`, plus the parallel `ActiveJobKind` and `ProcessJobKind` enums.
- `src/process/state.rs` repeats the same table in `ProcessExecutionKind::{active_job_kind, process_job_kind}` and stores `ActiveJobKind` in both `ActiveJob` and `JobOutcome`.
- `src/process/worker.rs` bounces between `request.intent.process_job_kind()`, `request.intent.active_job_kind()`, and `execution_request.kind.active_job_kind()` as one job moves from inbox to running state to final outcome.
- `src/ha/reconcile.rs`, `src/ha/worker.rs`, and their tests compare concrete process categories only; they do not need a second type identity for the same categories.
- `src/logging/core/runtime.rs` already consumes `ProcessJobKind`, so it is the better surviving enum boundary.

### Execution plan

1. Collapse process state and outcomes onto the existing `ProcessJobKind`.
   - Delete `ActiveJobKind`.
   - Change `ActiveJob.kind`, `JobOutcome.{Success,Failure,Timeout}.job_kind`, `ProcessState::{active_job, waiting_for_pg_observation, basebackup_completed_awaiting_pg_start}`, and HA callers to use `ProcessJobKind`.
   - Keep the concrete start variants (`StartPrimary`, `StartDetachedStandby`, `StartReplica`) for state/outcome comparisons.

2. Make `ProcessJobKind` the only mapping target.
   - Replace the duplicated `*_active_job_kind` / `*_process_job_kind` helpers with `job_kind()` helpers that return `ProcessJobKind`.
   - Keep `ProcessExecutionKind::job_kind()` for the spawned-command/logging view, returning `StartPostgres` for the `pg_ctl start` command if current tests still depend on that broader bucket.
   - Delete dead helpers that become redundant after the collapse, including `ProcessIntent::label()` if it remains unused.

3. Flatten worker transitions around the single surviving enum.
   - Update `src/process/worker.rs` to pass `ProcessJobKind` through busy rejection, preflight/preparation failures, active state, spawn failures, timeout handling, and exit outcomes.
   - Preserve the stage distinction between request-time start variants and the spawned `StartPostgres` command bucket without reintroducing a second enum.

4. Rebuild dependent tests on the collapsed boundary.
   - Update affected assertions in `src/process/cluster.rs`, `src/process/worker.rs`, `src/ha/reconcile.rs`, `src/ha/worker.rs`, and logging/runtime tests to assert against `ProcessJobKind`.
   - Remove assertions that only prove enum translation rather than observable behavior.

5. Validate reduction and repo health.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the reduction improves beyond the current `+5965 -8385 diff: -2420` baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.
   - Run the validation gates sequentially, not in parallel.

### Guardrails

- Reuse `ProcessJobKind`; do not introduce a new replacement enum.
- Preserve the distinction between request-time start variants and the spawned `StartPostgres` command logging unless the surviving tests show the broader bucket can be deleted too.
- If collapsing onto one enum starts forcing stage booleans or string parsing into the state machine, switch this plan back to `TO BE VERIFIED`.

NOW EXECUTE
