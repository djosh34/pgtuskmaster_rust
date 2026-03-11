#[path = "../../support/mod.rs"]
mod support;

#[test]
fn repeated_leadership_changes_preserve_single_primary() -> Result<(), String> {
    support::runner::run_feature_test(
        "repeated_leadership_changes_preserve_single_primary",
        "cucumber_tests/ha/features/repeated_leadership_changes_preserve_single_primary/repeated_leadership_changes_preserve_single_primary.feature",
    )
}
