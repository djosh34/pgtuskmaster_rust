#[path = "../../support/mod.rs"]
mod support;

#[test]
fn ha_old_primary_partitioned_from_majority_majority_elects_new_primary() -> Result<(), String> {
    support::runner::run_feature_test(
        "ha_old_primary_partitioned_from_majority_majority_elects_new_primary",
        "cucumber_tests/ha/features/ha_old_primary_partitioned_from_majority_majority_elects_new_primary/ha_old_primary_partitioned_from_majority_majority_elects_new_primary.feature",
    )
}
