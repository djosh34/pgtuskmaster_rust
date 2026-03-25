# Merged from: ha_two_replicas_stopped_then_one_replica_restarted_restores_quorum,
# ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins,
# ha_two_nodes_stopped_then_one_healthy_node_restarted_restores_service_while_other_stays_broken,
# ha_basebackup_clone_blocked_then_unblocked_replica_recovers,
# ha_rewind_fails_then_basebackup_rejoins_old_primary,
# ha_old_primary_partitioned_then_healed_rejoins_as_replica_after_majority_failover,
# ha_broken_replica_rejoin_attempt_does_not_destabilize_quorum.
# Deleted during the merge because blocker evidence, proof-row tracking, and primary-history checks
# now belong to scenario-agnostic runners instead of feature text.
Feature: ha_rejoin_and_restart_recovery
  Scenario: restarting replicas restores health after a two-replica outage
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "initial_primary"
    And I choose the two non-primary nodes as "stopped_node_a" and "stopped_node_b"
    When I kill the nodes named "stopped_node_a" and "stopped_node_b"
    Then cluster becomes healthy
    When I restart the node named "stopped_node_a"
    Then cluster becomes healthy
    When I restart the node named "stopped_node_b"
    Then cluster becomes healthy

  Scenario: a full database outage stays unhealthy until nodes return
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "initial_primary"
    When I kill all database nodes
    Then cluster becomes unhealthy
    When I start only the fixed nodes "node-a" and "node-b"
    Then cluster becomes healthy
    When I restart the node named "node-c"
    Then cluster becomes healthy

  Scenario: a blocked basebackup recovery keeps the cluster healthy and recovers after unblock
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "initial_primary"
    And I choose one non-primary node as "blocked_node"
    When I enable the "pg_basebackup" blocker on the node named "blocked_node"
    And I kill the node named "blocked_node"
    And I wipe the data directory on the node named "blocked_node"
    And I restart the node named "blocked_node"
    Then cluster becomes healthy
    When I disable the "pg_basebackup" blocker on the node named "blocked_node"
    And I restart the node named "blocked_node"
    Then cluster becomes healthy

  Scenario: a rewind failure still lets the old primary rejoin without losing cluster health
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "old_primary"
    And I enable the "pg_rewind" blocker on the node named "old_primary"
    When I fully isolate the node named "old_primary" from the cluster
    And I cut the node named "old_primary" off from DCS
    Then cluster becomes healthy
    When I heal network faults on the node named "old_primary"
    Then cluster becomes healthy
