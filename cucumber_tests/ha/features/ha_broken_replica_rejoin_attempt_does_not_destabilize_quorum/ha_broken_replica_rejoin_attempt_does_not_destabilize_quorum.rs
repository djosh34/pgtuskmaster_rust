#[path = "../../support/mod.rs"]
mod support;

#[test]
fn ha_broken_replica_rejoin_attempt_does_not_destabilize_quorum() -> Result<(), String> {
    support::runner::run_feature_test(
        "ha_broken_replica_rejoin_attempt_does_not_destabilize_quorum",
        "cucumber_tests/ha/features/ha_broken_replica_rejoin_attempt_does_not_destabilize_quorum/ha_broken_replica_rejoin_attempt_does_not_destabilize_quorum.feature",
    )
}
