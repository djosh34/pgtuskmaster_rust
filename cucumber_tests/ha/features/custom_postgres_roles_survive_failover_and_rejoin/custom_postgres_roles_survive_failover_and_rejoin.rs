#[path = "../../support/mod.rs"]
mod support;

#[test]
fn custom_postgres_roles_survive_failover_and_rejoin() -> Result<(), String> {
    support::runner::run_feature_test(
        "custom_postgres_roles_survive_failover_and_rejoin",
        "cucumber_tests/ha/features/custom_postgres_roles_survive_failover_and_rejoin/custom_postgres_roles_survive_failover_and_rejoin.feature",
    )
}
