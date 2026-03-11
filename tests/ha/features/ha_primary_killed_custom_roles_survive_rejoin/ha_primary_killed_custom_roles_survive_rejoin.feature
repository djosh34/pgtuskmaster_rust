Feature: ha_primary_killed_custom_roles_survive_rejoin
  Scenario: non-default replicator and rewinder roles survive failover and rejoin
    Given the "three_node_custom_roles" harness is running
    And I wait for exactly one stable primary as "old_primary"
    And I create a proof table for this feature
    And I insert proof row "1:before-custom-role-failover" through "old_primary"
    When I kill the node named "old_primary"
    Then exactly one primary exists across 2 running nodes as "new_primary"
    When I insert proof row "2:after-custom-role-failover" through "new_primary"
    And I restart the node named "old_primary"
    Then the node named "old_primary" rejoins as a replica
    And the 3 online nodes contain exactly the recorded proof rows
