#[path = "../../support/mod.rs"]
mod support;

#[test]
fn ha_two_nodes_stopped_then_one_healthy_node_restarted_restores_service_while_other_stays_broken(
) -> Result<(), String> {
    support::runner::run_feature_test(
        "ha_two_nodes_stopped_then_one_healthy_node_restarted_restores_service_while_other_stays_broken",
        "cucumber_tests/ha/features/ha_two_nodes_stopped_then_one_healthy_node_restarted_restores_service_while_other_stays_broken/ha_two_nodes_stopped_then_one_healthy_node_restarted_restores_service_while_other_stays_broken.feature",
    )
}
