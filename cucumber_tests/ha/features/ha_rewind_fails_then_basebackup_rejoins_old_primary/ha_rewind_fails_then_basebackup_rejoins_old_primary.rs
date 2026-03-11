#[path = "../../support/mod.rs"]
mod support;

#[test]
fn ha_rewind_fails_then_basebackup_rejoins_old_primary() -> Result<(), String> {
    support::runner::run_feature_test(
        "ha_rewind_fails_then_basebackup_rejoins_old_primary",
        "cucumber_tests/ha/features/ha_rewind_fails_then_basebackup_rejoins_old_primary/ha_rewind_fails_then_basebackup_rejoins_old_primary.feature",
    )
}
