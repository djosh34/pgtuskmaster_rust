## Plan: Collapse Managed Start Intent Onto StartPostgresSpec

### Why this reduction target

`src/process/planner.rs`, `src/process/cluster.rs`, `src/postgres_managed.rs`, and `src/postgres_managed_conf.rs` still restate the same Postgres start decision through too many intermediate types:

- `ProcessIntent::Start(PostgresStartIntent)` is the request boundary.
- `ClusterProcessPlan::StartManagedPostgres(ManagedPostgresStartIntent)` converts that into a second start enum only to carry replica conninfo and slot data.
- `process/cluster.rs` converts again into `StartPostgresSpec` and a separate `PostgresStartMode`.
- `postgres_managed_conf.rs` then matches `ManagedPostgresStartIntent` a second time to derive `hot_standby`, `primary_conninfo`, `primary_slot_name`, and recovery-signal behavior.

One `pg_ctl start` request is currently spread across four type identities plus duplicate match ladders.

### Current overlap already verified

- `src/process/planner.rs` converts `PostgresStartIntent::{Primary, DetachedStandby, Replica}` into `ManagedPostgresStartIntent::{primary, detached_standby, replica}` inside `ClusterProcessPlan::StartManagedPostgres`.
- `src/process/cluster.rs` immediately converts `ManagedPostgresStartIntent` again through `managed_start_mode()` into `PostgresStartMode`, materializes the managed config, and wraps the result in `ProcessExecutionKind::StartPostgres(StartPostgresSpec)`.
- `src/process/jobs.rs` defines `StartPostgresSpec` and `PostgresStartMode`, but `spec.mode` is only asserted in tests; `build_command()` does not need it to build `pg_ctl start`.
- `src/postgres_managed.rs` and `src/postgres_managed_conf.rs` still require `ManagedPostgresStartIntent` only to decide recovery signal and optional replica conninfo/slot rendering.
- `src/process/cluster.rs` tests and `src/process/planner.rs` tests currently prove the translation chain itself, not just the resulting start behavior and materialized files.

### Execution plan

1. Collapse the start pipeline onto the existing `StartPostgresSpec`.
   - Extend `StartPostgresSpec` to carry the start-only managed-config inputs still needed after leader resolution: replica source conninfo and optional slot name.
   - Remove `ManagedPostgresStartIntent`.
   - Remove `PostgresStartMode`.
   - Change `ClusterProcessPlan::StartManagedPostgres(...)` into a `StartPostgres(StartPostgresSpec)` variant so the planner produces the surviving execution-facing shape directly.

2. Let the planner build the full start spec.
   - In `src/process/planner.rs`, construct `StartPostgresSpec` directly for primary, detached-standby, and replica starts.
   - Fill deterministic execution fields there as well: `data_dir`, managed `config_file`, and `log_file` all already come from `RuntimeConfigV2`.
   - Resolve replica leader information once and store the resulting conninfo in the spec instead of re-encoding the same choice into a second enum.

3. Flatten cluster preparation around the single surviving start spec.
   - Delete the `managed_start_mode()` conversion and the `ManagedPostgresStartIntent` branch in `execution_request_from_plan()`.
   - Have `process/cluster.rs` materialize managed files from `StartPostgresSpec` directly before returning `ProcessExecutionKind::StartPostgres`.
   - Preserve the split between tracked request kind (`StartPrimary`, `StartDetachedStandby`, `StartReplica`) and spawned command kind (`StartPostgres`) without reintroducing another start enum.

4. Teach managed config rendering to consume the surviving boundary.
   - Update `materialize_managed_postgres_config()` and `ManagedPostgresConf`/render helpers to use the start variant from the tracked request kind together with the optional replica source fields on `StartPostgresSpec`.
   - Replace the current recovery-signal and `primary_conninfo`/`primary_slot_name` matches on `ManagedPostgresStartIntent` with one match on the surviving start data.
   - Delete the dead helper constructors `ManagedPostgresStartIntent::{primary, detached_standby, replica}` and any tests that only exercise them.

5. Rebuild tests around observable behavior.
   - Update planner and cluster tests to assert directly on `StartPostgresSpec` contents, materialized managed files, and tracked job kinds rather than intermediate enum conversions.
   - Update `postgres_managed.rs` and `postgres_managed_conf.rs` tests to build the surviving start spec/request data instead of `ManagedPostgresStartIntent`.
   - Remove assertions whose only purpose was proving `PostgresStartIntent -> ManagedPostgresStartIntent -> PostgresStartMode`.

6. Validate reduction and repo health.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the reduction improves beyond the current `+6044 -8507 diff: -2463` baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.
   - Run the validation gates sequentially, not in parallel.

### Guardrails

- Reuse `StartPostgresSpec`; do not introduce a new replacement start enum or another wrapper struct just to move the same fields around.
- Keep the tracked request kind separate from the spawned `StartPostgres` command kind so worker state and runtime logging semantics stay intact.
- If collapsing onto `StartPostgresSpec` starts forcing fake placeholder paths or impossible partially initialized specs in the planner, switch this plan back to `TO BE VERIFIED`.

NOW EXECUTE
