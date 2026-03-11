Feature: planned switchover changes primary cleanly
  Scenario: a planned switchover moves leadership to a different primary
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "old_primary"
    And I create a proof table for this feature
    And I insert proof row "1:before-planned-switchover" through "old_primary"
    Then the 3 online nodes contain exactly the recorded proof rows
    And I record the current pgtm primary and replicas views
    When I request a planned switchover
    Then I wait for a different stable primary than "old_primary" as "new_primary"
    And the node named "old_primary" remains online as a replica
    And pgtm primary points to "new_primary"
    And pgtm replicas list every cluster member except "new_primary"
    When I insert proof row "2:after-planned-switchover" through "new_primary"
    Then the 3 online nodes contain exactly the recorded proof rows
