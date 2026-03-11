#[path = "../../support/mod.rs"]
mod support;

#[test]
fn clone_failure_recovers_after_blocker_removed() -> Result<(), String> {
    support::runner::run_feature_test(
        "clone_failure_recovers_after_blocker_removed",
        "cucumber_tests/ha/features/clone_failure_recovers_after_blocker_removed/clone_failure_recovers_after_blocker_removed.feature",
    )
}
