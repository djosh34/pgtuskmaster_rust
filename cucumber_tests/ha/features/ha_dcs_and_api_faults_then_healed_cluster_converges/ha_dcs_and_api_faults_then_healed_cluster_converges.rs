#[path = "../../support/mod.rs"]
mod support;

#[test]
fn ha_dcs_and_api_faults_then_healed_cluster_converges() -> Result<(), String> {
    support::runner::run_feature_test(
        "ha_dcs_and_api_faults_then_healed_cluster_converges",
        "cucumber_tests/ha/features/ha_dcs_and_api_faults_then_healed_cluster_converges/ha_dcs_and_api_faults_then_healed_cluster_converges.feature",
    )
}
