#[path = "../../support/mod.rs"]
mod support;

#[test]
fn broken_replica_rejoin_does_not_block_healthy_quorum() -> Result<(), String> {
    support::runner::run_feature_test(
        "broken_replica_rejoin_does_not_block_healthy_quorum",
        "cucumber_tests/ha/features/broken_replica_rejoin_does_not_block_healthy_quorum/broken_replica_rejoin_does_not_block_healthy_quorum.feature",
    )
}
