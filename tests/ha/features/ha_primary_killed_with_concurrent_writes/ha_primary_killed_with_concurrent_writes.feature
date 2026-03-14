Feature: ha_primary_killed_with_concurrent_writes
  Scenario: a forced failover preserves single-primary behavior under concurrent writes
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "old_primary"
    And I create one workload table for this feature
    When I start a bounded concurrent write workload and record commit outcomes
    And I kill the node named "old_primary"
    Then exactly one primary exists across 2 running nodes as "new_primary"
    When I stop the workload and verify it committed at least one row without recording workload proof rows
    Then there is no dual-primary evidence and no split-brain write evidence during the transition window
    And the recorded workload evidence establishes a fencing cutoff with no later commits
    And I insert proof row "post-failover-proof" through "new_primary"
    Then the 2 online nodes contain at least the recorded proof rows
    When I restart the node named "old_primary"
    Then the node named "old_primary" rejoins as a replica
    And the 3 online nodes contain at least the recorded proof rows
