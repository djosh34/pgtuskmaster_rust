#[path = "../../support/mod.rs"]
mod support;

#[test]
fn ha_targeted_switchover_to_degraded_replica_is_rejected() -> Result<(), String> {
    support::runner::run_feature_test(
        "ha_targeted_switchover_to_degraded_replica_is_rejected",
        "cucumber_tests/ha/features/ha_targeted_switchover_to_degraded_replica_is_rejected/ha_targeted_switchover_to_degraded_replica_is_rejected.feature",
    )
}
