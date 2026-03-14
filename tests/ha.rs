#[path = "ha/support/mod.rs"]
mod support;

macro_rules! ha_feature_test {
    ($name:ident) => {
        #[test]
        fn $name() -> Result<(), String> {
            crate::support::runner::run_feature_test(
                stringify!($name),
                concat!(
                    "tests/ha/features/",
                    stringify!($name),
                    "/",
                    stringify!($name),
                    ".feature"
                ),
            )
        }
    };
}

ha_feature_test!(ha_primary_killed_then_rejoins_as_replica);
ha_feature_test!(ha_replica_stopped_primary_stays_primary);
ha_feature_test!(ha_two_replicas_stopped_then_one_replica_restarted_restores_quorum);
ha_feature_test!(ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins);
ha_feature_test!(ha_replica_flapped_primary_stays_primary);
ha_feature_test!(ha_planned_switchover_changes_primary_cleanly);
ha_feature_test!(ha_targeted_switchover_promotes_requested_replica);
ha_feature_test!(ha_planned_switchover_with_concurrent_writes);
ha_feature_test!(ha_primary_killed_with_concurrent_writes);
ha_feature_test!(ha_targeted_switchover_to_degraded_replica_is_rejected);
ha_feature_test!(ha_primary_killed_custom_roles_survive_rejoin);
ha_feature_test!(ha_basebackup_clone_blocked_then_unblocked_replica_recovers);
ha_feature_test!(ha_rewind_fails_then_basebackup_rejoins_old_primary);
ha_feature_test!(ha_repeated_failovers_preserve_single_primary);
ha_feature_test!(ha_lagging_replica_is_not_promoted_during_failover);
ha_feature_test!(ha_dcs_quorum_lost_enters_failsafe);
ha_feature_test!(ha_dcs_quorum_lost_fencing_blocks_post_cutoff_writes);
ha_feature_test!(ha_old_primary_partitioned_from_majority_majority_elects_new_primary);
ha_feature_test!(ha_replica_partitioned_from_majority_primary_stays_primary);
ha_feature_test!(ha_old_primary_partitioned_then_healed_rejoins_as_replica_after_majority_failover);
ha_feature_test!(ha_non_primary_api_isolated_primary_stays_primary);
ha_feature_test!(ha_replication_path_isolated_then_healed_replicas_catch_up);
ha_feature_test!(ha_dcs_and_api_faults_then_healed_cluster_converges);
ha_feature_test!(ha_primary_storage_stalled_then_new_primary_takes_over);
ha_feature_test!(
    ha_two_nodes_stopped_then_one_healthy_node_restarted_restores_service_while_other_stays_broken
);
ha_feature_test!(ha_broken_replica_rejoin_attempt_does_not_destabilize_quorum);
ha_feature_test!(ha_two_nodes_stopped_on_three_etcd_lone_survivor_never_keeps_primary);
ha_feature_test!(ha_old_primary_partitioned_from_majority_on_three_etcd_majority_elects_new_primary);
ha_feature_test!(ha_replica_partitioned_from_majority_on_three_etcd_primary_stays_primary);
ha_feature_test!(ha_primary_loses_local_etcd_on_three_etcd_loses_authority_until_local_dcs_recovers);
ha_feature_test!(ha_replica_loses_local_etcd_on_three_etcd_does_not_become_primary_and_primary_stays_primary);
ha_feature_test!(
    ha_all_dcs_services_stopped_on_three_etcd_enters_safe_degraded_mode_and_fences_post_cutoff_writes
);
