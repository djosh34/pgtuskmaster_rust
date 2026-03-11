#[path = "../../support/mod.rs"]
mod support;

#[test]
fn ha_basebackup_clone_blocked_then_unblocked_replica_recovers() -> Result<(), String> {
    support::runner::run_feature_test(
        "ha_basebackup_clone_blocked_then_unblocked_replica_recovers",
        "cucumber_tests/ha/features/ha_basebackup_clone_blocked_then_unblocked_replica_recovers/ha_basebackup_clone_blocked_then_unblocked_replica_recovers.feature",
    )
}
