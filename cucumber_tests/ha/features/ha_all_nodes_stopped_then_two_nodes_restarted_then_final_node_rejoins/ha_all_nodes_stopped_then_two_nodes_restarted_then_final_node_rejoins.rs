#[path = "../../support/mod.rs"]
mod support;

#[test]
fn ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins() -> Result<(), String> {
    support::runner::run_feature_test(
        "ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins",
        "cucumber_tests/ha/features/ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins/ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins.feature",
    )
}
