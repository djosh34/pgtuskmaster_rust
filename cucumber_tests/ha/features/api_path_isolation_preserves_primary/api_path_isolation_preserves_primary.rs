#[path = "../../support/mod.rs"]
mod support;

#[test]
fn api_path_isolation_preserves_primary() -> Result<(), String> {
    support::runner::run_feature_test(
        "api_path_isolation_preserves_primary",
        "cucumber_tests/ha/features/api_path_isolation_preserves_primary/api_path_isolation_preserves_primary.feature",
    )
}
