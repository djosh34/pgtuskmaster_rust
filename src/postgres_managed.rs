use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use thiserror::Error;

use crate::{
    config::{ApiTlsMode, InlineOrPath, RuntimeConfig},
    pginfo::conninfo::parse_pg_conninfo,
    postgres_managed_conf::{
        render_managed_postgres_conf, ManagedPostgresConf, ManagedPostgresConfError,
        ManagedPostgresStartIntent, ManagedPostgresTlsConfig, ManagedRecoverySignal,
        MANAGED_POSTGRESQL_CONF_NAME, MANAGED_RECOVERY_SIGNAL_NAME, MANAGED_STANDBY_SIGNAL_NAME,
    },
};

const MANAGED_PG_HBA_CONF_NAME: &str = "pgtm.pg_hba.conf";
const MANAGED_PG_IDENT_CONF_NAME: &str = "pgtm.pg_ident.conf";
const POSTGRESQL_AUTO_CONF_NAME: &str = "postgresql.auto.conf";
const QUARANTINED_POSTGRESQL_AUTO_CONF_NAME: &str = "pgtm.unmanaged.postgresql.auto.conf";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ManagedPostgresConfig {
    pub(crate) postgresql_conf_path: PathBuf,
    pub(crate) hba_path: PathBuf,
    pub(crate) ident_path: PathBuf,
    pub(crate) tls_cert_path: Option<PathBuf>,
    pub(crate) tls_key_path: Option<PathBuf>,
    pub(crate) tls_client_ca_path: Option<PathBuf>,
    pub(crate) standby_signal_path: PathBuf,
    pub(crate) recovery_signal_path: PathBuf,
    pub(crate) postgresql_auto_conf_path: PathBuf,
    pub(crate) quarantined_postgresql_auto_conf_path: PathBuf,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(crate) enum ManagedPostgresError {
    #[error("io error: {message}")]
    Io { message: String },
    #[error("invalid config: {message}")]
    InvalidConfig { message: String },
    #[error("invalid managed postgres state: {message}")]
    InvalidManagedState { message: String },
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
    let recovery_signal =
        absolutize_path(&cfg.postgres.data_dir.join(MANAGED_RECOVERY_SIGNAL_NAME))?;
    let postgresql_auto_conf =
        absolutize_path(&cfg.postgres.data_dir.join(POSTGRESQL_AUTO_CONF_NAME))?;
    let quarantined_postgresql_auto_conf = absolutize_path(
        &cfg.postgres
            .data_dir
            .join(QUARANTINED_POSTGRESQL_AUTO_CONF_NAME),
    )?;

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

    quarantine_postgresql_auto_conf(&postgresql_auto_conf, &quarantined_postgresql_auto_conf)?;
    materialize_recovery_signal_files(
        start_intent.recovery_signal(),
        &standby_signal,
        &recovery_signal,
    )?;

    Ok(ManagedPostgresConfig {
        postgresql_conf_path: managed_postgresql_conf,
        hba_path: managed_hba,
        ident_path: managed_ident,
        tls_cert_path: tls_files.cert_path,
        tls_key_path: tls_files.key_path,
        tls_client_ca_path: tls_files.client_ca_path,
        standby_signal_path: standby_signal,
        recovery_signal_path: recovery_signal,
        postgresql_auto_conf_path: postgresql_auto_conf,
        quarantined_postgresql_auto_conf_path: quarantined_postgresql_auto_conf,
    })
}

pub(crate) fn read_existing_replica_start_intent(
    data_dir: &Path,
) -> Result<Option<ManagedPostgresStartIntent>, ManagedPostgresError> {
    let recovery_signal = existing_recovery_signal(data_dir)?;
    let Some(recovery_signal) = recovery_signal else {
        return Ok(None);
    };

    let managed_conf_path = data_dir.join(MANAGED_POSTGRESQL_CONF_NAME);
    let rendered = fs::read_to_string(&managed_conf_path).map_err(|err| ManagedPostgresError::Io {
        message: format!(
            "failed to read existing managed postgres conf {}: {err}",
            managed_conf_path.display()
        ),
    })?;

    let primary_conninfo_raw = parse_managed_string_setting(rendered.as_str(), "primary_conninfo")?
        .ok_or_else(|| ManagedPostgresError::InvalidManagedState {
            message: format!(
                "existing managed replica state at {} is missing primary_conninfo",
                managed_conf_path.display()
            ),
        })?;
    let primary_conninfo =
        parse_pg_conninfo(primary_conninfo_raw.as_str()).map_err(|err| {
            ManagedPostgresError::InvalidManagedState {
                message: format!(
                    "existing managed primary_conninfo at {} is invalid: {err}",
                    managed_conf_path.display()
                ),
            }
        })?;
    let primary_slot_name = parse_managed_string_setting(rendered.as_str(), "primary_slot_name")?;

    match recovery_signal {
        ManagedRecoverySignal::Standby => Ok(Some(ManagedPostgresStartIntent::replica(
            primary_conninfo,
            primary_slot_name,
        ))),
        ManagedRecoverySignal::Recovery => Ok(Some(ManagedPostgresStartIntent::recovery(
            primary_conninfo,
            primary_slot_name,
        ))),
        ManagedRecoverySignal::None => Ok(None),
    }
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

fn existing_recovery_signal(data_dir: &Path) -> Result<Option<ManagedRecoverySignal>, ManagedPostgresError> {
    let standby_signal_path = data_dir.join(MANAGED_STANDBY_SIGNAL_NAME);
    let recovery_signal_path = data_dir.join(MANAGED_RECOVERY_SIGNAL_NAME);
    let standby_present = file_exists(standby_signal_path.as_path())?;
    let recovery_present = file_exists(recovery_signal_path.as_path())?;

    match (standby_present, recovery_present) {
        (false, false) => Ok(None),
        (true, false) => Ok(Some(ManagedRecoverySignal::Standby)),
        (false, true) => Ok(Some(ManagedRecoverySignal::Recovery)),
        (true, true) => Err(ManagedPostgresError::InvalidManagedState {
            message: format!(
                "conflicting managed recovery signal files exist at {} and {}",
                standby_signal_path.display(),
                recovery_signal_path.display()
            ),
        }),
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

fn file_exists(path: &Path) -> Result<bool, ManagedPostgresError> {
    match fs::metadata(path) {
        Ok(metadata) => Ok(metadata.is_file()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(err) => Err(ManagedPostgresError::Io {
            message: format!("failed to stat {}: {err}", path.display()),
        }),
    }
}

fn materialize_recovery_signal_files(
    recovery_signal: ManagedRecoverySignal,
    standby_signal: &Path,
    recovery_signal_path: &Path,
) -> Result<(), ManagedPostgresError> {
    match recovery_signal {
        ManagedRecoverySignal::None => {
            remove_file_if_exists(standby_signal)?;
            remove_file_if_exists(recovery_signal_path)?;
        }
        ManagedRecoverySignal::Standby => {
            write_atomic(standby_signal, b"", Some(0o644))?;
            remove_file_if_exists(recovery_signal_path)?;
        }
        ManagedRecoverySignal::Recovery => {
            write_atomic(recovery_signal_path, b"", Some(0o644))?;
            remove_file_if_exists(standby_signal)?;
        }
    }
    Ok(())
}

fn quarantine_postgresql_auto_conf(
    postgresql_auto_conf: &Path,
    quarantined_postgresql_auto_conf: &Path,
) -> Result<(), ManagedPostgresError> {
    match fs::rename(postgresql_auto_conf, quarantined_postgresql_auto_conf) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => {
            if file_exists(quarantined_postgresql_auto_conf)? {
                fs::remove_file(quarantined_postgresql_auto_conf).map_err(|remove_err| {
                    ManagedPostgresError::Io {
                        message: format!(
                            "failed to remove previous quarantined auto conf {} after rename error ({err}): {remove_err}",
                            quarantined_postgresql_auto_conf.display()
                        ),
                    }
                })?;
                fs::rename(postgresql_auto_conf, quarantined_postgresql_auto_conf).map_err(
                    |rename_err| ManagedPostgresError::Io {
                        message: format!(
                            "failed to quarantine {} to {}: {rename_err}",
                            postgresql_auto_conf.display(),
                            quarantined_postgresql_auto_conf.display()
                        ),
                    },
                )
            } else {
                Err(ManagedPostgresError::Io {
                    message: format!(
                        "failed to quarantine {} to {}: {err}",
                        postgresql_auto_conf.display(),
                        quarantined_postgresql_auto_conf.display()
                    ),
                })
            }
        }
    }
}

fn parse_managed_string_setting(
    contents: &str,
    key: &str,
) -> Result<Option<String>, ManagedPostgresError> {
    let prefix = format!("{key} = '");
    for line in contents.lines() {
        if let Some(rest) = line.strip_prefix(prefix.as_str()) {
            let Some(quoted) = rest.strip_suffix('\'') else {
                return Err(ManagedPostgresError::InvalidManagedState {
                    message: format!("managed config setting `{key}` is missing a closing quote"),
                });
            };
            return unescape_managed_string(quoted).map(Some);
        }
    }
    Ok(None)
}

fn unescape_managed_string(value: &str) -> Result<String, ManagedPostgresError> {
    let mut chars = value.chars().peekable();
    let mut out = String::with_capacity(value.len());
    while let Some(ch) = chars.next() {
        match ch {
            '\'' => {
                let Some(next) = chars.next() else {
                    return Err(ManagedPostgresError::InvalidManagedState {
                        message: "managed config string contains an unescaped single quote"
                            .to_string(),
                    });
                };
                if next != '\'' {
                    return Err(ManagedPostgresError::InvalidManagedState {
                        message: "managed config string contains an unescaped single quote"
                            .to_string(),
                    });
                }
                out.push('\'');
            }
            '\\' => {
                let Some(next) = chars.next() else {
                    return Err(ManagedPostgresError::InvalidManagedState {
                        message: "managed config string ends with a trailing backslash"
                            .to_string(),
                    });
                };
                out.push(next);
            }
            other => out.push(other),
        }
    }
    Ok(out)
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
        if file_exists(path)? {
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
    use std::{collections::BTreeMap, fs, io, path::PathBuf, time::Duration};

    use tokio::process::Command;
    use tokio::time::Instant;
    use tokio_postgres::NoTls;

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
        postgres_managed_conf::{
            ManagedPostgresStartIntent, MANAGED_POSTGRESQL_CONF_NAME,
            MANAGED_RECOVERY_SIGNAL_NAME,
        },
        test_harness::{
            binaries::require_pg16_bin_for_real_tests, namespace::NamespaceGuard,
            pg16::{prepare_pgdata_dir, spawn_pg16, PgHandle, PgInstanceSpec},
            ports::allocate_ports,
        },
    };

    use super::{
        materialize_managed_postgres_config, read_existing_replica_start_intent,
        ManagedPostgresError, POSTGRESQL_AUTO_CONF_NAME, QUARANTINED_POSTGRESQL_AUTO_CONF_NAME,
    };

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
        if managed_replica.recovery_signal_path.exists() {
            return Err(format!(
                "expected recovery.signal to be absent at {}",
                managed_replica.recovery_signal_path.display()
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
        if managed_primary.recovery_signal_path.exists() {
            return Err(format!(
                "expected recovery.signal to be removed at {}",
                managed_primary.recovery_signal_path.display()
            ));
        }

        fs::remove_dir_all(&data_dir)
            .map_err(|err| format!("remove temp dir {} failed: {err}", data_dir.display()))?;
        Ok(())
    }

    #[test]
    fn materialize_managed_postgres_config_creates_recovery_signal_and_cleans_standby_signal(
    ) -> Result<(), String> {
        let data_dir = unique_test_data_dir("recovery-signal");
        let cfg = sample_runtime_config(data_dir.clone());
        let standby_signal = data_dir.join("standby.signal");
        fs::create_dir_all(&data_dir)
            .map_err(|err| format!("create test dir {} failed: {err}", data_dir.display()))?;
        fs::write(&standby_signal, b"")
            .map_err(|err| format!("seed standby.signal {} failed: {err}", standby_signal.display()))?;

        let managed = materialize_managed_postgres_config(
            &cfg,
            &ManagedPostgresStartIntent::recovery(
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
            ),
        )
        .map_err(|err| format!("materialize recovery config failed: {err}"))?;

        if !managed.recovery_signal_path.exists() {
            return Err(format!(
                "expected recovery.signal to exist at {}",
                managed.recovery_signal_path.display()
            ));
        }
        if managed.standby_signal_path.exists() {
            return Err(format!(
                "expected standby.signal to be removed at {}",
                managed.standby_signal_path.display()
            ));
        }

        fs::remove_dir_all(&data_dir)
            .map_err(|err| format!("remove temp dir {} failed: {err}", data_dir.display()))?;
        Ok(())
    }

    #[test]
    fn materialize_managed_postgres_config_quarantines_postgresql_auto_conf() -> Result<(), String> {
        let data_dir = unique_test_data_dir("postgresql-auto-conf");
        let cfg = sample_runtime_config(data_dir.clone());
        let active_auto_conf = data_dir.join(POSTGRESQL_AUTO_CONF_NAME);
        let quarantined_auto_conf = data_dir.join(QUARANTINED_POSTGRESQL_AUTO_CONF_NAME);
        fs::create_dir_all(&data_dir)
            .map_err(|err| format!("create test dir {} failed: {err}", data_dir.display()))?;
        fs::write(&active_auto_conf, "primary_conninfo = 'stale'\n")
            .map_err(|err| format!("write active auto conf {} failed: {err}", active_auto_conf.display()))?;
        fs::write(&quarantined_auto_conf, "stale previous quarantine\n")
            .map_err(|err| format!("write quarantined auto conf {} failed: {err}", quarantined_auto_conf.display()))?;

        let managed =
            materialize_managed_postgres_config(&cfg, &ManagedPostgresStartIntent::primary())
                .map_err(|err| format!("materialize primary config failed: {err}"))?;

        if managed.postgresql_auto_conf_path.exists() {
            return Err(format!(
                "expected active postgresql.auto.conf to be absent at {}",
                managed.postgresql_auto_conf_path.display()
            ));
        }
        let quarantined = fs::read_to_string(&managed.quarantined_postgresql_auto_conf_path)
            .map_err(|err| {
                format!(
                    "read quarantined auto conf {} failed: {err}",
                    managed.quarantined_postgresql_auto_conf_path.display()
                )
            })?;
        if quarantined != "primary_conninfo = 'stale'\n" {
            return Err(format!(
                "unexpected quarantined auto conf contents at {}: {quarantined}",
                managed.quarantined_postgresql_auto_conf_path.display()
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

    #[tokio::test(flavor = "current_thread")]
    async fn materialize_managed_postgres_config_real_clone_start_quarantines_auto_conf_and_stale_signal(
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let postgres_bin = require_pg16_bin_for_real_tests("postgres")?;
        let initdb_bin = require_pg16_bin_for_real_tests("initdb")?;
        let basebackup_bin = require_pg16_bin_for_real_tests("pg_basebackup")?;

        let guard = NamespaceGuard::new("managed-config-real-start")?;
        let namespace = guard.namespace()?;

        let primary_data = prepare_pgdata_dir(namespace, "primary")?;
        let primary_socket = namespace.child_dir("run/primary");
        let primary_logs = namespace.child_dir("logs/primary");
        fs::create_dir_all(&primary_socket)?;
        fs::create_dir_all(&primary_logs)?;

        let primary_reservation = allocate_ports(1)?;
        let primary_port = primary_reservation.as_slice()[0];
        drop(primary_reservation);

        let mut primary = spawn_pg16(PgInstanceSpec {
            postgres_bin: postgres_bin.clone(),
            initdb_bin: initdb_bin.clone(),
            data_dir: primary_data,
            socket_dir: primary_socket,
            log_dir: primary_logs,
            port: primary_port,
            startup_timeout: Duration::from_secs(25),
        })
        .await?;

        let primary_dsn = format!(
            "host=127.0.0.1 port={} user=postgres dbname=postgres",
            primary_port
        );
        let run_result = async {
            wait_for_postgres_ready(&primary_dsn, Duration::from_secs(20)).await?;

            let replica_data = namespace.child_dir("pg16/replica/data");
            let replica_parent = replica_data
                .parent()
                .ok_or_else(|| real_test_error("replica data dir has no parent"))?;
            fs::create_dir_all(replica_parent)?;

            let basebackup_output = Command::new(&basebackup_bin)
                .arg("-h")
                .arg("127.0.0.1")
                .arg("-p")
                .arg(primary_port.to_string())
                .arg("-D")
                .arg(&replica_data)
                .arg("-U")
                .arg("postgres")
                .arg("-Fp")
                .arg("-Xs")
                .output()
                .await?;
            if !basebackup_output.status.success() {
                return Err(real_test_error(format!(
                    "pg_basebackup failed with status {}",
                    basebackup_output.status
                )));
            }

            fs::write(replica_data.join(POSTGRESQL_AUTO_CONF_NAME), "port = 1\n")?;
            fs::write(replica_data.join(MANAGED_RECOVERY_SIGNAL_NAME), b"")?;

            let replica_socket = namespace.child_dir("run/replica");
            let replica_logs = namespace.child_dir("logs/replica");
            fs::create_dir_all(&replica_socket)?;
            fs::create_dir_all(&replica_logs)?;

            let replica_reservation = allocate_ports(1)?;
            let replica_port = replica_reservation.as_slice()[0];
            drop(replica_reservation);

            let mut runtime_config = sample_runtime_config(replica_data.clone());
            runtime_config.postgres.listen_port = replica_port;
            runtime_config.postgres.socket_dir = replica_socket.clone();
            runtime_config.postgres.log_file = replica_logs.join("managed-postgres.log");
            runtime_config.postgres.pg_hba.source = InlineOrPath::Inline {
                content: concat!(
                    "local all all trust\n",
                    "host all all 127.0.0.1/32 trust\n",
                    "host replication all 127.0.0.1/32 trust\n",
                )
                .to_string(),
            };

            let managed = materialize_managed_postgres_config(
                &runtime_config,
                &ManagedPostgresStartIntent::replica(
                    PgConnInfo {
                        host: "127.0.0.1".to_string(),
                        port: primary_port,
                        user: "postgres".to_string(),
                        dbname: "postgres".to_string(),
                        application_name: None,
                        connect_timeout_s: Some(5),
                        ssl_mode: PgSslMode::Prefer,
                        options: None,
                    },
                    None,
                ),
            )
            .map_err(|err| real_test_error(format!("materialize managed config failed: {err}")))?;

            if managed.postgresql_auto_conf_path.exists() {
                return Err(real_test_error(format!(
                    "expected active postgresql.auto.conf to be absent at {}",
                    managed.postgresql_auto_conf_path.display()
                )));
            }
            if !managed.quarantined_postgresql_auto_conf_path.exists() {
                return Err(real_test_error(format!(
                    "expected quarantined postgresql.auto.conf to exist at {}",
                    managed.quarantined_postgresql_auto_conf_path.display()
                )));
            }
            if !managed.standby_signal_path.exists() {
                return Err(real_test_error(format!(
                    "expected standby.signal to exist at {}",
                    managed.standby_signal_path.display()
                )));
            }
            if managed.recovery_signal_path.exists() {
                return Err(real_test_error(format!(
                    "expected recovery.signal to be absent at {}",
                    managed.recovery_signal_path.display()
                )));
            }

            let stdout_file = fs::File::create(replica_logs.join("postgres.stdout.log"))?;
            let stderr_file = fs::File::create(replica_logs.join("postgres.stderr.log"))?;
            let mut replica_child = Command::new(&postgres_bin)
                .arg("-D")
                .arg(&replica_data)
                .arg("-c")
                .arg(format!("config_file={}", managed.postgresql_conf_path.display()))
                .stdout(stdout_file)
                .stderr(stderr_file)
                .spawn()?;

            let replica_dsn = format!(
                "host=127.0.0.1 port={} user=postgres dbname=postgres",
                replica_port
            );
            let replica_result = async {
                wait_for_postgres_ready(&replica_dsn, Duration::from_secs(25)).await?;
                let (client, connection) = tokio_postgres::connect(&replica_dsn, NoTls).await?;
                let connection_task = tokio::spawn(connection);

                let port = client.query_one("SHOW port", &[]).await?;
                let port_text: String = port.get(0);
                if port_text != replica_port.to_string() {
                    return Err(real_test_error(format!(
                        "expected postgres to listen on managed port {}, got {}",
                        replica_port, port_text
                    )));
                }

                let primary_conninfo = client.query_one("SHOW primary_conninfo", &[]).await?;
                let primary_conninfo_text: String = primary_conninfo.get(0);
                if !primary_conninfo_text.contains(primary_port.to_string().as_str()) {
                    return Err(real_test_error(format!(
                        "expected primary_conninfo to reference primary port {}, got {}",
                        primary_port, primary_conninfo_text
                    )));
                }

                let in_recovery = client.query_one("SELECT pg_is_in_recovery()", &[]).await?;
                let in_recovery_flag: bool = in_recovery.get(0);
                if !in_recovery_flag {
                    return Err(real_test_error(
                        "expected cloned node to start in recovery".to_string(),
                    ));
                }

                drop(client);
                connection_task.await??;
                Ok(())
            }
            .await;

            let shutdown_result = shutdown_child("replica", &mut replica_child).await;
            match (replica_result, shutdown_result) {
                (Ok(()), Ok(())) => Ok(()),
                (Err(err), Ok(())) => Err(err),
                (Ok(()), Err(err)) => Err(err),
                (Err(err), Err(clean_err)) => Err(real_test_error(format!(
                    "{err}; {clean_err}"
                ))),
            }
        }
        .await;

        let shutdown_primary = shutdown_pg_handle("primary", &mut primary).await;
        match (run_result, shutdown_primary) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(err), Ok(())) => Err(err),
            (Ok(()), Err(err)) => Err(err),
            (Err(err), Err(clean_err)) => Err(real_test_error(format!(
                "{err}; {clean_err}"
            ))),
        }
    }

    #[test]
    fn read_existing_replica_start_intent_reads_managed_replica_state() -> Result<(), String> {
        let data_dir = unique_test_data_dir("read-existing-replica");
        let cfg = sample_runtime_config(data_dir.clone());
        let expected =
            ManagedPostgresStartIntent::replica(sample_replica_conninfo(), Some("slot_a".to_string()));

        materialize_managed_postgres_config(&cfg, &expected)
            .map_err(|err| format!("materialize managed replica config failed: {err}"))?;

        let actual = read_existing_replica_start_intent(&data_dir)
            .map_err(|err| format!("read existing replica start intent failed: {err}"))?;
        if actual != Some(expected.clone()) {
            return Err(format!(
                "unexpected existing managed replica state: expected={expected:?} actual={actual:?}"
            ));
        }

        fs::remove_dir_all(&data_dir)
            .map_err(|err| format!("remove temp dir {} failed: {err}", data_dir.display()))?;
        Ok(())
    }

    #[test]
    fn read_existing_replica_start_intent_rejects_conflicting_signal_files() -> Result<(), String> {
        let data_dir = unique_test_data_dir("conflicting-signals");
        fs::create_dir_all(&data_dir)
            .map_err(|err| format!("create test dir {} failed: {err}", data_dir.display()))?;
        let standby_signal = data_dir.join("standby.signal");
        let recovery_signal = data_dir.join(MANAGED_RECOVERY_SIGNAL_NAME);
        fs::write(&standby_signal, b"")
            .map_err(|err| format!("write standby.signal {} failed: {err}", standby_signal.display()))?;
        fs::write(&recovery_signal, b"").map_err(|err| {
            format!(
                "write recovery.signal {} failed: {err}",
                recovery_signal.display()
            )
        })?;

        let actual = read_existing_replica_start_intent(&data_dir);
        match actual {
            Err(ManagedPostgresError::InvalidManagedState { message }) => {
                if !message.contains("conflicting managed recovery signal files") {
                    return Err(format!("unexpected invalid managed state message: {message}"));
                }
            }
            Ok(value) => {
                return Err(format!(
                    "expected conflicting signal files to fail, got {value:?}"
                ));
            }
            Err(err) => return Err(format!("unexpected error variant: {err}")),
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

    fn sample_replica_conninfo() -> PgConnInfo {
        sample_replica_conninfo_for_port(5432)
    }

    fn sample_replica_conninfo_for_port(port: u16) -> PgConnInfo {
        PgConnInfo {
            host: "leader.internal".to_string(),
            port,
            user: "replicator".to_string(),
            dbname: "postgres".to_string(),
            application_name: None,
            connect_timeout_s: Some(5),
            ssl_mode: PgSslMode::Prefer,
            options: None,
        }
    }

    async fn wait_for_postgres_ready(
        dsn: &str,
        timeout: Duration,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let deadline = Instant::now() + timeout;
        loop {
            match tokio_postgres::connect(dsn, NoTls).await {
                Ok((client, connection)) => {
                    let connection_task = tokio::spawn(connection);
                    client.simple_query("SELECT 1").await?;
                    drop(client);
                    connection_task.await??;
                    return Ok(());
                }
                Err(err) => {
                    if Instant::now() >= deadline {
                        return Err(Box::new(err));
                    }
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }
    }

    async fn shutdown_pg_handle(
        label: &str,
        handle: &mut PgHandle,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        handle.shutdown().await.map_err(|err| {
            real_test_error(format!("{label} shutdown failed: {err}"))
        })
    }

    async fn shutdown_child(
        _label: &str,
        child: &mut tokio::process::Child,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if child.try_wait()?.is_none() {
            child.start_kill()?;
            child.wait().await?;
        }
        Ok(())
    }

    fn real_test_error(message: impl Into<String>) -> Box<dyn std::error::Error + Send + Sync> {
        Box::new(io::Error::other(message.into()))
    }
}
