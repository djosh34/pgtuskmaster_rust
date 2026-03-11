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
