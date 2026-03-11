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
