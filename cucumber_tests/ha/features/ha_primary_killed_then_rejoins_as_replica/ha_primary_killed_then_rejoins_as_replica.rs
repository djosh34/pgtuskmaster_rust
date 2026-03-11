#[path = "../../support/mod.rs"]
mod support;

#[test]
fn ha_primary_killed_then_rejoins_as_replica() -> Result<(), String> {
    support::runner::run_feature_test(
        "ha_primary_killed_then_rejoins_as_replica",
        "cucumber_tests/ha/features/ha_primary_killed_then_rejoins_as_replica/ha_primary_killed_then_rejoins_as_replica.feature",
    )
}
