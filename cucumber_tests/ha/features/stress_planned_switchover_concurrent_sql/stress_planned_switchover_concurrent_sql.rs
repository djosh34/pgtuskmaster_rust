#[path = "../../support/mod.rs"]
mod support;

#[test]
fn stress_planned_switchover_concurrent_sql() -> Result<(), String> {
    support::runner::run_feature_test(
        "stress_planned_switchover_concurrent_sql",
        "cucumber_tests/ha/features/stress_planned_switchover_concurrent_sql/stress_planned_switchover_concurrent_sql.feature",
    )
}
