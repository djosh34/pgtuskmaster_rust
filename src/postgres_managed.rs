use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use thiserror::Error;

use crate::config::{ApiTlsMode, InlineOrPath, RuntimeConfig};

const MANAGED_PG_HBA_CONF_NAME: &str = "pgtm.pg_hba.conf";
const MANAGED_PG_IDENT_CONF_NAME: &str = "pgtm.pg_ident.conf";
const MANAGED_POSTGRESQL_CONF_NAME: &str = "pgtm.postgresql.conf";
const MANAGED_POSTGRESQL_CONF_HEADER: &str = "\
# This file is managed by pgtuskmaster.\n\
# Backup-era archive and restore settings have been removed.\n";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ManagedPostgresConfig {
    pub(crate) hba_path: PathBuf,
    pub(crate) ident_path: PathBuf,
    pub(crate) tls_cert_path: Option<PathBuf>,
    pub(crate) tls_key_path: Option<PathBuf>,
    pub(crate) tls_client_ca_path: Option<PathBuf>,
    pub(crate) extra_settings: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(crate) enum ManagedPostgresError {
    #[error("io error: {message}")]
    Io { message: String },
    #[error("invalid config: {message}")]
    InvalidConfig { message: String },
}

pub(crate) fn materialize_managed_postgres_config(
    cfg: &RuntimeConfig,
) -> Result<ManagedPostgresConfig, ManagedPostgresError> {
    let data_dir = cfg.postgres.data_dir.as_path();
    if data_dir.as_os_str().is_empty() {
        return Err(ManagedPostgresError::InvalidConfig {
            message: "postgres.data_dir must not be empty".to_string(),
        });
    }

    let managed_hba = absolutize_path(&cfg.postgres.data_dir.join(MANAGED_PG_HBA_CONF_NAME))?;
    let managed_ident = absolutize_path(&cfg.postgres.data_dir.join(MANAGED_PG_IDENT_CONF_NAME))?;
    let managed_postgresql_conf =
        absolutize_path(&cfg.postgres.data_dir.join(MANAGED_POSTGRESQL_CONF_NAME))?;

    let hba_contents =
        load_inline_or_path_string("postgres.pg_hba.source", &cfg.postgres.pg_hba.source)?;
    let ident_contents =
        load_inline_or_path_string("postgres.pg_ident.source", &cfg.postgres.pg_ident.source)?;

    write_atomic(&managed_hba, hba_contents.as_bytes(), Some(0o644))?;
    write_atomic(&managed_ident, ident_contents.as_bytes(), Some(0o644))?;
    write_atomic(
        &managed_postgresql_conf,
        MANAGED_POSTGRESQL_CONF_HEADER.as_bytes(),
        Some(0o644),
    )?;

    let mut tls_cert_path = None;
    let mut tls_key_path = None;
    let mut tls_client_ca_path = None;

    let mut extra_settings = BTreeMap::new();
    extra_settings.insert("hba_file".to_string(), managed_hba.display().to_string());
    extra_settings.insert(
        "ident_file".to_string(),
        managed_ident.display().to_string(),
    );

    match cfg.postgres.tls.mode {
        ApiTlsMode::Disabled => {
            extra_settings.insert("ssl".to_string(), "off".to_string());
        }
        ApiTlsMode::Optional | ApiTlsMode::Required => {
            extra_settings.insert("ssl".to_string(), "on".to_string());

            let identity = cfg.postgres.tls.identity.as_ref().ok_or_else(|| {
                ManagedPostgresError::InvalidConfig {
                    message:
                        "postgres.tls.identity must be configured when postgres.tls.mode is optional or required"
                            .to_string(),
                }
            })?;

            let cert_pem = load_inline_or_path_bytes(
                "postgres.tls.identity.cert_chain",
                &identity.cert_chain,
            )?;
            let key_pem = load_inline_or_path_bytes(
                "postgres.tls.identity.private_key",
                &identity.private_key,
            )?;

            let managed_cert = absolutize_path(&cfg.postgres.data_dir.join("pgtm.server.crt"))?;
            let managed_key = absolutize_path(&cfg.postgres.data_dir.join("pgtm.server.key"))?;
            write_atomic(&managed_cert, cert_pem.as_slice(), Some(0o644))?;
            write_atomic(&managed_key, key_pem.as_slice(), Some(0o600))?;

            extra_settings.insert(
                "ssl_cert_file".to_string(),
                managed_cert.display().to_string(),
            );
            extra_settings.insert(
                "ssl_key_file".to_string(),
                managed_key.display().to_string(),
            );

            tls_cert_path = Some(managed_cert);
            tls_key_path = Some(managed_key);

            if let Some(client_auth) = cfg.postgres.tls.client_auth.as_ref() {
                let ca_pem = load_inline_or_path_bytes(
                    "postgres.tls.client_auth.client_ca",
                    &client_auth.client_ca,
                )?;
                let managed_ca = absolutize_path(&cfg.postgres.data_dir.join("pgtm.ca.crt"))?;
                write_atomic(&managed_ca, ca_pem.as_slice(), Some(0o644))?;
                extra_settings.insert("ssl_ca_file".to_string(), managed_ca.display().to_string());
                tls_client_ca_path = Some(managed_ca);
            }
        }
    }

    Ok(ManagedPostgresConfig {
        hba_path: managed_hba,
        ident_path: managed_ident,
        tls_cert_path,
        tls_key_path,
        tls_client_ca_path,
        extra_settings,
    })
}

fn load_inline_or_path_string(
    field: &str,
    source: &InlineOrPath,
) -> Result<String, ManagedPostgresError> {
    match source {
        InlineOrPath::Path(path) | InlineOrPath::PathConfig { path } => fs::read_to_string(path)
            .map_err(|err| ManagedPostgresError::Io {
                message: format!("failed to read `{field}` from {}: {err}", path.display()),
            }),
        InlineOrPath::Inline { content } => Ok(content.clone()),
    }
}

fn load_inline_or_path_bytes(
    field: &str,
    source: &InlineOrPath,
) -> Result<Vec<u8>, ManagedPostgresError> {
    match source {
        InlineOrPath::Path(path) | InlineOrPath::PathConfig { path } => {
            fs::read(path).map_err(|err| ManagedPostgresError::Io {
                message: format!("failed to read `{field}` from {}: {err}", path.display()),
            })
        }
        InlineOrPath::Inline { content } => Ok(content.as_bytes().to_vec()),
    }
}

fn absolutize_path(path: &Path) -> Result<PathBuf, ManagedPostgresError> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    let cwd = std::env::current_dir().map_err(|err| ManagedPostgresError::Io {
        message: format!("failed to read current_dir: {err}"),
    })?;
    Ok(cwd.join(path))
}

fn write_atomic(
    path: &Path,
    contents: &[u8],
    mode: Option<u32>,
) -> Result<(), ManagedPostgresError> {
    let parent = path.parent().ok_or_else(|| ManagedPostgresError::Io {
        message: format!("path has no parent: {}", path.display()),
    })?;
    fs::create_dir_all(parent).map_err(|err| ManagedPostgresError::Io {
        message: format!("failed to create dir {}: {err}", parent.display()),
    })?;

    let pid = std::process::id();
    let millis = now_millis()?;
    let file_name = match path.file_name().and_then(|value| value.to_str()) {
        Some(name) if !name.is_empty() => name,
        _ => "managed",
    };
    let tmp = parent.join(format!(".{file_name}.tmp.{pid}.{millis}"));

    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&tmp)
        .map_err(|err| ManagedPostgresError::Io {
            message: format!("failed to create temp file {}: {err}", tmp.display()),
        })?;

    use std::io::Write;
    file.write_all(contents)
        .map_err(|err| ManagedPostgresError::Io {
            message: format!("failed to write temp file {}: {err}", tmp.display()),
        })?;
    file.sync_all().map_err(|err| ManagedPostgresError::Io {
        message: format!("failed to sync temp file {}: {err}", tmp.display()),
    })?;

    #[cfg(unix)]
    if let Some(mode) = mode {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&tmp, fs::Permissions::from_mode(mode)).map_err(|err| {
            ManagedPostgresError::Io {
                message: format!("failed to set permissions on {}: {err}", tmp.display()),
            }
        })?;
    }

    fs::rename(&tmp, path).or_else(|err| {
        if path.exists() {
            fs::remove_file(path).map_err(|remove_err| ManagedPostgresError::Io {
                message: format!(
                    "failed to remove existing {} after rename error ({err}): {remove_err}",
                    path.display()
                ),
            })?;
            fs::rename(&tmp, path).map_err(|rename_err| ManagedPostgresError::Io {
                message: format!(
                    "failed to rename {} to {}: {rename_err}",
                    tmp.display(),
                    path.display()
                ),
            })
        } else {
            Err(ManagedPostgresError::Io {
                message: format!(
                    "failed to rename {} to {}: {err}",
                    tmp.display(),
                    path.display()
                ),
            })
        }
    })?;

    Ok(())
}

fn now_millis() -> Result<u128, ManagedPostgresError> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| ManagedPostgresError::Io {
            message: format!("clock error: {err}"),
        })?;
    Ok(duration.as_millis())
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use crate::{
        config::{
            ApiAuthConfig, ApiConfig, ApiSecurityConfig, ApiTlsMode, BinaryPaths, ClusterConfig,
            DcsConfig, DebugConfig, FileSinkConfig, FileSinkMode, HaConfig, InlineOrPath,
            LogCleanupConfig, LogLevel, LoggingConfig, LoggingSinksConfig, PgHbaConfig,
            PgIdentConfig, PostgresConfig, PostgresConnIdentityConfig, PostgresLoggingConfig,
            PostgresRoleConfig, PostgresRolesConfig, ProcessConfig, RoleAuthConfig, RuntimeConfig,
            StderrSinkConfig, TlsServerConfig,
        },
        pginfo::conninfo::PgSslMode,
    };

    use super::materialize_managed_postgres_config;

    #[test]
    fn materialize_managed_postgres_config_creates_backup_free_postgresql_conf(
    ) -> Result<(), String> {
        let data_dir = unique_test_data_dir("postgresql-conf");
        let cfg = sample_runtime_config(data_dir.clone());

        let _managed = materialize_managed_postgres_config(&cfg)
            .map_err(|err| format!("materialize managed config failed: {err}"))?;

        let managed_postgresql_conf = data_dir.join(super::MANAGED_POSTGRESQL_CONF_NAME);
        let postgresql_conf = fs::read_to_string(&managed_postgresql_conf).map_err(|err| {
            format!(
                "read managed postgresql conf {} failed: {err}",
                managed_postgresql_conf.display()
            )
        })?;

        if !postgresql_conf.starts_with(super::MANAGED_POSTGRESQL_CONF_HEADER) {
            return Err(format!(
                "managed postgresql conf missing expected header: {:?}",
                postgresql_conf
            ));
        }
        if postgresql_conf.contains("archive_mode")
            || postgresql_conf.contains("archive_command")
            || postgresql_conf.contains("restore_command")
        {
            return Err(format!(
                "managed postgresql conf unexpectedly contains backup settings: {:?}",
                postgresql_conf
            ));
        }

        fs::remove_dir_all(&data_dir)
            .map_err(|err| format!("remove temp dir {} failed: {err}", data_dir.display()))?;
        Ok(())
    }

    #[test]
    fn materialize_managed_postgres_config_does_not_add_config_file_override() -> Result<(), String>
    {
        let data_dir = unique_test_data_dir("extra-settings");
        let cfg = sample_runtime_config(data_dir.clone());

        let managed = materialize_managed_postgres_config(&cfg)
            .map_err(|err| format!("materialize managed config failed: {err}"))?;

        if managed.extra_settings.contains_key("config_file") {
            return Err(format!(
                "managed postgres settings unexpectedly contain config_file: {:?}",
                managed.extra_settings
            ));
        }

        fs::remove_dir_all(&data_dir)
            .map_err(|err| format!("remove temp dir {} failed: {err}", data_dir.display()))?;
        Ok(())
    }

    fn unique_test_data_dir(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "pgtuskmaster-postgres-managed-{label}-{}-{}",
            std::process::id(),
            crate::logging::system_now_unix_millis()
        ))
    }

    fn sample_runtime_config(data_dir: PathBuf) -> RuntimeConfig {
        RuntimeConfig {
            cluster: ClusterConfig {
                name: "cluster-a".to_string(),
                member_id: "node-a".to_string(),
            },
            postgres: PostgresConfig {
                data_dir,
                connect_timeout_s: 5,
                listen_host: "127.0.0.1".to_string(),
                listen_port: 5432,
                socket_dir: "/tmp/pgtuskmaster/socket".into(),
                log_file: "/tmp/pgtuskmaster/postgres.log".into(),
                rewind_source_host: "127.0.0.1".to_string(),
                rewind_source_port: 5432,
                local_conn_identity: PostgresConnIdentityConfig {
                    user: "postgres".to_string(),
                    dbname: "postgres".to_string(),
                    ssl_mode: PgSslMode::Prefer,
                },
                rewind_conn_identity: PostgresConnIdentityConfig {
                    user: "rewinder".to_string(),
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
                        content: "local all all trust\n".to_string(),
                    },
                },
                pg_ident: PgIdentConfig {
                    source: InlineOrPath::Inline {
                        content: "# empty\n".to_string(),
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
                pg_rewind_timeout_ms: 1_000,
                bootstrap_timeout_ms: 1_000,
                fencing_timeout_ms: 1_000,
                binaries: BinaryPaths {
                    postgres: "/usr/bin/postgres".into(),
                    pg_ctl: "/usr/bin/pg_ctl".into(),
                    pg_rewind: "/usr/bin/pg_rewind".into(),
                    initdb: "/usr/bin/initdb".into(),
                    pg_basebackup: "/usr/bin/pg_basebackup".into(),
                    psql: "/usr/bin/psql".into(),
                },
            },
            logging: LoggingConfig {
                level: LogLevel::Info,
                capture_subprocess_output: true,
                postgres: PostgresLoggingConfig {
                    enabled: true,
                    pg_ctl_log_file: None,
                    log_dir: None,
                    poll_interval_ms: 200,
                    cleanup: LogCleanupConfig {
                        enabled: true,
                        max_files: 10,
                        max_age_seconds: 60,
                        protect_recent_seconds: 300,
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
            debug: DebugConfig { enabled: true },
        }
    }
}
