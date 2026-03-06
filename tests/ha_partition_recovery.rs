#[path = "ha/support/observer.rs"]
mod observer;
#[path = "ha/support/partition.rs"]
mod partition;

#[tokio::test(flavor = "current_thread")]
async fn e2e_partition_mixed_faults_heal_converges(
) -> Result<(), pgtuskmaster_rust::state::WorkerError> {
    partition::e2e_partition_mixed_faults_heal_converges().await
}
