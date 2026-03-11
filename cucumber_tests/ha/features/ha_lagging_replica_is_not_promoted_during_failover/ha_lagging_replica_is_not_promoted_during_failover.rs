#[path = "../../support/mod.rs"]
mod support;

#[test]
fn ha_lagging_replica_is_not_promoted_during_failover() -> Result<(), String> {
    support::runner::run_feature_test(
        "ha_lagging_replica_is_not_promoted_during_failover",
        "cucumber_tests/ha/features/ha_lagging_replica_is_not_promoted_during_failover/ha_lagging_replica_is_not_promoted_during_failover.feature",
    )
}
