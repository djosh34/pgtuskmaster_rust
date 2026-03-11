#[path = "../../support/mod.rs"]
mod support;

#[test]
fn targeted_switchover_promotes_requested_replica() -> Result<(), String> {
    support::runner::run_feature_test(
        "targeted_switchover_promotes_requested_replica",
        "cucumber_tests/ha/features/targeted_switchover_promotes_requested_replica/targeted_switchover_promotes_requested_replica.feature",
    )
}
