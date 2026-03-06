#[path = "ha/support/observer.rs"]
mod observer;
#[path = "ha/support/multi_node.rs"]
mod multi_node;

#[tokio::test(flavor = "current_thread")]
async fn e2e_multi_node_stress_planned_switchover_concurrent_sql(
) -> Result<(), pgtuskmaster_rust::state::WorkerError> {
    multi_node::e2e_multi_node_stress_planned_switchover_concurrent_sql().await
}

#[tokio::test(flavor = "current_thread")]
async fn e2e_multi_node_stress_unassisted_failover_concurrent_sql(
) -> Result<(), pgtuskmaster_rust::state::WorkerError> {
    multi_node::e2e_multi_node_stress_unassisted_failover_concurrent_sql().await
}
