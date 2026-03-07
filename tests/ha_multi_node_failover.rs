#[path = "ha/support/multi_node.rs"]
mod multi_node;
#[path = "ha/support/observer.rs"]
mod observer;

#[tokio::test(flavor = "current_thread")]
async fn e2e_multi_node_unassisted_failover_sql_consistency(
) -> Result<(), pgtuskmaster_rust::state::WorkerError> {
    multi_node::e2e_multi_node_unassisted_failover_sql_consistency().await
}

#[tokio::test(flavor = "current_thread")]
async fn e2e_multi_node_stress_planned_switchover_concurrent_sql(
) -> Result<(), pgtuskmaster_rust::state::WorkerError> {
    multi_node::e2e_multi_node_stress_planned_switchover_concurrent_sql().await
}

#[tokio::test(flavor = "current_thread")]
async fn e2e_multi_node_custom_postgres_role_names_survive_bootstrap_and_rewind(
) -> Result<(), pgtuskmaster_rust::state::WorkerError> {
    multi_node::e2e_multi_node_custom_postgres_role_names_survive_bootstrap_and_rewind().await
}

#[tokio::test(flavor = "current_thread")]
async fn e2e_multi_node_stress_unassisted_failover_concurrent_sql(
) -> Result<(), pgtuskmaster_rust::state::WorkerError> {
    multi_node::e2e_multi_node_stress_unassisted_failover_concurrent_sql().await
}

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
