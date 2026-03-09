#[path = "ha/support/observer.rs"]
mod observer;
#[path = "ha/support/partition.rs"]
mod partition;

#[tokio::test(flavor = "current_thread")]
async fn e2e_partition_minority_isolation_no_split_brain_rejoin(
) -> Result<(), pgtuskmaster_rust::state::WorkerError> {
    partition::e2e_partition_minority_isolation_no_split_brain_rejoin().await
}

#[tokio::test(flavor = "current_thread")]
async fn e2e_partition_primary_isolation_failover_no_split_brain(
) -> Result<(), pgtuskmaster_rust::state::WorkerError> {
    partition::e2e_partition_primary_isolation_failover_no_split_brain().await
}

#[tokio::test(flavor = "current_thread")]
async fn e2e_partition_api_path_isolation_preserves_primary(
) -> Result<(), pgtuskmaster_rust::state::WorkerError> {
    partition::e2e_partition_api_path_isolation_preserves_primary().await
}

#[tokio::test(flavor = "current_thread")]
async fn e2e_partition_primary_postgres_path_blocked_replicas_catch_up_after_heal(
) -> Result<(), pgtuskmaster_rust::state::WorkerError> {
    partition::e2e_partition_primary_postgres_path_blocked_replicas_catch_up_after_heal().await
}

#[tokio::test(flavor = "current_thread")]
async fn e2e_partition_mixed_faults_heal_converges(
) -> Result<(), pgtuskmaster_rust::state::WorkerError> {
    partition::e2e_partition_mixed_faults_heal_converges().await
}
