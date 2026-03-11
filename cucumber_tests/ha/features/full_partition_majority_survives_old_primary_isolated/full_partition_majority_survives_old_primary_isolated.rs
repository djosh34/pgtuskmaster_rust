#[path = "../../support/mod.rs"]
mod support;

#[test]
fn full_partition_majority_survives_old_primary_isolated() -> Result<(), String> {
    support::runner::run_feature_test(
        "full_partition_majority_survives_old_primary_isolated",
        "cucumber_tests/ha/features/full_partition_majority_survives_old_primary_isolated/full_partition_majority_survives_old_primary_isolated.feature",
    )
}
