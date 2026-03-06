## Task: Define worker state models and run step_once contracts <status>done</status> <passes>true</passes> <priority>ultra_high</priority>

<blocked_by>02-task-runtime-config-schema-defaults-parse-validate</blocked_by>

<description>
**Goal:** Create all worker state enums/context structs and expose only minimal cross-module contracts.

**Scope:**
- Create module skeletons for `pginfo`, `dcs`, `process`, `ha`, `api`, and `debug_api`.
- Add state types from the plan and `run(ctx)` / `step_once(&mut ctx)` signatures per worker.
- Ensure private-by-default internals and only required `pub(crate)` surfaces.

**Context from research:**
- Build Order steps 3 and 4.
- This task is structure-first and compiler-driven for downstream implementation.

**Expected outcome:**
- Complete typed interfaces exist so submodules can be implemented in parallel without API churn.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] `ProcessState`, `ProcessJobKind`, `JobOutcome`, `HaPhase`, `HaState`, `WorldSnapshot`, and `SystemSnapshot` are defined per plan.
- [x] Each worker module exports exactly `run` and `step_once` as `pub(crate)` contracts.
- [x] No broad `pub` leakage beyond crate-root needs.
- [x] Contract tests compile-check signature stability and module visibility.
- [x] Run `make check`.
- [x] Run `make test`.
- [x] Run `make lint`.
- [x] Run `make test`.
- [x] If any fail, use `$add-bug` skill to create bug task(s) in `.ralph/tasks/bugs/`. (N/A: all required commands passed)
</acceptance_criteria>

<execution_plan>
## Detailed Implementation Plan

1. Baseline and module scaffold discovery
- [x] Confirm current crate exports in `src/lib.rs` and preserve existing `config` and `state` surfaces.
- [x] Confirm no existing `src/{pginfo,dcs,process,ha,api,debug_api}` trees; if missing, create exact plan layout with `mod.rs` and split files.
- [x] Capture a pre-change compile baseline (`cargo check`) to isolate regressions from this task.

2. Define contract-first ownership boundaries
- [x] Keep internals private by default in all new modules.
- [x] Restrict cross-module interfaces to `pub(crate)` where needed for near-term tasks.
- [x] Limit worker contracts to `run(ctx)` and `step_once(&mut ctx)` as exported worker functions.

3. Add `pginfo` module skeleton and core state placeholders
- [x] Create:
  - [x] `src/pginfo/mod.rs`
  - [x] `src/pginfo/state.rs`
  - [x] `src/pginfo/query.rs`
  - [x] `src/pginfo/worker.rs`
- [x] Define placeholder-but-typed state contracts required for downstream compilation (`PgInfoState`, supporting structs/enums used by signatures).
- [x] Add `pub(crate)` `run` and `step_once` signatures in `worker.rs` with `todo!()`/stub body returning typed errors as appropriate.

4. Add `dcs` module skeleton and state/cache placeholders
- [x] Create:
  - [x] `src/dcs/mod.rs`
  - [x] `src/dcs/keys.rs`
  - [x] `src/dcs/state.rs`
  - [x] `src/dcs/store.rs`
  - [x] `src/dcs/worker.rs`
- [x] Define minimal typed `DcsState`, trust enum, and cache/container placeholders referenced by current and upcoming tasks.
- [x] Add `pub(crate)` `run` and `step_once` signatures in `worker.rs`.

5. Add `process` module and required state enums from plan
- [x] Create:
  - [x] `src/process/mod.rs`
  - [x] `src/process/jobs.rs`
  - [x] `src/process/state.rs`
  - [x] `src/process/worker.rs`
- [x] Implement plan-required enums in `state.rs`:
  - [x] `ProcessState`
  - [x] `ProcessJobKind`
  - [x] `JobOutcome`
- [x] Add minimal placeholder structs/enums for referenced payload types (`ActiveJob`, job specs, cancel reason, process errors) so signatures compile.
- [x] Add `pub(crate)` `run` and `step_once` signatures in `worker.rs`.

6. Add `ha` module and required phase/snapshot contracts
- [x] Create:
  - [x] `src/ha/mod.rs`
  - [x] `src/ha/state.rs`
  - [x] `src/ha/decide.rs`
  - [x] `src/ha/actions.rs`
  - [x] `src/ha/worker.rs`
- [x] Implement plan-required types:
  - [x] `HaPhase`
  - [x] `HaState`
  - [x] `WorldSnapshot`
- [x] Define `DecideInput`/`DecideOutput` placeholder contracts aligned with plan to prevent churn for task 07/08.
- [x] Add `pub(crate)` `run` and `step_once` signatures in `worker.rs`.

7. Add `api` and `debug_api` module scaffolds with system snapshot contract
- [x] Create:
  - [x] `src/api/mod.rs`
  - [x] `src/api/controller.rs`
  - [x] `src/api/fallback.rs`
  - [x] `src/api/worker.rs`
  - [x] `src/debug_api/mod.rs`
  - [x] `src/debug_api/snapshot.rs`
  - [x] `src/debug_api/worker.rs`
- [x] Implement `SystemSnapshot` contract in `debug_api/snapshot.rs` referencing versioned typed worker states.
- [x] Add `pub(crate)` `run` and `step_once` signatures for API worker and Debug API worker.
- [x] Add `build_snapshot` signature in debug API as plan-defined contract for downstream implementation.

8. Wire crate modules without over-exporting
- [x] Update `src/lib.rs` to declare new modules with crate-level visibility policy matching current project style.
- [x] In each `mod.rs`, expose only what downstream modules/tests need (prefer `pub(crate) use ...`; keep file-local helpers private).
- [x] Ensure no accidental `pub` leakage of internal worker helpers or internal state fields.

9. Add contract tests for signature stability and visibility
- [x] Add focused compile-level tests (unit/integration) validating:
  - [x] required enums/structs exist with expected names and module paths
  - [x] worker `run`/`step_once` symbols are available as `pub(crate)` within crate
  - [x] internals are not broadly public
- [x] Prefer tests that assert trait bounds/usability via compile-time type checks rather than behavior.

10. Verification and failure protocol
- [x] Run `make check`.
- [x] Run `make test`.
- [x] Run `make test`.
- [x] Run `make lint`.
- [x] If any command fails, invoke `$add-bug` skill and create bug task(s) in `.ralph/tasks/bugs/` with repro command and failing log excerpt.

11. Task bookkeeping and completion protocol (execution phase only)
- [x] Update this task file checkboxes as each acceptance criterion is proven.
- [x] Set header tags to done/passes true only after all required commands pass.
- [x] Run `/bin/bash .ralph/task_switch.sh` only after full pass.
- [x] Commit with required format: `task finished 03-task-worker-state-models-and-context-contracts: <summary + evidence + challenges>`.
- [x] Include `.ralph` updates and append learnings/surprises to `AGENTS.md`.
- [x] Append diary entry to progress log before exiting.

12. Skeptical verification amendments (added during TO BE VERIFIED)
- [x] Add a hard prerequisite gate before any implementation: verify task `02-task-runtime-config-schema-defaults-parse-validate` is complete with `<passes>true</passes>`; if not, stop execution and switch task rather than introducing layered churn.
- [x] Use crate-local unit contract tests (under `src/`) for `pub(crate)` symbol visibility checks and integration tests (under `tests/`) only for public crate-root surfaces.
- [x] During module wiring, avoid `pub mod ...` in `src/lib.rs`; prefer private modules plus selective `pub use`/`pub(crate) use` to minimize public API growth.
</execution_plan>

NOW EXECUTE
