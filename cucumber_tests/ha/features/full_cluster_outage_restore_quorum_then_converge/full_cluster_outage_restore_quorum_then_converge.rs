#[path = "../../support/mod.rs"]
mod support;

#[test]
fn full_cluster_outage_restore_quorum_then_converge() -> Result<(), String> {
    support::runner::run_feature_test(
        "full_cluster_outage_restore_quorum_then_converge",
        "cucumber_tests/ha/features/full_cluster_outage_restore_quorum_then_converge/full_cluster_outage_restore_quorum_then_converge.feature",
    )
}
