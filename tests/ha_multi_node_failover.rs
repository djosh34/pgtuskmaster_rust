#[path = "ha/support/multi_node.rs"]
mod multi_node;
#[path = "ha/support/observer.rs"]
mod observer;

#[tokio::test(flavor = "current_thread")]
async fn e2e_multi_node_unassisted_failover_sql_consistency(
) -> Result<(), pgtuskmaster_rust::state::WorkerError> {
    multi_node::e2e_multi_node_unassisted_failover_sql_consistency().await
}
