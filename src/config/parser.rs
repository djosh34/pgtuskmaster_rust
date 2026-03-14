use std::path::Path;

use serde::Deserialize;
use thiserror::Error;

use super::schema::{DcsAuthConfig, DcsTlsConfig, PgtmConfig, PostgresBinaryName, RuntimeConfig};

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read config file {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse config file {path}: {source}")]
    Parse {
        path: String,
        #[source]
        source: toml::de::Error,
    },
    #[error("invalid config field `{field}`: {message}")]
    Validation {
        field: &'static str,
        message: String,
    },
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum OperatorConfigDocument {
    Operator(Box<PgtmConfig>),
    Runtime(Box<RuntimeConfig>),
}

pub fn load_runtime_config(path: &Path) -> Result<RuntimeConfig, ConfigError> {
    let contents = read_config_file(path)?;
    let cfg: RuntimeConfig = toml::from_str(&contents).map_err(|source| ConfigError::Parse {
        path: path.display().to_string(),
        source,
    })?;
    validate_runtime_config(&cfg)?;
    Ok(cfg)
}

pub fn load_operator_config(path: &Path) -> Result<PgtmConfig, ConfigError> {
    let contents = read_config_file(path)?;
    let document: OperatorConfigDocument =
        toml::from_str(&contents).map_err(|source| ConfigError::Parse {
            path: path.display().to_string(),
            source,
        })?;

    let cfg = match document {
        OperatorConfigDocument::Operator(cfg) => *cfg,
        OperatorConfigDocument::Runtime(runtime) => {
            runtime.pgtm.ok_or_else(|| ConfigError::Validation {
                field: "pgtm",
                message: "missing operator config block in runtime document".to_string(),
            })?
        }
    };
    validate_operator_config(&cfg)?;
    Ok(cfg)
}

pub fn validate_runtime_config(cfg: &RuntimeConfig) -> Result<(), ConfigError> {
    validate_non_empty("cluster.name", cfg.cluster.name.as_str())?;
    validate_non_empty("cluster.scope", cfg.cluster.scope.as_str())?;
    validate_non_empty("cluster.member_id", cfg.cluster.member_id.as_str())?;

    if cfg.dcs.endpoints.is_empty() {
        return Err(ConfigError::Validation {
            field: "dcs.endpoints",
            message: "at least one endpoint is required".to_string(),
        });
    }

    validate_non_empty(
        "postgres.local_database",
        cfg.postgres.local_database.as_str(),
    )?;
    validate_non_empty(
        "postgres.rewind.database",
        cfg.postgres.rewind.database.as_str(),
    )?;
    validate_non_empty(
        "postgres.roles.superuser.username",
        cfg.postgres.roles.superuser.username.as_str(),
    )?;
    validate_non_empty(
        "postgres.roles.replicator.username",
        cfg.postgres.roles.replicator.username.as_str(),
    )?;
    validate_non_empty(
        "postgres.roles.rewinder.username",
        cfg.postgres.roles.rewinder.username.as_str(),
    )?;

    if cfg
        .dcs
        .endpoints
        .iter()
        .any(|endpoint| matches!(endpoint.scheme(), crate::config::DcsEndpointScheme::Https))
        && matches!(cfg.dcs.client.tls, DcsTlsConfig::Disabled)
    {
        return Err(ConfigError::Validation {
            field: "dcs.client.tls",
            message: "https DCS endpoints require `dcs.client.tls` to be configured".to_string(),
        });
    }

    if let DcsAuthConfig::Basic { username, .. } = &cfg.dcs.client.auth {
        validate_non_empty("dcs.client.auth.username", username.as_str())?;
    }

    for binary in [
        PostgresBinaryName::Initdb,
        PostgresBinaryName::PgBasebackup,
        PostgresBinaryName::PgRewind,
        PostgresBinaryName::PgCtl,
    ] {
        cfg.process
            .binaries
            .resolve_binary_path(binary)
            .map_err(|message| ConfigError::Validation {
                field: "process.binaries",
                message,
            })?;
    }

    if let Some(pgtm) = cfg.pgtm.as_ref() {
        validate_operator_config(pgtm)?;
    }

    Ok(())
}

pub fn validate_operator_config(cfg: &PgtmConfig) -> Result<(), ConfigError> {
    if let Some(base_url) = cfg.api.base_url.as_ref() {
        validate_non_empty("pgtm.api.base_url", base_url.as_str())?;
    }
    if let Some(advertised_url) = cfg.api.advertised_url.as_ref() {
        validate_non_empty("pgtm.api.advertised_url", advertised_url.as_str())?;
    }
    if let Some(primary_target) = cfg.primary_target.as_ref() {
        validate_non_empty("pgtm.primary_target.host", primary_target.host.as_str())?;
    }
    Ok(())
}

fn read_config_file(path: &Path) -> Result<String, ConfigError> {
    std::fs::read_to_string(path).map_err(|source| ConfigError::Io {
        path: path.display().to_string(),
        source,
    })
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), ConfigError> {
    if value.trim().is_empty() {
        return Err(ConfigError::Validation {
            field,
            message: "must not be empty".to_string(),
        });
    }
    Ok(())
}
