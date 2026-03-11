#[path = "../../support/mod.rs"]
mod support;

#[test]
fn ha_targeted_switchover_promotes_requested_replica() -> Result<(), String> {
    support::runner::run_feature_test(
        "ha_targeted_switchover_promotes_requested_replica",
        "cucumber_tests/ha/features/ha_targeted_switchover_promotes_requested_replica/ha_targeted_switchover_promotes_requested_replica.feature",
    )
}
