Feature: mixed network faults heal converges
  Scenario: combined dcs and api faults still converge safely after heal
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "initial_primary"
    And I choose one non-primary node as "api_isolated_node"
    And I create a proof table for this feature
    And I insert proof row "1:before-mixed-faults" through "initial_primary"
    When I cut the node named "initial_primary" off from DCS
    And I isolate the node named "api_isolated_node" from observer API access
    Then the node named "initial_primary" enters fail-safe or loses primary authority safely
    And there is no dual-primary evidence during the transition window
    When I heal all network faults
    Then exactly one primary exists across 3 running nodes as "final_primary"
    When I insert proof row "2:after-mixed-fault-heal" through "final_primary"
    Then the 3 online nodes contain exactly the recorded proof rows
