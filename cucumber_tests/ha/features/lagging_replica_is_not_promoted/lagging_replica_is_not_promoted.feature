Feature: lagging replica is not promoted
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
