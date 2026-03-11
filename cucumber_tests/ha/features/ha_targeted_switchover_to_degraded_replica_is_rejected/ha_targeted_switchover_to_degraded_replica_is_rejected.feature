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
