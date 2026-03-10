#[path = "../../support/mod.rs"]
mod support;

#[test]
fn primary_crash_rejoin() -> Result<(), String> {
    support::runner::run_feature_test(
        "primary_crash_rejoin",
        "cucumber_tests/ha/features/primary_crash_rejoin/primary_crash_rejoin.feature",
    )
}
