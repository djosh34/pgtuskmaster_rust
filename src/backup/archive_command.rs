use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    backup::{pgbackrest, ArchiveGetInput, ArchivePushInput, BackupOperation},
    config::{BackupProvider, RuntimeConfig},
};

const ARCHIVE_COMMAND_CONFIG_NAME: &str = "pgtm.pgbackrest.archive.json";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ArchiveCommandConfig {
    pub(crate) pgbackrest_bin: PathBuf,
    pub(crate) stanza: String,
    pub(crate) repo: String,
    pub(crate) pg1_path: PathBuf,
    pub(crate) archive_push_options: Vec<String>,
    pub(crate) archive_get_options: Vec<String>,
    pub(crate) api_local_addr: String,
    pub(crate) api_token: Option<String>,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(crate) enum ArchiveCommandError {
    #[error("io error: {message}")]
    Io { message: String },
    #[error("invalid config: {message}")]
    InvalidConfig { message: String },
    #[error("decode error: {message}")]
    Decode { message: String },
    #[error("pgbackrest render error: {message}")]
    Render { message: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RenderedCommand {
    pub(crate) program: PathBuf,
    pub(crate) args: Vec<String>,
}

pub(crate) fn archive_command_config_path(pgdata: &Path) -> PathBuf {
    pgdata.join(ARCHIVE_COMMAND_CONFIG_NAME)
}

pub(crate) fn materialize_archive_command_config(
    cfg: &RuntimeConfig,
) -> Result<PathBuf, ArchiveCommandError> {
    if !cfg.backup.enabled {
        return Err(ArchiveCommandError::InvalidConfig {
            message: "backup.enabled must be true to materialize archive command config"
                .to_string(),
        });
    }
    if cfg.postgres.data_dir.as_os_str().is_empty() {
        return Err(ArchiveCommandError::InvalidConfig {
            message: "postgres.data_dir must not be empty".to_string(),
        });
    }
    match cfg.backup.provider {
        BackupProvider::Pgbackrest => {}
    }

    let pgbackrest_bin = cfg.process.binaries.pgbackrest.clone().ok_or_else(|| {
        ArchiveCommandError::InvalidConfig {
            message: "process.binaries.pgbackrest must be configured when backup.enabled is true"
                .to_string(),
        }
    })?;
    if pgbackrest_bin.as_os_str().is_empty() {
        return Err(ArchiveCommandError::InvalidConfig {
            message: "process.binaries.pgbackrest must not be empty".to_string(),
        });
    }

    let pg_cfg =
        cfg.backup
            .pgbackrest
            .as_ref()
            .ok_or_else(|| ArchiveCommandError::InvalidConfig {
                message: "backup.pgbackrest must be configured when backup.enabled is true"
                    .to_string(),
            })?;

    let stanza = pg_cfg
        .stanza
        .clone()
        .ok_or_else(|| ArchiveCommandError::InvalidConfig {
            message: "backup.pgbackrest.stanza must be configured when backup.enabled is true"
                .to_string(),
        })?;
    if stanza.trim().is_empty() {
        return Err(ArchiveCommandError::InvalidConfig {
            message: "backup.pgbackrest.stanza must not be empty".to_string(),
        });
    }

    let repo = pg_cfg
        .repo
        .clone()
        .ok_or_else(|| ArchiveCommandError::InvalidConfig {
            message: "backup.pgbackrest.repo must be configured when backup.enabled is true"
                .to_string(),
        })?;
    if repo.trim().is_empty() {
        return Err(ArchiveCommandError::InvalidConfig {
            message: "backup.pgbackrest.repo must not be empty".to_string(),
        });
    }

    let api_local_addr = derive_api_local_addr(cfg.api.listen_addr.as_str())?;
    let api_token = select_api_token(cfg);

    let cfg_path = archive_command_config_path(&cfg.postgres.data_dir);
    validate_absolute_path("postgres.data_dir", cfg.postgres.data_dir.as_path())?;
    let config = ArchiveCommandConfig {
        pgbackrest_bin: validate_absolute_path(
            "process.binaries.pgbackrest",
            pgbackrest_bin.as_path(),
        )?,
        stanza,
        repo,
        pg1_path: validate_absolute_path("postgres.data_dir", cfg.postgres.data_dir.as_path())?,
        archive_push_options: pg_cfg.options.archive_push.clone(),
        archive_get_options: pg_cfg.options.archive_get.clone(),
        api_local_addr,
        api_token,
    };

    let json = serde_json::to_vec(&config).map_err(|err| ArchiveCommandError::Decode {
        message: format!("failed to serialize archive command config: {err}"),
    })?;
    write_atomic(cfg_path.as_path(), &json, Some(0o600))?;
    Ok(cfg_path)
}

pub(crate) fn render_archive_push_from_pgdata(
    pgdata: &Path,
    wal_path: &str,
) -> Result<RenderedCommand, ArchiveCommandError> {
    if pgdata.as_os_str().is_empty() {
        return Err(ArchiveCommandError::InvalidConfig {
            message: "pgdata must not be empty".to_string(),
        });
    }
    if wal_path.trim().is_empty() {
        return Err(ArchiveCommandError::InvalidConfig {
            message: "wal_path must not be empty".to_string(),
        });
    }
    let config = load_config(pgdata)?;
    let template = pgbackrest::render(BackupOperation::ArchivePush(ArchivePushInput {
        stanza: config.stanza.clone(),
        repo: config.repo.clone(),
        pg1_path: config.pg1_path.clone(),
        wal_path: wal_path.to_string(),
        options: config.archive_push_options.clone(),
    }))
    .map_err(|err| ArchiveCommandError::Render { message: err })?;
    Ok(RenderedCommand {
        program: config.pgbackrest_bin,
        args: template.args,
    })
}

pub(crate) fn render_archive_get_from_pgdata(
    pgdata: &Path,
    wal_segment: &str,
    destination_path: &str,
) -> Result<RenderedCommand, ArchiveCommandError> {
    if pgdata.as_os_str().is_empty() {
        return Err(ArchiveCommandError::InvalidConfig {
            message: "pgdata must not be empty".to_string(),
        });
    }
    if wal_segment.trim().is_empty() {
        return Err(ArchiveCommandError::InvalidConfig {
            message: "wal_segment must not be empty".to_string(),
        });
    }
    if destination_path.trim().is_empty() {
        return Err(ArchiveCommandError::InvalidConfig {
            message: "destination_path must not be empty".to_string(),
        });
    }
    let config = load_config(pgdata)?;
    let template = pgbackrest::render(BackupOperation::ArchiveGet(ArchiveGetInput {
        stanza: config.stanza.clone(),
        repo: config.repo.clone(),
        pg1_path: config.pg1_path.clone(),
        wal_segment: wal_segment.to_string(),
        destination_path: destination_path.to_string(),
        options: config.archive_get_options.clone(),
    }))
    .map_err(|err| ArchiveCommandError::Render { message: err })?;
    Ok(RenderedCommand {
        program: config.pgbackrest_bin,
        args: template.args,
    })
}

pub(crate) fn load_archive_command_config(
    pgdata: &Path,
) -> Result<ArchiveCommandConfig, ArchiveCommandError> {
    load_config(pgdata)
}

fn load_config(pgdata: &Path) -> Result<ArchiveCommandConfig, ArchiveCommandError> {
    let path = archive_command_config_path(pgdata);
    let raw = fs::read(&path).map_err(|err| ArchiveCommandError::Io {
        message: format!(
            "failed to read archive command config {}: {err}",
            path.display()
        ),
    })?;
    serde_json::from_slice(&raw).map_err(|err| ArchiveCommandError::Decode {
        message: format!(
            "failed to decode archive command config {}: {err}",
            path.display()
        ),
    })
}

fn validate_absolute_path(
    field: &'static str,
    path: &Path,
) -> Result<PathBuf, ArchiveCommandError> {
    if path.as_os_str().is_empty() {
        return Err(ArchiveCommandError::InvalidConfig {
            message: format!("{field} must not be empty"),
        });
    }
    if !path.is_absolute() {
        return Err(ArchiveCommandError::InvalidConfig {
            message: format!(
                "{field} must be an absolute path (got `{}`)",
                path.display()
            ),
        });
    }
    Ok(path.to_path_buf())
}

fn derive_api_local_addr(listen_addr: &str) -> Result<String, ArchiveCommandError> {
    let parsed = listen_addr.parse::<std::net::SocketAddr>();
    let port = match parsed {
        Ok(addr) => addr.port(),
        Err(_) => {
            let (_host, port) =
                listen_addr
                    .rsplit_once(':')
                    .ok_or_else(|| ArchiveCommandError::InvalidConfig {
                        message: format!("api.listen_addr must be host:port (got `{listen_addr}`)"),
                    })?;
            port.parse::<u16>()
                .map_err(|err| ArchiveCommandError::InvalidConfig {
                    message: format!(
                        "api.listen_addr port must be a valid u16 (got `{listen_addr}`): {err}"
                    ),
                })?
        }
    };
    if port == 0 {
        return Err(ArchiveCommandError::InvalidConfig {
            message: "api.listen_addr port must not be 0 for wal helper event emission".to_string(),
        });
    }
    Ok(format!("127.0.0.1:{port}"))
}

fn select_api_token(cfg: &RuntimeConfig) -> Option<String> {
    match &cfg.api.security.auth {
        crate::config::ApiAuthConfig::Disabled => None,
        crate::config::ApiAuthConfig::RoleTokens(tokens) => tokens
            .read_token
            .as_deref()
            .or(tokens.admin_token.as_deref())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| value.to_string()),
    }
}

fn now_millis() -> Result<u64, ArchiveCommandError> {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| ArchiveCommandError::Io {
            message: format!("system clock before unix epoch: {err}"),
        })?;
    u64::try_from(elapsed.as_millis()).map_err(|err| ArchiveCommandError::Io {
        message: format!("millis conversion failed: {err}"),
    })
}

fn write_atomic(
    path: &Path,
    contents: &[u8],
    mode: Option<u32>,
) -> Result<(), ArchiveCommandError> {
    let parent = path.parent().ok_or_else(|| ArchiveCommandError::Io {
        message: format!("path has no parent: {}", path.display()),
    })?;
    fs::create_dir_all(parent).map_err(|err| ArchiveCommandError::Io {
        message: format!("failed to create dir {}: {err}", parent.display()),
    })?;

    let (pid, millis) = (std::process::id(), now_millis()?);
    let file_name = match path.file_name().and_then(|s| s.to_str()) {
        Some(name) if !name.is_empty() => name,
        _ => "managed",
    };
    let tmp = parent.join(format!(".{file_name}.tmp.{pid}.{millis}"));

    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&tmp)
        .map_err(|err| ArchiveCommandError::Io {
            message: format!("failed to create temp file {}: {err}", tmp.display()),
        })?;

    use std::io::Write;
    file.write_all(contents)
        .map_err(|err| ArchiveCommandError::Io {
            message: format!("failed to write temp file {}: {err}", tmp.display()),
        })?;
    file.sync_all().map_err(|err| ArchiveCommandError::Io {
        message: format!("failed to sync temp file {}: {err}", tmp.display()),
    })?;

    #[cfg(unix)]
    if let Some(mode) = mode {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&tmp, fs::Permissions::from_mode(mode)).map_err(|err| {
            ArchiveCommandError::Io {
                message: format!("failed to set permissions on {}: {err}", tmp.display()),
            }
        })?;
    }

    fs::rename(&tmp, path).or_else(|err| {
        if path.exists() {
            fs::remove_file(path).map_err(|remove_err| ArchiveCommandError::Io {
                message: format!(
                    "failed to remove existing {} after rename error ({err}): {remove_err}",
                    path.display()
                ),
            })?;
            fs::rename(&tmp, path).map_err(|err| ArchiveCommandError::Io {
                message: format!(
                    "failed to rename {} to {}: {err}",
                    tmp.display(),
                    path.display()
                ),
            })
        } else {
            Err(ArchiveCommandError::Io {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    use crate::config::{
        ApiAuthConfig, ApiConfig, ApiSecurityConfig, ApiTlsMode, BackupConfig, BackupOptions,
        BinaryPaths, ClusterConfig, DcsConfig, DebugConfig, FileSinkConfig, FileSinkMode, HaConfig,
        InlineOrPath, LogCleanupConfig, LogLevel, LoggingConfig, LoggingSinksConfig,
        PgBackRestConfig, PgHbaConfig, PgIdentConfig, PostgresConfig, PostgresConnIdentityConfig,
        PostgresLoggingConfig, PostgresRoleConfig, PostgresRolesConfig, ProcessConfig,
        RoleAuthConfig, RuntimeConfig, StderrSinkConfig, TlsServerConfig,
    };
    use crate::pginfo::conninfo::PgSslMode;

    fn unique_temp_root(label: &str) -> PathBuf {
        let pid = std::process::id();
        let nanos = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(d) => d.as_nanos(),
            Err(_) => 0,
        };
        std::env::temp_dir().join(format!("pgtuskmaster-{label}-{pid}-{nanos}"))
    }

    fn sample_runtime_config(data_dir: PathBuf) -> RuntimeConfig {
        RuntimeConfig {
            cluster: ClusterConfig {
                name: "cluster-a".to_string(),
                member_id: "node-a".to_string(),
            },
            postgres: PostgresConfig {
                data_dir: data_dir.clone(),
                connect_timeout_s: 5,
                listen_host: "127.0.0.1".to_string(),
                listen_port: 5432,
                socket_dir: data_dir.join("socket"),
                log_file: data_dir.join("postgres.log"),
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
                loop_interval_ms: 1000,
                lease_ttl_ms: 10_000,
            },
            process: ProcessConfig {
                pg_rewind_timeout_ms: 1000,
                bootstrap_timeout_ms: 1000,
                fencing_timeout_ms: 1000,
                backup_timeout_ms: 1000,
                binaries: BinaryPaths {
                    postgres: "/usr/bin/postgres".into(),
                    pg_ctl: "/usr/bin/pg_ctl".into(),
                    pg_rewind: "/usr/bin/pg_rewind".into(),
                    initdb: "/usr/bin/initdb".into(),
                    pg_basebackup: "/usr/bin/pg_basebackup".into(),
                    psql: "/usr/bin/psql".into(),
                    pgbackrest: Some("/usr/bin/pgbackrest".into()),
                },
            },
            backup: BackupConfig {
                enabled: true,
                provider: BackupProvider::Pgbackrest,
                bootstrap: crate::config::BackupBootstrapConfig {
                    enabled: false,
                    takeover_policy: Default::default(),
                    recovery_mode: Default::default(),
                },
                pgbackrest: Some(PgBackRestConfig {
                    stanza: Some("stanza-a".to_string()),
                    repo: Some("1".to_string()),
                    options: BackupOptions::default(),
                }),
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
            debug: DebugConfig { enabled: false },
        }
    }

    #[test]
    fn config_path_is_under_pgdata() {
        let pgdata = PathBuf::from("/tmp/pgdata-a");
        let path = archive_command_config_path(pgdata.as_path());
        assert!(path.ends_with(ARCHIVE_COMMAND_CONFIG_NAME));
    }

    #[test]
    fn render_errors_when_config_missing() {
        let pgdata = std::env::temp_dir().join(format!(
            "pgtuskmaster-archive-cfg-missing-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&pgdata);
        let _ = fs::create_dir_all(&pgdata);

        let result = render_archive_push_from_pgdata(pgdata.as_path(), "/tmp/wal");
        assert!(result.is_err());
        if let Err(err) = result {
            let msg = err.to_string();
            assert!(
                msg.contains(ARCHIVE_COMMAND_CONFIG_NAME),
                "expected error to reference config file name, got: {msg}"
            );
        }

        let _ = fs::remove_dir_all(&pgdata);
    }

    #[test]
    fn materialize_rejects_relative_pgbackrest_bin() {
        let root = unique_temp_root("archive-command-config");
        let _ = fs::remove_dir_all(&root);
        let _ = fs::create_dir_all(&root);

        let mut cfg = sample_runtime_config(root.clone());
        cfg.process.binaries.pgbackrest = Some(PathBuf::from("pgbackrest"));
        let result = materialize_archive_command_config(&cfg);
        assert!(result.is_err());

        let _ = fs::remove_dir_all(&root);
    }
}
