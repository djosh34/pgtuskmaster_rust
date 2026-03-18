# Verbose Extra Context: Process Management Refactor

This context file exists to update `docs/src/explanation/process-management.md` after the process-domain refactor completed in March 2026.

The existing explanation page is stale in one important way:

- It still says the worker turns `ProcessIntentRequest` into `ProcessExecutionRequest` inside `materialize_execution_request(...)`.
- It still says the worker then lowers commands in `build_command(...)`.
- Those statements are no longer true.

The current architecture after the refactor is:

1. HA still emits the same small `ProcessIntent` surface.
2. `process_dispatch` still turns that into `ProcessIntentRequest`.
3. `src/process/worker.rs` remains responsible for:
   - inbox polling
   - busy rejection
   - start-postgres noop preflight
   - active-job state transitions
   - timeout handling
   - subprocess output draining
   - subprocess spawn lifecycle
   - logging and state publication
4. The worker no longer owns the internal switch that mixes planning, managed PostgreSQL config/session materialization, and external command lowering.
5. The worker now constructs a `ProcessCluster` facade and calls `ProcessCluster::prepare(...)`.

The new private process-domain modules are:

- `src/process/cluster.rs`
- `src/process/planner.rs`
- `src/process/session.rs`
- `src/process/tools.rs`

The intended explanation should describe those modules and their responsibilities accurately.

## Exact role of each new module

### `src/process/cluster.rs`

`ProcessCluster` is the concrete internal facade used by the worker.

It owns:

- the local process identity
- the stable `ProcessRuntimePlan`
- a typed `ProcessObservedSnapshot`
- a `ProcessIntentPlanner`
- a `ManagedPostgresSessionMaterializer`
- an `ExternalToolLowerer`

`ProcessCluster::production_from_ctx(...)` reads observed state once from the worker context and creates a typed snapshot that includes:

- latest `RuntimeConfig`
- latest `DcsView`
- inspected `ManagedRecoverySignal`

That snapshot is represented by `ProcessObservedSnapshot` in `src/process/state.rs`.

`ProcessCluster::prepare(...)` runs the internal process-domain pipeline:

1. planner: turn `ProcessIntent` into a first-class `ClusterProcessPlan`
2. session materializer: materialize authoritative managed PostgreSQL artifacts for start flows
3. tool lowerer: turn the plan plus prepared session into a `ProcessExecutionRequest` and `ProcessCommandSpec`

`PreparedProcessLaunch` now carries:

- `request: ProcessExecutionRequest`
- `command: ProcessCommandSpec`

`ProcessPreparationError` keeps stage-specific attribution:

- `Planning`
- `SessionMaterialization`
- `ToolLowering`

The worker logs those stage failures with stage-specific cause text, so observability still distinguishes planning/session/tool-lowering failures from spawn/runtime failures.

### `src/process/planner.rs`

This module owns intent planning.

Important ADTs introduced here:

- `ClusterProcessPlan`
- `ManagedStartPlan`
- `DesiredManagedPostgresSession`
- `ReplicaFollowPlan`

`DesiredManagedPostgresSession` is the new first-class desired managed PostgreSQL session/config ADT.

Current variants:

- `Primary`
- `DetachedStandby`
- `Follow(Box<ReplicaFollowPlan>)`

The `Follow` variant is boxed because clippy rejected the large enum variant shape otherwise.

The planner is now where the process domain owns:

- DCS trust/member lookup
- source-member validation
- basebackup source selection
- pg_rewind source selection
- primary-start rejection when managed recovery state is still present
- derivation of the desired managed PostgreSQL session for replica starts

The planner does not write files and does not spawn commands.

For replica-following starts, the planner reuses the existing source ADT:

- `MandatoryRoleSourceConn`

That means the plan explicitly carries:

- conninfo
- auth
- source role

instead of hiding those details in worker-local helper functions.

### `src/process/session.rs`

This module owns authoritative managed PostgreSQL runtime-file materialization for start flows.

It reuses:

- `ManagedPostgresStartIntent`
- `materialize_managed_postgres_config(...)`
- `managed_standby_auth_from_role_auth(...)`

The new boundary is:

- planner decides the desired session shape with `DesiredManagedPostgresSession`
- session materializer converts that shape into canonical `ManagedPostgresStartIntent`
- session materializer writes authoritative runtime artifacts and returns `PreparedManagedPostgresSession`

`PreparedManagedPostgresSession` currently wraps the produced `ManagedPostgresConfig`.

Important behavioral detail:

- `ProcessRuntimePlan::ensure_start_paths()` is now called from the session materializer for start flows before managed files are written.

Non-start plans return `None` from the session materializer.

### `src/process/tools.rs`

This module owns external tool lowering.

It now contains:

- lowering from `ClusterProcessPlan` plus optional prepared session into `ProcessExecutionRequest`
- command construction from `ProcessExecutionKind` into `ProcessCommandSpec`
- destructive data-dir wiping for bootstrap/basebackup preparation
- helper mappings for active job kind and execution job kind

This means `worker.rs` no longer contains the large `build_command(...)` match.

The external tool lowerer is also where bootstrap/basebackup destructive preparation now happens. That keeps planning pure and moves the destructive preparation closer to external tool execution.

### `src/process/state.rs`

This module gained `ProcessObservedSnapshot`:

- `runtime_config: RuntimeConfig`
- `dcs: DcsView`
- `managed_recovery_state: ManagedRecoverySignal`

The explanation page should say clearly that the worker hands a typed observed snapshot to the deeper process boundary instead of letting the worker-owned switch read runtime config, DCS, and managed recovery state ad hoc during execution-request construction.

## What remains unchanged externally

These facts should remain in the doc:

- HA still emits `ProcessIntent`
- the caller-facing process boundary remains small
- the worker still handles admission, preflight, lifecycle, timeout, output drain, spawn, and publication
- start-postgres preflight/noop behavior still exists in the worker
- subprocess logging still flows through `ProcessLogEvent` / `SubprocessLogEvent`

## What should be removed or rewritten from the existing explanation

Please remove or rewrite any statements that claim:

- `materialize_execution_request(...)` is the current mixed worker-owned boundary
- `build_command(...)` is still inside `src/process/worker.rs`
- the worker itself directly performs source resolution, primary/replica start-intent derivation, managed config materialization, and command lowering in one switch

Those descriptions are now obsolete.

## New tests added by the refactor

The refactor added deeper boundary tests:

- `process::planner::tests::planner_maps_process_intents_to_expected_plan_variants`
- `process::planner::tests::planner_rejects_primary_start_with_existing_managed_replica_state`
- `process::planner::tests::planner_uses_distinct_source_roles_for_basebackup_and_rewind`
- `process::session::tests::materialize_follow_session_writes_managed_files_without_tool_lowering`
- `process::session::tests::materialize_skips_non_start_plans`
- `process::tools::tests::lower_execution_request_for_basebackup_wipes_existing_data_dir_contents`
- `process::tools::tests::build_command_for_start_postgres_uses_prepared_session_paths`
- `process::cluster::tests::prepare_replica_start_runs_through_planner_session_and_tool_layers`

Those tests are relevant evidence that the new boundary is planner/session/tool/facade oriented rather than only worker-helper oriented.

## Validation results from this task

All required task gates passed after this refactor:

- `make check`
- `make test`
- `make lint`
- `make test-long`

If the doc mentions validation evidence, keep it factual and concise.
