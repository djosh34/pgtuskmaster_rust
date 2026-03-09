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

    #[tokio::test(flavor = "current_thread")]
    async fn runtime_node_set_replaces_task_without_losing_metadata() -> Result<(), WorkerError> {
        util::run_with_local_set(async {
            let mut runtime_nodes = RuntimeNodeSet::new();
            let runtime_cfg = runtime_config::sample_runtime_config();
            let postgres_log_file = env::temp_dir().join("ha-e2e-runtime-node-set.log");
            let first_task = tokio::task::spawn_local(async {
                std::future::pending::<Result<(), WorkerError>>().await
            });
            let replaced = runtime_nodes.insert(
                "node-b".to_string(),
                RuntimeNodeHandle {
                    runtime_cfg: runtime_cfg.clone(),
                    postgres_log_file: postgres_log_file.clone(),
                    task: first_task,
                },
            );
            assert!(replaced.is_none());

            let replacement_task = tokio::task::spawn_local(async {
                std::future::pending::<Result<(), WorkerError>>().await
            });
            runtime_nodes.replace_task("node-b", replacement_task)?;

            let (stored_cfg, stored_log_file) =
                runtime_nodes.metadata_for_node("node-b").ok_or_else(|| {
                    WorkerError::Message(
                        "runtime node metadata missing after replacement".to_string(),
                    )
                })?;
            assert_eq!(stored_cfg, &runtime_cfg);
            assert_eq!(stored_log_file, &postgres_log_file);

            runtime_nodes.shutdown_all().await;
            Ok(())
        })
        .await
    }
}
