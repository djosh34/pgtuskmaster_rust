#[path = "../../support/mod.rs"]
mod support;

#[test]
fn rewind_failure_falls_back_to_basebackup() -> Result<(), String> {
    support::runner::run_feature_test(
        "rewind_failure_falls_back_to_basebackup",
        "cucumber_tests/ha/features/rewind_failure_falls_back_to_basebackup/rewind_failure_falls_back_to_basebackup.feature",
    )
}
