#[path = "../../support/mod.rs"]
mod support;

#[test]
fn no_quorum_fencing_blocks_post_cutoff_commits() -> Result<(), String> {
    support::runner::run_feature_test(
        "no_quorum_fencing_blocks_post_cutoff_commits",
        "cucumber_tests/ha/features/no_quorum_fencing_blocks_post_cutoff_commits/no_quorum_fencing_blocks_post_cutoff_commits.feature",
    )
}
