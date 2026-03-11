#[path = "../../support/mod.rs"]
mod support;

#[test]
fn targeted_switchover_rejects_ineligible_member() -> Result<(), String> {
    support::runner::run_feature_test(
        "targeted_switchover_rejects_ineligible_member",
        "cucumber_tests/ha/features/targeted_switchover_rejects_ineligible_member/targeted_switchover_rejects_ineligible_member.feature",
    )
}
