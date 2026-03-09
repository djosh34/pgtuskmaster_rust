pub mod config;
pub mod handle;
pub mod ops;
pub mod startup;
pub mod util;

pub use config::{Mode, PostgresRoleOverrides, RecoveryBinaryOverrides, TestConfig, TimeoutConfig};
pub use handle::{NodeHandle, RuntimeNodeHandle, RuntimeNodeSet, TestClusterHandle};
pub use startup::start_cluster;

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::env;
    use std::time::Duration;

    use crate::state::WorkerError;
    use crate::test_harness::runtime_config;

    use super::{
        start_cluster, util, Mode, RecoveryBinaryOverrides, RuntimeNodeHandle, RuntimeNodeSet,
        TestConfig, TimeoutConfig,
    };

    #[test]
    fn test_config_validation_rejects_empty_fields() {
        let config = TestConfig {
            test_name: " ".to_string(),
            cluster_name: "".to_string(),
            scope: " ".to_string(),
            node_count: 0,
            namespace: None,
            etcd_members: Vec::new(),
            node_etcd_colocation: BTreeMap::new(),
            recovery_binary_overrides: BTreeMap::new(),
            postgres_roles: None,
            mode: Mode::Plain,
            timeouts: TimeoutConfig {
                command_timeout: Duration::from_secs(1),
                command_kill_wait_timeout: Duration::from_secs(1),
                http_step_timeout: Duration::from_secs(1),
                api_readiness_timeout: Duration::from_secs(1),
                bootstrap_primary_timeout: Duration::from_secs(1),
                scenario_timeout: Duration::from_secs(1),
            },
        };

        let result = config.validate();
        assert!(result.is_err());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn start_and_shutdown_plain_cluster_smoke() -> Result<(), WorkerError> {
        util::run_with_local_set(async {
            let config = TestConfig {
                test_name: "ha-e2e-harness-smoke-plain".to_string(),
                cluster_name: "cluster-e2e-harness-smoke".to_string(),
                scope: "scope-ha-e2e-harness-smoke".to_string(),
                node_count: 1,
                namespace: None,
                etcd_members: vec!["etcd-a".to_string()],
                node_etcd_colocation: BTreeMap::new(),
                recovery_binary_overrides: BTreeMap::new(),
                postgres_roles: None,
                mode: Mode::Plain,
                timeouts: TimeoutConfig {
                    command_timeout: Duration::from_secs(30),
                    command_kill_wait_timeout: Duration::from_secs(3),
                    http_step_timeout: Duration::from_secs(20),
                    api_readiness_timeout: Duration::from_secs(90),
                    bootstrap_primary_timeout: Duration::from_secs(90),
                    scenario_timeout: Duration::from_secs(120),
                },
            };

            let mut handle = start_cluster(config).await?;
            handle.shutdown().await?;
            Ok(())
        })
        .await
    }

    #[test]
    fn test_config_validation_rejects_relative_recovery_override_path() {
        let mut recovery_binary_overrides = BTreeMap::new();
        recovery_binary_overrides.insert(
            "node-1".to_string(),
            RecoveryBinaryOverrides {
                pg_basebackup: Some(std::path::PathBuf::from("relative/pg_basebackup")),
                pg_rewind: None,
            },
        );
        let config = TestConfig {
            test_name: "ha-e2e-invalid-recovery-binary".to_string(),
            cluster_name: "cluster-e2e-invalid-recovery-binary".to_string(),
            scope: "scope-ha-e2e-invalid-recovery-binary".to_string(),
            node_count: 1,
            namespace: None,
            etcd_members: vec!["etcd-a".to_string()],
            node_etcd_colocation: BTreeMap::new(),
            recovery_binary_overrides,
            postgres_roles: None,
            mode: Mode::Plain,
            timeouts: TimeoutConfig {
                command_timeout: Duration::from_secs(1),
                command_kill_wait_timeout: Duration::from_secs(1),
                http_step_timeout: Duration::from_secs(1),
                api_readiness_timeout: Duration::from_secs(1),
                bootstrap_primary_timeout: Duration::from_secs(1),
                scenario_timeout: Duration::from_secs(1),
            },
        };

        let result = config.validate();
        assert!(
            matches!(result, Err(WorkerError::Message(message)) if message.contains("absolute path"))
        );
    }

    #[test]
    fn test_config_validation_rejects_unknown_colocated_etcd_member() {
        let mut node_etcd_colocation = BTreeMap::new();
        node_etcd_colocation.insert("node-1".to_string(), "etcd-missing".to_string());
        let config = TestConfig {
            test_name: "ha-e2e-invalid-colocated-etcd".to_string(),
            cluster_name: "cluster-e2e-invalid-colocated-etcd".to_string(),
            scope: "scope-ha-e2e-invalid-colocated-etcd".to_string(),
            node_count: 1,
            namespace: None,
            etcd_members: vec!["etcd-a".to_string()],
            node_etcd_colocation,
            recovery_binary_overrides: BTreeMap::new(),
            postgres_roles: None,
            mode: Mode::Plain,
            timeouts: TimeoutConfig {
                command_timeout: Duration::from_secs(1),
                command_kill_wait_timeout: Duration::from_secs(1),
                http_step_timeout: Duration::from_secs(1),
                api_readiness_timeout: Duration::from_secs(1),
                bootstrap_primary_timeout: Duration::from_secs(1),
                scenario_timeout: Duration::from_secs(1),
            },
        };

        let result = config.validate();
        assert!(
            matches!(result, Err(WorkerError::Message(message)) if message.contains("unknown etcd member"))
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn whole_node_helper_state_validation_errors_are_actionable() -> Result<(), WorkerError> {
        util::run_with_local_set(async {
            let guard = crate::test_harness::namespace::NamespaceGuard::new(
                "ha-e2e-whole-node-helper-validation",
            )?;
            let mut handle = super::TestClusterHandle {
                guard,
                timeouts: TimeoutConfig {
                    command_timeout: Duration::from_secs(1),
                    command_kill_wait_timeout: Duration::from_secs(1),
                    http_step_timeout: Duration::from_secs(1),
                    api_readiness_timeout: Duration::from_secs(1),
                    bootstrap_primary_timeout: Duration::from_secs(1),
                    scenario_timeout: Duration::from_secs(1),
                },
                binaries: runtime_config::sample_binary_paths(),
                superuser_username: "postgres".to_string(),
                superuser_dbname: "postgres".to_string(),
                etcd: None,
                nodes: Vec::new(),
                runtime_nodes: RuntimeNodeSet::new(),
                etcd_proxies: BTreeMap::new(),
                api_proxies: BTreeMap::new(),
                pg_proxies: BTreeMap::new(),
                node_etcd_colocation: BTreeMap::new(),
                whole_node_outages: BTreeMap::from([(
                    "node-1".to_string(),
                    super::handle::WholeNodeOutageState {
                        kind: super::handle::WholeNodeOutageKind::CleanStop,
                        etcd_member_name: None,
                    },
                )]),
            };

            let stop_err = handle.stop_whole_node("node-1").await;
            assert!(
                matches!(stop_err, Err(WorkerError::Message(message)) if message.contains("already active"))
            );
            let kill_err = handle.kill_whole_node("node-1").await;
            assert!(
                matches!(kill_err, Err(WorkerError::Message(message)) if message.contains("already active"))
            );
            let restart_err = handle.restart_whole_node("node-2").await;
            assert!(
                matches!(restart_err, Err(WorkerError::Message(message)) if message.contains("without active outage"))
            );

            Ok(())
        })
        .await
    }

    #[tokio::test(flavor = "current_thread")]
    async fn runtime_node_set_preserves_metadata_for_offline_node() -> Result<(), WorkerError> {
        util::run_with_local_set(async {
            let mut runtime_nodes = RuntimeNodeSet::new();
            let runtime_cfg = runtime_config::sample_runtime_config();
            let postgres_log_file = env::temp_dir().join("ha-e2e-runtime-node-set.log");
            let replaced = runtime_nodes.insert(
                "node-b".to_string(),
                RuntimeNodeHandle {
                    runtime_cfg: runtime_cfg.clone(),
                    runtime_binary_path: env::temp_dir().join("pgtuskmaster"),
                    runtime_config_path: env::temp_dir().join("node-b.toml"),
                    postgres_log_file: postgres_log_file.clone(),
                    runtime_log_file: env::temp_dir().join("ha-e2e-runtime-node-set-runtime.log"),
                    state: super::handle::RuntimeNodeState::Offline,
                },
            );
            assert!(replaced.is_none());

            let (stored_cfg, stored_log_file) =
                runtime_nodes.metadata_for_node("node-b").ok_or_else(|| {
                    WorkerError::Message("runtime node metadata missing after insert".to_string())
                })?;
            assert_eq!(stored_cfg, &runtime_cfg);
            assert_eq!(stored_log_file, &postgres_log_file);

            runtime_nodes.shutdown_all().await?;
            Ok(())
        })
        .await
    }
}
