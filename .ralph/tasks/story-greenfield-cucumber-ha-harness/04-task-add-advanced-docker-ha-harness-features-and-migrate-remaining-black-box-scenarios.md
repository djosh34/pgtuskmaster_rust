## Task: Add Advanced Docker HA Harness Features And Migrate Remaining Black-Box Scenarios <status>not_started</status> <passes>false</passes>

<priority>high</priority>

<description>
**Goal:** Extend the greenfield Docker cucumber HA harness so it can absorb the less-easy-to-migrate black-box scenarios that still qualify for migration. This task must describe, implement, and verify the extra harness features required for those harder scenarios, and it must list the full advanced scenario set to migrate into the Docker-based test path. The key rule is the user's rule: if the behavior can be tested by starting real `pgtuskmaster` binaries and then controlling/observing them from the outside, primarily through `pgtm` and Docker/network control, it belongs here rather than in the legacy HA/E2E/half-BDD harness.

**Original user shift / motivation:** The user wants the migration boundary made explicit. The easy features land in tasks 01 and 02. After that, the repo should not drift back into the legacy harness out of convenience. This task must therefore be the contract for the harder-but-still-black-box stories, and it must say exactly what extra Docker harness capabilities are needed in terms of control, compose topology, network faulting, workload generation, fault wrappers, and other external orchestration. The user also wants the full scenario list written down here so task 03 can remove the corresponding legacy tests completely.

**Higher-order goal:** Finish the migration of black-box behavior testing onto one greenfield architecture: real containers, real binaries, operator-visible control and observation, one-scenario-per-feature cucumber structure, and no hidden dependence on the old harness for scenarios that are externally testable.

**Scope:**
- Extend the greenfield harness under `cucumber_tests/ha/` rather than adding more behavior to the legacy harness.
- Add only the extra harness features needed for externally testable scenarios. Keep true deep-control / unit-contract tests out of scope.
- Update `cucumber_tests/ha/support/...`, `cucumber_tests/ha/givens/...`, `Cargo.toml`, `Makefile`, and greenfield HA docs as required.
- Implement one scenario per feature for the advanced black-box stories listed below.
- Keep `pgtm` as the primary control and observation surface after startup whenever the behavior is exposed there; use Docker/network/fault controls only for fault injection and topology shaping.
- Explicit non-goals for this task:
- `src/worker_contract_tests.rs` style internal worker contract smoke
- `tests/bdd_state_watch.rs` style tiny channel-flow tests
- parser/config/exit-code tests that still belong as unit/CLI contract tests
- any scenario that fundamentally requires white-box internal hooks rather than real runtime orchestration
- The old runtime-only restart / “kill pgtuskmaster only” style scenario is not automatically in scope here. If it cannot be expressed honestly through the agreed greenfield operator model, it must stay out and be called out explicitly rather than half-migrated.

**Context from research:**
- The remaining black-box scenarios currently spread across legacy files include:
- `tests/ha_multi_node_failover.rs`
- `tests/ha_partition_isolation.rs`
- parts of `tests/cli_binary.rs`
- parts of `tests/bdd_api_http.rs`
- The research pass identified the following scenario families as still migratable into the greenfield Docker path once the harness grows the needed features:
- workload-driven switchover and failover
- targeted switchover rejection to an ineligible member
- custom postgres role / recovery config variants
- clone failure and rewind fallback recovery
- repeated leadership changes
- degraded / lagging replica promotion choice
- no-quorum fail-safe and fencing under DCS quorum loss
- full 1:2 partition majority/minority stories
- API-path, postgres-path, and mixed network fault stories
- storage/WAL stall replacement of a wedged primary
- broken returning node and broken replica rejoin stories
- The easy planned-switchover and targeted-switchover accepted-path features are already assigned to task 02 and must not be duplicated here.
- Existing old scenario names that this task is intended to replace or subsume include:
- `e2e_multi_node_stress_planned_switchover_concurrent_sql`
- `e2e_multi_node_stress_unassisted_failover_concurrent_sql`
- `e2e_multi_node_custom_postgres_role_names_survive_bootstrap_and_rewind`
- `e2e_multi_node_clone_failure_recovers_after_fault_removed`
- `e2e_multi_node_rewind_failure_falls_back_to_basebackup`
- `e2e_multi_node_repeated_leadership_changes_preserve_single_primary`
- `e2e_multi_node_degraded_replica_failover_promotes_only_healthy_target`
- `e2e_multi_node_rejects_targeted_switchover_to_ineligible_member`
- `e2e_no_quorum_enters_failsafe_strict_all_nodes`
- `e2e_no_quorum_fencing_blocks_post_cutoff_commits_and_preserves_integrity`
- `e2e_partition_minority_isolation_no_split_brain_rejoin`
- `e2e_partition_primary_isolation_failover_no_split_brain`
- `e2e_partition_api_path_isolation_preserves_primary`
- `e2e_partition_primary_postgres_path_blocked_replicas_catch_up_after_heal`
- `e2e_partition_mixed_faults_heal_converges`
- plus the still-open migrated story files from the old quorum task set: storage/WAL stall, broken returning node, lagging/stale replica promotion, minority old primary stale rejoin, broken replica rejoin
- The advanced harness features this task must define are external orchestration features, not internal worker hooks.

**Expected outcome:**
- The greenfield Docker cucumber harness can cover the remaining externally testable HA stories without falling back to the legacy harness.
- The repo gains a written and implemented contract for advanced fault orchestration: network partitions, DCS quorum loss, workload generation, recovery-binary blockers, lag/staleness shaping, storage stall injection, and config variants.
- Task 03 can then delete the corresponding legacy test infrastructure with confidence because the advanced migrated scenarios are explicitly enumerated and shipped here.

**Scenario contracts:** Every advanced migrated feature in this task must implement the concrete behavior defined below. The executor must not replace these with looser or different stories.

1. `stress_planned_switchover_concurrent_sql`
- Start `three_node_plain` and wait for exactly one stable primary.
- Create one workload table dedicated to this feature.
- Start a bounded concurrent write workload with recorded commit outcomes.
- While the workload is active, run `pgtm switchover request`.
- Prove exactly one different primary stabilizes.
- Prove there is no dual-primary evidence during the sampled transition window.
- Stop the workload and prove it committed at least one row.
- Insert one explicit post-switchover proof row through the final primary.
- Verify table-key integrity and convergence on the final cluster.

2. `stress_failover_concurrent_sql`
- Start `three_node_plain` and wait for exactly one stable primary.
- Create one workload table dedicated to this feature.
- Start a bounded concurrent write workload with recorded commit outcomes.
- While the workload is active, inject an unassisted failover fault against the current primary using the greenfield fault model, not the legacy harness.
- Prove exactly one different primary stabilizes.
- Prove there is no split-brain write evidence and no dual-primary evidence during the sampled transition window.
- Stop the workload and prove it committed at least one row.
- Insert one explicit post-failover proof row through the final primary.
- Verify table-key integrity and convergence on the final cluster.

3. `targeted_switchover_rejects_ineligible_member`
- Start `three_node_plain` and wait for exactly one stable primary.
- Select one replica and intentionally make it ineligible using the advanced degraded-member machinery.
- Run `pgtm switchover request --switchover-to <ineligible-replica>`.
- Prove the request is rejected with an operator-visible error.
- Prove the current primary does not change.
- Heal the ineligible replica.
- Insert one proof row through the unchanged primary.
- Verify convergence across the restored cluster.

4. `custom_postgres_roles_survive_failover_and_rejoin`
- Start a checked-in custom-role given with non-default replicator and rewinder usernames/passwords.
- Wait for exactly one stable primary.
- Create one proof table and insert proof row `1:before-custom-role-failover`.
- Inject failover against the current primary.
- Wait for a different primary.
- Insert proof row `2:after-custom-role-failover`.
- Wait for the old primary to rejoin as a replica under the custom-role configuration.
- Verify all nodes converge on both proof rows.

5. `clone_failure_recovers_after_blocker_removed`
- Start a cluster where one chosen node has a controllable `pg_basebackup` blocker.
- Wait for exactly one stable primary and prove the blocked node initially works before the forced reclone path.
- Create one proof table and insert proof row `1:before-clone-failure`.
- Force the blocked node into a fresh clone path by wiping its data and enabling the blocker.
- Insert proof row `2:during-clone-failure` while the blocked node is still broken.
- Prove the blocked node remains non-queryable and non-primary.
- Remove the blocker and restart that node.
- Wait for it to rejoin as a replica.
- Verify convergence on exactly rows `1:before-clone-failure`, `2:during-clone-failure`.

6. `rewind_failure_falls_back_to_basebackup`
- Start a cluster where the initial primary has a controllable `pg_rewind` blocker.
- Create one proof table and insert proof row `1:before-rewind-failure`.
- Force failover away from the initial primary.
- Insert proof row `2:after-failover`.
- Prove the old primary cannot complete `pg_rewind` but still rejoins through the fallback recovery path.
- Prove the old primary rejoins only as a replica.
- Verify convergence on exactly rows `1:before-rewind-failure`, `2:after-failover`.

7. `repeated_leadership_changes_preserve_single_primary`
- Start `three_node_plain` and wait for exactly one stable primary.
- Record the initial primary as `primary_a`.
- Inject a failover fault against `primary_a` and wait for `primary_b`.
- Inject a second failover fault against `primary_b` and wait for `primary_c`.
- Prove `primary_a`, `primary_b`, and `primary_c` are distinct in order when the topology allows that sequence.
- Prove there is never a dual-primary observation across the full churn window.

8. `lagging_replica_is_not_promoted`
- Start `three_node_plain` and wait for exactly one stable primary.
- Choose one replica as `degraded_replica` and the other as `healthy_replica`.
- Use the advanced lag/staleness machinery to make `degraded_replica` observably worse than `healthy_replica`.
- Create one proof table and insert proof row `1:before-lagging-failover`.
- Inject primary failure.
- Prove `healthy_replica` becomes the only primary.
- Prove `degraded_replica` does not become primary during the failover window.
- Insert proof row `2:after-lagging-failover` through `healthy_replica`.
- Heal the degraded replica if necessary and verify final convergence.

9. `no_quorum_enters_failsafe`
- Start `three_node_plain` and wait for exactly one stable primary.
- Stop a DCS quorum majority through the greenfield DCS fault controls.
- Prove all nodes lose quorum and no operator-visible primary remains.
- Prove the cluster enters fail-safe behavior rather than silently keeping a writable primary.
- Prove there is no dual-primary evidence during the no-quorum window.

10. `no_quorum_fencing_blocks_post_cutoff_commits`
- Start `three_node_plain` and wait for exactly one stable primary.
- Create one workload table and start a bounded concurrent write workload with recorded commit timestamps or equivalent cutoff evidence.
- Stop a DCS quorum majority while the workload is active.
- Determine the fail-safe cutoff using the greenfield recorded timing model.
- Prove post-cutoff commits are rejected or bounded according to the fencing contract.
- Restore quorum.
- Wait for exactly one stable primary again.
- Verify recovered table integrity against the allowed pre-cutoff commit set.

11. `full_partition_majority_survives_old_primary_isolated`
- Start `three_node_plain` and wait for exactly one stable primary.
- Ensure the current primary is the node that will become the 1-side minority.
- Create one proof table and insert proof row `1:before-primary-minority-partition`.
- Partition that old primary from the other two nodes across etcd, API, and postgres/replication paths together.
- Prove the majority side elects exactly one primary before heal.
- Prove the minority old primary is not an accepted primary outcome during the partition window.
- Insert proof row `2:on-majority-during-partition` through the majority primary.
- Heal the partition.
- Prove the old minority primary rejoins as a replica and final convergence includes both rows.

12. `full_partition_majority_survives_old_replica_isolated`
- Start `three_node_plain` and wait for exactly one stable primary.
- Choose one replica as the minority-isolated node.
- Create one proof table and insert proof row `1:before-replica-minority-partition`.
- Partition that replica from the two-node majority across etcd, API, and postgres/replication paths together.
- Prove the majority side preserves or converges to exactly one primary before heal.
- Prove the isolated replica does not self-promote.
- Insert proof row `2:on-majority-during-replica-partition`.
- Heal the partition and verify final convergence includes both rows.

13. `minority_old_primary_rejoins_safely_after_majority_failover`
- Start `three_node_plain` and make the current primary the 1-side minority in a full partition.
- Insert proof row `1:before-minority-old-primary-return`.
- Hold the partition long enough for the majority to elect a new primary and accept proof row `2:on-majority-after-failover`.
- Heal the partition.
- Prove the old minority primary does not remain or become primary after reconnect.
- Prove it follows the safe rejoin path as a replica.
- Verify final convergence includes both rows.

14. `api_path_isolation_preserves_primary`
- Start `three_node_plain` and wait for exactly one stable primary.
- Choose one non-primary node for API-only isolation.
- Apply API-path isolation only.
- Prove direct API observation to that node fails while cluster leadership stays unchanged.
- Prove the original primary remains the only primary throughout the isolation window.
- Heal the API path.
- Insert proof row `1:after-api-path-heal`.
- Verify convergence on all nodes.

15. `postgres_path_isolation_replicas_catch_up_after_heal`
- Start `three_node_plain` and wait for exactly one stable primary.
- Create one proof table and insert proof row `1:before-postgres-path-isolation`.
- Apply postgres/replication-path isolation from the primary to the replicas without removing the primary itself.
- Insert proof row `2:during-postgres-path-isolation` on the primary.
- Prove replicas do not see row `2:during-postgres-path-isolation` during the fault window.
- Heal the postgres path.
- Prove replicas catch up and final convergence includes rows `1:before-postgres-path-isolation`, `2:during-postgres-path-isolation`.

16. `mixed_network_faults_heal_converges`
- Start `three_node_plain` and wait for exactly one stable primary.
- Create one proof table and insert proof row `1:before-mixed-faults`.
- Apply a mixed fault: isolate the old primary from etcd and isolate a different node on the API path.
- Prove the old primary enters fail-safe or loses authority safely.
- Prove there is no dual-primary window.
- Heal all network faults.
- Wait for exactly one stable primary.
- Insert proof row `2:after-mixed-fault-heal`.
- Verify final convergence includes both rows.

17. `primary_storage_stall_replaced_by_new_primary`
- Start `three_node_plain` and wait for exactly one stable primary.
- Create one proof table and insert proof row `1:before-storage-stall`.
- Inject the advanced storage/WAL stall fault into the current primary so the node is wedged rather than cleanly dead.
- Prove the old primary is no longer a usable writable primary.
- Wait for a different node to become the only primary.
- Insert proof row `2:after-storage-stall-failover` through the new primary.
- Prove the wedged old primary does not remain or become primary.
- Heal or recover it as appropriate and verify final convergence includes both rows.

18. `two_node_loss_one_good_return_one_broken_return_recovers_service`
- Start `three_node_plain` and wait for exactly one stable primary.
- Create one proof table and insert proof row `1:before-two-node-loss`.
- Stop two node containers, leaving one survivor.
- Prove the lone survivor has no valid operator-visible primary outcome.
- Restart one stopped node normally.
- Keep the other stopped node broken through the advanced startup/recovery blocker.
- Wait for exactly one primary across the healthy pair.
- Insert proof row `2:after-good-return-before-broken-return-fix`.
- Prove the broken node does not block service restoration.
- Optionally heal the broken node and then verify final convergence.

19. `broken_replica_rejoin_does_not_block_healthy_quorum`
- Start `three_node_plain` and drive the cluster into a state with one healthy primary and one node attempting rejoin.
- Create one proof table and insert proof row `1:before-broken-rejoin`.
- Trigger a broken rejoin attempt for the chosen node using the advanced startup/recovery blocker.
- While the broken rejoin attempt is happening, insert proof row `2:during-broken-rejoin` through the healthy primary.
- Prove the healthy primary stays stable and unique.
- Prove the broken node never appears as primary during the broken rejoin window.
- Optionally heal the node and verify final convergence includes rows `1:before-broken-rejoin`, `2:during-broken-rejoin`.

</description>

<acceptance_criteria>
- [ ] The greenfield harness gains explicit advanced orchestration features under `cucumber_tests/ha/support/...` for all external-control needs this task covers:
- [ ] Docker/network fault control for full 1:2 partitioning and path-specific isolation across etcd, API, and PostgreSQL replication/data paths
- [ ] DCS quorum control through explicit etcd member/service stop-start or equivalent external faulting
- [ ] workload generation and commit/integrity telemetry for concurrent-SQL scenarios
- [ ] deterministic recovery-binary and startup fault wrappers for `pg_basebackup`, `pg_rewind`, and broken rejoin/startup cases
- [ ] lag/staleness / degraded-replica shaping sufficient to test promotion-choice safety
- [ ] storage/WAL stall style fault injection sufficient to wedge a primary without pretending the node vanished
- [ ] additional checked-in given/config variants when required, such as custom postgres role names
- [ ] The advanced scenario set is implemented as one scenario per feature, with tiny wrappers registered in `Cargo.toml`, for all scenarios this task migrates.
- [ ] At minimum, this task ships feature coverage for the following advanced scenario families:
- [ ] concurrent workload planned switchover
- [ ] concurrent workload failover
- [ ] targeted switchover rejection for an ineligible member
- [ ] custom postgres roles survive failover/rejoin
- [ ] clone failure recovers after blocker removal
- [ ] rewind failure falls back to basebackup
- [ ] repeated leadership changes preserve a single primary
- [ ] lagging/degraded replica is not promoted over the healthier candidate
- [ ] no-quorum fail-safe after DCS quorum loss
- [ ] no-quorum fencing blocks post-cutoff commits and preserves integrity
- [ ] full 1:2 partition where minority contains old primary
- [ ] full 1:2 partition where minority contains replica and majority preserves/retains authority
- [ ] minority old primary returns with stale view and is forced to rejoin safely
- [ ] API-path isolation preserves primary
- [ ] postgres-path isolation blocks replica catch-up until heal
- [ ] mixed network faults heal and converge
- [ ] primary storage/WAL stall causes replacement primary election
- [ ] two-node loss with one good return and one broken return still recovers service
- [ ] broken replica rejoin does not block healthy quorum availability
- [ ] Every advanced feature uses the greenfield Docker harness and operator-visible control/observation model; none of them import or call the legacy `tests/ha` or `src/test_harness/ha_e2e` code.
- [ ] The task explicitly records any scenario that still cannot honestly be expressed through the greenfield black-box model and splits it into a follow-up instead of silently keeping it in the old harness.
- [ ] Every advanced feature implements the exact scenario contract written in this task, including the specified fault shape, action order, topology assertions, and proof-row payloads; the executor does not silently substitute a different story.
- [ ] Docs are updated with the advanced harness capabilities, scenario inventory, and operator-control model, and stale docs are removed when relevant.
- [ ] `make check` passes cleanly
- [ ] `make test` passes cleanly
- [ ] `make test-long` passes cleanly
- [ ] `make lint` passes cleanly
- [ ] `<passes>true</passes>` is set only after every acceptance-criteria item and every required implementation-plan checkbox is complete
</acceptance_criteria>

## Detailed implementation plan

### Phase 1: Freeze the advanced migration boundary and the explicit non-goals
- [ ] Re-read tasks 01 and 02 and list exactly which advanced stories remain after the low-hanging outage and switchover features are done.
- [ ] Build an explicit “migrate now vs keep deep-control vs split later” table for the remaining old black-box scenarios from `tests/ha_multi_node_failover.rs`, `tests/ha_partition_isolation.rs`, and any black-box cases in `tests/cli_binary.rs` / `tests/bdd_api_http.rs`.
- [ ] Explicitly call out which old scenarios are not part of this task because they still require deep internal control rather than real runtime orchestration. Do not leave that decision implicit.

### Phase 2: Add the advanced harness capabilities
- [ ] Design and implement a greenfield network-fault layer under `cucumber_tests/ha/support/...` that can:
- [ ] isolate one node as the 1-side minority from the 2-side majority
- [ ] partition etcd/control-plane traffic separately from API traffic and postgres/replication traffic
- [ ] create API-only isolation
- [ ] create postgres-path-only isolation
- [ ] compose mixed faults and later heal them deterministically
- [ ] Design and implement explicit DCS quorum fault controls for the greenfield Docker topology:
- [ ] stop/start etcd members or equivalent DCS services
- [ ] prove majority loss and restoration through operator-visible observations
- [ ] Design and implement a reusable workload engine for greenfield cucumber:
- [ ] concurrent writers
- [ ] commit and rejection counting
- [ ] commit timestamp capture or equivalent cutoff analysis
- [ ] integrity verification after failover or fencing
- [ ] artifact capture for workload statistics
- [ ] Design and implement deterministic fault-wrapper support for recovery and startup paths:
- [ ] `pg_basebackup` failure / removal / retry
- [ ] `pg_rewind` failure with fallback path
- [ ] broken rejoin / broken startup blockers
- [ ] Design and implement degraded-replica shaping:
- [ ] replica made stale, lagging, or otherwise ineligible
- [ ] operator-visible evidence that it is the worse candidate
- [ ] promotion-choice assertions after primary loss
- [ ] Design and implement storage/WAL stall injection with externally observable evidence that the primary is wedged rather than cleanly dead.
- [ ] Add any required checked-in given/config variants, for example custom postgres role names, without collapsing back into runtime-generated opaque configs.

### Phase 3: Register the advanced feature directories and wrappers
- [ ] Add explicit feature directories plus tiny wrappers for each advanced migrated scenario. At minimum create feature roots for:
- [ ] `stress_planned_switchover_concurrent_sql`
- [ ] `stress_failover_concurrent_sql`
- [ ] `targeted_switchover_rejects_ineligible_member`
- [ ] `custom_postgres_roles_survive_failover_and_rejoin`
- [ ] `clone_failure_recovers_after_blocker_removed`
- [ ] `rewind_failure_falls_back_to_basebackup`
- [ ] `repeated_leadership_changes_preserve_single_primary`
- [ ] `lagging_replica_is_not_promoted`
- [ ] `no_quorum_enters_failsafe`
- [ ] `no_quorum_fencing_blocks_post_cutoff_commits`
- [ ] `full_partition_majority_survives_old_primary_isolated`
- [ ] `full_partition_majority_survives_old_replica_isolated`
- [ ] `minority_old_primary_rejoins_safely_after_majority_failover`
- [ ] `api_path_isolation_preserves_primary`
- [ ] `postgres_path_isolation_replicas_catch_up_after_heal`
- [ ] `mixed_network_faults_heal_converges`
- [ ] `primary_storage_stall_replaced_by_new_primary`
- [ ] `two_node_loss_one_good_return_one_broken_return_recovers_service`
- [ ] `broken_replica_rejoin_does_not_block_healthy_quorum`
- [ ] Register one `[[test]]` target in `Cargo.toml` per advanced wrapper.

### Phase 4: Implement the advanced scenarios exactly as specified
- [ ] Write and implement `stress_planned_switchover_concurrent_sql` exactly as specified in the scenario contract above.
- [ ] Write and implement `stress_failover_concurrent_sql` exactly as specified in the scenario contract above.
- [ ] Write and implement `targeted_switchover_rejects_ineligible_member` exactly as specified in the scenario contract above.
- [ ] Write and implement `custom_postgres_roles_survive_failover_and_rejoin` exactly as specified in the scenario contract above.
- [ ] Write and implement `clone_failure_recovers_after_blocker_removed` exactly as specified in the scenario contract above.
- [ ] Write and implement `rewind_failure_falls_back_to_basebackup` exactly as specified in the scenario contract above.
- [ ] Write and implement `repeated_leadership_changes_preserve_single_primary` exactly as specified in the scenario contract above.
- [ ] Write and implement `lagging_replica_is_not_promoted` exactly as specified in the scenario contract above.
- [ ] Write and implement `no_quorum_enters_failsafe` exactly as specified in the scenario contract above.
- [ ] Write and implement `no_quorum_fencing_blocks_post_cutoff_commits` exactly as specified in the scenario contract above.
- [ ] Write and implement `full_partition_majority_survives_old_primary_isolated` exactly as specified in the scenario contract above.
- [ ] Write and implement `full_partition_majority_survives_old_replica_isolated` exactly as specified in the scenario contract above.
- [ ] Write and implement `minority_old_primary_rejoins_safely_after_majority_failover` exactly as specified in the scenario contract above.
- [ ] Write and implement `api_path_isolation_preserves_primary` exactly as specified in the scenario contract above.
- [ ] Write and implement `postgres_path_isolation_replicas_catch_up_after_heal` exactly as specified in the scenario contract above.
- [ ] Write and implement `mixed_network_faults_heal_converges` exactly as specified in the scenario contract above.
- [ ] Write and implement `primary_storage_stall_replaced_by_new_primary` exactly as specified in the scenario contract above.
- [ ] Write and implement `two_node_loss_one_good_return_one_broken_return_recovers_service` exactly as specified in the scenario contract above.
- [ ] Write and implement `broken_replica_rejoin_does_not_block_healthy_quorum` exactly as specified in the scenario contract above.

### Phase 5: Enforce the greenfield boundary and prepare for task 03 cleanup
- [ ] Run repo-wide verification such as `rg -n "(tests/ha|tests/ha_|src/test_harness/ha_e2e)" cucumber_tests/ha Cargo.toml docs/src/how-to/run-tests.md` and confirm the advanced greenfield implementation does not depend on the legacy harness.
- [ ] In this task file, list the old test names and files that task 03 must delete once this migration lands, so cleanup is a mechanical follow-through rather than a fresh rediscovery exercise.
- [ ] Update docs and developer entrypoints so the advanced greenfield suite is discoverable and the old harness is clearly marked as pending removal by task 03.

### Phase 6: Verification and closeout
- [ ] Run targeted execution of each new advanced feature or scenario family so every advanced migrated story is exercised on top of the greenfield Docker harness.
- [ ] Run `make check`
- [ ] Run `make test`
- [ ] Run `make test-long`
- [ ] Run `make lint`
- [ ] Update this task file with completed checkboxes only after the work and every required gate actually pass.
- [ ] Only after all required checkboxes are complete, set `<passes>true</passes>`
- [ ] Run `/bin/bash .ralph/task_switch.sh`
- [ ] Commit all required files, including `.ralph/` updates, with a task-finished commit message that includes verification evidence
- [ ] Push with `git push`

TO BE VERIFIED
