#[path = "../../support/mod.rs"]
mod support;

#[test]
fn ha_primary_killed_with_concurrent_writes() -> Result<(), String> {
    support::runner::run_feature_test(
        "ha_primary_killed_with_concurrent_writes",
        "cucumber_tests/ha/features/ha_primary_killed_with_concurrent_writes/ha_primary_killed_with_concurrent_writes.feature",
    )
}
