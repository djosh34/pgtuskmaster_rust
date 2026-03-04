use std::path::Path;

use thiserror::Error;

use super::defaults::apply_defaults;
use super::schema::{PartialRuntimeConfig, RuntimeConfig};

const MIN_TIMEOUT_MS: u64 = 1;
const MAX_TIMEOUT_MS: u64 = 86_400_000;

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

pub fn load_runtime_config(path: &Path) -> Result<RuntimeConfig, ConfigError> {
    let contents = std::fs::read_to_string(path).map_err(|source| ConfigError::Io {
        path: path.display().to_string(),
        source,
    })?;

    let raw: PartialRuntimeConfig =
        toml::from_str(&contents).map_err(|source| ConfigError::Parse {
            path: path.display().to_string(),
            source,
        })?;

    let cfg = apply_defaults(raw);
    validate_runtime_config(&cfg)?;
    Ok(cfg)
}

pub fn validate_runtime_config(cfg: &RuntimeConfig) -> Result<(), ConfigError> {
    validate_non_empty_path("postgres.data_dir", &cfg.postgres.data_dir)?;
    validate_non_empty("postgres.listen_host", cfg.postgres.listen_host.as_str())?;
    validate_port("postgres.listen_port", cfg.postgres.listen_port)?;
    validate_non_empty_path("postgres.socket_dir", &cfg.postgres.socket_dir)?;
    validate_non_empty_path("postgres.log_file", &cfg.postgres.log_file)?;
    validate_non_empty(
        "postgres.rewind_source_host",
        cfg.postgres.rewind_source_host.as_str(),
    )?;
    validate_port(
        "postgres.rewind_source_port",
        cfg.postgres.rewind_source_port,
    )?;

    validate_non_empty_path("process.binaries.postgres", &cfg.process.binaries.postgres)?;
    validate_non_empty_path("process.binaries.pg_ctl", &cfg.process.binaries.pg_ctl)?;
    validate_non_empty_path(
        "process.binaries.pg_rewind",
        &cfg.process.binaries.pg_rewind,
    )?;
    validate_non_empty_path("process.binaries.initdb", &cfg.process.binaries.initdb)?;
    validate_non_empty_path(
        "process.binaries.pg_basebackup",
        &cfg.process.binaries.pg_basebackup,
    )?;
    validate_non_empty_path("process.binaries.psql", &cfg.process.binaries.psql)?;

    validate_timeout(
        "process.pg_rewind_timeout_ms",
        cfg.process.pg_rewind_timeout_ms,
    )?;
    validate_timeout(
        "process.bootstrap_timeout_ms",
        cfg.process.bootstrap_timeout_ms,
    )?;
    validate_timeout("process.fencing_timeout_ms", cfg.process.fencing_timeout_ms)?;

    validate_timeout(
        "logging.postgres.poll_interval_ms",
        cfg.logging.postgres.poll_interval_ms,
    )?;
    if let Some(path) = cfg.logging.postgres.pg_ctl_log_file.as_ref() {
        validate_non_empty_path("logging.postgres.pg_ctl_log_file", path)?;
    }
    if let Some(path) = cfg.logging.postgres.log_dir.as_ref() {
        validate_non_empty_path("logging.postgres.log_dir", path)?;
    }
    if let Some(path) = cfg.logging.postgres.archive_command_log_file.as_ref() {
        validate_non_empty_path("logging.postgres.archive_command_log_file", path)?;
    }
    if cfg.logging.postgres.cleanup.enabled {
        if cfg.logging.postgres.cleanup.max_files == 0 {
            return Err(ConfigError::Validation {
                field: "logging.postgres.cleanup.max_files",
                message: "must be greater than zero when cleanup is enabled".to_string(),
            });
        }
        if cfg.logging.postgres.cleanup.max_age_seconds == 0 {
            return Err(ConfigError::Validation {
                field: "logging.postgres.cleanup.max_age_seconds",
                message: "must be greater than zero when cleanup is enabled".to_string(),
            });
        }
    }

    if cfg.dcs.endpoints.is_empty() {
        return Err(ConfigError::Validation {
            field: "dcs.endpoints",
            message: "must contain at least one endpoint".to_string(),
        });
    }

    for endpoint in &cfg.dcs.endpoints {
        if endpoint.trim().is_empty() {
            return Err(ConfigError::Validation {
                field: "dcs.endpoints",
                message: "must not contain empty endpoint values".to_string(),
            });
        }
    }

    if cfg.dcs.scope.trim().is_empty() {
        return Err(ConfigError::Validation {
            field: "dcs.scope",
            message: "must not be empty".to_string(),
        });
    }

    if cfg.ha.loop_interval_ms == 0 {
        return Err(ConfigError::Validation {
            field: "ha.loop_interval_ms",
            message: "must be greater than zero".to_string(),
        });
    }

    if cfg.ha.lease_ttl_ms == 0 {
        return Err(ConfigError::Validation {
            field: "ha.lease_ttl_ms",
            message: "must be greater than zero".to_string(),
        });
    }

    if cfg.ha.lease_ttl_ms <= cfg.ha.loop_interval_ms {
        return Err(ConfigError::Validation {
            field: "ha.lease_ttl_ms",
            message: "must be greater than ha.loop_interval_ms".to_string(),
        });
    }

    validate_optional_non_empty("api.read_auth_token", cfg.api.read_auth_token.as_deref())?;
    validate_optional_non_empty("api.admin_auth_token", cfg.api.admin_auth_token.as_deref())?;

    Ok(())
}

fn validate_non_empty_path(field: &'static str, path: &Path) -> Result<(), ConfigError> {
    if path.as_os_str().is_empty() {
        return Err(ConfigError::Validation {
            field,
            message: "must not be empty".to_string(),
        });
    }
    Ok(())
}

fn validate_timeout(field: &'static str, value: u64) -> Result<(), ConfigError> {
    if !(MIN_TIMEOUT_MS..=MAX_TIMEOUT_MS).contains(&value) {
        return Err(ConfigError::Validation {
            field,
            message: format!("must be between {MIN_TIMEOUT_MS} and {MAX_TIMEOUT_MS} ms"),
        });
    }
    Ok(())
}

fn validate_port(field: &'static str, value: u16) -> Result<(), ConfigError> {
    if value == 0 {
        return Err(ConfigError::Validation {
            field,
            message: "must be greater than zero".to_string(),
        });
    }
    Ok(())
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

fn validate_optional_non_empty(
    field: &'static str,
    value: Option<&str>,
) -> Result<(), ConfigError> {
    if let Some(raw) = value {
        if raw.trim().is_empty() {
            return Err(ConfigError::Validation {
                field,
                message: "must not be empty when configured".to_string(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::config::schema::{
        ApiConfig, BinaryPaths, ClusterConfig, DcsConfig, DebugConfig, HaConfig, LogCleanupConfig,
        LogLevel, LoggingConfig, PostgresConfig, PostgresLoggingConfig, ProcessConfig, RuntimeConfig,
        SecurityConfig,
    };

    fn base_runtime_config() -> RuntimeConfig {
        RuntimeConfig {
            cluster: ClusterConfig {
                name: "cluster-a".to_string(),
                member_id: "member-a".to_string(),
            },
            postgres: PostgresConfig {
                data_dir: PathBuf::from("/var/lib/postgresql/data"),
                connect_timeout_s: 5,
                listen_host: "127.0.0.1".to_string(),
                listen_port: 5432,
                socket_dir: PathBuf::from("/tmp/pgtuskmaster/socket"),
                log_file: PathBuf::from("/tmp/pgtuskmaster/postgres.log"),
                rewind_source_host: "127.0.0.1".to_string(),
                rewind_source_port: 5432,
            },
            dcs: DcsConfig {
                endpoints: vec!["http://127.0.0.1:2379".to_string()],
                scope: "scope-a".to_string(),
            },
            ha: HaConfig {
                loop_interval_ms: 1_000,
                lease_ttl_ms: 10_000,
            },
            process: ProcessConfig {
                pg_rewind_timeout_ms: 120_000,
                bootstrap_timeout_ms: 300_000,
                fencing_timeout_ms: 30_000,
                binaries: BinaryPaths {
                    postgres: PathBuf::from("/usr/bin/postgres"),
                    pg_ctl: PathBuf::from("/usr/bin/pg_ctl"),
                    pg_rewind: PathBuf::from("/usr/bin/pg_rewind"),
                    initdb: PathBuf::from("/usr/bin/initdb"),
                    pg_basebackup: PathBuf::from("/usr/bin/pg_basebackup"),
                    psql: PathBuf::from("/usr/bin/psql"),
                },
            },
            logging: LoggingConfig {
                level: LogLevel::Info,
                capture_subprocess_output: true,
                postgres: PostgresLoggingConfig {
                    enabled: true,
                    pg_ctl_log_file: None,
                    log_dir: None,
                    archive_command_log_file: None,
                    poll_interval_ms: 200,
                    cleanup: LogCleanupConfig {
                        enabled: true,
                        max_files: 10,
                        max_age_seconds: 60,
                    },
                },
            },
            api: ApiConfig {
                listen_addr: "127.0.0.1:8080".to_string(),
                read_auth_token: None,
                admin_auth_token: None,
            },
            debug: DebugConfig { enabled: false },
            security: SecurityConfig {
                tls_enabled: false,
                auth_token: None,
            },
        }
    }

    #[test]
    fn validate_runtime_config_accepts_valid_config() {
        let cfg = base_runtime_config();
        assert!(validate_runtime_config(&cfg).is_ok());
    }

    #[test]
    fn validate_runtime_config_rejects_empty_binary_path() {
        let mut cfg = base_runtime_config();
        cfg.process.binaries.pg_ctl = PathBuf::new();

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "process.binaries.pg_ctl",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_rejects_bad_timeout() {
        let mut cfg = base_runtime_config();
        cfg.process.bootstrap_timeout_ms = 0;

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "process.bootstrap_timeout_ms",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_rejects_invalid_postgres_runtime_fields() {
        let mut cfg = base_runtime_config();
        cfg.postgres.listen_host = " ".to_string();
        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.listen_host",
                ..
            })
        ));

        let mut cfg = base_runtime_config();
        cfg.postgres.listen_port = 0;
        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.listen_port",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_rejects_missing_dcs_and_ha_invariants() {
        let mut cfg = base_runtime_config();
        cfg.dcs.endpoints.clear();

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "dcs.endpoints",
                ..
            })
        ));

        let mut cfg = base_runtime_config();
        cfg.ha.lease_ttl_ms = cfg.ha.loop_interval_ms;

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "ha.lease_ttl_ms",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_rejects_blank_api_tokens() {
        let mut cfg = base_runtime_config();
        cfg.api.read_auth_token = Some(" ".to_string());

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "api.read_auth_token",
                ..
            })
        ));

        let mut cfg = base_runtime_config();
        cfg.api.admin_auth_token = Some("\t".to_string());

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "api.admin_auth_token",
                ..
            })
        ));
    }

    #[test]
    fn load_runtime_config_roundtrip_and_defaults() -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!("runtime-config-{unique}.toml"));

        let toml = r#"
[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
rewind_source_host = "127.0.0.1"
rewind_source_port = 5432

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }
"#;

        std::fs::write(&path, toml)?;

        let cfg = load_runtime_config(&path)?;
        assert_eq!(cfg.postgres.connect_timeout_s, 5);
        assert_eq!(cfg.postgres.listen_host, "127.0.0.1");
        assert_eq!(cfg.postgres.listen_port, 5432);
        assert_eq!(
            cfg.postgres.socket_dir,
            PathBuf::from("/tmp/pgtuskmaster/socket")
        );
        assert_eq!(
            cfg.postgres.log_file,
            PathBuf::from("/tmp/pgtuskmaster/postgres.log")
        );
        assert_eq!(cfg.postgres.rewind_source_host, "127.0.0.1");
        assert_eq!(cfg.postgres.rewind_source_port, 5432);
        assert_eq!(cfg.process.pg_rewind_timeout_ms, 120_000);
        assert_eq!(cfg.api.listen_addr, "127.0.0.1:8080");

        let _ = std::fs::remove_file(path);
        Ok(())
    }

    #[test]
    fn load_runtime_config_rejects_invalid_file() -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!("runtime-config-invalid-{unique}.toml"));

        let toml = r#"
[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
rewind_source_host = "127.0.0.1"
rewind_source_port = 5432
unknown = 10

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(err, Err(ConfigError::Parse { .. })));

        let _ = std::fs::remove_file(path);
        Ok(())
    }
}
