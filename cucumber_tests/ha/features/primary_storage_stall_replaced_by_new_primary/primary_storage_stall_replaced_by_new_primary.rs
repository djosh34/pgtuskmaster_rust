#[path = "../../support/mod.rs"]
mod support;

#[test]
fn primary_storage_stall_replaced_by_new_primary() -> Result<(), String> {
    support::runner::run_feature_test(
        "primary_storage_stall_replaced_by_new_primary",
        "cucumber_tests/ha/features/primary_storage_stall_replaced_by_new_primary/primary_storage_stall_replaced_by_new_primary.feature",
    )
}
