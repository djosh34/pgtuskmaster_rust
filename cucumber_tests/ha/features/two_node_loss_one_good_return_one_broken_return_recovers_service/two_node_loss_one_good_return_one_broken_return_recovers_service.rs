#[path = "../../support/mod.rs"]
mod support;

#[test]
fn two_node_loss_one_good_return_one_broken_return_recovers_service() -> Result<(), String> {
    support::runner::run_feature_test(
        "two_node_loss_one_good_return_one_broken_return_recovers_service",
        "cucumber_tests/ha/features/two_node_loss_one_good_return_one_broken_return_recovers_service/two_node_loss_one_good_return_one_broken_return_recovers_service.feature",
    )
}
