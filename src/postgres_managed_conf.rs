use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use thiserror::Error;

use crate::config::RoleAuthConfig;
use crate::pginfo::state::{render_pg_conninfo, PgConnInfo};

pub(crate) const MANAGED_POSTGRESQL_CONF_NAME: &str = "pgtm.postgresql.conf";
pub(crate) const MANAGED_POSTGRESQL_CONF_HEADER: &str = "\
# This file is managed by pgtuskmaster.\n\
# Backup-era archive and restore settings have been removed.\n\
# Production TLS material must be supplied by the operator; pgtuskmaster only copies managed runtime files.\n";
pub(crate) const MANAGED_STANDBY_SIGNAL_NAME: &str = "standby.signal";
pub(crate) const MANAGED_RECOVERY_SIGNAL_NAME: &str = "recovery.signal";
const MANAGED_STANDBY_PASSFILE_NAME: &str = "pgtm.standby.passfile";

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ManagedRecoverySignal {
    None,
    Standby,
    Recovery,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ManagedStandbyAuth {
    PasswordPassfile { path: PathBuf },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ManagedPostgresStartIntent {
    Primary,
    DetachedStandby,
    Replica {
        primary_conninfo: PgConnInfo,
        standby_auth: ManagedStandbyAuth,
        primary_slot_name: Option<String>,
    },
    Recovery {
        primary_conninfo: PgConnInfo,
        standby_auth: ManagedStandbyAuth,
        primary_slot_name: Option<String>,
    },
}

impl ManagedPostgresStartIntent {
    pub(crate) fn primary() -> Self {
        Self::Primary
    }

    pub(crate) fn detached_standby() -> Self {
        Self::DetachedStandby
    }

    pub(crate) fn replica(
        primary_conninfo: PgConnInfo,
        standby_auth: ManagedStandbyAuth,
        primary_slot_name: Option<String>,
    ) -> Self {
        Self::Replica {
            primary_conninfo,
            standby_auth,
            primary_slot_name,
        }
    }

    pub(crate) fn recovery(
        primary_conninfo: PgConnInfo,
        standby_auth: ManagedStandbyAuth,
        primary_slot_name: Option<String>,
    ) -> Self {
        Self::Recovery {
            primary_conninfo,
            standby_auth,
            primary_slot_name,
        }
    }

    pub(crate) fn recovery_signal(&self) -> ManagedRecoverySignal {
        match self {
            Self::Primary => ManagedRecoverySignal::None,
            Self::DetachedStandby | Self::Replica { .. } => ManagedRecoverySignal::Standby,
            Self::Recovery { .. } => ManagedRecoverySignal::Recovery,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ManagedPostgresTlsConfig {
    Disabled,
    Enabled {
        cert_file: PathBuf,
        key_file: PathBuf,
        ca_file: Option<PathBuf>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ManagedPostgresConf {
    pub(crate) listen_addresses: String,
    pub(crate) port: u16,
    pub(crate) unix_socket_directories: PathBuf,
    pub(crate) hba_file: PathBuf,
    pub(crate) ident_file: PathBuf,
    pub(crate) tls: ManagedPostgresTlsConfig,
    pub(crate) start_intent: ManagedPostgresStartIntent,
    pub(crate) extra_gucs: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(crate) enum ManagedPostgresConfError {
    #[error("invalid extra guc `{key}`: {message}")]
    InvalidExtraGuc { key: String, message: String },
    #[error("extra guc `{key}` is reserved by pgtuskmaster")]
    ReservedExtraGuc { key: String },
    #[error("invalid primary_slot_name `{slot}`: {message}")]
    InvalidPrimarySlotName { slot: String, message: String },
}

pub(crate) fn render_managed_postgres_conf(
    conf: &ManagedPostgresConf,
) -> Result<String, ManagedPostgresConfError> {
    let mut rendered = String::from(MANAGED_POSTGRESQL_CONF_HEADER);

    push_string_setting(
        &mut rendered,
        "listen_addresses",
        conf.listen_addresses.as_str(),
    );
    push_u16_setting(&mut rendered, "port", conf.port);
    push_path_setting(
        &mut rendered,
        "unix_socket_directories",
        conf.unix_socket_directories.as_path(),
    );
    push_path_setting(&mut rendered, "hba_file", conf.hba_file.as_path());
    push_path_setting(&mut rendered, "ident_file", conf.ident_file.as_path());
    push_bool_setting(&mut rendered, "logging_collector", true);
    push_string_setting(&mut rendered, "log_destination", "jsonlog,stderr");

    match &conf.tls {
        ManagedPostgresTlsConfig::Disabled => {
            push_bool_setting(&mut rendered, "ssl", false);
        }
        ManagedPostgresTlsConfig::Enabled {
            cert_file,
            key_file,
            ca_file,
        } => {
            push_bool_setting(&mut rendered, "ssl", true);
            push_path_setting(&mut rendered, "ssl_cert_file", cert_file.as_path());
            push_path_setting(&mut rendered, "ssl_key_file", key_file.as_path());
            if let Some(path) = ca_file.as_ref() {
                push_path_setting(&mut rendered, "ssl_ca_file", path.as_path());
            }
        }
    }

    match &conf.start_intent {
        ManagedPostgresStartIntent::Primary => {
            push_bool_setting(&mut rendered, "hot_standby", false);
        }
        ManagedPostgresStartIntent::DetachedStandby => {
            push_bool_setting(&mut rendered, "hot_standby", true);
        }
        ManagedPostgresStartIntent::Replica {
            primary_conninfo,
            standby_auth,
            primary_slot_name,
        }
        | ManagedPostgresStartIntent::Recovery {
            primary_conninfo,
            standby_auth,
            primary_slot_name,
        } => {
            push_bool_setting(&mut rendered, "hot_standby", true);
            push_string_setting(
                &mut rendered,
                "primary_conninfo",
                render_managed_primary_conninfo(primary_conninfo, standby_auth).as_str(),
            );
            if let Some(slot) = primary_slot_name.as_ref() {
                validate_primary_slot_name(slot.as_str())?;
                push_string_setting(&mut rendered, "primary_slot_name", slot.as_str());
            }
        }
    }

    for (key, value) in &conf.extra_gucs {
        validate_extra_guc_entry(key.as_str(), value.as_str())?;
        push_string_setting(&mut rendered, key.as_str(), value.as_str());
    }

    Ok(rendered)
}

pub(crate) fn validate_extra_guc_entry(
    key: &str,
    value: &str,
) -> Result<(), ManagedPostgresConfError> {
    validate_extra_guc_name(key)?;
    validate_extra_guc_value(key, value)?;
    Ok(())
}

pub(crate) fn managed_standby_passfile_path(data_dir: &Path) -> PathBuf {
    data_dir.join(MANAGED_STANDBY_PASSFILE_NAME)
}

pub(crate) fn managed_standby_auth_from_role_auth(
    auth: &RoleAuthConfig,
    data_dir: &Path,
) -> ManagedStandbyAuth {
    match auth {
        RoleAuthConfig::Password { .. } => ManagedStandbyAuth::PasswordPassfile {
            path: managed_standby_passfile_path(data_dir),
        },
    }
}

fn render_managed_primary_conninfo(
    conninfo: &PgConnInfo,
    standby_auth: &ManagedStandbyAuth,
) -> String {
    let ManagedStandbyAuth::PasswordPassfile { path } = standby_auth;
    let mut rendered = render_pg_conninfo(conninfo);
    rendered.push(' ');
    rendered.push_str("passfile=");
    rendered.push_str(render_conninfo_value(path.display().to_string().as_str()).as_str());
    rendered
}

fn validate_extra_guc_name(key: &str) -> Result<(), ManagedPostgresConfError> {
    if key.is_empty() {
        return Err(ManagedPostgresConfError::InvalidExtraGuc {
            key: key.to_string(),
            message: "name must not be empty".to_string(),
        });
    }

    if RESERVED_EXTRA_GUC_KEYS.contains(&key) {
        return Err(ManagedPostgresConfError::ReservedExtraGuc {
            key: key.to_string(),
        });
    }

    for component in key.split('.') {
        if component.is_empty() {
            return Err(ManagedPostgresConfError::InvalidExtraGuc {
                key: key.to_string(),
                message: "name must not contain empty namespace components".to_string(),
            });
        }

        let mut chars = component.chars();
        let Some(first) = chars.next() else {
            return Err(ManagedPostgresConfError::InvalidExtraGuc {
                key: key.to_string(),
                message: "name must not contain empty namespace components".to_string(),
            });
        };
        if !(first.is_ascii_alphabetic() || first == '_') {
            return Err(ManagedPostgresConfError::InvalidExtraGuc {
                key: key.to_string(),
                message: "each namespace component must start with an ASCII letter or underscore"
                    .to_string(),
            });
        }
        if !chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '$') {
            return Err(ManagedPostgresConfError::InvalidExtraGuc {
                key: key.to_string(),
                message:
                    "name may only contain ASCII letters, digits, underscore, dollar sign, and dots"
                        .to_string(),
            });
        }
    }

    Ok(())
}

fn validate_extra_guc_value(key: &str, value: &str) -> Result<(), ManagedPostgresConfError> {
    if value.chars().any(char::is_control) {
        return Err(ManagedPostgresConfError::InvalidExtraGuc {
            key: key.to_string(),
            message: "value must not contain control characters".to_string(),
        });
    }
    Ok(())
}

fn validate_primary_slot_name(slot: &str) -> Result<(), ManagedPostgresConfError> {
    if slot.is_empty() {
        return Err(ManagedPostgresConfError::InvalidPrimarySlotName {
            slot: slot.to_string(),
            message: "slot name must not be empty".to_string(),
        });
    }
    if !slot
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
    {
        return Err(ManagedPostgresConfError::InvalidPrimarySlotName {
            slot: slot.to_string(),
            message: "slot name may only contain lowercase ASCII letters, digits, and underscore"
                .to_string(),
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

fn render_conninfo_value(value: &str) -> String {
    if value.is_empty()
        || value
            .chars()
            .any(|ch| ch.is_whitespace() || ch == '\'' || ch == '\\')
    {
        let escaped = value
            .chars()
            .map(|ch| match ch {
                '\'' => "\\'".to_string(),
                '\\' => "\\\\".to_string(),
                other => other.to_string(),
            })
            .collect::<String>();
        format!("'{escaped}'")
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, path::PathBuf};

    use crate::pginfo::state::{PgConnInfo, PgSslMode};

    use super::{
        managed_standby_passfile_path, render_managed_postgres_conf, validate_extra_guc_entry,
        ManagedPostgresConf, ManagedPostgresConfError, ManagedPostgresStartIntent,
        ManagedPostgresTlsConfig, ManagedRecoverySignal, ManagedStandbyAuth,
        MANAGED_POSTGRESQL_CONF_HEADER, MANAGED_STANDBY_PASSFILE_NAME,
    };

    fn sample_conf() -> ManagedPostgresConf {
        ManagedPostgresConf {
            listen_addresses: "127.0.0.1".to_string(),
            port: 5432,
            unix_socket_directories: PathBuf::from("/tmp/pgtm socket"),
            hba_file: PathBuf::from("/var/lib/postgresql/data/pgtm.pg_hba.conf"),
            ident_file: PathBuf::from("/var/lib/postgresql/data/pgtm.pg_ident.conf"),
            tls: ManagedPostgresTlsConfig::Enabled {
                cert_file: PathBuf::from("/var/lib/postgresql/data/pgtm.server.crt"),
                key_file: PathBuf::from("/var/lib/postgresql/data/pgtm.server.key"),
                ca_file: Some(PathBuf::from("/var/lib/postgresql/data/pgtm.ca.crt")),
            },
            start_intent: ManagedPostgresStartIntent::replica(
                PgConnInfo {
                    host: "leader.internal".to_string(),
                    port: 5432,
                    user: "replicator".to_string(),
                    dbname: "postgres".to_string(),
                    application_name: Some("node-b".to_string()),
                    connect_timeout_s: Some(5),
                    ssl_mode: PgSslMode::Require,
                    ssl_root_cert: Some(PathBuf::from("/var/lib/postgresql/data/pgtm.ca.crt")),
                    options: Some("-c wal_receiver_status_interval=5s".to_string()),
                },
                ManagedStandbyAuth::PasswordPassfile {
                    path: managed_standby_passfile_path(
                        PathBuf::from("/var/lib/postgresql/data").as_path(),
                    ),
                },
                Some("slot_a".to_string()),
            ),
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
        }
    }

    #[test]
    fn render_managed_postgres_conf_is_deterministic() -> Result<(), String> {
        let a = render_managed_postgres_conf(&sample_conf())
            .map_err(|err| format!("render failed: {err}"))?;
        let b = render_managed_postgres_conf(&sample_conf())
            .map_err(|err| format!("render failed: {err}"))?;
        assert_eq!(a, b);
        Ok(())
    }

    #[test]
    fn render_managed_postgres_conf_keeps_owned_settings_before_extra_gucs() -> Result<(), String> {
        let rendered = render_managed_postgres_conf(&sample_conf())
            .map_err(|err| format!("render failed: {err}"))?;
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
        let rendered = render_managed_postgres_conf(&sample_conf())
            .map_err(|err| format!("render failed: {err}"))?;
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
        let rendered = render_managed_postgres_conf(&sample_conf())
            .map_err(|err| format!("render failed: {err}"))?;
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
            "primary_conninfo = 'host=leader.internal port=5432 user=replicator dbname=postgres application_name=node-b connect_timeout=5 sslmode=require sslrootcert=/var/lib/postgresql/data/pgtm.ca.crt options=''-c wal_receiver_status_interval=5s'' passfile=/var/lib/postgresql/data/pgtm.standby.passfile'",
        ) {
            return Err(format!(
                "missing quoted primary_conninfo in rendered conf: {rendered}"
            ));
        }
        Ok(())
    }

    #[test]
    fn render_managed_postgres_conf_renders_booleans_and_replica_fields() -> Result<(), String> {
        let rendered = render_managed_postgres_conf(&sample_conf())
            .map_err(|err| format!("render failed: {err}"))?;
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
        let mut conf = sample_conf();
        conf.tls = ManagedPostgresTlsConfig::Disabled;
        conf.start_intent = ManagedPostgresStartIntent::Primary;
        let rendered =
            render_managed_postgres_conf(&conf).map_err(|err| format!("render failed: {err}"))?;
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
        let mut conf = sample_conf();
        conf.start_intent = ManagedPostgresStartIntent::detached_standby();
        let rendered =
            render_managed_postgres_conf(&conf).map_err(|err| format!("render failed: {err}"))?;
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
    fn managed_start_intent_tracks_recovery_signal_state() {
        assert_eq!(
            ManagedPostgresStartIntent::primary().recovery_signal(),
            ManagedRecoverySignal::None
        );
        assert_eq!(
            ManagedPostgresStartIntent::detached_standby().recovery_signal(),
            ManagedRecoverySignal::Standby
        );
        assert_eq!(
            sample_conf().start_intent.recovery_signal(),
            ManagedRecoverySignal::Standby
        );
        assert_eq!(
            ManagedPostgresStartIntent::recovery(
                PgConnInfo {
                    host: "leader.internal".to_string(),
                    port: 5432,
                    user: "replicator".to_string(),
                    dbname: "postgres".to_string(),
                    application_name: None,
                    connect_timeout_s: None,
                    ssl_mode: PgSslMode::Prefer,
                    ssl_root_cert: None,
                    options: None,
                },
                ManagedStandbyAuth::PasswordPassfile {
                    path: PathBuf::from("/var/lib/postgresql/data")
                        .join(MANAGED_STANDBY_PASSFILE_NAME),
                },
                None,
            )
            .recovery_signal(),
            ManagedRecoverySignal::Recovery
        );
    }

    #[test]
    fn validate_extra_guc_entry_rejects_reserved_keys() {
        assert_eq!(
            validate_extra_guc_entry("port", "5432"),
            Err(ManagedPostgresConfError::ReservedExtraGuc {
                key: "port".to_string(),
            })
        );
        assert_eq!(
            validate_extra_guc_entry("log_destination", "stderr"),
            Err(ManagedPostgresConfError::ReservedExtraGuc {
                key: "log_destination".to_string(),
            })
        );
    }

    #[test]
    fn validate_extra_guc_entry_rejects_invalid_names() {
        assert_eq!(
            validate_extra_guc_entry("invalid-name", "on"),
            Err(ManagedPostgresConfError::InvalidExtraGuc {
                key: "invalid-name".to_string(),
                message:
                    "name may only contain ASCII letters, digits, underscore, dollar sign, and dots"
                        .to_string(),
            })
        );
    }

    #[test]
    fn validate_extra_guc_entry_rejects_control_characters_in_values() {
        assert_eq!(
            validate_extra_guc_entry("application_name", "node-a\nnode-b"),
            Err(ManagedPostgresConfError::InvalidExtraGuc {
                key: "application_name".to_string(),
                message: "value must not contain control characters".to_string(),
            })
        );
    }

    #[test]
    fn validate_extra_guc_entry_rejects_recovery_override_keys() {
        assert_eq!(
            validate_extra_guc_entry("restore_command", "cp /archive/%f %p"),
            Err(ManagedPostgresConfError::ReservedExtraGuc {
                key: "restore_command".to_string(),
            })
        );
        assert_eq!(
            validate_extra_guc_entry("recovery_target_timeline", "latest"),
            Err(ManagedPostgresConfError::ReservedExtraGuc {
                key: "recovery_target_timeline".to_string(),
            })
        );
    }
}
