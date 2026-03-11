#[path = "../../support/mod.rs"]
mod support;

#[test]
fn no_quorum_enters_failsafe() -> Result<(), String> {
    support::runner::run_feature_test(
        "no_quorum_enters_failsafe",
        "cucumber_tests/ha/features/no_quorum_enters_failsafe/no_quorum_enters_failsafe.feature",
    )
}
