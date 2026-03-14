Feature: ha_primary_loses_local_etcd_on_three_etcd_loses_authority_until_local_dcs_recovers
  Scenario: a primary that loses its local etcd drops authority and stays degraded until local dcs returns
    Given the "three_node_three_etcd" harness is running
    And I wait for exactly one stable primary as "old_primary"
    And I create a proof table for this feature
    And I insert proof row "1:before-primary-local-etcd-loss" through "old_primary"
    When I start tracking primary history
    And I stop the local DCS service for node named "old_primary"
    Then the node named "old_primary" enters fail-safe or loses primary authority safely
    And there is no dual-primary evidence during the transition window
    When I start the local DCS service for node named "old_primary"
    Then I wait for exactly one stable primary as "restored_primary"
    When I insert proof row "2:after-primary-local-etcd-recovery" through "restored_primary"
    Then the 3 online nodes contain exactly the recorded proof rows
