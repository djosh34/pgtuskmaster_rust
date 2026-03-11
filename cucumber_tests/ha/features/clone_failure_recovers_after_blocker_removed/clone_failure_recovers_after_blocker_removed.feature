Feature: clone failure recovers after blocker removed
  Scenario: a blocked basebackup clone path recovers after the blocker is removed
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "initial_primary"
    And I choose one non-primary node as "blocked_node"
    And I create a proof table for this feature
    And I insert proof row "1:before-clone-failure" through "initial_primary"
    When I enable the "pg_basebackup" blocker on the node named "blocked_node"
    And I kill the node named "blocked_node"
    And I wipe the data directory on the node named "blocked_node"
    And I start tracking primary history
    And I restart the node named "blocked_node"
    And I insert proof row "2:during-clone-failure" through "initial_primary"
    Then the node named "blocked_node" is not queryable
    And the primary history never included "blocked_node"
    When I disable the "pg_basebackup" blocker on the node named "blocked_node"
    And I restart the node named "blocked_node"
    Then the node named "blocked_node" emitted blocker evidence for "pg_basebackup"
    And the node named "blocked_node" rejoins as a replica
    And the 3 online nodes contain exactly the recorded proof rows
