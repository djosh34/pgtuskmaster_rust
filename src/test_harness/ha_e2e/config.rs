use std::time::Duration;

use crate::config::{BackupRecoveryMode, BackupTakeoverPolicy};
use crate::state::WorkerError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Plain,
    PartitionProxy,
}

#[derive(Clone, Debug, Default)]
pub struct PgBackRestHarnessOptions {
    pub backup: Vec<String>,
    pub info: Vec<String>,
    pub check: Vec<String>,
    pub restore: Vec<String>,
    pub archive_push: Vec<String>,
    pub archive_get: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct PgBackRestHarnessConfig {
    pub stanza: String,
    pub repo: String,
    /// Relative to the test namespace root directory.
    pub repo1_path_rel: String,
    pub options: PgBackRestHarnessOptions,
}

#[derive(Clone, Debug)]
pub struct BackupHarnessConfig {
    pub enabled: bool,
    pub bootstrap_enabled: bool,
    pub takeover_policy: BackupTakeoverPolicy,
    pub recovery_mode: BackupRecoveryMode,
    pub pgbackrest: Option<PgBackRestHarnessConfig>,
}

#[derive(Clone, Debug)]
pub struct TimeoutConfig {
    pub command_timeout: Duration,
    pub command_kill_wait_timeout: Duration,
    pub http_step_timeout: Duration,
    pub api_readiness_timeout: Duration,
    pub bootstrap_primary_timeout: Duration,
    pub scenario_timeout: Duration,
}

#[derive(Clone, Debug)]
pub struct TestConfig {
    pub test_name: String,
    pub cluster_name: String,
    pub scope: String,
    pub node_count: usize,
    pub etcd_members: Vec<String>,
    pub mode: Mode,
    pub timeouts: TimeoutConfig,
    pub backup: Option<BackupHarnessConfig>,
}

impl TestConfig {
    pub fn validate(&self) -> Result<(), WorkerError> {
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
