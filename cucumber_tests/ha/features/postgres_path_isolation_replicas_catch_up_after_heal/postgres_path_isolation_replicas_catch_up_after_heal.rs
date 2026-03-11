#[path = "../../support/mod.rs"]
mod support;

#[test]
fn postgres_path_isolation_replicas_catch_up_after_heal() -> Result<(), String> {
    support::runner::run_feature_test(
        "postgres_path_isolation_replicas_catch_up_after_heal",
        "cucumber_tests/ha/features/postgres_path_isolation_replicas_catch_up_after_heal/postgres_path_isolation_replicas_catch_up_after_heal.feature",
    )
}
