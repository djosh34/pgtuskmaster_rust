use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use thiserror::Error;

use crate::config::{ApiTlsMode, InlineOrPath, RuntimeConfig};

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

    let managed_hba = absolutize_path(&cfg.postgres.data_dir.join("pgtm.pg_hba.conf"))?;
    let managed_ident = absolutize_path(&cfg.postgres.data_dir.join("pgtm.pg_ident.conf"))?;

    let hba_contents = load_inline_or_path_string("postgres.pg_hba.source", &cfg.postgres.pg_hba.source)?;
    let ident_contents =
        load_inline_or_path_string("postgres.pg_ident.source", &cfg.postgres.pg_ident.source)?;

    write_atomic(&managed_hba, hba_contents.as_bytes(), Some(0o644))?;
    write_atomic(&managed_ident, ident_contents.as_bytes(), Some(0o644))?;

    let mut tls_cert_path = None;
    let mut tls_key_path = None;
    let mut tls_client_ca_path = None;

    let mut extra_settings = BTreeMap::new();
    extra_settings.insert(
        "hba_file".to_string(),
        managed_hba.display().to_string(),
    );
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

            let cert_pem =
                load_inline_or_path_bytes("postgres.tls.identity.cert_chain", &identity.cert_chain)?;
            let key_pem =
                load_inline_or_path_bytes("postgres.tls.identity.private_key", &identity.private_key)?;

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
                let ca_pem =
                    load_inline_or_path_bytes("postgres.tls.client_auth.client_ca", &client_auth.client_ca)?;
                let managed_ca = absolutize_path(&cfg.postgres.data_dir.join("pgtm.ca.crt"))?;
                write_atomic(&managed_ca, ca_pem.as_slice(), Some(0o644))?;
                extra_settings.insert(
                    "ssl_ca_file".to_string(),
                    managed_ca.display().to_string(),
                );
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
    let raw = match source {
        InlineOrPath::Path(path) | InlineOrPath::PathConfig { path } => fs::read_to_string(path)
            .map_err(|err| ManagedPostgresError::Io {
                message: format!("failed to read `{field}` from {}: {err}", path.display()),
            })?,
        InlineOrPath::Inline { content } => content.clone(),
    };
    Ok(raw)
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

fn write_atomic(path: &Path, contents: &[u8], mode: Option<u32>) -> Result<(), ManagedPostgresError> {
    let parent = path.parent().ok_or_else(|| ManagedPostgresError::Io {
        message: format!("path has no parent: {}", path.display()),
    })?;
    fs::create_dir_all(parent).map_err(|err| ManagedPostgresError::Io {
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
            fs::rename(&tmp, path).map_err(|err| ManagedPostgresError::Io {
                message: format!("failed to rename {} to {}: {err}", tmp.display(), path.display()),
            })
        } else {
            Err(ManagedPostgresError::Io {
                message: format!("failed to rename {} to {}: {err}", tmp.display(), path.display()),
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
            DcsConfig, DebugConfig, HaConfig, InlineOrPath, LogCleanupConfig, LogLevel,
            LoggingConfig, PgHbaConfig, PgIdentConfig, PostgresConnIdentityConfig, PostgresConfig,
            PostgresLoggingConfig, PostgresRoleConfig, PostgresRolesConfig, ProcessConfig,
            RoleAuthConfig, RuntimeConfig, StderrSinkConfig, TlsClientAuthConfig, TlsServerConfig,
            TlsServerIdentityConfig,
        },
        pginfo::conninfo::PgSslMode,
        test_harness::tls::build_adversarial_tls_fixture,
    };

    use super::materialize_managed_postgres_config;

    type BoxError = Box<dyn std::error::Error + Send + Sync>;
    type TestResult = Result<(), BoxError>;

    fn temp_dir(label: &str) -> PathBuf {
        let unique = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(value) => value.as_nanos(),
            Err(_) => 0,
        };
        std::env::temp_dir().join(format!(
            "pgtuskmaster-managed-{label}-{unique}-{}",
            std::process::id()
        ))
    }

    fn sample_runtime_config(data_dir: PathBuf, tls: TlsServerConfig) -> RuntimeConfig {
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
                    user: "rewinder".to_string(),
                    dbname: "postgres".to_string(),
                    ssl_mode: PgSslMode::Prefer,
                },
                tls,
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
                        content: concat!(
                            "local all all trust\n",
                            "host all all 127.0.0.1/32 trust\n",
                        )
                        .to_string(),
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
                    archive_command_log_file: None,
                    poll_interval_ms: 200,
                    cleanup: LogCleanupConfig {
                        enabled: true,
                        max_files: 10,
                        max_age_seconds: 60,
                    },
                },
                sinks: crate::config::LoggingSinksConfig {
                    stderr: StderrSinkConfig { enabled: true },
                    file: crate::config::FileSinkConfig {
                        enabled: false,
                        path: None,
                        mode: crate::config::FileSinkMode::Append,
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
    fn materialize_writes_hba_ident_and_disables_ssl_when_tls_disabled() -> TestResult {
        let dir = temp_dir("plain");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir)?;

        let cfg = sample_runtime_config(
            dir.clone(),
            TlsServerConfig {
                mode: ApiTlsMode::Disabled,
                identity: None,
                client_auth: None,
            },
        );
        let managed = materialize_managed_postgres_config(&cfg)
            .map_err(|err| std::io::Error::other(err.to_string()))?;

        if !managed.hba_path.exists() {
            return Err(Box::new(std::io::Error::other("hba file missing")));
        }
        if !managed.ident_path.exists() {
            return Err(Box::new(std::io::Error::other("ident file missing")));
        }

        let hba_on_disk = fs::read_to_string(&managed.hba_path)?;
        if !hba_on_disk.contains("host all all 127.0.0.1/32 trust") {
            return Err(Box::new(std::io::Error::other(
                "unexpected hba contents",
            )));
        }

        if managed.extra_settings.get("ssl").map(|s| s.as_str()) != Some("off") {
            return Err(Box::new(std::io::Error::other(
                "expected ssl=off in extra settings",
            )));
        }

        let _ = fs::remove_dir_all(&dir);
        Ok(())
    }

    #[test]
    fn materialize_writes_tls_material_and_settings_when_enabled() -> TestResult {
        let fixture = build_adversarial_tls_fixture()?;

        let dir = temp_dir("tls");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir)?;

        let cfg = sample_runtime_config(
            dir.clone(),
            TlsServerConfig {
                mode: ApiTlsMode::Required,
                identity: Some(TlsServerIdentityConfig {
                    cert_chain: InlineOrPath::Inline {
                        content: fixture.valid_server.cert_pem.clone(),
                    },
                    private_key: InlineOrPath::Inline {
                        content: fixture.valid_server.key_pem.clone(),
                    },
                }),
                client_auth: Some(TlsClientAuthConfig {
                    client_ca: InlineOrPath::Inline {
                        content: fixture.trusted_client_ca.cert.cert_pem.clone(),
                    },
                    require_client_cert: false,
                }),
            },
        );

        let managed = materialize_managed_postgres_config(&cfg)
            .map_err(|err| std::io::Error::other(err.to_string()))?;

        if managed.extra_settings.get("ssl").map(|s| s.as_str()) != Some("on") {
            return Err(Box::new(std::io::Error::other(
                "expected ssl=on in extra settings",
            )));
        }
        let cert_file = managed
            .tls_cert_path
            .ok_or_else(|| std::io::Error::other("missing tls cert path"))?;
        let key_file = managed
            .tls_key_path
            .ok_or_else(|| std::io::Error::other("missing tls key path"))?;
        let ca_file = managed
            .tls_client_ca_path
            .ok_or_else(|| std::io::Error::other("missing tls ca path"))?;

        if !cert_file.exists() || !key_file.exists() || !ca_file.exists() {
            return Err(Box::new(std::io::Error::other("tls files missing")));
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(&key_file)?.permissions().mode() & 0o777;
            if mode != 0o600 {
                return Err(Box::new(std::io::Error::other(format!(
                    "expected key file mode 0600, got {:o}",
                    mode
                ))));
            }
        }

        let _ = fs::remove_dir_all(&dir);
        Ok(())
    }
}
