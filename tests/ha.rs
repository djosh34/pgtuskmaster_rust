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

ha_feature_test!(ha_replica_faults_keep_cluster_healthy);
ha_feature_test!(ha_primary_faults_fail_over_then_recover);
ha_feature_test!(ha_quorum_loss_and_dcs_loss);
ha_feature_test!(ha_rejoin_and_restart_recovery);
ha_feature_test!(ha_operator_switchovers);
