use std::path::Path;

use thiserror::Error;

use super::defaults::apply_defaults;
use super::schema::{ConfigVersion, PartialRuntimeConfig, RuntimeConfig, RuntimeConfigV2Input};

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

    #[derive(serde::Deserialize)]
    struct ConfigEnvelope {
        config_version: Option<ConfigVersion>,
    }

    let envelope: ConfigEnvelope =
        toml::from_str(&contents).map_err(|source| ConfigError::Parse {
            path: path.display().to_string(),
            source,
        })?;

    match envelope.config_version.unwrap_or(ConfigVersion::V1) {
        ConfigVersion::V1 => {
            let raw: PartialRuntimeConfig =
                toml::from_str(&contents).map_err(|source| ConfigError::Parse {
                    path: path.display().to_string(),
                    source,
                })?;
            let cfg = apply_defaults(raw);
            validate_runtime_config(&cfg)?;
            Ok(cfg)
        }
        ConfigVersion::V2 => {
            let _: RuntimeConfigV2Input =
                toml::from_str(&contents).map_err(|source| ConfigError::Parse {
                    path: path.display().to_string(),
                    source,
                })?;
            Err(ConfigError::Validation {
                field: "config_version",
                message: "config_version=v2 is recognized but not executable yet (task 02 implements v2 normalization)"
                    .to_string(),
            })
        }
    }
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

    validate_non_empty(
        "postgres.local_conn_identity.user",
        cfg.postgres.local_conn_identity.user.as_str(),
    )?;
    validate_non_empty(
        "postgres.local_conn_identity.dbname",
        cfg.postgres.local_conn_identity.dbname.as_str(),
    )?;
    validate_non_empty(
        "postgres.rewind_conn_identity.user",
        cfg.postgres.rewind_conn_identity.user.as_str(),
    )?;
    validate_non_empty(
        "postgres.rewind_conn_identity.dbname",
        cfg.postgres.rewind_conn_identity.dbname.as_str(),
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

    if let Some(path) = cfg.logging.sinks.file.path.as_ref() {
        validate_non_empty_path("logging.sinks.file.path", path)?;
    }

    if cfg.logging.sinks.file.enabled && cfg.logging.sinks.file.path.is_none() {
        return Err(ConfigError::Validation {
            field: "logging.sinks.file.path",
            message: "must be configured when logging.sinks.file.enabled is true".to_string(),
        });
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

    match &cfg.api.security.auth {
        crate::config::ApiAuthConfig::Disabled => {}
        crate::config::ApiAuthConfig::RoleTokens(tokens) => {
            validate_optional_non_empty(
                "api.security.auth.role_tokens.read_token",
                tokens.read_token.as_deref(),
            )?;
            validate_optional_non_empty(
                "api.security.auth.role_tokens.admin_token",
                tokens.admin_token.as_deref(),
            )?;
        }
    }

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
            ApiAuthConfig, ApiConfig, ApiRoleTokensConfig, ApiSecurityConfig, ApiTlsMode, BinaryPaths,
            ClusterConfig, DcsConfig, DebugConfig, FileSinkConfig, FileSinkMode, HaConfig,
            InlineOrPath, LogCleanupConfig, LogLevel, LoggingConfig, LoggingSinksConfig, PgHbaConfig,
            PgIdentConfig, PostgresConnIdentityConfig, PostgresConfig, PostgresLoggingConfig,
            PostgresRoleConfig, PostgresRolesConfig, ProcessConfig, RoleAuthConfig, RuntimeConfig,
            StderrSinkConfig, TlsServerConfig,
        };
        use crate::pginfo::conninfo::PgSslMode;

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
                    local_conn_identity: PostgresConnIdentityConfig {
                        user: "postgres".to_string(),
                        dbname: "postgres".to_string(),
                        ssl_mode: PgSslMode::Prefer,
                    },
                    rewind_conn_identity: PostgresConnIdentityConfig {
                        user: "postgres".to_string(),
                        dbname: "postgres".to_string(),
                        ssl_mode: PgSslMode::Prefer,
                    },
                    tls: TlsServerConfig {
                        mode: ApiTlsMode::Disabled,
                        identity: None,
                        client_auth: None,
                    },
                    roles: PostgresRolesConfig {
                        superuser: PostgresRoleConfig {
                            username: "postgres".to_string(),
                            auth: RoleAuthConfig::Tls,
                        },
                        replicator: PostgresRoleConfig {
                            username: "replicator".to_string(),
                            auth: RoleAuthConfig::Tls,
                        },
                        rewinder: PostgresRoleConfig {
                            username: "rewinder".to_string(),
                            auth: RoleAuthConfig::Tls,
                        },
                    },
                    pg_hba: PgHbaConfig {
                        source: InlineOrPath::Inline {
                            content: String::new(),
                        },
                    },
                    pg_ident: PgIdentConfig {
                        source: InlineOrPath::Inline {
                            content: String::new(),
                        },
                    },
                },
                dcs: DcsConfig {
                    endpoints: vec!["http://127.0.0.1:2379".to_string()],
                    scope: "scope-a".to_string(),
                    init: None,
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
                sinks: LoggingSinksConfig {
                    stderr: StderrSinkConfig { enabled: true },
                    file: FileSinkConfig {
                        enabled: false,
                        path: None,
                        mode: FileSinkMode::Append,
                    },
                },
                },
                api: ApiConfig {
                    listen_addr: "127.0.0.1:8080".to_string(),
                    security: ApiSecurityConfig {
                        tls: TlsServerConfig {
                            mode: ApiTlsMode::Disabled,
                            identity: None,
                            client_auth: None,
                        },
                        auth: ApiAuthConfig::Disabled,
                    },
                },
                debug: DebugConfig { enabled: false },
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
            cfg.api.security.auth = ApiAuthConfig::RoleTokens(ApiRoleTokensConfig {
                read_token: Some(" ".to_string()),
                admin_token: None,
            });

            let err = validate_runtime_config(&cfg);
            assert!(matches!(
                err,
                Err(ConfigError::Validation {
                    field: "api.security.auth.role_tokens.read_token",
                    ..
                })
            ));

            let mut cfg = base_runtime_config();
            cfg.api.security.auth = ApiAuthConfig::RoleTokens(ApiRoleTokensConfig {
                read_token: None,
                admin_token: Some("\t".to_string()),
            });

            let err = validate_runtime_config(&cfg);
            assert!(matches!(
                err,
                Err(ConfigError::Validation {
                    field: "api.security.auth.role_tokens.admin_token",
                    ..
                })
            ));
        }

    #[test]
    fn validate_runtime_config_rejects_file_sink_enabled_without_path() {
        let mut cfg = base_runtime_config();
        cfg.logging.sinks.file.enabled = true;
        cfg.logging.sinks.file.path = None;

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "logging.sinks.file.path",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_rejects_file_sink_empty_path() {
        let mut cfg = base_runtime_config();
        cfg.logging.sinks.file.enabled = true;
        cfg.logging.sinks.file.path = Some(PathBuf::new());

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "logging.sinks.file.path",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_accepts_file_sink_enabled_with_path() {
        let mut cfg = base_runtime_config();
        cfg.logging.sinks.file.enabled = true;
        cfg.logging.sinks.file.path = Some(PathBuf::from("/tmp/pgtuskmaster.jsonl"));

        assert!(validate_runtime_config(&cfg).is_ok());
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

    #[test]
    fn load_runtime_config_v2_is_recognized_but_fails_closed(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!("runtime-config-v2-{unique}.toml"));

        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
connect_timeout_s = 5
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
rewind_source_host = "127.0.0.1"
rewind_source_port = 5432
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "tls" } }, replicator = { username = "replicator", auth = { type = "tls" } }, rewinder = { username = "rewinder", auth = { type = "tls" } } }
pg_hba = { source = { content = "" } }
pg_ident = { source = { content = "" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
pg_rewind_timeout_ms = 120000
bootstrap_timeout_ms = 300000
fencing_timeout_ms = 30000
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[logging]
level = "info"
capture_subprocess_output = true
postgres = { enabled = true, poll_interval_ms = 200, cleanup = { enabled = true, max_files = 10, max_age_seconds = 60 } }
sinks = { stderr = { enabled = true }, file = { enabled = false, mode = "append" } }

[api]
listen_addr = "127.0.0.1:8080"
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }

[debug]
enabled = false
"#;

        std::fs::write(&path, toml)?;
        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "config_version",
                ..
            })
        ));

        let _ = std::fs::remove_file(path);
        Ok(())
    }
}
