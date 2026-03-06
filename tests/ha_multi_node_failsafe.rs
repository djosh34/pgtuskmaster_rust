#[path = "ha/support/observer.rs"]
mod observer;
#[path = "ha/support/multi_node.rs"]
mod multi_node;

#[tokio::test(flavor = "current_thread")]
async fn e2e_no_quorum_enters_failsafe_strict_all_nodes(
) -> Result<(), pgtuskmaster_rust::state::WorkerError> {
    multi_node::e2e_no_quorum_enters_failsafe_strict_all_nodes().await
}

#[tokio::test(flavor = "current_thread")]
async fn e2e_no_quorum_fencing_blocks_post_cutoff_commits_and_preserves_integrity(
) -> Result<(), pgtuskmaster_rust::state::WorkerError> {
    multi_node::e2e_no_quorum_fencing_blocks_post_cutoff_commits_and_preserves_integrity().await
}
