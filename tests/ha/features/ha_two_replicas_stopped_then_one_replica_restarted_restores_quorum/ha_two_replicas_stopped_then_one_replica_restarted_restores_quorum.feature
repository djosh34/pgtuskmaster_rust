Feature: ha_two_replicas_stopped_then_one_replica_restarted_restores_quorum
  Scenario: two replicas stop, then one returns and restores quorum
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "initial_primary"
    And I choose the two non-primary nodes as "stopped_node_a" and "stopped_node_b"
    And I create a proof table for this feature
    And I insert proof row "1:before-two-node-outage" through "initial_primary"
    Then the 3 online nodes contain exactly the recorded proof rows
    When I kill the nodes named "stopped_node_a" and "stopped_node_b"
    Then there is no operator-visible primary across 1 online node
    And the lone online node is not treated as a writable primary
    When I restart the node named "stopped_node_a"
    Then exactly one primary exists across 2 running nodes as "restored_primary"
    When I insert proof row "2:after-quorum-restore-before-full-heal" through "restored_primary"
    Then the cluster is degraded but operational across 2 running nodes
    When I restart the node named "stopped_node_b"
    Then the node named "stopped_node_b" rejoins as a replica
    And the 3 online nodes contain exactly the recorded proof rows
