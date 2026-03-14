Feature: ha_two_nodes_stopped_then_one_healthy_node_restarted_restores_service_while_other_stays_broken
  Scenario: one healthy return restores service even while another node stays broken
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "initial_primary"
    And I choose the two non-primary nodes as "stopped_node_a" and "stopped_node_b"
    And I create a proof table for this feature
    And I insert proof row "1:before-two-node-loss" through "initial_primary"
    When I kill the nodes named "stopped_node_a" and "stopped_node_b"
    Then the primary named "initial_primary" remains the only primary
    When I restart the node named "stopped_node_a"
    And I enable the "startup" blocker on the node named "stopped_node_b"
    And I start the node named "stopped_node_b" but keep it marked unavailable
    Then exactly one primary exists across 2 running nodes as "restored_primary"
    When I insert proof row "2:after-good-return-before-broken-return-fix" through "restored_primary"
    Then the cluster is degraded but operational across 2 running nodes
    When I disable the "startup" blocker on the node named "stopped_node_b"
    And I restart the node named "stopped_node_b"
    Then the 3 online nodes contain exactly the recorded proof rows
