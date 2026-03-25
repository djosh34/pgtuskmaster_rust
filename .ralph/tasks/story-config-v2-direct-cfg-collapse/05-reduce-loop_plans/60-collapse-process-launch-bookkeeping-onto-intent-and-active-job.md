## Plan: Collapse Process Launch Bookkeeping Onto Intent And Active Job

### Why this is the next reduction target

The current process-launch path still re-encodes the same job metadata in too many adjacent owners:

- `src/process/jobs.rs:19` already maps `ProcessIntent` to the tracked `ProcessJobKind`.
- `src/process/cluster.rs:48` copies that identity into `PreparedProcessLaunch { id, tracked_job_kind, timeout_ms, command }`.
- `src/process/worker.rs:480` immediately explodes that struct back into a derived deadline, a cloned command, and a second `ActiveJob`.
- `src/process/state.rs:63` stores `ActiveRuntime { launch, deadline_at, handle }` while `ProcessState::Running` stores another `ActiveJob` with the same id, kind, and deadline.

That is a boundary problem, not domain complexity. `cluster` should prepare process commands and materialize start config; it should not courier request identity, timeout policy, and runtime-state metadata into `worker`. The canonical owners already exist:

- `ProcessIntent` for job identity and timeout policy,
- `ActiveJob` for runtime state,
- `ProcessCommandSpec` for the executable command.

This slice should flatten the pass-through launch bookkeeping instead of introducing yet another wrapper.

### Current overlap already verified

- `src/process/jobs.rs:20` and `src/process/cluster.rs:192` derive the tracked job kind from the same `ProcessIntent`.
- `src/process/cluster.rs:429` computes timeout policy from `ProcessJobKind`, but `src/process/worker.rs:480` only needs the resulting deadline for `ActiveJob`, so `PreparedProcessLaunch.timeout_ms` is just transit metadata.
- `src/process/worker.rs:510` builds `ActiveJob` from the same `prepared_launch.id`, `prepared_launch.tracked_job_kind`, and `deadline_at` that `src/process/state.rs:63` stores again inside `ActiveRuntime`.
- `src/process/jobs.rs:113` stores `ProcessCommandSpec.job_kind`, but `src/process/worker.rs:481` also carries the tracked job kind separately because start intents report `StartPrimary`/`StartReplica` outcomes while the spawned command is still `StartPostgres`.

### Execution plan

1. Move process-launch policy onto the existing intent types.
   - Extend `ProcessIntent` with the policy methods that are currently smeared across `cluster` and `worker`.
   - Keep one method for tracked outcome kind and one method for command-log kind so start intents can still report `StartPrimary`/`StartReplica` outcomes while subprocess logs stay `StartPostgres`.
   - Move timeout selection next to those methods so `cluster` no longer owns `default_timeout_ms(...)`.

2. Shrink or delete `PreparedProcessLaunch` so `cluster` only returns the prepared command.
   - Remove request id, tracked job kind, and timeout fields from the launch handoff.
   - Keep config materialization and command construction in `src/process/cluster.rs`; that is the real ownership boundary.
   - If the remaining wrapper is only `ProcessCommandSpec`, delete `PreparedProcessLaunch` entirely.

3. Collapse runtime bookkeeping onto `ActiveJob`.
   - Rebuild `src/process/state.rs` so `ActiveRuntime` owns the process handle plus only the metadata that is not already in `ActiveJob`.
   - Remove the duplicate standalone `deadline_at` field from `ActiveRuntime`; `ActiveJob.deadline_at` is already canonical.
   - In `src/process/worker.rs`, construct `ActiveJob` directly from `request.id`, `request.intent`, and `now`, then store it once for both published state and active runtime control.

4. Delete the pass-through job-kind plumbing from `ProcessCommandSpec` if it is no longer the smallest owner.
   - Prefer deriving the command-log kind from the active request/runtime metadata instead of carrying it inside the command builder result.
   - Keep command builders focused on executable details: program, args, env, capture policy.
   - If removing `ProcessCommandSpec.job_kind` makes start-job logging noisier or larger, stop and keep only the reductions from steps 1 to 3.

5. Update process tests around the new boundary.
   - Rewrite `src/process/cluster.rs` tests so they assert the prepared command and intent-owned policy rather than duplicated tracked-vs-command fields on a wrapper DTO.
   - Update `src/process/worker.rs` tests to assert the direct `ActiveJob` construction and preserved `StartPostgres` logging semantics for start intents.

6. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and require the diff to stay net-negative.
   - Run `make check`.
   - Run `make lint`.
   - Run `make test`.
   - Run `make test-long`.

### Guardrails

- Do not invent a new launch-metadata struct just to replace `PreparedProcessLaunch` one-for-one.
- Reuse `ProcessIntent` and `ActiveJob`; those are already the canonical domain owners.
- Preserve the distinction between tracked outcome kind and spawned-command log kind for start intents.
- Keep `postgres_managed` start-config materialization driven by the existing start intent semantics; do not widen it to a more generic taxonomy.
- If removing `ProcessCommandSpec.job_kind` forces bigger adapter code than it deletes, keep that field for this slice and still remove the other duplicated bookkeeping.

### Expected yield

- Delete the pass-through launch DTO or shrink it to a trivial shell.
- Delete duplicated timeout and deadline bookkeeping between `cluster`, `worker`, and `state`.
- Reduce the tracked-vs-command job-kind plumbing to intent-owned policy instead of scattering it across three structs.
- Simplify the process test surface by asserting one owner per concern.

NOW EXECUTE
