Feature: ha_non_primary_api_isolated_primary_stays_primary
  Scenario: observer api isolation of a non-primary does not trigger a failover
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "initial_primary"
    And I choose one non-primary node as "api_isolated_node"
    When I isolate the node named "api_isolated_node" from observer API access
    Then direct API observation to "api_isolated_node" fails
    And the primary named "initial_primary" remains the only primary
    When I heal network faults on the node named "api_isolated_node"
    And I insert proof row "1:after-api-path-heal" through "initial_primary"
    Then the 3 online nodes contain exactly the recorded proof rows
