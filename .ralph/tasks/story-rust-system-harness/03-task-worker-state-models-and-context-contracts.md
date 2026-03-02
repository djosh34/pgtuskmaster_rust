---
## Task: Define worker state models and run step_once contracts <status>not_started</status> <passes>false</passes> <priority>ultra_high</priority>

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
- [ ] `ProcessState`, `ProcessJobKind`, `JobOutcome`, `HaPhase`, `HaState`, `WorldSnapshot`, and `SystemSnapshot` are defined per plan.
- [ ] Each worker module exports exactly `run` and `step_once` as `pub(crate)` contracts.
- [ ] No broad `pub` leakage beyond crate-root needs.
- [ ] Contract tests compile-check signature stability and module visibility.
- [ ] Run `make check`.
- [ ] Run `make test`.
- [ ] Run `make lint`.
- [ ] Run `make test-bdd`.
- [ ] If any fail, use `$add-bug` skill to create bug task(s) in `.ralph/tasks/bugs/`.
</acceptance_criteria>
