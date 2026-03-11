#[path = "../../support/mod.rs"]
mod support;

#[test]
fn ha_primary_killed_custom_roles_survive_rejoin() -> Result<(), String> {
    support::runner::run_feature_test(
        "ha_primary_killed_custom_roles_survive_rejoin",
        "cucumber_tests/ha/features/ha_primary_killed_custom_roles_survive_rejoin/ha_primary_killed_custom_roles_survive_rejoin.feature",
    )
}
