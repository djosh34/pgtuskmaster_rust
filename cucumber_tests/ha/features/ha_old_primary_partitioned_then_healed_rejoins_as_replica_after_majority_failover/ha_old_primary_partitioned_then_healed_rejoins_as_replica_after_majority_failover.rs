#[path = "../../support/mod.rs"]
mod support;

#[test]
fn ha_old_primary_partitioned_then_healed_rejoins_as_replica_after_majority_failover() -> Result<(), String> {
    support::runner::run_feature_test(
        "ha_old_primary_partitioned_then_healed_rejoins_as_replica_after_majority_failover",
        "cucumber_tests/ha/features/ha_old_primary_partitioned_then_healed_rejoins_as_replica_after_majority_failover/ha_old_primary_partitioned_then_healed_rejoins_as_replica_after_majority_failover.feature",
    )
}
