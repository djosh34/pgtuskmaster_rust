Feature: ha_all_dcs_services_stopped_on_three_etcd_enters_safe_degraded_mode_and_fences_post_cutoff_writes
  Scenario: losing every etcd member removes authority, exposes fail-safe state, and fences post-cutoff writes
    Given the "three_node_three_etcd" harness is running
    And I wait for exactly one stable primary as "initial_primary"
    And I create one workload table for this feature
    When I start a bounded concurrent write workload and record commit outcomes
    And I stop the DCS service
    Then there is no operator-visible primary across 3 online nodes
    And every running node reports fail_safe in debug output
    And there is no dual-primary evidence during the transition window
    When I start the DCS service
    Then I wait for exactly one stable primary as "restored_primary"
    When I stop the workload and verify it committed at least one row
    Then the recorded workload evidence establishes a fencing cutoff with no later commits
    And the 3 online nodes contain exactly the recorded proof rows
