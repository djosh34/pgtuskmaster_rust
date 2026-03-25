# Merged from: ha_dcs_quorum_lost_enters_failsafe,
# ha_all_dcs_services_stopped_on_three_etcd_enters_safe_degraded_mode_and_fences_post_cutoff_writes,
# ha_dcs_quorum_lost_fencing_blocks_post_cutoff_writes,
# ha_primary_loses_local_etcd_on_three_etcd_loses_authority_until_local_dcs_recovers,
# ha_two_nodes_stopped_on_three_etcd_lone_survivor_never_keeps_primary.
# Deleted during the merge because workload and fencing assertions moved out of feature text.
# Deleted instead of forcing a misleading reduced outcome:
# ha_primary_loses_local_etcd_on_three_etcd_loses_authority_until_local_dcs_recovers.
Feature: ha_quorum_loss_and_dcs_loss
  Scenario: losing a DCS quorum majority makes the cluster unhealthy until quorum returns
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "initial_primary"
    When I stop a DCS quorum majority
    Then cluster becomes unhealthy
    When I restore DCS quorum
    Then cluster becomes healthy
  Scenario: losing every DCS service makes the cluster unhealthy until DCS returns
    Given the "three_node_three_etcd" harness is running
    And I wait for exactly one stable primary as "initial_primary"
    When I stop the DCS service
    Then cluster becomes unhealthy
    When I start the DCS service
    Then cluster becomes healthy
  Scenario: a lone survivor with only local DCS stays unhealthy
    Given the "three_node_three_etcd" harness is running
    And I wait for exactly one stable primary as "initial_primary"
    And I choose the two non-primary nodes as "stopped_replica_a" and "stopped_replica_b"
    When I stop the local DCS service for node named "stopped_replica_a"
    And I stop the local DCS service for node named "stopped_replica_b"
    And I kill the nodes named "stopped_replica_a" and "stopped_replica_b"
    Then cluster becomes unhealthy
