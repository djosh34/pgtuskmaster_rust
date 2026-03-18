#[path = "ha/support/mod.rs"]
mod support;

macro_rules! ha_scenario_test {
    ($feature:ident, $name:ident, $scenario_name:literal) => {
        #[test]
        fn $name() -> Result<(), String> {
            crate::support::runner::run_feature_test(
                stringify!($feature),
                concat!(
                    "tests/ha/features/",
                    stringify!($feature),
                    "/",
                    stringify!($feature),
                    ".feature"
                ),
                Some($scenario_name),
            )
        }
    };
}

mod ha_replica_faults_keep_cluster_healthy {
    ha_scenario_test!(
        ha_replica_faults_keep_cluster_healthy,
        stopped_replica,
        "a stopped replica does not prevent the cluster from staying healthy"
    );
    ha_scenario_test!(
        ha_replica_faults_keep_cluster_healthy,
        isolated_replica_majority_healthy,
        "isolating a replica from the cluster still leaves a healthy majority"
    );
    ha_scenario_test!(
        ha_replica_faults_keep_cluster_healthy,
        api_isolated_replica,
        "isolating a replica from observer API access does not make the cluster unhealthy"
    );
    ha_scenario_test!(
        ha_replica_faults_keep_cluster_healthy,
        replica_local_dcs_loss,
        "a replica that loses its local DCS still leaves the cluster healthy"
    );
}

mod ha_primary_faults_fail_over_then_recover {
    ha_scenario_test!(
        ha_primary_faults_fail_over_then_recover,
        killed_primary_fails_over_and_recovers,
        "a killed primary fails over and the cluster returns to health after restart"
    );
    ha_scenario_test!(
        ha_primary_faults_fail_over_then_recover,
        primary_isolated_from_majority,
        "a primary isolated from the majority still lets the cluster recover"
    );
}

mod ha_quorum_loss_and_dcs_loss {
    ha_scenario_test!(
        ha_quorum_loss_and_dcs_loss,
        dcs_quorum_majority_loss,
        "losing a DCS quorum majority makes the cluster unhealthy until quorum returns"
    );
    ha_scenario_test!(
        ha_quorum_loss_and_dcs_loss,
        all_dcs_services_lost,
        "losing every DCS service makes the cluster unhealthy until DCS returns"
    );
    ha_scenario_test!(
        ha_quorum_loss_and_dcs_loss,
        lone_survivor_with_only_local_dcs,
        "a lone survivor with only local DCS stays unhealthy"
    );
}

mod ha_rejoin_and_restart_recovery {
    ha_scenario_test!(
        ha_rejoin_and_restart_recovery,
        restarting_replicas_restores_health,
        "restarting replicas restores health after a two-replica outage"
    );
    ha_scenario_test!(
        ha_rejoin_and_restart_recovery,
        full_database_outage_stays_unhealthy_until_nodes_return,
        "a full database outage stays unhealthy until nodes return"
    );
    ha_scenario_test!(
        ha_rejoin_and_restart_recovery,
        blocked_basebackup_recovery_recovers_after_unblock,
        "a blocked basebackup recovery keeps the cluster healthy and recovers after unblock"
    );
    ha_scenario_test!(
        ha_rejoin_and_restart_recovery,
        rewind_failure_old_primary_rejoins,
        "a rewind failure still lets the old primary rejoin without losing cluster health"
    );
}

mod ha_operator_switchovers {
    ha_scenario_test!(
        ha_operator_switchovers,
        planned_switchover_keeps_cluster_healthy,
        "a planned switchover keeps the cluster healthy"
    );
    ha_scenario_test!(
        ha_operator_switchovers,
        targeted_switchover_promotes_requested_replica,
        "a targeted switchover promotes the requested replica"
    );
    ha_scenario_test!(
        ha_operator_switchovers,
        targeted_switchover_to_degraded_replica_rejected,
        "a targeted switchover to a degraded replica is rejected while the cluster stays healthy"
    );
}
