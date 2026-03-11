#[path = "../../support/mod.rs"]
mod support;

#[test]
fn replica_flap_keeps_primary_stable() -> Result<(), String> {
    support::runner::run_feature_test(
        "replica_flap_keeps_primary_stable",
        "cucumber_tests/ha/features/replica_flap_keeps_primary_stable/replica_flap_keeps_primary_stable.feature",
    )
}
