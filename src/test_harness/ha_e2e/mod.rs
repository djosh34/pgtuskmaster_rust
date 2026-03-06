pub(crate) mod config;
pub(crate) mod handle;
pub(crate) mod ops;
pub(crate) mod startup;
pub(crate) mod util;

pub(crate) use config::{BackupHarnessConfig, Mode, TestConfig, TimeoutConfig};
pub(crate) use handle::NodeHandle;
pub(crate) use startup::start_cluster;

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::state::WorkerError;

    use super::{start_cluster, util, Mode, TestConfig, TimeoutConfig};

    #[test]
    fn test_config_validation_rejects_empty_fields() {
        let config = TestConfig {
            test_name: " ".to_string(),
            cluster_name: "".to_string(),
            scope: " ".to_string(),
            node_count: 0,
            etcd_members: Vec::new(),
            mode: Mode::Plain,
            timeouts: TimeoutConfig {
                command_timeout: Duration::from_secs(1),
                command_kill_wait_timeout: Duration::from_secs(1),
                http_step_timeout: Duration::from_secs(1),
                api_readiness_timeout: Duration::from_secs(1),
                bootstrap_primary_timeout: Duration::from_secs(1),
                scenario_timeout: Duration::from_secs(1),
            },
            backup: None,
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
                etcd_members: vec!["etcd-a".to_string()],
                mode: Mode::Plain,
                timeouts: TimeoutConfig {
                    command_timeout: Duration::from_secs(30),
                    command_kill_wait_timeout: Duration::from_secs(3),
                    http_step_timeout: Duration::from_secs(20),
                    api_readiness_timeout: Duration::from_secs(90),
                    bootstrap_primary_timeout: Duration::from_secs(90),
                    scenario_timeout: Duration::from_secs(120),
                },
                backup: None,
            };

            let mut handle = start_cluster(config).await?;
            handle.shutdown().await?;
            Ok(())
        })
        .await
    }
}
