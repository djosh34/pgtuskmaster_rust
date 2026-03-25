## Plan: Collapse Process Start Job-Kind Shadow Taxonomy

### Why this is the next reduction target

The process module is still carrying two parallel job-kind stories for the same start operation.

- `src/process/jobs.rs:21-40` already derives the semantic job kind from `ProcessIntent`, but then re-derives a second `command_job_kind()` that exists only to flatten every start intent back into `StartPostgres`.
- `src/process/worker.rs:480-518` stores the semantic kind in `ActiveJob.kind`, but separately stores the shadow command kind in `ActiveRuntime.command_job_kind`, so the worker has to keep both notions of the same job alive while the child runs.
- `src/process/cluster.rs:284-301` hardcodes `ProcessCommandSpec.job_kind` to `StartPostgres` for every start command, even though the request already knows whether it is `StartPrimary`, `StartDetachedStandby`, or `StartReplica`.
- `src/process/cluster.rs:859-935` and `src/logging/core/runtime.rs:868-889` spend real test code proving that tracked start kinds and command start kinds intentionally diverge.

That is a boundary mistake. The semantic start kind already exists at the intent boundary and is what HA state cares about (`src/process/state.rs:142-160`), so the process runtime should use that single kind end-to-end instead of creating a second generic start label just for subprocess logging.

### Current overlap already verified

- `src/process/jobs.rs:36-40` recreates a second mapping source with `command_job_kind()`, while `src/process/jobs.rs:73-79` already maps start intents onto their canonical `ProcessJobKind`.
- `src/process/jobs.rs:108-135` keeps `ProcessJobKind::StartPostgres` alongside `StartPrimary`, `StartDetachedStandby`, and `StartReplica`, even though only the latter three carry business meaning in state and HA reconciliation.
- `src/process/worker.rs:487-639` uses the shadow command kind only for spawn/log/output plumbing, while outcomes still persist the semantic kind from `ActiveJob.kind`.
- `src/process/state.rs:63-66` adds `ActiveRuntime.command_job_kind` only because the spawned child logs a different kind than the state machine tracks.

### Execution plan

1. Remove the shadow start command kind and keep one canonical process job kind source.
   - Delete `ProcessIntent::command_job_kind()`.
   - Delete `ProcessJobKind::StartPostgres`.
   - Make `ProcessIntent::job_kind()` and `PostgresStartIntent::job_kind()` the only start-kind mapping path.

2. Carry the semantic start kind through command preparation and runtime execution.
   - Change `prepare_process_launch(...)` and `build_start_postgres_command(...)` so start commands use the request's semantic job kind (`StartPrimary`, `StartDetachedStandby`, or `StartReplica`) instead of a synthetic `StartPostgres`.
   - Remove `ActiveRuntime.command_job_kind` and rely on the active job's kind when emitting spawn, started, timeout, exit, poll-failure, and subprocess-output events.
   - Keep `ProcessCommandSpec.job_kind` only if it still buys a real invariant after the collapse; otherwise remove that field too instead of recreating the same courier under a new name.

3. Update logs and tests around the surviving taxonomy.
   - Rewrite cluster and worker tests that currently assert on separate tracked-vs-command kinds so they assert one canonical kind instead.
   - Update logging tests such as `src/logging/core/runtime.rs:868-889` to expect the concrete start kind on subprocess records.
   - Delete any helper assertions that exist only to prove the old split taxonomy.

4. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and require the diff to remain net-negative.
   - Run `make check`.
   - Run `make lint`.
   - Run `make test`.
   - Run `make test-long`.

### Guardrails

- Preserve HA behavior that depends on distinguishing `StartPrimary`, `StartDetachedStandby`, and `StartReplica`; that semantic split is the one that should survive.
- Keep the generic preflight event names (`StartPostgresAlreadyRunning`, `StartPostgresPreflightFailed`) unless renaming them is clearly net-reductive in the same slice.
- Do not introduce a replacement wrapper enum for "tracked kind vs command kind". The point of this slice is to delete that split, not rename it.

### Expected yield

- Delete `ProcessIntent::command_job_kind()` and the `ProcessJobKind::StartPostgres` variant.
- Delete `ActiveRuntime.command_job_kind` and the duplicate log/output plumbing it forces through `src/process/worker.rs`.
- Shorten tests that currently spend code proving a distinction the runtime no longer needs.

NOW EXECUTE
