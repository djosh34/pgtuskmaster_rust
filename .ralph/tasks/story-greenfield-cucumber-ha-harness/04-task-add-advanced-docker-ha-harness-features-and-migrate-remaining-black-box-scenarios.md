## Task: Add Advanced Docker HA Harness Features And Migrate Remaining Black-Box Scenarios <status>not_started</status> <passes>false</passes>

<priority>high</priority>

<description>
Add the advanced greenfield Docker HA harness capabilities required for the remaining black-box scenarios that can still be tested by running real `pgtuskmaster` binaries and controlling them externally. This task contains the exact advanced harness requirements and the exact scenario contracts.

It is explicitly not a requirement that every advanced scenario pass against the product before this task is considered complete. The requirement is that every advanced scenario is created and executable on the greenfield harness, and that each run produces enough evidence to show whether a failure is a real HA behavior failure in the system under test rather than a harness failure.

Any trustworthy HA or product failure exposed by these advanced feature runs must create a bug task with add-bug, and that bug task must contain `<blocked_by>` tags for every task in `story-greenfield-cucumber-ha-harness`.
Another explicit requirement, is that the tests must (just like before), be able to succesfully executed in parallel.
Serial execution of tests, is a failure of this task.

HARD REQUIREMENT: DO NOT SOLVE ANY TEST FAILURES THAT ARE IN `src/`, instead create bug tasks using add-bug, blocked by this story.
Any attempt of solving bugs outside the scope of this harness are STRICTLY FORBIDDEN!
JUST RUN ALL TESTS IN PARALLEL, GATHER FAILURES, ADD BUGS, AND MOVE ON!

Advanced harness capabilities required in this task:
- full 1:2 network partition control
- path-specific network isolation for etcd, API, and postgres/replication traffic
- DCS quorum loss and restore control
- concurrent SQL workload generation with recorded commit and rejection outcomes
- deterministic blockers for `pg_basebackup`, `pg_rewind`, broken startup, and broken rejoin flows
- degraded / lagging / ineligible replica shaping
- storage or WAL stall style wedging of a primary
- checked-in given variants when a scenario needs non-default node configuration

Each scenario below is one feature, one `.feature` file, and one tiny Rust wrapper.

**Scenario contracts**

1. `stress_planned_switchover_concurrent_sql`
- Start `three_node_plain`.
- Wait for exactly one stable primary.
- Create one workload table for this feature.
- Start a bounded concurrent write workload and record commit outcomes.
- While the workload is active, run `pgtm switchover request`.
- Wait for exactly one different primary to stabilize.
- Verify there is no dual-primary evidence during the transition window.
- Stop the workload and verify it committed at least one row.
- Insert proof row `post-switchover-proof` through the final primary.
- Verify table-key integrity and final convergence on all nodes.

2. `stress_failover_concurrent_sql`
- Start `three_node_plain`.
- Wait for exactly one stable primary.
- Create one workload table for this feature.
- Start a bounded concurrent write workload and record commit outcomes.
- While the workload is active, kill or wedge the current primary using the greenfield fault model.
- Wait for exactly one different primary to stabilize.
- Verify there is no dual-primary evidence and no split-brain write evidence during the transition window.
- Stop the workload and verify it committed at least one row.
- Insert proof row `post-failover-proof` through the final primary.
- Verify table-key integrity and final convergence on all nodes.

3. `targeted_switchover_rejects_ineligible_member`
- Start `three_node_plain`.
- Wait for exactly one stable primary.
- Choose one replica as `ineligible_replica`.
- Use the advanced degraded-member machinery to make `ineligible_replica` ineligible for promotion.
- Run `pgtm switchover request --switchover-to <ineligible_replica>`.
- Verify the request is rejected with an operator-visible error.
- Verify the current primary does not change.
- Heal `ineligible_replica`.
- Insert proof row `after-rejected-targeted-switchover` through the unchanged primary.
- Verify final convergence on all nodes.

4. `custom_postgres_roles_survive_failover_and_rejoin`
- Start a checked-in given that uses non-default replicator and rewinder role names.
- Wait for exactly one stable primary.
- Create a proof table for this feature.
- Insert proof row `1:before-custom-role-failover`.
- Force failover away from the initial primary.
- Wait for exactly one different primary.
- Insert proof row `2:after-custom-role-failover`.
- Wait for the old primary to rejoin as a replica under the custom-role configuration.
- Verify final convergence on exactly:
- `1:before-custom-role-failover`
- `2:after-custom-role-failover`

5. `clone_failure_recovers_after_blocker_removed`
- Start a cluster where one chosen node has a controllable `pg_basebackup` blocker.
- Wait for exactly one stable primary.
- Create a proof table for this feature.
- Insert proof row `1:before-clone-failure`.
- Force the blocked node onto a fresh clone path by wiping its data and enabling the blocker.
- Insert proof row `2:during-clone-failure` while the blocked node is still broken.
- Verify the blocked node is not queryable and is never primary during the fault window.
- Remove the blocker and restart that node.
- Wait for that node to rejoin as a replica.
- Verify final convergence on exactly:
- `1:before-clone-failure`
- `2:during-clone-failure`

6. `rewind_failure_falls_back_to_basebackup`
- Start a cluster where the initial primary has a controllable `pg_rewind` blocker.
- Wait for exactly one stable primary.
- Create a proof table for this feature.
- Insert proof row `1:before-rewind-failure`.
- Force failover away from the initial primary.
- Wait for exactly one different primary.
- Insert proof row `2:after-failover`.
- Verify the old primary cannot complete `pg_rewind`.
- Verify the old primary still rejoins through the fallback recovery path and only as a replica.
- Verify final convergence on exactly:
- `1:before-rewind-failure`
- `2:after-failover`

7. `repeated_leadership_changes_preserve_single_primary`
- Start `three_node_plain`.
- Wait for exactly one stable primary and record it as `primary_a`.
- Force failover away from `primary_a`.
- Wait for exactly one different primary and record it as `primary_b`.
- Force failover away from `primary_b`.
- Wait for exactly one different primary and record it as `primary_c`.
- Verify `primary_a`, `primary_b`, and `primary_c` are distinct if the topology allows that sequence.
- Verify there is never a dual-primary observation across the full churn window.

8. `lagging_replica_is_not_promoted`
- Start `three_node_plain`.
- Wait for exactly one stable primary.
- Choose one replica as `degraded_replica`.
- Choose the other replica as `healthy_replica`.
- Use the advanced lag or staleness machinery to make `degraded_replica` observably worse than `healthy_replica`.
- Create a proof table for this feature.
- Insert proof row `1:before-lagging-failover`.
- Force primary failure.
- Verify `healthy_replica` becomes the only primary.
- Verify `degraded_replica` does not become primary during the failover window.
- Insert proof row `2:after-lagging-failover` through `healthy_replica`.
- Heal the degraded replica if needed.
- Verify final convergence on exactly:
- `1:before-lagging-failover`
- `2:after-lagging-failover`

9. `no_quorum_enters_failsafe`
- Start `three_node_plain`.
- Wait for exactly one stable primary.
- Stop a DCS quorum majority.
- Verify all nodes lose quorum.
- Verify no operator-visible primary remains.
- Verify the cluster enters fail-safe behavior instead of silently keeping a writable primary.
- Verify there is no dual-primary evidence during the no-quorum window.

10. `no_quorum_fencing_blocks_post_cutoff_commits`
- Start `three_node_plain`.
- Wait for exactly one stable primary.
- Create one workload table for this feature.
- Start a bounded concurrent write workload and record commit timing or equivalent cutoff evidence.
- Stop a DCS quorum majority while the workload is active.
- Determine the fail-safe cutoff from the recorded workload evidence.
- Verify post-cutoff commits are rejected or bounded according to the fencing contract.
- Restore DCS quorum.
- Wait for exactly one stable primary again.
- Verify recovered table integrity against the allowed pre-cutoff commit set.

11. `full_partition_majority_survives_old_primary_isolated`
- Start `three_node_plain`.
- Wait for exactly one stable primary.
- Make the current primary the node that will become the 1-side minority.
- Create a proof table for this feature.
- Insert proof row `1:before-primary-minority-partition`.
- Partition that old primary from the other two nodes across etcd, API, and postgres/replication paths together.
- Verify the majority side elects exactly one primary before heal.
- Verify the minority old primary is not an accepted primary outcome during the partition window.
- Insert proof row `2:on-majority-during-partition` through the majority primary.
- Heal the partition.
- Verify the old minority primary rejoins as a replica.
- Verify final convergence on exactly:
- `1:before-primary-minority-partition`
- `2:on-majority-during-partition`

12. `full_partition_majority_survives_old_replica_isolated`
- Start `three_node_plain`.
- Wait for exactly one stable primary.
- Choose one replica as the minority-isolated node.
- Create a proof table for this feature.
- Insert proof row `1:before-replica-minority-partition`.
- Partition that replica from the two-node majority across etcd, API, and postgres/replication paths together.
- Verify the majority side preserves or converges to exactly one primary before heal.
- Verify the isolated replica does not self-promote.
- Insert proof row `2:on-majority-during-replica-partition` through the majority primary.
- Heal the partition.
- Verify final convergence on exactly:
- `1:before-replica-minority-partition`
- `2:on-majority-during-replica-partition`

13. `minority_old_primary_rejoins_safely_after_majority_failover`
- Start `three_node_plain`.
- Make the current primary the 1-side minority in a full partition.
- Create a proof table for this feature.
- Insert proof row `1:before-minority-old-primary-return`.
- Hold the partition until the majority elects a new primary.
- Insert proof row `2:on-majority-after-failover` through that new majority primary.
- Heal the partition.
- Verify the old minority primary does not remain or become primary after reconnect.
- Verify the old minority primary rejoins safely as a replica.
- Verify final convergence on exactly:
- `1:before-minority-old-primary-return`
- `2:on-majority-after-failover`

14. `api_path_isolation_preserves_primary`
- Start `three_node_plain`.
- Wait for exactly one stable primary.
- Choose one non-primary node for API-only isolation.
- Apply API-only isolation to that node.
- Verify direct API observation to that node fails during the fault window.
- Verify the original primary remains the only primary throughout the isolation window.
- Heal the API path.
- Insert proof row `1:after-api-path-heal`.
- Verify final convergence on all nodes.

15. `postgres_path_isolation_replicas_catch_up_after_heal`
- Start `three_node_plain`.
- Wait for exactly one stable primary.
- Create a proof table for this feature.
- Insert proof row `1:before-postgres-path-isolation`.
- Apply postgres or replication path isolation from the primary to the replicas without removing the primary itself.
- Insert proof row `2:during-postgres-path-isolation` on the primary.
- Verify the replicas do not contain `2:during-postgres-path-isolation` during the fault window.
- Heal the postgres path.
- Verify the replicas catch up.
- Verify final convergence on exactly:
- `1:before-postgres-path-isolation`
- `2:during-postgres-path-isolation`

16. `mixed_network_faults_heal_converges`
- Start `three_node_plain`.
- Wait for exactly one stable primary.
- Create a proof table for this feature.
- Insert proof row `1:before-mixed-faults`.
- Apply a mixed fault: isolate the old primary from etcd and isolate a different node on the API path.
- Verify the old primary enters fail-safe or loses authority safely.
- Verify there is no dual-primary window.
- Heal all network faults.
- Wait for exactly one stable primary.
- Insert proof row `2:after-mixed-fault-heal`.
- Verify final convergence on exactly:
- `1:before-mixed-faults`
- `2:after-mixed-fault-heal`

17. `primary_storage_stall_replaced_by_new_primary`
- Start `three_node_plain`.
- Wait for exactly one stable primary.
- Create a proof table for this feature.
- Insert proof row `1:before-storage-stall`.
- Inject the advanced storage or WAL stall fault into the current primary so the node is wedged rather than cleanly dead.
- Verify the old primary is no longer a usable writable primary.
- Wait for a different node to become the only primary.
- Insert proof row `2:after-storage-stall-failover` through the new primary.
- Verify the wedged old primary does not remain or become primary.
- Heal or recover the old primary as appropriate.
- Verify final convergence on exactly:
- `1:before-storage-stall`
- `2:after-storage-stall-failover`

18. `two_node_loss_one_good_return_one_broken_return_recovers_service`
- Start `three_node_plain`.
- Wait for exactly one stable primary.
- Create a proof table for this feature.
- Insert proof row `1:before-two-node-loss`.
- Stop two node containers and leave one survivor.
- Verify the lone survivor has no valid operator-visible primary outcome.
- Restart one stopped node normally.
- Keep the other stopped node broken with the advanced startup or recovery blocker.
- Wait for exactly one primary across the healthy pair.
- Insert proof row `2:after-good-return-before-broken-return-fix`.
- Verify the broken node does not block service restoration.
- Optionally heal the broken node.
- Verify final convergence on at least:
- `1:before-two-node-loss`
- `2:after-good-return-before-broken-return-fix`

19. `broken_replica_rejoin_does_not_block_healthy_quorum`
- Start `three_node_plain`.
- Reach a state with one healthy primary and one node attempting rejoin.
- Create a proof table for this feature.
- Insert proof row `1:before-broken-rejoin`.
- Trigger a broken rejoin attempt for the chosen node using the advanced startup or recovery blocker.
- While the broken rejoin attempt is active, insert proof row `2:during-broken-rejoin` through the healthy primary.
- Verify the healthy primary stays stable and unique.
- Verify the broken node never appears as primary during the broken rejoin window.
- Optionally heal the broken node.
- Verify final convergence on at least:
- `1:before-broken-rejoin`
- `2:during-broken-rejoin`
</description>

<acceptance_criteria>
- [ ] The greenfield harness gains all advanced capabilities listed in this task.
- [ ] One feature directory, one `.feature` file, one tiny wrapper, and one explicit `[[test]]` target exist for each of the 19 scenarios in this task.
- [ ] Every advanced feature implements the exact scenario contract written in this task and does not silently substitute a different story.
- [ ] Every advanced feature uses the greenfield Docker harness and does not import or call the legacy `tests/ha` or `src/test_harness/ha_e2e` code.
- [ ] Required checked-in given variants exist for scenarios that need non-default configuration.
- [ ] All advanced feature wrappers can be executed on the greenfield harness.
- [ ] Each advanced feature run produces enough evidence to distinguish a harness failure from an HA behavior failure in the system under test.
- [ ] If a scenario fails, the failure is captured after the harness has successfully applied the intended setup and fault choreography, so the failure is attributable to product behavior rather than harness breakage.
- [ ] Every trustworthy HA or product failure found while running these advanced features creates a bug task with add-bug with `<blocked_by>` tags for:
- [ ] `.ralph/tasks/story-greenfield-cucumber-ha-harness/01-task-build-independent-cucumber-docker-ha-harness-and-primary-crash-rejoin.md`
- [ ] `.ralph/tasks/story-greenfield-cucumber-ha-harness/02-task-add-low-hanging-ha-quorum-and-switchover-cucumber-features-on-greenfield-runner.md`
- [ ] `.ralph/tasks/story-greenfield-cucumber-ha-harness/03-task-deep-clean-legacy-black-box-test-infrastructure-after-greenfield-migration.md`
- [ ] `.ralph/tasks/story-greenfield-cucumber-ha-harness/04-task-add-advanced-docker-ha-harness-features-and-migrate-remaining-black-box-scenarios.md`
- [ ] `<passes>true</passes>` is set only after every acceptance criterion and required checkbox is complete.
</acceptance_criteria>

## Detailed implementation plan

### Phase 1: Add the advanced harness capabilities
- [ ] Add full 1:2 partition control.
- [ ] Add path-specific isolation for etcd, API, and postgres or replication traffic.
- [ ] Add DCS quorum loss and restore control.
- [ ] Add concurrent SQL workload generation and commit or rejection telemetry.
- [ ] Add deterministic blockers for `pg_basebackup`, `pg_rewind`, broken startup, and broken rejoin.
- [ ] Add degraded, lagging, or ineligible replica shaping.
- [ ] Add storage or WAL stall injection that wedges a primary.
- [ ] Add checked-in given variants needed by these scenarios.

### Phase 2: Add the 19 feature directories and wrappers
- [ ] Add one feature directory, `.feature` file, wrapper `.rs` file, and explicit `[[test]]` target for each scenario in this task.

### Phase 3: Implement the exact scenario contracts
- [ ] Implement `stress_planned_switchover_concurrent_sql` exactly as written.
- [ ] Implement `stress_failover_concurrent_sql` exactly as written.
- [ ] Implement `targeted_switchover_rejects_ineligible_member` exactly as written.
- [ ] Implement `custom_postgres_roles_survive_failover_and_rejoin` exactly as written.
- [ ] Implement `clone_failure_recovers_after_blocker_removed` exactly as written.
- [ ] Implement `rewind_failure_falls_back_to_basebackup` exactly as written.
- [ ] Implement `repeated_leadership_changes_preserve_single_primary` exactly as written.
- [ ] Implement `lagging_replica_is_not_promoted` exactly as written.
- [ ] Implement `no_quorum_enters_failsafe` exactly as written.
- [ ] Implement `no_quorum_fencing_blocks_post_cutoff_commits` exactly as written.
- [ ] Implement `full_partition_majority_survives_old_primary_isolated` exactly as written.
- [ ] Implement `full_partition_majority_survives_old_replica_isolated` exactly as written.
- [ ] Implement `minority_old_primary_rejoins_safely_after_majority_failover` exactly as written.
- [ ] Implement `api_path_isolation_preserves_primary` exactly as written.
- [ ] Implement `postgres_path_isolation_replicas_catch_up_after_heal` exactly as written.
- [ ] Implement `mixed_network_faults_heal_converges` exactly as written.
- [ ] Implement `primary_storage_stall_replaced_by_new_primary` exactly as written.
- [ ] Implement `two_node_loss_one_good_return_one_broken_return_recovers_service` exactly as written.
- [ ] Implement `broken_replica_rejoin_does_not_block_healthy_quorum` exactly as written.

### Phase 4: Verification and closeout
- [ ] Run targeted execution for the advanced feature wrappers.
- [ ] For each advanced wrapper run, record whether the result is:
- [ ] harness failure
- [ ] product or HA scenario failure
- [ ] successful scenario pass
- [ ] Fix harness failures until every advanced feature can be executed to a trustworthy outcome.
- [ ] For every trustworthy product or HA scenario failure, create a bug task with add-bug and add `<blocked_by>` tags for every task in this story.
- [ ] Do not leave scenarios uncreated just because they currently expose product bugs.
- [ ] Update this task file only after the work and verification are actually complete.
- [ ] Only after all required checkboxes are complete, set `<passes>true</passes>`.
- [ ] Run `/bin/bash .ralph/task_switch.sh`.
- [ ] Commit all required files, including `.ralph/` updates, with a task-finished commit message that includes verification evidence.
- [ ] Push with `git push`.

TO BE VERIFIED
