Feature: replica outage keeps primary stable
  Scenario: a replica outage keeps the current primary stable
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "initial_primary"
    And I choose one non-primary node as "stopped_replica"
    And I create a proof table for this feature
    And I insert proof row "1:before-replica-outage" through "initial_primary"
    Then the 3 online nodes contain exactly the recorded proof rows
    When I kill the node named "stopped_replica"
    Then pgtm primary points to "initial_primary"
    And the primary named "initial_primary" remains the only primary
    And the remaining online non-primary node is a replica
    When I insert proof row "2:during-replica-outage" through "initial_primary"
    And I restart the node named "stopped_replica"
    Then the node named "stopped_replica" rejoins as a replica
    And the 3 online nodes contain exactly the recorded proof rows
