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
