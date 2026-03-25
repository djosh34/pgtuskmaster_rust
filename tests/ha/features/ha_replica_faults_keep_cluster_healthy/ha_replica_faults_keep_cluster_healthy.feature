# Merged from: ha_replica_stopped_primary_stays_primary, ha_replica_flapped_primary_stays_primary,
# ha_replica_partitioned_from_majority_primary_stays_primary,
# ha_replica_partitioned_from_majority_on_three_etcd_primary_stays_primary,
# ha_replica_loses_local_etcd_on_three_etcd_does_not_become_primary_and_primary_stays_primary,
# ha_non_primary_api_isolated_primary_stays_primary.
# Deleted as implementation-biased inventory during the merge: ha_replication_path_isolated_then_healed_replicas_catch_up.
Feature: ha_replica_faults_keep_cluster_healthy
  Scenario: a stopped replica does not prevent the cluster from staying healthy
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "initial_primary"
    And I choose one non-primary node as "stopped_replica"
    When I kill the node named "stopped_replica"
    Then cluster becomes healthy
    When I restart the node named "stopped_replica"
    Then cluster becomes healthy

  Scenario: isolating a replica from the cluster still leaves a healthy majority
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "initial_primary"
    And I choose one non-primary node as "isolated_replica"
    When I fully isolate the node named "isolated_replica" from the cluster
    Then cluster becomes healthy
    When I heal all network faults
    Then cluster becomes healthy

  Scenario: isolating a replica from observer API access does not make the cluster unhealthy
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "initial_primary"
    And I choose one non-primary node as "api_isolated_replica"
    When I isolate the node named "api_isolated_replica" from observer API access
    Then cluster becomes healthy
    When I heal network faults on the node named "api_isolated_replica"
    Then cluster becomes healthy

  Scenario: a replica that loses its local DCS still leaves the cluster healthy
    Given the "three_node_three_etcd" harness is running
    And I wait for exactly one stable primary as "initial_primary"
    And I choose one non-primary node as "isolated_replica"
    When I stop the local DCS service for node named "isolated_replica"
    Then cluster becomes healthy
    When I start the local DCS service for node named "isolated_replica"
    Then cluster becomes healthy
