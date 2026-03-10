Feature: primary crash failover and rejoin
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
