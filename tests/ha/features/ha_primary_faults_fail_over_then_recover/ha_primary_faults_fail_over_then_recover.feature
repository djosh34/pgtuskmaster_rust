# Merged from: ha_primary_killed_then_rejoins_as_replica, ha_primary_killed_with_concurrent_writes,
# ha_old_primary_partitioned_from_majority_majority_elects_new_primary,
# ha_old_primary_partitioned_from_majority_on_three_etcd_majority_elects_new_primary,
# ha_primary_storage_stalled_then_new_primary_takes_over, ha_repeated_failovers_preserve_single_primary,
# ha_lagging_replica_is_not_promoted_during_failover, ha_dcs_and_api_faults_then_healed_cluster_converges.
# Deleted during the merge because they only differed by proof/workload/history assertions:
# ha_primary_killed_custom_roles_survive_rejoin.
# Deleted instead of forcing a misleading reduced outcome:
# ha_primary_storage_stalled_then_new_primary_takes_over.
Feature: ha_primary_faults_fail_over_then_recover
  Scenario: a killed primary fails over and the cluster returns to health after restart
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "old_primary"
    When I kill the node named "old_primary"
    Then cluster becomes healthy
    When I restart the node named "old_primary"
    Then cluster becomes healthy

  Scenario: a primary isolated from the majority still lets the cluster recover
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "old_primary"
    When I fully isolate the node named "old_primary" from the cluster
    Then cluster becomes healthy
    When I heal all network faults
    Then cluster becomes healthy
