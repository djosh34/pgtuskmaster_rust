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

    for role_key in cfg.postgres.roles.extra.keys() {
        if matches!(role_key.as_str(), "superuser" | "replicator" | "rewinder") {
            return Err(ConfigError::Validation {
                field: "postgres.roles.extra",
                message: format!(
                    "managed extra role key `{}` is reserved for mandatory postgres roles",
                    role_key.as_str()
                ),
            });
        }

    }

    validate_unique_managed_role_usernames(cfg)?;

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

fn validate_unique_managed_role_usernames(cfg: &RuntimeConfig) -> Result<(), ConfigError> {
    let managed_usernames = [
        (
            "postgres.roles.mandatory.superuser.username".to_string(),
            cfg.postgres
                .roles
                .mandatory
                .superuser
                .username
                .as_str()
                .to_string(),
        ),
        (
            "postgres.roles.mandatory.replicator.username".to_string(),
            cfg.postgres
                .roles
                .mandatory
                .replicator
                .username
                .as_str()
                .to_string(),
        ),
        (
            "postgres.roles.mandatory.rewinder.username".to_string(),
            cfg.postgres
                .roles
                .mandatory
                .rewinder
                .username
                .as_str()
                .to_string(),
        ),
    ]
    .into_iter()
    .chain(cfg.postgres.roles.extra.iter().map(|(role_key, role)| {
        (
            format!("postgres.roles.extra.{}.username", role_key.as_str()),
            role.role.username.as_str().to_string(),
        )
    }))
    .collect::<Vec<_>>();

    let mut seen = std::collections::BTreeMap::<String, String>::new();
    for (field, username) in managed_usernames {
        if let Some(first_field) = seen.insert(username.clone(), field.clone()) {
            return Err(ConfigError::Validation {
                field: "postgres.roles",
                message: format!(
                    "managed postgres role username `{username}` is declared more than once (`{first_field}` and `{field}`)"
                ),
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        config::{
            validate_runtime_config, ExtraManagedPostgresRoleConfig, ManagedPostgresRoleKey,
            PostgresRoleConfig, PostgresRoleName, PostgresRolePrivilege, RoleAuthConfig,
            SecretSource,
        },
        dev_support::runtime_config::RuntimeConfigBuilder,
    };

    fn inline_password(value: &str) -> SecretSource {
        SecretSource::Inline {
            content: value.to_string(),
        }
    }

    #[test]
    fn rejects_reserved_extra_managed_role_key() -> Result<(), String> {
        let cfg = RuntimeConfigBuilder::new()
            .transform_postgres(|postgres| crate::config::PostgresConfig {
                roles: crate::config::PostgresRolesConfig {
                    extra: BTreeMap::from([(
                        ManagedPostgresRoleKey("superuser".to_string()),
                        ExtraManagedPostgresRoleConfig {
                            role: PostgresRoleConfig {
                                username: PostgresRoleName("analytics".to_string()),
                                auth: RoleAuthConfig::Password {
                                    password: inline_password("analytics-secret"),
                                },
                            },
                            privilege: PostgresRolePrivilege::Login,
                            member_of: Vec::new(),
                        },
                    )]),
                    ..postgres.roles
                },
                ..postgres
            })
            .build();

        let err = match validate_runtime_config(&cfg) {
            Ok(()) => {
                return Err("expected reserved extra managed role key to be rejected".to_string());
            }
            Err(err) => err,
        };

        match err {
            crate::config::ConfigError::Validation { field, message } => {
                if field != "postgres.roles.extra" {
                    return Err(format!("unexpected field `{field}`"));
                }
                if !message.contains("reserved") {
                    return Err(format!("unexpected message `{message}`"));
                }
            }
            other => return Err(format!("unexpected error variant: {other}")),
        }

        Ok(())
    }

    #[test]
    fn rejects_duplicate_managed_role_usernames() -> Result<(), String> {
        let cfg = RuntimeConfigBuilder::new()
            .transform_postgres(|postgres| crate::config::PostgresConfig {
                roles: crate::config::PostgresRolesConfig {
                    extra: BTreeMap::from([(
                        ManagedPostgresRoleKey("analytics".to_string()),
                        ExtraManagedPostgresRoleConfig {
                            role: PostgresRoleConfig {
                                username: postgres.roles.mandatory.replicator.username.clone(),
                                auth: RoleAuthConfig::Password {
                                    password: inline_password("analytics-secret"),
                                },
                            },
                            privilege: PostgresRolePrivilege::Login,
                            member_of: Vec::new(),
                        },
                    )]),
                    ..postgres.roles
                },
                ..postgres
            })
            .build();

        let err = match validate_runtime_config(&cfg) {
            Ok(()) => {
                return Err(
                    "expected duplicate managed postgres usernames to be rejected".to_string()
                );
            }
            Err(err) => err,
        };

        match err {
            crate::config::ConfigError::Validation { field, message } => {
                if field != "postgres.roles" {
                    return Err(format!("unexpected field `{field}`"));
                }
                if !message.contains("declared more than once") {
                    return Err(format!("unexpected message `{message}`"));
                }
            }
            other => return Err(format!("unexpected error variant: {other}")),
        }

        Ok(())
    }

    #[test]
    fn rejects_empty_extra_managed_role_username() -> Result<(), String> {
        match PostgresRoleName::try_from(" ") {
            Ok(_) => Err("expected empty managed postgres username to be rejected".to_string()),
            Err(_) => Ok(()),
        }
    }
}
