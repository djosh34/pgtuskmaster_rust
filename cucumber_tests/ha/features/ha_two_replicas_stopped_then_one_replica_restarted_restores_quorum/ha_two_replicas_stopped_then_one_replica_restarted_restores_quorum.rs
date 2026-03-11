#[path = "../../support/mod.rs"]
mod support;

#[test]
fn ha_two_replicas_stopped_then_one_replica_restarted_restores_quorum() -> Result<(), String> {
    support::runner::run_feature_test(
        "ha_two_replicas_stopped_then_one_replica_restarted_restores_quorum",
        "cucumber_tests/ha/features/ha_two_replicas_stopped_then_one_replica_restarted_restores_quorum/ha_two_replicas_stopped_then_one_replica_restarted_restores_quorum.feature",
    )
}
