#[path = "../../support/mod.rs"]
mod support;

#[test]
fn replica_outage_keeps_primary_stable() -> Result<(), String> {
    support::runner::run_feature_test(
        "replica_outage_keeps_primary_stable",
        "cucumber_tests/ha/features/replica_outage_keeps_primary_stable/replica_outage_keeps_primary_stable.feature",
    )
}
