# Merged from: ha_planned_switchover_changes_primary_cleanly,
# ha_targeted_switchover_promotes_requested_replica,
# ha_planned_switchover_with_concurrent_writes,
# ha_targeted_switchover_to_degraded_replica_is_rejected.
# Deleted during the merge because workload, proof-row, pgtm-view, and rejection-detail assertions
# no longer belong in feature text.
Feature: ha_operator_switchovers
  Scenario: a planned switchover keeps the cluster healthy
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "old_primary"
    When I request a planned switchover
    Then cluster becomes healthy
  Scenario: a targeted switchover promotes the requested replica
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "old_primary"
    And I choose one non-primary node as "target_replica"
    When I request a targeted switchover to "target_replica"
    Then "target_replica" becomes primary
    And cluster becomes healthy
  Scenario: a targeted switchover to a degraded replica is rejected while the cluster stays healthy
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "initial_primary"
    And I choose one non-primary node as "ineligible_replica"
    When I fully isolate the node named "ineligible_replica" from the cluster
    And I attempt a targeted switchover to "ineligible_replica" and capture the operator-visible error
    Then cluster becomes healthy
    When I heal all network faults
    Then cluster becomes healthy
