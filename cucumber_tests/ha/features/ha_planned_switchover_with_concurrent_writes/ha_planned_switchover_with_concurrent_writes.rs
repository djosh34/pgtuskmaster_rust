#[path = "../../support/mod.rs"]
mod support;

#[test]
fn ha_planned_switchover_with_concurrent_writes() -> Result<(), String> {
    support::runner::run_feature_test(
        "ha_planned_switchover_with_concurrent_writes",
        "cucumber_tests/ha/features/ha_planned_switchover_with_concurrent_writes/ha_planned_switchover_with_concurrent_writes.feature",
    )
}
