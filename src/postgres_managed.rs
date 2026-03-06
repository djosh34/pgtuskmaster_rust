use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use thiserror::Error;

use crate::config::{ApiTlsMode, BackupTakeoverPolicy, InlineOrPath, RuntimeConfig};

const MANAGED_POSTGRESQL_CONF_NAME: &str = "pgtm.postgresql.conf";
const MANAGED_RECOVERY_SIGNAL_NAME: &str = "recovery.signal";
const MANAGED_STANDBY_SIGNAL_NAME: &str = "standby.signal";
const MANAGED_ARCHIVE_HELPER_CONFIG_NAME: &str = "pgtm.pgbackrest.archive.json";

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

    let hba_contents =
        load_inline_or_path_string("postgres.pg_hba.source", &cfg.postgres.pg_hba.source)?;
    let ident_contents =
        load_inline_or_path_string("postgres.pg_ident.source", &cfg.postgres.pg_ident.source)?;

    write_atomic(&managed_hba, hba_contents.as_bytes(), Some(0o644))?;
    write_atomic(&managed_ident, ident_contents.as_bytes(), Some(0o644))?;

    let mut tls_cert_path = None;
    let mut tls_key_path = None;
    let mut tls_client_ca_path = None;

    let mut extra_settings = BTreeMap::new();
    extra_settings.insert("hba_file".to_string(), managed_hba.display().to_string());
    extra_settings.insert(
        "ident_file".to_string(),
        managed_ident.display().to_string(),
    );

    if cfg.backup.bootstrap.enabled {
        let managed_conf =
            absolutize_path(&cfg.postgres.data_dir.join(MANAGED_POSTGRESQL_CONF_NAME))?;
        ensure_managed_postgresql_conf(&managed_conf)?;
        extra_settings.insert(
            "config_file".to_string(),
            managed_conf.display().to_string(),
        );
    }

    if cfg.backup.enabled {
        crate::backup::archive_command::materialize_archive_command_config(cfg).map_err(|err| {
            ManagedPostgresError::InvalidConfig {
                message: format!("materialize archive command config failed: {err}"),
            }
        })?;

        let helper = crate::self_exe::get().map_err(|err| ManagedPostgresError::InvalidConfig {
            message: format!("failed to resolve self executable path for wal helper: {err}"),
        })?;
        if !helper.is_absolute() {
            return Err(ManagedPostgresError::InvalidConfig {
                message: format!(
                    "wal helper path must be absolute, got `{}`",
                    helper.display()
                ),
            });
        }
        let archive_command = render_postgres_wal_helper_command(
            helper.as_path(),
            cfg.postgres.data_dir.as_path(),
            WalHelperKind::ArchivePush,
        )?;
        let restore_command = render_postgres_wal_helper_command(
            helper.as_path(),
            cfg.postgres.data_dir.as_path(),
            WalHelperKind::ArchiveGet,
        )?;

        extra_settings.insert("archive_mode".to_string(), "on".to_string());
        extra_settings.insert("archive_command".to_string(), archive_command.clone());
        extra_settings.insert("restore_command".to_string(), restore_command.clone());

        let managed_conf =
            absolutize_path(&cfg.postgres.data_dir.join(MANAGED_POSTGRESQL_CONF_NAME))?;
        write_managed_postgresql_conf(
            managed_conf.as_path(),
            archive_command.as_str(),
            restore_command.as_str(),
        )?;
    }

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

pub(crate) fn takeover_restored_data_dir(
    cfg: &RuntimeConfig,
    policy: BackupTakeoverPolicy,
    write_recovery_signal: bool,
) -> Result<(), ManagedPostgresError> {
    let data_dir = cfg.postgres.data_dir.as_path();
    if data_dir.as_os_str().is_empty() {
        return Err(ManagedPostgresError::InvalidConfig {
            message: "postgres.data_dir must not be empty".to_string(),
        });
    }

    // Ensure the directory exists; pgBackRest restore should create it, but we want clear errors if not.
    fs::create_dir_all(data_dir).map_err(|err| ManagedPostgresError::Io {
        message: format!(
            "failed to create postgres.data_dir {}: {err}",
            data_dir.display()
        ),
    })?;

    let quarantine_dir = if matches!(policy, BackupTakeoverPolicy::Quarantine) {
        let millis = now_millis()?;
        Some(data_dir.join(format!("pgtm.quarantine.{millis}")))
    } else {
        None
    };
    if let Some(dir) = quarantine_dir.as_ref() {
        fs::create_dir_all(dir).map_err(|err| ManagedPostgresError::Io {
            message: format!("failed to create quarantine dir {}: {err}", dir.display()),
        })?;
    }

    // Remove/quarantine known config artifacts that can interfere with managed startup.
    // Note: We intentionally remove postgresql.auto.conf so backup-era ALTER SYSTEM settings cannot apply.
    //
    // We intentionally keep postgresql.conf: the process worker starts Postgres with `pg_ctl -D <data_dir>`
    // and does not force a managed `config_file`, so a missing postgresql.conf would prevent startup.
    let explicit_paths = [
        "postgresql.auto.conf",
        "pg_hba.conf",
        "pg_ident.conf",
        MANAGED_RECOVERY_SIGNAL_NAME,
        MANAGED_STANDBY_SIGNAL_NAME,
    ];
    for name in explicit_paths {
        quarantine_or_delete_path(&data_dir.join(name), quarantine_dir.as_ref(), policy)?;
    }

    // Remove/quarantine any stale managed artifacts.
    let entries = fs::read_dir(data_dir).map_err(|err| ManagedPostgresError::Io {
        message: format!(
            "failed to read postgres.data_dir {}: {err}",
            data_dir.display()
        ),
    })?;
    for entry in entries {
        let entry = entry.map_err(|err| ManagedPostgresError::Io {
            message: format!("failed to read_dir entry: {err}"),
        })?;
        let path = entry.path();
        let file_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        if file_name.starts_with("pgtm.") {
            quarantine_or_delete_path(&path, quarantine_dir.as_ref(), policy)?;
        }
    }

    if write_recovery_signal {
        // Freshly own recovery intent for restore bootstrap: always archive-recovery (not standby).
        let recovery_signal = data_dir.join(MANAGED_RECOVERY_SIGNAL_NAME);
        write_atomic(&recovery_signal, b"", Some(0o644))?;
    }

    // Ensure we can write a managed config_file path before starting Postgres.
    let managed_conf = absolutize_path(&cfg.postgres.data_dir.join(MANAGED_POSTGRESQL_CONF_NAME))?;
    ensure_managed_postgresql_conf(&managed_conf)?;

    if cfg.backup.enabled {
        crate::backup::archive_command::materialize_archive_command_config(cfg).map_err(|err| {
            ManagedPostgresError::InvalidConfig {
                message: format!("materialize archive command config failed: {err}"),
            }
        })?;
        let helper = crate::self_exe::get().map_err(|err| ManagedPostgresError::InvalidConfig {
            message: format!("failed to resolve self executable path for wal helper: {err}"),
        })?;
        if !helper.is_absolute() {
            return Err(ManagedPostgresError::InvalidConfig {
                message: format!(
                    "wal helper path must be absolute, got `{}`",
                    helper.display()
                ),
            });
        }
        let archive_command = render_postgres_wal_helper_command(
            helper.as_path(),
            cfg.postgres.data_dir.as_path(),
            WalHelperKind::ArchivePush,
        )?;
        let restore_command = render_postgres_wal_helper_command(
            helper.as_path(),
            cfg.postgres.data_dir.as_path(),
            WalHelperKind::ArchiveGet,
        )?;
        write_managed_postgresql_conf(
            managed_conf.as_path(),
            archive_command.as_str(),
            restore_command.as_str(),
        )?;
    }

    Ok(())
}

fn ensure_managed_postgresql_conf(path: &Path) -> Result<(), ManagedPostgresError> {
    if path.as_os_str().is_empty() {
        return Err(ManagedPostgresError::InvalidConfig {
            message: "managed postgresql.conf path must not be empty".to_string(),
        });
    }
    let contents = b"# managed by pgtuskmaster\n";
    write_atomic(path, contents, Some(0o644))?;
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WalHelperKind {
    ArchivePush,
    ArchiveGet,
}

fn render_postgres_wal_helper_command(
    helper_exe: &Path,
    pgdata: &Path,
    kind: WalHelperKind,
) -> Result<String, ManagedPostgresError> {
    if helper_exe.as_os_str().is_empty() {
        return Err(ManagedPostgresError::InvalidConfig {
            message: "wal helper executable path must not be empty".to_string(),
        });
    }
    if pgdata.as_os_str().is_empty() {
        return Err(ManagedPostgresError::InvalidConfig {
            message: "postgres.data_dir must not be empty".to_string(),
        });
    }

    let mut tokens = vec![
        helper_exe.display().to_string(),
        "wal".to_string(),
        "--pgdata".to_string(),
        pgdata.display().to_string(),
    ];
    match kind {
        WalHelperKind::ArchivePush => {
            tokens.push("archive-push".to_string());
            tokens.push("%p".to_string());
        }
        WalHelperKind::ArchiveGet => {
            tokens.push("archive-get".to_string());
            tokens.push("%f".to_string());
            tokens.push("%p".to_string());
        }
    }

    render_shell_command_from_tokens(tokens.as_slice())
}

fn render_shell_command_from_tokens(tokens: &[String]) -> Result<String, ManagedPostgresError> {
    if tokens.is_empty() {
        return Err(ManagedPostgresError::InvalidConfig {
            message: "wal helper command token list must not be empty".to_string(),
        });
    }
    let mut out = String::new();
    for (idx, token) in tokens.iter().enumerate() {
        if token.is_empty() {
            return Err(ManagedPostgresError::InvalidConfig {
                message: "wal helper command token must not be empty".to_string(),
            });
        }
        if token.contains('\0') || token.contains('\n') || token.contains('\r') {
            return Err(ManagedPostgresError::InvalidConfig {
                message: "wal helper command token contains invalid characters".to_string(),
            });
        }
        if idx > 0 {
            out.push(' ');
        }
        out.push('"');
        out.push_str(escape_shell_double_quoted(token.as_str())?.as_str());
        out.push('"');
    }
    Ok(out)
}

fn escape_shell_double_quoted(token: &str) -> Result<String, ManagedPostgresError> {
    let mut out = String::with_capacity(token.len());
    for ch in token.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '$' => out.push_str("\\$"),
            '`' => out.push_str("\\`"),
            _ => out.push(ch),
        }
    }
    Ok(out)
}

fn escape_postgres_conf_single_quoted(value: &str) -> Result<String, ManagedPostgresError> {
    if value.contains('\0') || value.contains('\n') || value.contains('\r') {
        return Err(ManagedPostgresError::InvalidConfig {
            message: "managed postgresql.conf value contains invalid characters".to_string(),
        });
    }
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\'' => out.push_str("''"),
            _ => out.push(ch),
        }
    }
    Ok(out)
}

fn write_managed_postgresql_conf(
    path: &Path,
    archive_command: &str,
    restore_command: &str,
) -> Result<(), ManagedPostgresError> {
    if path.as_os_str().is_empty() {
        return Err(ManagedPostgresError::InvalidConfig {
            message: "managed postgresql.conf path must not be empty".to_string(),
        });
    }

    let mut contents = String::new();
    contents.push_str("# managed by pgtuskmaster\n");
    contents.push_str("# DO NOT EDIT BY HAND\n");
    contents.push('\n');
    contents.push_str("archive_mode = on\n");
    contents.push_str("archive_command = '");
    contents.push_str(escape_postgres_conf_single_quoted(archive_command)?.as_str());
    contents.push_str("'\n");
    contents.push_str("restore_command = '");
    contents.push_str(escape_postgres_conf_single_quoted(restore_command)?.as_str());
    contents.push_str("'\n");
    contents.push('\n');
    contents.push_str("# helper config written by pgtuskmaster\n");
    contents.push_str("# ");
    contents.push_str(MANAGED_ARCHIVE_HELPER_CONFIG_NAME);
    contents.push('\n');

    write_atomic(path, contents.as_bytes(), Some(0o644))?;
    Ok(())
}

fn quarantine_or_delete_path(
    path: &Path,
    quarantine_dir: Option<&PathBuf>,
    policy: BackupTakeoverPolicy,
) -> Result<(), ManagedPostgresError> {
    let meta = match fs::metadata(path) {
        Ok(meta) => meta,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(err) => {
            return Err(ManagedPostgresError::Io {
                message: format!("failed to stat {}: {err}", path.display()),
            })
        }
    };
    if !meta.is_file() {
        return Ok(());
    }

    match policy {
        BackupTakeoverPolicy::Delete => {
            fs::remove_file(path).map_err(|err| ManagedPostgresError::Io {
                message: format!("failed to remove {}: {err}", path.display()),
            })?;
        }
        BackupTakeoverPolicy::Quarantine => {
            let quarantine_dir =
                quarantine_dir.ok_or_else(|| ManagedPostgresError::InvalidConfig {
                    message: "quarantine policy requires a quarantine dir".to_string(),
                })?;
            let file_name = match path.file_name().and_then(|s| s.to_str()) {
                Some(name) if !name.is_empty() => name,
                _ => "managed",
            };
            let millis = now_millis()?;
            let target = quarantine_dir.join(format!("{file_name}.{millis}"));
            fs::rename(path, &target).map_err(|err| ManagedPostgresError::Io {
                message: format!(
                    "failed to quarantine {} to {}: {err}",
                    path.display(),
                    target.display()
                ),
            })?;
        }
    }
    Ok(())
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
                message: format!(
                    "failed to rename {} to {}: {err}",
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

    use tokio::sync::mpsc;
    use tokio::time::{Duration, Instant};

    use crate::{
        config::{
            ApiAuthConfig, ApiConfig, ApiSecurityConfig, ApiTlsMode, BackupConfig, BinaryPaths,
            ClusterConfig, DcsConfig, DebugConfig, HaConfig, InlineOrPath, LogCleanupConfig,
            LogLevel, LoggingConfig, PgHbaConfig, PgIdentConfig, PostgresConfig,
            PostgresConnIdentityConfig, PostgresLoggingConfig, PostgresRoleConfig,
            PostgresRolesConfig, ProcessConfig, RoleAuthConfig, RuntimeConfig, StderrSinkConfig,
            TlsClientAuthConfig, TlsServerConfig, TlsServerIdentityConfig,
        },
        pginfo::conninfo::PgSslMode,
        process::{
            jobs::{BootstrapSpec, DemoteSpec, ShutdownMode, StartPostgresSpec},
            state::{ProcessJobKind, ProcessJobRequest, ProcessState},
            worker::{step_once as process_step_once, TokioCommandRunner},
        },
        state::{new_state_channel, JobId, UnixMillis, WorkerStatus},
        test_harness::tls::build_adversarial_tls_fixture,
        test_harness::{
            binaries::require_pg16_process_binaries_for_real_tests, namespace::NamespaceGuard,
            pg16::prepare_pgdata_dir, ports::allocate_ports,
        },
    };

    use super::{materialize_managed_postgres_config, takeover_restored_data_dir};

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
                backup_timeout_ms: 1000,
                binaries: BinaryPaths {
                    postgres: "/usr/bin/postgres".into(),
                    pg_ctl: "/usr/bin/pg_ctl".into(),
                    pg_rewind: "/usr/bin/pg_rewind".into(),
                    initdb: "/usr/bin/initdb".into(),
                    pg_basebackup: "/usr/bin/pg_basebackup".into(),
                    psql: "/usr/bin/psql".into(),
                    pgbackrest: None,
                },
            },
            backup: BackupConfig::default(),
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
    fn managed_postgres_config_renders_wal_helper_commands_with_placeholders() -> TestResult {
        crate::self_exe::init_from_current_exe()
            .map_err(|err| std::io::Error::other(format!("{err}")))?;

        let root = temp_dir("managed-wal-helper-render");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root)?;

        let mut cfg = sample_runtime_config(
            root.clone(),
            TlsServerConfig {
                mode: ApiTlsMode::Disabled,
                identity: None,
                client_auth: None,
            },
        );
        cfg.backup.enabled = true;
        cfg.process.binaries.pgbackrest = Some(PathBuf::from("/usr/bin/pgbackrest"));
        if let Some(pg) = cfg.backup.pgbackrest.as_mut() {
            pg.stanza = Some("stanza-a".to_string());
            pg.repo = Some("1".to_string());
            pg.options.archive_push = vec!["--quote=it's fine".to_string()];
        }

        let out = materialize_managed_postgres_config(&cfg)?;
        let archive = out
            .extra_settings
            .get("archive_command")
            .ok_or_else(|| std::io::Error::other("archive_command missing"))?;
        let restore = out
            .extra_settings
            .get("restore_command")
            .ok_or_else(|| std::io::Error::other("restore_command missing"))?;

        assert!(archive.contains("\"wal\""));
        assert!(archive.contains("\"archive-push\""));
        assert!(archive.contains("\"%p\""));

        assert!(restore.contains("\"wal\""));
        assert!(restore.contains("\"archive-get\""));
        assert!(restore.contains("\"%f\""));
        assert!(restore.contains("\"%p\""));

        let _ = fs::remove_dir_all(&root);
        Ok(())
    }

    async fn wait_for_job_success(
        ctx: &mut crate::process::state::ProcessWorkerCtx,
        job_id: &JobId,
        timeout: Duration,
    ) -> Result<(), crate::state::WorkerError> {
        let started = Instant::now();
        while started.elapsed() < timeout {
            process_step_once(ctx).await?;
            if let ProcessState::Idle {
                last_outcome: Some(outcome),
                ..
            } = &ctx.state
            {
                match outcome {
                    crate::process::state::JobOutcome::Success { id, .. } if id == job_id => {
                        return Ok(())
                    }
                    crate::process::state::JobOutcome::Failure { id, error, .. }
                        if id == job_id =>
                    {
                        return Err(crate::state::WorkerError::Message(format!(
                            "process job {} failed: {error}",
                            id.0
                        )));
                    }
                    crate::process::state::JobOutcome::Timeout { id, .. } if id == job_id => {
                        return Err(crate::state::WorkerError::Message(format!(
                            "process job {} timed out",
                            id.0
                        )));
                    }
                    _ => {}
                }
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        Err(crate::state::WorkerError::Message(format!(
            "timed out waiting for job {}",
            job_id.0
        )))
    }

    #[tokio::test(flavor = "current_thread")]
    async fn takeover_prevents_backup_era_max_connections_from_affecting_start(
    ) -> Result<(), crate::state::WorkerError> {
        let binaries = require_pg16_process_binaries_for_real_tests().map_err(|err| {
            crate::state::WorkerError::Message(format!(
                "require pg16 process binaries for test failed: {err}"
            ))
        })?;

        let guard = NamespaceGuard::new("takeover-max-connections").map_err(|err| {
            crate::state::WorkerError::Message(format!("namespace guard failed: {err}"))
        })?;
        let ns = guard.namespace().map_err(|err| {
            crate::state::WorkerError::Message(format!("namespace handle failed: {err}"))
        })?;

        let mut reservation = allocate_ports(1).map_err(|err| {
            crate::state::WorkerError::Message(format!("allocate ports failed: {err}"))
        })?;
        let port = reservation.as_slice()[0];

        let data_dir = prepare_pgdata_dir(ns, "node-a").map_err(|err| {
            crate::state::WorkerError::Message(format!("prepare pgdata dir failed: {err}"))
        })?;
        let socket_dir = ns.child_dir("sock");
        let log_file = ns.child_dir("runtime/pg_ctl.log");
        if let Some(parent) = log_file.parent() {
            fs::create_dir_all(parent).map_err(|err| {
                crate::state::WorkerError::Message(format!("create log parent failed: {err}"))
            })?;
        }
        fs::create_dir_all(&socket_dir).map_err(|err| {
            crate::state::WorkerError::Message(format!("create socket dir failed: {err}"))
        })?;

        let mut cfg = sample_runtime_config(
            data_dir.clone(),
            TlsServerConfig {
                mode: ApiTlsMode::Disabled,
                identity: None,
                client_auth: None,
            },
        );
        cfg.process.binaries = binaries.clone();
        cfg.postgres.data_dir = data_dir.clone();
        cfg.postgres.socket_dir = socket_dir.clone();
        cfg.postgres.listen_port = port;
        cfg.postgres.log_file = log_file.clone();
        cfg.logging.postgres.cleanup.enabled = false;
        cfg.backup.enabled = true;
        cfg.backup.bootstrap.enabled = true;
        if let Some(pg_cfg) = cfg.backup.pgbackrest.as_mut() {
            pg_cfg.stanza = Some("stanza-a".to_string());
            pg_cfg.repo = Some("1".to_string());
        }

        let initial = ProcessState::Idle {
            worker: WorkerStatus::Starting,
            last_outcome: None,
        };
        let (publisher, _subscriber) = new_state_channel(initial.clone(), UnixMillis(0));
        let (tx, rx) = mpsc::unbounded_channel();
        let mut process_ctx = crate::process::state::ProcessWorkerCtx {
            poll_interval: Duration::from_millis(5),
            config: cfg.process.clone(),
            log: crate::logging::LogHandle::null(),
            capture_subprocess_output: true,
            state: initial,
            publisher,
            inbox: rx,
            inbox_disconnected_logged: false,
            command_runner: Box::new(TokioCommandRunner),
            active_runtime: None,
            last_rejection: None,
            now: Box::new(crate::process::worker::system_now_unix_millis),
        };

        let bootstrap_id = JobId("bootstrap".to_string());
        tx.send(ProcessJobRequest {
            id: bootstrap_id.clone(),
            kind: ProcessJobKind::Bootstrap(BootstrapSpec {
                data_dir: data_dir.clone(),
                superuser_username: cfg.postgres.roles.superuser.username.clone(),
                timeout_ms: Some(30_000),
            }),
        })
        .map_err(|_| crate::state::WorkerError::Message("send bootstrap job failed".to_string()))?;
        wait_for_job_success(&mut process_ctx, &bootstrap_id, Duration::from_secs(30)).await?;

        // Simulate backup-era config artifacts.
        fs::write(data_dir.join("postgresql.conf"), b"max_connections=1\n").map_err(|err| {
            crate::state::WorkerError::Message(format!("write postgresql.conf failed: {err}"))
        })?;
        fs::write(
            data_dir.join("postgresql.auto.conf"),
            b"max_connections=1\n",
        )
        .map_err(|err| {
            crate::state::WorkerError::Message(format!("write postgresql.auto.conf failed: {err}"))
        })?;

        takeover_restored_data_dir(&cfg, crate::config::BackupTakeoverPolicy::Delete, false)
            .map_err(|err| crate::state::WorkerError::Message(format!("takeover failed: {err}")))?;

        let managed = materialize_managed_postgres_config(&cfg).map_err(|err| {
            crate::state::WorkerError::Message(format!("materialize managed config failed: {err}"))
        })?;

        reservation.release_port(port).map_err(|err| {
            crate::state::WorkerError::Message(format!("release port failed: {err}"))
        })?;
        let start_id = JobId("start".to_string());
        tx.send(ProcessJobRequest {
            id: start_id.clone(),
            kind: ProcessJobKind::StartPostgres(StartPostgresSpec {
                data_dir: data_dir.clone(),
                host: "127.0.0.1".to_string(),
                port,
                socket_dir: socket_dir.clone(),
                log_file: log_file.clone(),
                extra_postgres_settings: managed.extra_settings.clone(),
                wait_seconds: Some(30),
                timeout_ms: Some(60_000),
            }),
        })
        .map_err(|_| crate::state::WorkerError::Message("send start job failed".to_string()))?;
        wait_for_job_success(&mut process_ctx, &start_id, Duration::from_secs(60)).await?;

        let port_s = port.to_string();
        let output = tokio::process::Command::new(&binaries.psql)
            .args([
                "-h",
                "127.0.0.1",
                "-p",
                port_s.as_str(),
                "-U",
                cfg.postgres.roles.superuser.username.as_str(),
                "-d",
                cfg.postgres.local_conn_identity.dbname.as_str(),
                "-tA",
                "-c",
                "SHOW max_connections;",
            ])
            .output()
            .await
            .map_err(|err| crate::state::WorkerError::Message(format!("psql failed: {err}")))?;
        if !output.status.success() {
            return Err(crate::state::WorkerError::Message(format!(
                "psql show max_connections exited unsuccessfully: {:?}",
                output.status.code()
            )));
        }
        let stdout = String::from_utf8(output.stdout).map_err(|err| {
            crate::state::WorkerError::Message(format!("psql stdout utf8 decode failed: {err}"))
        })?;
        let value = stdout.trim();
        if value == "1" {
            return Err(crate::state::WorkerError::Message(
                "expected max_connections to not be 1 after takeover".to_string(),
            ));
        }

        let stop_id = JobId("stop".to_string());
        tx.send(ProcessJobRequest {
            id: stop_id.clone(),
            kind: ProcessJobKind::Demote(DemoteSpec {
                data_dir,
                mode: ShutdownMode::Fast,
                timeout_ms: Some(20_000),
            }),
        })
        .map_err(|_| crate::state::WorkerError::Message("send stop job failed".to_string()))?;
        wait_for_job_success(&mut process_ctx, &stop_id, Duration::from_secs(30)).await?;

        drop(reservation);
        Ok(())
    }

    #[test]
    fn takeover_quarantines_backup_era_conf_and_writes_recovery_signal() -> TestResult {
        let dir = temp_dir("takeover-quarantine");
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

        fs::write(dir.join("postgresql.conf"), b"max_connections=1\n")?;
        fs::write(dir.join("postgresql.auto.conf"), b"max_connections=1\n")?;
        fs::write(dir.join("pg_hba.conf"), b"local all all trust\n")?;
        fs::write(dir.join("pg_ident.conf"), b"# empty\n")?;
        fs::write(dir.join("standby.signal"), b"")?;
        fs::write(dir.join("recovery.signal"), b"")?;
        fs::write(dir.join("pgtm.stale"), b"stale\n")?;

        takeover_restored_data_dir(&cfg, crate::config::BackupTakeoverPolicy::Quarantine, true)
            .map_err(|err| std::io::Error::other(err.to_string()))?;

        if dir.join("postgresql.auto.conf").exists()
            || dir.join("pg_hba.conf").exists()
            || dir.join("pg_ident.conf").exists()
            || dir.join("standby.signal").exists()
            || dir.join("pgtm.stale").exists()
        {
            return Err(Box::new(std::io::Error::other(
                "expected conflicting files to be removed from data dir",
            )));
        }
        if !dir.join("recovery.signal").exists() {
            return Err(Box::new(std::io::Error::other(
                "expected recovery.signal to exist after takeover",
            )));
        }
        if !dir.join("pgtm.postgresql.conf").exists() {
            return Err(Box::new(std::io::Error::other(
                "expected managed pgtm.postgresql.conf to be created",
            )));
        }

        let mut quarantine_dirs = Vec::new();
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let name = entry.file_name();
                if name.to_string_lossy().starts_with("pgtm.quarantine.") {
                    quarantine_dirs.push(entry.path());
                }
            }
        }
        if quarantine_dirs.len() != 1 {
            return Err(Box::new(std::io::Error::other(format!(
                "expected exactly one quarantine dir, got {}",
                quarantine_dirs.len()
            ))));
        }

        let _ = fs::remove_dir_all(&dir);
        Ok(())
    }

    #[test]
    fn takeover_succeeds_when_backup_shipped_no_postgresql_conf() -> TestResult {
        let dir = temp_dir("takeover-missing-postgresql-conf");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir)?;

        let mut cfg = sample_runtime_config(
            dir.clone(),
            TlsServerConfig {
                mode: ApiTlsMode::Disabled,
                identity: None,
                client_auth: None,
            },
        );
        cfg.backup.enabled = true;
        cfg.process.binaries.pgbackrest = Some(PathBuf::from("/usr/bin/pgbackrest"));
        if let Some(pg_cfg) = cfg.backup.pgbackrest.as_mut() {
            pg_cfg.stanza = Some("stanza-a".to_string());
            pg_cfg.repo = Some("1".to_string());
            pg_cfg.options.archive_push = vec!["--repo1-path=/var/lib/pgbackrest".to_string()];
            pg_cfg.options.archive_get = vec!["--repo1-path=/var/lib/pgbackrest".to_string()];
        }

        // Intentionally do not create postgresql.conf / postgresql.auto.conf.
        takeover_restored_data_dir(&cfg, crate::config::BackupTakeoverPolicy::Delete, true)
            .map_err(|err| std::io::Error::other(err.to_string()))?;

        if !dir.join("pgtm.pgbackrest.archive.json").exists() {
            return Err(Box::new(std::io::Error::other(
                "expected archive helper config file to exist after takeover",
            )));
        }

        let managed_conf = dir.join("pgtm.postgresql.conf");
        if !managed_conf.exists() {
            return Err(Box::new(std::io::Error::other(
                "expected managed postgresql.conf to exist after takeover",
            )));
        }
        let contents = fs::read_to_string(&managed_conf)?;
        if !contents.contains("archive_command") || !contents.contains("restore_command") {
            return Err(Box::new(std::io::Error::other(
                "expected managed postgresql.conf to include archive/restore command wiring",
            )));
        }

        let _ = fs::remove_dir_all(&dir);
        Ok(())
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
            return Err(Box::new(std::io::Error::other("unexpected hba contents")));
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
    fn backup_enabled_materialize_writes_managed_archive_wiring_files() -> TestResult {
        let dir = temp_dir("managed-archive-wiring");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir)?;

        let mut cfg = sample_runtime_config(
            dir.clone(),
            TlsServerConfig {
                mode: ApiTlsMode::Disabled,
                identity: None,
                client_auth: None,
            },
        );
        cfg.backup.enabled = true;
        cfg.process.binaries.pgbackrest = Some(PathBuf::from("/usr/bin/pgbackrest"));
        if let Some(pg_cfg) = cfg.backup.pgbackrest.as_mut() {
            pg_cfg.stanza = Some("stanza-a".to_string());
            pg_cfg.repo = Some("1".to_string());
            pg_cfg.options.archive_push = vec!["--repo1-path=/var/lib/pgbackrest".to_string()];
            pg_cfg.options.archive_get = vec!["--repo1-path=/var/lib/pgbackrest".to_string()];
        }

        let _managed = materialize_managed_postgres_config(&cfg)
            .map_err(|err| std::io::Error::other(err.to_string()))?;

        let helper_cfg = dir.join("pgtm.pgbackrest.archive.json");
        if !helper_cfg.exists() {
            return Err(Box::new(std::io::Error::other(
                "expected archive helper config file to exist",
            )));
        }

        let managed_conf = dir.join("pgtm.postgresql.conf");
        if !managed_conf.exists() {
            return Err(Box::new(std::io::Error::other(
                "expected managed postgresql.conf to exist",
            )));
        }
        let contents = fs::read_to_string(&managed_conf)?;
        if !contents.contains("archive_command") || !contents.contains("restore_command") {
            return Err(Box::new(std::io::Error::other(
                "expected managed postgresql.conf to include archive/restore command wiring",
            )));
        }
        if !contents.contains("wal") {
            return Err(Box::new(std::io::Error::other(
                "expected managed postgresql.conf to invoke wal helper",
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

    #[test]
    fn materialize_rejects_tls_required_without_identity() -> TestResult {
        let dir = temp_dir("tls-required-missing-identity");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir)?;

        let cfg = sample_runtime_config(
            dir.clone(),
            TlsServerConfig {
                mode: ApiTlsMode::Required,
                identity: None,
                client_auth: None,
            },
        );

        let result = materialize_managed_postgres_config(&cfg);
        if !matches!(
            result,
            Err(super::ManagedPostgresError::InvalidConfig { .. })
        ) {
            return Err(Box::new(std::io::Error::other(
                "expected tls required without identity to be rejected",
            )));
        }

        let _ = fs::remove_dir_all(&dir);
        Ok(())
    }
}
