#[path = "../../support/mod.rs"]
mod support;

#[test]
fn lagging_replica_is_not_promoted() -> Result<(), String> {
    support::runner::run_feature_test(
        "lagging_replica_is_not_promoted",
        "cucumber_tests/ha/features/lagging_replica_is_not_promoted/lagging_replica_is_not_promoted.feature",
    )
}
