use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use thiserror::Error;

use crate::{
    config_v2::{types::TlsConfig, RuntimeConfigV2},
    pginfo::{conninfo::render_conninfo_value, state::PgConnInfo},
    process::jobs::ProcessJobKind,
};

pub(crate) const MANAGED_POSTGRESQL_CONF_NAME: &str = "pgtm.postgresql.conf";
const MANAGED_POSTGRESQL_CONF_HEADER: &str = "\
# This file is managed by pgtuskmaster.\n\
# Backup-era archive and restore settings have been removed.\n\
# Production TLS material must be supplied by the operator as direct file paths.\n";
pub(crate) const MANAGED_STANDBY_SIGNAL_NAME: &str = "standby.signal";
pub(crate) const MANAGED_RECOVERY_SIGNAL_NAME: &str = "recovery.signal";
const MANAGED_STANDBY_PASSFILE_NAME: &str = "pgtm.standby.passfile";
const POSTGRESQL_AUTO_CONF_NAME: &str = "postgresql.auto.conf";
const QUARANTINED_POSTGRESQL_AUTO_CONF_NAME: &str = "pgtm.unmanaged.postgresql.auto.conf";

const RESERVED_EXTRA_GUC_KEYS: &[&str] = &[
    "archive_cleanup_command",
    "config_file",
    "hba_file",
    "hot_standby",
    "ident_file",
    "listen_addresses",
    "log_destination",
    "logging_collector",
    "port",
    "primary_conninfo",
    "primary_slot_name",
    "promote_trigger_file",
    "recovery_end_command",
    "recovery_min_apply_delay",
    "recovery_target",
    "recovery_target_action",
    "recovery_target_inclusive",
    "recovery_target_lsn",
    "recovery_target_name",
    "recovery_target_time",
    "recovery_target_timeline",
    "recovery_target_xid",
    "restore_command",
    "ssl",
    "ssl_ca_file",
    "ssl_cert_file",
    "ssl_key_file",
    "trigger_file",
    "unix_socket_directories",
];

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(crate) enum ManagedPostgresError {
    #[error("io error: {message}")]
    Io { message: String },
    #[error("invalid config: {message}")]
    InvalidConfig { message: String },
    #[error("invalid managed postgres state: {message}")]
    InvalidManagedState { message: String },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ManagedRecoverySignal {
    None,
    Standby,
    Recovery,
}

#[derive(Clone, Copy)]
struct ManagedRenderPaths<'a> {
    hba: &'a Path,
    ident: &'a Path,
    standby_passfile: &'a Path,
}

pub(crate) fn materialize_managed_postgres_config(
    cfg: &RuntimeConfigV2,
    tracked_job_kind: ProcessJobKind,
    primary_conninfo: Option<&PgConnInfo>,
    primary_slot_name: Option<&str>,
) -> Result<(), ManagedPostgresError> {
    let data_dir = cfg.postgres.data_dir.as_path();
    if data_dir.as_os_str().is_empty() {
        return Err(ManagedPostgresError::InvalidConfig {
            message: "postgres.data_dir must not be empty".to_string(),
        });
    }

    let managed_hba = absolutize_path(&cfg.postgres.pg_hba_file)?;
    let managed_ident = absolutize_path(&cfg.postgres.pg_ident_file)?;
    let managed_postgresql_conf = absolutize_path(&managed_postgresql_conf_path(data_dir))?;
    let managed_standby_passfile = absolutize_path(&managed_standby_passfile_path(data_dir))?;
    let standby_signal = absolutize_path(&managed_standby_signal_path(data_dir))?;
    let recovery_signal = absolutize_path(&managed_recovery_signal_path(data_dir))?;
    let postgresql_auto_conf = absolutize_path(&managed_postgresql_auto_conf_path(data_dir))?;
    let quarantined_postgresql_auto_conf =
        absolutize_path(&quarantined_postgresql_auto_conf_path(data_dir))?;

    write_atomic(
        &managed_hba,
        cfg.postgres.pg_hba_contents.as_bytes(),
        Some(0o644),
    )?;
    write_atomic(
        &managed_ident,
        cfg.postgres.pg_ident_contents.as_bytes(),
        Some(0o644),
    )?;

    let managed_tls_config = managed_tls_config(cfg)?;
    materialize_managed_standby_passfile(
        cfg,
        tracked_job_kind,
        primary_conninfo,
        managed_standby_passfile.as_path(),
    )?;
    let render_paths = ManagedRenderPaths {
        hba: managed_hba.as_path(),
        ident: managed_ident.as_path(),
        standby_passfile: managed_standby_passfile.as_path(),
    };
    let rendered_conf = render_managed_postgres_conf(
        cfg,
        tracked_job_kind,
        primary_conninfo,
        primary_slot_name,
        &render_paths,
        managed_tls_config.as_ref(),
    )?;
    write_atomic(
        &managed_postgresql_conf,
        rendered_conf.as_bytes(),
        Some(0o644),
    )?;

    quarantine_postgresql_auto_conf(&postgresql_auto_conf, &quarantined_postgresql_auto_conf)?;
    materialize_recovery_signal_files(
        managed_recovery_signal_for_start_job(tracked_job_kind)?,
        &standby_signal,
        &recovery_signal,
    )?;

    Ok(())
}

pub(crate) fn inspect_managed_recovery_state(
    data_dir: &Path,
) -> Result<ManagedRecoverySignal, ManagedPostgresError> {
    existing_recovery_signal(data_dir).map(|state| state.unwrap_or(ManagedRecoverySignal::None))
}

fn managed_tls_config(cfg: &RuntimeConfigV2) -> Result<Option<TlsConfig>, ManagedPostgresError> {
    match &cfg.postgres.tls {
        None => Ok(None),
        Some(tls) => Ok(Some(TlsConfig {
            cert: resolve_existing_configured_file(
                "postgres.tls.identity.cert_chain",
                tls.cert.as_path(),
            )?,
            key: resolve_existing_configured_file(
                "postgres.tls.identity.private_key",
                tls.key.as_path(),
            )?,
            ca_cert: tls
                .ca_cert
                .as_ref()
                .map(|path| {
                    resolve_existing_configured_file(
                        "postgres.tls.client_auth.client_ca",
                        path.as_path(),
                    )
                })
                .transpose()?,
        })),
    }
}

fn resolve_existing_configured_file(
    field: &str,
    path: &Path,
) -> Result<PathBuf, ManagedPostgresError> {
    let path = absolutize_path(path)?;
    match fs::metadata(&path) {
        Ok(metadata) if metadata.is_file() => Ok(path),
        Ok(_) => Err(ManagedPostgresError::InvalidConfig {
            message: format!("{field} must point to a file: {}", path.display()),
        }),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            Err(ManagedPostgresError::InvalidConfig {
                message: format!("{field} points to a missing file: {}", path.display()),
            })
        }
        Err(err) => Err(ManagedPostgresError::Io {
            message: format!("failed to stat {}: {err}", path.display()),
        }),
    }
}

fn materialize_managed_standby_passfile(
    cfg: &RuntimeConfigV2,
    tracked_job_kind: ProcessJobKind,
    primary_conninfo: Option<&PgConnInfo>,
    managed_passfile_path: &Path,
) -> Result<Option<PathBuf>, ManagedPostgresError> {
    let primary_conninfo = match tracked_job_kind {
        ProcessJobKind::StartPrimary | ProcessJobKind::StartDetachedStandby => {
            remove_file_if_exists(managed_passfile_path)?;
            return Ok(None);
        }
        ProcessJobKind::StartReplica => {
            primary_conninfo.ok_or_else(|| ManagedPostgresError::InvalidConfig {
                message: "replica start requires primary_conninfo".to_string(),
            })?
        }
        other => {
            return Err(ManagedPostgresError::InvalidConfig {
                message: format!(
                    "managed postgres config requires a start job kind, got `{}`",
                    other.as_str()
                ),
            });
        }
    };

    let password = cfg.postgres.replicator.password.as_str().to_string();
    let rendered = render_libpq_passfile_entry(primary_conninfo, password.as_str())?;
    write_atomic(managed_passfile_path, rendered.as_bytes(), Some(0o600))?;
    Ok(Some(managed_passfile_path.to_path_buf()))
}

fn render_libpq_passfile_entry(
    conninfo: &PgConnInfo,
    password: &str,
) -> Result<String, ManagedPostgresError> {
    const STREAMING_REPLICATION_DATABASE: &str = "replication";
    let (host, port) = passfile_target_fields(conninfo.route.endpoint());

    if [
        host.as_str(),
        conninfo.user.as_str(),
        password,
        STREAMING_REPLICATION_DATABASE,
    ]
    .iter()
    .any(|value| value.chars().any(|ch| ch == '\n' || ch == '\r'))
    {
        return Err(ManagedPostgresError::InvalidConfig {
            message: "managed standby passfile fields must not contain newlines".to_string(),
        });
    }

    Ok(format!(
        "{}:{}:{}:{}:{}\n",
        escape_libpq_passfile_field(host.as_str()),
        port,
        STREAMING_REPLICATION_DATABASE,
        escape_libpq_passfile_field(conninfo.user.as_str()),
        escape_libpq_passfile_field(password),
    ))
}

fn passfile_target_fields(endpoint: &crate::state::PgEndpoint) -> (String, u16) {
    match endpoint {
        crate::state::PgEndpoint::Tcp { host, port } => (host.clone(), *port),
        crate::state::PgEndpoint::UnixSocket { socket_dir, port } => {
            (socket_dir.display().to_string(), *port)
        }
    }
}

fn escape_libpq_passfile_field(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            ':' | '\\' => {
                escaped.push('\\');
                escaped.push(ch);
            }
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn render_managed_postgres_conf(
    cfg: &RuntimeConfigV2,
    tracked_job_kind: ProcessJobKind,
    primary_conninfo: Option<&PgConnInfo>,
    primary_slot_name: Option<&str>,
    paths: &ManagedRenderPaths<'_>,
    managed_tls_config: Option<&TlsConfig>,
) -> Result<String, ManagedPostgresError> {
    let mut rendered = String::from(MANAGED_POSTGRESQL_CONF_HEADER);

    push_string_setting(
        &mut rendered,
        "listen_addresses",
        cfg.postgres.listen_host.as_str(),
    );
    push_u16_setting(&mut rendered, "port", cfg.postgres.listen_port);
    push_path_setting(
        &mut rendered,
        "unix_socket_directories",
        cfg.postgres.socket_dir.as_path(),
    );
    push_path_setting(&mut rendered, "hba_file", paths.hba);
    push_path_setting(&mut rendered, "ident_file", paths.ident);
    push_bool_setting(&mut rendered, "logging_collector", true);
    push_string_setting(&mut rendered, "log_destination", "jsonlog,stderr");

    match managed_tls_config {
        None => {
            push_bool_setting(&mut rendered, "ssl", false);
        }
        Some(tls) => {
            push_bool_setting(&mut rendered, "ssl", true);
            push_path_setting(&mut rendered, "ssl_cert_file", tls.cert.as_path());
            push_path_setting(&mut rendered, "ssl_key_file", tls.key.as_path());
            if let Some(path) = tls.ca_cert.as_ref() {
                push_path_setting(&mut rendered, "ssl_ca_file", path.as_path());
            }
        }
    }

    match tracked_job_kind {
        ProcessJobKind::StartPrimary => {
            reject_replica_source_fields(primary_conninfo, primary_slot_name)?;
            push_bool_setting(&mut rendered, "hot_standby", false);
        }
        ProcessJobKind::StartDetachedStandby => {
            reject_replica_source_fields(primary_conninfo, primary_slot_name)?;
            push_bool_setting(&mut rendered, "hot_standby", true);
        }
        ProcessJobKind::StartReplica => {
            let primary_conninfo =
                primary_conninfo.ok_or_else(|| ManagedPostgresError::InvalidConfig {
                    message: "replica start requires primary_conninfo".to_string(),
                })?;
            push_bool_setting(&mut rendered, "hot_standby", true);
            let mut primary_conninfo_with_passfile = primary_conninfo.to_string();
            primary_conninfo_with_passfile.push(' ');
            primary_conninfo_with_passfile.push_str("passfile=");
            primary_conninfo_with_passfile.push_str(
                render_conninfo_value(paths.standby_passfile.display().to_string().as_str())
                    .as_str(),
            );
            push_string_setting(
                &mut rendered,
                "primary_conninfo",
                primary_conninfo_with_passfile.as_str(),
            );
            if let Some(slot) = primary_slot_name {
                validate_primary_slot_name(slot)?;
                push_string_setting(&mut rendered, "primary_slot_name", slot);
            }
        }
        other => {
            return Err(ManagedPostgresError::InvalidConfig {
                message: format!(
                    "managed postgres config requires a start job kind, got `{}`",
                    other.as_str()
                ),
            });
        }
    }

    for (key, value) in &cfg.postgres.extra_gucs {
        validate_extra_guc_entry(key.as_str(), value.as_str())?;
        push_string_setting(&mut rendered, key.as_str(), value.as_str());
    }

    Ok(rendered)
}

fn managed_recovery_signal_for_start_job(
    job_kind: ProcessJobKind,
) -> Result<ManagedRecoverySignal, ManagedPostgresError> {
    match job_kind {
        ProcessJobKind::StartPrimary => Ok(ManagedRecoverySignal::None),
        ProcessJobKind::StartDetachedStandby | ProcessJobKind::StartReplica => {
            Ok(ManagedRecoverySignal::Standby)
        }
        other => Err(ManagedPostgresError::InvalidConfig {
            message: format!(
                "managed postgres config requires a start job kind, got `{}`",
                other.as_str()
            ),
        }),
    }
}

pub(crate) fn managed_postgresql_conf_path(data_dir: &Path) -> PathBuf {
    data_dir.join(MANAGED_POSTGRESQL_CONF_NAME)
}

pub(crate) fn managed_standby_passfile_path(data_dir: &Path) -> PathBuf {
    data_dir.join(MANAGED_STANDBY_PASSFILE_NAME)
}

fn managed_standby_signal_path(data_dir: &Path) -> PathBuf {
    data_dir.join(MANAGED_STANDBY_SIGNAL_NAME)
}

fn managed_recovery_signal_path(data_dir: &Path) -> PathBuf {
    data_dir.join(MANAGED_RECOVERY_SIGNAL_NAME)
}

fn managed_postgresql_auto_conf_path(data_dir: &Path) -> PathBuf {
    data_dir.join(POSTGRESQL_AUTO_CONF_NAME)
}

fn quarantined_postgresql_auto_conf_path(data_dir: &Path) -> PathBuf {
    data_dir.join(QUARANTINED_POSTGRESQL_AUTO_CONF_NAME)
}

fn validate_extra_guc_entry(key: &str, value: &str) -> Result<(), ManagedPostgresError> {
    validate_extra_guc_name(key)?;
    if value.chars().any(char::is_control) {
        return Err(ManagedPostgresError::InvalidConfig {
            message: format!(
                "postgres.extra_gucs entry `{key}` invalid: value must not contain control characters"
            ),
        });
    }
    Ok(())
}

fn reject_replica_source_fields(
    primary_conninfo: Option<&PgConnInfo>,
    primary_slot_name: Option<&str>,
) -> Result<(), ManagedPostgresError> {
    if primary_conninfo.is_some() || primary_slot_name.is_some() {
        return Err(ManagedPostgresError::InvalidConfig {
            message: "only replica starts may carry primary_conninfo or primary_slot_name"
                .to_string(),
        });
    }
    Ok(())
}

fn validate_extra_guc_name(key: &str) -> Result<(), ManagedPostgresError> {
    if key.is_empty() {
        return Err(ManagedPostgresError::InvalidConfig {
            message: "postgres.extra_gucs entry `` invalid: name must not be empty".to_string(),
        });
    }

    if RESERVED_EXTRA_GUC_KEYS.contains(&key) {
        return Err(ManagedPostgresError::InvalidConfig {
            message: format!("postgres.extra_gucs entry `{key}` is reserved by pgtuskmaster"),
        });
    }

    for component in key.split('.') {
        if component.is_empty() {
            return Err(ManagedPostgresError::InvalidConfig {
                message: format!(
                    "postgres.extra_gucs entry `{key}` invalid: name must not contain empty namespace components"
                ),
            });
        }

        let mut chars = component.chars();
        let Some(first) = chars.next() else {
            return Err(ManagedPostgresError::InvalidConfig {
                message: format!(
                    "postgres.extra_gucs entry `{key}` invalid: name must not contain empty namespace components"
                ),
            });
        };
        if !(first.is_ascii_alphabetic() || first == '_') {
            return Err(ManagedPostgresError::InvalidConfig {
                message: format!(
                    "postgres.extra_gucs entry `{key}` invalid: each namespace component must start with an ASCII letter or underscore"
                ),
            });
        }
        if !chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '$') {
            return Err(ManagedPostgresError::InvalidConfig {
                message: format!(
                    "postgres.extra_gucs entry `{key}` invalid: name may only contain ASCII letters, digits, underscore, dollar sign, and dots"
                ),
            });
        }
    }

    Ok(())
}

fn validate_primary_slot_name(slot: &str) -> Result<(), ManagedPostgresError> {
    if slot.is_empty() {
        return Err(ManagedPostgresError::InvalidConfig {
            message: "managed replica slot `` invalid: slot name must not be empty".to_string(),
        });
    }
    if !slot
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
    {
        return Err(ManagedPostgresError::InvalidConfig {
            message: format!(
                "managed replica slot `{slot}` invalid: slot name may only contain lowercase ASCII letters, digits, and underscore"
            ),
        });
    }
    Ok(())
}

fn push_path_setting(output: &mut String, key: &str, value: &Path) {
    push_string_setting(output, key, value.display().to_string().as_str());
}

fn push_u16_setting(output: &mut String, key: &str, value: u16) {
    output.push_str(key);
    output.push_str(" = ");
    output.push_str(value.to_string().as_str());
    output.push('\n');
}

fn push_bool_setting(output: &mut String, key: &str, value: bool) {
    output.push_str(key);
    output.push_str(" = ");
    output.push_str(if value { "on" } else { "off" });
    output.push('\n');
}

fn push_string_setting(output: &mut String, key: &str, value: &str) {
    output.push_str(key);
    output.push_str(" = '");
    output.push_str(escape_postgres_conf_string(value).as_str());
    output.push_str("'\n");
}

fn escape_postgres_conf_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\'' => escaped.push_str("''"),
            '\\' => escaped.push_str("\\\\"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn existing_recovery_signal(
    data_dir: &Path,
) -> Result<Option<ManagedRecoverySignal>, ManagedPostgresError> {
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
    use std::{
        collections::BTreeMap,
        fs, io,
        path::{Path, PathBuf},
        time::Duration,
    };

    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    use tokio::process::Command;
    use tokio::time::Instant;
    use tokio_postgres::NoTls;

    use crate::{
        config_v2::{managed_postgres_test_config, types::TlsConfig, RuntimeConfigV2},
        dev_support::{
            binaries::require_pg16_bin_for_real_tests,
            namespace::NamespaceGuard,
            pg16::{prepare_pgdata_dir, spawn_pg16, PgHandle, PgInstanceSpec},
            ports::allocate_ports,
        },
        pginfo::{
            conninfo::{PgClientTls, PgSslMode},
            state::PgConnInfo,
        },
        process::jobs::ProcessJobKind,
        state::PgEndpoint,
    };

    use super::{
        inspect_managed_recovery_state, managed_postgresql_auto_conf_path,
        managed_postgresql_conf_path, managed_recovery_signal_for_start_job,
        managed_recovery_signal_path, managed_standby_passfile_path, managed_standby_signal_path,
        materialize_managed_postgres_config, quarantined_postgresql_auto_conf_path,
        render_managed_postgres_conf, validate_extra_guc_entry, ManagedPostgresError,
        ManagedRecoverySignal, ManagedRenderPaths, MANAGED_POSTGRESQL_CONF_HEADER,
        MANAGED_RECOVERY_SIGNAL_NAME,
    };

    fn sample_managed_config(data_dir: PathBuf) -> Result<RuntimeConfigV2, String> {
        managed_postgres_test_config(data_dir).map_err(|err| err.to_string())
    }

    fn sample_render_config() -> Result<RuntimeConfigV2, String> {
        let cfg = sample_managed_config(PathBuf::from("/var/lib/postgresql/data"))?;
        Ok(RuntimeConfigV2 {
            postgres: crate::config_v2::types::PostgresConfig {
                listen_host: "127.0.0.1".to_string(),
                listen_port: 5432,
                socket_dir: PathBuf::from("/tmp/pgtm socket"),
                pg_hba_file: PathBuf::from("/var/lib/postgresql/data/pgtm.pg_hba.conf"),
                pg_ident_file: PathBuf::from("/var/lib/postgresql/data/pgtm.pg_ident.conf"),
                extra_gucs: BTreeMap::from([
                    (
                        "log_line_prefix".to_string(),
                        "%m [%p] leader='node-a'".to_string(),
                    ),
                    (
                        "shared_preload_libraries".to_string(),
                        "pg_stat_statements".to_string(),
                    ),
                ]),
                ..cfg.postgres
            },
            ..cfg
        })
    }

    fn sample_render_replica_conninfo() -> Result<PgConnInfo, String> {
        Ok(PgConnInfo {
            route: crate::state::PgRoute::tcp("leader.internal".to_string(), 5432)?,
            user: "replicator".to_string(),
            dbname: "postgres".to_string(),
            application_name: Some("node-b".to_string()),
            connect_timeout_s: Some(5),
            options: Some("-c wal_receiver_status_interval=5s".to_string()),
            tls: PgClientTls {
                mode: PgSslMode::Require,
                root_cert: Some(PathBuf::from("/etc/pgtuskmaster/tls/client-ca.crt")),
                client_cert: None,
                client_key: None,
            },
        })
    }

    fn sample_render_tls_config() -> TlsConfig {
        TlsConfig {
            cert: PathBuf::from("/etc/pgtuskmaster/tls/server.crt"),
            key: PathBuf::from("/etc/pgtuskmaster/tls/server.key"),
            ca_cert: Some(PathBuf::from("/etc/pgtuskmaster/tls/client-ca.crt")),
        }
    }

    fn render_sample_conf() -> Result<String, String> {
        let cfg = sample_render_config()?;
        let tls = sample_render_tls_config();
        let primary_conninfo = sample_render_replica_conninfo()?;
        render_managed_postgres_conf(
            &cfg,
            ProcessJobKind::StartReplica,
            Some(&primary_conninfo),
            Some("slot_a"),
            &ManagedRenderPaths {
                hba: cfg.postgres.pg_hba_file.as_path(),
                ident: cfg.postgres.pg_ident_file.as_path(),
                standby_passfile: managed_standby_passfile_path(cfg.postgres.data_dir.as_path())
                    .as_path(),
            },
            Some(&tls),
        )
        .map_err(|err| format!("render failed: {err}"))
    }

    #[test]
    fn render_managed_postgres_conf_is_deterministic() -> Result<(), String> {
        let a = render_sample_conf()?;
        let b = render_sample_conf()?;
        assert_eq!(a, b);
        Ok(())
    }

    #[test]
    fn render_managed_postgres_conf_keeps_owned_settings_before_extra_gucs() -> Result<(), String> {
        let rendered = render_sample_conf()?;
        let primary_slot_index = rendered
            .find("primary_slot_name =")
            .ok_or_else(|| "missing primary_slot_name line".to_string())?;
        let extra_index = rendered
            .find("log_line_prefix =")
            .ok_or_else(|| "missing log_line_prefix line".to_string())?;
        if primary_slot_index >= extra_index {
            return Err(format!(
                "expected owned settings before extra gucs: primary_slot_index={primary_slot_index} extra_index={extra_index}"
            ));
        }
        Ok(())
    }

    #[test]
    fn render_managed_postgres_conf_sorts_extra_gucs() -> Result<(), String> {
        let rendered = render_sample_conf()?;
        let first = rendered
            .find("log_line_prefix =")
            .ok_or_else(|| "missing log_line_prefix".to_string())?;
        let second = rendered
            .find("shared_preload_libraries =")
            .ok_or_else(|| "missing shared_preload_libraries".to_string())?;
        if first >= second {
            return Err(format!(
                "expected sorted extra gucs order: first={first} second={second}"
            ));
        }
        Ok(())
    }

    #[test]
    fn render_managed_postgres_conf_quotes_and_escapes_string_values() -> Result<(), String> {
        let rendered = render_sample_conf()?;
        if !rendered.contains("unix_socket_directories = '/tmp/pgtm socket'") {
            return Err(format!(
                "missing quoted socket dir in rendered conf: {rendered}"
            ));
        }
        if !rendered.contains("log_line_prefix = '%m [%p] leader=''node-a'''") {
            return Err(format!(
                "missing escaped quoted log_line_prefix in rendered conf: {rendered}"
            ));
        }
        if !rendered.contains(
            "primary_conninfo = 'host=leader.internal port=5432 user=replicator dbname=postgres application_name=node-b connect_timeout=5 sslmode=require sslrootcert=/etc/pgtuskmaster/tls/client-ca.crt options=''-c wal_receiver_status_interval=5s'' passfile=/var/lib/postgresql/data/pgtm.standby.passfile'",
        ) {
            return Err(format!(
                "missing quoted primary_conninfo in rendered conf: {rendered}"
            ));
        }
        Ok(())
    }

    #[test]
    fn render_managed_postgres_conf_renders_booleans_and_replica_fields() -> Result<(), String> {
        let rendered = render_sample_conf()?;
        if !rendered.starts_with(MANAGED_POSTGRESQL_CONF_HEADER) {
            return Err(format!("missing managed header: {rendered}"));
        }
        if !rendered.contains("logging_collector = on") {
            return Err(format!("missing logging_collector=on: {rendered}"));
        }
        if !rendered.contains("log_destination = 'jsonlog,stderr'") {
            return Err(format!("missing jsonlog destination: {rendered}"));
        }
        if !rendered.contains("ssl = on") {
            return Err(format!("missing ssl=on: {rendered}"));
        }
        if !rendered.contains("hot_standby = on") {
            return Err(format!("missing hot_standby=on: {rendered}"));
        }
        if !rendered.contains("primary_slot_name = 'slot_a'") {
            return Err(format!("missing primary_slot_name: {rendered}"));
        }
        Ok(())
    }

    #[test]
    fn render_managed_postgres_conf_renders_primary_without_replica_only_fields(
    ) -> Result<(), String> {
        let cfg = sample_render_config()?;
        let rendered = render_managed_postgres_conf(
            &cfg,
            ProcessJobKind::StartPrimary,
            None,
            None,
            &ManagedRenderPaths {
                hba: cfg.postgres.pg_hba_file.as_path(),
                ident: cfg.postgres.pg_ident_file.as_path(),
                standby_passfile: managed_standby_passfile_path(cfg.postgres.data_dir.as_path())
                    .as_path(),
            },
            None,
        )
        .map_err(|err| format!("render failed: {err}"))?;
        if !rendered.contains("ssl = off") {
            return Err(format!("missing ssl=off: {rendered}"));
        }
        if !rendered.contains("hot_standby = off") {
            return Err(format!("missing hot_standby=off: {rendered}"));
        }
        if rendered.contains("primary_conninfo") || rendered.contains("primary_slot_name") {
            return Err(format!(
                "primary config unexpectedly rendered replica fields: {rendered}"
            ));
        }
        Ok(())
    }

    #[test]
    fn render_managed_postgres_conf_renders_detached_standby_without_source_fields(
    ) -> Result<(), String> {
        let cfg = sample_render_config()?;
        let tls = sample_render_tls_config();
        let rendered = render_managed_postgres_conf(
            &cfg,
            ProcessJobKind::StartDetachedStandby,
            None,
            None,
            &ManagedRenderPaths {
                hba: cfg.postgres.pg_hba_file.as_path(),
                ident: cfg.postgres.pg_ident_file.as_path(),
                standby_passfile: managed_standby_passfile_path(cfg.postgres.data_dir.as_path())
                    .as_path(),
            },
            Some(&tls),
        )
        .map_err(|err| format!("render failed: {err}"))?;
        if !rendered.contains("hot_standby = on") {
            return Err(format!("missing hot_standby=on: {rendered}"));
        }
        if rendered.contains("primary_conninfo") || rendered.contains("primary_slot_name") {
            return Err(format!(
                "detached standby config unexpectedly rendered replica source fields: {rendered}"
            ));
        }
        Ok(())
    }

    #[test]
    fn start_job_kind_tracks_recovery_signal_state() -> Result<(), String> {
        assert_eq!(
            managed_recovery_signal_for_start_job(ProcessJobKind::StartPrimary)
                .map_err(|err| err.to_string())?,
            ManagedRecoverySignal::None
        );
        assert_eq!(
            managed_recovery_signal_for_start_job(ProcessJobKind::StartDetachedStandby)
                .map_err(|err| err.to_string())?,
            ManagedRecoverySignal::Standby
        );
        assert_eq!(
            managed_recovery_signal_for_start_job(ProcessJobKind::StartReplica)
                .map_err(|err| err.to_string())?,
            ManagedRecoverySignal::Standby
        );
        Ok(())
    }

    #[test]
    fn validate_extra_guc_entry_rejects_reserved_keys() {
        assert_eq!(
            validate_extra_guc_entry("port", "5432"),
            Err(ManagedPostgresError::InvalidConfig {
                message: "postgres.extra_gucs entry `port` is reserved by pgtuskmaster".to_string(),
            })
        );
        assert_eq!(
            validate_extra_guc_entry("log_destination", "stderr"),
            Err(ManagedPostgresError::InvalidConfig {
                message: "postgres.extra_gucs entry `log_destination` is reserved by pgtuskmaster"
                    .to_string(),
            })
        );
    }

    #[test]
    fn validate_extra_guc_entry_rejects_invalid_names() {
        assert_eq!(
            validate_extra_guc_entry("invalid-name", "on"),
            Err(ManagedPostgresError::InvalidConfig {
                message:
                    "postgres.extra_gucs entry `invalid-name` invalid: name may only contain ASCII letters, digits, underscore, dollar sign, and dots"
                        .to_string(),
            })
        );
    }

    #[test]
    fn validate_extra_guc_entry_rejects_control_characters_in_values() {
        assert_eq!(
            validate_extra_guc_entry("application_name", "node-a\nnode-b"),
            Err(ManagedPostgresError::InvalidConfig {
                message:
                    "postgres.extra_gucs entry `application_name` invalid: value must not contain control characters"
                        .to_string(),
            })
        );
    }

    #[test]
    fn validate_extra_guc_entry_rejects_recovery_override_keys() {
        assert_eq!(
            validate_extra_guc_entry("restore_command", "cp /archive/%f %p"),
            Err(ManagedPostgresError::InvalidConfig {
                message: "postgres.extra_gucs entry `restore_command` is reserved by pgtuskmaster"
                    .to_string(),
            })
        );
        assert_eq!(
            validate_extra_guc_entry("recovery_target_timeline", "latest"),
            Err(ManagedPostgresError::InvalidConfig {
                message:
                    "postgres.extra_gucs entry `recovery_target_timeline` is reserved by pgtuskmaster"
                        .to_string(),
            })
        );
    }

    #[test]
    fn materialize_managed_postgres_config_creates_authoritative_postgresql_conf(
    ) -> Result<(), String> {
        let data_dir = unique_test_data_dir("postgresql-conf");
        let cfg = sample_managed_config(data_dir.clone())?;
        let postgresql_conf_path = managed_postgresql_conf_path(data_dir.as_path());

        materialize_managed_postgres_config(&cfg, ProcessJobKind::StartPrimary, None, None)
            .map_err(|err| format!("materialize managed config failed: {err}"))?;

        let postgresql_conf = fs::read_to_string(&postgresql_conf_path).map_err(|err| {
            format!(
                "read managed postgresql conf {} failed: {err}",
                postgresql_conf_path.display()
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
        if !postgresql_conf.contains("logging_collector = on") {
            return Err(format!(
                "managed postgresql conf missing logging_collector=on: {postgresql_conf}"
            ));
        }
        if !postgresql_conf.contains("log_destination = 'jsonlog,stderr'") {
            return Err(format!(
                "managed postgresql conf missing jsonlog destination: {postgresql_conf}"
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
    fn materialize_managed_postgres_config_creates_and_removes_standby_signal() -> Result<(), String>
    {
        let data_dir = unique_test_data_dir("standby-signal");
        let cfg = sample_managed_config(data_dir.clone())?;
        let standby_signal_path = managed_standby_signal_path(data_dir.as_path());
        let recovery_signal_path = managed_recovery_signal_path(data_dir.as_path());
        let replica_primary_conninfo = PgConnInfo {
            route: crate::state::PgRoute::new(tcp_connect_target("leader.internal", 5432)?, None),
            user: "replicator".to_string(),
            dbname: "postgres".to_string(),
            application_name: None,
            connect_timeout_s: Some(5),
            options: None,
            tls: PgClientTls {
                mode: PgSslMode::Prefer,
                root_cert: None,
                client_cert: None,
                client_key: None,
            },
        };

        materialize_managed_postgres_config(
            &cfg,
            ProcessJobKind::StartReplica,
            Some(&replica_primary_conninfo),
            None,
        )
        .map_err(|err| format!("materialize replica config failed: {err}"))?;
        if !standby_signal_path.exists() {
            return Err(format!(
                "expected standby.signal to exist at {}",
                standby_signal_path.display()
            ));
        }
        if recovery_signal_path.exists() {
            return Err(format!(
                "expected recovery.signal to be absent at {}",
                recovery_signal_path.display()
            ));
        }

        materialize_managed_postgres_config(&cfg, ProcessJobKind::StartPrimary, None, None)
            .map_err(|err| format!("materialize primary config failed: {err}"))?;
        if standby_signal_path.exists() {
            return Err(format!(
                "expected standby.signal to be removed at {}",
                standby_signal_path.display()
            ));
        }
        if recovery_signal_path.exists() {
            return Err(format!(
                "expected recovery.signal to be removed at {}",
                recovery_signal_path.display()
            ));
        }

        fs::remove_dir_all(&data_dir)
            .map_err(|err| format!("remove temp dir {} failed: {err}", data_dir.display()))?;
        Ok(())
    }

    #[test]
    fn materialize_managed_postgres_config_writes_managed_standby_passfile() -> Result<(), String> {
        let data_dir = unique_test_data_dir("standby-passfile");
        let cfg = sample_managed_config(data_dir.clone())?;
        let passfile_path = managed_standby_passfile_path(data_dir.as_path());

        materialize_managed_postgres_config(
            &cfg,
            ProcessJobKind::StartReplica,
            Some(&sample_replica_conninfo()?),
            None,
        )
        .map_err(|err| format!("materialize replica config failed: {err}"))?;

        let contents = fs::read_to_string(&passfile_path).map_err(|err| {
            format!(
                "read standby passfile {} failed: {err}",
                passfile_path.display()
            )
        })?;
        if contents != "leader.internal:5432:replication:replicator:secret-password\n" {
            return Err(format!(
                "unexpected standby passfile contents at {}: {contents:?}",
                passfile_path.display()
            ));
        }

        #[cfg(unix)]
        {
            let mode = fs::metadata(&passfile_path)
                .map_err(|err| format!("stat standby passfile failed: {err}"))?
                .permissions()
                .mode()
                & 0o777;
            if mode != 0o600 {
                return Err(format!(
                    "expected standby passfile mode 0600, got {:o} at {}",
                    mode,
                    passfile_path.display()
                ));
            }
        }

        fs::remove_dir_all(&data_dir)
            .map_err(|err| format!("remove temp dir {} failed: {err}", data_dir.display()))?;
        Ok(())
    }

    #[test]
    fn materialize_managed_postgres_config_removes_stale_standby_passfile_on_primary_start(
    ) -> Result<(), String> {
        let data_dir = unique_test_data_dir("stale-standby-passfile");
        let cfg = sample_managed_config(data_dir.clone())?;
        fs::create_dir_all(&data_dir)
            .map_err(|err| format!("create test dir {} failed: {err}", data_dir.display()))?;
        let stale_path = managed_standby_passfile_path(&data_dir);
        fs::write(&stale_path, "stale-password\n").map_err(|err| {
            format!(
                "write stale standby passfile {} failed: {err}",
                stale_path.display()
            )
        })?;

        materialize_managed_postgres_config(&cfg, ProcessJobKind::StartPrimary, None, None)
            .map_err(|err| format!("materialize primary config failed: {err}"))?;
        if stale_path.exists() {
            return Err(format!(
                "expected stale standby passfile to be removed at {}",
                stale_path.display()
            ));
        }

        fs::remove_dir_all(&data_dir)
            .map_err(|err| format!("remove temp dir {} failed: {err}", data_dir.display()))?;
        Ok(())
    }

    #[test]
    fn materialize_managed_postgres_config_quarantines_postgresql_auto_conf() -> Result<(), String>
    {
        let data_dir = unique_test_data_dir("postgresql-auto-conf");
        let cfg = sample_managed_config(data_dir.clone())?;
        let active_auto_conf = managed_postgresql_auto_conf_path(data_dir.as_path());
        let quarantined_auto_conf = quarantined_postgresql_auto_conf_path(data_dir.as_path());
        fs::create_dir_all(&data_dir)
            .map_err(|err| format!("create test dir {} failed: {err}", data_dir.display()))?;
        fs::write(&active_auto_conf, "primary_conninfo = 'stale'\n").map_err(|err| {
            format!(
                "write active auto conf {} failed: {err}",
                active_auto_conf.display()
            )
        })?;
        fs::write(&quarantined_auto_conf, "stale previous quarantine\n").map_err(|err| {
            format!(
                "write quarantined auto conf {} failed: {err}",
                quarantined_auto_conf.display()
            )
        })?;

        materialize_managed_postgres_config(&cfg, ProcessJobKind::StartPrimary, None, None)
            .map_err(|err| format!("materialize primary config failed: {err}"))?;

        if active_auto_conf.exists() {
            return Err(format!(
                "expected active postgresql.auto.conf to be absent at {}",
                active_auto_conf.display()
            ));
        }
        let quarantined = fs::read_to_string(&quarantined_auto_conf).map_err(|err| {
            format!(
                "read quarantined auto conf {} failed: {err}",
                quarantined_auto_conf.display()
            )
        })?;
        if quarantined != "primary_conninfo = 'stale'\n" {
            return Err(format!(
                "unexpected quarantined auto conf contents at {}: {quarantined}",
                quarantined_auto_conf.display()
            ));
        }

        fs::remove_dir_all(&data_dir)
            .map_err(|err| format!("remove temp dir {} failed: {err}", data_dir.display()))?;
        Ok(())
    }

    #[test]
    fn materialize_managed_postgres_config_rejects_reserved_extra_guc() -> Result<(), String> {
        let data_dir = unique_test_data_dir("reserved-extra");
        let mut cfg = sample_managed_config(data_dir.clone())?;
        cfg.postgres
            .extra_gucs
            .insert("config_file".to_string(), "/tmp/override.conf".to_string());

        assert_eq!(
            materialize_managed_postgres_config(&cfg, ProcessJobKind::StartPrimary, None, None,),
            Err(ManagedPostgresError::InvalidConfig {
                message: "postgres.extra_gucs entry `config_file` is reserved by pgtuskmaster"
                    .to_string(),
            })
        );

        let _ = fs::remove_dir_all(&data_dir);
        Ok(())
    }

    #[test]
    fn materialize_managed_postgres_config_reuses_configured_tls_paths_without_copying(
    ) -> Result<(), String> {
        let data_dir = unique_test_data_dir("tls");
        let mut cfg = sample_managed_config(data_dir.clone())?;
        let managed_conf_path = managed_postgresql_conf_path(data_dir.as_path());
        let source_tls_dir = data_dir.join("source-tls");
        fs::create_dir_all(&source_tls_dir).map_err(|err| {
            format!(
                "create source tls dir {} failed: {err}",
                source_tls_dir.display()
            )
        })?;
        let cert = source_tls_dir.join("server.crt");
        let key = source_tls_dir.join("server.key");
        let ca_cert = source_tls_dir.join("client-ca.crt");
        fs::write(&cert, "CERT")
            .map_err(|err| format!("write {} failed: {err}", cert.display()))?;
        fs::write(&key, "KEY").map_err(|err| format!("write {} failed: {err}", key.display()))?;
        fs::write(&ca_cert, "CA")
            .map_err(|err| format!("write {} failed: {err}", ca_cert.display()))?;
        cfg.postgres.tls = Some(crate::config_v2::types::TlsConfig {
            cert: cert.clone(),
            key: key.clone(),
            ca_cert: Some(ca_cert.clone()),
        });

        materialize_managed_postgres_config(&cfg, ProcessJobKind::StartPrimary, None, None)
            .map_err(|err| format!("materialize managed config failed: {err}"))?;
        let managed_conf = fs::read_to_string(&managed_conf_path).map_err(|err| {
            format!(
                "read managed postgresql conf {} failed: {err}",
                managed_conf_path.display()
            )
        })?;

        if !managed_conf.contains(cert.display().to_string().as_str()) {
            return Err(format!(
                "managed config should reference configured cert path {}, got {managed_conf}",
                cert.display()
            ));
        }
        if !managed_conf.contains(key.display().to_string().as_str()) {
            return Err(format!(
                "managed config should reference configured key path {}, got {managed_conf}",
                key.display()
            ));
        }
        if !managed_conf.contains(ca_cert.display().to_string().as_str()) {
            return Err(format!(
                "managed config should reference configured CA path {}, got {managed_conf}",
                ca_cert.display()
            ));
        }
        for forbidden in [
            format!("{}.server.crt", "pgtm"),
            format!("{}.server.key", "pgtm"),
            format!("{}.ca.crt", "pgtm"),
        ] {
            let copied_path = data_dir.join(forbidden);
            if copied_path.exists() {
                return Err(format!(
                    "managed config must not copy TLS material to {}",
                    copied_path.display()
                ));
            }
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
            data_dir: primary_data.clone(),
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
            let (primary_client, primary_connection) =
                tokio_postgres::connect(&primary_dsn, NoTls).await?;
            let primary_connection_task = tokio::spawn(primary_connection);
            primary_client
                .batch_execute(concat!(
                    "CREATE ROLE replicator WITH LOGIN REPLICATION PASSWORD 'secret-password';",
                    "CREATE TABLE IF NOT EXISTS public.passfile_replay_test (",
                    "id integer PRIMARY KEY, note text NOT NULL",
                    ");",
                ))
                .await?;
            append_to_file(
                primary_data.join("pg_hba.conf").as_path(),
                concat!(
                    "\n",
                    "host replication replicator 127.0.0.1/32 scram-sha-256\n",
                ),
            )?;
            let _ = primary_client
                .query_one("SELECT pg_reload_conf()", &[])
                .await?;

            let replica_data = namespace.child_dir("pg16/replica/data");
            let replica_parent = replica_data
                .parent()
                .ok_or_else(|| real_test_error("replica data dir has no parent"))?;
            fs::create_dir_all(replica_parent)?;

            let basebackup_output = Command::new(&basebackup_bin)
                .env("PGPASSWORD", "secret-password")
                .arg("-h")
                .arg("127.0.0.1")
                .arg("-p")
                .arg(primary_port.to_string())
                .arg("-D")
                .arg(&replica_data)
                .arg("-U")
                .arg("replicator")
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

            let active_auto_conf = managed_postgresql_auto_conf_path(replica_data.as_path());
            let quarantined_auto_conf =
                quarantined_postgresql_auto_conf_path(replica_data.as_path());
            let standby_signal_path = managed_standby_signal_path(replica_data.as_path());
            let recovery_signal_path = managed_recovery_signal_path(replica_data.as_path());
            let managed_conf_path = managed_postgresql_conf_path(replica_data.as_path());
            let standby_passfile = managed_standby_passfile_path(replica_data.as_path());

            fs::write(&active_auto_conf, "port = 1\n")?;
            fs::write(replica_data.join(MANAGED_RECOVERY_SIGNAL_NAME), b"")?;

            let replica_socket = namespace.child_dir("run/replica");
            let replica_logs = namespace.child_dir("logs/replica");
            fs::create_dir_all(&replica_socket)?;
            fs::create_dir_all(&replica_logs)?;

            let replica_reservation = allocate_ports(1)?;
            let replica_port = replica_reservation.as_slice()[0];
            drop(replica_reservation);

            let mut runtime_config = sample_managed_config(replica_data.clone())
                .map_err(real_test_error)?;
            runtime_config.postgres.listen_port = replica_port;
            runtime_config.postgres.cluster_advertise =
                crate::state::PgRoute::tcp(
                    runtime_config.postgres.listen_host.clone(),
                    replica_port,
                )
                .map_err(real_test_error)?;
            runtime_config.postgres.socket_dir = replica_socket.clone();
            runtime_config.postgres.log_file = replica_logs.join("managed-postgres.log");
            runtime_config.postgres.pg_hba_contents = concat!(
                "local all all trust\n",
                "host all all 127.0.0.1/32 trust\n",
                "host replication all 127.0.0.1/32 trust\n",
            )
            .to_string();
            let replica_primary_conninfo = PgConnInfo {
                route: crate::state::PgRoute::new(
                    tcp_connect_target("127.0.0.1", primary_port).map_err(real_test_error)?,
                    None,
                ),
                user: "replicator".to_string(),
                dbname: "postgres".to_string(),
                application_name: None,
                connect_timeout_s: Some(5),
                options: None,
                tls: PgClientTls {
                    mode: PgSslMode::Prefer,
                    root_cert: None,
                    client_cert: None,
                    client_key: None,
                },
            };

            materialize_managed_postgres_config(
                &runtime_config,
                ProcessJobKind::StartReplica,
                Some(&replica_primary_conninfo),
                None,
            )
            .map_err(|err| real_test_error(format!("materialize managed config failed: {err}")))?;

            if active_auto_conf.exists() {
                return Err(real_test_error(format!(
                    "expected active postgresql.auto.conf to be absent at {}",
                    active_auto_conf.display()
                )));
            }
            if !quarantined_auto_conf.exists() {
                return Err(real_test_error(format!(
                    "expected quarantined postgresql.auto.conf to exist at {}",
                    quarantined_auto_conf.display()
                )));
            }
            if !standby_signal_path.exists() {
                return Err(real_test_error(format!(
                    "expected standby.signal to exist at {}",
                    standby_signal_path.display()
                )));
            }
            if recovery_signal_path.exists() {
                return Err(real_test_error(format!(
                    "expected recovery.signal to be absent at {}",
                    recovery_signal_path.display()
                )));
            }

            let stdout_file = fs::File::create(replica_logs.join("postgres.stdout.log"))?;
            let stderr_file = fs::File::create(replica_logs.join("postgres.stderr.log"))?;
            let mut replica_child = Command::new(&postgres_bin)
                .arg("-D")
                .arg(&replica_data)
                .arg("-c")
                .arg(format!("config_file={}", managed_conf_path.display()))
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
                if !primary_conninfo_text.contains("passfile=") {
                    return Err(real_test_error(format!(
                        "expected primary_conninfo to include managed passfile, got {}",
                        primary_conninfo_text
                    )));
                }
                if !primary_conninfo_text.contains(standby_passfile.display().to_string().as_str())
                {
                    return Err(real_test_error(format!(
                        "expected primary_conninfo to reference standby passfile {}, got {}",
                        standby_passfile.display(),
                        primary_conninfo_text
                    )));
                }

                let in_recovery = client.query_one("SELECT pg_is_in_recovery()", &[]).await?;
                let in_recovery_flag: bool = in_recovery.get(0);
                if !in_recovery_flag {
                    return Err(real_test_error(
                        "expected cloned node to start in recovery".to_string(),
                    ));
                }

                let passfile_contents = fs::read_to_string(&standby_passfile).map_err(|err| {
                    real_test_error(format!(
                        "read managed standby passfile {} failed: {err}",
                        standby_passfile.display()
                    ))
                })?;
                if !passfile_contents.contains(":replication:replicator:secret-password") {
                    return Err(real_test_error(format!(
                        "expected standby passfile to contain replication-scope replicator credentials, got {:?}",
                        passfile_contents
                    )));
                }

                primary_client
                    .execute(
                        "INSERT INTO public.passfile_replay_test (id, note) VALUES ($1, $2)",
                        &[&1_i32, &"after-startup"],
                    )
                    .await?;
                wait_for_replica_row(
                    &client,
                    "SELECT note FROM public.passfile_replay_test WHERE id = 1",
                    "after-startup",
                    Duration::from_secs(20),
                )
                .await?;

                drop(client);
                connection_task.await??;
                drop(primary_client);
                primary_connection_task.await??;
                Ok(())
            }
            .await;

            let shutdown_result = shutdown_child("replica", &mut replica_child).await;
            match (replica_result, shutdown_result) {
                (Ok(()), Ok(())) => Ok(()),
                (Err(err), Ok(())) => Err(err),
                (Ok(()), Err(err)) => Err(err),
                (Err(err), Err(clean_err)) => Err(real_test_error(format!("{err}; {clean_err}"))),
            }
        }
        .await;

        let shutdown_primary = shutdown_pg_handle("primary", &mut primary).await;
        match (run_result, shutdown_primary) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(err), Ok(())) => Err(err),
            (Ok(()), Err(err)) => Err(err),
            (Err(err), Err(clean_err)) => Err(real_test_error(format!("{err}; {clean_err}"))),
        }
    }

    #[test]
    fn inspect_managed_recovery_state_reports_replica_signal() -> Result<(), String> {
        let data_dir = unique_test_data_dir("inspect-managed-recovery");
        let cfg = sample_managed_config(data_dir.clone())?;
        let expected = sample_replica_conninfo()?;

        materialize_managed_postgres_config(
            &cfg,
            ProcessJobKind::StartReplica,
            Some(&expected),
            Some("slot_a"),
        )
        .map_err(|err| format!("materialize managed replica config failed: {err}"))?;

        let actual = inspect_managed_recovery_state(&data_dir)
            .map_err(|err| format!("inspect managed recovery state failed: {err}"))?;
        if actual != ManagedRecoverySignal::Standby {
            return Err(format!(
                "unexpected managed recovery state: expected={:?} actual={actual:?}",
                ManagedRecoverySignal::Standby
            ));
        }

        fs::remove_dir_all(&data_dir)
            .map_err(|err| format!("remove temp dir {} failed: {err}", data_dir.display()))?;
        Ok(())
    }

    #[test]
    fn inspect_managed_recovery_state_rejects_conflicting_signal_files() -> Result<(), String> {
        let data_dir = unique_test_data_dir("conflicting-signals");
        fs::create_dir_all(&data_dir)
            .map_err(|err| format!("create test dir {} failed: {err}", data_dir.display()))?;
        let standby_signal = data_dir.join("standby.signal");
        let recovery_signal = data_dir.join(MANAGED_RECOVERY_SIGNAL_NAME);
        fs::write(&standby_signal, b"").map_err(|err| {
            format!(
                "write standby.signal {} failed: {err}",
                standby_signal.display()
            )
        })?;
        fs::write(&recovery_signal, b"").map_err(|err| {
            format!(
                "write recovery.signal {} failed: {err}",
                recovery_signal.display()
            )
        })?;

        let actual = inspect_managed_recovery_state(&data_dir);
        match actual {
            Err(ManagedPostgresError::InvalidManagedState { message }) => {
                if !message.contains("conflicting managed recovery signal files") {
                    return Err(format!(
                        "unexpected invalid managed state message: {message}"
                    ));
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

    fn sample_replica_conninfo() -> Result<PgConnInfo, String> {
        sample_replica_conninfo_for_port(5432)
    }

    fn sample_replica_conninfo_for_port(port: u16) -> Result<PgConnInfo, String> {
        Ok(PgConnInfo {
            route: crate::state::PgRoute::new(tcp_connect_target("leader.internal", port)?, None),
            user: "replicator".to_string(),
            dbname: "postgres".to_string(),
            application_name: None,
            connect_timeout_s: Some(5),
            options: None,
            tls: PgClientTls {
                mode: PgSslMode::Prefer,
                root_cert: None,
                client_cert: None,
                client_key: None,
            },
        })
    }

    fn tcp_connect_target(host: &str, port: u16) -> Result<PgEndpoint, String> {
        PgEndpoint::tcp(host.to_string(), port)
    }

    fn append_to_file(path: &Path, contents: &str) -> Result<(), io::Error> {
        let mut file = fs::OpenOptions::new().append(true).open(path)?;
        use std::io::Write;
        file.write_all(contents.as_bytes())?;
        file.sync_all()
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

    async fn wait_for_replica_row(
        client: &tokio_postgres::Client,
        query: &str,
        expected: &str,
        timeout: Duration,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let deadline = Instant::now() + timeout;

        loop {
            let last_outcome = match client.query_opt(query, &[]).await {
                Ok(Some(row)) => {
                    let actual: String = row.get(0);
                    if actual == expected {
                        return Ok(());
                    }
                    format!("unexpected row value {actual:?}")
                }
                Ok(None) => "row not replayed yet".to_string(),
                Err(err) => err.to_string(),
            };

            if Instant::now() >= deadline {
                return Err(real_test_error(format!(
                    "timed out waiting for replica replay; last outcome: {}",
                    last_outcome
                )));
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    async fn shutdown_pg_handle(
        label: &str,
        handle: &mut PgHandle,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        handle
            .shutdown()
            .await
            .map_err(|err| real_test_error(format!("{label} shutdown failed: {err}")))
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
