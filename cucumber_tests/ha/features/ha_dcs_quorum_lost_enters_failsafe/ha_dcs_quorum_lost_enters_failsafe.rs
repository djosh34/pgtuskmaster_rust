#[path = "../../support/mod.rs"]
mod support;

#[test]
fn ha_dcs_quorum_lost_enters_failsafe() -> Result<(), String> {
    support::runner::run_feature_test(
        "ha_dcs_quorum_lost_enters_failsafe",
        "cucumber_tests/ha/features/ha_dcs_quorum_lost_enters_failsafe/ha_dcs_quorum_lost_enters_failsafe.feature",
    )
}
