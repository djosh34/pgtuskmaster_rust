use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::state::WorkerError;
use crate::test_harness::binaries::validate_executable_file;
use crate::test_harness::namespace::TestNamespace;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Plain,
    PartitionProxy,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostgresRoleOverrides {
    pub replicator_username: String,
    pub replicator_password: String,
    pub rewinder_username: String,
    pub rewinder_password: String,
}

impl Default for PostgresRoleOverrides {
    fn default() -> Self {
        Self {
            replicator_username: "replicator".to_string(),
            replicator_password: "secret-password".to_string(),
            rewinder_username: "rewinder".to_string(),
            rewinder_password: "secret-password".to_string(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RecoveryBinaryOverrides {
    pub pg_basebackup: Option<PathBuf>,
    pub pg_rewind: Option<PathBuf>,
}

#[derive(Clone, Debug)]
pub struct TestConfig {
    pub test_name: String,
    pub cluster_name: String,
    pub scope: String,
    pub node_count: usize,
    pub namespace: Option<TestNamespace>,
    pub etcd_members: Vec<String>,
    pub recovery_binary_overrides: BTreeMap<String, RecoveryBinaryOverrides>,
    pub postgres_roles: Option<PostgresRoleOverrides>,
    pub mode: Mode,
    pub timeouts: TimeoutConfig,
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
        if let Some(namespace) = self.namespace.as_ref() {
            validate_namespace(namespace)?;
        }
        if self.etcd_members.is_empty() {
            return Err(WorkerError::Message(
                "TestConfig.etcd_members must include at least one member".to_string(),
            ));
        }

        if let Some(postgres_roles) = self.postgres_roles.as_ref() {
            validate_non_empty_field(
                "TestConfig.postgres_roles.replicator_username",
                postgres_roles.replicator_username.as_str(),
            )?;
            validate_non_empty_field(
                "TestConfig.postgres_roles.replicator_password",
                postgres_roles.replicator_password.as_str(),
            )?;
            validate_non_empty_field(
                "TestConfig.postgres_roles.rewinder_username",
                postgres_roles.rewinder_username.as_str(),
            )?;
            validate_non_empty_field(
                "TestConfig.postgres_roles.rewinder_password",
                postgres_roles.rewinder_password.as_str(),
            )?;
        }

        let mut seen = BTreeSet::new();
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

        for (node_id, overrides) in &self.recovery_binary_overrides {
            validate_known_node_id(node_id.as_str(), self.node_count)?;
            if overrides.pg_basebackup.is_none() && overrides.pg_rewind.is_none() {
                return Err(WorkerError::Message(format!(
                    "TestConfig.recovery_binary_overrides[{node_id}] must override at least one recovery binary"
                )));
            }
            if let Some(path) = overrides.pg_basebackup.as_deref() {
                validate_recovery_binary_override_path(node_id.as_str(), "pg_basebackup", path)?;
            }
            if let Some(path) = overrides.pg_rewind.as_deref() {
                validate_recovery_binary_override_path(node_id.as_str(), "pg_rewind", path)?;
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

        Ok(())
    }
}

fn validate_non_empty_field(field: &str, value: &str) -> Result<(), WorkerError> {
    if value.trim().is_empty() {
        return Err(WorkerError::Message(format!("{field} must not be empty")));
    }

    Ok(())
}

fn validate_namespace(namespace: &TestNamespace) -> Result<(), WorkerError> {
    validate_non_empty_field("TestConfig.namespace.id", namespace.id.as_str())?;
    if !namespace.root_dir.is_absolute() {
        return Err(WorkerError::Message(format!(
            "TestConfig.namespace.root_dir must be absolute: {}",
            namespace.root_dir.display()
        )));
    }
    Ok(())
}

fn validate_known_node_id(node_id: &str, node_count: usize) -> Result<(), WorkerError> {
    validate_non_empty_field("TestConfig.recovery_binary_overrides node id", node_id)?;
    let suffix = node_id.strip_prefix("node-").ok_or_else(|| {
        WorkerError::Message(format!(
            "TestConfig.recovery_binary_overrides contains unsupported node id: {node_id}"
        ))
    })?;
    let index = suffix.parse::<usize>().map_err(|err| {
        WorkerError::Message(format!(
            "TestConfig.recovery_binary_overrides contains invalid node id {node_id}: {err}"
        ))
    })?;
    if index == 0 || index > node_count {
        return Err(WorkerError::Message(format!(
            "TestConfig.recovery_binary_overrides node id {node_id} is outside configured node_count={node_count}"
        )));
    }
    Ok(())
}

fn validate_recovery_binary_override_path(
    node_id: &str,
    binary_name: &str,
    path: &Path,
) -> Result<(), WorkerError> {
    let field = format!("TestConfig.recovery_binary_overrides[{node_id}].{binary_name}");
    if path.as_os_str().is_empty() {
        return Err(WorkerError::Message(format!("{field} must not be empty")));
    }
    if !path.is_absolute() {
        return Err(WorkerError::Message(format!(
            "{field} must be an absolute path: {}",
            path.display()
        )));
    }
    validate_executable_file(path, field.as_str())
        .map_err(|err| WorkerError::Message(err.to_string()))?;
    Ok(())
}
