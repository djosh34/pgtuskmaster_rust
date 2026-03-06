use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use thiserror::Error;

use crate::{
    config::{ApiTlsMode, InlineOrPath, RuntimeConfig},
    postgres_managed_conf::{
        render_managed_postgres_conf, ManagedPostgresConf, ManagedPostgresConfError,
        ManagedPostgresStartIntent, ManagedPostgresTlsConfig, MANAGED_POSTGRESQL_CONF_NAME,
        MANAGED_STANDBY_SIGNAL_NAME,
    },
};

const MANAGED_PG_HBA_CONF_NAME: &str = "pgtm.pg_hba.conf";
const MANAGED_PG_IDENT_CONF_NAME: &str = "pgtm.pg_ident.conf";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ManagedPostgresConfig {
    pub(crate) postgresql_conf_path: PathBuf,
    pub(crate) hba_path: PathBuf,
    pub(crate) ident_path: PathBuf,
    pub(crate) tls_cert_path: Option<PathBuf>,
    pub(crate) tls_key_path: Option<PathBuf>,
    pub(crate) tls_client_ca_path: Option<PathBuf>,
    pub(crate) standby_signal_path: PathBuf,
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
    start_intent: &ManagedPostgresStartIntent,
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
    let standby_signal = absolutize_path(&cfg.postgres.data_dir.join(MANAGED_STANDBY_SIGNAL_NAME))?;

    let hba_contents =
        load_inline_or_path_string("postgres.pg_hba.source", &cfg.postgres.pg_hba.source)?;
    let ident_contents =
        load_inline_or_path_string("postgres.pg_ident.source", &cfg.postgres.pg_ident.source)?;

    write_atomic(&managed_hba, hba_contents.as_bytes(), Some(0o644))?;
    write_atomic(&managed_ident, ident_contents.as_bytes(), Some(0o644))?;

    let tls_files = materialize_tls_files(cfg)?;
    let managed_conf = ManagedPostgresConf {
        listen_addresses: cfg.postgres.listen_host.clone(),
        port: cfg.postgres.listen_port,
        unix_socket_directories: cfg.postgres.socket_dir.clone(),
        hba_file: managed_hba.clone(),
        ident_file: managed_ident.clone(),
        tls: tls_files.managed_tls_config.clone(),
        start_intent: start_intent.clone(),
        extra_gucs: cfg.postgres.extra_gucs.clone(),
    };
    let rendered_conf =
        render_managed_postgres_conf(&managed_conf).map_err(map_managed_conf_error)?;
    write_atomic(
        &managed_postgresql_conf,
        rendered_conf.as_bytes(),
        Some(0o644),
    )?;

    if start_intent.creates_standby_signal() {
        write_atomic(&standby_signal, b"", Some(0o644))?;
    } else {
        remove_file_if_exists(&standby_signal)?;
    }

    Ok(ManagedPostgresConfig {
        postgresql_conf_path: managed_postgresql_conf,
        hba_path: managed_hba,
        ident_path: managed_ident,
        tls_cert_path: tls_files.cert_path,
        tls_key_path: tls_files.key_path,
        tls_client_ca_path: tls_files.client_ca_path,
        standby_signal_path: standby_signal,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MaterializedTlsFiles {
    managed_tls_config: ManagedPostgresTlsConfig,
    cert_path: Option<PathBuf>,
    key_path: Option<PathBuf>,
    client_ca_path: Option<PathBuf>,
}

fn materialize_tls_files(
    cfg: &RuntimeConfig,
) -> Result<MaterializedTlsFiles, ManagedPostgresError> {
    match cfg.postgres.tls.mode {
        ApiTlsMode::Disabled => Ok(MaterializedTlsFiles {
            managed_tls_config: ManagedPostgresTlsConfig::Disabled,
            cert_path: None,
            key_path: None,
            client_ca_path: None,
        }),
        ApiTlsMode::Optional | ApiTlsMode::Required => {
            let identity = cfg.postgres.tls.identity.as_ref().ok_or_else(|| {
                ManagedPostgresError::InvalidConfig {
                    message:
                        "postgres.tls.identity must be configured with user-supplied certificate material when postgres.tls.mode is optional or required"
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

            // Production TLS credentials are operator-supplied; pgtuskmaster only copies them
            // into managed runtime files under PGDATA before PostgreSQL starts.
            write_atomic(&managed_cert, cert_pem.as_slice(), Some(0o644))?;
            write_atomic(&managed_key, key_pem.as_slice(), Some(0o600))?;

            let client_ca_path = if let Some(client_auth) = cfg.postgres.tls.client_auth.as_ref() {
                let ca_pem = load_inline_or_path_bytes(
                    "postgres.tls.client_auth.client_ca",
                    &client_auth.client_ca,
                )?;
                let managed_ca = absolutize_path(&cfg.postgres.data_dir.join("pgtm.ca.crt"))?;
                write_atomic(&managed_ca, ca_pem.as_slice(), Some(0o644))?;
                Some(managed_ca)
            } else {
                None
            };

            Ok(MaterializedTlsFiles {
                managed_tls_config: ManagedPostgresTlsConfig::Enabled {
                    cert_file: managed_cert.clone(),
                    key_file: managed_key.clone(),
                    ca_file: client_ca_path.clone(),
                },
                cert_path: Some(managed_cert),
                key_path: Some(managed_key),
                client_ca_path,
            })
        }
    }
}

fn map_managed_conf_error(err: ManagedPostgresConfError) -> ManagedPostgresError {
    match err {
        ManagedPostgresConfError::InvalidExtraGuc { key, message } => {
            ManagedPostgresError::InvalidConfig {
                message: format!("postgres.extra_gucs entry `{key}` invalid: {message}"),
            }
        }
        ManagedPostgresConfError::ReservedExtraGuc { key } => ManagedPostgresError::InvalidConfig {
            message: format!("postgres.extra_gucs entry `{key}` is reserved by pgtuskmaster"),
        },
        ManagedPostgresConfError::InvalidPrimarySlotName { slot, message } => {
            ManagedPostgresError::InvalidConfig {
                message: format!("managed replica slot `{slot}` invalid: {message}"),
            }
        }
    }
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

fn remove_file_if_exists(path: &Path) -> Result<(), ManagedPostgresError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(ManagedPostgresError::Io {
            message: format!("failed to remove {}: {err}", path.display()),
        }),
    }
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
    use std::{collections::BTreeMap, fs, path::PathBuf};

    use crate::{
        config::{
            ApiAuthConfig, ApiConfig, ApiSecurityConfig, ApiTlsMode, BinaryPaths, ClusterConfig,
            DcsConfig, DebugConfig, FileSinkConfig, FileSinkMode, HaConfig, InlineOrPath,
            LogCleanupConfig, LogLevel, LoggingConfig, LoggingSinksConfig, PgHbaConfig,
            PgIdentConfig, PostgresConfig, PostgresConnIdentityConfig, PostgresLoggingConfig,
            PostgresRoleConfig, PostgresRolesConfig, ProcessConfig, RoleAuthConfig, RuntimeConfig,
            StderrSinkConfig, TlsServerConfig,
        },
        pginfo::{conninfo::PgSslMode, state::PgConnInfo},
        postgres_managed_conf::{ManagedPostgresStartIntent, MANAGED_POSTGRESQL_CONF_NAME},
    };

    use super::{materialize_managed_postgres_config, ManagedPostgresError};

    #[test]
    fn materialize_managed_postgres_config_creates_authoritative_postgresql_conf(
    ) -> Result<(), String> {
        let data_dir = unique_test_data_dir("postgresql-conf");
        let cfg = sample_runtime_config(data_dir.clone());

        let managed =
            materialize_managed_postgres_config(&cfg, &ManagedPostgresStartIntent::primary())
                .map_err(|err| format!("materialize managed config failed: {err}"))?;

        let postgresql_conf = fs::read_to_string(&managed.postgresql_conf_path).map_err(|err| {
            format!(
                "read managed postgresql conf {} failed: {err}",
                managed.postgresql_conf_path.display()
            )
        })?;

        if !postgresql_conf.contains("listen_addresses = '127.0.0.1'") {
            return Err(format!(
                "managed postgresql conf missing listen_addresses: {postgresql_conf}"
            ));
        }
        if !postgresql_conf.contains("hba_file =") || !postgresql_conf.contains("ident_file =") {
            return Err(format!(
                "managed postgresql conf missing managed file paths: {postgresql_conf}"
            ));
        }
        if !postgresql_conf.contains("hot_standby = off") {
            return Err(format!(
                "managed postgresql conf missing primary hot_standby=off: {postgresql_conf}"
            ));
        }
        if postgresql_conf.contains("archive_mode")
            || postgresql_conf.contains("archive_command")
            || postgresql_conf.contains("restore_command")
        {
            return Err(format!(
                "managed postgresql conf unexpectedly contains backup settings: {postgresql_conf}"
            ));
        }

        fs::remove_dir_all(&data_dir)
            .map_err(|err| format!("remove temp dir {} failed: {err}", data_dir.display()))?;
        Ok(())
    }

    #[test]
    fn materialize_managed_postgres_config_uses_config_file_path_for_startup() -> Result<(), String>
    {
        let data_dir = unique_test_data_dir("config-file");
        let cfg = sample_runtime_config(data_dir.clone());

        let managed =
            materialize_managed_postgres_config(&cfg, &ManagedPostgresStartIntent::primary())
                .map_err(|err| format!("materialize managed config failed: {err}"))?;

        let expected = data_dir.join(MANAGED_POSTGRESQL_CONF_NAME);
        if managed.postgresql_conf_path != expected {
            return Err(format!(
                "unexpected postgresql_conf_path: expected={} got={}",
                expected.display(),
                managed.postgresql_conf_path.display()
            ));
        }

        fs::remove_dir_all(&data_dir)
            .map_err(|err| format!("remove temp dir {} failed: {err}", data_dir.display()))?;
        Ok(())
    }

    #[test]
    fn materialize_managed_postgres_config_creates_and_removes_standby_signal() -> Result<(), String>
    {
        let data_dir = unique_test_data_dir("standby-signal");
        let cfg = sample_runtime_config(data_dir.clone());
        let replica_start = ManagedPostgresStartIntent::replica(
            PgConnInfo {
                host: "leader.internal".to_string(),
                port: 5432,
                user: "replicator".to_string(),
                dbname: "postgres".to_string(),
                application_name: None,
                connect_timeout_s: Some(5),
                ssl_mode: PgSslMode::Prefer,
                options: None,
            },
            None,
        );

        let managed_replica = materialize_managed_postgres_config(&cfg, &replica_start)
            .map_err(|err| format!("materialize replica config failed: {err}"))?;
        if !managed_replica.standby_signal_path.exists() {
            return Err(format!(
                "expected standby.signal to exist at {}",
                managed_replica.standby_signal_path.display()
            ));
        }

        let managed_primary =
            materialize_managed_postgres_config(&cfg, &ManagedPostgresStartIntent::primary())
                .map_err(|err| format!("materialize primary config failed: {err}"))?;
        if managed_primary.standby_signal_path.exists() {
            return Err(format!(
                "expected standby.signal to be removed at {}",
                managed_primary.standby_signal_path.display()
            ));
        }

        fs::remove_dir_all(&data_dir)
            .map_err(|err| format!("remove temp dir {} failed: {err}", data_dir.display()))?;
        Ok(())
    }

    #[test]
    fn materialize_managed_postgres_config_rejects_reserved_extra_guc() {
        let data_dir = unique_test_data_dir("reserved-extra");
        let mut cfg = sample_runtime_config(data_dir.clone());
        cfg.postgres
            .extra_gucs
            .insert("config_file".to_string(), "/tmp/override.conf".to_string());

        assert_eq!(
            materialize_managed_postgres_config(&cfg, &ManagedPostgresStartIntent::primary()),
            Err(ManagedPostgresError::InvalidConfig {
                message: "postgres.extra_gucs entry `config_file` is reserved by pgtuskmaster"
                    .to_string(),
            })
        );

        let _ = fs::remove_dir_all(&data_dir);
    }

    #[test]
    fn materialize_managed_postgres_config_writes_managed_tls_files_for_user_supplied_identity(
    ) -> Result<(), String> {
        let data_dir = unique_test_data_dir("tls");
        let mut cfg = sample_runtime_config(data_dir.clone());
        cfg.postgres.tls = TlsServerConfig {
            mode: ApiTlsMode::Required,
            identity: Some(crate::config::TlsServerIdentityConfig {
                cert_chain: InlineOrPath::Inline {
                    content: "CERT".to_string(),
                },
                private_key: InlineOrPath::Inline {
                    content: "KEY".to_string(),
                },
            }),
            client_auth: Some(crate::config::TlsClientAuthConfig {
                client_ca: InlineOrPath::Inline {
                    content: "CA".to_string(),
                },
                require_client_cert: true,
            }),
        };

        let managed =
            materialize_managed_postgres_config(&cfg, &ManagedPostgresStartIntent::primary())
                .map_err(|err| format!("materialize managed config failed: {err}"))?;

        let cert = managed
            .tls_cert_path
            .ok_or_else(|| "missing managed cert path".to_string())?;
        let key = managed
            .tls_key_path
            .ok_or_else(|| "missing managed key path".to_string())?;
        let ca = managed
            .tls_client_ca_path
            .ok_or_else(|| "missing managed ca path".to_string())?;

        if fs::read_to_string(&cert).map_err(|err| err.to_string())? != "CERT" {
            return Err(format!("unexpected cert contents at {}", cert.display()));
        }
        if fs::read_to_string(&key).map_err(|err| err.to_string())? != "KEY" {
            return Err(format!("unexpected key contents at {}", key.display()));
        }
        if fs::read_to_string(&ca).map_err(|err| err.to_string())? != "CA" {
            return Err(format!("unexpected ca contents at {}", ca.display()));
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
                extra_gucs: BTreeMap::new(),
            },
            dcs: DcsConfig {
                endpoints: vec!["http://127.0.0.1:2379".to_string()],
                scope: "cluster-a".to_string(),
                init: None,
            },
            ha: HaConfig {
                loop_interval_ms: 500,
                lease_ttl_ms: 5_000,
            },
            process: ProcessConfig {
                pg_rewind_timeout_ms: 30_000,
                bootstrap_timeout_ms: 30_000,
                fencing_timeout_ms: 10_000,
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
