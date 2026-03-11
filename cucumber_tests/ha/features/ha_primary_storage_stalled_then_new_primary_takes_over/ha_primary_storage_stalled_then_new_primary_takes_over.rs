#[path = "../../support/mod.rs"]
mod support;

#[test]
fn ha_primary_storage_stalled_then_new_primary_takes_over() -> Result<(), String> {
    support::runner::run_feature_test(
        "ha_primary_storage_stalled_then_new_primary_takes_over",
        "cucumber_tests/ha/features/ha_primary_storage_stalled_then_new_primary_takes_over/ha_primary_storage_stalled_then_new_primary_takes_over.feature",
    )
}
