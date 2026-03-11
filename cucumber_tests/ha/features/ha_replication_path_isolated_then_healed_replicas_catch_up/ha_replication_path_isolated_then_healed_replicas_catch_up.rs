#[path = "../../support/mod.rs"]
mod support;

#[test]
fn ha_replication_path_isolated_then_healed_replicas_catch_up() -> Result<(), String> {
    support::runner::run_feature_test(
        "ha_replication_path_isolated_then_healed_replicas_catch_up",
        "cucumber_tests/ha/features/ha_replication_path_isolated_then_healed_replicas_catch_up/ha_replication_path_isolated_then_healed_replicas_catch_up.feature",
    )
}
