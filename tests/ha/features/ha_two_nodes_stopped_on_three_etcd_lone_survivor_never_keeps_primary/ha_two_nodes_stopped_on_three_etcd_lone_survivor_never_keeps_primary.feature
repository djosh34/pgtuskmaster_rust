Feature: ha_two_nodes_stopped_on_three_etcd_lone_survivor_never_keeps_primary
  Scenario: a lone survivor with only its local etcd loses authority instead of remaining writable
    Given the "three_node_three_etcd" harness is running
    And I wait for exactly one stable primary as "initial_primary"
    And I choose the two non-primary nodes as "stopped_replica_a" and "stopped_replica_b"
    When I stop the local DCS service for node named "stopped_replica_a"
    And I stop the local DCS service for node named "stopped_replica_b"
    And I kill the nodes named "stopped_replica_a" and "stopped_replica_b"
    Then there is no operator-visible primary across 1 online node
    And every running node reports fail_safe in debug output
