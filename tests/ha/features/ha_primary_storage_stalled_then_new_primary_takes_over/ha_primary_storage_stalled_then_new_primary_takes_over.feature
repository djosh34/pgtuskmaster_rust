Feature: ha_primary_storage_stalled_then_new_primary_takes_over
  Scenario: a wedged primary is replaced without becoming authoritative again
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "initial_primary"
    And I create a proof table for this feature
    And I insert proof row "1:before-storage-stall" through "initial_primary"
    And I record marker "storage_stall"
    When I wedge the node named "initial_primary"
    Then I wait for a different stable primary than "initial_primary" as "final_primary"
    And the node named "initial_primary" never becomes primary after marker "storage_stall"
    And there is no dual-primary evidence during the transition window
    When I insert proof row "2:after-storage-stall-failover" through "final_primary"
    And I unwedge the node named "initial_primary"
    Then the node named "initial_primary" rejoins as a replica
    And the 3 online nodes contain exactly the recorded proof rows
