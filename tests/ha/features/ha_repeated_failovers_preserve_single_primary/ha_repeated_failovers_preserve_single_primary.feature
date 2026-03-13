Feature: ha_repeated_failovers_preserve_single_primary
  Scenario: repeated failovers preserve a single primary across successive topology changes
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "primary_a"
    And I start tracking primary history
    When I kill the node named "primary_a"
    Then exactly one primary exists across 2 running nodes as "primary_b"
    And the primary history never included "primary_a"
    When I restart the node named "primary_a"
    And the node named "primary_a" rejoins as a replica
    And I isolate the node named "primary_a" from all peers on the "postgres" path
    And I start tracking primary history
    When I kill the node named "primary_b"
    Then exactly one primary exists across 2 running nodes as "primary_c"
    And the primary history never included "primary_b"
    And there is no dual-primary evidence during the transition window
