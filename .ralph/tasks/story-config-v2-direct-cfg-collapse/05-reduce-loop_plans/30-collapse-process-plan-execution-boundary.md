## Plan: Collapse Process Plan And Execution Boundary

### Why this reduction target

`src/process/cluster.rs`, `src/process/planner.rs`, `src/process/state.rs`, and `src/process/source.rs` still carry a courier boundary that no longer earns its keep:

- `ProcessIntentPlanner::plan()` builds a `ClusterProcessPlan`.
- `ClusterProcessPlan` has the same six variants and the same payload structs as `ProcessExecutionKind`.
- `execution_request_from_plan()` in `src/process/cluster.rs` immediately remaps each `ClusterProcessPlan` variant into the matching `ProcessExecutionKind` variant, only adding the side effects that already belong to launch preparation (`wipe_data_dir()` and managed config materialization).
- `src/process/source.rs` exists only so `planner.rs` can call `source_from_member()` and then convert its private error back into `ProcessError`.

This is the same over-abstracted pattern as the last slice: one owner computes the real execution payload, serializes it through an intermediate enum/module, and the next owner immediately reconstructs the same shape.

### Current overlap already verified

- `src/process/planner.rs` defines `ClusterProcessPlan::{Bootstrap, BaseBackup, PgRewind, StartPostgres, Promote, Demote}` using the same payload structs already used by `ProcessExecutionKind`.
- `src/process/cluster.rs` matches every `ClusterProcessPlan` arm just to clone the same payload into `ProcessExecutionKind`, with only start-config materialization and data-dir wiping as side effects.
- `src/process/state.rs` owns `ProcessExecutionKind`, but that enum is not process-state specific; it is a job-spec boundary shared by planner, launcher, and worker.
- `src/process/source.rs` is only referenced by `src/process/planner.rs`; no other production module owns or reuses that file-level boundary.

### Execution plan

1. Make one canonical process execution enum.
   - Remove `ClusterProcessPlan`.
   - Move `ProcessExecutionKind` beside the other process job/spec types in `src/process/jobs.rs`, where the payload structs already live.
   - Change `ProcessIntentPlanner::plan()` to return `ProcessExecutionKind` directly instead of a second wrapper enum.

2. Collapse planner-only source materialization into the planner owner.
   - Move `SourceMaterializationError` and `source_from_member()` into `src/process/planner.rs` as private helpers.
   - Delete `src/process/source.rs`.
   - Remove the `process::source` module declaration from `src/process/mod.rs`.
   - Keep the same validation behavior and error text, but map failures to `ProcessError` at the planner boundary without a separate module hop.

3. Flatten launch preparation around the surviving execution kind.
   - Replace `execution_request_from_plan()` with a helper that takes `ProcessExecutionKind` directly, applies the existing side effects, and builds `ProcessExecutionRequest`.
   - Keep `tracked_job_kind` on `ProcessExecutionRequest` so start-primary / start-replica / start-detached-standby tracking remains intact even though the executable payload is the shared `StartPostgres` variant.
   - Preserve `build_command()` and worker timeout behavior against the same canonical `ProcessExecutionKind`.

4. Rebuild tests around the surviving owner types.
   - Update planner tests in `src/process/planner.rs` to assert on `ProcessExecutionKind` instead of `ClusterProcessPlan`.
   - Move the `source.rs` unit coverage into `planner.rs` so the deleted file does not leave gaps around self-target, empty-host, non-primary, and role-specific TLS/auth behavior.
   - Update any cluster tests that pattern-match the intermediate plan enum so they assert on the final execution request kind directly.

5. Validate the reduction.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the reduction improves beyond the current `+6626 -9235 diff: -2609` baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.

### Guardrails

- Reuse the existing process spec structs; do not introduce a replacement “planned” DTO or another enum layer.
- Do not merge planner logic back into worker execution; keep intent resolution separate from command spawning, but remove the redundant handoff type between them.
- If removing `ClusterProcessPlan` forces awkward tracked-job bookkeeping or test-only wrappers that recreate the same abstraction under another name, switch this plan back to `TO BE VERIFIED`.

NOW EXECUTE
