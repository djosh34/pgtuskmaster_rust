#[path = "../../support/mod.rs"]
mod support;

#[test]
fn ha_non_primary_api_isolated_primary_stays_primary() -> Result<(), String> {
    support::runner::run_feature_test(
        "ha_non_primary_api_isolated_primary_stays_primary",
        "cucumber_tests/ha/features/ha_non_primary_api_isolated_primary_stays_primary/ha_non_primary_api_isolated_primary_stays_primary.feature",
    )
}
