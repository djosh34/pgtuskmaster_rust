Feature: ha_planned_switchover_with_concurrent_writes
  Scenario: a planned switchover preserves single-primary behavior under concurrent writes
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "old_primary"
    And I create one workload table for this feature
    When I start a bounded concurrent write workload and record commit outcomes
    And I request a planned switchover
    Then I wait for a different stable primary than "old_primary" as "new_primary"
    And there is no dual-primary evidence during the transition window
    When I stop the workload and verify it committed at least one row
    And I insert proof row "post-switchover-proof" through "new_primary"
    Then the 3 online nodes contain exactly the recorded proof rows
