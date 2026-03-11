#[path = "../../support/mod.rs"]
mod support;

#[test]
fn planned_switchover_changes_primary_cleanly() -> Result<(), String> {
    support::runner::run_feature_test(
        "planned_switchover_changes_primary_cleanly",
        "cucumber_tests/ha/features/planned_switchover_changes_primary_cleanly/planned_switchover_changes_primary_cleanly.feature",
    )
}
