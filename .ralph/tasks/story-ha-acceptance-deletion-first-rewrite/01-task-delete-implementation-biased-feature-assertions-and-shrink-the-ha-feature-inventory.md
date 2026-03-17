## Task: Delete Implementation-Biased Feature Assertions And Shrink The HA Feature Inventory <status>completed</status> <passes>true</passes>

<priority>high</priority>

<description>
**Goal:** Rewrite the HA `.feature` inventory so scenarios stop asserting implementation details and stop embedding correctness logic that should live in perpetual scenario-agnostic runners. The higher-order goal is to make the feature text describe only topology/given selection, fault actions, and the minimal business outcomes the PO asked for:

- `Then cluster becomes healthy`
- `Then cluster becomes unhealthy`
- the single switchover exception where a scenario may say a specific node becomes primary

This task is intentionally deletion-first. Do not preserve old scenario wording just because it already exists. Remove the old assertions before later tasks add back the two perpetual guarantees and the minimal outcome DSL.

**Context from research:**
- There are 32 HA feature files today. 29 of them use proof-row propagation scaffolding, 18 use primary-history or dual-primary tracking, 11 use exact `pgtm`/API/debug assertions, and 4 use workload/fencing evidence. That is the surface area this story is intentionally deleting.
- `tests/ha/features/` currently contains a large one-file-per-scenario tree with many assertions that are explicitly out of bounds for the requested rewrite.
- Examples that must disappear from feature text because they belong in perpetual runners or are implementation-biased:
  - `When I start a bounded concurrent write workload and record commit outcomes`
  - `When I stop the workload and verify it committed at least one row`
  - `Then there is no dual-primary evidence during the transition window`
  - `Then there is no dual-primary evidence and no split-brain write evidence during the transition window`
  - `Then exactly one primary exists across 2 running nodes as "majority_primary"`
  - `Then I wait for exactly one stable primary as "restored_primary"`
  - `Then the 3 online nodes contain exactly the recorded proof rows`
  - `Then the node named "old_primary" rejoins as a replica`
  - `Then the primary history never included "old_primary"`
  - `Then the node named "blocked_node" emitted blocker evidence for "pg_basebackup"`
  - any `reports state`, `is not queryable`, `remains offline`, `logs contain`, or similar implementation-level expectations
- The current feature inventory is materially larger than needed for the intended product question: "do we still end up with a self-managing HA PostgreSQL cluster?"
- `tests/ha.rs` and `tests/nextest_config_contract.rs` currently assume the present feature-file inventory and must be updated alongside the feature reduction.
- If this story conflicts with `.ralph/tasks/story-ctl-operator-experience/07-task-refactor-ha-acceptance-suite-around-node-state-invariants.md`, follow this story. The prior task still preserved too much scenario-local assertion structure.

**Scope:**
- Rewrite the `.feature` files under `tests/ha/features/`.
- Merge, rename, or delete current feature files aggressively. Do not preserve one file per current scenario class.
- Update `tests/ha.rs` to match the surviving feature file list.
- Update `tests/nextest_config_contract.rs` if the feature-count or inventory assertions need to change after the reduction.

**Target feature shape:**
- Each scenario may keep:
  - a given
  - fault or operator actions such as kill, restart, isolate, stop quorum, heal, request switchover, wipe data, enable/disable blocker
  - the reduced outcome checks from this story
- Each scenario must stop spelling out internal safety logic that the perpetual runners will own.
- The final feature inventory must be radically smaller than the current tree. Aim for one merged feature file per major failure family, not per current micro-scenario.
- A concrete target inventory that fits the PO request is:
  - `ha_replica_faults_keep_cluster_healthy.feature`
  - `ha_primary_faults_fail_over_then_recover.feature`
  - `ha_quorum_loss_and_dcs_loss.feature`
  - `ha_rejoin_and_restart_recovery.feature`
  - `ha_operator_switchovers.feature`

**Expected outcome:**
- The HA feature suite reads as a small product-level scenario inventory instead of a procedural script full of proof-row, workload, primary-history, and implementation-role assertions.
- The surviving feature files are materially fewer and clearer.
- Later tasks can add the perpetual guarantees and minimal outcome steps without fighting old scenario baggage.

</description>

<acceptance_criteria>
- [x] Rewrite `tests/ha/features/` so feature text no longer contains workload-start/stop steps, proof-row assertions, dual-primary assertions, primary-history assertions, role/state assertions, log assertions, or blocker-evidence assertions.
- [x] Keep only the reduced outcome family in feature text: `cluster becomes healthy`, `cluster becomes unhealthy`, and the narrow switchover-specific `"<node-or-alias>" becomes primary` style check.
- [x] Radically reduce the inventory under `tests/ha/features/`; the post-rewrite suite should converge to the 5-file target inventory above unless the codebase proves an even smaller inventory is correct.
- [x] Update `tests/ha.rs` so it references only the surviving merged feature files.
- [x] Update `tests/nextest_config_contract.rs` so any feature-count expectation matches the reduced inventory.
- [x] Document in the surviving feature files which scenarios were intentionally merged or deleted, so the reduction is explicit rather than accidental.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

### Execution plan
1. Inventory the current `tests/ha/features/**/*.feature` files by fault family and delete the one-file-per-micro-scenario assumption.
2. Collapse the current tree into a small merged inventory grouped by major failure families such as failover/rejoin, partitions, DCS quorum loss, switchover, and recovery-path failures.
3. Replace old scenario assertions with only the reduced outcome wording required by this story.
4. Update `tests/ha.rs` and any feature-count contract checks to the new inventory.
5. Run the required repo validation gates only after the reduced inventory compiles cleanly against the new step surface from later tasks.

### Constraints for execution
- Deletion comes first. Do not preserve old proof/workload/primary-history wording "for safety".
- Do not keep feature files that differ only by old assertions that this story explicitly removes.
- Do not add new implementation-heavy step wording to compensate for the removed lines.
- Do not run `cargo test`; use the required `make` targets, and use `cargo nextest` only for focused local iteration if absolutely needed before the final validation gates.

NOW EXECUTE
