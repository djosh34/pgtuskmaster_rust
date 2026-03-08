# Managed PostgreSQL Conf Reference

The `src/postgres_managed_conf.rs` module defines the managed PostgreSQL configuration model, start-intent types, `primary_conninfo` render and parse helpers, and validation rules for operator-supplied extra GUCs.

## Module Surface

| Constant | Value | Purpose |
|---|---|---|
| `MANAGED_POSTGRESQL_CONF_NAME` | `pgtm.postgresql.conf` | canonical managed config file name |
| `MANAGED_POSTGRESQL_CONF_HEADER` | multi-line comment header | declares the file is managed, removes backup-era archive and restore settings, and states that production TLS material must be operator-supplied |
| `MANAGED_STANDBY_SIGNAL_NAME` | `standby.signal` | standby-mode signal file name |
| `MANAGED_RECOVERY_SIGNAL_NAME` | `recovery.signal` | recovery-mode signal file name |
| `MANAGED_STANDBY_PASSFILE_NAME` | `pgtm.standby.passfile` | managed libpq passfile name |

## Core Types

### `ManagedRecoverySignal`

| Variant |
|---|
| `None` |
| `Standby` |
| `Recovery` |

### `ManagedStandbyAuth`

| Variant | Fields |
|---|---|
| `NoPassword` | none |
| `PasswordPassfile` | `path: PathBuf` |

### `ManagedPrimaryConninfo`

| Field | Type |
|---|---|
| `conninfo` | `PgConnInfo` |
| `standby_auth` | `ManagedStandbyAuth` |

### `ManagedPostgresStartIntent`

| Variant | Fields |
|---|---|
| `Primary` | none |
| `Replica` | `primary_conninfo: PgConnInfo`, `standby_auth: ManagedStandbyAuth`, `primary_slot_name: Option<String>` |
| `Recovery` | `primary_conninfo: PgConnInfo`, `standby_auth: ManagedStandbyAuth`, `primary_slot_name: Option<String>` |

Helper constructors:

- `ManagedPostgresStartIntent::primary()`
- `ManagedPostgresStartIntent::replica(primary_conninfo, standby_auth, primary_slot_name)`
- `ManagedPostgresStartIntent::recovery(primary_conninfo, standby_auth, primary_slot_name)`

### `ManagedPostgresTlsConfig`

| Variant | Fields |
|---|---|
| `Disabled` | none |
| `Enabled` | `cert_file: PathBuf`, `key_file: PathBuf`, `ca_file: Option<PathBuf>` |

### `ManagedPostgresConf`

| Field | Type |
|---|---|
| `listen_addresses` | `String` |
| `port` | `u16` |
| `unix_socket_directories` | `PathBuf` |
| `hba_file` | `PathBuf` |
| `ident_file` | `PathBuf` |
| `tls` | `ManagedPostgresTlsConfig` |
| `start_intent` | `ManagedPostgresStartIntent` |
| `extra_gucs` | `BTreeMap<String, String>` |

### `ManagedPostgresConfError`

| Variant | Fields | Meaning |
|---|---|---|
| `InvalidExtraGuc` | `key: String`, `message: String` | extra GUC name or value validation failed |
| `ReservedExtraGuc` | `key: String` | key is reserved by pgtuskmaster |
| `InvalidPrimarySlotName` | `slot: String`, `message: String` | primary slot name validation failed |

### `ManagedPrimaryConninfoError`

| Variant | Fields | Meaning |
|---|---|---|
| `Syntax` | `String` | conninfo parser syntax violation |
| `DuplicateKey` | `String` | duplicate `passfile` token |
| `InvalidUpstream` | `String` | remaining upstream conninfo failed `parse_pg_conninfo` |
| `InvalidPassfilePath` | `path: PathBuf`, `message: String` | passfile path validation failed |

## Rendered Configuration Model

`render_managed_postgres_conf(conf)` writes a complete managed `pgtm.postgresql.conf` with deterministic ordering:

1. `MANAGED_POSTGRESQL_CONF_HEADER`
2. `listen_addresses`, `port`, `unix_socket_directories`, `hba_file`, `ident_file`
3. TLS settings
4. role settings from `start_intent`
5. validated extra GUCs in sorted key order

### TLS Rendering

| TLS config | Rendered settings |
|---|---|
| `Disabled` | `ssl = off` |
| `Enabled { cert_file, key_file, ca_file }` | `ssl = on`, `ssl_cert_file`, `ssl_key_file`, and `ssl_ca_file` when `ca_file` is present |

### Start-Intent Rendering

| Start intent | Rendered settings |
|---|---|
| `Primary` | `hot_standby = off`; omits `primary_conninfo` and `primary_slot_name` |
| `Replica { .. }` or `Recovery { .. }` | `hot_standby = on`, `primary_conninfo`, and `primary_slot_name` when present |

### Rendering Helpers

| Helper | Behavior |
|---|---|
| `push_string_setting` | renders `key = 'value'`, doubles single quotes, and escapes backslashes |
| `push_bool_setting` | renders `on` or `off` |
| `push_u16_setting` | renders decimal integer form |

## Start-Intent And Recovery-Signal Mapping

`ManagedPostgresStartIntent::recovery_signal()` maps:

| Intent | Signal |
|---|---|
| `Primary` | `None` |
| `Replica` | `Standby` |
| `Recovery` | `Recovery` |

Helper functions:

| Function | Behavior |
|---|---|
| `managed_standby_passfile_path(data_dir)` | returns `data_dir.join("pgtm.standby.passfile")` |
| `managed_standby_auth_from_role_auth(auth, data_dir)` | maps `RoleAuthConfig::Tls` to `NoPassword` and `RoleAuthConfig::Password` to `PasswordPassfile` at the managed standby passfile path |

## Primary Conninfo Render And Parse Rules

### `render_managed_primary_conninfo`

`render_managed_primary_conninfo(conninfo, standby_auth)`:

- starts from `render_pg_conninfo(conninfo)`
- appends `passfile=...` only for `PasswordPassfile`
- quotes a conninfo value when it is empty or contains whitespace, single quotes, or backslashes

### `parse_managed_primary_conninfo`

`parse_managed_primary_conninfo(input, data_dir)`:

- parses key-value tokens with a custom cursor
- allows at most one `passfile` token and rejects duplicates
- validates the passfile path with `validate_managed_passfile_path`
- parses the remaining upstream tokens with `parse_pg_conninfo`
- returns `ManagedStandbyAuth::NoPassword` when no `passfile` token is present

### Passfile Path Validation

`validate_managed_passfile_path(data_dir, passfile_path)` requires:

- an absolute path
- a path under the managed data directory
- an exact match with `managed_standby_passfile_path(data_dir)`

### Conninfo Cursor Rules

`parse_conninfo_entries(input)`:

- parses whitespace-separated `key=value` tokens
- rejects whitespace before `=`
- rejects empty keys
- rejects empty unquoted values
- supports backslash escapes in single-quoted values
- rejects unterminated quoted values and unterminated escape sequences

## Validation Rules

### Extra GUC Names

`validate_extra_guc_name(key)` requires:

- a non-empty key
- no reserved key match
- non-empty dot-separated namespace components
- each namespace component to start with an ASCII letter or underscore
- remaining component characters limited to ASCII letters, digits, underscore, or dollar sign

Reserved keys include pgtuskmaster-owned and recovery-related settings such as:

- `config_file`
- `hba_file`
- `hot_standby`
- `ident_file`
- `listen_addresses`
- `port`
- `primary_conninfo`
- `primary_slot_name`
- `restore_command`
- `recovery_target_time`
- `recovery_target_timeline`
- `ssl`
- `ssl_ca_file`
- `ssl_cert_file`
- `ssl_key_file`
- `trigger_file`
- `unix_socket_directories`

### Extra GUC Values

`validate_extra_guc_value(key, value)` rejects control characters.

### Primary Slot Names

`validate_primary_slot_name(slot)` rejects:

- empty slot names
- characters outside lowercase ASCII letters, digits, and underscore

## Verified Behaviors

Tests in `src/postgres_managed_conf.rs` verify:

- deterministic managed-conf rendering
- pgtuskmaster-owned settings before extra GUCs
- sorted extra GUC output
- correct quoting and escaping of string values
- correct rendering of booleans and replica-only fields
- omission of replica-only fields for primary intent
- recovery-signal mapping from `ManagedPostgresStartIntent`
- rejection of reserved keys, invalid names, control characters, and recovery override keys
- round-trip parsing of `primary_conninfo` with managed passfile auth
- rejection of passfile paths outside PGDATA
- rejection of malformed quoted conninfo input
