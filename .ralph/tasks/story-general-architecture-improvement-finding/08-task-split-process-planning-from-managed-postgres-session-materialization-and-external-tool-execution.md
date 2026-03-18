## Task: Split Process Planning From Managed Postgres Session Materialization And External Tool Execution <status>completed</status> <passes>true</passes>

<priority>high</priority>

<description>
**Goal:** Refactor the process domain so the code that decides the desired managed PostgreSQL session/config for a loop is owned separately from the code that manages subprocesses and tool invocations. The higher-order goal is to deepen the `src/process/` module boundary so callers still submit a small `ProcessIntent`, but the process domain internally owns three distinct concerns with clear ADTs: planning, authoritative managed PostgreSQL session/config materialization, and external tool lowering/execution. This task exists because the earlier HA/process boundary cleanup was necessary but not sufficient: `src/process/worker.rs` still intermingles DCS/source resolution, `ManagedPostgresStartIntent` derivation, managed file materialization, and command construction in one worker-owned switch.

**What was explicitly requested and must be preserved:**
- "Could postgres config struct creation not be the same for each ha loop, instead of intermingled with process."
- "Also postgres management is a lot different than config setting and running smth like pgbasebackup."
- Do not treat this as an HA boundary task. Treat it as a deeper follow-up inside `src/process/`.
- Keep the caller-facing boundary small. The improvement should be mostly internal to the process domain.

**Problem statement from current research:**
- `src/ha/reconcile.rs` already emits a small `ProcessIntent` surface, which is good. The remaining ownership problem is inside the process domain, not HA.
- `src/process/worker.rs` `materialize_execution_request(...)` currently mixes:
  - process policy and DCS source-member resolution for replica/basebackup/rewind requests
  - derivation of `ManagedPostgresStartIntent` for primary, detached standby, and replica starts
  - managed PostgreSQL config/session materialization through `materialize_managed_postgres_config(...)`
  - construction of execution-layer `ProcessExecutionKind`
- `src/process/worker.rs` `build_command(...)` then mixes external tool lowering for `initdb`, `pg_ctl`, `pg_basebackup`, and `pg_rewind` in the same worker module that already owns request polling, active-job state, logging, timeouts, and subprocess handle management.
- `src/postgres_managed.rs` and `src/postgres_managed_conf.rs` already contain the authoritative managed PostgreSQL session/config logic, but their use is hidden inside the process worker's request-materialization path instead of behind a dedicated process-owned session boundary.
- `src/process/source.rs` already contains source-member validation and conninfo assembly logic, but it is still orchestrated directly from the worker switch rather than from a dedicated planner boundary.
- The result is that "what managed PostgreSQL session/config should exist" is not modeled as a first-class ADT. It is reconstructed ad hoc in the same path that decides whether to run `pg_ctl`, `pg_basebackup`, `pg_rewind`, or `initdb`.

**Concrete repo evidence from research:**
- [`src/ha/reconcile.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/reconcile.rs#L60)
  - HA already decides only high-level `ProcessIntent` values such as `Bootstrap`, `ProvisionReplica`, `Start`, `Promote`, and `Demote`.
- [`src/process/worker.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/process/worker.rs#L1285)
  - `materialize_execution_request(...)` is currently the mixed boundary that should be split.
- [`src/process/worker.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/process/worker.rs#L1379)
  - `primary_start_intent(...)` inspects managed recovery state inline in the worker path.
- [`src/process/worker.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/process/worker.rs#L1398)
  - `replica_start_intent(...)` performs DCS-backed source selection and standby-auth derivation inline in the worker path.
- [`src/process/worker.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/process/worker.rs#L1422)
  - `materialize_start_postgres(...)` calls managed config materialization during execution-request construction rather than behind a dedicated session/materialization boundary.
- [`src/process/worker.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/process/worker.rs#L1103)
  - `build_command(...)` owns lowering of all external tool invocations.
- [`src/process/source.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/process/source.rs#L24)
  - Replica/basebackup/rewind source validation and conninfo assembly already exist and should move under the new planning boundary.
- [`src/postgres_managed.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/postgres_managed.rs#L52)
  - Authoritative managed PostgreSQL runtime-file materialization already exists and should become its own owned stage instead of being driven directly from the worker switch.
- [`src/postgres_managed_conf.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/postgres_managed_conf.rs#L65)
  - `ManagedPostgresStartIntent` is already the right seed for a desired-session ADT and should be reused instead of creating another duplicated start vocabulary.

**Recommended interface direction from the design pass:**
- Keep a small concrete facade for callers, because the dominant runtime path should stay trivial for `ProcessWorkerCtx`.
- Internally split the process domain into three owned stages:
  1. pure planning from `ProcessIntent` + observed snapshot -> a typed cluster/process plan
  2. managed PostgreSQL session/config materialization from a desired session ADT -> prepared managed session artifacts
  3. external tool lowering from the plan + prepared session -> launchable subprocess request/command
- Do not expose a wide trait graph to HA or runtime. Keep ports/adapters private to the process domain if traits are needed at all.
- Reuse existing ADTs where they already fit: `ProcessIntent`, `ManagedPostgresStartIntent`, `BootstrapSpec`, `BaseBackupSpec`, `PgRewindSpec`, `PromoteSpec`, `DemoteSpec`, `StartPostgresSpec`, `ManagedPostgresConfig`.
- Introduce a first-class desired managed PostgreSQL session/config ADT so "config/session shape" is no longer implicit in execution-request creation.

**Chosen hybrid over other designs:**
- Keep the common-caller ergonomics of a concrete `ProcessCluster` facade with a small `prepare(...)` or `plan_then_prepare(...)` entry point.
- Internally adopt the sharper split from the ports/adapters-style design:
  - planner for DCS/source/policy translation
  - managed session materializer for authoritative PostgreSQL files and signals
  - external tool lowerer/executor for `pg_ctl`, `pg_basebackup`, `pg_rewind`, and `initdb`
- Do not adopt the fully general public trait-heavy interface as the caller-facing shape. That is too wide for the dominant path in this repo.

**Illustrative code sketch for the target shape:**
```rust
pub(crate) struct ProcessObservedSnapshot {
    pub(crate) runtime_config: RuntimeConfig,
    pub(crate) dcs: DcsView,
    pub(crate) managed_recovery_state: ManagedRecoverySignal,
}

pub(crate) struct ProcessCluster {
    planner: ProcessIntentPlanner,
    sessions: ManagedPostgresSessionMaterializer,
    tools: ExternalToolLowerer,
}

impl ProcessCluster {
    pub(crate) fn production_from_ctx(ctx: &ProcessWorkerCtx) -> Self;

    pub(crate) fn from_snapshot(
        identity: ProcessNodeIdentity,
        plan: ProcessRuntimePlan,
        observed: ProcessObservedSnapshot,
    ) -> Self;

    pub(crate) fn prepare(
        &self,
        request: ProcessIntentRequest,
    ) -> Result<PreparedProcessLaunch, ProcessError>;
}

pub(crate) enum ClusterProcessPlan {
    Bootstrap(BootstrapSpec),
    ProvisionReplica(ReplicaProvisionPlan),
    StartManagedPostgres(ManagedStartPlan),
    Promote(PromoteSpec),
    Demote(DemoteSpec),
}

pub(crate) struct ManagedStartPlan {
    pub(crate) launch: PostgresStartMode,
    pub(crate) desired_session: DesiredManagedPostgresSession,
}

pub(crate) enum DesiredManagedPostgresSession {
    Primary,
    DetachedStandby,
    Follow {
        source: ReplicaSource,
        standby_auth: ManagedStandbyAuth,
        primary_slot_name: Option<String>,
    },
}

pub(crate) struct PreparedProcessLaunch {
    pub(crate) id: JobId,
    pub(crate) action: PreparedProcessAction,
    pub(crate) command: ProcessCommandSpec,
}
```

**Important non-goals for this task:**
- Do not change HA reconciliation semantics in `src/ha/decide.rs` or `src/ha/reconcile.rs` unless a mechanical adaptation is required by the narrower process boundary.
- Do not redesign DCS ownership or worker runtime assembly outside the process-domain slice.
- Do not solve this by introducing a large new public abstraction layer that callers must understand.
- Do not duplicate `ManagedPostgresStartIntent` with another parallel "desired start" enum if the existing type can be reused or wrapped narrowly.
- Do not keep config/session planning implicit inside command-building logic.

**Scope:**
- Refactor the process-domain request materialization path in `src/process/worker.rs` so worker orchestration no longer owns the full plan/materialize/lower switch.
- Introduce a first-class process planning boundary that owns DCS trust/member resolution, source selection, and start-session derivation.
- Introduce or extract a dedicated managed PostgreSQL session/config materialization boundary that owns authoritative files, TLS copy, standby passfile, signal files, and `postgresql.auto.conf` quarantine.
- Refactor external tool lowering so command construction for `initdb`, `pg_ctl`, `pg_basebackup`, and `pg_rewind` is owned separately from planning and from managed session/config materialization.
- Rework tests so boundary tests target the planner, session materializer, and tool lowerer/facade instead of only narrow worker helpers.
- Reduce direct worker knowledge of `RuntimeConfig`, `DcsView`, `ManagedPostgresStartIntent`, and command-shape details to one snapshot/facade handoff.

**Context from research:**
- Current narrow tests already exist in multiple places:
  - [`src/postgres_managed.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/postgres_managed.rs#L646) onward tests authoritative managed runtime-file materialization in isolation.
  - [`src/postgres_managed_conf.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/postgres_managed_conf.rs#L472) onward tests config rendering rules in isolation.
  - [`src/process/worker.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/process/worker.rs#L1606) has focused worker/noop tests.
- Those tests prove the logic exists, but the boundary is still shallow because the worker coordinates too many concepts directly.
- The new tests should replace internal-shape assertions where boundary tests can exercise the deeper process cluster interface or the new planner/session/tool seams directly.
- The earlier completed task [`03-task-refactor-the-ha-process-boundary-around-a-dedicated-process-intent-adapter-and-remove-secret-bearing-process-defaults-from-ha.md`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/.ralph/tasks/story-general-architecture-improvement-finding/03-task-refactor-the-ha-process-boundary-around-a-dedicated-process-intent-adapter-and-remove-secret-bearing-process-defaults-from-ha.md) should be treated as prerequisite context, not as the place to add more work. This is a new follow-up focused on `src/process/`.

**Expected outcome:**
- `src/ha/reconcile.rs` continues to emit small `ProcessIntent` values and remains ignorant of process-internal execution details.
- The process worker becomes a thin orchestrator around one concrete process facade instead of directly mixing planning, managed session materialization, and external tool lowering.
- The desired managed PostgreSQL session/config becomes a first-class ADT rather than an implicit byproduct of `materialize_execution_request(...)`.
- Config/session shaping is clearly distinct from process/tool execution, matching the architectural concern raised in the PO request.
- Boundary tests can prove process behavior without needing to understand every internal helper or recreate the current mixed worker switch.

</description>

<acceptance_criteria>
- [x] Refactor [`src/process/worker.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/process/worker.rs) so request polling, active-job state, logging, timeout handling, and subprocess handle management remain in the worker, but plan derivation / managed session materialization / tool lowering are no longer intermingled in one worker-owned switch.
- [x] Introduce a process-owned planning boundary that consumes `ProcessIntent` plus a typed observed snapshot and owns DCS trust/member lookup, source selection, and start-session derivation.
- [x] Introduce a first-class desired managed PostgreSQL session/config ADT, reusing `ManagedPostgresStartIntent` where appropriate instead of duplicating the start vocabulary.
- [x] Move authoritative managed PostgreSQL runtime-file work behind a dedicated session/config materialization boundary, reusing and narrowing `src/postgres_managed.rs` and `src/postgres_managed_conf.rs` rather than leaving them as worker-called helpers.
- [x] Refactor command construction for `initdb`, `pg_ctl`, `pg_basebackup`, and `pg_rewind` behind a distinct external tool lowering boundary instead of mixing it with planning or managed-session derivation.
- [x] Keep the caller-facing process boundary small: runtime/worker callers should use a concrete facade, not a wide public trait graph.
- [x] planning start-primary, start-detached-standby, start-replica, basebackup, pg_rewind, promote, and demote requests
- [x] primary-start rejection when managed recovery state proves replica-managed state is still present
- [x] source-member validation and conninfo/auth derivation for basebackup vs pg_rewind
- [x] authoritative managed PostgreSQL session/config materialization independent from subprocess lowering
- [x] external tool lowering independent from DCS/planning logic
- [x] delete or simplify shallow/internal tests that become redundant once the new deeper boundary tests exist
- [x] Preserve or improve logging/observability so failures can still identify whether they happened in planning, session materialization, command lowering, spawn, or runtime process handling.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

### Execution plan
1. Introduce a typed observed snapshot for process planning so the worker reads `RuntimeConfig`, `DcsView`, and managed recovery facts once and hands them to the deeper process boundary.
2. Extract a planner that turns `ProcessIntent` into a first-class process plan, reusing existing spec types and `ManagedPostgresStartIntent` wherever possible.
3. Extract a managed PostgreSQL session/config materializer that owns authoritative runtime files and returns prepared managed-session artifacts needed for start flows.
4. Extract an external tool lowerer/executor that owns `ProcessCommandSpec` construction for `initdb`, `pg_ctl`, `pg_basebackup`, and `pg_rewind`.
5. Collapse the worker into a thin orchestration shell around the new process facade, while preserving existing active-job lifecycle and logging behavior.
6. Replace narrow internal tests with deeper boundary tests for planner/session/tool/facade behavior, keeping only the low-level tests that still prove distinct reusable logic.
7. Run the required validation gates in repo-preferred order:
   - `make check`
   - `make lint`
   - `make test`
   - `make test-long`
8. Only after all gates pass, update docs if this refactor changes any documented internal architecture surface, then complete task closeout.

### Constraints for execution
- Do not reintroduce HA ownership of process-internal config/session/tool details.
- Do not create a second duplicated start/session vocabulary if `ManagedPostgresStartIntent` can remain canonical.
- Prefer type-driven consolidation over new helper-function sprawl.
- Keep local-substitutable dependencies private to the process domain; do not spray traits through the crate without a real ownership reason.
- Do not run `cargo test`; use the required `make` targets, and use `cargo nextest` only for focused local iteration if absolutely needed before the final `make` gates.

NOW EXECUTE
