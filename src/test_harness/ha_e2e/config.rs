use std::time::Duration;

use crate::config::{BackupRecoveryMode, BackupTakeoverPolicy};
use crate::state::WorkerError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Mode {
    Plain,
    PartitionProxy,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct PgBackRestHarnessOptions {
    pub(crate) backup: Vec<String>,
    pub(crate) info: Vec<String>,
    pub(crate) check: Vec<String>,
    pub(crate) restore: Vec<String>,
    pub(crate) archive_push: Vec<String>,
    pub(crate) archive_get: Vec<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct PgBackRestHarnessConfig {
    pub(crate) stanza: String,
    pub(crate) repo: String,
    /// Relative to the test namespace root directory.
    pub(crate) repo1_path_rel: String,
    pub(crate) options: PgBackRestHarnessOptions,
}

#[derive(Clone, Debug)]
pub(crate) struct BackupHarnessConfig {
    pub(crate) enabled: bool,
    pub(crate) bootstrap_enabled: bool,
    pub(crate) takeover_policy: BackupTakeoverPolicy,
    pub(crate) recovery_mode: BackupRecoveryMode,
    pub(crate) pgbackrest: Option<PgBackRestHarnessConfig>,
}

#[derive(Clone, Debug)]
pub(crate) struct TimeoutConfig {
    pub(crate) command_timeout: Duration,
    pub(crate) command_kill_wait_timeout: Duration,
    pub(crate) http_step_timeout: Duration,
    pub(crate) api_readiness_timeout: Duration,
    pub(crate) bootstrap_primary_timeout: Duration,
    pub(crate) scenario_timeout: Duration,
}

#[derive(Clone, Debug)]
pub(crate) struct TestConfig {
    pub(crate) test_name: String,
    pub(crate) cluster_name: String,
    pub(crate) scope: String,
    pub(crate) node_count: usize,
    pub(crate) etcd_members: Vec<String>,
    pub(crate) mode: Mode,
    pub(crate) timeouts: TimeoutConfig,
    pub(crate) backup: Option<BackupHarnessConfig>,
}

impl TestConfig {
    pub(crate) fn validate(&self) -> Result<(), WorkerError> {
        if self.test_name.trim().is_empty() {
            return Err(WorkerError::Message(
                "TestConfig.test_name must not be empty".to_string(),
            ));
        }
        if self.cluster_name.trim().is_empty() {
            return Err(WorkerError::Message(
                "TestConfig.cluster_name must not be empty".to_string(),
            ));
        }
        if self.scope.trim().is_empty() {
            return Err(WorkerError::Message(
                "TestConfig.scope must not be empty".to_string(),
            ));
        }
        if self.node_count == 0 {
            return Err(WorkerError::Message(
                "TestConfig.node_count must be greater than zero".to_string(),
            ));
        }
        if self.etcd_members.is_empty() {
            return Err(WorkerError::Message(
                "TestConfig.etcd_members must include at least one member".to_string(),
            ));
        }

        let mut seen = std::collections::BTreeSet::new();
        for name in &self.etcd_members {
            let trimmed = name.trim();
            if trimmed.is_empty() {
                return Err(WorkerError::Message(
                    "TestConfig.etcd_members contains an empty name".to_string(),
                ));
            }
            if !seen.insert(trimmed.to_string()) {
                return Err(WorkerError::Message(format!(
                    "TestConfig.etcd_members contains duplicate member name: {trimmed}"
                )));
            }
        }

        if self.timeouts.command_timeout.is_zero() {
            return Err(WorkerError::Message(
                "TestConfig.timeouts.command_timeout must be non-zero".to_string(),
            ));
        }
        if self.timeouts.http_step_timeout.is_zero() {
            return Err(WorkerError::Message(
                "TestConfig.timeouts.http_step_timeout must be non-zero".to_string(),
            ));
        }
        if self.timeouts.api_readiness_timeout.is_zero() {
            return Err(WorkerError::Message(
                "TestConfig.timeouts.api_readiness_timeout must be non-zero".to_string(),
            ));
        }
        if self.timeouts.bootstrap_primary_timeout.is_zero() {
            return Err(WorkerError::Message(
                "TestConfig.timeouts.bootstrap_primary_timeout must be non-zero".to_string(),
            ));
        }
        if self.timeouts.scenario_timeout.is_zero() {
            return Err(WorkerError::Message(
                "TestConfig.timeouts.scenario_timeout must be non-zero".to_string(),
            ));
        }

        if let Some(backup) = self.backup.as_ref() {
            if backup.enabled {
                let pgbackrest = backup.pgbackrest.as_ref().ok_or_else(|| {
                    WorkerError::Message(
                        "TestConfig.backup.enabled requires TestConfig.backup.pgbackrest"
                            .to_string(),
                    )
                })?;
                if pgbackrest.stanza.trim().is_empty() {
                    return Err(WorkerError::Message(
                        "TestConfig.backup.pgbackrest.stanza must not be empty".to_string(),
                    ));
                }
                if pgbackrest.repo.trim().is_empty() {
                    return Err(WorkerError::Message(
                        "TestConfig.backup.pgbackrest.repo must not be empty".to_string(),
                    ));
                }
                if pgbackrest.repo1_path_rel.trim().is_empty() {
                    return Err(WorkerError::Message(
                        "TestConfig.backup.pgbackrest.repo1_path_rel must not be empty".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }
}
