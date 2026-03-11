#[path = "../../support/mod.rs"]
mod support;

#[test]
fn ha_replica_flapped_primary_stays_primary() -> Result<(), String> {
    support::runner::run_feature_test(
        "ha_replica_flapped_primary_stays_primary",
        "cucumber_tests/ha/features/ha_replica_flapped_primary_stays_primary/ha_replica_flapped_primary_stays_primary.feature",
    )
}
