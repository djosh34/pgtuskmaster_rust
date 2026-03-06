---
## Task: Untangle HA worker into facts, plan, and apply layers <status>done</status> <passes>true</passes>

<description>
**Goal:** Restructure HA runtime code so the worker clearly separates fact collection, pure decision selection, effect lowering, and effect application without forcing the design into object-heavy “executor” patterns.

**Scope:**
- Edit `src/ha/worker.rs`, `src/ha/mod.rs`, and any new helper modules needed to separate plan application by concern.
- Keep the worker as a thin orchestrator while moving low-level DCS path handling, process request assembly, filesystem mutations, and event payload construction into clearer domain helpers.
- Make the worker responsible for logging the chosen `HaDecision`, lowering it into executable effects, and then applying the lowered plan.
- Preserve the existing DCS worker and pginfo worker responsibilities while moving stray logic back to the layer where it belongs.

**Context from research:**
- We agreed the main need is to untangle the architecture, not necessarily to introduce heavyweight executor objects.
- `src/ha/worker.rs` currently mixes:
  - state snapshot collection
  - pure decision invocation
  - DCS path formatting and writes
  - process job request construction
  - filesystem mutation helpers
  - event/log attribute map construction
- We also discussed that runtime pollers already exist for DCS and pg state ingestion; what is missing is a clean separation so HA applies a plan instead of carrying random low-level concerns.

**Expected outcome:**
- HA worker code reads like:
  - collect facts
  - choose decision
  - lower decision
  - apply plan
  - publish state
- Non-HA concerns are pushed behind small typed helpers grouped by domain instead of being interleaved in one large function/file.
- The codebase becomes easier to continue migrating toward a functional style because the imperative edge is isolated.

**Story test policy:**
- This task is not complete until `make check`, `make test`, `make test-long`, and `make lint` all pass.
- If long HA coverage fails during this task, fix the real issue here rather than deferring it.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] Modify `src/ha/worker.rs`, `src/ha/mod.rs`, and any new helper modules introduced for plan-application untangling.
- [x] Keep `step_once` focused on orchestration rather than low-level implementation details.
- [x] Make `step_once` consume the high-level `HaDecision`, log it, call `HaDecision::lower()`, and apply the lowered effect plan without re-embedding planning logic in the worker.
- [x] Move DCS path knowledge and raw DCS write/delete mechanics out of the main HA worker control flow.
- [x] Move process request construction out of the main HA worker control flow into typed helpers or modules.
- [x] Move filesystem mutation helpers out of the main HA worker control flow into the correct domain layer.
- [x] Keep existing DCS/pginfo polling responsibilities intact and move any half-misplaced logic to the correct module rather than duplicating polling concepts inside HA tests/runtime code.
- [x] Update worker tests to reflect the new facts/plan/apply split and verify the new helper boundaries.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make test-long` — passes cleanly
- [x] `make lint` — passes cleanly
</acceptance_criteria>

<execution_plan>
## Detailed Execution Plan (Reviewed Draft 2, 2026-03-06)

1. Reconfirm the actual gap before editing
- The core facts/decision/lower split already exists in `src/ha/{decision,decide,lower}.rs`; this task is now about the apply boundary and worker readability, not about redesigning the decision machine again.
- `src/ha/worker.rs` is still carrying four different responsibilities in one file:
  - orchestration (`run`, `step_once`)
  - effect-plan application sequencing
  - low-level DCS path/render/write/delete mechanics
  - process request assembly, filesystem mutation, and structured event payload construction
- Execution must preserve the current behavior and deterministic bucket order while moving those concerns behind smaller typed helpers.
- The task file still says to skip `make test-long`, but the higher-level workflow for this run requires `make check`, `make test`, `make test-long`, and `make lint` before completion. Treat the skip text as stale story guidance and correct the task file/docs during execution instead of following it.

2. Target architecture after the refactor
- `src/ha/worker.rs` should read as a thin pipeline:
  - collect world snapshot
  - decide
  - lower
  - log selected decision and plan
  - apply lowered plan
  - publish next state
  - emit phase/role transition events
- `src/ha/worker.rs` should keep only orchestration and final state publication responsibilities.
- Introduce small helper modules rather than object-heavy executors:
  - `src/ha/apply.rs`: own `apply_effect_plan(...)`, bucket sequencing, typed dispatch error aggregation, and the small DCS coordination helpers that are tightly coupled to effect application
  - `src/ha/process_dispatch.rs`: own process job request construction, managed-config materialization, and filesystem prep like data-dir wiping
  - `src/ha/events.rs`: own HA event attribute construction plus decision/plan/action/lease event helpers so the worker stops hand-building `BTreeMap`s for dispatch-side logs
- Keep visibility narrow with `pub(crate)` and do not introduce trait-heavy “executor” abstractions unless a concrete compile-time seam is actually needed.
- Keep `phase` / `role` transition emission in `worker.rs` beside state publication; those events describe published-state transitions rather than dispatch mechanics.

3. Keep the apply boundary typed and bucket-driven
- Keep `HaDecision::lower()` and `HaEffectPlan` as the only planning contract.
- `apply_effect_plan(...)` in the new apply module must accept `&HaEffectPlan` directly and dispatch in explicit bucket order rather than reconstructing planning logic in the worker.
- Preserve the existing deterministic order unless a test proves it is wrong:
  - postgres
  - lease
  - switchover
  - replication
  - safety
- Keep `HaAction`/`ActionId` only as apply-layer operation identifiers if they still help with logging and job IDs; do not let them become the primary planning model again.
- Preserve best-effort application semantics with aggregated typed errors, because existing tests already assert that mixed DCS/process failures still report all surfaced errors.

4. Extract DCS path knowledge and raw store mutation out of the worker
- Move `leader_path(...)`, `switchover_path(...)`, and DCS error formatting out of `worker.rs` into `apply.rs` as private coordination helpers first.
- Add focused helpers for:
  - acquire leader lease
  - release leader lease
  - clear switchover request
- Those helpers should return typed `ActionDispatchError` values or typed intermediate results that `apply.rs` can translate without stringly branching in the worker.
- Keep the worker unaware of raw `/{scope}/leader` and `/{scope}/switchover` path formatting details.
- Only split those helpers into a separate `coordination.rs` if `apply.rs` still ends up too large after the main extraction; do not fragment the module tree prematurely.

5. Extract process request assembly and filesystem mutation out of the worker
- Move all `ProcessJobRequest` construction out of `dispatch_action(...)` into `process_dispatch.rs`.
- Add typed builders/helpers for:
  - `StartPostgres`
  - `Promote`
  - `Demote`
  - `PgRewind`
  - `BaseBackup`
  - `Bootstrap`
  - `Fencing`
- Keep `process_job_id(...)` close to process dispatch or another apply-layer helper so the worker no longer owns job-id formatting.
- Move `wipe_data_dir(...)` into `process_dispatch.rs` as a private helper used only by recovery/bootstrap application.
- Ensure managed config materialization for `StartPostgres` stays in the process-dispatch layer, not in the worker.
- Preserve all current runtime inputs (`RuntimeConfig`, `ProcessDispatchDefaults`, `scope`, `self_id`, `ha_tick`, action index) without adding `unwrap`, `expect`, or panicking filesystem behavior.

6. Extract event payload construction and logging helpers
- Move `ha_base_attrs(...)`, `serialize_attr_value(...)`, and the `emit_ha_*` helpers into `events.rs`.
- Keep event names, severities, and metadata stable unless tests/docs prove a correction is needed.
- Provide small helpers for:
  - decision selected
  - effect plan selected
  - action intent/dispatch/result
  - lease transition
- This module should own the repetitive `BTreeMap<String, serde_json::Value>` construction so `step_once(...)` and `apply_effect_plan(...)` become readable.
- Do not collapse all logging into one generic function that hides event semantics; the helpers should remain explicit and domain-named.
- Keep phase/role transition comparison and emission in `worker.rs`, but reuse shared attribute serialization from `events.rs` if that reduces duplication without hiding ownership.

7. Refactor `step_once(...)` only after helper boundaries exist
- First create the new helper modules and move code with behavior-preserving tests.
- Then shrink `step_once(...)` so it becomes straightforward orchestration:
  - `let world = world_snapshot(ctx);`
  - `let output = decide(...);`
  - `let plan = output.outcome.decision.lower();`
  - emit decision/plan events through `events.rs`
  - `let dispatch_errors = apply_effect_plan(ctx, output.next.tick, &plan);`
  - publish next state and emit transition events
- Keep the pure `world_snapshot(...)` helper either in `worker.rs` or a tiny facts-oriented helper only if that move makes the orchestration materially clearer. Do not create a new module just to move one small function.
- Preserve current worker faulting behavior: any collected apply errors must still become `WorkerStatus::Faulted(WorkerError::Message(format_dispatch_errors(...)))`.

8. Update module wiring and file ownership cleanly
- Update `src/ha/mod.rs` to register the new helper modules with narrow visibility.
- Keep decision/lowering modules untouched unless compile-time ownership forces a tiny import cleanup.
- Avoid circular dependencies:
  - `worker.rs` depends on `apply.rs` and `events.rs`
  - `apply.rs` depends on `process_dispatch.rs` and `events.rs`
  - `decision.rs` / `lower.rs` must stay pure and must not depend on apply-side modules
- If `ActionDispatchError` logically belongs to apply rather than worker after extraction, move it into `apply.rs` and update imports/tests accordingly.

9. Testing strategy during execution
- Update existing worker dispatch tests to target the new boundaries instead of only the old `worker::dispatch_effect_plan(...)` location.
- Add or move focused unit tests so each extracted concern is pinned:
  - `apply.rs`: correct leader/switchover paths, DCS error mapping, bucket sequencing, best-effort error aggregation, and switchover clear behavior
  - `process_dispatch.rs`: correct `ProcessJobRequest` construction and filesystem failure handling
  - `events.rs`: stable serialized payload fields for at least one decision/plan/action/lease event path
- Keep one integration-style worker test proving that `step_once(...)` still performs decide -> lower -> apply -> publish in order, and that phase/role transition logging still happens at the publication boundary.
- If extraction exposes existing missing coverage or flaky behavior, fix the tests rather than weakening assertions.

10. Documentation updates required in the same pass
- Update `docs/src/contributors/ha-pipeline.md` so it names the new apply/process-dispatch/event helper boundaries, and explicitly states that publication-time phase/role transition logs stay in `worker.rs`.
- Check `docs/src/contributors/worker-wiring.md` for any stale HA worker ownership statements and update/remove them if they no longer match the code.
- Remove or rewrite any stale text that still says this task intentionally skips `make test-long`, because task completion in this run requires the full gate set.

11. Execution order for the later `NOW EXECUTE` pass
1. Add new helper module files and wire them in `src/ha/mod.rs`.
2. Move process request assembly and filesystem prep into `process_dispatch.rs` with tests.
3. Move decision/plan/action/lease event attribute helpers into `events.rs` with tests or updated assertions.
4. Introduce `apply_effect_plan(...)` in `apply.rs`, move `ActionDispatchError` plus per-bucket dispatch there, and fold in the small DCS coordination helpers with tests.
5. Shrink `src/ha/worker.rs` to orchestration plus final state publication/transition logging only.
7. Update contributor docs to match the new module boundaries and task policy reality.
8. Run targeted HA tests first for fast feedback.
9. Run the required full gates in this order:
   - `make check`
   - `make test`
   - `make test-long`
   - `make lint`
10. Only after all gates pass, tick acceptance boxes, set `<passes>true</passes>`, run `/bin/bash .ralph/task_switch.sh`, commit, and push.

12. Verification conclusions to honor during execution
- Default to three helper modules (`apply.rs`, `process_dispatch.rs`, `events.rs`); do not create `coordination.rs` unless the extracted DCS helpers demonstrably remain too large or awkward inside `apply.rs`.
- Keep phase/role transition logging in `worker.rs` because it belongs to the publish boundary, not the apply boundary.
- Preserve the current bucket order `postgres -> lease -> switchover -> replication -> safety` unless a failing test demonstrates a concrete ordering bug.
- Keep `HaAction` only if it still serves as a compact apply-layer operation identifier after extraction; if the implementation becomes clearer without it and tests remain strong, deleting it is allowed.
- Keep `world_snapshot(...)` in `worker.rs` unless the final orchestration still reads poorly after the apply/event extractions.
</execution_plan>

NOW EXECUTE
