Feature: ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins
  Scenario: the cluster comes back with two fixed nodes first, then converges after the final node returns
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "initial_primary"
    And I create a proof table for this feature
    And I insert proof row "1:before-full-cluster-outage" through "initial_primary"
    Then the 3 online nodes contain exactly the recorded proof rows
    When I kill all database nodes
    And I start only the fixed nodes "node-a" and "node-b"
    Then exactly one primary exists across 2 running nodes as "restored_primary"
    When I insert proof row "2:after-two-node-restore-before-final-node" through "restored_primary"
    Then the node named "node-c" remains offline
    When I restart the node named "node-c"
    Then the node named "node-c" rejoins as a replica
    And pgtm primary points to "restored_primary"
    And the 3 online nodes contain exactly the recorded proof rows
