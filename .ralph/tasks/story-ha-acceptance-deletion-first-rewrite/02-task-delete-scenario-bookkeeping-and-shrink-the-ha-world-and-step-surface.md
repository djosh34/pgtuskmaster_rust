## Task: Delete Scenario Bookkeeping And Shrink The HA World And Step Surface <status>completed</status> <passes>true</passes>

<priority>high</priority>

<description>
**Goal:** Remove the current scenario-local bookkeeping layers that make the HA suite overengineered: proof-row tracking, bounded workload state, primary-history windows, marker bookkeeping, command-output caches, unsampled/observation scope state, and other test-only structs that exist only to support the old assertion style. The higher-order goal is to make `tests/ha/support/world/mod.rs` almost empty and make `tests/ha/support/steps/mod.rs` a thin adapter over a minimal harness API instead of a giant mixed-concern file.

This task is deliberately destructive. Delete the old bookkeeping first so later add-back tasks do not inherit the old shape by accident.

**Context from research:**
- `tests/ha/support/world/mod.rs` currently defines a large struct tree for scenario bookkeeping, including:
  - `ScenarioState`
  - `MemberAlias`
  - `MarkerName`
  - `ProofRow`
  - `ProofTableName`
  - `WritablePrimaryTarget`
  - `ScenarioAlias`
  - `MemberSet`
  - `AliasRegistry`
  - `ProofLedger`
  - `WorkloadState`
  - `CommandState`
  - `ObservationScope`
  - `TransitionWindow`
  - `InvariantHistory`
- `tests/ha/support/workload/mod.rs` adds another struct tree:
  - `SqlWorkloadHandle`
  - `WorkloadEvent`
  - `WorkloadOutcome`
  - `WorkloadSummary`
- `tests/ha/support/steps/mod.rs` currently contains the step families that the PO explicitly wants gone, including proof-row steps, bounded workload steps, dual-primary steps, fencing-cutoff steps, primary-history steps, queryability/log steps, blocker-evidence steps, and many role-specific assertions.
- If this story conflicts with `.ralph/tasks/story-ctl-operator-experience/07-task-refactor-ha-acceptance-suite-around-node-state-invariants.md`, follow this story. The new request is to delete far more scaffolding, not merely rename it.

**Scope:**
- Rewrite `tests/ha/support/steps/mod.rs` around the reduced feature vocabulary from this story.
- Aggressively shrink `tests/ha/support/world/mod.rs` so it keeps only the minimum scenario state still needed after the deletion.
- Delete `tests/ha/support/workload/mod.rs` if the new architecture no longer needs it. If a tiny replacement is needed later, create it in a later task instead of preserving this module.
- Update `tests/ha/support/mod.rs` and any adjacent support modules to remove dead exports and dead helpers.

**Specific deletion targets:**
- Remove step definitions for:
  - proof-table creation and proof-row insertion/visibility
  - bounded workload start/stop and workload-summary assertions
  - dual-primary and split-brain evidence assertions
  - fencing-cutoff assertions
  - primary-history and marker assertions
  - log-content and blocker-evidence assertions
  - generic queryability assertions
  - implementation-specific role/rejoin/offline assertions that the reduced feature DSL no longer allows
- Remove world state whose only purpose was to support those steps.
- Remove helper functions that only exist to keep the deleted steps alive.

**Expected outcome:**
- The HA step layer becomes much smaller and is aligned with the reduced feature inventory.
- The HA world becomes minimal instead of acting like a large mutable scenario database.
- The current workload/proof/history scaffolding is gone, making room for simpler later tasks.

</description>

<acceptance_criteria>
- [x] Delete `tests/ha/support/workload/mod.rs` entirely, or reduce it to nothing more than a tiny replacement that is demonstrably required by later tasks; do not preserve the current workload struct tree.
- [x] Rewrite `tests/ha/support/steps/mod.rs` so it no longer exposes proof-row, workload, dual-primary, fencing-cutoff, primary-history, queryability, log, or blocker-evidence step definitions.
- [x] Shrink `tests/ha/support/world/mod.rs` so the current scenario bookkeeping struct tree is drastically reduced; remove every struct whose only purpose was to support the deleted step families.
- [x] Remove dead exports from `tests/ha/support/mod.rs` and any adjacent support modules after the deletion.
- [x] Ensure the remaining world/step boundary is small enough that a reader can identify the surviving scenario state in one short pass.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

### Execution plan
1. Delete the old step families from `tests/ha/support/steps/mod.rs` according to the reduced feature DSL.
2. Delete or inline the world/workload structs that no longer have a reason to exist after those steps are removed.
3. Rebuild the surviving `HaWorld` state around the tiny set of values still required by the reduced feature suite.
4. Remove dead helper functions and module exports after the compile errors reveal the remaining accidental complexity.
5. Run the required repo validation gates after the reduced harness compiles and the surviving feature suite is wired again.

### Constraints for execution
- Prefer deletion over renaming. If a struct or helper only exists for the old scenario bookkeeping model, remove it.
- Do not recreate the same complexity behind new names.
- Keep the remaining world state intentionally small, even if that means rethinking current helper boundaries.
- Do not run `cargo test`; use the required `make` targets, and use `cargo nextest` only for focused local iteration if absolutely needed before the final validation gates.

NOW EXECUTE
