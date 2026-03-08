# Managed PostgreSQL Runtime Files

The `src/postgres_managed.rs` module materializes and rereads managed PostgreSQL runtime files under `cfg.postgres.data_dir`. It writes managed config files, TLS assets, standby-auth files, and recovery signal files. It does not start PostgreSQL processes.

## Core types

### `ManagedPostgresError`

| Variant | Fields |
|---|---|
| `Io` | `message: String` |
| `InvalidConfig` | `message: String` |
| `InvalidManagedState` | `message: String` |

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

### `ManagedPostgresTlsConfig`

| Variant | Fields |
|---|---|
| `Disabled` | none |
| `Enabled` | `cert_file: PathBuf`, `key_file: PathBuf`, `ca_file: Option<PathBuf>` |

### `ManagedPostgresStartIntent`

| Variant | Fields |
|---|---|
| `Primary` | none |
| `Replica` | `primary_conninfo: PgConnInfo`, `standby_auth: ManagedStandbyAuth`, `primary_slot_name: Option<String>` |
| `Recovery` | `primary_conninfo: PgConnInfo`, `standby_auth: ManagedStandbyAuth`, `primary_slot_name: Option<String>` |

`ManagedPostgresStartIntent::recovery_signal()` maps variants to `ManagedRecoverySignal`:

| Variant | Signal |
|---|---|
| `Primary` | `None` |
| `Replica` | `Standby` |
| `Recovery` | `Recovery` |

### `ManagedPostgresConfig`

| Field | Type |
|---|---|
| `postgresql_conf_path` | `PathBuf` |
| `hba_path` | `PathBuf` |
| `ident_path` | `PathBuf` |
| `standby_passfile_path` | `Option<PathBuf>` |
| `tls_cert_path` | `Option<PathBuf>` |
| `tls_key_path` | `Option<PathBuf>` |
| `tls_client_ca_path` | `Option<PathBuf>` |
| `standby_signal_path` | `PathBuf` |
| `recovery_signal_path` | `PathBuf` |
| `postgresql_auto_conf_path` | `PathBuf` |
| `quarantined_postgresql_auto_conf_path` | `PathBuf` |

### `ManagedPostgresConf`

Configuration struct rendered into `pgtm.postgresql.conf`.

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

## Managed file set

`materialize_managed_postgres_config(cfg, start_intent)` writes the following under `cfg.postgres.data_dir`:

| Filename | Mode | Purpose |
|---|---|---|
| `pgtm.postgresql.conf` | `0644` | Rendered managed PostgreSQL configuration |
| `pgtm.pg_hba.conf` | `0644` | HBA rules from `postgres.pg_hba.source` |
| `pgtm.pg_ident.conf` | `0644` | Ident rules from `postgres.pg_ident.source` |
| `pgtm.standby.passfile` | `0600` | Managed libpq passfile for `PasswordPassfile` auth |
| `pgtm.server.crt` | `0644` | Managed copy of the PostgreSQL TLS server certificate |
| `pgtm.server.key` | `0600` | Managed copy of the PostgreSQL TLS server private key |
| `pgtm.ca.crt` | `0644` | Managed copy of the client CA when client auth is configured |
| `standby.signal` | not written with `write_atomic` | Standby-mode signal file |
| `recovery.signal` | not written with `write_atomic` | Recovery-mode signal file |
| `postgresql.auto.conf` | existing file | Active PostgreSQL auto-config that may be quarantined |
| `pgtm.unmanaged.postgresql.auto.conf` | rename target | Quarantine target for existing auto-config |

Constants:

- `MANAGED_PG_HBA_CONF_NAME`: `"pgtm.pg_hba.conf"`
- `MANAGED_PG_IDENT_CONF_NAME`: `"pgtm.pg_ident.conf"`
- `POSTGRESQL_AUTO_CONF_NAME`: `"postgresql.auto.conf"`
- `QUARANTINED_POSTGRESQL_AUTO_CONF_NAME`: `"pgtm.unmanaged.postgresql.auto.conf"`

## Materialization pipeline

`materialize_managed_postgres_config(cfg, start_intent)` performs:

1. Validates non-empty `cfg.postgres.data_dir`.
2. Writes `pgtm.pg_hba.conf` from `postgres.pg_hba.source`.
3. Writes `pgtm.pg_ident.conf` from `postgres.pg_ident.source`.
4. Materializes TLS files and determines `ManagedPostgresTlsConfig`.
5. Normalizes standby-auth paths to the managed passfile location.
6. Materializes the optional standby passfile for replica or recovery intent.
7. Renders `ManagedPostgresConf` and writes `pgtm.postgresql.conf`.
8. Quarantines existing `postgresql.auto.conf` to `pgtm.unmanaged.postgresql.auto.conf`.
9. Updates `standby.signal` and `recovery.signal` for the selected start intent.
10. Returns `ManagedPostgresConfig` with managed paths.

## Standby auth materialization

`normalize_standby_auth_paths` rewrites `PasswordPassfile` auth to the managed standby passfile path under PGDATA.

`materialize_managed_standby_passfile` behavior:

| Intent And Auth | Action |
|---|---|
| `Primary` | removes stale managed passfile and returns `None` |
| `Replica` or `Recovery` with `NoPassword` | removes the managed passfile |
| `Replica` or `Recovery` with `PasswordPassfile` | resolves the replicator password, writes one libpq passfile entry with mode `0600`, and returns the managed path |

`render_libpq_passfile_entry` rejects newline characters in host, dbname, user, and password fields, escapes `:` and `\`, and renders one trailing newline.

`resolve_role_password` requires password auth for replicator role when managed standby passfile materialization is requested.

## TLS materialization

`materialize_tls_files` returns `ManagedPostgresTlsConfig::Disabled` when `cfg.postgres.tls.mode` is `Disabled`.

When `cfg.postgres.tls.mode` is `Optional` or `Required`:

- `cfg.postgres.tls.identity` must be present
- certificate material is copied to `pgtm.server.crt` with mode `0644`
- key material is copied to `pgtm.server.key` with mode `0600`
- if `cfg.postgres.tls.client_auth` is present, CA material is copied to `pgtm.ca.crt` with mode `0644`

The module copies operator-supplied TLS material and does not generate credentials.

## Signal-file behavior

Recovery signal files are mutually exclusive.

| Start Intent | `standby.signal` | `recovery.signal` |
|---|---|---|
| `Primary` | removed | removed |
| `Replica` | created | removed |
| `Recovery` | removed | created |

## Readback and runtime integration boundary

### `read_existing_replica_start_intent`

`read_existing_replica_start_intent(data_dir)`:

- checks `standby.signal` and `recovery.signal`
- returns `Ok(None)` when neither signal file exists
- returns `InvalidManagedState` if both signal files exist
- reads `pgtm.postgresql.conf`
- requires `primary_conninfo`
- parses `primary_conninfo` through `parse_managed_primary_conninfo`
- reads optional `primary_slot_name`
- reconstructs `Replica` or `Recovery` from the signal file

### Runtime integration

`runtime::run_start_job` materializes managed config via `materialize_managed_postgres_config` and starts `ProcessJobKind::StartPostgres` with `config_file = managed.postgresql_conf_path`.

Related tests:

- `build_command_start_postgres_uses_managed_config_file_override` in `src/process/worker.rs`
- `start_postgres_dispatch_builds_request_with_managed_settings` in `src/ha/process_dispatch.rs`
