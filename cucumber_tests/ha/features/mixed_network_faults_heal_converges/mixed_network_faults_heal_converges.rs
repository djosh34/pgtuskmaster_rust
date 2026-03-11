#[path = "../../support/mod.rs"]
mod support;

#[test]
fn mixed_network_faults_heal_converges() -> Result<(), String> {
    support::runner::run_feature_test(
        "mixed_network_faults_heal_converges",
        "cucumber_tests/ha/features/mixed_network_faults_heal_converges/mixed_network_faults_heal_converges.feature",
    )
}
