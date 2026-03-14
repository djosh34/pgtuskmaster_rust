Feature: ha_replica_loses_local_etcd_on_three_etcd_does_not_become_primary_and_primary_stays_primary
  Scenario: a replica that loses its local etcd does not self-promote while the current primary stays authoritative
    Given the "three_node_three_etcd" harness is running
    And I wait for exactly one stable primary as "stable_primary"
    And I choose one non-primary node as "isolated_replica"
    And I create a proof table for this feature
    And I insert proof row "1:before-replica-local-etcd-loss" through "stable_primary"
    When I start tracking primary history
    And I stop the local DCS service for node named "isolated_replica"
    Then the primary named "stable_primary" remains the only primary
    And the primary history never included "isolated_replica"
    When I insert proof row "2:on-primary-after-replica-local-etcd-loss" through "stable_primary"
    When I start the local DCS service for node named "isolated_replica"
    Then the node named "isolated_replica" rejoins as a replica
    And the 3 online nodes contain exactly the recorded proof rows
