#[path = "../../support/mod.rs"]
mod support;

#[test]
fn two_node_outage_one_return_restores_quorum() -> Result<(), String> {
    support::runner::run_feature_test(
        "two_node_outage_one_return_restores_quorum",
        "cucumber_tests/ha/features/two_node_outage_one_return_restores_quorum/two_node_outage_one_return_restores_quorum.feature",
    )
}
