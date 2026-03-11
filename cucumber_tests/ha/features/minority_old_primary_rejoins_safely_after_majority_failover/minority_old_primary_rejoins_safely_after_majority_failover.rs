#[path = "../../support/mod.rs"]
mod support;

#[test]
fn minority_old_primary_rejoins_safely_after_majority_failover() -> Result<(), String> {
    support::runner::run_feature_test(
        "minority_old_primary_rejoins_safely_after_majority_failover",
        "cucumber_tests/ha/features/minority_old_primary_rejoins_safely_after_majority_failover/minority_old_primary_rejoins_safely_after_majority_failover.feature",
    )
}
