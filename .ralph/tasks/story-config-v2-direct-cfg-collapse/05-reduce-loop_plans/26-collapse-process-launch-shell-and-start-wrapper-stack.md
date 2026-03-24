## Plan: Collapse Process Launch Shell And Start Wrapper Stack

### Why this reduction target

`src/process/cluster.rs`, `src/process/session.rs`, and `src/process/tools.rs` currently split one process-launch workflow across thin zero-state wrappers and start-only DTOs that mostly rename existing state:

- `ProcessCluster` stores `cfg`, an observed snapshot, and three stateless helpers, but `prepare(...)` only runs planner -> session materialization -> lowering -> command build.
- `ManagedStartPlan`, `DesiredManagedPostgresSession`, and `ReplicaFollowPlan` re-encode the same primary/detached-standby/replica start distinction that `ManagedPostgresStartIntent` already owns in `src/postgres_managed_conf.rs`.
- `PreparedManagedPostgresSession` is a one-field wrapper over `ManagedPostgresConfig`.
- `ExternalToolLowerer::lower_execution_request(...)` ignores `_observed` and mostly clones `ClusterProcessPlan` into `ProcessExecutionKind`, then immediately peels `PreparedManagedPostgresSession.config` back apart to build `StartPostgresSpec`.

That is one launch boundary spread across too many process-local types.

### Current overlap already verified

- `src/process/planner.rs` builds `DesiredManagedPostgresSession::{Primary, DetachedStandby, Follow(Box<ReplicaFollowPlan>)}` even though `ManagedPostgresStartIntent::{Primary, DetachedStandby, Replica { primary_conninfo, primary_slot_name }}` already carries the same start shape.
- `src/process/session.rs` converts those start-only wrappers back into `ManagedPostgresStartIntent`, runs `ensure_start_paths(cfg)`, materializes managed config files, and rewraps the result in `PreparedManagedPostgresSession`.
- `src/process/tools.rs` immediately unwraps `PreparedManagedPostgresSession.config` to build `StartPostgresSpec`, while every non-start branch just clones an existing plan spec into `ProcessExecutionKind`.
- `src/process/worker.rs` only consumes the prepared launch request/command pair and does not use `ProcessCluster` itself as a reusable domain boundary.

### Execution plan

1. Collapse the start-wrapper stack onto the existing managed-start type.
   - Change `ClusterProcessPlan::StartManagedPostgres(...)` to carry `ManagedPostgresStartIntent` directly.
   - Delete `ManagedStartPlan`, `DesiredManagedPostgresSession`, and `ReplicaFollowPlan`.
   - Keep source-member resolution and current start semantics exactly the same.

2. Collapse session materialization onto the real artifact.
   - Replace `PreparedManagedPostgresSession` with `ManagedPostgresConfig` directly, or inline the start-materialization branch if that deletes more code.
   - Keep `ensure_start_paths(...)` and `materialize_managed_postgres_config(...)` as the only start-materialization boundaries.
   - If `src/process/session.rs` becomes a tiny pass-through helper file, merge the remaining helper into the owning module and delete the file.

3. Delete the pass-through cluster/lowering shell.
   - Replace `ProcessCluster` and `ExternalToolLowerer` with one preparation path that owns observed-snapshot capture, planning, optional managed-config materialization, `ProcessExecutionRequest` construction, and command building.
   - Keep `ProcessExecutionKind` as the executable boundary because worker timeout/logging/spawn code already depends on it.
   - Remove the unused `_observed` argument and any stage-label helpers that only exist to stringify the deleted shell layers.

4. Rebuild worker and unit tests on the collapsed boundary.
   - Update `src/process/worker.rs` to call the new preparation helper directly.
   - Rewrite the affected unit tests in `cluster.rs`, `session.rs`, and `tools.rs` to assert on the surviving types and outputs rather than the deleted wrappers.
   - Delete emptied modules if they no longer own a real boundary after the refactor.

5. Validate reduction and repo health.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the reduction improves beyond the current `+5293 -7554 diff: -2261` baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.
   - Run the validation gates sequentially, not in parallel.

### Guardrails

- Reuse `ManagedPostgresStartIntent` and `ManagedPostgresConfig`; do not introduce a new process-local start enum or wrapper.
- Keep planning pure: config files should still be materialized only during launch preparation, not inside DCS/source resolution helpers.
- Preserve current error wording where tests already pin it; only retouch messages when the deleted shell labels are the only moving part.
- If collapsing `ProcessCluster` starts forcing a giant god-function in `src/process/worker.rs`, switch this plan back to `TO BE VERIFIED`.

NOW EXECUTE
