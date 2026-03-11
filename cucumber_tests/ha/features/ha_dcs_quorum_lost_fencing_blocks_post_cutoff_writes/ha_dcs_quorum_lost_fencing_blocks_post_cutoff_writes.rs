#[path = "../../support/mod.rs"]
mod support;

#[test]
fn ha_dcs_quorum_lost_fencing_blocks_post_cutoff_writes() -> Result<(), String> {
    support::runner::run_feature_test(
        "ha_dcs_quorum_lost_fencing_blocks_post_cutoff_writes",
        "cucumber_tests/ha/features/ha_dcs_quorum_lost_fencing_blocks_post_cutoff_writes/ha_dcs_quorum_lost_fencing_blocks_post_cutoff_writes.feature",
    )
}
