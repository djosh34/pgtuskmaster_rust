# HA decision structure bundle

This file consolidates the HA acceptance feature corpus and the runtime code/state that drives bootstrap, normal operation, degraded operation, and recovery.

Rules used while assembling this bundle:
- All `tests/ha/features/**/*.feature` files are included in full.
- Runtime snippets come from production `src/` code only.
- Test modules and obvious test scaffolding are excluded.
- Every code block carries its source path.

## HA feature files included
- `tests/ha/features/ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins/ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins.feature`
- `tests/ha/features/ha_basebackup_clone_blocked_then_unblocked_replica_recovers/ha_basebackup_clone_blocked_then_unblocked_replica_recovers.feature`
- `tests/ha/features/ha_broken_replica_rejoin_attempt_does_not_destabilize_quorum/ha_broken_replica_rejoin_attempt_does_not_destabilize_quorum.feature`
- `tests/ha/features/ha_dcs_and_api_faults_then_healed_cluster_converges/ha_dcs_and_api_faults_then_healed_cluster_converges.feature`
- `tests/ha/features/ha_dcs_quorum_lost_enters_failsafe/ha_dcs_quorum_lost_enters_failsafe.feature`
- `tests/ha/features/ha_dcs_quorum_lost_fencing_blocks_post_cutoff_writes/ha_dcs_quorum_lost_fencing_blocks_post_cutoff_writes.feature`
- `tests/ha/features/ha_lagging_replica_is_not_promoted_during_failover/ha_lagging_replica_is_not_promoted_during_failover.feature`
- `tests/ha/features/ha_non_primary_api_isolated_primary_stays_primary/ha_non_primary_api_isolated_primary_stays_primary.feature`
- `tests/ha/features/ha_old_primary_partitioned_from_majority_majority_elects_new_primary/ha_old_primary_partitioned_from_majority_majority_elects_new_primary.feature`
- `tests/ha/features/ha_old_primary_partitioned_then_healed_rejoins_as_replica_after_majority_failover/ha_old_primary_partitioned_then_healed_rejoins_as_replica_after_majority_failover.feature`
- `tests/ha/features/ha_planned_switchover_changes_primary_cleanly/ha_planned_switchover_changes_primary_cleanly.feature`
- `tests/ha/features/ha_planned_switchover_with_concurrent_writes/ha_planned_switchover_with_concurrent_writes.feature`
- `tests/ha/features/ha_primary_killed_custom_roles_survive_rejoin/ha_primary_killed_custom_roles_survive_rejoin.feature`
- `tests/ha/features/ha_primary_killed_then_rejoins_as_replica/ha_primary_killed_then_rejoins_as_replica.feature`
- `tests/ha/features/ha_primary_killed_with_concurrent_writes/ha_primary_killed_with_concurrent_writes.feature`
- `tests/ha/features/ha_primary_storage_stalled_then_new_primary_takes_over/ha_primary_storage_stalled_then_new_primary_takes_over.feature`
- `tests/ha/features/ha_repeated_failovers_preserve_single_primary/ha_repeated_failovers_preserve_single_primary.feature`
- `tests/ha/features/ha_replica_flapped_primary_stays_primary/ha_replica_flapped_primary_stays_primary.feature`
- `tests/ha/features/ha_replica_partitioned_from_majority_primary_stays_primary/ha_replica_partitioned_from_majority_primary_stays_primary.feature`
- `tests/ha/features/ha_replica_stopped_primary_stays_primary/ha_replica_stopped_primary_stays_primary.feature`
- `tests/ha/features/ha_replication_path_isolated_then_healed_replicas_catch_up/ha_replication_path_isolated_then_healed_replicas_catch_up.feature`
- `tests/ha/features/ha_rewind_fails_then_basebackup_rejoins_old_primary/ha_rewind_fails_then_basebackup_rejoins_old_primary.feature`
- `tests/ha/features/ha_targeted_switchover_promotes_requested_replica/ha_targeted_switchover_promotes_requested_replica.feature`
- `tests/ha/features/ha_targeted_switchover_to_degraded_replica_is_rejected/ha_targeted_switchover_to_degraded_replica_is_rejected.feature`
- `tests/ha/features/ha_two_nodes_stopped_then_one_healthy_node_restarted_restores_service_while_other_stays_broken/ha_two_nodes_stopped_then_one_healthy_node_restarted_restores_service_while_other_stays_broken.feature`
- `tests/ha/features/ha_two_replicas_stopped_then_one_replica_restarted_restores_quorum/ha_two_replicas_stopped_then_one_replica_restarted_restores_quorum.feature`

## HA feature corpus

### Feature path: `tests/ha/features/ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins/ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins.feature`

```gherkin
Feature: ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins
  Scenario: the cluster comes back with two fixed nodes first, then converges after the final node returns
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "initial_primary"
    And I create a proof table for this feature
    And I insert proof row "1:before-full-cluster-outage" through "initial_primary"
    Then the 3 online nodes contain exactly the recorded proof rows
    When I kill all database nodes
    And I start only the fixed nodes "node-a" and "node-b"
    Then exactly one primary exists across 2 running nodes as "restored_primary"
    When I insert proof row "2:after-two-node-restore-before-final-node" through "restored_primary"
    Then the node named "node-c" remains offline
    When I restart the node named "node-c"
    Then the node named "node-c" rejoins as a replica
    And pgtm primary points to "restored_primary"
    And the 3 online nodes contain exactly the recorded proof rows

```

### Feature path: `tests/ha/features/ha_basebackup_clone_blocked_then_unblocked_replica_recovers/ha_basebackup_clone_blocked_then_unblocked_replica_recovers.feature`

```gherkin
Feature: ha_basebackup_clone_blocked_then_unblocked_replica_recovers
  Scenario: a blocked basebackup clone path recovers after the blocker is removed
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "initial_primary"
    And I choose one non-primary node as "blocked_node"
    And I create a proof table for this feature
    And I insert proof row "1:before-clone-failure" through "initial_primary"
    When I enable the "pg_basebackup" blocker on the node named "blocked_node"
    And I kill the node named "blocked_node"
    And I wipe the data directory on the node named "blocked_node"
    And I start tracking primary history
    And I restart the node named "blocked_node"
    And I insert proof row "2:during-clone-failure" through "initial_primary"
    Then the node named "blocked_node" is not queryable
    And the primary history never included "blocked_node"
    When I disable the "pg_basebackup" blocker on the node named "blocked_node"
    And I restart the node named "blocked_node"
    Then the node named "blocked_node" emitted blocker evidence for "pg_basebackup"
    And the node named "blocked_node" rejoins as a replica
    And the 3 online nodes contain exactly the recorded proof rows

```

### Feature path: `tests/ha/features/ha_broken_replica_rejoin_attempt_does_not_destabilize_quorum/ha_broken_replica_rejoin_attempt_does_not_destabilize_quorum.feature`

```gherkin
Feature: ha_broken_replica_rejoin_attempt_does_not_destabilize_quorum
  Scenario: a broken rejoin attempt does not destabilize the healthy primary
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "initial_primary"
    And I choose one non-primary node as "healthy_replica"
    And I record the remaining replica as "broken_replica"
    And I create a proof table for this feature
    And I insert proof row "1:before-broken-rejoin" through "initial_primary"
    When I kill the node named "broken_replica"
    And I record marker "broken_rejoin"
    And I enable the "rejoin" blocker on the node named "broken_replica"
    And I start the node named "broken_replica" but keep it marked unavailable
    And I insert proof row "2:during-broken-rejoin" through "initial_primary"
    Then the primary named "initial_primary" remains the only primary
    And the node named "broken_replica" never becomes primary after marker "broken_rejoin"
    When I disable the "rejoin" blocker on the node named "broken_replica"
    And I restart the node named "broken_replica"
    Then the 3 online nodes contain exactly the recorded proof rows

```

### Feature path: `tests/ha/features/ha_dcs_and_api_faults_then_healed_cluster_converges/ha_dcs_and_api_faults_then_healed_cluster_converges.feature`

```gherkin
Feature: ha_dcs_and_api_faults_then_healed_cluster_converges
  Scenario: combined dcs and api faults still converge safely after heal
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "initial_primary"
    And I choose one non-primary node as "api_isolated_node"
    And I create a proof table for this feature
    And I insert proof row "1:before-mixed-faults" through "initial_primary"
    When I cut the node named "initial_primary" off from DCS
    And I isolate the node named "api_isolated_node" from observer API access
    Then the node named "initial_primary" enters fail-safe or loses primary authority safely
    And there is no dual-primary evidence during the transition window
    When I heal all network faults
    Then exactly one primary exists across 3 running nodes as "final_primary"
    When I insert proof row "2:after-mixed-fault-heal" through "final_primary"
    Then the 3 online nodes contain exactly the recorded proof rows

```

### Feature path: `tests/ha/features/ha_dcs_quorum_lost_enters_failsafe/ha_dcs_quorum_lost_enters_failsafe.feature`

```gherkin
Feature: ha_dcs_quorum_lost_enters_failsafe
  Scenario: losing DCS quorum removes the operator-visible primary and exposes fail-safe behavior
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "initial_primary"
    When I stop a DCS quorum majority
    Then there is no operator-visible primary across 3 online node
    And every running node reports fail_safe in debug output
    And there is no dual-primary evidence during the transition window
    When I restore DCS quorum
    Then I wait for exactly one stable primary as "restored_primary"

```

### Feature path: `tests/ha/features/ha_dcs_quorum_lost_fencing_blocks_post_cutoff_writes/ha_dcs_quorum_lost_fencing_blocks_post_cutoff_writes.feature`

```gherkin
Feature: ha_dcs_quorum_lost_fencing_blocks_post_cutoff_writes
  Scenario: fail-safe fencing eventually rejects post-cutoff writes and preserves pre-cutoff commits
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "initial_primary"
    And I create one workload table for this feature
    When I start a bounded concurrent write workload and record commit outcomes
    And I stop a DCS quorum majority
    Then there is no operator-visible primary across 3 online node
    And every running node reports fail_safe in debug output
    When I restore DCS quorum
    Then I wait for exactly one stable primary as "restored_primary"
    When I stop the workload and verify it committed at least one row
    Then the recorded workload evidence establishes a fencing cutoff with no later commits
    And the 3 online nodes contain exactly the recorded proof rows

```

### Feature path: `tests/ha/features/ha_lagging_replica_is_not_promoted_during_failover/ha_lagging_replica_is_not_promoted_during_failover.feature`

```gherkin
Feature: ha_lagging_replica_is_not_promoted_during_failover
  Scenario: a degraded replica is not promoted during failover
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "old_primary"
    And I choose one non-primary node as "degraded_replica"
    And I record the remaining replica as "healthy_replica"
    And I create a proof table for this feature
    And I insert proof row "1:before-lagging-failover" through "old_primary"
    When I isolate the nodes named "old_primary" and "degraded_replica" on the "postgres" path
    And I start tracking primary history
    And I kill the node named "old_primary"
    Then exactly one primary exists across 2 running nodes as "healthy_replica"
    And the primary history never included "degraded_replica"
    When I insert proof row "2:after-lagging-failover" through "healthy_replica"
    And I heal all network faults
    And I restart the node named "old_primary"
    Then the node named "old_primary" rejoins as a replica
    And the 3 online nodes contain exactly the recorded proof rows

```

### Feature path: `tests/ha/features/ha_non_primary_api_isolated_primary_stays_primary/ha_non_primary_api_isolated_primary_stays_primary.feature`

```gherkin
Feature: ha_non_primary_api_isolated_primary_stays_primary
  Scenario: observer api isolation of a non-primary does not trigger a failover
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "initial_primary"
    And I choose one non-primary node as "api_isolated_node"
    When I isolate the node named "api_isolated_node" from observer API access
    Then direct API observation to "api_isolated_node" fails
    And the primary named "initial_primary" remains the only primary
    When I heal network faults on the node named "api_isolated_node"
    And I insert proof row "1:after-api-path-heal" through "initial_primary"
    Then the 3 online nodes contain exactly the recorded proof rows

```

### Feature path: `tests/ha/features/ha_old_primary_partitioned_from_majority_majority_elects_new_primary/ha_old_primary_partitioned_from_majority_majority_elects_new_primary.feature`

```gherkin
Feature: ha_old_primary_partitioned_from_majority_majority_elects_new_primary
  Scenario: a primary isolated into the minority is not accepted while the majority elects a new primary
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "old_primary"
    And I create a proof table for this feature
    And I insert proof row "1:before-primary-minority-partition" through "old_primary"
    When I start tracking primary history
    And I fully isolate the node named "old_primary" from the cluster
    Then exactly one primary exists across 2 running nodes as "majority_primary"
    And the primary history never included "old_primary"
    When I insert proof row "2:on-majority-during-partition" through "majority_primary"
    And I heal all network faults
    Then the node named "old_primary" rejoins as a replica
    And the 3 online nodes contain exactly the recorded proof rows

```

### Feature path: `tests/ha/features/ha_old_primary_partitioned_then_healed_rejoins_as_replica_after_majority_failover/ha_old_primary_partitioned_then_healed_rejoins_as_replica_after_majority_failover.feature`

```gherkin
Feature: ha_old_primary_partitioned_then_healed_rejoins_as_replica_after_majority_failover
  Scenario: an old primary isolated into the minority rejoins only as a replica after the majority fails over
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "old_primary"
    And I create a proof table for this feature
    And I insert proof row "1:before-minority-old-primary-return" through "old_primary"
    When I start tracking primary history
    And I fully isolate the node named "old_primary" from the cluster
    Then exactly one primary exists across 2 running nodes as "majority_primary"
    And the primary history never included "old_primary"
    When I insert proof row "2:on-majority-after-failover" through "majority_primary"
    And I start tracking primary history
    And I heal all network faults
    Then the node named "old_primary" rejoins as a replica
    And the primary history never included "old_primary"
    And the 3 online nodes contain exactly the recorded proof rows

```

### Feature path: `tests/ha/features/ha_planned_switchover_changes_primary_cleanly/ha_planned_switchover_changes_primary_cleanly.feature`

```gherkin
Feature: ha_planned_switchover_changes_primary_cleanly
  Scenario: a planned switchover moves leadership to a different primary
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "old_primary"
    And I create a proof table for this feature
    And I insert proof row "1:before-planned-switchover" through "old_primary"
    Then the 3 online nodes contain exactly the recorded proof rows
    And I record the current pgtm primary and replicas views
    When I request a planned switchover
    Then I wait for a different stable primary than "old_primary" as "new_primary"
    And the node named "old_primary" remains online as a replica
    And pgtm primary points to "new_primary"
    And pgtm replicas list every cluster member except "new_primary"
    When I insert proof row "2:after-planned-switchover" through "new_primary"
    Then the 3 online nodes contain exactly the recorded proof rows

```

### Feature path: `tests/ha/features/ha_planned_switchover_with_concurrent_writes/ha_planned_switchover_with_concurrent_writes.feature`

```gherkin
Feature: ha_planned_switchover_with_concurrent_writes
  Scenario: a planned switchover preserves single-primary behavior under concurrent writes
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "old_primary"
    And I create one workload table for this feature
    When I start a bounded concurrent write workload and record commit outcomes
    And I request a planned switchover
    Then I wait for a different stable primary than "old_primary" as "new_primary"
    And there is no dual-primary evidence during the transition window
    When I stop the workload and verify it committed at least one row
    And I insert proof row "post-switchover-proof" through "new_primary"
    Then the 3 online nodes contain exactly the recorded proof rows

```

### Feature path: `tests/ha/features/ha_primary_killed_custom_roles_survive_rejoin/ha_primary_killed_custom_roles_survive_rejoin.feature`

```gherkin
Feature: ha_primary_killed_custom_roles_survive_rejoin
  Scenario: non-default replicator and rewinder roles survive failover and rejoin
    Given the "three_node_custom_roles" harness is running
    And I wait for exactly one stable primary as "old_primary"
    And I create a proof table for this feature
    And I insert proof row "1:before-custom-role-failover" through "old_primary"
    When I kill the node named "old_primary"
    Then exactly one primary exists across 2 running nodes as "new_primary"
    When I insert proof row "2:after-custom-role-failover" through "new_primary"
    And I restart the node named "old_primary"
    Then the node named "old_primary" rejoins as a replica
    And the 3 online nodes contain exactly the recorded proof rows

```

### Feature path: `tests/ha/features/ha_primary_killed_then_rejoins_as_replica/ha_primary_killed_then_rejoins_as_replica.feature`

```gherkin
Feature: ha_primary_killed_then_rejoins_as_replica
  Scenario: a killed primary fails over and later rejoins as a replica
    Given the "three_node_plain" harness is running
    And the cluster reaches one stable primary
    When the current primary container crashes
    Then after the configured HA lease deadline a different node becomes the only primary
    And I can write a proof row through the new primary
    When I start the killed node container again
    Then after the configured recovery deadline the restarted node rejoins as a replica
    And the proof row is visible from the restarted node
    And the cluster still has exactly one primary

```

### Feature path: `tests/ha/features/ha_primary_killed_with_concurrent_writes/ha_primary_killed_with_concurrent_writes.feature`

```gherkin
Feature: ha_primary_killed_with_concurrent_writes
  Scenario: a forced failover preserves single-primary behavior under concurrent writes
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "old_primary"
    And I create one workload table for this feature
    When I start a bounded concurrent write workload and record commit outcomes
    And I kill the node named "old_primary"
    Then exactly one primary exists across 2 running nodes as "new_primary"
    When I stop the workload and verify it committed at least one row
    Then there is no dual-primary evidence and no split-brain write evidence during the transition window
    And I insert proof row "post-failover-proof" through "new_primary"
    Then the 2 online nodes contain exactly the recorded proof rows
    When I restart the node named "old_primary"
    Then the node named "old_primary" rejoins as a replica
    And the 3 online nodes contain exactly the recorded proof rows

```

### Feature path: `tests/ha/features/ha_primary_storage_stalled_then_new_primary_takes_over/ha_primary_storage_stalled_then_new_primary_takes_over.feature`

```gherkin
Feature: ha_primary_storage_stalled_then_new_primary_takes_over
  Scenario: a wedged primary is replaced without becoming authoritative again
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "initial_primary"
    And I create a proof table for this feature
    And I insert proof row "1:before-storage-stall" through "initial_primary"
    And I record marker "storage_stall"
    When I wedge the node named "initial_primary"
    Then I wait for a different stable primary than "initial_primary" as "final_primary"
    And the node named "initial_primary" never becomes primary after marker "storage_stall"
    And there is no dual-primary evidence during the transition window
    When I insert proof row "2:after-storage-stall-failover" through "final_primary"
    And I unwedge the node named "initial_primary"
    Then the node named "initial_primary" rejoins as a replica
    And the 3 online nodes contain exactly the recorded proof rows

```

### Feature path: `tests/ha/features/ha_repeated_failovers_preserve_single_primary/ha_repeated_failovers_preserve_single_primary.feature`

```gherkin
Feature: ha_repeated_failovers_preserve_single_primary
  Scenario: repeated failovers preserve a single primary and distinct leaders when topology allows
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "primary_a"
    And I start tracking primary history
    When I kill the node named "primary_a"
    Then exactly one primary exists across 2 running nodes as "primary_b"
    And the primary history never included "primary_a"
    When I restart the node named "primary_a"
    And the node named "primary_a" rejoins as a replica
    And I cut the node named "primary_a" off from DCS
    And I start tracking primary history
    When I kill the node named "primary_b"
    Then exactly one primary exists across 2 running nodes as "primary_c"
    And the primary history never included "primary_b"
    Then the aliases "primary_a", "primary_b", and "primary_c" are distinct
    And there is no dual-primary evidence during the transition window

```

### Feature path: `tests/ha/features/ha_replica_flapped_primary_stays_primary/ha_replica_flapped_primary_stays_primary.feature`

```gherkin
Feature: ha_replica_flapped_primary_stays_primary
  Scenario: repeatedly flapping a replica keeps the same primary
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "initial_primary"
    And I choose one non-primary node as "flapping_replica"
    And I create a proof table for this feature
    And I insert proof row "1:before-flap" through "initial_primary"
    Then the 3 online nodes contain exactly the recorded proof rows
    When I kill the node named "flapping_replica"
    Then the primary named "initial_primary" remains the only primary
    When I insert proof row "2:during-flap-cycle-1" through "initial_primary"
    And I restart the node named "flapping_replica"
    Then the node named "flapping_replica" rejoins as a replica
    When I kill the node named "flapping_replica"
    Then the primary named "initial_primary" remains the only primary
    When I insert proof row "3:during-flap-cycle-2" through "initial_primary"
    And I restart the node named "flapping_replica"
    Then the node named "flapping_replica" rejoins as a replica
    When I kill the node named "flapping_replica"
    Then the primary named "initial_primary" remains the only primary
    When I insert proof row "4:during-flap-cycle-3" through "initial_primary"
    And I restart the node named "flapping_replica"
    Then the node named "flapping_replica" rejoins as a replica
    And the 3 online nodes contain exactly the recorded proof rows

```

### Feature path: `tests/ha/features/ha_replica_partitioned_from_majority_primary_stays_primary/ha_replica_partitioned_from_majority_primary_stays_primary.feature`

```gherkin
Feature: ha_replica_partitioned_from_majority_primary_stays_primary
  Scenario: an isolated replica does not self-promote while the majority preserves a single primary
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "stable_primary"
    And I choose one non-primary node as "isolated_replica"
    And I create a proof table for this feature
    And I insert proof row "1:before-replica-minority-partition" through "stable_primary"
    When I start tracking primary history
    And I fully isolate the node named "isolated_replica" from the cluster
    Then exactly one primary exists across 2 running nodes as "majority_primary"
    And the primary history never included "isolated_replica"
    When I insert proof row "2:on-majority-during-replica-partition" through "majority_primary"
    And I heal all network faults
    Then the 3 online nodes contain exactly the recorded proof rows

```

### Feature path: `tests/ha/features/ha_replica_stopped_primary_stays_primary/ha_replica_stopped_primary_stays_primary.feature`

```gherkin
Feature: ha_replica_stopped_primary_stays_primary
  Scenario: a replica outage keeps the current primary stable
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "initial_primary"
    And I choose one non-primary node as "stopped_replica"
    And I create a proof table for this feature
    And I insert proof row "1:before-replica-outage" through "initial_primary"
    Then the 3 online nodes contain exactly the recorded proof rows
    When I kill the node named "stopped_replica"
    Then pgtm primary points to "initial_primary"
    And the primary named "initial_primary" remains the only primary
    And the remaining online non-primary node is a replica
    When I insert proof row "2:during-replica-outage" through "initial_primary"
    And I restart the node named "stopped_replica"
    Then the node named "stopped_replica" rejoins as a replica
    And the 3 online nodes contain exactly the recorded proof rows

```

### Feature path: `tests/ha/features/ha_replication_path_isolated_then_healed_replicas_catch_up/ha_replication_path_isolated_then_healed_replicas_catch_up.feature`

```gherkin
Feature: ha_replication_path_isolated_then_healed_replicas_catch_up
  Scenario: replicas lag during replication-path isolation and catch up after heal
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "initial_primary"
    And I choose the two non-primary nodes as "replica_a" and "replica_b"
    And I create a proof table for this feature
    And I insert proof row "1:before-postgres-path-isolation" through "initial_primary"
    When I isolate the nodes named "initial_primary" and "replica_a" on the "postgres" path
    And I isolate the nodes named "initial_primary" and "replica_b" on the "postgres" path
    And I insert proof row "2:during-postgres-path-isolation" through "initial_primary"
    Then the nodes named "replica_a" and "replica_b" do not yet contain proof row "2:during-postgres-path-isolation"
    When I heal all network faults
    Then the 3 online nodes contain exactly the recorded proof rows

```

### Feature path: `tests/ha/features/ha_rewind_fails_then_basebackup_rejoins_old_primary/ha_rewind_fails_then_basebackup_rejoins_old_primary.feature`

```gherkin
Feature: ha_rewind_fails_then_basebackup_rejoins_old_primary
  Scenario: a rewind failure still allows the old primary to rejoin as a replica
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "old_primary"
    And I enable the "pg_rewind" blocker on the node named "old_primary"
    And I create a proof table for this feature
    And I insert proof row "1:before-rewind-failure" through "old_primary"
    When I fully isolate the node named "old_primary" from the cluster
    And I cut the node named "old_primary" off from DCS
    Then exactly one primary exists across 2 running nodes as "new_primary"
    When I insert proof row "2:after-failover" through "new_primary"
    And I heal network faults on the node named "old_primary"
    Then the node named "old_primary" emitted blocker evidence for "pg_rewind"
    And the node named "old_primary" rejoins as a replica
    And the 3 online nodes contain exactly the recorded proof rows

```

### Feature path: `tests/ha/features/ha_targeted_switchover_promotes_requested_replica/ha_targeted_switchover_promotes_requested_replica.feature`

```gherkin
Feature: ha_targeted_switchover_promotes_requested_replica
  Scenario: a targeted switchover promotes the chosen replica and not the other one
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "old_primary"
    And I choose one non-primary node as "target_replica"
    And I record the remaining replica as "other_replica"
    And I create a proof table for this feature
    And I insert proof row "1:before-targeted-switchover" through "old_primary"
    Then the 3 online nodes contain exactly the recorded proof rows
    When I request a targeted switchover to "target_replica"
    Then I wait for the primary named "target_replica" to become the only primary
    And the primary history never included "other_replica"
    And the node named "old_primary" remains online as a replica
    And pgtm primary points to "target_replica"
    And pgtm replicas list every cluster member except "target_replica"
    When I insert proof row "2:after-targeted-switchover" through "target_replica"
    Then the 3 online nodes contain exactly the recorded proof rows

```

### Feature path: `tests/ha/features/ha_targeted_switchover_to_degraded_replica_is_rejected/ha_targeted_switchover_to_degraded_replica_is_rejected.feature`

```gherkin
Feature: ha_targeted_switchover_to_degraded_replica_is_rejected
  Scenario: a targeted switchover request to a degraded replica is rejected
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "initial_primary"
    And I choose one non-primary node as "ineligible_replica"
    And I create a proof table for this feature
    When I fully isolate the node named "ineligible_replica" from the cluster
    And I attempt a targeted switchover to "ineligible_replica" and capture the operator-visible error
    Then the last operator-visible error is recorded
    And the primary named "initial_primary" remains the only primary
    When I heal all network faults
    And I insert proof row "after-rejected-targeted-switchover" through "initial_primary"
    Then the 3 online nodes contain exactly the recorded proof rows

```

### Feature path: `tests/ha/features/ha_two_nodes_stopped_then_one_healthy_node_restarted_restores_service_while_other_stays_broken/ha_two_nodes_stopped_then_one_healthy_node_restarted_restores_service_while_other_stays_broken.feature`

```gherkin
Feature: ha_two_nodes_stopped_then_one_healthy_node_restarted_restores_service_while_other_stays_broken
  Scenario: one healthy return restores service even while another node stays broken
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "initial_primary"
    And I choose the two non-primary nodes as "stopped_node_a" and "stopped_node_b"
    And I create a proof table for this feature
    And I insert proof row "1:before-two-node-loss" through "initial_primary"
    When I kill the nodes named "stopped_node_a" and "stopped_node_b"
    Then the lone online node is not treated as a writable primary
    When I restart the node named "stopped_node_a"
    And I enable the "startup" blocker on the node named "stopped_node_b"
    And I start the node named "stopped_node_b" but keep it marked unavailable
    Then exactly one primary exists across 2 running nodes as "restored_primary"
    When I insert proof row "2:after-good-return-before-broken-return-fix" through "restored_primary"
    Then the cluster is degraded but operational across 2 running nodes
    When I disable the "startup" blocker on the node named "stopped_node_b"
    And I restart the node named "stopped_node_b"
    Then the 3 online nodes contain exactly the recorded proof rows

```

### Feature path: `tests/ha/features/ha_two_replicas_stopped_then_one_replica_restarted_restores_quorum/ha_two_replicas_stopped_then_one_replica_restarted_restores_quorum.feature`

```gherkin
Feature: ha_two_replicas_stopped_then_one_replica_restarted_restores_quorum
  Scenario: two replicas stop, then one returns and restores quorum
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "initial_primary"
    And I choose the two non-primary nodes as "stopped_node_a" and "stopped_node_b"
    And I create a proof table for this feature
    And I insert proof row "1:before-two-node-outage" through "initial_primary"
    Then the 3 online nodes contain exactly the recorded proof rows
    When I kill the nodes named "stopped_node_a" and "stopped_node_b"
    Then there is no operator-visible primary across 1 online node
    And the lone online node is not treated as a writable primary
    When I restart the node named "stopped_node_a"
    Then exactly one primary exists across 2 running nodes as "restored_primary"
    When I insert proof row "2:after-quorum-restore-before-full-heal" through "restored_primary"
    Then the cluster is degraded but operational across 2 running nodes
    When I restart the node named "stopped_node_b"
    Then the node named "stopped_node_b" rejoins as a replica
    And the 3 online nodes contain exactly the recorded proof rows

```

# Runtime wiring and startup

## Startup enums, runtime entry, and process defaults

Source path: `src/runtime/node.rs`

```rust
use std::{
    collections::BTreeMap,
    fs,
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use thiserror::Error;
use tokio::{net::TcpListener, sync::mpsc};
use tokio_postgres::NoTls;

use crate::{
    api::worker::ApiWorkerCtx,
    config::{
        load_runtime_config, resolve_secret_string, validate_runtime_config, ConfigError,
        RoleAuthConfig, RuntimeConfig,
    },
    dcs::{
        etcd_store::EtcdDcsStore,
        state::{DcsCache, DcsState, DcsTrust, DcsWorkerCtx, MemberRole},
        store::{refresh_from_etcd_watch, DcsStore},
    },
    debug_api::{
        snapshot::{build_snapshot, AppLifecycle, DebugSnapshotCtx},
        worker::{DebugApiContractStubInputs, DebugApiCtx},
    },
    ha::source_conn::{basebackup_resume_source_from_member, basebackup_source_from_member},
    ha::state::{
        HaPhase, HaState, HaWorkerContractStubInputs, HaWorkerCtx, ProcessDispatchDefaults,
    },
    logging::{
        AppEvent, AppEventHeader, SeverityText, StructuredFields, SubprocessLineRecord,
        SubprocessStream,
    },
    pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
    postgres_managed_conf::{managed_standby_auth_from_role_auth, ManagedPostgresStartIntent},
    process::{
        jobs::{
            BaseBackupSpec, BootstrapSpec, ProcessCommandRunner, ProcessExit, ReplicatorSourceConn,
            StartPostgresSpec,
        },
        state::{ProcessJobKind, ProcessState, ProcessWorkerCtx},
        worker::{
            build_command, start_postgres_preflight_is_already_running, system_now_unix_millis,
            timeout_for_kind, TokioCommandRunner,
        },
    },
    state::{new_state_channel, MemberId, UnixMillis, WorkerStatus},
};

const STARTUP_OUTPUT_DRAIN_MAX_BYTES: usize = 256 * 1024;
const STARTUP_JOB_POLL_INTERVAL: Duration = Duration::from_millis(20);
const PROCESS_WORKER_POLL_INTERVAL: Duration = Duration::from_millis(10);

#[derive(Clone, Debug)]
enum StartupAction {
    ClaimInitLockAndSeedConfig,
    RunJob(Box<ProcessJobKind>),
    StartPostgres(Box<ManagedPostgresStartIntent>),
    EnsureRequiredRoles,
}

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("config error: {0}")]
    Config(#[from] ConfigError),
    #[error("startup planning failed: {0}")]
    StartupPlanning(String),
    #[error("startup execution failed: {0}")]
    StartupExecution(String),
    #[error("api bind failed at `{listen_addr}`: {message}")]
    ApiBind {
        listen_addr: std::net::SocketAddr,
        message: String,
    },
    #[error("worker failed: {0}")]
    Worker(String),
    #[error("time error: {0}")]
    Time(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum StartupMode {
    InitializePrimary {
        start_intent: Box<ManagedPostgresStartIntent>,
    },
    CloneReplica {
        leader_member_id: MemberId,
        source: Box<ReplicatorSourceConn>,
        start_intent: Box<ManagedPostgresStartIntent>,
    },
    ResumeExisting {
        start_intent: Box<ManagedPostgresStartIntent>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DataDirState {
    Missing,
    Empty,
    Existing,
}

#[derive(Clone, Copy, Debug)]
enum RuntimeEventKind {
    StartupEntered,
    DataDirInspected,
    DcsCacheProbe,
    ModeSelected,
    ActionsPlanned,
    Action,
    Phase,
    SubprocessLogEmitFailed,
}

impl RuntimeEventKind {
    fn name(self) -> &'static str {
        match self {
            Self::StartupEntered => "runtime.startup.entered",
            Self::DataDirInspected => "runtime.startup.data_dir.inspected",
            Self::DcsCacheProbe => "runtime.startup.dcs_cache_probe",
            Self::ModeSelected => "runtime.startup.mode_selected",
            Self::ActionsPlanned => "runtime.startup.actions_planned",
            Self::Action => "runtime.startup.action",
            Self::Phase => "runtime.startup.phase",
            Self::SubprocessLogEmitFailed => "runtime.startup.subprocess_log_emit_failed",
        }
    }
}

fn runtime_event(
    kind: RuntimeEventKind,
    result: &str,
    severity: SeverityText,
    message: impl Into<String>,
) -> AppEvent {
    AppEvent::new(
        severity,
        message,
        AppEventHeader::new(kind.name(), "runtime", result),
    )
}

fn runtime_base_fields(cfg: &RuntimeConfig, startup_run_id: &str) -> StructuredFields {
    let mut fields = StructuredFields::new();
    fields.insert("scope", cfg.dcs.scope.clone());
    fields.insert("member_id", cfg.cluster.member_id.clone());
    fields.insert("startup_run_id", startup_run_id.to_string());
    fields
}

fn startup_mode_label(startup_mode: &StartupMode) -> String {
    format!("{startup_mode:?}").to_lowercase()
}

fn startup_action_kind_label(action: &StartupAction) -> &'static str {
    match action {
        StartupAction::ClaimInitLockAndSeedConfig => "claim_init_lock_and_seed_config",
        StartupAction::RunJob(_) => "run_job",
        StartupAction::StartPostgres(_) => "start_postgres",
        StartupAction::EnsureRequiredRoles => "ensure_required_roles",
    }
}

pub async fn run_node_from_config_path(path: &Path) -> Result<(), RuntimeError> {
    let cfg = load_runtime_config(path)?;
    run_node_from_config(cfg).await
}

pub async fn run_node_from_config(cfg: RuntimeConfig) -> Result<(), RuntimeError> {
    validate_runtime_config(&cfg)?;

    let logging = crate::logging::bootstrap(&cfg).map_err(|err| {
        RuntimeError::StartupExecution(format!("logging bootstrap failed: {err}"))
    })?;
    let log = logging.handle.clone();
    let startup_run_id = format!(
        "{}-{}",
        cfg.cluster.member_id,
        crate::logging::system_now_unix_millis()
    );
    let mut event = runtime_event(
        RuntimeEventKind::StartupEntered,
        "ok",
        SeverityText::Info,
        "runtime starting",
    );
    let fields = event.fields_mut();
    fields.append_json_map(runtime_base_fields(&cfg, startup_run_id.as_str()).into_attributes());
    fields.insert(
        "logging.level",
        format!("{:?}", cfg.logging.level).to_lowercase(),
    );
    log.emit_app_event("runtime::run_node_from_config", event)
        .map_err(|err| {
            RuntimeError::StartupExecution(format!("runtime start log emit failed: {err}"))
        })?;

    let process_defaults = process_defaults_from_config(&cfg);
    let startup_mode = plan_startup(&cfg, &process_defaults, &log, startup_run_id.as_str())?;
    execute_startup(
        &cfg,
        &process_defaults,
        &startup_mode,
        &log,
        startup_run_id.as_str(),
    )
    .await?;

    run_workers(cfg, process_defaults, log).await
}

fn process_defaults_from_config(cfg: &RuntimeConfig) -> ProcessDispatchDefaults {
    ProcessDispatchDefaults {
        postgres_host: cfg.postgres.listen_host.clone(),
        postgres_port: cfg.postgres.listen_port,
        socket_dir: cfg.postgres.socket_dir.clone(),
        log_file: cfg.postgres.log_file.clone(),
        replicator_username: cfg.postgres.roles.replicator.username.clone(),
        replicator_auth: cfg.postgres.roles.replicator.auth.clone(),
        rewinder_username: cfg.postgres.roles.rewinder.username.clone(),
        rewinder_auth: cfg.postgres.roles.rewinder.auth.clone(),
        remote_dbname: cfg.postgres.rewind_conn_identity.dbname.clone(),
        remote_ssl_mode: cfg.postgres.rewind_conn_identity.ssl_mode,
        remote_ssl_root_cert: cfg.postgres.rewind_conn_identity.ca_cert.clone(),
        connect_timeout_s: cfg.postgres.connect_timeout_s,
        shutdown_mode: crate::process::jobs::ShutdownMode::Fast,
    }
}

fn advertised_postgres_port(cfg: &RuntimeConfig) -> u16 {
    cfg.postgres
        .advertise_port
        .unwrap_or(cfg.postgres.listen_port)
}

fn plan_startup(
    cfg: &RuntimeConfig,
    process_defaults: &ProcessDispatchDefaults,
    log: &crate::logging::LogHandle,
    startup_run_id: &str,
) -> Result<StartupMode, RuntimeError> {
    plan_startup_with_probe(cfg, process_defaults, log, startup_run_id, probe_dcs_cache)
}

```

## Startup planning: data-dir inspection, DCS probe, and initial mode selection wrapper

Source path: `src/runtime/node.rs`

```rust
fn plan_startup_with_probe(
    cfg: &RuntimeConfig,
    process_defaults: &ProcessDispatchDefaults,
    log: &crate::logging::LogHandle,
    startup_run_id: &str,
    probe: impl Fn(&RuntimeConfig) -> Result<DcsCache, RuntimeError>,
) -> Result<StartupMode, RuntimeError> {
    let data_dir_state = match inspect_data_dir(&cfg.postgres.data_dir) {
        Ok(value) => {
            let mut event = runtime_event(
                RuntimeEventKind::DataDirInspected,
                "ok",
                SeverityText::Debug,
                "data dir inspected",
            );
            let fields = event.fields_mut();
            fields.append_json_map(runtime_base_fields(cfg, startup_run_id).into_attributes());
            fields.insert(
                "postgres.data_dir",
                cfg.postgres.data_dir.display().to_string(),
            );
            fields.insert("data_dir_state", format!("{value:?}").to_lowercase());
            log.emit_app_event("runtime::plan_startup", event)
                .map_err(|err| {
                    RuntimeError::StartupPlanning(format!(
                        "data dir inspection log emit failed: {err}"
                    ))
                })?;
            value
        }
        Err(err) => {
            let mut event = runtime_event(
                RuntimeEventKind::DataDirInspected,
                "failed",
                SeverityText::Error,
                "data dir inspection failed",
            );
            let fields = event.fields_mut();
            fields.append_json_map(runtime_base_fields(cfg, startup_run_id).into_attributes());
            fields.insert(
                "postgres.data_dir",
                cfg.postgres.data_dir.display().to_string(),
            );
            fields.insert("error", err.to_string());
            log.emit_app_event("runtime::plan_startup", event)
                .map_err(|emit_err| {
                    RuntimeError::StartupPlanning(format!(
                        "data dir inspection log emit failed: {emit_err}"
                    ))
                })?;
            return Err(err);
        }
    };

    let cache = match probe(cfg) {
        Ok(cache) => {
            let mut event = runtime_event(
                RuntimeEventKind::DcsCacheProbe,
                "ok",
                SeverityText::Info,
                "startup dcs cache probe ok",
            );
            let fields = event.fields_mut();
            fields.append_json_map(runtime_base_fields(cfg, startup_run_id).into_attributes());
            fields.insert("dcs_probe_status", "ok");
            log.emit_app_event("runtime::plan_startup", event)
                .map_err(|err| {
                    RuntimeError::StartupPlanning(format!("dcs cache probe log emit failed: {err}"))
                })?;
            Some(cache)
        }
        Err(err) => {
            let mut event = runtime_event(
                RuntimeEventKind::DcsCacheProbe,
                "failed",
                SeverityText::Warn,
                "startup dcs cache probe failed; continuing without cache",
            );
            let fields = event.fields_mut();
            fields.append_json_map(runtime_base_fields(cfg, startup_run_id).into_attributes());
            fields.insert("error", err.to_string());
            fields.insert("dcs_probe_status", "failed");
            log.emit_app_event("runtime::plan_startup", event)
                .map_err(|emit_err| {
                    RuntimeError::StartupPlanning(format!(
                        "dcs cache probe log emit failed: {emit_err}"
                    ))
                })?;
            None
        }
    };

    let startup_mode = select_startup_mode(
        data_dir_state,
        cfg.postgres.data_dir.as_path(),
        cache.as_ref(),
        &cfg.cluster.member_id,
        process_defaults,
    )?;

    let mut event = runtime_event(
        RuntimeEventKind::ModeSelected,
        "ok",
        SeverityText::Info,
        "startup mode selected",
    );
    let fields = event.fields_mut();
    fields.append_json_map(runtime_base_fields(cfg, startup_run_id).into_attributes());
    fields.insert("startup_mode", startup_mode_label(&startup_mode));
    log.emit_app_event("runtime::plan_startup", event)
        .map_err(|err| {
            RuntimeError::StartupPlanning(format!("startup mode log emit failed: {err}"))
        })?;

    Ok(startup_mode)
}

fn inspect_data_dir(data_dir: &Path) -> Result<DataDirState, RuntimeError> {
    match fs::metadata(data_dir) {
        Ok(meta) => {
            if !meta.is_dir() {
                return Err(RuntimeError::StartupPlanning(format!(
                    "postgres.data_dir is not a directory: {}",
                    data_dir.display()
                )));
            }
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Ok(DataDirState::Missing);
        }
        Err(err) => {
            return Err(RuntimeError::StartupPlanning(format!(
                "failed to inspect data dir {}: {err}",
                data_dir.display()
            )));
        }
    }

    if data_dir.join("PG_VERSION").exists() {
        return Ok(DataDirState::Existing);
    }

    let mut entries = fs::read_dir(data_dir).map_err(|err| {
        RuntimeError::StartupPlanning(format!(
            "failed to read data dir {}: {err}",
            data_dir.display()
        ))
    })?;

    if entries.next().is_none() {
        Ok(DataDirState::Empty)
    } else {
        Err(RuntimeError::StartupPlanning(format!(
            "ambiguous data dir state: `{}` is non-empty but has no PG_VERSION",
            data_dir.display()
        )))
    }
}

fn probe_dcs_cache(cfg: &RuntimeConfig) -> Result<DcsCache, RuntimeError> {
    let mut store =
        EtcdDcsStore::connect(cfg.dcs.endpoints.clone(), &cfg.dcs.scope).map_err(|err| {
            RuntimeError::StartupPlanning(format!("failed to connect dcs for startup probe: {err}"))
        })?;

    let events = store.drain_watch_events().map_err(|err| {
        RuntimeError::StartupPlanning(format!("failed to read startup dcs events: {err}"))
    })?;

    let mut cache = DcsCache {
        members: BTreeMap::new(),
        leader: None,
        switchover: None,
        config: cfg.clone(),
        init_lock: None,
    };

    refresh_from_etcd_watch(&cfg.dcs.scope, &mut cache, events).map_err(|err| {
        RuntimeError::StartupPlanning(format!("failed to decode startup dcs snapshot: {err}"))
    })?;

    Ok(cache)
}

```

## Startup mode selection and source-member derivation

Source path: `src/runtime/node.rs`

```rust
fn select_startup_mode(
    data_dir_state: DataDirState,
    data_dir: &Path,
    cache: Option<&DcsCache>,
    self_member_id: &str,
    process_defaults: &ProcessDispatchDefaults,
) -> Result<StartupMode, RuntimeError> {
    match data_dir_state {
        DataDirState::Existing => Ok(StartupMode::ResumeExisting {
            start_intent: Box::new(select_resume_start_intent(
                data_dir,
                cache,
                self_member_id,
                process_defaults,
            )?),
        }),
        DataDirState::Missing | DataDirState::Empty => {
            let init_lock_present = cache
                .and_then(|snapshot| snapshot.init_lock.as_ref())
                .is_some();
            let self_member_id = MemberId(self_member_id.to_string());

            let leader = leader_from_leader_key(cache, &self_member_id).or_else(|| {
                if init_lock_present {
                    foreign_healthy_primary_member(cache, &self_member_id)
                } else {
                    None
                }
            });

            match leader {
                Some(leader_member) => {
                    let source = basebackup_source_from_member(
                        &self_member_id,
                        &leader_member,
                        process_defaults,
                    )
                    .map_err(|err| RuntimeError::StartupPlanning(err.to_string()))?;
                    Ok(StartupMode::CloneReplica {
                        leader_member_id: leader_member.member_id.clone(),
                        source: Box::new(source.clone()),
                        start_intent: Box::new(replica_start_intent_from_source(&source, data_dir)),
                    })
                }
                None => {
                    if init_lock_present {
                        Err(RuntimeError::StartupPlanning(
                            "cluster is already initialized (dcs init lock present) but no healthy primary is available for basebackup"
                                .to_string(),
                        ))
                    } else {
                        Ok(StartupMode::InitializePrimary {
                            start_intent: Box::new(ManagedPostgresStartIntent::primary()),
                        })
                    }
                }
            }
        }
    }
}

fn select_resume_start_intent(
    data_dir: &Path,
    cache: Option<&DcsCache>,
    self_member_id: &str,
    process_defaults: &ProcessDispatchDefaults,
) -> Result<ManagedPostgresStartIntent, RuntimeError> {
    let self_member_id = MemberId(self_member_id.to_string());
    let managed_recovery_state = crate::postgres_managed::inspect_managed_recovery_state(data_dir)
        .map_err(|err| RuntimeError::StartupPlanning(err.to_string()))?;
    let has_local_managed_replica_residue =
        managed_recovery_state != crate::postgres_managed_conf::ManagedRecoverySignal::None;

    let Some(cache) = cache else {
        if has_local_managed_replica_residue {
            return Err(RuntimeError::StartupPlanning(
                "existing postgres data dir contains managed replica recovery state but startup dcs cache probe was unavailable; cannot rebuild authoritative startup intent"
                    .to_string(),
            ));
        }
        return Ok(ManagedPostgresStartIntent::primary());
    };

    if cache
        .leader
        .as_ref()
        .map(|record| record.member_id == self_member_id)
        .unwrap_or(false)
    {
        return Ok(ManagedPostgresStartIntent::primary());
    }

    if let Some(leader_member) = leader_from_leader_key(Some(cache), &self_member_id)
        .or_else(|| foreign_healthy_primary_member(Some(cache), &self_member_id))
    {
        let source =
            basebackup_source_from_member(&self_member_id, &leader_member, process_defaults)
                .map_err(|err| RuntimeError::StartupPlanning(err.to_string()))?;
        return Ok(replica_start_intent_from_source(&source, data_dir));
    }

    if local_primary_member(cache, &self_member_id).is_some() {
        return Ok(ManagedPostgresStartIntent::primary());
    }

    if has_local_managed_replica_residue {
        if let Some(source_member) = resume_replica_source_member(cache, &self_member_id) {
            let source = basebackup_resume_source_from_member(
                &self_member_id,
                &source_member,
                process_defaults,
            )
            .map_err(|err| RuntimeError::StartupPlanning(err.to_string()))?;
            return Ok(replica_start_intent_from_source(&source, data_dir));
        }
        return Err(RuntimeError::StartupPlanning(
            "existing postgres data dir contains managed replica recovery state but no healthy primary is available in DCS to rebuild authoritative managed config"
                .to_string(),
        ));
    }

    Ok(ManagedPostgresStartIntent::primary())
}

fn leader_from_leader_key(
    cache: Option<&DcsCache>,
    self_member_id: &MemberId,
) -> Option<crate::dcs::state::MemberRecord> {
    let snapshot = cache?;
    let leader_record = snapshot.leader.as_ref()?;
    if leader_record.member_id == *self_member_id {
        return None;
    }
    let member = snapshot.members.get(&leader_record.member_id)?;
    let eligible = member.role == MemberRole::Primary && member.sql == SqlStatus::Healthy;
    if eligible {
        Some(member.clone())
    } else {
        None
    }
}

fn foreign_healthy_primary_member(
    cache: Option<&DcsCache>,
    self_member_id: &MemberId,
) -> Option<crate::dcs::state::MemberRecord> {
    cache?
        .members
        .values()
        .find(|member| {
            member.member_id != *self_member_id
                && member.role == MemberRole::Primary
                && member.sql == SqlStatus::Healthy
        })
        .cloned()
}

fn local_primary_member<'a>(
    cache: &'a DcsCache,
    self_member_id: &MemberId,
) -> Option<&'a crate::dcs::state::MemberRecord> {
    cache
        .members
        .get(self_member_id)
        .filter(|member| member.role == MemberRole::Primary && member.sql == SqlStatus::Healthy)
}

fn resume_replica_source_member(
    cache: &DcsCache,
    self_member_id: &MemberId,
) -> Option<crate::dcs::state::MemberRecord> {
    relaxed_leader_from_leader_key(cache, self_member_id).or_else(|| {
        cache
            .members
            .values()
            .filter(|member| member.member_id != *self_member_id)
            .max_by_key(|member| member.updated_at)
            .cloned()
    })
}

fn relaxed_leader_from_leader_key(
    cache: &DcsCache,
    self_member_id: &MemberId,
) -> Option<crate::dcs::state::MemberRecord> {
    let leader_record = cache.leader.as_ref()?;
    if leader_record.member_id == *self_member_id {
        return None;
    }

    cache.members.get(&leader_record.member_id).cloned()
}

fn replica_start_intent_from_source(
    source: &ReplicatorSourceConn,
    data_dir: &Path,
) -> ManagedPostgresStartIntent {
    ManagedPostgresStartIntent::replica(
        source.conninfo.clone(),
        managed_standby_auth_from_role_auth(&source.auth, data_dir),
        None,
    )
}

```

## Init-lock seeding and startup execution orchestration

Source path: `src/runtime/node.rs`

```rust
fn claim_dcs_init_lock_and_seed_config(cfg: &RuntimeConfig) -> Result<(), String> {
    let init_path = format!("/{}/init", cfg.dcs.scope.trim_matches('/'));
    let config_path = format!("/{}/config", cfg.dcs.scope.trim_matches('/'));

    let mut store = EtcdDcsStore::connect(cfg.dcs.endpoints.clone(), &cfg.dcs.scope)
        .map_err(|err| format!("connect failed: {err}"))?;

    let encoded_init = serde_json::to_string(&crate::dcs::state::InitLockRecord {
        holder: MemberId(cfg.cluster.member_id.clone()),
    })
    .map_err(|err| format!("encode init lock record failed: {err}"))?;

    let claimed = store
        .put_path_if_absent(init_path.as_str(), encoded_init)
        .map_err(|err| format!("init lock write failed at `{init_path}`: {err}"))?;
    if !claimed {
        return Err(format!(
            "cluster already initialized (init lock exists at `{init_path}`)"
        ));
    }

    if let Some(init_cfg) = cfg.dcs.init.as_ref() {
        if init_cfg.write_on_bootstrap {
            let _seeded = store
                .put_path_if_absent(config_path.as_str(), init_cfg.payload_json.clone())
                .map_err(|err| format!("seed config failed at `{config_path}`: {err}"))?;
        }
    }

    Ok(())
}

async fn execute_startup(
    cfg: &RuntimeConfig,
    process_defaults: &ProcessDispatchDefaults,
    startup_mode: &StartupMode,
    log: &crate::logging::LogHandle,
    startup_run_id: &str,
) -> Result<(), RuntimeError> {
    ensure_start_paths(process_defaults, &cfg.postgres.data_dir)?;

    let actions = build_startup_actions(cfg, startup_mode)?;

    let mut planned_event = runtime_event(
        RuntimeEventKind::ActionsPlanned,
        "ok",
        SeverityText::Debug,
        "startup actions planned",
    );
    let fields = planned_event.fields_mut();
    fields.append_json_map(runtime_base_fields(cfg, startup_run_id).into_attributes());
    fields.insert("startup_mode", startup_mode_label(startup_mode));
    fields.insert("startup_actions_total", actions.len());
    log.emit_app_event("runtime::execute_startup", planned_event)
        .map_err(|err| {
            RuntimeError::StartupExecution(format!("startup actions log emit failed: {err}"))
        })?;

    for (action_index, action) in actions.into_iter().enumerate() {
        let action_kind = startup_action_kind_label(&action);
        let mut action_fields = runtime_base_fields(cfg, startup_run_id);
        action_fields.insert("startup_mode", startup_mode_label(startup_mode));
        action_fields.insert("startup_action_index", action_index);
        action_fields.insert("startup_action_kind", action_kind);
        let mut started_event = runtime_event(
            RuntimeEventKind::Action,
            "started",
            SeverityText::Info,
            "startup action started",
        );
        started_event
            .fields_mut()
            .append_json_map(action_fields.clone().into_attributes());
        log.emit_app_event("runtime::execute_startup", started_event)
            .map_err(|err| {
                RuntimeError::StartupExecution(format!("startup action log emit failed: {err}"))
            })?;

        if let StartupAction::StartPostgres(_) = &action {
            emit_startup_phase(log, "start", "start postgres with managed config").map_err(
                |err| {
                    RuntimeError::StartupExecution(format!("startup phase log emit failed: {err}"))
                },
            )?;
        }

        let result = match action {
            StartupAction::ClaimInitLockAndSeedConfig => {
                claim_dcs_init_lock_and_seed_config(cfg).map_err(|err| {
                    RuntimeError::StartupExecution(format!("dcs init lock claim failed: {err}"))
                })?;
                Ok(())
            }
            StartupAction::RunJob(job) => run_startup_job(cfg, *job, log).await,
            StartupAction::StartPostgres(start_intent) => {
                run_start_job(cfg, process_defaults, start_intent.as_ref(), log).await
            }
            StartupAction::EnsureRequiredRoles => {
                ensure_required_roles(cfg, process_defaults).await
            }
        };

        match result {
            Ok(()) => {
                let mut done_event = runtime_event(
                    RuntimeEventKind::Action,
                    "ok",
                    SeverityText::Info,
                    "startup action completed",
                );
                done_event
                    .fields_mut()
                    .append_json_map(action_fields.into_attributes());
                log.emit_app_event("runtime::execute_startup", done_event)
                    .map_err(|err| {
                        RuntimeError::StartupExecution(format!(
                            "startup action log emit failed: {err}"
                        ))
                    })?;
            }
            Err(err) => {
                let mut failed_event = runtime_event(
                    RuntimeEventKind::Action,
                    "failed",
                    SeverityText::Error,
                    "startup action failed",
                );
                let fields = failed_event.fields_mut();
                fields.append_json_map(action_fields.into_attributes());
                fields.insert("error", err.to_string());
                log.emit_app_event("runtime::execute_startup", failed_event)
                    .map_err(|emit_err| {
                        RuntimeError::StartupExecution(format!(
                            "startup action failure log emit failed: {emit_err}"
                        ))
                    })?;
                return Err(err);
            }
        };
    }

    Ok(())
}

```

## Startup action planning and role bootstrap

Source path: `src/runtime/node.rs`

```rust
fn build_startup_actions(
    cfg: &RuntimeConfig,
    startup_mode: &StartupMode,
) -> Result<Vec<StartupAction>, RuntimeError> {
    match startup_mode {
        StartupMode::InitializePrimary { start_intent } => Ok(vec![
            StartupAction::ClaimInitLockAndSeedConfig,
            StartupAction::RunJob(Box::new(ProcessJobKind::Bootstrap(BootstrapSpec {
                data_dir: cfg.postgres.data_dir.clone(),
                superuser_username: cfg.postgres.roles.superuser.username.clone(),
                timeout_ms: Some(cfg.process.bootstrap_timeout_ms),
            }))),
            StartupAction::StartPostgres(start_intent.clone()),
            StartupAction::EnsureRequiredRoles,
        ]),
        StartupMode::CloneReplica {
            source,
            start_intent,
            ..
        } => Ok(vec![
            StartupAction::RunJob(Box::new(ProcessJobKind::BaseBackup(BaseBackupSpec {
                data_dir: cfg.postgres.data_dir.clone(),
                source: source.as_ref().clone(),
                timeout_ms: Some(cfg.process.bootstrap_timeout_ms),
            }))),
            StartupAction::StartPostgres(start_intent.clone()),
        ]),
        StartupMode::ResumeExisting { start_intent } => {
            if has_postmaster_pid(&cfg.postgres.data_dir) {
                Ok(Vec::new())
            } else {
                Ok(vec![StartupAction::StartPostgres(start_intent.clone())])
            }
        }
    }
}

async fn ensure_required_roles(
    cfg: &RuntimeConfig,
    process_defaults: &ProcessDispatchDefaults,
) -> Result<(), RuntimeError> {
    let mut config = tokio_postgres::Config::new();
    config.host_path(process_defaults.socket_dir.as_path());
    config.port(process_defaults.postgres_port);
    config.user(cfg.postgres.roles.superuser.username.as_str());
    config.dbname(cfg.postgres.local_conn_identity.dbname.as_str());
    config.connect_timeout(Duration::from_secs(cfg.postgres.connect_timeout_s.into()));
    if let RoleAuthConfig::Password { password } = &cfg.postgres.roles.superuser.auth {
        let resolved = resolve_secret_string("postgres.roles.superuser.auth.password", password)
            .map_err(|err| {
                RuntimeError::StartupExecution(format!(
                    "failed to resolve bootstrap superuser password for role provisioning: {err}"
                ))
            })?;
        config.password(resolved);
    }

    let (client, connection) = config.connect(NoTls).await.map_err(|err| {
        RuntimeError::StartupExecution(format!(
            "failed to connect to local postgres for role provisioning: {err}"
        ))
    })?;
    let connection_task = tokio::spawn(connection);

    let provision_sql = render_required_role_sql(cfg)?;
    client
        .batch_execute(provision_sql.as_str())
        .await
        .map_err(|err| {
            RuntimeError::StartupExecution(format!(
                "failed to provision required postgres roles: {err}"
            ))
        })?;
    drop(client);

    let connection_result = connection_task.await.map_err(|err| {
        RuntimeError::StartupExecution(format!(
            "role provisioning connection task join failed: {err}"
        ))
    })?;
    connection_result.map_err(|err| {
        RuntimeError::StartupExecution(format!(
            "role provisioning connection ended with an error: {err}"
        ))
    })
}

fn render_required_role_sql(cfg: &RuntimeConfig) -> Result<String, RuntimeError> {
    let superuser = render_role_provision_block(
        cfg.postgres.roles.superuser.username.as_str(),
        &cfg.postgres.roles.superuser.auth,
        "LOGIN SUPERUSER NOREPLICATION",
    )?;
    let replicator = render_role_provision_block(
        cfg.postgres.roles.replicator.username.as_str(),
        &cfg.postgres.roles.replicator.auth,
        "LOGIN REPLICATION NOSUPERUSER",
    )?;
    let rewinder = render_role_provision_block(
        cfg.postgres.roles.rewinder.username.as_str(),
        &cfg.postgres.roles.rewinder.auth,
        "LOGIN NOREPLICATION NOSUPERUSER",
    )?;
    let rewinder_grants = render_rewinder_grants_sql(cfg.postgres.roles.rewinder.username.as_str());
    Ok(format!(
        "{superuser}\n{replicator}\n{rewinder}\n{rewinder_grants}"
    ))
}

fn render_role_provision_block(
    username: &str,
    auth: &RoleAuthConfig,
    attributes: &str,
) -> Result<String, RuntimeError> {
    let username_literal = sql_literal(username);
    let role_statement = match auth {
        RoleAuthConfig::Tls => {
            format!("format('ALTER ROLE %I WITH {attributes}', {username_literal})")
        }
        RoleAuthConfig::Password { password } => {
            let resolved = resolve_secret_string("runtime role provisioning password", password)
                .map_err(|err| {
                    RuntimeError::StartupExecution(format!(
                    "failed to resolve runtime role provisioning password for `{username}`: {err}"
                ))
                })?;
            let password_literal = sql_literal(resolved.as_str());
            format!(
                "format('ALTER ROLE %I WITH {attributes} PASSWORD %L', {username_literal}, {password_literal})"
            )
        }
    };
    Ok(format!(
        "DO $$\nBEGIN\n  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = {username_literal}) THEN\n    EXECUTE format('CREATE ROLE %I', {username_literal});\n  END IF;\n  EXECUTE {role_statement};\nEND\n$$;"
    ))
}

fn sql_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn sql_identifier(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn render_rewinder_grants_sql(username: &str) -> String {
    let role = sql_identifier(username);
    [
        "GRANT EXECUTE ON FUNCTION pg_catalog.pg_ls_dir(text, boolean, boolean) TO ",
        role.as_str(),
        ";",
        "\nGRANT EXECUTE ON FUNCTION pg_catalog.pg_stat_file(text, boolean) TO ",
        role.as_str(),
        ";",
        "\nGRANT EXECUTE ON FUNCTION pg_catalog.pg_read_binary_file(text) TO ",
        role.as_str(),
        ";",
        "\nGRANT EXECUTE ON FUNCTION pg_catalog.pg_read_binary_file(text, bigint, bigint, boolean) TO ",
        role.as_str(),
        ";",
    ]
    .concat()
}

```

## Startup process execution helpers

Source path: `src/runtime/node.rs`

```rust
async fn run_start_job(
    cfg: &RuntimeConfig,
    process_defaults: &ProcessDispatchDefaults,
    start_intent: &ManagedPostgresStartIntent,
    log: &crate::logging::LogHandle,
) -> Result<(), RuntimeError> {
    if start_postgres_preflight_is_already_running(cfg.postgres.data_dir.as_path()).map_err(
        |err| {
            RuntimeError::StartupExecution(format!(
                "startup start-postgres preflight failed: {err}"
            ))
        },
    )? {
        emit_startup_phase(
            log,
            "start",
            "postgres already running; startup start_postgres is a noop",
        )
        .map_err(|err| {
            RuntimeError::StartupExecution(format!("startup phase log emit failed: {err}"))
        })?;
        return Ok(());
    }

    let managed = crate::postgres_managed::materialize_managed_postgres_config(cfg, start_intent)
        .map_err(|err| {
        RuntimeError::StartupExecution(format!("materialize managed postgres config failed: {err}"))
    })?;
    run_startup_job(
        cfg,
        ProcessJobKind::StartPostgres(StartPostgresSpec {
            data_dir: cfg.postgres.data_dir.clone(),
            config_file: managed.postgresql_conf_path,
            log_file: process_defaults.log_file.clone(),
            wait_seconds: Some(30),
            timeout_ms: Some(cfg.process.bootstrap_timeout_ms),
        }),
        log,
    )
    .await
}

async fn run_startup_job(
    cfg: &RuntimeConfig,
    job: ProcessJobKind,
    log: &crate::logging::LogHandle,
) -> Result<(), RuntimeError> {
    let mut runner = TokioCommandRunner;
    let timeout_ms = timeout_for_kind(&job, &cfg.process);
    let job_id = crate::state::JobId(format!("startup-{}", std::process::id()));
    let command = build_command(
        &cfg.process,
        &job_id,
        &job,
        cfg.logging.capture_subprocess_output,
    )
    .map_err(|err| {
        RuntimeError::StartupExecution(format!("startup command build failed: {err}"))
    })?;
    let log_identity = command.log_identity.clone();
    let command_display = format!("{} {}", command.program.display(), command.args.join(" "));

    let mut handle = runner.spawn(command).map_err(|err| {
        RuntimeError::StartupExecution(format!(
            "startup command spawn failed for `{command_display}`: {err}"
        ))
    })?;

    let started = system_now_unix_millis().map_err(|err| RuntimeError::Time(err.to_string()))?;
    let deadline = started.0.saturating_add(timeout_ms);

    loop {
        let lines = handle
            .drain_output(STARTUP_OUTPUT_DRAIN_MAX_BYTES)
            .await
            .map_err(|err| {
                RuntimeError::StartupExecution(format!(
                    "startup process output drain failed: {err}"
                ))
            })?;
        for line in lines {
            if let Err(err) = emit_startup_subprocess_line(log, &log_identity, line.clone()) {
                let mut event = runtime_event(
                    RuntimeEventKind::SubprocessLogEmitFailed,
                    "failed",
                    SeverityText::Warn,
                    "startup subprocess line emit failed",
                );
                let fields = event.fields_mut();
                fields.insert("job_id", log_identity.job_id.0.clone());
                fields.insert("job_kind", log_identity.job_kind.clone());
                fields.insert("binary", log_identity.binary.clone());
                fields.insert(
                    "stream",
                    match line.stream {
                        crate::process::jobs::ProcessOutputStream::Stdout => "stdout",
                        crate::process::jobs::ProcessOutputStream::Stderr => "stderr",
                    },
                );
                fields.insert("bytes_len", line.bytes.len());
                fields.insert("error", err.to_string());
                log.emit_app_event("runtime::run_startup_job", event)
                    .map_err(|emit_err| {
                        RuntimeError::StartupExecution(format!(
                            "startup subprocess emit failure log emit failed: {emit_err}"
                        ))
                    })?;
            }
        }

        match handle.poll_exit().map_err(|err| {
            RuntimeError::StartupExecution(format!("startup process poll failed: {err}"))
        })? {
            Some(ProcessExit::Success) => return Ok(()),
            Some(ProcessExit::Failure { code }) => {
                let lines = handle
                    .drain_output(STARTUP_OUTPUT_DRAIN_MAX_BYTES)
                    .await
                    .map_err(|err| {
                        RuntimeError::StartupExecution(format!(
                            "startup process output drain failed: {err}"
                        ))
                    })?;
                for line in lines {
                    emit_startup_subprocess_line(log, &log_identity, line).map_err(|err| {
                        RuntimeError::StartupExecution(format!(
                            "startup subprocess line emit failed: {err}"
                        ))
                    })?;
                }
                return Err(RuntimeError::StartupExecution(format!(
                    "startup command `{command_display}` exited unsuccessfully (code: {code:?})"
                )));
            }
            None => {}
        }

        let now = system_now_unix_millis().map_err(|err| RuntimeError::Time(err.to_string()))?;
        if now.0 >= deadline {
            handle.cancel().await.map_err(|err| {
                RuntimeError::StartupExecution(format!(
                    "startup command `{command_display}` timeout cancellation failed: {err}"
                ))
            })?;
            let lines = handle
                .drain_output(STARTUP_OUTPUT_DRAIN_MAX_BYTES)
                .await
                .map_err(|err| {
                    RuntimeError::StartupExecution(format!(
                        "startup process output drain failed: {err}"
                    ))
                })?;
            for line in lines {
                emit_startup_subprocess_line(log, &log_identity, line).map_err(|err| {
                    RuntimeError::StartupExecution(format!(
                        "startup subprocess line emit failed: {err}"
                    ))
                })?;
            }
            return Err(RuntimeError::StartupExecution(format!(
                "startup command `{command_display}` timed out after {timeout_ms} ms"
            )));
        }

        tokio::time::sleep(STARTUP_JOB_POLL_INTERVAL).await;
    }
}

```

## Worker graph bootstrap and initial published states

Source path: `src/runtime/node.rs`

```rust
async fn run_workers(
    cfg: RuntimeConfig,
    process_defaults: ProcessDispatchDefaults,
    log: crate::logging::LogHandle,
) -> Result<(), RuntimeError> {
    let now = now_unix_millis()?;

    let (_cfg_publisher, cfg_subscriber) = new_state_channel(cfg.clone(), now);
    let (pg_publisher, pg_subscriber) = new_state_channel(initial_pg_state(), now);

    let initial_dcs = DcsState {
        worker: WorkerStatus::Starting,
        trust: DcsTrust::NotTrusted,
        cache: DcsCache {
            members: BTreeMap::new(),
            leader: None,
            switchover: None,
            config: cfg.clone(),
            init_lock: None,
        },
        last_refresh_at: None,
    };
    let (dcs_publisher, dcs_subscriber) = new_state_channel(initial_dcs, now);

    let initial_process = ProcessState::Idle {
        worker: WorkerStatus::Starting,
        last_outcome: None,
    };
    let (process_publisher, process_subscriber) = new_state_channel(initial_process.clone(), now);

    let initial_ha = HaState {
        worker: WorkerStatus::Starting,
        phase: HaPhase::Init,
        tick: 0,
        decision: crate::ha::decision::HaDecision::NoChange,
    };
    let (ha_publisher, ha_subscriber) = new_state_channel(initial_ha, now);

    let initial_debug_snapshot = build_snapshot(
        &DebugSnapshotCtx {
            app: AppLifecycle::Running,
            config: cfg_subscriber.latest(),
            pg: pg_subscriber.latest(),
            dcs: dcs_subscriber.latest(),
            process: process_subscriber.latest(),
            ha: ha_subscriber.latest(),
        },
        now,
        0,
        &[],
        &[],
    );
    let (debug_publisher, debug_subscriber) = new_state_channel(initial_debug_snapshot, now);

    let self_id = MemberId(cfg.cluster.member_id.clone());
    let scope = cfg.dcs.scope.clone();

    let pg_ctx = crate::pginfo::state::PgInfoWorkerCtx {
        self_id: self_id.clone(),
        postgres_conninfo: local_postgres_conninfo(
            &process_defaults,
            &cfg.postgres.local_conn_identity,
            cfg.postgres.roles.superuser.username.as_str(),
            cfg.postgres.connect_timeout_s,
        ),
        poll_interval: Duration::from_millis(cfg.ha.loop_interval_ms),
        publisher: pg_publisher,
        log: log.clone(),
        last_emitted_sql_status: None,
    };

    let dcs_store = EtcdDcsStore::connect(cfg.dcs.endpoints.clone(), &scope)
        .map_err(|err| RuntimeError::Worker(format!("dcs store connect failed: {err}")))?;
    let dcs_ctx = DcsWorkerCtx {
        self_id: self_id.clone(),
        scope: scope.clone(),
        poll_interval: Duration::from_millis(cfg.ha.loop_interval_ms),
        local_postgres_host: cfg.postgres.listen_host.clone(),
        local_postgres_port: advertised_postgres_port(&cfg),
        local_api_url: advertised_operator_api_url(&cfg),
        pg_subscriber: pg_subscriber.clone(),
        publisher: dcs_publisher,
        store: Box::new(dcs_store),
        log: log.clone(),
        cache: DcsCache {
            members: BTreeMap::new(),
            leader: None,
            switchover: None,
            config: cfg.clone(),
            init_lock: None,
        },
        last_published_pg_version: None,
        last_emitted_store_healthy: None,
        last_emitted_trust: None,
    };

    let (process_inbox_tx, process_inbox_rx) = mpsc::unbounded_channel();
    let process_ctx = ProcessWorkerCtx {
        poll_interval: PROCESS_WORKER_POLL_INTERVAL,
        config: cfg.process.clone(),
        log: log.clone(),
        capture_subprocess_output: cfg.logging.capture_subprocess_output,
        state: initial_process,
        publisher: process_publisher,
        inbox: process_inbox_rx,
        inbox_disconnected_logged: false,
        command_runner: Box::new(TokioCommandRunner),
        active_runtime: None,
        last_rejection: None,
        now: Box::new(system_now_unix_millis),
    };

    let ha_store = EtcdDcsStore::connect_with_leader_lease(
        cfg.dcs.endpoints.clone(),
        &scope,
        cfg.ha.lease_ttl_ms,
    )
    .map_err(|err| RuntimeError::Worker(format!("ha store connect failed: {err}")))?;
    let mut ha_ctx = HaWorkerCtx::contract_stub(HaWorkerContractStubInputs {
        publisher: ha_publisher,
        config_subscriber: cfg_subscriber.clone(),
        pg_subscriber: pg_subscriber.clone(),
        dcs_subscriber: dcs_subscriber.clone(),
        process_subscriber: process_subscriber.clone(),
        process_inbox: process_inbox_tx,
        dcs_store: Box::new(ha_store),
        scope: scope.clone(),
        self_id: self_id.clone(),
    });
    ha_ctx.poll_interval = Duration::from_millis(cfg.ha.loop_interval_ms);
    ha_ctx.now = Box::new(system_now_unix_millis);
    ha_ctx.process_defaults = process_defaults;
    ha_ctx.log = log.clone();

    let mut debug_ctx = DebugApiCtx::contract_stub(DebugApiContractStubInputs {
        publisher: debug_publisher,
        config_subscriber: cfg_subscriber.clone(),
        pg_subscriber: pg_subscriber.clone(),
        dcs_subscriber: dcs_subscriber.clone(),
        process_subscriber: process_subscriber.clone(),
        ha_subscriber: ha_subscriber.clone(),
    });
    debug_ctx.app = AppLifecycle::Running;
    debug_ctx.poll_interval = Duration::from_millis(cfg.ha.loop_interval_ms);
    debug_ctx.now = Box::new(system_now_unix_millis);

    let api_store = EtcdDcsStore::connect(cfg.dcs.endpoints.clone(), &scope)
        .map_err(|err| RuntimeError::Worker(format!("api store connect failed: {err}")))?;
    let listener = TcpListener::bind(cfg.api.listen_addr)
        .await
        .map_err(|err| RuntimeError::ApiBind {
            listen_addr: cfg.api.listen_addr,
            message: err.to_string(),
        })?;
    let mut api_ctx = ApiWorkerCtx::new(listener, cfg_subscriber, Box::new(api_store), log.clone());
    api_ctx.set_ha_snapshot_subscriber(debug_subscriber);
    let server_tls = crate::tls::build_rustls_server_config(&cfg.api.security.tls)
        .map_err(|err| RuntimeError::Worker(format!("api tls config build failed: {err}")))?;
    api_ctx
        .configure_tls(cfg.api.security.tls.mode, server_tls)
        .map_err(|err| RuntimeError::Worker(format!("api tls configure failed: {err}")))?;
    let require_client_cert = match cfg.api.security.tls.client_auth.as_ref() {
        Some(auth) => auth.require_client_cert,
        None => false,
    };
    api_ctx.set_require_client_cert(require_client_cert);

    tokio::try_join!(
        crate::pginfo::worker::run(pg_ctx),
        crate::dcs::worker::run(dcs_ctx),
        crate::process::worker::run(process_ctx),
        crate::logging::postgres_ingest::run(crate::logging::postgres_ingest::build_ctx(
            cfg.clone(),
            log.clone(),
        )),
        crate::ha::worker::run(ha_ctx),
        crate::debug_api::worker::run(debug_ctx),
        crate::api::worker::run(api_ctx),
    )
    .map_err(|err| RuntimeError::Worker(err.to_string()))?;

    Ok(())
}

fn advertised_operator_api_url(cfg: &RuntimeConfig) -> Option<String> {
    if let Some(api_url) = cfg.pgtm.as_ref().and_then(|pgtm| pgtm.api_url.clone()) {
        return Some(api_url);
    }

    if cfg.api.listen_addr.ip().is_unspecified() {
        return None;
    }

    let scheme = match cfg.api.security.tls.mode {
        crate::config::ApiTlsMode::Disabled => "http",
        crate::config::ApiTlsMode::Optional | crate::config::ApiTlsMode::Required => "https",
    };
    Some(format!("{scheme}://{}", cfg.api.listen_addr))
}

fn local_postgres_conninfo(
    process_defaults: &ProcessDispatchDefaults,
    identity: &crate::config::PostgresConnIdentityConfig,
    superuser_username: &str,
    connect_timeout_s: u32,
) -> crate::pginfo::state::PgConnInfo {
    crate::pginfo::state::PgConnInfo {
        host: process_defaults.socket_dir.display().to_string(),
        port: process_defaults.postgres_port,
        user: superuser_username.to_string(),
        dbname: identity.dbname.clone(),
        application_name: None,
        connect_timeout_s: Some(connect_timeout_s),
        ssl_mode: identity.ssl_mode,
        ssl_root_cert: identity.ca_cert.clone(),
        options: None,
    }
}

fn initial_pg_state() -> PgInfoState {
    PgInfoState::Unknown {
        common: PgInfoCommon {
            worker: WorkerStatus::Starting,
            sql: SqlStatus::Unknown,
            readiness: Readiness::Unknown,
            timeline: None,
            pg_config: PgConfig {
                port: None,
                hot_standby: None,
                primary_conninfo: None,
                primary_slot_name: None,
                extra: BTreeMap::new(),
            },
            last_refresh_at: None,
        },
    }
}

fn now_unix_millis() -> Result<UnixMillis, RuntimeError> {
```

# State structs and decision model

## Source path: `src/ha/state.rs`

```rust
use std::{path::PathBuf, time::Duration};

use crate::{
    config::{RoleAuthConfig, RuntimeConfig},
    dcs::{state::DcsState, store::DcsLeaderStore},
    logging::LogHandle,
    pginfo::state::{PgInfoState, PgSslMode},
    process::{
        jobs::ShutdownMode,
        state::{ProcessJobRequest, ProcessState},
    },
    state::{
        MemberId, StatePublisher, StateSubscriber, UnixMillis, Versioned, WorkerError, WorkerStatus,
    },
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use super::decision::{HaDecision, PhaseOutcome};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HaPhase {
    Init,
    WaitingPostgresReachable,
    WaitingDcsTrusted,
    WaitingSwitchoverSuccessor,
    Replica,
    CandidateLeader,
    Primary,
    Rewinding,
    Bootstrapping,
    Fencing,
    FailSafe,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HaState {
    pub(crate) worker: WorkerStatus,
    pub(crate) phase: HaPhase,
    pub(crate) tick: u64,
    pub(crate) decision: HaDecision,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WorldSnapshot {
    pub(crate) config: Versioned<RuntimeConfig>,
    pub(crate) pg: Versioned<PgInfoState>,
    pub(crate) dcs: Versioned<DcsState>,
    pub(crate) process: Versioned<ProcessState>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DecideInput {
    pub(crate) current: HaState,
    pub(crate) world: WorldSnapshot,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DecideOutput {
    pub(crate) next: HaState,
    pub(crate) outcome: PhaseOutcome,
}

pub(crate) struct HaWorkerCtx {
    pub(crate) poll_interval: Duration,
    pub(crate) state: HaState,
    pub(crate) publisher: StatePublisher<HaState>,
    pub(crate) config_subscriber: StateSubscriber<RuntimeConfig>,
    pub(crate) pg_subscriber: StateSubscriber<PgInfoState>,
    pub(crate) dcs_subscriber: StateSubscriber<DcsState>,
    pub(crate) process_subscriber: StateSubscriber<ProcessState>,
    pub(crate) process_inbox: UnboundedSender<ProcessJobRequest>,
    pub(crate) dcs_store: Box<dyn DcsLeaderStore>,
    pub(crate) scope: String,
    pub(crate) self_id: MemberId,
    pub(crate) process_defaults: ProcessDispatchDefaults,
    pub(crate) now: Box<dyn FnMut() -> Result<UnixMillis, WorkerError> + Send>,
    pub(crate) log: LogHandle,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessDispatchDefaults {
    pub(crate) postgres_host: String,
    pub(crate) postgres_port: u16,
    pub(crate) socket_dir: PathBuf,
    pub(crate) log_file: PathBuf,
    pub(crate) replicator_username: String,
    pub(crate) replicator_auth: RoleAuthConfig,
    pub(crate) rewinder_username: String,
    pub(crate) rewinder_auth: RoleAuthConfig,
    pub(crate) remote_dbname: String,
    pub(crate) remote_ssl_mode: PgSslMode,
    pub(crate) remote_ssl_root_cert: Option<PathBuf>,
    pub(crate) connect_timeout_s: u32,
    pub(crate) shutdown_mode: ShutdownMode,
}

impl ProcessDispatchDefaults {
    pub(crate) fn contract_stub() -> Self {
        Self {
            postgres_host: "127.0.0.1".to_string(),
            postgres_port: 5432,
            socket_dir: PathBuf::from("/tmp/pgtuskmaster/socket"),
            log_file: PathBuf::from("/tmp/pgtuskmaster/postgres.log"),
            replicator_username: "replicator".to_string(),
            replicator_auth: contract_stub_password_auth(),
            rewinder_username: "rewinder".to_string(),
            rewinder_auth: contract_stub_password_auth(),
            remote_dbname: "postgres".to_string(),
            remote_ssl_mode: PgSslMode::Prefer,
            remote_ssl_root_cert: None,
            connect_timeout_s: 5,
            shutdown_mode: ShutdownMode::Fast,
        }
    }
}

fn contract_stub_password_auth() -> RoleAuthConfig {
    RoleAuthConfig::Password {
        password: crate::config::SecretSource::Inline {
            content: "secret-password".to_string(),
        },
    }
}

pub(crate) struct HaWorkerContractStubInputs {
    pub(crate) publisher: StatePublisher<HaState>,
    pub(crate) config_subscriber: StateSubscriber<RuntimeConfig>,
    pub(crate) pg_subscriber: StateSubscriber<PgInfoState>,
    pub(crate) dcs_subscriber: StateSubscriber<DcsState>,
    pub(crate) process_subscriber: StateSubscriber<ProcessState>,
    pub(crate) process_inbox: UnboundedSender<ProcessJobRequest>,
    pub(crate) dcs_store: Box<dyn DcsLeaderStore>,
    pub(crate) scope: String,
    pub(crate) self_id: MemberId,
}

impl HaWorkerCtx {
    pub(crate) fn contract_stub(inputs: HaWorkerContractStubInputs) -> Self {
        let HaWorkerContractStubInputs {
            publisher,
            config_subscriber,
            pg_subscriber,
            dcs_subscriber,
            process_subscriber,
            process_inbox,
            dcs_store,
            scope,
            self_id,
        } = inputs;

        Self {
            poll_interval: Duration::from_millis(10),
            state: HaState {
                worker: WorkerStatus::Starting,
                phase: HaPhase::Init,
                tick: 0,
                decision: HaDecision::NoChange,
            },
            publisher,
            config_subscriber,
            pg_subscriber,
            dcs_subscriber,
            process_subscriber,
            process_inbox,
            dcs_store,
            scope,
            self_id,
            process_defaults: ProcessDispatchDefaults::contract_stub(),
            now: Box::new(|| Ok(UnixMillis(0))),
            log: LogHandle::disabled(),
        }
    }
}
```

## Source path: `src/ha/decision.rs`

```rust
use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{
    dcs::state::{member_record_is_fresh, DcsTrust, MemberRecord, MemberRole},
    pginfo::state::{PgInfoState, Readiness, SqlStatus},
    process::{
        jobs::ActiveJobKind,
        state::{JobOutcome, ProcessState},
    },
    state::{MemberId, TimelineId, UnixMillis},
};

use super::state::{HaPhase, WorldSnapshot};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DecisionFacts {
    pub(crate) self_member_id: MemberId,
    pub(crate) trust: DcsTrust,
    pub(crate) postgres_reachable: bool,
    pub(crate) postgres_primary: bool,
    pub(crate) pg_observed_at: UnixMillis,
    pub(crate) leader_member_id: Option<MemberId>,
    pub(crate) active_leader_member_id: Option<MemberId>,
    pub(crate) followable_member_id: Option<MemberId>,
    pub(crate) switchover_pending: bool,
    pub(crate) pending_switchover_target: Option<MemberId>,
    pub(crate) eligible_switchover_targets: BTreeSet<MemberId>,
    pub(crate) i_am_leader: bool,
    pub(crate) has_other_leader_record: bool,
    pub(crate) has_available_other_leader: bool,
    pub(crate) rewind_required: bool,
    pub(crate) process_state: ProcessState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ProcessActivity {
    Running,
    IdleNoOutcome,
    IdleSuccess,
    IdleFailure,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PhaseOutcome {
    pub(crate) next_phase: HaPhase,
    pub(crate) decision: HaDecision,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum HaDecision {
    #[default]
    NoChange,
    WaitForPostgres {
        start_requested: bool,
        leader_member_id: Option<MemberId>,
    },
    WaitForDcsTrust,
    AttemptLeadership,
    FollowLeader {
        leader_member_id: MemberId,
    },
    BecomePrimary {
        promote: bool,
    },
    CompleteSwitchover,
    StepDown(StepDownPlan),
    RecoverReplica {
        strategy: RecoveryStrategy,
    },
    FenceNode,
    ReleaseLeaderLease {
        reason: LeaseReleaseReason,
    },
    EnterFailSafe {
        release_leader_lease: bool,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct StepDownPlan {
    pub(crate) reason: StepDownReason,
    pub(crate) release_leader_lease: bool,
    pub(crate) fence: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum StepDownReason {
    Switchover,
    ForeignLeaderDetected { leader_member_id: MemberId },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum RecoveryStrategy {
    Rewind { leader_member_id: MemberId },
    BaseBackup { leader_member_id: MemberId },
    Bootstrap,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum LeaseReleaseReason {
    FencingComplete,
    PostgresUnreachable,
}

impl DecisionFacts {
    pub(crate) fn from_world(world: &WorldSnapshot) -> Self {
        let self_member_id = MemberId(world.config.value.cluster.member_id.clone());
        let leader_member_id = world
            .dcs
            .value
            .cache
            .leader
            .as_ref()
            .map(|record| record.member_id.clone());
        let active_leader_member_id = leader_member_id
            .clone()
            .filter(|leader_id| is_available_primary_leader(world, leader_id));
        let followable_member_id = active_leader_member_id.clone().or_else(|| {
            world
                .dcs
                .value
                .cache
                .members
                .values()
                .find(|member| {
                    member.member_id != self_member_id
                        && member_record_is_fresh(
                            member,
                            &world.dcs.value.cache,
                            world.dcs.updated_at,
                        )
                        && member.role == MemberRole::Primary
                        && member.sql == SqlStatus::Healthy
                        && member.readiness == Readiness::Ready
                })
                .map(|member| member.member_id.clone())
        });
        let eligible_switchover_targets = eligible_switchover_targets(world);
        let i_am_leader = leader_member_id.as_ref() == Some(&self_member_id);
        let has_other_leader_record = leader_member_id
            .as_ref()
            .map(|leader_id| leader_id != &self_member_id)
            .unwrap_or(false);
        let has_available_other_leader = active_leader_member_id
            .as_ref()
            .map(|leader_id| leader_id != &self_member_id)
            .unwrap_or(false);

        Self {
            self_member_id,
            trust: world.dcs.value.trust.clone(),
            postgres_reachable: is_postgres_reachable(&world.pg.value),
            postgres_primary: is_local_primary(&world.pg.value),
            pg_observed_at: world.pg.updated_at,
            leader_member_id,
            active_leader_member_id: active_leader_member_id.clone(),
            followable_member_id: followable_member_id.clone(),
            switchover_pending: world.dcs.value.cache.switchover.is_some(),
            pending_switchover_target: world
                .dcs
                .value
                .cache
                .switchover
                .as_ref()
                .and_then(|request| request.switchover_to.clone()),
            eligible_switchover_targets,
            i_am_leader,
            has_other_leader_record,
            has_available_other_leader,
            rewind_required: followable_member_id
                .as_ref()
                .map(|leader_id| should_rewind_from_leader(world, leader_id))
                .unwrap_or(false),
            process_state: world.process.value.clone(),
        }
    }
}

impl ProcessActivity {
    fn from_process_state(process: &ProcessState, expected_kinds: &[ActiveJobKind]) -> Self {
        match process {
            ProcessState::Running { active, .. } => {
                if expected_kinds.contains(&active.kind) {
                    Self::Running
                } else {
                    Self::IdleNoOutcome
                }
            }
            ProcessState::Idle {
                last_outcome: Some(JobOutcome::Success { job_kind, .. }),
                ..
            } => {
                if expected_kinds.contains(job_kind) {
                    Self::IdleSuccess
                } else {
                    Self::IdleNoOutcome
                }
            }
            ProcessState::Idle {
                last_outcome:
                    Some(JobOutcome::Failure { job_kind, .. } | JobOutcome::Timeout { job_kind, .. }),
                ..
            } => {
                if expected_kinds.contains(job_kind) {
                    Self::IdleFailure
                } else {
                    Self::IdleNoOutcome
                }
            }
            ProcessState::Idle {
                last_outcome: None, ..
            } => Self::IdleNoOutcome,
        }
    }
}

impl DecisionFacts {
    pub(crate) fn start_postgres_can_be_requested(&self) -> bool {
        !matches!(self.process_state, ProcessState::Running { .. })
    }

    pub(crate) fn rewind_activity(&self) -> ProcessActivity {
        ProcessActivity::from_process_state(&self.process_state, &[ActiveJobKind::PgRewind])
    }

    pub(crate) fn bootstrap_activity(&self) -> ProcessActivity {
        ProcessActivity::from_process_state(
            &self.process_state,
            &[ActiveJobKind::BaseBackup, ActiveJobKind::Bootstrap],
        )
    }

    pub(crate) fn fencing_activity(&self) -> ProcessActivity {
        ProcessActivity::from_process_state(&self.process_state, &[ActiveJobKind::Fencing])
    }

    pub(crate) fn switchover_target_is_eligible(&self, member_id: &MemberId) -> bool {
        self.eligible_switchover_targets.contains(member_id)
    }
}

impl PhaseOutcome {
    pub(crate) fn new(next_phase: HaPhase, decision: HaDecision) -> Self {
        Self {
            next_phase,
            decision,
        }
    }
}

impl HaDecision {
    pub(crate) fn label(&self) -> &'static str {
        match self {
            Self::NoChange => "no_change",
            Self::WaitForPostgres { .. } => "wait_for_postgres",
            Self::WaitForDcsTrust => "wait_for_dcs_trust",
            Self::AttemptLeadership => "attempt_leadership",
            Self::FollowLeader { .. } => "follow_leader",
            Self::BecomePrimary { .. } => "become_primary",
            Self::CompleteSwitchover => "complete_switchover",
            Self::StepDown(_) => "step_down",
            Self::RecoverReplica { .. } => "recover_replica",
            Self::FenceNode => "fence_node",
            Self::ReleaseLeaderLease { .. } => "release_leader_lease",
            Self::EnterFailSafe { .. } => "enter_fail_safe",
        }
    }

    pub(crate) fn detail(&self) -> Option<String> {
        match self {
            Self::NoChange | Self::WaitForDcsTrust | Self::AttemptLeadership | Self::FenceNode => {
                None
            }
            Self::WaitForPostgres {
                start_requested,
                leader_member_id,
            } => {
                let leader_detail = leader_member_id
                    .as_ref()
                    .map(|leader| leader.0.as_str())
                    .unwrap_or("none");
                Some(format!(
                    "start_requested={start_requested}, leader_member_id={leader_detail}"
                ))
            }
            Self::FollowLeader { leader_member_id } => Some(leader_member_id.0.clone()),
            Self::BecomePrimary { promote } => Some(format!("promote={promote}")),
            Self::CompleteSwitchover => None,
            Self::StepDown(plan) => Some(format!(
                "reason={}, release_leader_lease={}, fence={}",
                plan.reason.label(),
                plan.release_leader_lease,
                plan.fence
            )),
            Self::RecoverReplica { strategy } => Some(strategy.label()),
            Self::ReleaseLeaderLease { reason } => Some(reason.label()),
            Self::EnterFailSafe {
                release_leader_lease,
            } => Some(format!("release_leader_lease={release_leader_lease}")),
        }
    }
}

impl StepDownReason {
    fn label(&self) -> String {
        match self {
            Self::Switchover => "switchover".to_string(),
            Self::ForeignLeaderDetected { leader_member_id } => {
                format!("foreign_leader_detected:{}", leader_member_id.0)
            }
        }
    }
}

impl RecoveryStrategy {
    fn label(&self) -> String {
        match self {
            Self::Rewind { leader_member_id } => format!("rewind:{}", leader_member_id.0),
            Self::BaseBackup { leader_member_id } => {
                format!("base_backup:{}", leader_member_id.0)
            }
            Self::Bootstrap => "bootstrap".to_string(),
        }
    }
}

impl LeaseReleaseReason {
    fn label(&self) -> String {
        match self {
            Self::FencingComplete => "fencing_complete".to_string(),
            Self::PostgresUnreachable => "postgres_unreachable".to_string(),
        }
    }
}

fn is_postgres_reachable(state: &PgInfoState) -> bool {
    let sql = match state {
        PgInfoState::Unknown { common } => &common.sql,
        PgInfoState::Primary { common, .. } => &common.sql,
        PgInfoState::Replica { common, .. } => &common.sql,
    };
    matches!(sql, SqlStatus::Healthy)
}

fn is_local_primary(state: &PgInfoState) -> bool {
    matches!(
        state,
        PgInfoState::Primary {
            common,
            ..
        } if matches!(common.sql, SqlStatus::Healthy)
    )
}

fn should_rewind_from_leader(world: &WorldSnapshot, leader_member_id: &MemberId) -> bool {
    if !is_local_primary(&world.pg.value) {
        return false;
    }

    let Some(local_timeline) = pg_timeline(&world.pg.value) else {
        return false;
    };

    let leader_timeline = world
        .dcs
        .value
        .cache
        .members
        .get(leader_member_id)
        .and_then(|member| member.timeline);

    leader_timeline
        .map(|timeline| timeline != local_timeline)
        .unwrap_or(false)
}

fn pg_timeline(state: &PgInfoState) -> Option<TimelineId> {
    match state {
        PgInfoState::Unknown { common } => common.timeline,
        PgInfoState::Primary { common, .. } => common.timeline,
        PgInfoState::Replica { common, .. } => common.timeline,
    }
}

fn is_available_primary_leader(world: &WorldSnapshot, leader_member_id: &MemberId) -> bool {
    let Some(member) = world.dcs.value.cache.members.get(leader_member_id) else {
        return false;
    };

    member_record_is_fresh(member, &world.dcs.value.cache, world.dcs.updated_at)
        && matches!(member.role, crate::dcs::state::MemberRole::Primary)
        && matches!(member.sql, SqlStatus::Healthy)
        && matches!(member.readiness, Readiness::Ready)
}

pub(crate) fn eligible_switchover_targets(world: &WorldSnapshot) -> BTreeSet<MemberId> {
    world
        .dcs
        .value
        .cache
        .members
        .values()
        .filter(|member| {
            switchover_target_is_eligible_member(
                member,
                &world.dcs.value.cache,
                world.dcs.updated_at,
            )
        })
        .map(|member| member.member_id.clone())
        .collect()
}

pub(crate) fn switchover_target_is_eligible_member(
    member: &MemberRecord,
    cache: &crate::dcs::state::DcsCache,
    observed_at: crate::state::UnixMillis,
) -> bool {
    member_record_is_fresh(member, cache, observed_at)
        && member.role == MemberRole::Replica
        && member.sql == SqlStatus::Healthy
        && member.readiness == Readiness::Ready
}

```

## Source path: `src/ha/actions.rs`

```rust
use crate::state::MemberId;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ActionId {
    AcquireLeaderLease,
    ReleaseLeaderLease,
    ClearSwitchover,
    FollowLeader(String),
    StartRewind,
    StartBaseBackup,
    RunBootstrap,
    FenceNode,
    WipeDataDir,
    SignalFailSafe,
    StartPostgres,
    PromoteToPrimary,
    DemoteToReplica,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum HaAction {
    AcquireLeaderLease,
    ReleaseLeaderLease,
    ClearSwitchover,
    FollowLeader { leader_member_id: String },
    StartRewind { leader_member_id: MemberId },
    StartBaseBackup { leader_member_id: MemberId },
    RunBootstrap,
    FenceNode,
    WipeDataDir,
    SignalFailSafe,
    StartPostgres,
    PromoteToPrimary,
    DemoteToReplica,
}

impl HaAction {
    pub(crate) fn id(&self) -> ActionId {
        match self {
            Self::AcquireLeaderLease => ActionId::AcquireLeaderLease,
            Self::ReleaseLeaderLease => ActionId::ReleaseLeaderLease,
            Self::ClearSwitchover => ActionId::ClearSwitchover,
            Self::FollowLeader { leader_member_id } => {
                ActionId::FollowLeader(leader_member_id.clone())
            }
            Self::StartRewind { .. } => ActionId::StartRewind,
            Self::StartBaseBackup { .. } => ActionId::StartBaseBackup,
            Self::RunBootstrap => ActionId::RunBootstrap,
            Self::FenceNode => ActionId::FenceNode,
            Self::WipeDataDir => ActionId::WipeDataDir,
            Self::SignalFailSafe => ActionId::SignalFailSafe,
            Self::StartPostgres => ActionId::StartPostgres,
            Self::PromoteToPrimary => ActionId::PromoteToPrimary,
            Self::DemoteToReplica => ActionId::DemoteToReplica,
        }
    }
}

impl ActionId {
    pub(crate) fn label(&self) -> String {
        match self {
            Self::AcquireLeaderLease => "acquire_leader_lease".to_string(),
            Self::ReleaseLeaderLease => "release_leader_lease".to_string(),
            Self::ClearSwitchover => "clear_switchover".to_string(),
            Self::FollowLeader(leader) => format!("follow_leader_{leader}"),
            Self::StartRewind => "start_rewind".to_string(),
            Self::StartBaseBackup => "start_basebackup".to_string(),
            Self::RunBootstrap => "run_bootstrap".to_string(),
            Self::FenceNode => "fence_node".to_string(),
            Self::WipeDataDir => "wipe_data_dir".to_string(),
            Self::SignalFailSafe => "signal_failsafe".to_string(),
            Self::StartPostgres => "start_postgres".to_string(),
            Self::PromoteToPrimary => "promote_to_primary".to_string(),
            Self::DemoteToReplica => "demote_to_replica".to_string(),
        }
    }
}
```

## Source path: `src/ha/lower.rs`

```rust
use serde::{Deserialize, Serialize};

use crate::state::MemberId;

use super::decision::{HaDecision, RecoveryStrategy, StepDownPlan};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct HaEffectPlan {
    pub(crate) lease: LeaseEffect,
    pub(crate) switchover: SwitchoverEffect,
    pub(crate) replication: ReplicationEffect,
    pub(crate) postgres: PostgresEffect,
    pub(crate) safety: SafetyEffect,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum LeaseEffect {
    #[default]
    None,
    AcquireLeader,
    ReleaseLeader,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum SwitchoverEffect {
    #[default]
    None,
    ClearRequest,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum ReplicationEffect {
    #[default]
    None,
    FollowLeader {
        leader_member_id: MemberId,
    },
    RecoverReplica {
        strategy: RecoveryStrategy,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum PostgresEffect {
    #[default]
    None,
    Start,
    Promote,
    Demote,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum SafetyEffect {
    #[default]
    None,
    FenceNode,
    SignalFailSafe,
}

impl HaDecision {
    pub(crate) fn lower(&self) -> HaEffectPlan {
        match self {
            Self::NoChange | Self::WaitForDcsTrust => HaEffectPlan::default(),
            Self::WaitForPostgres {
                start_requested, ..
            } => HaEffectPlan {
                postgres: if *start_requested {
                    PostgresEffect::Start
                } else {
                    PostgresEffect::None
                },
                ..HaEffectPlan::default()
            },
            Self::AttemptLeadership => HaEffectPlan {
                lease: LeaseEffect::AcquireLeader,
                ..HaEffectPlan::default()
            },
            Self::FollowLeader { leader_member_id } => HaEffectPlan {
                replication: ReplicationEffect::FollowLeader {
                    leader_member_id: leader_member_id.clone(),
                },
                ..HaEffectPlan::default()
            },
            Self::BecomePrimary { promote } => HaEffectPlan {
                postgres: if *promote {
                    PostgresEffect::Promote
                } else {
                    PostgresEffect::None
                },
                ..HaEffectPlan::default()
            },
            Self::CompleteSwitchover => HaEffectPlan {
                switchover: SwitchoverEffect::ClearRequest,
                ..HaEffectPlan::default()
            },
            Self::StepDown(plan) => lower_step_down(plan),
            Self::RecoverReplica { strategy } => HaEffectPlan {
                replication: ReplicationEffect::RecoverReplica {
                    strategy: strategy.clone(),
                },
                ..HaEffectPlan::default()
            },
            Self::FenceNode => HaEffectPlan {
                safety: SafetyEffect::FenceNode,
                ..HaEffectPlan::default()
            },
            Self::ReleaseLeaderLease { .. } => HaEffectPlan {
                lease: LeaseEffect::ReleaseLeader,
                ..HaEffectPlan::default()
            },
            Self::EnterFailSafe {
                release_leader_lease,
            } => HaEffectPlan {
                lease: if *release_leader_lease {
                    LeaseEffect::ReleaseLeader
                } else {
                    LeaseEffect::None
                },
                safety: SafetyEffect::FenceNode,
                ..HaEffectPlan::default()
            },
        }
    }
}

pub(crate) fn lower_decision(decision: &HaDecision) -> HaEffectPlan {
    decision.lower()
}

impl HaEffectPlan {
    pub(crate) fn len(&self) -> usize {
        self.dispatch_step_count()
    }

    pub(crate) fn dispatch_step_count(&self) -> usize {
        lease_effect_step_count(&self.lease)
            + switchover_effect_step_count(&self.switchover)
            + replication_effect_step_count(&self.replication)
            + postgres_effect_step_count(&self.postgres)
            + safety_effect_step_count(&self.safety)
    }
}

fn lower_step_down(plan: &StepDownPlan) -> HaEffectPlan {
    HaEffectPlan {
        lease: if plan.release_leader_lease {
            LeaseEffect::ReleaseLeader
        } else {
            LeaseEffect::None
        },
        switchover: SwitchoverEffect::None,
        replication: ReplicationEffect::None,
        postgres: PostgresEffect::Demote,
        safety: if plan.fence {
            SafetyEffect::FenceNode
        } else {
            SafetyEffect::None
        },
    }
}

pub(crate) fn lease_effect_step_count(effect: &LeaseEffect) -> usize {
    match effect {
        LeaseEffect::None => 0,
        LeaseEffect::AcquireLeader | LeaseEffect::ReleaseLeader => 1,
    }
}

pub(crate) fn switchover_effect_step_count(effect: &SwitchoverEffect) -> usize {
    match effect {
        SwitchoverEffect::None => 0,
        SwitchoverEffect::ClearRequest => 1,
    }
}

pub(crate) fn replication_effect_step_count(effect: &ReplicationEffect) -> usize {
    match effect {
        ReplicationEffect::None => 0,
        ReplicationEffect::FollowLeader { .. } => 1,
        ReplicationEffect::RecoverReplica { strategy } => match strategy {
            RecoveryStrategy::Rewind { .. } => 1,
            RecoveryStrategy::BaseBackup { .. } | RecoveryStrategy::Bootstrap => 2,
        },
    }
}

pub(crate) fn postgres_effect_step_count(effect: &PostgresEffect) -> usize {
    match effect {
        PostgresEffect::None => 0,
        PostgresEffect::Start | PostgresEffect::Promote | PostgresEffect::Demote => 1,
    }
}

pub(crate) fn safety_effect_step_count(effect: &SafetyEffect) -> usize {
    match effect {
        SafetyEffect::None => 0,
        SafetyEffect::FenceNode | SafetyEffect::SignalFailSafe => 1,
    }
}

```

## Source path: `src/dcs/state.rs`

```rust
use std::collections::BTreeMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{
    config::RuntimeConfig,
    logging::LogHandle,
    pginfo::state::{PgInfoState, Readiness, SqlStatus},
    state::{
        MemberId, StatePublisher, StateSubscriber, TimelineId, UnixMillis, Version, WalLsn,
        WorkerStatus,
    },
};

use super::store::DcsStore;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DcsTrust {
    FullQuorum,
    FailSafe,
    NotTrusted,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum MemberRole {
    Unknown,
    Primary,
    Replica,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct MemberRecord {
    pub(crate) member_id: MemberId,
    pub(crate) postgres_host: String,
    pub(crate) postgres_port: u16,
    pub(crate) api_url: Option<String>,
    pub(crate) role: MemberRole,
    pub(crate) sql: SqlStatus,
    pub(crate) readiness: Readiness,
    pub(crate) timeline: Option<TimelineId>,
    pub(crate) write_lsn: Option<WalLsn>,
    pub(crate) replay_lsn: Option<WalLsn>,
    pub(crate) updated_at: UnixMillis,
    pub(crate) pg_version: Version,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct LeaderRecord {
    pub(crate) member_id: MemberId,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SwitchoverRequest {
    #[serde(default)]
    pub(crate) switchover_to: Option<MemberId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct InitLockRecord {
    pub(crate) holder: MemberId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DcsCache {
    pub(crate) members: BTreeMap<MemberId, MemberRecord>,
    pub(crate) leader: Option<LeaderRecord>,
    pub(crate) switchover: Option<SwitchoverRequest>,
    pub(crate) config: RuntimeConfig,
    pub(crate) init_lock: Option<InitLockRecord>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DcsState {
    pub(crate) worker: WorkerStatus,
    pub(crate) trust: DcsTrust,
    pub(crate) cache: DcsCache,
    pub(crate) last_refresh_at: Option<UnixMillis>,
}

pub(crate) struct DcsWorkerCtx {
    pub(crate) self_id: MemberId,
    pub(crate) scope: String,
    pub(crate) poll_interval: Duration,
    pub(crate) local_postgres_host: String,
    pub(crate) local_postgres_port: u16,
    pub(crate) local_api_url: Option<String>,
    pub(crate) pg_subscriber: StateSubscriber<PgInfoState>,
    pub(crate) publisher: StatePublisher<DcsState>,
    pub(crate) store: Box<dyn DcsStore>,
    pub(crate) log: LogHandle,
    pub(crate) cache: DcsCache,
    pub(crate) last_published_pg_version: Option<Version>,
    pub(crate) last_emitted_store_healthy: Option<bool>,
    pub(crate) last_emitted_trust: Option<DcsTrust>,
}

pub(crate) fn evaluate_trust(
    etcd_healthy: bool,
    cache: &DcsCache,
    self_id: &MemberId,
    now: UnixMillis,
) -> DcsTrust {
    if !etcd_healthy {
        return DcsTrust::NotTrusted;
    }

    let Some(self_member) = cache.members.get(self_id) else {
        return DcsTrust::FailSafe;
    };
    if !member_record_is_fresh(self_member, cache, now) {
        return DcsTrust::FailSafe;
    }

    if !has_fresh_quorum(cache, now) {
        return DcsTrust::FailSafe;
    }

    DcsTrust::FullQuorum
}

pub(crate) fn member_record_is_fresh(
    record: &MemberRecord,
    cache: &DcsCache,
    now: UnixMillis,
) -> bool {
    let max_age_ms = cache.config.ha.lease_ttl_ms;
    now.0.saturating_sub(record.updated_at.0) <= max_age_ms
}

fn fresh_member_count(cache: &DcsCache, now: UnixMillis) -> usize {
    cache
        .members
        .values()
        .filter(|record| member_record_is_fresh(record, cache, now))
        .count()
}

fn has_fresh_quorum(cache: &DcsCache, now: UnixMillis) -> bool {
    let fresh_members = fresh_member_count(cache, now);

    // The current runtime only knows the observed DCS member set. Until there is an explicit
    // configured membership source, multi-member quorum stays conservative: one fresh member is
    // only trusted in a single-member view, and any larger observed view requires at least two
    // fresh members.
    if cache.members.len() <= 1 {
        fresh_members == 1
    } else {
        fresh_members >= 2
    }
}

pub(crate) fn build_local_member_record(
    self_id: &MemberId,
    postgres_host: &str,
    postgres_port: u16,
    api_url: Option<&str>,
    pg_state: &PgInfoState,
    now: UnixMillis,
    pg_version: Version,
) -> MemberRecord {
    match pg_state {
        PgInfoState::Unknown { common } => MemberRecord {
            member_id: self_id.clone(),
            postgres_host: postgres_host.to_string(),
            postgres_port,
            api_url: api_url.map(ToString::to_string),
            role: MemberRole::Unknown,
            sql: common.sql.clone(),
            readiness: common.readiness.clone(),
            timeline: common.timeline,
            write_lsn: None,
            replay_lsn: None,
            updated_at: now,
            pg_version,
        },
        PgInfoState::Primary {
            common, wal_lsn, ..
        } => MemberRecord {
            member_id: self_id.clone(),
            postgres_host: postgres_host.to_string(),
            postgres_port,
            api_url: api_url.map(ToString::to_string),
            role: MemberRole::Primary,
            sql: common.sql.clone(),
            readiness: common.readiness.clone(),
            timeline: common.timeline,
            write_lsn: Some(*wal_lsn),
            replay_lsn: None,
            updated_at: now,
            pg_version,
        },
        PgInfoState::Replica {
            common, replay_lsn, ..
        } => MemberRecord {
            member_id: self_id.clone(),
            postgres_host: postgres_host.to_string(),
            postgres_port,
            api_url: api_url.map(ToString::to_string),
            role: MemberRole::Replica,
            sql: common.sql.clone(),
            readiness: common.readiness.clone(),
            timeline: common.timeline,
            write_lsn: None,
            replay_lsn: Some(*replay_lsn),
            updated_at: now,
            pg_version,
        },
    }
}

```

## Source path: `src/pginfo/state.rs`

```rust
use std::time::Duration;

use serde::{Deserialize, Serialize};

pub(crate) use super::conninfo::{render_pg_conninfo, PgConnInfo, PgSslMode};
use super::query::PgPollData;
use crate::logging::LogHandle;
use crate::state::StatePublisher;
use crate::state::{MemberId, TimelineId, UnixMillis, WalLsn, WorkerStatus};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum SqlStatus {
    Unknown,
    Healthy,
    Unreachable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum Readiness {
    Unknown,
    Ready,
    NotReady,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PgConfig {
    pub(crate) port: Option<u16>,
    pub(crate) hot_standby: Option<bool>,
    pub(crate) primary_conninfo: Option<PgConnInfo>,
    pub(crate) primary_slot_name: Option<String>,
    pub(crate) extra: std::collections::BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ReplicationSlotInfo {
    pub(crate) name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UpstreamInfo {
    pub(crate) member_id: MemberId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PgInfoCommon {
    pub(crate) worker: WorkerStatus,
    pub(crate) sql: SqlStatus,
    pub(crate) readiness: Readiness,
    pub(crate) timeline: Option<TimelineId>,
    pub(crate) pg_config: PgConfig,
    pub(crate) last_refresh_at: Option<UnixMillis>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PgInfoState {
    Unknown {
        common: PgInfoCommon,
    },
    Primary {
        common: PgInfoCommon,
        wal_lsn: WalLsn,
        slots: Vec<ReplicationSlotInfo>,
    },
    Replica {
        common: PgInfoCommon,
        replay_lsn: WalLsn,
        follow_lsn: Option<WalLsn>,
        upstream: Option<UpstreamInfo>,
    },
}

#[derive(Clone, Debug)]
pub(crate) struct PgInfoWorkerCtx {
    pub(crate) self_id: MemberId,
    pub(crate) postgres_conninfo: PgConnInfo,
    pub(crate) poll_interval: Duration,
    pub(crate) publisher: StatePublisher<PgInfoState>,
    pub(crate) log: LogHandle,
    pub(crate) last_emitted_sql_status: Option<SqlStatus>,
}

pub(crate) fn derive_readiness(sql: &SqlStatus, is_ready: bool) -> Readiness {
    match sql {
        SqlStatus::Healthy => {
            if is_ready {
                Readiness::Ready
            } else {
                Readiness::NotReady
            }
        }
        SqlStatus::Unknown => Readiness::Unknown,
        SqlStatus::Unreachable => Readiness::NotReady,
    }
}

pub(crate) fn to_member_status(
    worker_status: WorkerStatus,
    sql_status: SqlStatus,
    polled_at: UnixMillis,
    poll: Option<PgPollData>,
) -> PgInfoState {
    let readiness_signal = poll.as_ref().map(|value| value.is_ready).unwrap_or(false);
    let timeline = poll.as_ref().and_then(|value| value.timeline);
    let common = PgInfoCommon {
        worker: worker_status,
        sql: sql_status.clone(),
        readiness: derive_readiness(&sql_status, readiness_signal),
        timeline,
        pg_config: PgConfig {
            port: None,
            hot_standby: None,
            primary_conninfo: None,
            primary_slot_name: None,
            extra: std::collections::BTreeMap::new(),
        },
        last_refresh_at: Some(polled_at),
    };

    let Some(polled) = poll else {
        return PgInfoState::Unknown { common };
    };

    if polled.in_recovery {
        return PgInfoState::Replica {
            common,
            replay_lsn: polled
                .replay_lsn
                .or(polled.receive_lsn)
                .unwrap_or(WalLsn(0)),
            follow_lsn: polled.receive_lsn,
            upstream: None,
        };
    }

    if let Some(wal_lsn) = polled.current_wal_lsn {
        return PgInfoState::Primary {
            common,
            wal_lsn,
            slots: polled
                .slot_names
                .into_iter()
                .map(|name| ReplicationSlotInfo { name })
                .collect(),
        };
    }

    PgInfoState::Unknown { common }
}

```

## Source path: `src/process/state.rs`

```rust
use std::time::Duration;

use tokio::sync::mpsc::UnboundedReceiver;

use crate::{
    config::ProcessConfig,
    logging::LogHandle,
    state::{JobId, StatePublisher, UnixMillis, WorkerError, WorkerStatus},
};

use super::jobs::{
    ActiveJob, ActiveJobKind, BaseBackupSpec, BootstrapSpec, DemoteSpec, FencingSpec, PgRewindSpec,
    ProcessCommandRunner, ProcessError, ProcessHandle, ProcessLogIdentity, PromoteSpec,
    StartPostgresSpec,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ProcessState {
    Idle {
        worker: WorkerStatus,
        last_outcome: Option<JobOutcome>,
    },
    Running {
        worker: WorkerStatus,
        active: ActiveJob,
    },
}

impl ProcessState {
    #[cfg(test)]
    pub(crate) fn running_job_id(&self) -> Option<&JobId> {
        match self {
            Self::Idle { .. } => None,
            Self::Running { active, .. } => Some(&active.id),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ProcessJobKind {
    Bootstrap(BootstrapSpec),
    BaseBackup(BaseBackupSpec),
    PgRewind(PgRewindSpec),
    Promote(PromoteSpec),
    Demote(DemoteSpec),
    StartPostgres(StartPostgresSpec),
    Fencing(FencingSpec),
}

impl ProcessJobKind {
    pub(crate) fn label(&self) -> &'static str {
        match self {
            Self::Bootstrap(_) => "bootstrap",
            Self::BaseBackup(_) => "basebackup",
            Self::PgRewind(_) => "pg_rewind",
            Self::Promote(_) => "promote",
            Self::Demote(_) => "demote",
            Self::StartPostgres(_) => "start_postgres",
            Self::Fencing(_) => "fencing",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessJobRequest {
    pub(crate) id: JobId,
    pub(crate) kind: ProcessJobKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessJobRejection {
    pub(crate) id: JobId,
    pub(crate) error: ProcessError,
    pub(crate) rejected_at: UnixMillis,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum JobOutcome {
    Success {
        id: JobId,
        job_kind: ActiveJobKind,
        finished_at: UnixMillis,
    },
    Failure {
        id: JobId,
        job_kind: ActiveJobKind,
        error: ProcessError,
        finished_at: UnixMillis,
    },
    Timeout {
        id: JobId,
        job_kind: ActiveJobKind,
        finished_at: UnixMillis,
    },
}

pub(crate) struct ActiveRuntime {
    pub(crate) request: ProcessJobRequest,
    pub(crate) deadline_at: UnixMillis,
    pub(crate) handle: Box<dyn ProcessHandle>,
    pub(crate) log_identity: ProcessLogIdentity,
}

pub(crate) struct ProcessWorkerCtx {
    pub(crate) poll_interval: Duration,
    pub(crate) config: ProcessConfig,
    pub(crate) log: LogHandle,
    pub(crate) capture_subprocess_output: bool,
    pub(crate) state: ProcessState,
    pub(crate) publisher: StatePublisher<ProcessState>,
    pub(crate) inbox: UnboundedReceiver<ProcessJobRequest>,
    pub(crate) inbox_disconnected_logged: bool,
    pub(crate) command_runner: Box<dyn ProcessCommandRunner>,
    pub(crate) active_runtime: Option<ActiveRuntime>,
    pub(crate) last_rejection: Option<ProcessJobRejection>,
    pub(crate) now: Box<dyn FnMut() -> Result<UnixMillis, WorkerError> + Send>,
}

impl ProcessWorkerCtx {
    #[cfg(test)]
    pub(crate) fn contract_stub(
        config: ProcessConfig,
        publisher: StatePublisher<ProcessState>,
        inbox: UnboundedReceiver<ProcessJobRequest>,
    ) -> Self {
        Self {
            poll_interval: Duration::from_millis(10),
            config,
            log: LogHandle::null(),
            capture_subprocess_output: false,
            state: ProcessState::Idle {
                worker: WorkerStatus::Starting,
                last_outcome: None,
            },
            publisher,
            inbox,
            inbox_disconnected_logged: false,
            command_runner: Box::new(crate::process::jobs::NoopCommandRunner),
            active_runtime: None,
            last_rejection: None,
            now: Box::new(|| Ok(UnixMillis(0))),
        }
    }
}
```

## Source path: `src/process/jobs.rs`

```rust
use std::{future::Future, path::PathBuf, pin::Pin};

use thiserror::Error;

use crate::config::{resolve_secret_string, RoleAuthConfig, SecretSource};
use crate::pginfo::state::PgConnInfo;
use crate::state::{JobId, UnixMillis};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BootstrapSpec {
    pub(crate) data_dir: PathBuf,
    pub(crate) superuser_username: String,
    pub(crate) timeout_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ReplicatorSourceConn {
    pub(crate) conninfo: PgConnInfo,
    pub(crate) auth: RoleAuthConfig,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RewinderSourceConn {
    pub(crate) conninfo: PgConnInfo,
    pub(crate) auth: RoleAuthConfig,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PgRewindSpec {
    pub(crate) target_data_dir: PathBuf,
    pub(crate) source: RewinderSourceConn,
    pub(crate) timeout_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BaseBackupSpec {
    pub(crate) data_dir: PathBuf,
    pub(crate) source: ReplicatorSourceConn,
    pub(crate) timeout_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PromoteSpec {
    pub(crate) data_dir: PathBuf,
    pub(crate) wait_seconds: Option<u64>,
    pub(crate) timeout_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DemoteSpec {
    pub(crate) data_dir: PathBuf,
    pub(crate) mode: ShutdownMode,
    pub(crate) timeout_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct StartPostgresSpec {
    pub(crate) data_dir: PathBuf,
    pub(crate) config_file: PathBuf,
    pub(crate) log_file: PathBuf,
    pub(crate) wait_seconds: Option<u64>,
    pub(crate) timeout_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FencingSpec {
    pub(crate) data_dir: PathBuf,
    pub(crate) mode: ShutdownMode,
    pub(crate) timeout_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ShutdownMode {
    Fast,
    Immediate,
}

impl ShutdownMode {
    pub(crate) fn as_pg_ctl_arg(&self) -> &'static str {
        match self {
            Self::Fast => "fast",
            Self::Immediate => "immediate",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ActiveJobKind {
    Bootstrap,
    BaseBackup,
    PgRewind,
    Promote,
    Demote,
    StartPostgres,
    Fencing,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ActiveJob {
    pub(crate) id: JobId,
    pub(crate) kind: ActiveJobKind,
    pub(crate) started_at: UnixMillis,
    pub(crate) deadline_at: UnixMillis,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessCommandSpec {
    pub(crate) program: PathBuf,
    pub(crate) args: Vec<String>,
    pub(crate) env: Vec<ProcessEnvVar>,
    pub(crate) capture_output: bool,
    pub(crate) log_identity: ProcessLogIdentity,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessEnvVar {
    pub(crate) key: String,
    pub(crate) value: ProcessEnvValue,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ProcessEnvValue {
    Secret(SecretSource),
}

impl ProcessEnvValue {
    pub(crate) fn resolve_string_for_key(&self, key: &str) -> Result<String, ProcessError> {
        match self {
            Self::Secret(secret) => resolve_secret_source_string(key, secret),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessLogIdentity {
    pub(crate) job_id: JobId,
    pub(crate) job_kind: String,
    pub(crate) binary: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProcessOutputStream {
    Stdout,
    Stderr,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessOutputLine {
    pub(crate) stream: ProcessOutputStream,
    pub(crate) bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ProcessExit {
    Success,
    Failure { code: Option<i32> },
}

pub(crate) trait ProcessHandle: Send {
    fn poll_exit(&mut self) -> Result<Option<ProcessExit>, ProcessError>;
    fn drain_output<'a>(
        &'a mut self,
        max_bytes: usize,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<ProcessOutputLine>, ProcessError>> + Send + 'a>>;
    fn cancel<'a>(
        &'a mut self,
    ) -> Pin<Box<dyn Future<Output = Result<(), ProcessError>> + Send + 'a>>;
}

pub(crate) trait ProcessCommandRunner: Send {
    fn spawn(&mut self, spec: ProcessCommandSpec) -> Result<Box<dyn ProcessHandle>, ProcessError>;
}

```

# HA control flow

## Source path: `src/ha/decide.rs`

```rust
use crate::{dcs::state::DcsTrust, process::jobs::ActiveJobKind, state::MemberId};

use super::{
    decision::{
        DecisionFacts, HaDecision, LeaseReleaseReason, PhaseOutcome, ProcessActivity,
        RecoveryStrategy, StepDownPlan, StepDownReason,
    },
    state::{DecideInput, DecideOutput, HaPhase, HaState},
};

pub(crate) fn decide(input: DecideInput) -> DecideOutput {
    let facts = DecisionFacts::from_world(&input.world);
    let current = input.current;
    let outcome = decide_phase(&current, &facts);
    let next = HaState {
        worker: current.worker,
        phase: outcome.next_phase.clone(),
        tick: current.tick.saturating_add(1),
        decision: outcome.decision.clone(),
    };

    DecideOutput { next, outcome }
}

pub(crate) fn decide_phase(current: &HaState, facts: &DecisionFacts) -> PhaseOutcome {
    if !matches!(facts.trust, DcsTrust::FullQuorum) {
        if facts.postgres_primary {
            return PhaseOutcome::new(
                HaPhase::FailSafe,
                HaDecision::EnterFailSafe {
                    release_leader_lease: false,
                },
            );
        }
        return PhaseOutcome::new(HaPhase::FailSafe, HaDecision::NoChange);
    }

    match current.phase {
        HaPhase::Init => PhaseOutcome::new(
            HaPhase::WaitingPostgresReachable,
            HaDecision::WaitForPostgres {
                start_requested: false,
                leader_member_id: None,
            },
        ),
        HaPhase::WaitingPostgresReachable => decide_waiting_postgres_reachable(facts),
        HaPhase::WaitingDcsTrusted => decide_waiting_dcs_trusted(current, facts),
        HaPhase::WaitingSwitchoverSuccessor => decide_waiting_switchover_successor(facts),
        HaPhase::Replica => decide_replica(facts),
        HaPhase::CandidateLeader => decide_candidate_leader(facts),
        HaPhase::Primary => decide_primary(current, facts),
        HaPhase::Rewinding => decide_rewinding(facts),
        HaPhase::Bootstrapping => decide_bootstrapping(facts),
        HaPhase::Fencing => decide_fencing(facts),
        HaPhase::FailSafe => decide_fail_safe(current, facts),
    }
}

fn decide_waiting_postgres_reachable(facts: &DecisionFacts) -> PhaseOutcome {
    if facts.postgres_reachable {
        return PhaseOutcome::new(HaPhase::WaitingDcsTrusted, HaDecision::WaitForDcsTrust);
    }

    if completed_start_postgres_successfully(facts) {
        return PhaseOutcome::new(HaPhase::WaitingDcsTrusted, HaDecision::WaitForDcsTrust);
    }

    wait_for_postgres(facts)
}

fn decide_waiting_dcs_trusted(current: &HaState, facts: &DecisionFacts) -> PhaseOutcome {
    if !facts.postgres_reachable {
        let released_after_fencing = matches!(
            current.decision,
            HaDecision::ReleaseLeaderLease {
                reason: LeaseReleaseReason::FencingComplete,
            }
        );
        if released_after_fencing {
            if let Some(leader_member_id) =
                recovery_leader_member_id(facts).or_else(|| other_leader_record(facts))
            {
                return PhaseOutcome::new(
                    HaPhase::Bootstrapping,
                    HaDecision::RecoverReplica {
                        strategy: RecoveryStrategy::BaseBackup { leader_member_id },
                    },
                );
            }

            return PhaseOutcome::new(HaPhase::WaitingDcsTrusted, HaDecision::WaitForDcsTrust);
        }

        if waiting_for_pginfo_after_successful_start(facts) {
            return PhaseOutcome::new(HaPhase::WaitingDcsTrusted, HaDecision::WaitForDcsTrust);
        }

        return wait_for_postgres(facts);
    }

    if facts.active_leader_member_id.as_ref() == Some(&facts.self_member_id) {
        return PhaseOutcome::new(
            HaPhase::Primary,
            HaDecision::BecomePrimary { promote: false },
        );
    }

    match follow_target(facts) {
        Some(leader_member_id) => PhaseOutcome::new(
            HaPhase::Replica,
            HaDecision::FollowLeader { leader_member_id },
        ),
        None if !facts.postgres_primary => {
            PhaseOutcome::new(HaPhase::WaitingDcsTrusted, HaDecision::WaitForDcsTrust)
        }
        None => PhaseOutcome::new(HaPhase::CandidateLeader, HaDecision::AttemptLeadership),
    }
}

fn decide_waiting_switchover_successor(facts: &DecisionFacts) -> PhaseOutcome {
    if facts
        .leader_member_id
        .as_ref()
        .map(|leader_member_id| leader_member_id == &facts.self_member_id)
        .unwrap_or(true)
    {
        return PhaseOutcome::new(
            HaPhase::WaitingSwitchoverSuccessor,
            HaDecision::WaitForDcsTrust,
        );
    }

    if !facts.postgres_reachable {
        return wait_for_postgres(facts);
    }

    match follow_target(facts) {
        Some(leader_member_id) => PhaseOutcome::new(
            HaPhase::Replica,
            HaDecision::FollowLeader { leader_member_id },
        ),
        None => PhaseOutcome::new(
            HaPhase::WaitingSwitchoverSuccessor,
            HaDecision::WaitForDcsTrust,
        ),
    }
}

fn decide_replica(facts: &DecisionFacts) -> PhaseOutcome {
    if !facts.postgres_reachable {
        return wait_for_postgres(facts);
    }

    if facts.switchover_pending
        && facts.active_leader_member_id.as_ref() == Some(&facts.self_member_id)
    {
        return PhaseOutcome::new(HaPhase::Replica, HaDecision::NoChange);
    }

    match facts.active_leader_member_id.as_ref() {
        Some(leader_member_id) if leader_member_id == &facts.self_member_id => PhaseOutcome::new(
            HaPhase::Primary,
            HaDecision::BecomePrimary { promote: true },
        ),
        Some(leader_member_id) if facts.rewind_required => PhaseOutcome::new(
            HaPhase::Rewinding,
            HaDecision::RecoverReplica {
                strategy: RecoveryStrategy::Rewind {
                    leader_member_id: leader_member_id.clone(),
                },
            },
        ),
        Some(leader_member_id) => PhaseOutcome::new(
            HaPhase::Replica,
            HaDecision::FollowLeader {
                leader_member_id: leader_member_id.clone(),
            },
        ),
        None if targeted_switchover_blocks_leadership_attempt(facts) => {
            PhaseOutcome::new(HaPhase::Replica, HaDecision::WaitForDcsTrust)
        }
        None => PhaseOutcome::new(HaPhase::CandidateLeader, HaDecision::AttemptLeadership),
    }
}

fn decide_candidate_leader(facts: &DecisionFacts) -> PhaseOutcome {
    if !facts.postgres_reachable {
        return wait_for_postgres(facts);
    }

    if facts.i_am_leader {
        return PhaseOutcome::new(
            HaPhase::Primary,
            HaDecision::BecomePrimary {
                promote: !facts.postgres_primary,
            },
        );
    }

    if let Some(leader_member_id) = follow_target(facts) {
        return PhaseOutcome::new(
            HaPhase::Replica,
            HaDecision::FollowLeader { leader_member_id },
        );
    }

    if targeted_switchover_blocks_leadership_attempt(facts) {
        return PhaseOutcome::new(HaPhase::CandidateLeader, HaDecision::WaitForDcsTrust);
    }

    PhaseOutcome::new(HaPhase::CandidateLeader, HaDecision::AttemptLeadership)
}

fn decide_primary(current: &HaState, facts: &DecisionFacts) -> PhaseOutcome {
    if switchover_completion_observed(current, facts) {
        return PhaseOutcome::new(HaPhase::Primary, HaDecision::CompleteSwitchover);
    }

    if facts.switchover_pending && facts.i_am_leader {
        return PhaseOutcome::new(
            HaPhase::WaitingSwitchoverSuccessor,
            HaDecision::StepDown(StepDownPlan {
                reason: StepDownReason::Switchover,
                release_leader_lease: true,
                fence: false,
            }),
        );
    }

    if !facts.postgres_reachable {
        if facts.i_am_leader {
            return PhaseOutcome::new(
                HaPhase::Rewinding,
                HaDecision::ReleaseLeaderLease {
                    reason: LeaseReleaseReason::PostgresUnreachable,
                },
            );
        }
        return match recovery_leader_member_id(facts) {
            Some(leader_member_id) => PhaseOutcome::new(
                HaPhase::Rewinding,
                HaDecision::RecoverReplica {
                    strategy: RecoveryStrategy::Rewind { leader_member_id },
                },
            ),
            None => PhaseOutcome::new(HaPhase::Rewinding, HaDecision::NoChange),
        };
    }

    match other_leader_record(facts) {
        Some(leader_member_id) => PhaseOutcome::new(
            HaPhase::Fencing,
            HaDecision::StepDown(StepDownPlan {
                reason: StepDownReason::ForeignLeaderDetected { leader_member_id },
                release_leader_lease: true,
                fence: true,
            }),
        ),
        None => {
            if facts.i_am_leader {
                PhaseOutcome::new(HaPhase::Primary, HaDecision::NoChange)
            } else {
                PhaseOutcome::new(HaPhase::Primary, HaDecision::AttemptLeadership)
            }
        }
    }
}

fn decide_rewinding(facts: &DecisionFacts) -> PhaseOutcome {
    match facts.rewind_activity() {
        ProcessActivity::Running => PhaseOutcome::new(HaPhase::Rewinding, HaDecision::NoChange),
        ProcessActivity::IdleSuccess => match follow_target(facts) {
            Some(leader_member_id) => PhaseOutcome::new(
                HaPhase::Replica,
                HaDecision::FollowLeader { leader_member_id },
            ),
            None => PhaseOutcome::new(HaPhase::Replica, HaDecision::NoChange),
        },
        ProcessActivity::IdleFailure => match recovery_after_rewind_failure(facts) {
            Some(strategy) => PhaseOutcome::new(
                HaPhase::Bootstrapping,
                HaDecision::RecoverReplica { strategy },
            ),
            None => PhaseOutcome::new(HaPhase::Rewinding, HaDecision::NoChange),
        },
        ProcessActivity::IdleNoOutcome => match recovery_leader_member_id(facts) {
            Some(leader_member_id) => PhaseOutcome::new(
                HaPhase::Rewinding,
                HaDecision::RecoverReplica {
                    strategy: RecoveryStrategy::Rewind { leader_member_id },
                },
            ),
            None => PhaseOutcome::new(HaPhase::Rewinding, HaDecision::NoChange),
        },
    }
}

fn decide_bootstrapping(facts: &DecisionFacts) -> PhaseOutcome {
    match facts.bootstrap_activity() {
        ProcessActivity::Running => PhaseOutcome::new(HaPhase::Bootstrapping, HaDecision::NoChange),
        ProcessActivity::IdleSuccess => wait_for_postgres(facts),
        ProcessActivity::IdleFailure => PhaseOutcome::new(HaPhase::Fencing, HaDecision::FenceNode),
        ProcessActivity::IdleNoOutcome => match recovery_after_rewind_failure(facts) {
            Some(strategy) => PhaseOutcome::new(
                HaPhase::Bootstrapping,
                HaDecision::RecoverReplica { strategy },
            ),
            None => PhaseOutcome::new(HaPhase::Bootstrapping, HaDecision::NoChange),
        },
    }
}

fn decide_fencing(facts: &DecisionFacts) -> PhaseOutcome {
    match facts.fencing_activity() {
        ProcessActivity::Running => PhaseOutcome::new(HaPhase::Fencing, HaDecision::NoChange),
        ProcessActivity::IdleSuccess => PhaseOutcome::new(
            HaPhase::WaitingDcsTrusted,
            HaDecision::ReleaseLeaderLease {
                reason: LeaseReleaseReason::FencingComplete,
            },
        ),
        ProcessActivity::IdleFailure => PhaseOutcome::new(
            HaPhase::FailSafe,
            HaDecision::EnterFailSafe {
                release_leader_lease: false,
            },
        ),
        ProcessActivity::IdleNoOutcome => {
            PhaseOutcome::new(HaPhase::Fencing, HaDecision::FenceNode)
        }
    }
}

fn decide_fail_safe(current: &HaState, facts: &DecisionFacts) -> PhaseOutcome {
    match facts.fencing_activity() {
        ProcessActivity::Running => PhaseOutcome::new(HaPhase::FailSafe, HaDecision::NoChange),
        _ if facts.postgres_primary => decide_primary(current, facts),
        _ if facts.i_am_leader => PhaseOutcome::new(
            HaPhase::FailSafe,
            HaDecision::ReleaseLeaderLease {
                reason: LeaseReleaseReason::FencingComplete,
            },
        ),
        _ => PhaseOutcome::new(HaPhase::WaitingDcsTrusted, HaDecision::WaitForDcsTrust),
    }
}

fn wait_for_postgres(facts: &DecisionFacts) -> PhaseOutcome {
    let recovery_leader_member_id = recovery_leader_member_id(facts);
    let has_unvalidated_foreign_leader =
        recovery_leader_member_id.is_none() && other_leader_record(facts).is_some();

    PhaseOutcome::new(
        HaPhase::WaitingPostgresReachable,
        HaDecision::WaitForPostgres {
            start_requested: facts.start_postgres_can_be_requested()
                && !has_unvalidated_foreign_leader,
            leader_member_id: recovery_leader_member_id.or_else(|| other_leader_record(facts)),
        },
    )
}

fn recovery_after_rewind_failure(facts: &DecisionFacts) -> Option<RecoveryStrategy> {
    recovery_leader_member_id(facts)
        .map(|leader_member_id| RecoveryStrategy::BaseBackup { leader_member_id })
}

fn recovery_leader_member_id(facts: &DecisionFacts) -> Option<MemberId> {
    facts
        .followable_member_id
        .clone()
        .filter(|leader_member_id| leader_member_id != &facts.self_member_id)
}

fn follow_target(facts: &DecisionFacts) -> Option<MemberId> {
    facts
        .followable_member_id
        .clone()
        .filter(|leader_member_id| leader_member_id != &facts.self_member_id)
}

fn targeted_switchover_blocks_leadership_attempt(facts: &DecisionFacts) -> bool {
    match facts.pending_switchover_target.as_ref() {
        Some(target_member_id) => {
            target_member_id != &facts.self_member_id
                || !facts.switchover_target_is_eligible(target_member_id)
        }
        None => false,
    }
}

fn switchover_completion_observed(current: &HaState, facts: &DecisionFacts) -> bool {
    if !facts.switchover_pending || !facts.i_am_leader {
        return false;
    }

    match facts.pending_switchover_target.as_ref() {
        Some(target_member_id) => target_member_id == &facts.self_member_id,
        None => matches!(
            current.decision,
            HaDecision::BecomePrimary { .. } | HaDecision::CompleteSwitchover
        ),
    }
}

fn other_leader_record(facts: &DecisionFacts) -> Option<MemberId> {
    facts
        .leader_member_id
        .clone()
        .filter(|leader_member_id| leader_member_id != &facts.self_member_id)
}

fn completed_start_postgres_successfully(facts: &DecisionFacts) -> bool {
    matches!(
        &facts.process_state,
        crate::process::state::ProcessState::Idle {
            last_outcome: Some(crate::process::state::JobOutcome::Success {
                job_kind: ActiveJobKind::StartPostgres,
                ..
            }),
            ..
        }
    )
}

fn waiting_for_pginfo_after_successful_start(facts: &DecisionFacts) -> bool {
    matches!(
        &facts.process_state,
        crate::process::state::ProcessState::Idle {
            last_outcome: Some(
                crate::process::state::JobOutcome::Success {
                    job_kind: ActiveJobKind::StartPostgres,
                    finished_at,
                    ..
                }
            ),
            ..
        } if *finished_at >= facts.pg_observed_at
    )
}

```

## Source path: `src/ha/worker.rs`

```rust
use crate::{
    process::{jobs::ActiveJobKind, state::ProcessState},
    state::{WorkerError, WorkerStatus},
};

use super::{
    apply::{apply_effect_plan, format_dispatch_errors},
    decide::decide,
    events::{
        emit_ha_decision_selected, emit_ha_effect_plan_selected, emit_ha_phase_transition,
        emit_ha_role_transition, ha_role_label,
    },
    state::{DecideInput, HaWorkerCtx, WorldSnapshot},
};

pub(crate) async fn run(mut ctx: HaWorkerCtx) -> Result<(), WorkerError> {
    let mut interval = tokio::time::interval(ctx.poll_interval);
    loop {
        tokio::select! {
            changed = ctx.pg_subscriber.changed() => {
                changed.map_err(|err| {
                    WorkerError::Message(format!("ha pg subscriber closed: {err}"))
                })?;
            }
            changed = ctx.dcs_subscriber.changed() => {
                changed.map_err(|err| {
                    WorkerError::Message(format!("ha dcs subscriber closed: {err}"))
                })?;
            }
            changed = ctx.process_subscriber.changed() => {
                changed.map_err(|err| {
                    WorkerError::Message(format!("ha process subscriber closed: {err}"))
                })?;
            }
            changed = ctx.config_subscriber.changed() => {
                changed.map_err(|err| {
                    WorkerError::Message(format!("ha config subscriber closed: {err}"))
                })?;
            }
            _ = interval.tick() => {}
        }
        step_once(&mut ctx).await?;
    }
}

pub(crate) async fn step_once(ctx: &mut HaWorkerCtx) -> Result<(), WorkerError> {
    let prev_phase = ctx.state.phase.clone();
    let world = world_snapshot(ctx);
    let process_state = world.process.value.clone();
    let output = decide(DecideInput {
        current: ctx.state.clone(),
        world,
    });
    let plan = output.outcome.decision.lower();
    let skip_redundant_process_dispatch =
        should_skip_redundant_process_dispatch(&ctx.state, &output.next, &process_state);

    emit_ha_decision_selected(ctx, output.next.tick, &output.outcome.decision, &plan)?;
    emit_ha_effect_plan_selected(ctx, output.next.tick, &plan)?;
    let published_next = crate::ha::state::HaState {
        worker: WorkerStatus::Running,
        ..output.next.clone()
    };
    let now = (ctx.now)()?;

    ctx.publisher
        .publish(published_next.clone(), now)
        .map_err(|err| WorkerError::Message(format!("ha publish failed: {err}")))?;

    if prev_phase != published_next.phase {
        emit_ha_phase_transition(ctx, published_next.tick, &prev_phase, &published_next.phase)?;
    }

    let prev_role = ha_role_label(&prev_phase);
    let next_role = ha_role_label(&published_next.phase);
    if prev_role != next_role {
        emit_ha_role_transition(ctx, published_next.tick, prev_role, next_role)?;
    }

    ctx.state = published_next.clone();

    let dispatch_errors = if skip_redundant_process_dispatch {
        Vec::new()
    } else {
        apply_effect_plan(ctx, published_next.tick, &plan)?
    };
    if !dispatch_errors.is_empty() {
        let faulted = crate::ha::state::HaState {
            worker: WorkerStatus::Faulted(WorkerError::Message(format_dispatch_errors(
                &dispatch_errors,
            ))),
            ..published_next
        };
        let faulted_now = (ctx.now)()?;
        ctx.publisher
            .publish(faulted.clone(), faulted_now)
            .map_err(|err| WorkerError::Message(format!("ha publish failed: {err}")))?;
        ctx.state = faulted;
    }

    Ok(())
}

fn world_snapshot(ctx: &HaWorkerCtx) -> WorldSnapshot {
    WorldSnapshot {
        config: ctx.config_subscriber.latest(),
        pg: ctx.pg_subscriber.latest(),
        dcs: ctx.dcs_subscriber.latest(),
        process: ctx.process_subscriber.latest(),
    }
}

fn should_skip_redundant_process_dispatch(
    current: &crate::ha::state::HaState,
    next: &crate::ha::state::HaState,
    process_state: &ProcessState,
) -> bool {
    current.phase == next.phase
        && current.decision == next.decision
        && decision_is_already_active(&next.decision, process_state)
}

fn decision_is_already_active(
    decision: &crate::ha::decision::HaDecision,
    process_state: &ProcessState,
) -> bool {
    match decision {
        crate::ha::decision::HaDecision::WaitForPostgres {
            start_requested: true,
            ..
        } => process_state_is_running_one_of(process_state, &[ActiveJobKind::StartPostgres]),
        crate::ha::decision::HaDecision::RecoverReplica { strategy } => match strategy {
            crate::ha::decision::RecoveryStrategy::Rewind { .. } => {
                process_state_is_running_one_of(process_state, &[ActiveJobKind::PgRewind])
            }
            crate::ha::decision::RecoveryStrategy::BaseBackup { .. } => {
                process_state_is_running_one_of(process_state, &[ActiveJobKind::BaseBackup])
            }
            crate::ha::decision::RecoveryStrategy::Bootstrap => {
                process_state_is_running_one_of(process_state, &[ActiveJobKind::Bootstrap])
            }
        },
        crate::ha::decision::HaDecision::FenceNode => {
            process_state_is_running_one_of(process_state, &[ActiveJobKind::Fencing])
        }
        _ => false,
    }
}

fn process_state_is_running_one_of(
    process_state: &ProcessState,
    expected_kinds: &[ActiveJobKind],
) -> bool {
    match process_state {
        ProcessState::Running { active, .. } => expected_kinds.contains(&active.kind),
        ProcessState::Idle { .. } => false,
    }
}

```

## Source path: `src/ha/process_dispatch.rs`

```rust
use std::{fs, path::Path};

use thiserror::Error;

use crate::{
    config::RuntimeConfig,
    dcs::state::MemberRecord,
    ha::decision::HaDecision,
    postgres_managed_conf::{managed_standby_auth_from_role_auth, ManagedPostgresStartIntent},
    process::{
        jobs::{
            BaseBackupSpec, BootstrapSpec, DemoteSpec, FencingSpec, PgRewindSpec, PromoteSpec,
            ShutdownMode, StartPostgresSpec,
        },
        state::{ProcessJobKind, ProcessJobRequest},
    },
    state::{JobId, MemberId},
};

use super::{
    actions::{ActionId, HaAction},
    source_conn::{basebackup_source_from_member, rewind_source_from_member},
    state::HaWorkerCtx,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProcessDispatchOutcome {
    Applied,
    Skipped,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(crate) enum ProcessDispatchError {
    #[error("process send failed for action `{action:?}`: {message}")]
    ProcessSend { action: ActionId, message: String },
    #[error("managed config materialization failed for action `{action:?}`: {message}")]
    ManagedConfig { action: ActionId, message: String },
    #[error("filesystem operation failed for action `{action:?}`: {message}")]
    Filesystem { action: ActionId, message: String },
    #[error("remote source selection failed for action `{action:?}`: {message}")]
    SourceSelection { action: ActionId, message: String },
    #[error("process dispatch does not support action `{action:?}`")]
    UnsupportedAction { action: ActionId },
}

pub(crate) fn dispatch_process_action(
    ctx: &mut HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    action: &HaAction,
    runtime_config: &RuntimeConfig,
) -> Result<ProcessDispatchOutcome, ProcessDispatchError> {
    match action {
        HaAction::AcquireLeaderLease | HaAction::ReleaseLeaderLease | HaAction::ClearSwitchover => {
            Err(ProcessDispatchError::UnsupportedAction {
                action: action.id(),
            })
        }
        HaAction::StartPostgres => {
            let start_intent = start_intent_from_dcs(
                ctx,
                start_postgres_leader_member_id(ctx),
                runtime_config.postgres.data_dir.as_path(),
            )?;
            let managed = crate::postgres_managed::materialize_managed_postgres_config(
                runtime_config,
                &start_intent,
            )
            .map_err(|err| ProcessDispatchError::ManagedConfig {
                action: action.id(),
                message: err.to_string(),
            })?;
            let request = ProcessJobRequest {
                id: process_job_id(&ctx.scope, &ctx.self_id, action, action_index, ha_tick),
                kind: ProcessJobKind::StartPostgres(StartPostgresSpec {
                    data_dir: runtime_config.postgres.data_dir.clone(),
                    config_file: managed.postgresql_conf_path,
                    log_file: ctx.process_defaults.log_file.clone(),
                    wait_seconds: None,
                    timeout_ms: None,
                }),
            };
            send_process_request(ctx, action.id(), request)?;
            Ok(ProcessDispatchOutcome::Applied)
        }
        HaAction::PromoteToPrimary => {
            let request = ProcessJobRequest {
                id: process_job_id(&ctx.scope, &ctx.self_id, action, action_index, ha_tick),
                kind: ProcessJobKind::Promote(PromoteSpec {
                    data_dir: runtime_config.postgres.data_dir.clone(),
                    wait_seconds: None,
                    timeout_ms: None,
                }),
            };
            send_process_request(ctx, action.id(), request)?;
            Ok(ProcessDispatchOutcome::Applied)
        }
        HaAction::DemoteToReplica => {
            let request = ProcessJobRequest {
                id: process_job_id(&ctx.scope, &ctx.self_id, action, action_index, ha_tick),
                kind: ProcessJobKind::Demote(DemoteSpec {
                    data_dir: runtime_config.postgres.data_dir.clone(),
                    mode: ctx.process_defaults.shutdown_mode.clone(),
                    timeout_ms: None,
                }),
            };
            send_process_request(ctx, action.id(), request)?;
            Ok(ProcessDispatchOutcome::Applied)
        }
        HaAction::StartRewind { leader_member_id } => {
            let source = validate_rewind_source(ctx, action.id(), leader_member_id)?;
            let request = ProcessJobRequest {
                id: process_job_id(&ctx.scope, &ctx.self_id, action, action_index, ha_tick),
                kind: ProcessJobKind::PgRewind(PgRewindSpec {
                    target_data_dir: runtime_config.postgres.data_dir.clone(),
                    source,
                    timeout_ms: None,
                }),
            };
            send_process_request(ctx, action.id(), request)?;
            Ok(ProcessDispatchOutcome::Applied)
        }
        HaAction::StartBaseBackup { leader_member_id } => {
            let source = validate_basebackup_source(ctx, action.id(), leader_member_id)?;
            let request = ProcessJobRequest {
                id: process_job_id(&ctx.scope, &ctx.self_id, action, action_index, ha_tick),
                kind: ProcessJobKind::BaseBackup(BaseBackupSpec {
                    data_dir: runtime_config.postgres.data_dir.clone(),
                    source,
                    timeout_ms: Some(runtime_config.process.bootstrap_timeout_ms),
                }),
            };
            send_process_request(ctx, action.id(), request)?;
            Ok(ProcessDispatchOutcome::Applied)
        }
        HaAction::RunBootstrap => {
            let request = ProcessJobRequest {
                id: process_job_id(&ctx.scope, &ctx.self_id, action, action_index, ha_tick),
                kind: ProcessJobKind::Bootstrap(BootstrapSpec {
                    data_dir: runtime_config.postgres.data_dir.clone(),
                    superuser_username: runtime_config.postgres.roles.superuser.username.clone(),
                    timeout_ms: None,
                }),
            };
            send_process_request(ctx, action.id(), request)?;
            Ok(ProcessDispatchOutcome::Applied)
        }
        HaAction::FenceNode => {
            let request = ProcessJobRequest {
                id: process_job_id(&ctx.scope, &ctx.self_id, action, action_index, ha_tick),
                kind: ProcessJobKind::Fencing(FencingSpec {
                    data_dir: runtime_config.postgres.data_dir.clone(),
                    mode: ShutdownMode::Immediate,
                    timeout_ms: None,
                }),
            };
            send_process_request(ctx, action.id(), request)?;
            Ok(ProcessDispatchOutcome::Applied)
        }
        HaAction::WipeDataDir => {
            wipe_data_dir(runtime_config.postgres.data_dir.as_path()).map_err(|message| {
                ProcessDispatchError::Filesystem {
                    action: action.id(),
                    message,
                }
            })?;
            Ok(ProcessDispatchOutcome::Applied)
        }
        HaAction::FollowLeader { .. } | HaAction::SignalFailSafe => {
            Ok(ProcessDispatchOutcome::Skipped)
        }
    }
}

pub(crate) fn validate_rewind_source(
    ctx: &HaWorkerCtx,
    action: ActionId,
    leader_member_id: &crate::state::MemberId,
) -> Result<crate::process::jobs::RewinderSourceConn, ProcessDispatchError> {
    let member = resolve_source_member(ctx, action.clone(), leader_member_id)?;
    rewind_source_from_member(&ctx.self_id, &member, &ctx.process_defaults).map_err(|err| {
        ProcessDispatchError::SourceSelection {
            action,
            message: err.to_string(),
        }
    })
}

pub(crate) fn validate_basebackup_source(
    ctx: &HaWorkerCtx,
    action: ActionId,
    leader_member_id: &crate::state::MemberId,
) -> Result<crate::process::jobs::ReplicatorSourceConn, ProcessDispatchError> {
    let member = resolve_source_member(ctx, action.clone(), leader_member_id)?;
    basebackup_source_from_member(&ctx.self_id, &member, &ctx.process_defaults).map_err(|err| {
        ProcessDispatchError::SourceSelection {
            action,
            message: err.to_string(),
        }
    })
}

fn resolve_source_member(
    ctx: &HaWorkerCtx,
    action: ActionId,
    leader_member_id: &crate::state::MemberId,
) -> Result<MemberRecord, ProcessDispatchError> {
    let dcs = ctx.dcs_subscriber.latest();
    dcs.value
        .cache
        .members
        .get(leader_member_id)
        .cloned()
        .ok_or_else(|| ProcessDispatchError::SourceSelection {
            action,
            message: format!(
                "target member `{}` not present in DCS cache",
                leader_member_id.0
            ),
        })
}

fn send_process_request(
    ctx: &mut HaWorkerCtx,
    action: ActionId,
    request: ProcessJobRequest,
) -> Result<(), ProcessDispatchError> {
    ctx.process_inbox
        .send(request)
        .map_err(|err| ProcessDispatchError::ProcessSend {
            action,
            message: err.to_string(),
        })
}

fn start_postgres_leader_member_id(ctx: &HaWorkerCtx) -> Option<&MemberId> {
    match &ctx.state.decision {
        HaDecision::WaitForPostgres {
            leader_member_id, ..
        } => leader_member_id.as_ref(),
        _ => None,
    }
}

fn start_intent_from_dcs(
    ctx: &HaWorkerCtx,
    replica_leader_member_id: Option<&MemberId>,
    data_dir: &Path,
) -> Result<ManagedPostgresStartIntent, ProcessDispatchError> {
    if let Some(leader_member_id) = replica_leader_member_id {
        let leader = resolve_source_member(ctx, ActionId::StartPostgres, leader_member_id)?;
        let source = basebackup_source_from_member(&ctx.self_id, &leader, &ctx.process_defaults)
            .map_err(|err| ProcessDispatchError::SourceSelection {
                action: ActionId::StartPostgres,
                message: err.to_string(),
            })?;
        return Ok(ManagedPostgresStartIntent::replica(
            source.conninfo.clone(),
            managed_standby_auth_from_role_auth(&source.auth, data_dir),
            None,
        ));
    }

    let managed_recovery_state = crate::postgres_managed::inspect_managed_recovery_state(data_dir)
        .map_err(|err| ProcessDispatchError::ManagedConfig {
            action: ActionId::StartPostgres,
            message: err.to_string(),
        })?;
    if managed_recovery_state != crate::postgres_managed_conf::ManagedRecoverySignal::None {
        return Err(ProcessDispatchError::ManagedConfig {
            action: ActionId::StartPostgres,
            message:
                "existing postgres data dir contains managed replica recovery state but no leader-derived source is available to rebuild authoritative managed config"
                    .to_string(),
        });
    }

    Ok(ManagedPostgresStartIntent::primary())
}

fn process_job_id(
    scope: &str,
    self_id: &crate::state::MemberId,
    action: &HaAction,
    index: usize,
    tick: u64,
) -> JobId {
    JobId(format!(
        "ha-{}-{}-{}-{}-{}",
        scope.trim_matches('/'),
        self_id.0,
        tick,
        index,
        action.id().label(),
    ))
}

fn wipe_data_dir(data_dir: &Path) -> Result<(), String> {
    if data_dir.as_os_str().is_empty() {
        return Err("wipe_data_dir data_dir must not be empty".to_string());
    }
    if data_dir.exists() {
        fs::remove_dir_all(data_dir)
            .map_err(|err| format!("wipe_data_dir remove_dir_all failed: {err}"))?;
    }
    fs::create_dir_all(data_dir)
        .map_err(|err| format!("wipe_data_dir create_dir_all failed: {err}"))?;
    set_postgres_data_dir_permissions(data_dir)?;
    Ok(())
}

fn set_postgres_data_dir_permissions(data_dir: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(data_dir, fs::Permissions::from_mode(0o700))
            .map_err(|err| format!("wipe_data_dir set_permissions failed: {err}"))?;
    }

    #[cfg(not(unix))]
    {
        let _ = data_dir;
    }

    Ok(())
}

```

## Source path: `src/ha/apply.rs`

```rust
use thiserror::Error;

use crate::{dcs::store::DcsStoreError, state::WorkerError};

use super::{
    actions::{ActionId, HaAction},
    events::{
        emit_ha_action_dispatch, emit_ha_action_intent, emit_ha_action_result_failed,
        emit_ha_action_result_ok, emit_ha_action_result_skipped, emit_ha_lease_transition,
    },
    lower::{
        HaEffectPlan, LeaseEffect, PostgresEffect, ReplicationEffect, SafetyEffect,
        SwitchoverEffect,
    },
    process_dispatch::{
        dispatch_process_action, validate_basebackup_source, ProcessDispatchError,
        ProcessDispatchOutcome,
    },
    state::HaWorkerCtx,
};

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(crate) enum ActionDispatchError {
    #[error("process send failed for action `{action:?}`: {message}")]
    ProcessSend { action: ActionId, message: String },
    #[error("managed config materialization failed for action `{action:?}`: {message}")]
    ManagedConfig { action: ActionId, message: String },
    #[error("filesystem operation failed for action `{action:?}`: {message}")]
    Filesystem { action: ActionId, message: String },
    #[error("dcs write failed for action `{action:?}` at `{path}`: {message}")]
    DcsWrite {
        action: ActionId,
        path: String,
        message: String,
    },
    #[error("dcs delete failed for action `{action:?}` at `{path}`: {message}")]
    DcsDelete {
        action: ActionId,
        path: String,
        message: String,
    },
}

pub(crate) fn apply_effect_plan(
    ctx: &mut HaWorkerCtx,
    ha_tick: u64,
    plan: &HaEffectPlan,
) -> Result<Vec<ActionDispatchError>, WorkerError> {
    let runtime_config = ctx.config_subscriber.latest().value;
    let mut errors = Vec::new();
    let mut action_index = 0usize;

    action_index = dispatch_postgres_effect(
        ctx,
        ha_tick,
        action_index,
        &plan.postgres,
        &runtime_config,
        &mut errors,
    )?;
    action_index = dispatch_lease_effect(
        ctx,
        ha_tick,
        action_index,
        &plan.lease,
        &runtime_config,
        &mut errors,
    )?;
    action_index = dispatch_switchover_effect(
        ctx,
        ha_tick,
        action_index,
        &plan.switchover,
        &runtime_config,
        &mut errors,
    )?;
    action_index = dispatch_replication_effect(
        ctx,
        ha_tick,
        action_index,
        &plan.replication,
        &runtime_config,
        &mut errors,
    )?;
    let _ = dispatch_safety_effect(
        ctx,
        ha_tick,
        action_index,
        &plan.safety,
        &runtime_config,
        &mut errors,
    )?;

    Ok(errors)
}

pub(crate) fn format_dispatch_errors(errors: &[ActionDispatchError]) -> String {
    let mut details = String::new();
    for (index, err) in errors.iter().enumerate() {
        if index > 0 {
            details.push_str("; ");
        }
        details.push_str(&err.to_string());
    }
    format!(
        "ha dispatch failed with {} error(s): {details}",
        errors.len()
    )
}

fn dispatch_postgres_effect(
    ctx: &mut HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    effect: &PostgresEffect,
    runtime_config: &crate::config::RuntimeConfig,
    errors: &mut Vec<ActionDispatchError>,
) -> Result<usize, WorkerError> {
    match effect {
        PostgresEffect::None => Ok(action_index),
        PostgresEffect::Start => dispatch_effect_action(
            ctx,
            ha_tick,
            action_index,
            HaAction::StartPostgres,
            runtime_config,
            errors,
        ),
        PostgresEffect::Promote => dispatch_effect_action(
            ctx,
            ha_tick,
            action_index,
            HaAction::PromoteToPrimary,
            runtime_config,
            errors,
        ),
        PostgresEffect::Demote => dispatch_effect_action(
            ctx,
            ha_tick,
            action_index,
            HaAction::DemoteToReplica,
            runtime_config,
            errors,
        ),
    }
}

fn dispatch_lease_effect(
    ctx: &mut HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    effect: &LeaseEffect,
    runtime_config: &crate::config::RuntimeConfig,
    errors: &mut Vec<ActionDispatchError>,
) -> Result<usize, WorkerError> {
    match effect {
        LeaseEffect::None => Ok(action_index),
        LeaseEffect::AcquireLeader => dispatch_effect_action(
            ctx,
            ha_tick,
            action_index,
            HaAction::AcquireLeaderLease,
            runtime_config,
            errors,
        ),
        LeaseEffect::ReleaseLeader => dispatch_effect_action(
            ctx,
            ha_tick,
            action_index,
            HaAction::ReleaseLeaderLease,
            runtime_config,
            errors,
        ),
    }
}

fn dispatch_switchover_effect(
    ctx: &mut HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    effect: &SwitchoverEffect,
    runtime_config: &crate::config::RuntimeConfig,
    errors: &mut Vec<ActionDispatchError>,
) -> Result<usize, WorkerError> {
    match effect {
        SwitchoverEffect::None => Ok(action_index),
        SwitchoverEffect::ClearRequest => dispatch_effect_action(
            ctx,
            ha_tick,
            action_index,
            HaAction::ClearSwitchover,
            runtime_config,
            errors,
        ),
    }
}

fn dispatch_replication_effect(
    ctx: &mut HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    effect: &ReplicationEffect,
    runtime_config: &crate::config::RuntimeConfig,
    errors: &mut Vec<ActionDispatchError>,
) -> Result<usize, WorkerError> {
    match effect {
        ReplicationEffect::None => Ok(action_index),
        ReplicationEffect::FollowLeader { leader_member_id } => dispatch_effect_action(
            ctx,
            ha_tick,
            action_index,
            HaAction::FollowLeader {
                leader_member_id: leader_member_id.0.clone(),
            },
            runtime_config,
            errors,
        ),
        ReplicationEffect::RecoverReplica { strategy } => match strategy {
            crate::ha::decision::RecoveryStrategy::Rewind { leader_member_id } => {
                dispatch_effect_action(
                    ctx,
                    ha_tick,
                    action_index,
                    HaAction::StartRewind {
                        leader_member_id: leader_member_id.clone(),
                    },
                    runtime_config,
                    errors,
                )
            }
            crate::ha::decision::RecoveryStrategy::BaseBackup { leader_member_id } => {
                if let Err(err) =
                    validate_basebackup_source(ctx, ActionId::StartBaseBackup, leader_member_id)
                {
                    errors.push(map_process_dispatch_error(err));
                    return Ok(action_index);
                }
                let next_index = dispatch_effect_action(
                    ctx,
                    ha_tick,
                    action_index,
                    HaAction::WipeDataDir,
                    runtime_config,
                    errors,
                )?;
                dispatch_effect_action(
                    ctx,
                    ha_tick,
                    next_index,
                    HaAction::StartBaseBackup {
                        leader_member_id: leader_member_id.clone(),
                    },
                    runtime_config,
                    errors,
                )
            }
            crate::ha::decision::RecoveryStrategy::Bootstrap => {
                let next_index = dispatch_effect_action(
                    ctx,
                    ha_tick,
                    action_index,
                    HaAction::WipeDataDir,
                    runtime_config,
                    errors,
                )?;
                dispatch_effect_action(
                    ctx,
                    ha_tick,
                    next_index,
                    HaAction::RunBootstrap,
                    runtime_config,
                    errors,
                )
            }
        },
    }
}

fn dispatch_safety_effect(
    ctx: &mut HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    effect: &SafetyEffect,
    runtime_config: &crate::config::RuntimeConfig,
    errors: &mut Vec<ActionDispatchError>,
) -> Result<usize, WorkerError> {
    match effect {
        SafetyEffect::None => Ok(action_index),
        SafetyEffect::FenceNode => dispatch_effect_action(
            ctx,
            ha_tick,
            action_index,
            HaAction::FenceNode,
            runtime_config,
            errors,
        ),
        SafetyEffect::SignalFailSafe => dispatch_effect_action(
            ctx,
            ha_tick,
            action_index,
            HaAction::SignalFailSafe,
            runtime_config,
            errors,
        ),
    }
}

fn dispatch_effect_action(
    ctx: &mut HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    action: HaAction,
    runtime_config: &crate::config::RuntimeConfig,
    errors: &mut Vec<ActionDispatchError>,
) -> Result<usize, WorkerError> {
    emit_ha_action_intent(ctx, ha_tick, action_index, &action)?;
    emit_ha_action_dispatch(ctx, ha_tick, action_index, &action)?;

    if let Some(error) = dispatch_action(ctx, ha_tick, action_index, &action, runtime_config)? {
        errors.push(error);
    }

    Ok(action_index.saturating_add(1))
}

fn dispatch_action(
    ctx: &mut HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    action: &HaAction,
    runtime_config: &crate::config::RuntimeConfig,
) -> Result<Option<ActionDispatchError>, WorkerError> {
    match action {
        HaAction::AcquireLeaderLease => {
            let dispatch_result = acquire_leader_lease(ctx);
            dcs_dispatch_result(
                ctx,
                ha_tick,
                action_index,
                action,
                leader_path(&ctx.scope),
                dispatch_result,
                true,
            )
        }
        HaAction::ReleaseLeaderLease => {
            let dispatch_result = release_leader_lease(ctx);
            dcs_dispatch_result(
                ctx,
                ha_tick,
                action_index,
                action,
                leader_path(&ctx.scope),
                dispatch_result,
                false,
            )
        }
        HaAction::ClearSwitchover => {
            let path = switchover_path(&ctx.scope);
            let result = clear_switchover_request(ctx);
            dcs_delete_result(ctx, ha_tick, action_index, action, path, result)
        }
        _ => {
            let result =
                dispatch_process_action(ctx, ha_tick, action_index, action, runtime_config);
            process_dispatch_result(ctx, ha_tick, action_index, action, result)
        }
    }
}

fn dcs_dispatch_result(
    ctx: &HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    action: &HaAction,
    path: String,
    result: Result<(), DcsStoreError>,
    acquired: bool,
) -> Result<Option<ActionDispatchError>, WorkerError> {
    match result {
        Ok(()) => {
            emit_ha_action_result_ok(ctx, ha_tick, action_index, action)?;
            emit_ha_lease_transition(ctx, ha_tick, acquired)?;
            Ok(None)
        }
        Err(err) => {
            let message = err.to_string();
            emit_ha_action_result_failed(ctx, ha_tick, action_index, action, message)?;
            let error = if acquired {
                ActionDispatchError::DcsWrite {
                    action: action.id(),
                    path,
                    message: dcs_error_message(err),
                }
            } else {
                ActionDispatchError::DcsDelete {
                    action: action.id(),
                    path,
                    message: dcs_error_message(err),
                }
            };
            Ok(Some(error))
        }
    }
}

fn dcs_delete_result(
    ctx: &HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    action: &HaAction,
    path: String,
    result: Result<(), DcsStoreError>,
) -> Result<Option<ActionDispatchError>, WorkerError> {
    match result {
        Ok(()) => {
            emit_ha_action_result_ok(ctx, ha_tick, action_index, action)?;
            Ok(None)
        }
        Err(err) => {
            let message = err.to_string();
            emit_ha_action_result_failed(ctx, ha_tick, action_index, action, message)?;
            Ok(Some(ActionDispatchError::DcsDelete {
                action: action.id(),
                path,
                message: dcs_error_message(err),
            }))
        }
    }
}

fn process_dispatch_result(
    ctx: &HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    action: &HaAction,
    result: Result<ProcessDispatchOutcome, ProcessDispatchError>,
) -> Result<Option<ActionDispatchError>, WorkerError> {
    match result {
        Ok(ProcessDispatchOutcome::Applied) => {
            emit_ha_action_result_ok(ctx, ha_tick, action_index, action)?;
            Ok(None)
        }
        Ok(ProcessDispatchOutcome::Skipped) => {
            emit_ha_action_result_skipped(ctx, ha_tick, action_index, action)?;
            Ok(None)
        }
        Err(ProcessDispatchError::UnsupportedAction { action }) => {
            Err(WorkerError::Message(format!(
                "ha apply routed unsupported process action `{}`",
                action.label()
            )))
        }
        Err(err) => {
            let message = err.to_string();
            emit_ha_action_result_failed(ctx, ha_tick, action_index, action, message)?;
            Ok(Some(map_process_dispatch_error(err)))
        }
    }
}

fn acquire_leader_lease(ctx: &mut HaWorkerCtx) -> Result<(), DcsStoreError> {
    ctx.dcs_store.acquire_leader_lease(&ctx.scope, &ctx.self_id)
}

fn release_leader_lease(ctx: &mut HaWorkerCtx) -> Result<(), DcsStoreError> {
    ctx.dcs_store.release_leader_lease(&ctx.scope, &ctx.self_id)
}

fn clear_switchover_request(ctx: &mut HaWorkerCtx) -> Result<(), DcsStoreError> {
    ctx.dcs_store.clear_switchover(&ctx.scope)
}

fn leader_path(scope: &str) -> String {
    format!("/{}/leader", scope.trim_matches('/'))
}

fn switchover_path(scope: &str) -> String {
    format!("/{}/switchover", scope.trim_matches('/'))
}

fn dcs_error_message(error: DcsStoreError) -> String {
    error.to_string()
}

fn map_process_dispatch_error(error: ProcessDispatchError) -> ActionDispatchError {
    match error {
        ProcessDispatchError::ProcessSend { action, message } => {
            ActionDispatchError::ProcessSend { action, message }
        }
        ProcessDispatchError::ManagedConfig { action, message } => {
            ActionDispatchError::ManagedConfig { action, message }
        }
        ProcessDispatchError::Filesystem { action, message } => {
            ActionDispatchError::Filesystem { action, message }
        }
        ProcessDispatchError::SourceSelection { action, message } => {
            ActionDispatchError::ProcessSend { action, message }
        }
        ProcessDispatchError::UnsupportedAction { action } => ActionDispatchError::ProcessSend {
            action,
            message: "unsupported process action".to_string(),
        },
    }
}
```

# Input worker loops feeding the HA world snapshot

## Source path: `src/pginfo/worker.rs`

```rust
use crate::state::{UnixMillis, WorkerStatus};
use crate::{
    logging::{AppEvent, AppEventHeader, SeverityText, StructuredFields},
    state::WorkerError,
};

use super::query::poll_once;
use super::state::{to_member_status, PgInfoState, PgInfoWorkerCtx, SqlStatus};

fn pginfo_append_base_fields(fields: &mut StructuredFields, ctx: &PgInfoWorkerCtx) {
    fields.insert("member_id", ctx.self_id.0.clone());
}

fn pginfo_event(severity: SeverityText, message: &str, name: &str, result: &str) -> AppEvent {
    AppEvent::new(
        severity,
        message,
        AppEventHeader::new(name, "pginfo", result),
    )
}

fn emit_pginfo_event(
    ctx: &PgInfoWorkerCtx,
    origin: &str,
    event: AppEvent,
    error_prefix: &str,
) -> Result<(), WorkerError> {
    ctx.log
        .emit_app_event(origin, event)
        .map_err(|err| WorkerError::Message(format!("{error_prefix}: {err}")))
}

fn sql_status_label(status: &SqlStatus) -> String {
    format!("{status:?}").to_lowercase()
}

pub(crate) async fn run(mut ctx: PgInfoWorkerCtx) -> Result<(), WorkerError> {
    loop {
        step_once(&mut ctx).await?;
        tokio::time::sleep(ctx.poll_interval).await;
    }
}

pub(crate) async fn step_once(ctx: &mut PgInfoWorkerCtx) -> Result<(), WorkerError> {
    let now = now_unix_millis()?;
    let poll = poll_once(&ctx.postgres_conninfo).await;
    let next_state = match poll {
        Ok(polled) => {
            to_member_status(WorkerStatus::Running, SqlStatus::Healthy, now, Some(polled))
        }
        Err(ref err) => {
            let mut event = pginfo_event(
                SeverityText::Warn,
                "pginfo poll failed",
                "pginfo.poll_failed",
                "failed",
            );
            let fields = event.fields_mut();
            pginfo_append_base_fields(fields, ctx);
            fields.insert("error", err.to_string());
            emit_pginfo_event(
                ctx,
                "pginfo_worker::step_once",
                event,
                "pginfo poll failure log emit failed",
            )?;
            to_member_status(WorkerStatus::Running, SqlStatus::Unreachable, now, None)
        }
    };

    let next_sql = pginfo_sql_status(&next_state);
    let prev_sql = ctx
        .last_emitted_sql_status
        .clone()
        .unwrap_or(SqlStatus::Unknown);
    if prev_sql != next_sql {
        let (severity, result) = match (prev_sql.clone(), next_sql.clone()) {
            (SqlStatus::Healthy, SqlStatus::Unreachable) => (SeverityText::Warn, "failed"),
            (SqlStatus::Unreachable, SqlStatus::Healthy) => (SeverityText::Info, "recovered"),
            _ => (SeverityText::Debug, "ok"),
        };
        let mut event = pginfo_event(
            severity,
            "pginfo sql status transition",
            "pginfo.sql_transition",
            result,
        );
        let fields = event.fields_mut();
        pginfo_append_base_fields(fields, ctx);
        fields.insert("sql_status_prev", sql_status_label(&prev_sql));
        fields.insert("sql_status_next", sql_status_label(&next_sql));
        emit_pginfo_event(
            ctx,
            "pginfo_worker::step_once",
            event,
            "pginfo sql transition log emit failed",
        )?;
        ctx.last_emitted_sql_status = Some(next_sql.clone());
    }

    ctx.publisher.publish(next_state, now).map_err(|err| {
        WorkerError::Message(format!(
            "pginfo publish failed for {:?}: {err}",
            ctx.self_id
        ))
    })?;
    Ok(())
}

fn pginfo_sql_status(state: &PgInfoState) -> SqlStatus {
    match state {
        PgInfoState::Unknown { common } => common.sql.clone(),
        PgInfoState::Primary { common, .. } => common.sql.clone(),
        PgInfoState::Replica { common, .. } => common.sql.clone(),
    }
}

fn now_unix_millis() -> Result<UnixMillis, WorkerError> {
    let elapsed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|err| WorkerError::Message(format!("system clock before unix epoch: {err}")))?;
    let millis = u64::try_from(elapsed.as_millis())
        .map_err(|err| WorkerError::Message(format!("unix millis conversion failed: {err}")))?;
    Ok(UnixMillis(millis))
}

```

## Source path: `src/dcs/worker.rs`

```rust
use crate::{
    logging::{AppEvent, AppEventHeader, SeverityText, StructuredFields},
    state::WorkerError,
};

use super::{
    keys::DcsKey,
    state::{
        build_local_member_record, evaluate_trust, DcsCache, DcsState, DcsTrust, DcsWorkerCtx,
        InitLockRecord, LeaderRecord, MemberRecord, SwitchoverRequest,
    },
    store::{refresh_from_etcd_watch, write_local_member},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DcsValue {
    Member(MemberRecord),
    Leader(LeaderRecord),
    Switchover(SwitchoverRequest),
    Config(Box<crate::config::RuntimeConfig>),
    InitLock(InitLockRecord),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DcsWatchUpdate {
    Put { key: DcsKey, value: Box<DcsValue> },
    Delete { key: DcsKey },
}

fn dcs_append_base_fields(fields: &mut StructuredFields, ctx: &DcsWorkerCtx) {
    fields.insert("scope", ctx.scope.clone());
    fields.insert("member_id", ctx.self_id.0.clone());
}

fn dcs_event(severity: SeverityText, message: &str, name: &str, result: &str) -> AppEvent {
    AppEvent::new(severity, message, AppEventHeader::new(name, "dcs", result))
}

fn emit_dcs_event(
    ctx: &DcsWorkerCtx,
    origin: &str,
    event: AppEvent,
    error_prefix: &str,
) -> Result<(), WorkerError> {
    ctx.log
        .emit_app_event(origin, event)
        .map_err(|err| WorkerError::Message(format!("{error_prefix}: {err}")))
}

fn dcs_io_error_severity(err: &crate::dcs::store::DcsStoreError) -> SeverityText {
    match err {
        crate::dcs::store::DcsStoreError::Io(_) => SeverityText::Warn,
        _ => SeverityText::Error,
    }
}

fn dcs_refresh_error_severity(err: &crate::dcs::store::DcsStoreError) -> SeverityText {
    match err {
        crate::dcs::store::DcsStoreError::Io(_)
        | crate::dcs::store::DcsStoreError::InvalidKey(_)
        | crate::dcs::store::DcsStoreError::MissingValue(_) => SeverityText::Warn,
        _ => SeverityText::Error,
    }
}

pub(crate) async fn run(mut ctx: DcsWorkerCtx) -> Result<(), WorkerError> {
    loop {
        step_once(&mut ctx).await?;
        tokio::time::sleep(ctx.poll_interval).await;
    }
}

pub(crate) fn apply_watch_update(cache: &mut DcsCache, update: DcsWatchUpdate) {
    match update {
        DcsWatchUpdate::Put { key, value } => match (key, *value) {
            (DcsKey::Member(member_id), DcsValue::Member(record)) => {
                cache.members.insert(member_id, record);
            }
            (DcsKey::Leader, DcsValue::Leader(record)) => {
                cache.leader = Some(record);
            }
            (DcsKey::Switchover, DcsValue::Switchover(record)) => {
                cache.switchover = Some(record);
            }
            (DcsKey::Config, DcsValue::Config(config)) => {
                cache.config = *config;
            }
            (DcsKey::InitLock, DcsValue::InitLock(record)) => {
                cache.init_lock = Some(record);
            }
            _ => {}
        },
        DcsWatchUpdate::Delete { key } => match key {
            DcsKey::Member(member_id) => {
                cache.members.remove(&member_id);
            }
            DcsKey::Leader => {
                cache.leader = None;
            }
            DcsKey::Switchover => {
                cache.switchover = None;
            }
            DcsKey::Config => {}
            DcsKey::InitLock => {
                cache.init_lock = None;
            }
        },
    }
}

pub(crate) async fn step_once(ctx: &mut DcsWorkerCtx) -> Result<(), WorkerError> {
    let now = now_unix_millis()?;
    let pg_snapshot = ctx.pg_subscriber.latest();

    let mut store_healthy = ctx.store.healthy();
    let must_publish_local_member = store_healthy;
    let mut local_member_publish_succeeded = false;

    if must_publish_local_member {
        let local_member = build_local_member_record(
            &ctx.self_id,
            ctx.local_postgres_host.as_str(),
            ctx.local_postgres_port,
            ctx.local_api_url.as_deref(),
            &pg_snapshot.value,
            now,
            pg_snapshot.version,
        );
        match write_local_member(ctx.store.as_mut(), &ctx.scope, &local_member) {
            Ok(()) => {
                ctx.last_published_pg_version = Some(pg_snapshot.version);
                ctx.cache.members.insert(ctx.self_id.clone(), local_member);
                local_member_publish_succeeded = true;
            }
            Err(err) => {
                let mut event = dcs_event(
                    dcs_io_error_severity(&err),
                    "dcs local member write failed",
                    "dcs.local_member.write_failed",
                    "failed",
                );
                let fields = event.fields_mut();
                dcs_append_base_fields(fields, ctx);
                fields.insert("error", err.to_string());
                emit_dcs_event(
                    ctx,
                    "dcs_worker::step_once",
                    event,
                    "dcs local member write log emit failed",
                )?;
                store_healthy = false;
            }
        }
    }

    let events = match ctx.store.drain_watch_events() {
        Ok(events) => events,
        Err(err) => {
            let mut event = dcs_event(
                dcs_io_error_severity(&err),
                "dcs watch drain failed",
                "dcs.watch.drain_failed",
                "failed",
            );
            let fields = event.fields_mut();
            dcs_append_base_fields(fields, ctx);
            fields.insert("error", err.to_string());
            emit_dcs_event(
                ctx,
                "dcs_worker::step_once",
                event,
                "dcs drain log emit failed",
            )?;
            store_healthy = false;
            Vec::new()
        }
    };
    match refresh_from_etcd_watch(&ctx.scope, &mut ctx.cache, events) {
        Ok(result) => {
            if result.had_errors {
                let mut event = dcs_event(
                    SeverityText::Warn,
                    "dcs watch refresh had errors",
                    "dcs.watch.apply_had_errors",
                    "failed",
                );
                let fields = event.fields_mut();
                dcs_append_base_fields(fields, ctx);
                fields.insert("applied", result.applied);
                emit_dcs_event(
                    ctx,
                    "dcs_worker::step_once",
                    event,
                    "dcs refresh had_errors log emit failed",
                )?;
                store_healthy = false;
            }
        }
        Err(err) => {
            let mut event = dcs_event(
                dcs_refresh_error_severity(&err),
                "dcs watch refresh failed",
                "dcs.watch.refresh_failed",
                "failed",
            );
            let fields = event.fields_mut();
            dcs_append_base_fields(fields, ctx);
            fields.insert("error", err.to_string());
            emit_dcs_event(
                ctx,
                "dcs_worker::step_once",
                event,
                "dcs refresh log emit failed",
            )?;
            store_healthy = false;
        }
    }

    let trust = if local_member_publish_succeeded {
        evaluate_trust(store_healthy, &ctx.cache, &ctx.self_id, now)
    } else {
        DcsTrust::NotTrusted
    };
    let worker = if store_healthy {
        crate::state::WorkerStatus::Running
    } else {
        crate::state::WorkerStatus::Faulted(WorkerError::Message("dcs store unhealthy".to_string()))
    };

    let next = DcsState {
        worker,
        trust: if store_healthy {
            trust
        } else {
            DcsTrust::NotTrusted
        },
        cache: ctx.cache.clone(),
        last_refresh_at: Some(now),
    };
    if ctx.last_emitted_store_healthy != Some(store_healthy) {
        ctx.last_emitted_store_healthy = Some(store_healthy);
        let mut event = dcs_event(
            if store_healthy {
                SeverityText::Info
            } else {
                SeverityText::Warn
            },
            "dcs store health transition",
            "dcs.store.health_transition",
            if store_healthy { "recovered" } else { "failed" },
        );
        let fields = event.fields_mut();
        dcs_append_base_fields(fields, ctx);
        fields.insert("store_healthy", store_healthy);
        emit_dcs_event(
            ctx,
            "dcs_worker::step_once",
            event,
            "dcs health transition log emit failed",
        )?;
    }
    if ctx.last_emitted_trust.as_ref() != Some(&next.trust) {
        let prev = ctx
            .last_emitted_trust
            .as_ref()
            .map(|value| format!("{value:?}").to_lowercase())
            .unwrap_or_else(|| "unknown".to_string());
        ctx.last_emitted_trust = Some(next.trust.clone());
        let mut event = dcs_event(
            SeverityText::Info,
            "dcs trust transition",
            "dcs.trust.transition",
            "ok",
        );
        let fields = event.fields_mut();
        dcs_append_base_fields(fields, ctx);
        fields.insert("trust_prev", prev);
        fields.insert("trust_next", format!("{:?}", next.trust).to_lowercase());
        emit_dcs_event(
            ctx,
            "dcs_worker::step_once",
            event,
            "dcs trust transition log emit failed",
        )?;
    }
    ctx.publisher
        .publish(next, now)
        .map_err(|err| WorkerError::Message(format!("dcs publish failed: {err}")))?;
    Ok(())
}

fn now_unix_millis() -> Result<crate::state::UnixMillis, WorkerError> {
    let elapsed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|err| WorkerError::Message(format!("system clock before unix epoch: {err}")))?;
    let millis = u64::try_from(elapsed.as_millis())
        .map_err(|err| WorkerError::Message(format!("unix millis conversion failed: {err}")))?;
    Ok(crate::state::UnixMillis(millis))
}

```

## Process worker loop, acceptance, dispatch, active job ticking, and command construction

Source path: `src/process/worker.rs`

```rust
pub(crate) fn can_accept_job(state: &ProcessState) -> bool {
    matches!(state, ProcessState::Idle { .. })
}

pub(crate) async fn run(mut ctx: ProcessWorkerCtx) -> Result<(), WorkerError> {
    let mut event = process_event(
        ProcessEventKind::RunStarted,
        "ok",
        SeverityText::Debug,
        "process worker run started",
    );
    event
        .fields_mut()
        .insert("capture_subprocess_output", ctx.capture_subprocess_output);
    ctx.log
        .emit_app_event("process_worker::run", event)
        .map_err(|err| {
            WorkerError::Message(format!("process worker start log emit failed: {err}"))
        })?;
    loop {
        step_once(&mut ctx).await?;
        tokio::time::sleep(ctx.poll_interval).await;
    }
}

pub(crate) async fn step_once(ctx: &mut ProcessWorkerCtx) -> Result<(), WorkerError> {
    match ctx.inbox.try_recv() {
        Ok(request) => {
            let mut event = process_event(
                ProcessEventKind::RequestReceived,
                "ok",
                SeverityText::Debug,
                "process job request received",
            );
            event.fields_mut().append_json_map(
                process_job_fields(&request.id, request.kind.label()).into_attributes(),
            );
            ctx.log
                .emit_app_event("process_worker::step_once", event)
                .map_err(|err| {
                    WorkerError::Message(format!("process request log emit failed: {err}"))
                })?;
            start_job(ctx, request).await?;
        }
        Err(TryRecvError::Empty) => {}
        Err(TryRecvError::Disconnected) => {
            if !ctx.inbox_disconnected_logged {
                ctx.inbox_disconnected_logged = true;
                ctx.log
                    .emit_app_event(
                        "process_worker::step_once",
                        process_event(
                            ProcessEventKind::InboxDisconnected,
                            "failed",
                            SeverityText::Warn,
                            "process worker inbox disconnected",
                        ),
                    )
                    .map_err(|err| {
                        WorkerError::Message(format!(
                            "process inbox disconnected log emit failed: {err}"
                        ))
                    })?;
            }
        }
    }

    tick_active_job(ctx).await
}

fn parse_postmaster_pid(pid_file: &Path) -> Result<u32, ProcessError> {
    let contents = fs::read_to_string(pid_file).map_err(|err| {
        ProcessError::InvalidSpec(format!(
            "read postmaster.pid {} failed: {err}",
            pid_file.display()
        ))
    })?;
    let first_line = contents.lines().next().ok_or_else(|| {
        ProcessError::InvalidSpec(format!(
            "postmaster.pid {} missing pid line",
            pid_file.display()
        ))
    })?;
    let trimmed = first_line.trim();
    if trimmed.is_empty() {
        return Err(ProcessError::InvalidSpec(format!(
            "postmaster.pid {} pid line is empty",
            pid_file.display()
        )));
    }
    trimmed.parse::<u32>().map_err(|err| {
        ProcessError::InvalidSpec(format!(
            "parse postmaster.pid pid '{trimmed}' failed: {err}"
        ))
    })
}

fn postmaster_pid_data_dir_matches(pid_file: &Path, data_dir: &Path) -> Result<bool, ProcessError> {
    let contents = fs::read_to_string(pid_file).map_err(|err| {
        ProcessError::InvalidSpec(format!(
            "read postmaster.pid {} failed: {err}",
            pid_file.display()
        ))
    })?;
    let Some(raw_data_dir) = contents.lines().nth(1) else {
        return Ok(false);
    };
    let trimmed = raw_data_dir.trim();
    if trimmed.is_empty() {
        return Ok(false);
    }
    Ok(Path::new(trimmed) == data_dir)
}

fn pid_exists(pid: u32) -> Result<bool, ProcessError> {
    #[cfg(unix)]
    {
        let pid_i32 = i32::try_from(pid).map_err(|err| {
            ProcessError::InvalidSpec(format!("postmaster pid {pid} i32 conversion failed: {err}"))
        })?;
        let rc = unsafe { libc::kill(pid_i32, 0) };
        if rc == 0 {
            return Ok(true);
        }
        let err = std::io::Error::last_os_error();
        let raw = err.raw_os_error();
        if raw == Some(libc::ESRCH) {
            return Ok(false);
        }
        if raw == Some(libc::EPERM) {
            return Ok(true);
        }
        Err(ProcessError::InvalidSpec(format!(
            "kill(0) failed for pid={pid}: {err}"
        )))
    }
    #[cfg(not(unix))]
    {
        let _ = pid;
        Ok(true)
    }
}

fn pid_matches_data_dir(pid: u32, data_dir: &Path, pid_file: &Path) -> Result<bool, ProcessError> {
    if !pid_exists(pid)? {
        return Ok(false);
    }

    #[cfg(unix)]
    {
        let cmdline_path = std::path::PathBuf::from(format!("/proc/{pid}/cmdline"));
        let cmdline = match fs::read(&cmdline_path) {
            Ok(bytes) => bytes,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(false),
            Err(err) => {
                return Err(ProcessError::InvalidSpec(format!(
                    "read {} failed: {err}",
                    cmdline_path.display()
                )));
            }
        };
        let data_dir_text = data_dir.display().to_string();
        let cmdline_args = cmdline
            .split(|byte| *byte == 0)
            .filter(|arg| !arg.is_empty())
            .map(|arg| String::from_utf8_lossy(arg))
            .collect::<Vec<_>>();
        let has_data_dir = cmdline_args
            .iter()
            .any(|arg| arg.contains(data_dir_text.as_str()));
        let has_postgres_argv = cmdline_args.iter().any(|arg| {
            std::path::Path::new(arg.as_ref())
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| matches!(name, "postgres" | "postmaster"))
                .unwrap_or(false)
        });
        if !has_postgres_argv {
            return Ok(false);
        }
        if has_data_dir {
            return Ok(true);
        }
        postmaster_pid_data_dir_matches(pid_file, data_dir)
    }
    #[cfg(not(unix))]
    {
        let _ = pid_file;
        let _ = data_dir;
        Ok(true)
    }
}

fn remove_file_best_effort(path: &Path) -> Result<(), ProcessError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(ProcessError::InvalidSpec(format!(
            "remove file {} failed: {err}",
            path.display()
        ))),
    }
}

fn fencing_preflight_is_already_stopped(data_dir: &Path) -> Result<bool, ProcessError> {
    let pid_file = data_dir.join("postmaster.pid");
    if !pid_file.exists() {
        return Ok(true);
    }

    let pid = parse_postmaster_pid(&pid_file)?;
    if pid_matches_data_dir(pid, data_dir, &pid_file)? {
        return Ok(false);
    }

    // Stale pid file: treat as already fenced to avoid `pg_ctl stop -w` waiting forever.
    remove_file_best_effort(&pid_file)?;
    let opts_file = data_dir.join("postmaster.opts");
    remove_file_best_effort(&opts_file)?;
    Ok(true)
}

pub(crate) fn start_postgres_preflight_is_already_running(
    data_dir: &Path,
) -> Result<bool, ProcessError> {
    let pid_file = data_dir.join("postmaster.pid");
    if !pid_file.exists() {
        return Ok(false);
    }

    let pid = parse_postmaster_pid(&pid_file)?;
    if pid_matches_data_dir(pid, data_dir, &pid_file)? {
        return Ok(true);
    }

    remove_file_best_effort(&pid_file)?;
    let opts_file = data_dir.join("postmaster.opts");
    remove_file_best_effort(&opts_file)?;
    Ok(false)
}

pub(crate) async fn start_job(
    ctx: &mut ProcessWorkerCtx,
    request: ProcessJobRequest,
) -> Result<(), WorkerError> {
    if !can_accept_job(&ctx.state) {
        let now = current_time(ctx)?;
        ctx.last_rejection = Some(ProcessJobRejection {
            id: request.id,
            error: ProcessError::Busy,
            rejected_at: now,
        });
        let mut event = process_event(
            ProcessEventKind::BusyReject,
            "failed",
            SeverityText::Warn,
            "process worker busy; rejecting job",
        );
        let rejected_job_id = ctx
            .last_rejection
            .as_ref()
            .map(|rejection| rejection.id.clone())
            .unwrap_or_else(|| JobId("unknown".to_string()));
        event.fields_mut().append_json_map(
            process_job_fields(&rejected_job_id, request.kind.label()).into_attributes(),
        );
        ctx.log
            .emit_app_event("process_worker::start_job", event)
            .map_err(|err| {
                WorkerError::Message(format!("process busy reject log emit failed: {err}"))
            })?;
        return Ok(());
    }

    let now = current_time(ctx)?;
    let timeout_ms = timeout_for_kind(&request.kind, &ctx.config);
    let deadline_at = UnixMillis(now.0.saturating_add(timeout_ms));

    if let ProcessJobKind::Fencing(spec) = &request.kind {
        match fencing_preflight_is_already_stopped(spec.data_dir.as_path()) {
            Ok(true) => {
                let mut event = process_event(
                    ProcessEventKind::FencingNoop,
                    "ok",
                    SeverityText::Info,
                    "fencing preflight: postgres already stopped",
                );
                let fields = event.fields_mut();
                fields.append_json_map(
                    process_job_fields(&request.id, request.kind.label()).into_attributes(),
                );
                fields.insert("data_dir", spec.data_dir.display().to_string());
                ctx.log
                    .emit_app_event("process_worker::start_job", event)
                    .map_err(|err| {
                        WorkerError::Message(format!("process fencing noop log emit failed: {err}"))
                    })?;
                transition_to_idle(
                    ctx,
                    JobOutcome::Success {
                        id: request.id,
                        job_kind: active_kind(&request.kind),
                        finished_at: now,
                    },
                    now,
                )?;
                return Ok(());
            }
            Ok(false) => {}
            Err(error) => {
                let mut event = process_event(
                    ProcessEventKind::FencingPreflightFailed,
                    "failed",
                    SeverityText::Error,
                    "fencing preflight failed",
                );
                let fields = event.fields_mut();
                fields.append_json_map(
                    process_job_fields(&request.id, request.kind.label()).into_attributes(),
                );
                fields.insert("error", error.to_string());
                ctx.log
                    .emit_app_event("process_worker::start_job", event)
                    .map_err(|err| {
                        WorkerError::Message(format!(
                            "process fencing preflight log emit failed: {err}"
                        ))
                    })?;
                transition_to_idle(
                    ctx,
                    JobOutcome::Failure {
                        id: request.id,
                        job_kind: active_kind(&request.kind),
                        error,
                        finished_at: now,
                    },
                    now,
                )?;
                return Ok(());
            }
        }
    }

    if let ProcessJobKind::StartPostgres(spec) = &request.kind {
        match start_postgres_preflight_is_already_running(spec.data_dir.as_path()) {
            Ok(true) => {
                let mut event = process_event(
                    ProcessEventKind::StartPostgresNoop,
                    "ok",
                    SeverityText::Info,
                    "start postgres preflight: postgres already running",
                );
                let fields = event.fields_mut();
                fields.append_json_map(
                    process_job_fields(&request.id, request.kind.label()).into_attributes(),
                );
                fields.insert("data_dir", spec.data_dir.display().to_string());
                ctx.log
                    .emit_app_event("process_worker::start_job", event)
                    .map_err(|err| {
                        WorkerError::Message(format!(
                            "process start-postgres noop log emit failed: {err}"
                        ))
                    })?;
                transition_to_idle(
                    ctx,
                    JobOutcome::Success {
                        id: request.id,
                        job_kind: active_kind(&request.kind),
                        finished_at: now,
                    },
                    now,
                )?;
                return Ok(());
            }
            Ok(false) => {}
            Err(error) => {
                let mut event = process_event(
                    ProcessEventKind::StartPostgresPreflightFailed,
                    "failed",
                    SeverityText::Error,
                    "start postgres preflight failed",
                );
                let fields = event.fields_mut();
                fields.append_json_map(
                    process_job_fields(&request.id, request.kind.label()).into_attributes(),
                );
                fields.insert("error", error.to_string());
                ctx.log
                    .emit_app_event("process_worker::start_job", event)
                    .map_err(|err| {
                        WorkerError::Message(format!(
                            "process start-postgres preflight log emit failed: {err}"
                        ))
                    })?;
                transition_to_idle(
                    ctx,
                    JobOutcome::Failure {
                        id: request.id,
                        job_kind: active_kind(&request.kind),
                        error,
                        finished_at: now,
                    },
                    now,
                )?;
                return Ok(());
            }
        }
    }

    let command = match build_command(
        &ctx.config,
        &request.id,
        &request.kind,
        ctx.capture_subprocess_output,
    ) {
        Ok(command) => command,
        Err(error) => {
            let mut event = process_event(
                ProcessEventKind::BuildCommandFailed,
                "failed",
                SeverityText::Error,
                "process build command failed",
            );
            let fields = event.fields_mut();
            fields.append_json_map(
                process_job_fields(&request.id, request.kind.label()).into_attributes(),
            );
            fields.insert("error", error.to_string());
            ctx.log
                .emit_app_event("process_worker::start_job", event)
                .map_err(|err| {
                    WorkerError::Message(format!("process build command log emit failed: {err}"))
                })?;
            transition_to_idle(
                ctx,
                JobOutcome::Failure {
                    id: request.id,
                    job_kind: active_kind(&request.kind),
                    error,
                    finished_at: now,
                },
                now,
            )?;
            return Ok(());
        }
    };

    let log_identity = command.log_identity.clone();
    let handle = match ctx.command_runner.spawn(command) {
        Ok(handle) => handle,
        Err(error) => {
            let mut event = process_event(
                ProcessEventKind::SpawnFailed,
                "failed",
                SeverityText::Error,
                "process spawn failed",
            );
            let fields = event.fields_mut();
            fields.append_json_map(
                process_job_fields(&request.id, request.kind.label()).into_attributes(),
            );
            fields.insert("error", error.to_string());
            ctx.log
                .emit_app_event("process_worker::start_job", event)
                .map_err(|err| {
                    WorkerError::Message(format!("process spawn log emit failed: {err}"))
                })?;
            transition_to_idle(
                ctx,
                JobOutcome::Failure {
                    id: request.id,
                    job_kind: active_kind(&request.kind),
                    error,
                    finished_at: now,
                },
                now,
            )?;
            return Ok(());
        }
    };

    let active = ActiveJob {
        id: request.id.clone(),
        kind: active_kind(&request.kind),
        started_at: now,
        deadline_at,
    };

    ctx.active_runtime = Some(ActiveRuntime {
        request,
        deadline_at,
        handle,
        log_identity,
    });
    ctx.state = ProcessState::Running {
        worker: WorkerStatus::Running,
        active,
    };
    let mut event = process_event(
        ProcessEventKind::Started,
        "ok",
        SeverityText::Info,
        "process job started",
    );
    let runtime_fields = ctx
        .active_runtime
        .as_ref()
        .map(|runtime| process_log_identity_fields(&runtime.log_identity).into_attributes())
        .unwrap_or_default();
    event.fields_mut().append_json_map(runtime_fields);
    ctx.log
        .emit_app_event("process_worker::start_job", event)
        .map_err(|err| {
            WorkerError::Message(format!("process job started log emit failed: {err}"))
        })?;
    publish_state(ctx, now)
}

pub(crate) async fn tick_active_job(ctx: &mut ProcessWorkerCtx) -> Result<(), WorkerError> {
    let mut runtime = match ctx.active_runtime.take() {
        Some(runtime) => runtime,
        None => return Ok(()),
    };

    let now = current_time(ctx)?;
    match runtime
        .handle
        .drain_output(PROCESS_OUTPUT_DRAIN_MAX_BYTES)
        .await
    {
        Ok(lines) => {
            for line in lines {
                if let Err(err) =
                    emit_subprocess_line(&ctx.log, &runtime.log_identity, line.clone())
                {
                    emit_process_output_emit_failed(ctx, &runtime.log_identity, &line, &err)?;
                }
            }
        }
        Err(err) => {
            let mut event = process_event(
                ProcessEventKind::OutputDrainFailed,
                "failed",
                SeverityText::Warn,
                "process output drain failed",
            );
            let fields = event.fields_mut();
            fields.append_json_map(
                process_log_identity_fields(&runtime.log_identity).into_attributes(),
            );
            fields.insert("error", err.to_string());
            ctx.log
                .emit_app_event("process_worker::tick_active_job", event)
                .map_err(|emit_err| {
                    WorkerError::Message(format!(
                        "process output drain log emit failed: {emit_err}"
                    ))
                })?;
        }
    }
    if now.0 >= runtime.deadline_at.0 {
        let mut timeout_event = process_event(
            ProcessEventKind::Timeout,
            "timeout",
            SeverityText::Warn,
            "process job timed out; cancelling",
        );
        timeout_event
            .fields_mut()
            .append_json_map(process_log_identity_fields(&runtime.log_identity).into_attributes());
        ctx.log
            .emit_app_event("process_worker::tick_active_job", timeout_event)
            .map_err(|err| {
                WorkerError::Message(format!("process timeout log emit failed: {err}"))
            })?;
        let cancel_result = runtime.handle.cancel().await;
        match runtime
            .handle
            .drain_output(PROCESS_OUTPUT_DRAIN_MAX_BYTES)
            .await
        {
            Ok(lines) => {
                for line in lines {
                    if let Err(err) =
                        emit_subprocess_line(&ctx.log, &runtime.log_identity, line.clone())
                    {
                        emit_process_output_emit_failed(ctx, &runtime.log_identity, &line, &err)?;
                    }
                }
            }
            Err(err) => {
                let mut event = process_event(
                    ProcessEventKind::OutputDrainFailed,
                    "failed",
                    SeverityText::Warn,
                    "process output drain failed",
                );
                let fields = event.fields_mut();
                fields.append_json_map(
                    process_log_identity_fields(&runtime.log_identity).into_attributes(),
                );
                fields.insert("error", err.to_string());
                ctx.log
                    .emit_app_event("process_worker::tick_active_job", event)
                    .map_err(|emit_err| {
                        WorkerError::Message(format!(
                            "process output drain log emit failed: {emit_err}"
                        ))
                    })?;
            }
        }
        let outcome = match cancel_result {
            Ok(()) => JobOutcome::Timeout {
                id: runtime.request.id,
                job_kind: active_kind(&runtime.request.kind),
                finished_at: now,
            },
            Err(error) => JobOutcome::Failure {
                id: runtime.request.id,
                job_kind: active_kind(&runtime.request.kind),
                error,
                finished_at: now,
            },
        };
        transition_to_idle(ctx, outcome, now)?;
        return Ok(());
    }

    let poll = runtime.handle.poll_exit();
    match poll {
        Ok(None) => {
            ctx.active_runtime = Some(runtime);
            Ok(())
        }
        Ok(Some(ProcessExit::Success)) => {
            match runtime
                .handle
                .drain_output(PROCESS_OUTPUT_DRAIN_MAX_BYTES)
                .await
            {
                Ok(lines) => {
                    for line in lines {
                        if let Err(err) =
                            emit_subprocess_line(&ctx.log, &runtime.log_identity, line.clone())
                        {
                            emit_process_output_emit_failed(
                                ctx,
                                &runtime.log_identity,
                                &line,
                                &err,
                            )?;
                        }
                    }
                }
                Err(err) => {
                    let mut event = process_event(
                        ProcessEventKind::OutputDrainFailed,
                        "failed",
                        SeverityText::Warn,
                        "process output drain failed",
                    );
                    let fields = event.fields_mut();
                    fields.append_json_map(
                        process_log_identity_fields(&runtime.log_identity).into_attributes(),
                    );
                    fields.insert("error", err.to_string());
                    ctx.log
                        .emit_app_event("process_worker::tick_active_job", event)
                        .map_err(|emit_err| {
                            WorkerError::Message(format!(
                                "process output drain log emit failed: {emit_err}"
                            ))
                        })?;
                }
            }
            let job_id = runtime.request.id.clone();
            let outcome = JobOutcome::Success {
                id: job_id.clone(),
                job_kind: active_kind(&runtime.request.kind),
                finished_at: now,
            };
            let mut event = process_event(
                ProcessEventKind::Exited,
                "ok",
                SeverityText::Info,
                "process job exited successfully",
            );
            event.fields_mut().append_json_map(
                process_log_identity_fields(&runtime.log_identity).into_attributes(),
            );
            ctx.log
                .emit_app_event("process_worker::tick_active_job", event)
                .map_err(|err| {
                    WorkerError::Message(format!("process exit log emit failed: {err}"))
                })?;
            transition_to_idle(ctx, outcome, now)
        }
        Ok(Some(exit)) => {
            match runtime
                .handle
                .drain_output(PROCESS_OUTPUT_DRAIN_MAX_BYTES)
                .await
            {
                Ok(lines) => {
                    for line in lines {
                        if let Err(err) =
                            emit_subprocess_line(&ctx.log, &runtime.log_identity, line.clone())
                        {
                            emit_process_output_emit_failed(
                                ctx,
                                &runtime.log_identity,
                                &line,
                                &err,
                            )?;
                        }
                    }
                }
                Err(err) => {
                    let mut event = process_event(
                        ProcessEventKind::OutputDrainFailed,
                        "failed",
                        SeverityText::Warn,
                        "process output drain failed",
                    );
                    let fields = event.fields_mut();
                    fields.append_json_map(
                        process_log_identity_fields(&runtime.log_identity).into_attributes(),
                    );
                    fields.insert("error", err.to_string());
                    ctx.log
                        .emit_app_event("process_worker::tick_active_job", event)
                        .map_err(|emit_err| {
                            WorkerError::Message(format!(
                                "process output drain log emit failed: {emit_err}"
                            ))
                        })?;
                }
            }
            let exit_error = ProcessError::from_exit(exit);
            let job_id = runtime.request.id.clone();
            let outcome = JobOutcome::Failure {
                id: job_id.clone(),
                job_kind: active_kind(&runtime.request.kind),
                error: exit_error.clone(),
                finished_at: now,
            };
            let mut event = process_event(
                ProcessEventKind::Exited,
                "failed",
                SeverityText::Warn,
                "process job exited unsuccessfully",
            );
            let fields = event.fields_mut();
            fields.append_json_map(
                process_log_identity_fields(&runtime.log_identity).into_attributes(),
            );
            fields.insert("error", exit_error.to_string());
            ctx.log
                .emit_app_event("process_worker::tick_active_job", event)
                .map_err(|err| {
                    WorkerError::Message(format!("process exit log emit failed: {err}"))
                })?;
            transition_to_idle(ctx, outcome, now)
        }
        Err(error) => {
            match runtime
                .handle
                .drain_output(PROCESS_OUTPUT_DRAIN_MAX_BYTES)
                .await
            {
                Ok(lines) => {
                    for line in lines {
                        if let Err(err) =
                            emit_subprocess_line(&ctx.log, &runtime.log_identity, line.clone())
                        {
                            emit_process_output_emit_failed(
                                ctx,
                                &runtime.log_identity,
                                &line,
                                &err,
                            )?;
                        }
                    }
                }
                Err(err) => {
                    let mut event = process_event(
                        ProcessEventKind::OutputDrainFailed,
                        "failed",
                        SeverityText::Warn,
                        "process output drain failed",
                    );
                    let fields = event.fields_mut();
                    fields.append_json_map(
                        process_log_identity_fields(&runtime.log_identity).into_attributes(),
                    );
                    fields.insert("error", err.to_string());
                    ctx.log
                        .emit_app_event("process_worker::tick_active_job", event)
                        .map_err(|emit_err| {
                            WorkerError::Message(format!(
                                "process output drain log emit failed: {emit_err}"
                            ))
                        })?;
                }
            }
            let job_id = runtime.request.id.clone();
            let outcome = JobOutcome::Failure {
                id: job_id.clone(),
                job_kind: active_kind(&runtime.request.kind),
                error,
                finished_at: now,
            };
            let mut event = process_event(
                ProcessEventKind::PollFailed,
                "failed",
                SeverityText::Error,
                "process job poll failed",
            );
            let fields = event.fields_mut();
            fields.append_json_map(
                process_log_identity_fields(&runtime.log_identity).into_attributes(),
            );
            fields.insert("error", outcome_error_string(&outcome));
            ctx.log
                .emit_app_event("process_worker::tick_active_job", event)
                .map_err(|err| {
                    WorkerError::Message(format!("process poll failure log emit failed: {err}"))
                })?;
            transition_to_idle(ctx, outcome, now)
        }
    }
}

fn process_log_identity_fields(identity: &ProcessLogIdentity) -> StructuredFields {
    let mut fields = process_job_fields(&identity.job_id, identity.job_kind.as_str());
    fields.insert("binary", identity.binary.clone());
    fields
}

fn emit_process_output_emit_failed(
    ctx: &ProcessWorkerCtx,
    identity: &ProcessLogIdentity,
    line: &ProcessOutputLine,
    error: &crate::logging::LogError,
) -> Result<(), WorkerError> {
    let mut event = process_event(
        ProcessEventKind::OutputEmitFailed,
        "failed",
        SeverityText::Warn,
        "process subprocess output emit failed",
    );
    let fields = event.fields_mut();
    fields.append_json_map(process_log_identity_fields(identity).into_attributes());
    fields.insert(
        "stream",
        match line.stream {
            ProcessOutputStream::Stdout => "stdout",
            ProcessOutputStream::Stderr => "stderr",
        },
    );
    fields.insert("bytes_len", line.bytes.len());
    fields.insert("error", error.to_string());
    ctx.log
        .emit_app_event("process_worker::emit_subprocess_line", event)
        .map_err(|emit_err| {
            WorkerError::Message(format!(
                "process output emit failure log emit failed: {emit_err}"
            ))
        })?;
    Ok(())
}

fn outcome_error_string(outcome: &JobOutcome) -> String {
    match outcome {
        JobOutcome::Success { .. } => "success".to_string(),
        JobOutcome::Timeout { .. } => "timeout".to_string(),
        JobOutcome::Failure { error, .. } => error.to_string(),
    }
}

fn emit_subprocess_line(
    log: &LogHandle,
    identity: &ProcessLogIdentity,
    line: ProcessOutputLine,
) -> Result<(), crate::logging::LogError> {
    let stream = match line.stream {
        ProcessOutputStream::Stdout => SubprocessStream::Stdout,
        ProcessOutputStream::Stderr => SubprocessStream::Stderr,
    };

    log.emit_raw_record(
        SubprocessLineRecord::new(
            crate::logging::LogProducer::PgTool,
            "process_worker",
            identity.job_id.0.clone(),
            identity.job_kind.clone(),
            identity.binary.clone(),
            stream,
            line.bytes,
        )
        .into_raw_record()?,
    )
}

fn transition_to_idle(
    ctx: &mut ProcessWorkerCtx,
    outcome: JobOutcome,
    now: UnixMillis,
) -> Result<(), WorkerError> {
    ctx.state = ProcessState::Idle {
        worker: WorkerStatus::Running,
        last_outcome: Some(outcome),
    };
    publish_state(ctx, now)
}

fn publish_state(ctx: &mut ProcessWorkerCtx, now: UnixMillis) -> Result<(), WorkerError> {
    ctx.publisher
        .publish(ctx.state.clone(), now)
        .map_err(|err| WorkerError::Message(format!("process publish failed: {err}")))?;
    Ok(())
}

fn current_time(ctx: &mut ProcessWorkerCtx) -> Result<UnixMillis, WorkerError> {
    (ctx.now)()
}

pub(crate) fn system_now_unix_millis() -> Result<UnixMillis, WorkerError> {
    let elapsed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|err| WorkerError::Message(format!("system clock before unix epoch: {err}")))?;
    let millis = u64::try_from(elapsed.as_millis())
        .map_err(|err| WorkerError::Message(format!("unix millis conversion failed: {err}")))?;
    Ok(UnixMillis(millis))
}

pub(crate) fn timeout_for_kind(kind: &ProcessJobKind, config: &ProcessConfig) -> u64 {
    match kind {
        ProcessJobKind::Bootstrap(spec) => spec.timeout_ms.unwrap_or(config.bootstrap_timeout_ms),
        ProcessJobKind::BaseBackup(spec) => spec.timeout_ms.unwrap_or(config.bootstrap_timeout_ms),
        ProcessJobKind::PgRewind(spec) => spec.timeout_ms.unwrap_or(config.pg_rewind_timeout_ms),
        ProcessJobKind::Fencing(spec) => spec.timeout_ms.unwrap_or(config.fencing_timeout_ms),
        ProcessJobKind::Promote(spec) => spec.timeout_ms.unwrap_or(config.bootstrap_timeout_ms),
        ProcessJobKind::Demote(spec) => spec.timeout_ms.unwrap_or(config.fencing_timeout_ms),
        ProcessJobKind::StartPostgres(spec) => {
            spec.timeout_ms.unwrap_or(config.bootstrap_timeout_ms)
        }
    }
}

fn active_kind(kind: &ProcessJobKind) -> ActiveJobKind {
    match kind {
        ProcessJobKind::Bootstrap(_) => ActiveJobKind::Bootstrap,
        ProcessJobKind::BaseBackup(_) => ActiveJobKind::BaseBackup,
        ProcessJobKind::PgRewind(_) => ActiveJobKind::PgRewind,
        ProcessJobKind::Promote(_) => ActiveJobKind::Promote,
        ProcessJobKind::Demote(_) => ActiveJobKind::Demote,
        ProcessJobKind::StartPostgres(_) => ActiveJobKind::StartPostgres,
        ProcessJobKind::Fencing(_) => ActiveJobKind::Fencing,
    }
}

pub(crate) fn build_command(
    config: &ProcessConfig,
    job_id: &JobId,
    kind: &ProcessJobKind,
    capture_output: bool,
) -> Result<ProcessCommandSpec, ProcessError> {
    match kind {
        ProcessJobKind::Bootstrap(spec) => {
            validate_non_empty_path("bootstrap.data_dir", &spec.data_dir)?;
            if spec.superuser_username.trim().is_empty() {
                return Err(ProcessError::InvalidSpec(
                    "bootstrap.superuser_username must not be empty".to_string(),
                ));
            }
            let program = config.binaries.initdb.clone();
            Ok(ProcessCommandSpec {
                program: program.clone(),
                args: vec![
                    "-D".to_string(),
                    spec.data_dir.display().to_string(),
                    "-A".to_string(),
                    "trust".to_string(),
                    "-U".to_string(),
                    spec.superuser_username.clone(),
                ],
                env: Vec::new(),
                capture_output,
                log_identity: ProcessLogIdentity {
                    job_id: job_id.clone(),
                    job_kind: job_kind_label(kind).to_string(),
                    binary: binary_label(program.as_path()),
                },
            })
        }
        ProcessJobKind::BaseBackup(spec) => {
            validate_non_empty_path("basebackup.data_dir", &spec.data_dir)?;
            if spec.source.conninfo.host.trim().is_empty() {
                return Err(ProcessError::InvalidSpec(
                    "basebackup.source_conninfo.host must not be empty".to_string(),
                ));
            }
            if spec.source.conninfo.user.trim().is_empty() {
                return Err(ProcessError::InvalidSpec(
                    "basebackup.source_conninfo.user must not be empty".to_string(),
                ));
            }
            if spec.source.conninfo.dbname.trim().is_empty() {
                return Err(ProcessError::InvalidSpec(
                    "basebackup.source_conninfo.dbname must not be empty".to_string(),
                ));
            }
            let program = config.binaries.pg_basebackup.clone();
            Ok(ProcessCommandSpec {
                program: program.clone(),
                args: vec![
                    "--dbname".to_string(),
                    render_pg_conninfo(&spec.source.conninfo),
                    "-D".to_string(),
                    spec.data_dir.display().to_string(),
                    "-Fp".to_string(),
                    "-Xs".to_string(),
                ],
                env: role_auth_env(&spec.source.auth),
                capture_output,
                log_identity: ProcessLogIdentity {
                    job_id: job_id.clone(),
                    job_kind: job_kind_label(kind).to_string(),
                    binary: binary_label(program.as_path()),
                },
            })
        }
        ProcessJobKind::PgRewind(spec) => {
            validate_non_empty_path("pg_rewind.target_data_dir", &spec.target_data_dir)?;
            if spec.source.conninfo.host.trim().is_empty() {
                return Err(ProcessError::InvalidSpec(
                    "pg_rewind.source_conninfo.host must not be empty".to_string(),
                ));
            }
            if spec.source.conninfo.user.trim().is_empty() {
                return Err(ProcessError::InvalidSpec(
                    "pg_rewind.source_conninfo.user must not be empty".to_string(),
                ));
            }
            if spec.source.conninfo.dbname.trim().is_empty() {
                return Err(ProcessError::InvalidSpec(
                    "pg_rewind.source_conninfo.dbname must not be empty".to_string(),
                ));
            }
            let program = config.binaries.pg_rewind.clone();
            Ok(ProcessCommandSpec {
                program: program.clone(),
                args: vec![
                    "--target-pgdata".to_string(),
                    spec.target_data_dir.display().to_string(),
                    "--source-server".to_string(),
                    render_pg_conninfo(&spec.source.conninfo),
                ],
                env: role_auth_env(&spec.source.auth),
                capture_output,
                log_identity: ProcessLogIdentity {
                    job_id: job_id.clone(),
                    job_kind: job_kind_label(kind).to_string(),
                    binary: binary_label(program.as_path()),
                },
            })
        }
        ProcessJobKind::Promote(spec) => {
            validate_non_empty_path("promote.data_dir", &spec.data_dir)?;
            let mut args = vec![
                "-D".to_string(),
                spec.data_dir.display().to_string(),
                "promote".to_string(),
                "-w".to_string(),
            ];
            if let Some(wait_seconds) = spec.wait_seconds {
                args.push("-t".to_string());
                args.push(wait_seconds.to_string());
            }
            let program = config.binaries.pg_ctl.clone();
            Ok(ProcessCommandSpec {
                program: program.clone(),
                args,
                env: Vec::new(),
                capture_output,
                log_identity: ProcessLogIdentity {
                    job_id: job_id.clone(),
                    job_kind: job_kind_label(kind).to_string(),
                    binary: binary_label(program.as_path()),
                },
            })
        }
        ProcessJobKind::Demote(spec) => {
            validate_non_empty_path("demote.data_dir", &spec.data_dir)?;
            let program = config.binaries.pg_ctl.clone();
            Ok(ProcessCommandSpec {
                program: program.clone(),
                args: vec![
                    "-D".to_string(),
                    spec.data_dir.display().to_string(),
                    "stop".to_string(),
                    "-m".to_string(),
                    spec.mode.as_pg_ctl_arg().to_string(),
                    "-w".to_string(),
                ],
                env: Vec::new(),
                capture_output,
                log_identity: ProcessLogIdentity {
                    job_id: job_id.clone(),
                    job_kind: job_kind_label(kind).to_string(),
                    binary: binary_label(program.as_path()),
                },
            })
        }
        ProcessJobKind::StartPostgres(spec) => {
            validate_non_empty_path("start_postgres.data_dir", &spec.data_dir)?;
            validate_non_empty_path("start_postgres.config_file", &spec.config_file)?;
            validate_non_empty_path("start_postgres.log_file", &spec.log_file)?;
            let wait_seconds = spec.wait_seconds.unwrap_or(PG_CTL_DEFAULT_WAIT_SECONDS);
            let option_tokens = vec![
                "-c".to_string(),
                format!("config_file={}", spec.config_file.display()),
            ];
            let options = render_pg_ctl_option_string(&option_tokens)?;
            let program = config.binaries.pg_ctl.clone();
            Ok(ProcessCommandSpec {
                program: program.clone(),
                args: vec![
                    "-D".to_string(),
                    spec.data_dir.display().to_string(),
                    "-l".to_string(),
                    spec.log_file.display().to_string(),
                    "-o".to_string(),
                    options,
                    "start".to_string(),
                    "-w".to_string(),
                    "-t".to_string(),
                    wait_seconds.to_string(),
                ],
                env: Vec::new(),
                capture_output,
                log_identity: ProcessLogIdentity {
                    job_id: job_id.clone(),
                    job_kind: job_kind_label(kind).to_string(),
                    binary: binary_label(program.as_path()),
                },
            })
        }
        ProcessJobKind::Fencing(spec) => {
            validate_non_empty_path("fencing.data_dir", &spec.data_dir)?;
            let program = config.binaries.pg_ctl.clone();
            Ok(ProcessCommandSpec {
                program: program.clone(),
                args: vec![
                    "-D".to_string(),
                    spec.data_dir.display().to_string(),
                    "stop".to_string(),
                    "-m".to_string(),
                    spec.mode.as_pg_ctl_arg().to_string(),
                    "-w".to_string(),
                ],
                env: Vec::new(),
                capture_output,
                log_identity: ProcessLogIdentity {
                    job_id: job_id.clone(),
                    job_kind: job_kind_label(kind).to_string(),
                    binary: binary_label(program.as_path()),
                },
            })
        }
    }
}

fn role_auth_env(auth: &RoleAuthConfig) -> Vec<ProcessEnvVar> {
    match auth {
        RoleAuthConfig::Tls => Vec::new(),
        RoleAuthConfig::Password { password } => vec![ProcessEnvVar {
            key: "PGPASSWORD".to_string(),
            value: ProcessEnvValue::Secret(password.clone()),
        }],
    }
}

fn job_kind_label(kind: &ProcessJobKind) -> &'static str {
    match kind {
        ProcessJobKind::Bootstrap(_) => "bootstrap",
        ProcessJobKind::BaseBackup(_) => "basebackup",
        ProcessJobKind::PgRewind(_) => "pg_rewind",
        ProcessJobKind::Promote(_) => "promote",
        ProcessJobKind::Demote(_) => "demote",
        ProcessJobKind::StartPostgres(_) => "start_postgres",
        ProcessJobKind::Fencing(_) => "fencing",
    }
}

fn binary_label(path: &std::path::Path) -> String {
    match path.file_name().and_then(|s| s.to_str()) {
        Some(name) if !name.trim().is_empty() => name.to_string(),
        _ => path.display().to_string(),
    }
}

fn validate_non_empty_path(field: &str, value: &std::path::Path) -> Result<(), ProcessError> {
    if value.as_os_str().is_empty() {
        return Err(ProcessError::InvalidSpec(format!(
            "{field} must not be empty"
        )));
    }
    Ok(())
}

fn render_pg_ctl_option_string(tokens: &[String]) -> Result<String, ProcessError> {
    let mut out = String::new();
    for (index, raw) in tokens.iter().enumerate() {
        let escaped = escape_pg_ctl_option_token(raw.as_str())?;
        if index > 0 {
            out.push(' ');
        }
        out.push_str(escaped.as_str());
    }
    Ok(out)
}

fn escape_pg_ctl_option_token(token: &str) -> Result<String, ProcessError> {
```
