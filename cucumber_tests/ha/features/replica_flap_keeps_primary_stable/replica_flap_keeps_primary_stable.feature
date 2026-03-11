Feature: replica flap keeps primary stable
  Scenario: repeatedly flapping a replica keeps the same primary
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "initial_primary"
    And I choose one non-primary node as "flapping_replica"
    And I create a proof table for this feature
    And I insert proof row "1:before-flap" through "initial_primary"
    Then the 3 online nodes contain exactly the recorded proof rows
    When I kill the node named "flapping_replica"
    Then the primary named "initial_primary" remains the only primary
    When I insert proof row "2:during-flap-cycle-1" through "initial_primary"
    And I restart the node named "flapping_replica"
    Then the node named "flapping_replica" rejoins as a replica
    When I kill the node named "flapping_replica"
    Then the primary named "initial_primary" remains the only primary
    When I insert proof row "3:during-flap-cycle-2" through "initial_primary"
    And I restart the node named "flapping_replica"
    Then the node named "flapping_replica" rejoins as a replica
    When I kill the node named "flapping_replica"
    Then the primary named "initial_primary" remains the only primary
    When I insert proof row "4:during-flap-cycle-3" through "initial_primary"
    And I restart the node named "flapping_replica"
    Then the node named "flapping_replica" rejoins as a replica
    And the 3 online nodes contain exactly the recorded proof rows
