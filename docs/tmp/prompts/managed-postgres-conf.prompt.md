Answer using only the information in this prompt.
Do not ask to inspect files, browse, run tools, or fetch more context.
If required information is missing, say exactly what is missing.

You are writing prose for an mdBook documentation page.
Return only markdown for the page body requested.
Use ASCII punctuation only.
Do not use em dashes.
Do not invent facts.

[Task]
- Revise an existing reference page so it stays strictly in Diataxis reference form.

[Page path]
- docs/src/reference/managed-postgres-conf.md

[Page goal]
- Reference the managed PostgreSQL config model, primary_conninfo render and parse rules, and extra GUC validation surface.

[Audience]
- Operators and contributors who need accurate repo-backed facts while working with pgtuskmaster.

[User need]
- Consult the machinery surface, data model, constraints, constants, and behavior without being taught procedures or background explanations.

[mdBook context]
- This is an mdBook page under docs/src/reference/.
- Keep headings and lists suitable for mdBook.
- Do not add verification notes, scratch notes, or commentary about how the page was produced.

[Diataxis guidance]
- This page must stay in the reference quadrant: cognition plus application.
- Describe and only describe.
- Structure the page to mirror the machinery, not a guessed workflow.
- Use neutral, technical language.
- Examples are allowed only when they illustrate the surface concisely.
- Do not include step-by-step operations, recommendations, rationale, or explanations of why the design exists.
- If action or explanation seems necessary, keep the page neutral and mention the boundary without turning the page into a how-to or explanation article.

[Required structure]
- Overview\n- Module constants\n- Core types\n- Rendered configuration model\n- Start-intent and recovery-signal mapping\n- Primary conninfo render and parse rules\n- Validation rules

[Output requirements]
- Preserve or improve the title so it matches the machinery.
- Prefer compact tables where they clarify enums, fields, constants, routes, or error variants.
- Include only facts supported by the supplied source excerpts.
- If the current page contains unsupported claims, remove or correct them rather than hedging.

[Existing page to revise]

# Managed PostgreSQL Configuration Reference

The `src/postgres_managed_conf.rs` module defines the managed PostgreSQL configuration model, start-intent types, `primary_conninfo` render and parse helpers, and validation rules for operator-supplied extra GUCs.

## Overview

This module provides:

- Constants for managed file naming and header content
- Enum types for recovery signals, standby authentication, TLS configuration, and start intent
- A managed configuration struct with TLS, start intent, networking, and operator-supplied extra GUCs
- Rendering functions that produce a deterministic `pgtm.postgresql.conf`
- Parse and validation functions for `primary_conninfo` and operator GUCs

## Module Constants

| Constant | Value | Purpose |
|---|---|---|
| `MANAGED_POSTGRESQL_CONF_NAME` | `pgtm.postgresql.conf` | Canonical managed configuration file name |
| `MANAGED_POSTGRESQL_CONF_HEADER` | Multi-line comment string | Declares the file is managed, removes backup-era archive and restore settings, and states that production TLS material must be operator-supplied |
| `MANAGED_STANDBY_SIGNAL_NAME` | `standby.signal` | Standby-mode signal file name |
| `MANAGED_RECOVERY_SIGNAL_NAME` | `recovery.signal` | Recovery-mode signal file name |
| `MANAGED_STANDBY_PASSFILE_NAME` | `pgtm.standby.passfile` | Managed libpq passfile name |

## Core Types

### `ManagedRecoverySignal`

| Variant | Meaning |
|---|---|
| `None` | No recovery signal file |
| `Standby` | `standby.signal` file present |
| `Recovery` | `recovery.signal` file present |

### `ManagedStandbyAuth`

| Variant | Fields | Purpose |
|---|---|---|
| `NoPassword` | none | TLS-based authentication |
| `PasswordPassfile` | `path: PathBuf` | Password authentication via managed libpq passfile |

### `ManagedPrimaryConninfo`

| Field | Type | Purpose |
|---|---|---|
| `conninfo` | `PgConnInfo` | Upstream connection parameters |
| `standby_auth` | `ManagedStandbyAuth` | Standby authentication configuration |

### `ManagedPostgresStartIntent`

| Variant | Fields | Purpose |
|---|---|---|
| `Primary` | none | Primary role, no upstream replication |
| `Replica` | `primary_conninfo: PgConnInfo`, `standby_auth: ManagedStandbyAuth`, `primary_slot_name: Option<String>` | Streaming replica role |
| `Recovery` | `primary_conninfo: PgConnInfo`, `standby_auth: ManagedStandbyAuth`, `primary_slot_name: Option<String>` | PITR/recovery role |

Helper constructors:

- `ManagedPostgresStartIntent::primary()`
- `ManagedPostgresStartIntent::replica(primary_conninfo, standby_auth, primary_slot_name)`
- `ManagedPostgresStartIntent::recovery(primary_conninfo, standby_auth, primary_slot_name)`

### `ManagedPostgresTlsConfig`

| Variant | Fields | Rendered Value |
|---|---|---|
| `Disabled` | none | `ssl = off` |
| `Enabled` | `cert_file: PathBuf`, `key_file: PathBuf`, `ca_file: Option<PathBuf>` | `ssl = on`, `ssl_cert_file`, `ssl_key_file`, and `ssl_ca_file` when present |

### `ManagedPostgresConf`

| Field | Type | Purpose |
|---|---|---|
| `listen_addresses` | `String` | PostgreSQL listen_addresses |
| `port` | `u16` | PostgreSQL port |
| `unix_socket_directories` | `PathBuf` | Unix socket directories |
| `hba_file` | `PathBuf` | Host-based authentication file path |
| `ident_file` | `PathBuf` | Ident authentication file path |
| `tls` | `ManagedPostgresTlsConfig` | TLS configuration |
| `start_intent` | `ManagedPostgresStartIntent` | Start intent and replication role |
| `extra_gucs` | `BTreeMap<String, String>` | Operator-supplied GUCs |

### `ManagedPostgresConfError`

| Variant | Fields | Meaning |
|---|---|---|
| `InvalidExtraGuc` | `key: String`, `message: String` | Extra GUC name or value validation failed |
| `ReservedExtraGuc` | `key: String` | Key is reserved by pgtuskmaster |
| `InvalidPrimarySlotName` | `slot: String`, `message: String` | Primary slot name validation failed |

### `ManagedPrimaryConninfoError`

| Variant | Fields | Meaning |
|---|---|---|
| `Syntax` | `String` | Conninfo parser syntax violation |
| `DuplicateKey` | `String` | Duplicate `passfile` token |
| `InvalidUpstream` | `String` | Remaining upstream conninfo failed `parse_pg_conninfo` |
| `InvalidPassfilePath` | `path: PathBuf`, `message: String` | Passfile path validation failed |

## Rendered Configuration Model

`render_managed_postgres_conf(conf)` produces a complete managed `pgtm.postgresql.conf` with deterministic ordering:

1. `MANAGED_POSTGRESQL_CONF_HEADER`
2. Network settings: `listen_addresses`, `port`, `unix_socket_directories`, `hba_file`, `ident_file`
3. TLS settings based on `tls` variant
4. Role settings derived from `start_intent`
5. Validated `extra_gucs` in sorted key order

### TLS Rendering

| TLS Config | Rendered Lines |
|---|---|
| `Disabled` | `ssl = off` |
| `Enabled { cert_file, key_file, ca_file }` | `ssl = on`, `ssl_cert_file = '...'`, `ssl_key_file = '...'`, and `ssl_ca_file = '...'` when `ca_file` is present |

### Start-Intent Rendering

| Start Intent | Rendered Lines |
|---|---|
| `Primary` | `hot_standby = off`; omits `primary_conninfo` and `primary_slot_name` |
| `Replica { .. }` or `Recovery { .. }` | `hot_standby = on`, `primary_conninfo = '...'`, and `primary_slot_name = '...'` when present |

### Rendering Helpers

| Helper | Behavior |
|---|---|
| `push_string_setting` | Renders `key = 'value'`, doubling single quotes, and escaping backslashes |
| `push_bool_setting` | Renders `on` or `off` |
| `push_u16_setting` | Renders decimal integer form |
| `push_path_setting` | Renders path via `push_string_setting` with display conversion |
| `escape_postgres_conf_string` | Doubles single quotes and escapes backslashes |

## Start-Intent and Recovery-Signal Mapping

`ManagedPostgresStartIntent::recovery_signal()` maps:

| Intent | Signal | Signal File |
|---|---|---|
| `Primary` | `None` | None |
| `Replica { .. }` | `Standby` | `standby.signal` |
| `Recovery { .. }` | `Recovery` | `recovery.signal` |

Helper functions:

- `managed_standby_passfile_path(data_dir)` returns `data_dir.join("pgtm.standby.passfile")`
- `managed_standby_auth_from_role_auth(auth, data_dir)` maps `RoleAuthConfig::Tls` to `NoPassword` and `RoleAuthConfig::Password` to `PasswordPassfile` at the managed standby passfile path

## Primary Conninfo Render and Parse Rules

### `render_managed_primary_conninfo`

`render_managed_primary_conninfo(conninfo, standby_auth)`:

- Starts from `render_pg_conninfo(conninfo)` output
- Appends `passfile='...'` only for `PasswordPassfile { path }` variant
- Quotes values using `render_conninfo_value`

### `parse_managed_primary_conninfo`

`parse_managed_primary_conninfo(input, data_dir)` parses tokens:

- Uses `parse_conninfo_entries` to extract key-value pairs
- Allows at most one `passfile` token; returns `DuplicateKey` error on duplicates
- Validates passfile path with `validate_managed_passfile_path`
- Parses remaining upstream tokens with `parse_pg_conninfo`
- Returns `ManagedStandbyAuth::NoPassword` when no `passfile` token is present

### Passfile Path Validation

`validate_managed_passfile_path(data_dir, passfile_path)` requires:

- Path is absolute
- Path is under the managed data directory
- Path exactly matches `managed_standby_passfile_path(data_dir)`

### Conninfo Cursor Rules

`parse_conninfo_entries(input)` tokenizes using `ManagedConninfoCursor` with these rules:

- Parses whitespace-separated `key=value` pairs
- Rejects whitespace before `=`
- Rejects empty keys
- Rejects empty unquoted values
- Supports backslash escapes in single-quoted values
- Rejects unterminated quoted values and unterminated escape sequences

## Validation Rules

### Reserved Extra GUC Keys

The following keys are reserved and cannot be used in `extra_gucs`:

| Reserved Keys |
|---|
| `archive_cleanup_command` |
| `config_file` |
| `hba_file` |
| `hot_standby` |
| `ident_file` |
| `listen_addresses` |
| `port` |
| `primary_conninfo` |
| `primary_slot_name` |
| `promote_trigger_file` |
| `recovery_end_command` |
| `recovery_min_apply_delay` |
| `recovery_target` |
| `recovery_target_action` |
| `recovery_target_inclusive` |
| `recovery_target_lsn` |
| `recovery_target_name` |
| `recovery_target_time` |
| `recovery_target_timeline` |
| `recovery_target_xid` |
| `restore_command` |
| `ssl` |
| `ssl_ca_file` |
| `ssl_cert_file` |
| `ssl_key_file` |
| `trigger_file` |
| `unix_socket_directories` |

### Extra GUC Name Validation

`validate_extra_guc_name(key)` requires:

- Non-empty key
- No reserved key match
- Non-empty dot-separated namespace components
- Each component must start with ASCII letter or underscore
- Remaining characters limited to ASCII letters, digits, underscore, or dollar sign
- Returns `InvalidExtraGuc` with descriptive message on violation
- Returns `ReservedExtraGuc` when key matches `RESERVED_EXTRA_GUC_KEYS`

### Extra GUC Value Validation

`validate_extra_guc_value(key, value)` rejects control characters, returning `InvalidExtraGuc` with "value must not contain control characters" message.

### Primary Slot Name Validation

`validate_primary_slot_name(slot)` requires:

- Non-empty slot name
- Characters limited to lowercase ASCII letters, digits, and underscore
- Returns `InvalidPrimarySlotName` with descriptive message on violation

[Repo facts and source excerpts]

--- BEGIN FILE: src/postgres_managed_conf.rs ---
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use thiserror::Error;

use crate::config::RoleAuthConfig;
use crate::pginfo::{
    conninfo::parse_pg_conninfo,
    state::{render_pg_conninfo, PgConnInfo},
};

pub(crate) const MANAGED_POSTGRESQL_CONF_NAME: &str = "pgtm.postgresql.conf";
pub(crate) const MANAGED_POSTGRESQL_CONF_HEADER: &str = "\
# This file is managed by pgtuskmaster.\n\
# Backup-era archive and restore settings have been removed.\n\
# Production TLS material must be supplied by the operator; pgtuskmaster only copies managed runtime files.\n";
pub(crate) const MANAGED_STANDBY_SIGNAL_NAME: &str = "standby.signal";
pub(crate) const MANAGED_RECOVERY_SIGNAL_NAME: &str = "recovery.signal";
pub(crate) const MANAGED_STANDBY_PASSFILE_NAME: &str = "pgtm.standby.passfile";

const RESERVED_EXTRA_GUC_KEYS: &[&str] = &[
    "archive_cleanup_command",
    "config_file",
    "hba_file",
    "hot_standby",
    "ident_file",
    "listen_addresses",
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
    NoPassword,
    PasswordPassfile { path: PathBuf },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ManagedPrimaryConninfo {
    pub(crate) conninfo: PgConnInfo,
    pub(crate) standby_auth: ManagedStandbyAuth,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ManagedPostgresStartIntent {
    Primary,
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
            Self::Replica { .. } => ManagedRecoverySignal::Standby,
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

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(crate) enum ManagedPrimaryConninfoError {
    #[error("managed primary_conninfo syntax error: {0}")]
    Syntax(String),
    #[error("managed primary_conninfo duplicate key `{0}`")]
    DuplicateKey(String),
    #[error("managed primary_conninfo invalid upstream conninfo: {0}")]
    InvalidUpstream(String),
    #[error("managed primary_conninfo invalid passfile `{path}`: {message}")]
    InvalidPassfilePath { path: PathBuf, message: String },
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
        RoleAuthConfig::Tls => ManagedStandbyAuth::NoPassword,
        RoleAuthConfig::Password { .. } => ManagedStandbyAuth::PasswordPassfile {
            path: managed_standby_passfile_path(data_dir),
        },
    }
}

pub(crate) fn render_managed_primary_conninfo(
    conninfo: &PgConnInfo,
    standby_auth: &ManagedStandbyAuth,
) -> String {
    let mut rendered = render_pg_conninfo(conninfo);
    if let ManagedStandbyAuth::PasswordPassfile { path } = standby_auth {
        rendered.push(' ');
        rendered.push_str("passfile=");
        rendered.push_str(render_conninfo_value(path.display().to_string().as_str()).as_str());
    }
    rendered
}

pub(crate) fn parse_managed_primary_conninfo(
    input: &str,
    data_dir: &Path,
) -> Result<ManagedPrimaryConninfo, ManagedPrimaryConninfoError> {
    let entries = parse_conninfo_entries(input)?;
    let mut passfile = None;
    let mut upstream_tokens = Vec::with_capacity(entries.len());

    for (key, value) in entries {
        if key == "passfile" {
            if passfile.is_some() {
                return Err(ManagedPrimaryConninfoError::DuplicateKey(key));
            }
            passfile = Some(validate_managed_passfile_path(data_dir, PathBuf::from(value))?);
        } else {
            upstream_tokens.push(format!("{key}={}", render_conninfo_value(value.as_str())));
        }
    }

    let upstream = parse_pg_conninfo(upstream_tokens.join(" ").as_str())
        .map_err(|err| ManagedPrimaryConninfoError::InvalidUpstream(err.to_string()))?;
    let standby_auth = match passfile {
        Some(path) => ManagedStandbyAuth::PasswordPassfile { path },
        None => ManagedStandbyAuth::NoPassword,
    };

    Ok(ManagedPrimaryConninfo {
        conninfo: upstream,
        standby_auth,
    })
}

pub(crate) fn validate_extra_guc_name(key: &str) -> Result<(), ManagedPostgresConfError> {
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

fn validate_managed_passfile_path(
    data_dir: &Path,
    passfile_path: PathBuf,
) -> Result<PathBuf, ManagedPrimaryConninfoError> {
    if !passfile_path.is_absolute() {
        return Err(ManagedPrimaryConninfoError::InvalidPassfilePath {
            path: passfile_path,
            message: "passfile path must be absolute".to_string(),
        });
    }

    let absolute_data_dir = absolutize_path(data_dir)?;
    let expected_path = absolutize_path(&managed_standby_passfile_path(&absolute_data_dir))?;
    if !passfile_path.starts_with(&absolute_data_dir) {
        return Err(ManagedPrimaryConninfoError::InvalidPassfilePath {
            path: passfile_path,
            message: format!(
                "passfile path must stay under managed data dir {}",
                absolute_data_dir.display()
            ),
        });
    }

    if passfile_path != expected_path {
        return Err(ManagedPrimaryConninfoError::InvalidPassfilePath {
            path: passfile_path,
            message: format!(
                "passfile path must match expected managed path {}",
                expected_path.display()
            ),
        });
    }

    Ok(passfile_path)
}

fn absolutize_path(path: &Path) -> Result<PathBuf, ManagedPrimaryConninfoError> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }

    let cwd = std::env::current_dir().map_err(|err| {
        ManagedPrimaryConninfoError::Syntax(format!("failed to read current_dir: {err}"))
    })?;
    Ok(cwd.join(path))
}

fn parse_conninfo_entries(input: &str) -> Result<Vec<(String, String)>, ManagedPrimaryConninfoError> {
    let mut cursor = ManagedConninfoCursor::new(input);
    let mut entries = Vec::new();

    while cursor.skip_whitespace() {
        let key = cursor.parse_key()?;
        cursor.expect_equals()?;
        let value = cursor.parse_value()?;
        cursor.require_token_boundary()?;
        entries.push((key, value));
    }

    Ok(entries)
}

struct ManagedConninfoCursor<'a> {
    src: &'a str,
    index: usize,
}

impl<'a> ManagedConninfoCursor<'a> {
    fn new(src: &'a str) -> Self {
        Self { src, index: 0 }
    }

    fn skip_whitespace(&mut self) -> bool {
        while let Some(ch) = self.peek_char() {
            if !ch.is_whitespace() {
                break;
            }
            let _ = self.next_char();
        }
        self.index < self.src.len()
    }

    fn parse_key(&mut self) -> Result<String, ManagedPrimaryConninfoError> {
        let mut key = String::new();
        while let Some(ch) = self.peek_char() {
            if ch == '=' {
                break;
            }
            if ch.is_whitespace() {
                return Err(ManagedPrimaryConninfoError::Syntax(
                    "whitespace before '=' in conninfo key/value pair".to_string(),
                ));
            }
            key.push(ch);
            let _ = self.next_char();
        }

        if key.is_empty() {
            return Err(ManagedPrimaryConninfoError::Syntax(
                "empty conninfo key is not allowed".to_string(),
            ));
        }

        Ok(key)
    }

    fn expect_equals(&mut self) -> Result<(), ManagedPrimaryConninfoError> {
        match self.next_char() {
            Some('=') => Ok(()),
            _ => Err(ManagedPrimaryConninfoError::Syntax(
                "expected '=' after conninfo key".to_string(),
            )),
        }
    }

    fn parse_value(&mut self) -> Result<String, ManagedPrimaryConninfoError> {
        match self.peek_char() {
            Some('\'') => self.parse_quoted_value(),
            Some(_) => self.parse_unquoted_value(),
            None => Err(ManagedPrimaryConninfoError::Syntax(
                "missing conninfo value after '='".to_string(),
            )),
        }
    }

    fn parse_unquoted_value(&mut self) -> Result<String, ManagedPrimaryConninfoError> {
        let mut value = String::new();
        while let Some(ch) = self.peek_char() {
            if ch.is_whitespace() {
                break;
            }
            value.push(ch);
            let _ = self.next_char();
        }
        if value.is_empty() {
            return Err(ManagedPrimaryConninfoError::Syntax(
                "conninfo value must not be empty".to_string(),
            ));
        }
        Ok(value)
    }

    fn parse_quoted_value(&mut self) -> Result<String, ManagedPrimaryConninfoError> {
        let _ = self.next_char();
        let mut value = String::new();
        loop {
            match self.next_char() {
                Some('\'') => break,
                Some('\\') => {
                    let Some(next) = self.next_char() else {
                        return Err(ManagedPrimaryConninfoError::Syntax(
                            "unterminated escape sequence in quoted conninfo value".to_string(),
                        ));
                    };
                    value.push(next);
                }
                Some(ch) => value.push(ch),
                None => {
                    return Err(ManagedPrimaryConninfoError::Syntax(
                        "unterminated quoted conninfo value".to_string(),
                    ));
                }
            }
        }
        Ok(value)
    }

    fn require_token_boundary(&mut self) -> Result<(), ManagedPrimaryConninfoError> {
        match self.peek_char() {
            Some(ch) if ch.is_whitespace() => Ok(()),
            None => Ok(()),
            Some(_) => Err(ManagedPrimaryConninfoError::Syntax(
                "conninfo pairs must be separated by whitespace".to_string(),
            )),
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.src[self.index..].chars().next()
    }

    fn next_char(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        self.index += ch.len_utf8();
        Some(ch)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, path::PathBuf};

    use crate::pginfo::state::{PgConnInfo, PgSslMode};

    use super::{
        managed_standby_passfile_path, parse_managed_primary_conninfo,
        render_managed_postgres_conf, validate_extra_guc_entry, ManagedPostgresConf,
        ManagedPostgresConfError, ManagedPostgresStartIntent, ManagedPostgresTlsConfig,
        ManagedPrimaryConninfo, ManagedPrimaryConninfoError, ManagedRecoverySignal,
        ManagedStandbyAuth, MANAGED_POSTGRESQL_CONF_HEADER,
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
                    options: Some("-c wal_receiver_status_interval=5s".to_string()),
                },
                ManagedStandbyAuth::PasswordPassfile {
                    path: managed_standby_passfile_path(PathBuf::from("/var/lib/postgresql/data").as_path()),
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
            "primary_conninfo = 'host=leader.internal port=5432 user=replicator dbname=postgres application_name=node-b connect_timeout=5 sslmode=require options=''-c wal_receiver_status_interval=5s'' passfile=/var/lib/postgresql/data/pgtm.standby.passfile'",
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
    fn managed_start_intent_tracks_recovery_signal_state() {
        assert_eq!(
            ManagedPostgresStartIntent::primary().recovery_signal(),
            ManagedRecoverySignal::None
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
                    options: None,
                },
                ManagedStandbyAuth::NoPassword,
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

    #[test]
    fn parse_managed_primary_conninfo_round_trips_passfile_auth() -> Result<(), String> {
        let data_dir = PathBuf::from("/var/lib/postgresql/data");
        let rendered = "host=leader.internal port=5432 user=replicator dbname=postgres application_name=node-b connect_timeout=5 sslmode=require options='-c wal_receiver_status_interval=5s' passfile=/var/lib/postgresql/data/pgtm.standby.passfile";
        let parsed = parse_managed_primary_conninfo(rendered, &data_dir)
            .map_err(|err| format!("parse failed: {err}"))?;
        assert_eq!(
            parsed,
            ManagedPrimaryConninfo {
                conninfo: PgConnInfo {
                    host: "leader.internal".to_string(),
                    port: 5432,
                    user: "replicator".to_string(),
                    dbname: "postgres".to_string(),
                    application_name: Some("node-b".to_string()),
                    connect_timeout_s: Some(5),
                    ssl_mode: PgSslMode::Require,
                    options: Some("-c wal_receiver_status_interval=5s".to_string()),
                },
                standby_auth: ManagedStandbyAuth::PasswordPassfile {
                    path: data_dir.join("pgtm.standby.passfile"),
                },
            }
        );
        Ok(())
    }

    #[test]
    fn parse_managed_primary_conninfo_rejects_passfile_outside_pgdata() {
        let err = parse_managed_primary_conninfo(
            "host=leader.internal port=5432 user=replicator dbname=postgres passfile=/tmp/bad.pass",
            PathBuf::from("/var/lib/postgresql/data").as_path(),
        );
        assert_eq!(
            err,
            Err(ManagedPrimaryConninfoError::InvalidPassfilePath {
                path: PathBuf::from("/tmp/bad.pass"),
                message:
                    "passfile path must stay under managed data dir /var/lib/postgresql/data"
                        .to_string(),
            })
        );
    }

    #[test]
    fn parse_managed_primary_conninfo_rejects_malformed_quotes() {
        let err = parse_managed_primary_conninfo(
            "host=leader.internal port=5432 user='replicator dbname=postgres",
            PathBuf::from("/var/lib/postgresql/data").as_path(),
        );
        assert_eq!(
            err,
            Err(ManagedPrimaryConninfoError::Syntax(
                "unterminated quoted conninfo value".to_string()
            ))
        );
    }
}

--- END FILE: src/postgres_managed_conf.rs ---

