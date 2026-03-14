Feature: ha_replica_partitioned_from_majority_on_three_etcd_primary_stays_primary
  Scenario: an isolated replica with only its local etcd does not self-promote while the majority preserves one primary
    Given the "three_node_three_etcd" harness is running
    And I wait for exactly one stable primary as "stable_primary"
    And I choose one non-primary node as "isolated_replica"
    And I create a proof table for this feature
    And I insert proof row "1:before-replica-minority-partition" through "stable_primary"
    When I start tracking primary history
    And I fully isolate the node named "isolated_replica" from the cluster
    Then the primary named "stable_primary" remains the only primary
    And the primary history never included "isolated_replica"
    When I insert proof row "2:on-majority-during-replica-partition" through "stable_primary"
    And I heal all network faults
    Then the 3 online nodes contain exactly the recorded proof rows
