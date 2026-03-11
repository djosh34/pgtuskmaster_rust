Feature: no quorum fencing blocks post cutoff commits
  Scenario: fail-safe fencing eventually rejects post-cutoff writes and preserves pre-cutoff commits
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "initial_primary"
    And I create one workload table for this feature
    When I start a bounded concurrent write workload and record commit outcomes
    And I stop a DCS quorum majority
    Then there is no operator-visible primary across 3 online node
    And every running node reports fail_safe in debug output
    When I restore DCS quorum
    Then I wait for exactly one stable primary as "restored_primary"
    When I stop the workload and verify it committed at least one row
    Then the recorded workload evidence establishes a fencing cutoff with no later commits
    And the 3 online nodes contain exactly the recorded proof rows
