#[path = "../../support/mod.rs"]
mod support;

#[test]
fn ha_planned_switchover_changes_primary_cleanly() -> Result<(), String> {
    support::runner::run_feature_test(
        "ha_planned_switchover_changes_primary_cleanly",
        "cucumber_tests/ha/features/ha_planned_switchover_changes_primary_cleanly/ha_planned_switchover_changes_primary_cleanly.feature",
    )
}
