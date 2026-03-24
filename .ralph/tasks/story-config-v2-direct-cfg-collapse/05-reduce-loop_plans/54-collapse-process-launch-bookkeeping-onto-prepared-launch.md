## Plan: Collapse Process Launch Bookkeeping Onto Prepared Launch

### Why this is the next reduction target

The remaining process launch path still carries the same launch facts through a courier layer that no longer earns its keep:

- `src/process/cluster.rs` builds `PreparedProcessLaunch { request: ProcessExecutionRequest, command }`, but `ProcessExecutionRequest` only repeats `id`, `tracked_job_kind`, and `timeout_ms` for the same launch.
- `src/process/worker.rs` immediately unpacks `prepared_launch.request`, recomputes `deadline_at`, copies `tracked_job_kind` into `ActiveJob`, and stores `request`, `job_kind`, and `tracked_job_kind` again in `ActiveRuntime`.
- `src/process/state.rs` therefore holds both `ActiveRuntime.request` and separate `job_kind` / `tracked_job_kind` fields even though those are already present inside the prepared launch plus command.
- The tracked-vs-command distinction is real for start intents, but the extra wrapper is not. `ProcessIntent::Start(PostgresStartIntent::Replica { .. })` still tracks `StartReplica` while spawning a `StartPostgres` command, and that distinction can live directly on one canonical prepared launch record.

This is a good next slice because it reuses the surviving `PreparedProcessLaunch` type instead of inventing another action DTO, and it deletes bookkeeping from three of the largest remaining process files:

- `src/process/cluster.rs` (1180 lines)
- `src/process/worker.rs` (1100 lines)
- `src/process/state.rs` (201 lines)

### Current overlap already verified

- `src/process/cluster.rs` currently constructs `PreparedProcessLaunch` by nesting `ProcessExecutionRequest` inside it, even though the enclosing struct is already the one worker-facing launch boundary.
- `src/process/worker.rs` at the launch boundary immediately performs this sequence:
  - `let execution_request = prepared_launch.request;`
  - `let deadline_at = now + execution_request.timeout_ms;`
  - `let tracked_job_kind = execution_request.tracked_job_kind;`
  - `let job_kind = command.job_kind;`
  - store all of that again in `ActiveRuntime`
- `src/process/state.rs` defines:
  - `ProcessExecutionRequest`
  - `ActiveRuntime { request, deadline_at, handle, job_kind, tracked_job_kind }`
  which is two layers of bookkeeping for one spawned process.
- `src/process/cluster.rs` tests still assert through the nested courier (`prepared.request.tracked_job_kind`, `prepared.request.timeout_ms`) rather than through the actual prepared launch owner.

### Execution plan

1. Make `PreparedProcessLaunch` the single canonical launch record.
   - In `src/process/cluster.rs`, replace `PreparedProcessLaunch { request: ProcessExecutionRequest, command }` with fields directly on the struct:
     - `id`
     - `tracked_job_kind`
     - `timeout_ms`
     - `command`
   - Delete `ProcessExecutionRequest` from `src/process/state.rs`.
   - Keep `command.job_kind` as the spawned subprocess/logging kind and `tracked_job_kind` as the outward job-state kind; do not merge those semantics.

2. Collapse runtime bookkeeping onto the prepared launch.
   - In `src/process/state.rs`, replace `ActiveRuntime.request`, `job_kind`, and `tracked_job_kind` with one `launch: PreparedProcessLaunch` plus the existing `deadline_at` and `handle`.
   - Update `src/process/worker.rs` to use `runtime.launch` directly for:
     - deadline calculation from `launch.timeout_ms`
     - active-job construction from `launch.id` and `launch.tracked_job_kind`
     - log emission from `launch.command.job_kind`
     - success/failure/timeout outcomes from `launch.tracked_job_kind`
   - If a tiny helper on `PreparedProcessLaunch` deletes repeated field access without adding abstraction weight, use it; otherwise keep direct field reads.

3. Rebuild cluster and worker tests around the surviving owner.
   - Update `src/process/cluster.rs` assertions from `prepared.request.*` to `prepared.*`.
   - Update any `src/process/worker.rs` tests that inspect runtime state or outcomes so they assert against the flattened launch record instead of the deleted courier.
   - Prefer rewriting existing assertions over adding new fixture helpers.

4. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and require improvement beyond the current validated baseline `+8017 -11156 diff: -3139`.
   - Run `make check`.
   - Run `make lint`.
   - Run `make test`.
   - Run `make test-long`.

### Guardrails

- Reuse `PreparedProcessLaunch`; do not introduce a replacement `LaunchRequest`, `PreparedExecution`, or similar wrapper.
- Do not collapse `tracked_job_kind` onto `command.job_kind`; the split is still required for start-intent state accounting.
- Do not push timeout policy back into `worker.rs`; keep timeout selection in launch preparation and only store the chosen `timeout_ms` on the prepared launch.
- If deleting `ProcessExecutionRequest` forces a second wrapper type in `worker.rs` or test-only scaffolding that recreates the same boundary, switch this plan back to `TO BE VERIFIED`.

### Expected yield

- Delete one short-lived struct (`ProcessExecutionRequest`) outright.
- Shrink `ActiveRuntime` by removing duplicated launch metadata.
- Remove the unpack-and-repack launch bookkeeping in `src/process/worker.rs`.
- Flatten the process-launch test surface so assertions follow the real owner instead of a nested courier.

NOW EXECUTE
