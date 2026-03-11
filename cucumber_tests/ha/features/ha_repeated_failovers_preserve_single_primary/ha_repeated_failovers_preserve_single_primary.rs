#[path = "../../support/mod.rs"]
mod support;

#[test]
fn ha_repeated_failovers_preserve_single_primary() -> Result<(), String> {
    support::runner::run_feature_test(
        "ha_repeated_failovers_preserve_single_primary",
        "cucumber_tests/ha/features/ha_repeated_failovers_preserve_single_primary/ha_repeated_failovers_preserve_single_primary.feature",
    )
}
