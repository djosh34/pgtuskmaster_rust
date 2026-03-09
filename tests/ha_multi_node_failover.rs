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
async fn e2e_multi_node_clone_failure_recovers_after_fault_removed(
) -> Result<(), pgtuskmaster_rust::state::WorkerError> {
    multi_node::e2e_multi_node_clone_failure_recovers_after_fault_removed().await
}

#[tokio::test(flavor = "current_thread")]
async fn e2e_multi_node_rewind_failure_falls_back_to_basebackup(
) -> Result<(), pgtuskmaster_rust::state::WorkerError> {
    multi_node::e2e_multi_node_rewind_failure_falls_back_to_basebackup().await
}

#[tokio::test(flavor = "current_thread")]
async fn e2e_multi_node_cli_primary_and_replicas_follow_switchover(
) -> Result<(), pgtuskmaster_rust::state::WorkerError> {
    multi_node::e2e_multi_node_cli_primary_and_replicas_follow_switchover().await
}

#[tokio::test(flavor = "current_thread")]
async fn e2e_multi_node_stress_unassisted_failover_concurrent_sql(
) -> Result<(), pgtuskmaster_rust::state::WorkerError> {
    multi_node::e2e_multi_node_stress_unassisted_failover_concurrent_sql().await
}

#[tokio::test(flavor = "current_thread")]
async fn e2e_multi_node_primary_runtime_restart_recovers_without_split_brain(
) -> Result<(), pgtuskmaster_rust::state::WorkerError> {
    multi_node::e2e_multi_node_primary_runtime_restart_recovers_without_split_brain().await
}

#[tokio::test(flavor = "current_thread")]
async fn e2e_multi_node_repeated_leadership_changes_preserve_single_primary(
) -> Result<(), pgtuskmaster_rust::state::WorkerError> {
    multi_node::e2e_multi_node_repeated_leadership_changes_preserve_single_primary().await
}

#[tokio::test(flavor = "current_thread")]
async fn e2e_multi_node_degraded_replica_failover_promotes_only_healthy_target(
) -> Result<(), pgtuskmaster_rust::state::WorkerError> {
    multi_node::e2e_multi_node_degraded_replica_failover_promotes_only_healthy_target().await
}

#[tokio::test(flavor = "current_thread")]
async fn e2e_multi_node_rejects_targeted_switchover_to_ineligible_member(
) -> Result<(), pgtuskmaster_rust::state::WorkerError> {
    multi_node::e2e_multi_node_rejects_targeted_switchover_to_ineligible_member().await
}

#[tokio::test(flavor = "current_thread")]
async fn e2e_multi_node_targeted_switchover_promotes_requested_replica(
) -> Result<(), pgtuskmaster_rust::state::WorkerError> {
    multi_node::e2e_multi_node_targeted_switchover_promotes_requested_replica().await
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
