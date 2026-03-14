Feature: ha_dcs_quorum_lost_enters_failsafe
  Scenario: losing DCS quorum removes the operator-visible primary and exposes fail-safe behavior
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "initial_primary"
    When I stop a DCS quorum majority
    Then there is no operator-visible primary across running nodes
    And every running node reports fail_safe in debug output
    And there is no dual-primary evidence during the transition window
    When I restore DCS quorum
    Then I wait for exactly one stable primary as "restored_primary"
