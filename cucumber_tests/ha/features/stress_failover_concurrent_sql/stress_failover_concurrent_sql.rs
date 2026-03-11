#[path = "../../support/mod.rs"]
mod support;

#[test]
fn stress_failover_concurrent_sql() -> Result<(), String> {
    support::runner::run_feature_test(
        "stress_failover_concurrent_sql",
        "cucumber_tests/ha/features/stress_failover_concurrent_sql/stress_failover_concurrent_sql.feature",
    )
}
