Feature: ha_rewind_fails_then_basebackup_rejoins_old_primary
  Scenario: a rewind failure still allows the old primary to rejoin as a replica
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "old_primary"
    And I enable the "pg_rewind" blocker on the node named "old_primary"
    And I create a proof table for this feature
    And I insert proof row "1:before-rewind-failure" through "old_primary"
    When I fully isolate the node named "old_primary" from the cluster
    And I cut the node named "old_primary" off from DCS
    Then exactly one primary exists across 2 running nodes as "new_primary"
    When I insert proof row "2:after-failover" through "new_primary"
    And I heal network faults on the node named "old_primary"
    Then the node named "old_primary" emitted blocker evidence for "pg_rewind"
    And the node named "old_primary" rejoins as a replica
    And the 3 online nodes contain exactly the recorded proof rows
