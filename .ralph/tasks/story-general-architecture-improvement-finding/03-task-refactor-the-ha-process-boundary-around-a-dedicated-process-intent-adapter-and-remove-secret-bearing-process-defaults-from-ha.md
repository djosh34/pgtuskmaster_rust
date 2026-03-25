## Task: Refactor The HA->process Boundary Around A Dedicated Process Intent Adapter And Remove Secret-Bearing Process Defaults From HA <status>completed</status> <passes>true</passes>

<priority>high</priority>

<description>
**Goal:** Refactor the HA/process boundary so HA owns reconciliation intent and process-policy decisions only, while the process domain owns process request materialization, source/auth assembly, and execution-specific command types. The higher-order goal is to stop HA from carrying process secrets, process startup details, and duplicated process vocabularies that should belong to one owner component. This task should reduce interface size, make impossible states unrepresentable, and remove a major ownership tangle between `src/ha/` and `src/process/`.

**Original general architectural request that this task must preserve:**
- "just like the dcs refactor task, i want a fully general improvement finding task"
- "make packages/mods more private"
- "reduce code interface between other components, make as small as possible interface"
- "find/checks/refactors radically internally to reduce code duplication. tries to simplify logic, de-spagthify, clean up old legacy logic/tests/shit"
- "untangle spagethi dependencies: just like dcs was controlled in many parts of the code, instead of a single worker. Find some other component that can be untangled, made almost fully private except very scoped/small interface, and thereby massively improving code quality, testability, reducing code in general (less code = better), cleaning up shit, making it more readable"

**Problem statement from current research:**
- HA currently owns both the high-level reconciliation plan and the low-level process request encoding. That means one domain is responsible for both deciding *what* should happen and encoding *how* the process worker must execute it.
- `src/ha/state.rs` currently stores a secret-bearing `ProcessDispatchDefaults` bag inside `HaWorkerCtx`. That bag contains `replicator_auth`, `rewinder_auth`, SSL CA material, socket/log paths, port, and other process-owned details. Runtime builds that bag in `src/runtime/node.rs` and injects it into HA, so HA becomes a carrier of process execution data rather than just process intent.
- `src/ha/process_dispatch.rs` currently constructs concrete process-layer request types such as `BootstrapSpec`, `BaseBackupSpec`, `PgRewindSpec`, `StartPostgresSpec`, `PromoteSpec`, and `DemoteSpec`, even though those types live under `src/process/`.
- `src/ha/source_conn.rs` currently assembles secret-bearing `ReplicatorSourceConn` and `RewinderSourceConn` values for process jobs. Secrets cross into HA only so HA can pass them right back to the process worker.
- HA also has a duplicated mirrored vocabulary for process state and planned commands in `src/ha/types.rs`, while the process worker has its own separate job/state model in `src/process/jobs.rs` and `src/process/state.rs`. The duplication is already lossy: process reports only `ActiveJobKind::StartPostgres`, while HA distinguishes `StartPrimary`, `StartDetachedStandby`, and `StartReplica`, and current mapping collapses that difference back down.
- `ReconcileAction` currently mixes process actions, DCS actions, publication updates, and required-role management in one enum. `src/ha/worker.rs` hand-routes variants, while `src/ha/process_dispatch.rs` must reject non-process variants as unsupported. That is a direct signal that the type boundary is wrong and impossible states are currently representable.
- `HaState` publishes `planned_commands: Vec<ReconcileAction>`, and API exposes `HaState`, so the mixed internal command encoding is not even private to HA internals today.

**Concrete repo evidence from research:**
- `src/ha/state.rs`
  - `HaWorkerCtx` stores `process_inbox: UnboundedSender<ProcessJobRequest>` and `process_defaults: ProcessDispatchDefaults`.
  - `ProcessDispatchDefaults` owns process credentials and connection details that HA should not need to own.
  - `HaWorkerCtx::contract_stub(...)` plus `ProcessDispatchDefaults::contract_stub()` show the public HA construction boundary is built around this leak.
- `src/runtime/node.rs`
  - `process_defaults_from_config(...)` constructs `ProcessDispatchDefaults` directly from `RuntimeConfig`.
  - `run_workers(...)` constructs a stub HA worker and then mutates `ha_ctx.process_defaults = process_defaults`.
- `src/ha/process_dispatch.rs`
  - `dispatch_process_action(...)` directly builds process job specs and sends `ProcessJobRequest` values.
  - `dispatch_process_action(...)` rejects non-process `ReconcileAction` variants as unsupported, proving the enum is broader than the process adapter contract.
- `src/ha/source_conn.rs`
  - `basebackup_source_from_member(...)` and `rewind_source_from_member(...)` build process-side connection structs using HA-owned defaults and cloned auth material.
- `src/ha/types.rs`
  - HA keeps its own process-oriented job vocabulary and conversion glue around process state.
  - `ReconcileAction` mixes multiple side-effect families into one type.
- `src/process/jobs.rs` and `src/process/state.rs`
  - Process already has its own authoritative job vocabulary and worker request types.
- `src/ha/worker.rs`
  - `execute_action(...)` routes some actions directly to DCS/local operations and others through process dispatch, which confirms the boundary is split awkwardly.
- `src/api/mod.rs` and any API DTOs/state exposure that surface `HaState`
  - These are the places to re-evaluate whether internal mixed command types should remain externally visible.

**Required architectural direction:**
- HA should produce a smaller typed intent surface that describes only the process work HA wants done, not full process execution specs with embedded secrets and process-owned details.
- The process domain should own translation from HA intent plus typed runtime/process context into concrete process requests/specs.
- HA should stop owning `RoleAuthConfig`, process connection defaults, and other secret-bearing process dispatch state.
- The final type shape should separate process actions from DCS/publication/role-management actions so invalid combinations are not representable.
- If HA still needs to publish a planned-action view, that view should be a deliberate read-only/public model rather than the same internal mixed command enum used for execution routing.

**Important non-goals for this task:**
- Do not change HA decision semantics in `src/ha/decide.rs` unless required by the new boundary shape.
- Do not redesign DCS command semantics or subprocess command execution behavior in `src/process/worker.rs`.
- Do not solve this by only renaming the current types while preserving the same ownership split.

**Scope:**
- Refactor the HA/process boundary across `src/ha/` and `src/process/` so HA no longer constructs raw `ProcessJobRequest` payloads itself.
- Remove secret-bearing process defaults and process-only connection/auth assembly from `HaWorkerCtx`.
- Rework `ReconcileAction` and related execution routing so process-directed work is represented separately from non-process side effects.
- Re-evaluate HA planned-command/state exposure and conversion glue so duplicated process vocabularies are reduced or removed.
- Update runtime wiring in `src/runtime/node.rs` to match the narrower HA and process boundaries.
- Update tests around HA action routing, process request translation, and any externally visible planned-command/read-model behavior that changes.

**Expected outcome:**
- HA owns reconciliation intent, not process request construction details.
- The process domain becomes the single owner of process request/spec materialization and source/auth assembly.
- HA no longer carries secret-bearing process defaults or broad process-specific config state.
- The boundary between HA and process is smaller, more private, and more typed.
- Duplicated or lossy process vocabularies across `src/ha/` and `src/process/` are substantially reduced.

</description>

<acceptance_criteria>
- [x] Refactor `src/ha/state.rs` so `HaWorkerCtx` no longer owns secret-bearing process dispatch defaults such as role auth material, SSL root certs, and other process-only connection details.
- [x] Refactor `src/runtime/node.rs` so runtime no longer constructs and injects a broad `ProcessDispatchDefaults` bag into HA.
- [x] Refactor `src/ha/process_dispatch.rs` so HA no longer materializes raw `ProcessJobRequest` values and process-owned spec structs directly.
- [x] Refactor `src/ha/source_conn.rs` so secret-bearing process source/auth assembly is no longer owned by HA modules.
- [x] Rework `src/ha/types.rs` and related routing so mixed side-effect actions are split into a narrower typed process-intent boundary plus non-process actions, and non-process variants are no longer representable inside the process dispatch adapter.
- [x] Re-evaluate duplicated process job/state vocabulary across `src/ha/types.rs`, `src/process/jobs.rs`, and `src/process/state.rs`; the final boundary must reduce lossy duplication and make ownership clearer.
- [x] Update `src/ha/worker.rs` and any related execution/routing code so the new HA/process boundary is enforced in real worker flow.
- [x] Update any affected public/API-facing HA state exposure, including `src/api/mod.rs` and related DTO/read-model code, if internal mixed execution types are currently leaking through `HaState`.
- [x] Add or update focused tests proving that HA decision behavior is preserved while process request translation now happens behind the narrower boundary.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

### Execution plan
1. Finish the `ReconcilePlan` ADT rollout so HA worker execution consumes the split `publication` / `coordination` / `local` / `process` plan shape directly, and remove the remaining legacy `ReconcileAction` call sites and test fixtures.
2. Complete the process-domain intent boundary by replacing remaining `ProcessJobRequest` / `ProcessJobKind` entry points with `ProcessIntentRequest` plus process-owned materialization to concrete execution requests inside `src/process/worker.rs`.
3. Wire the live runtime/process context through the process worker so source member resolution, managed config generation, and secret-bearing auth assembly are fully process-owned and no longer reconstructed in HA.
4. Finish the active-job and HA-world vocabulary cleanup so start modes remain explicit (`StartPrimary`, `StartDetachedStandby`, `StartReplica`) and HA failure/process-state reporting uses the process-owned authoritative kinds.
5. Update API/controller/test-harness shaping so `HaState.planned_actions` remains a deliberate read model instead of leaking execution enums, then refresh the focused HA/process routing tests around the new boundary.
6. After the design compiles and the routing/tests are updated, run the required validation gates in repo-preferred order:
   - `make check`
   - `make lint`
   - `make test`
   - `make test-long`
7. Only after all checks pass, update docs for any boundary/read-model changes using the `k2-docs-loop` skill, remove stale docs if needed, then complete task closeout (`<passes>true</passes>`, task switch, commit, push).

### Constraints for execution
- Do not reintroduce secret-bearing process defaults or source/auth assembly into `src/ha/`.
- Keep the plan/product-type split: HA should emit process intent plus non-process action families, and the process adapter must only accept `ProcessIntent`.
- If execution shows the current ADTs are still wrong, switch this task back out of execution, describe the type gap, and stop immediately.
- Do not run `cargo test`; use the required `make` targets, and use `cargo nextest` only for focused local iteration if absolutely needed before the final `make` gates.

NOW EXECUTE
