## Plan: Delete Process Execution Courier Layer

### Why this reduction target

`src/process/cluster.rs`, `src/process/planner.rs`, `src/process/jobs.rs`, `src/process/state.rs`, `src/process/worker.rs`, and `src/postgres_managed.rs` still carry a courier boundary that no longer earns its keep:

- `prepare_process_launch(...)` asks `ProcessIntentPlanner::plan(...)` for a `ProcessExecutionKind`, then immediately performs side effects from that same enum and separately builds the `ProcessCommandSpec` that is actually spawned.
- `ProcessExecutionRequest` stores the whole `ProcessExecutionKind`, but `src/process/worker.rs` only keeps it around to derive a timeout and preserve tracked job bookkeeping after the command has already been built.
- `StartPostgresSpec` ferries `data_dir`, `config_file`, `log_file`, `primary_conninfo`, and `primary_slot_name`, even though the path fields already live in `cfg.postgres.*` and `src/postgres_managed.rs` only reads the replica-source fields.

That is the same boundary smell as the previous slices: a planning layer computes the real launch facts, a request layer re-carries them, and the runtime only needs the finished command plus a timeout.

### Current overlap already verified

- The remaining production consumers of `ProcessExecutionKind` and the per-command spec structs are confined to `src/process/planner.rs`, `src/process/cluster.rs`, `src/process/worker.rs`, and `src/postgres_managed.rs`.
- `src/process/cluster.rs` still revalidates many cfg-derived path/source fields right before rendering shell arguments, even though those values were just chosen by the planner/source-resolution path.
- `src/postgres_managed.rs` only reads `StartPostgresSpec.primary_conninfo` and `StartPostgresSpec.primary_slot_name`; it does not need the rest of that struct once cfg-owned paths are read directly.
- `ProcessIntentPlanner` is only used by `src/process/cluster.rs`, so the planner/runtime handoff is internal and can be collapsed without widening public API.

### Execution plan

1. Collapse launch preparation onto intents and prepared launches.
   - Delete `ProcessExecutionKind` and the command-specific spec structs from `src/process/jobs.rs`.
   - Change `ProcessExecutionRequest` to hold only `id`, `tracked_job_kind`, and `timeout_ms`.
   - Make `prepare_process_launch(...)` match on `ProcessIntent` directly and return the finished `PreparedProcessLaunch`.

2. Delete the planner boundary.
   - Remove `ProcessIntentPlanner` and delete `src/process/planner.rs`.
   - Move source-member resolution helpers into `src/process/cluster.rs` as private launch-preparation helpers.
   - Preserve current error behavior for no-quorum DCS, self-targets, empty hosts, and non-primary sources.

3. Delete the managed-start courier.
   - Change `materialize_managed_postgres_config(...)`, `materialize_managed_standby_passfile(...)`, `render_managed_postgres_conf(...)`, and `reject_replica_source_fields(...)` to accept `Option<&PgConnInfo>` and `Option<&str>` instead of `&StartPostgresSpec`.
   - Build the start-postgres command directly from `cfg.postgres.data_dir`, `cfg.postgres.log_file`, and `managed_postgresql_conf_path(...)`.

4. Simplify worker timeout handling and tests.
   - Remove the `timeout_for_kind(...)` enum match in `src/process/worker.rs` and use `execution_request.timeout_ms` directly.
   - Rewrite process tests around observable outputs: built command args, tracked job kind, timeout, source-role behavior, and managed-file side effects.
   - Fold planner-only test coverage into `src/process/cluster.rs` so the deleted file leaves no gap.

5. Validate the reduction.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the reduction improves beyond the current `+7273 -10090 diff: -2817` baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.

### Guardrails

- Do not replace the deleted enums/specs with another renamed launch-plan wrapper stack.
- Reuse `ProcessIntent`, `ProcessJobKind`, and cfg-owned path state instead of inventing new DTOs.
- If direct intent matching inside `src/process/cluster.rs` forces a second courier layer or a large test-only wrapper, switch this plan back to `TO BE VERIFIED`.

NOW EXECUTE
