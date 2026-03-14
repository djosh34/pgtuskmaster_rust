Feature: ha_replication_path_isolated_then_healed_replicas_catch_up
  Scenario: replicas lag during replication-path isolation and catch up after heal
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "initial_primary"
    And I choose the two non-primary nodes as "replica_a" and "replica_b"
    And I create a proof table for this feature
    And I insert proof row "1:before-postgres-path-isolation" through "initial_primary"
    When I isolate the nodes named "initial_primary" and "replica_a" on the "postgres" path
    And I isolate the nodes named "initial_primary" and "replica_b" on the "postgres" path
    And I insert proof row "2:during-postgres-path-isolation" through "initial_primary"
    Then the nodes named "replica_a" and "replica_b" do not yet contain proof row "2:during-postgres-path-isolation"
    When I heal all network faults
    Then the 3 online nodes contain exactly the recorded proof rows
