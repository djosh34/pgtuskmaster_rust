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
