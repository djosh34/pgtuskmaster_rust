# Runtime Configuration Loader and Schema

This reference documents the runtime configuration loader and schema machinery.

## Module Surface

| Module | Description |
|--------|-------------|
| `src/config/parser.rs` | TOML loading, version dispatch, normalization, and validation |
| `src/config/schema.rs` | Normalized config structs, secure v2 input structs, enums, and serde shape |
| `src/config/defaults.rs` | Safe defaults and process-config normalization helpers |

## Loader Pipeline

`load_runtime_config(path)` in `src/config/parser.rs` reads the versioned TOML file and returns a normalized `RuntimeConfig` or a `ConfigError`.

### ConfigError Variants

| Variant | Fields | Description |
|---------|--------|-------------|
| `Io` | `path`, `source` | Filesystem read failure |
| `Parse` | `path`, `source` | TOML syntax error |
| `Validation` | `field`, `message` | Schema or semantic invariant violation |

### Version Handling

- Missing `config_version` returns `Validation` for field `config_version` with message `missing required field; set config_version = "v2" to use the explicit secure schema`.
- `config_version = "v1"` calls `probe_legacy_v1_shape_for_diagnostics(contents)` and returns `Validation` for field `config_version` stating that v1 is no longer supported because it depends on implicit security defaults and must migrate to `config_version = "v2"`.
- `config_version = "v2"` deserializes `RuntimeConfigV2Input`, normalizes it to `RuntimeConfig`, validates the normalized config, and returns it.

`probe_legacy_v1_shape_for_diagnostics` attempts to parse the TOML to `toml::Value`, removes `config_version` from the top-level table when possible, and attempts deserialization into `PartialRuntimeConfig`. It does not replace the v1 validation result with a parse error.

## Normalized RuntimeConfig Structure

| Section | Fields |
|---------|--------|
| `cluster` | `name`, `member_id` |
| `postgres` | `data_dir`, `connect_timeout_s`, `listen_host`, `listen_port`, `socket_dir`, `log_file`, `local_conn_identity`, `rewind_conn_identity`, `tls`, `roles`, `pg_hba`, `pg_ident`, `extra_gucs` |
| `dcs` | `endpoints`, `scope`, `init` |
| `ha` | `loop_interval_ms`, `lease_ttl_ms` |
| `process` | `pg_rewind_timeout_ms`, `bootstrap_timeout_ms`, `fencing_timeout_ms`, `binaries` |
| `logging` | `level`, `capture_subprocess_output`, `postgres`, `sinks` |
| `api` | `listen_addr`, `security` |
| `debug` | `enabled` |

## Key Nested Types

| Type | Fields/Description |
|------|---------------------|
| `PostgresConnIdentityConfig` | `user`, `dbname`, `ssl_mode` |
| `PostgresRolesConfig` | `superuser`, `replicator`, `rewinder` (each `PostgresRoleConfig`) |
| `PostgresRoleConfig` | `username`, `auth` |
| `RoleAuthConfig` | `type = "tls"` with no fields, or `type = "password"` with `password: SecretSource` |
| `PgHbaConfig` | `source: InlineOrPath` |
| `PgIdentConfig` | `source: InlineOrPath` |
| `DcsInitConfig` | `payload_json`, `write_on_bootstrap` |
| `BinaryPaths` | `postgres`, `pg_ctl`, `pg_rewind`, `initdb`, `pg_basebackup`, `psql` |
| `PostgresLoggingConfig` | `enabled`, `pg_ctl_log_file`, `log_dir`, `poll_interval_ms`, `cleanup` |
| `LogCleanupConfig` | `enabled`, `max_files`, `max_age_seconds`, `protect_recent_seconds` |
| `LoggingSinksConfig` | `stderr`, `file` |
| `StderrSinkConfig` | `enabled` |
| `FileSinkConfig` | `enabled`, `path`, `mode` |
| `FileSinkMode` | `append`, `truncate` |
| `ApiSecurityConfig` | `tls`, `auth` |
| `TlsServerConfig` | `mode`, `identity`, `client_auth` |
| `ApiTlsMode` | `disabled`, `optional`, `required` |
| `TlsServerIdentityConfig` | `cert_chain`, `private_key` |
| `TlsClientAuthConfig` | `client_ca`, `require_client_cert` |
| `ApiAuthConfig` | `type = "disabled"` with no fields, or `type = "role_tokens"` with `ApiRoleTokensConfig` |
| `ApiRoleTokensConfig` | `read_token`, `admin_token` |
| `InlineOrPath` | Bare path string, object with `path`, or object with `content` |
| `SecretSource` | Wraps `InlineOrPath`; `Debug` redacts inline content and renders path-backed values as `path(...)` |

## Required Input Blocks

| Block | Required Fields |
|-------|-----------------|
| `postgres.local_conn_identity` | `user`, `dbname`, `ssl_mode` |
| `postgres.rewind_conn_identity` | `user`, `dbname`, `ssl_mode` |
| `postgres.roles.superuser` | `username`, `auth` |
| `postgres.roles.replicator` | `username`, `auth` |
| `postgres.roles.rewinder` | `username`, `auth` |
| `postgres.pg_hba` | `source` |
| `postgres.pg_ident` | `source` |
| `api.security` | `auth` |
| `process.binaries` | `postgres`, `pg_ctl`, `pg_rewind`, `initdb`, `pg_basebackup`, `psql` |

All `auth` blocks must contain `type`. When `type = "password"`, `password` is required and must be non-empty.

## Default Values

| Field | Default |
|-------|---------|
| `api.listen_addr` | `127.0.0.1:8080` |
| `debug.enabled` | `false` |
| `logging.level` | `info` |
| `logging.capture_subprocess_output` | `true` |
| `logging.postgres.enabled` | `true` |
| `logging.postgres.pg_ctl_log_file` | `None` |
| `logging.postgres.log_dir` | `None` |
| `logging.postgres.poll_interval_ms` | `200` |
| `logging.postgres.cleanup.enabled` | `true` |
| `logging.postgres.cleanup.max_files` | `50` |
| `logging.postgres.cleanup.max_age_seconds` | `604800` |
| `logging.postgres.cleanup.protect_recent_seconds` | `300` |
| `logging.sinks.stderr.enabled` | `true` |
| `logging.sinks.file.enabled` | `false` |
| `logging.sinks.file.path` | `None` |
| `logging.sinks.file.mode` | `append` |
| `postgres.connect_timeout_s` | `5` |
| `postgres.extra_gucs` | Empty map |
| `process.pg_rewind_timeout_ms` | `120000` |
| `process.bootstrap_timeout_ms` | `300000` |
| `process.fencing_timeout_ms` | `30000` |

## Normalization Rules

TLS server config: When present in the secure schema, requires `mode`. Normalizes `identity` only when provided.

TLS server identity: When `identity` is present, requires non-empty `cert_chain` and `private_key`.

Role auth config: Maps `type = "tls"` to `RoleAuthConfig::Tls`. Maps `type = "password"` to `RoleAuthConfig::Password { password }` and requires `password`.

Connection identity: `normalize_postgres_conn_identity_v2` requires non-empty `user` and `dbname` and a present `ssl_mode`.

Path normalization: `normalize_path_lexical` removes `.` components and pops one path component for each `..` component before ownership comparisons.

Process config normalization: `normalize_process_config` requires `process.binaries` and fills defaults for `pg_rewind_timeout_ms`, `bootstrap_timeout_ms`, and `fencing_timeout_ms` when those fields are omitted.

## Validation Rules

### Path Constraints

These paths must be absolute and use `normalize_path_lexical`:

- All `process.binaries` fields
- `postgres.log_file`
- `logging.postgres.pg_ctl_log_file` when configured
- `logging.postgres.log_dir` when configured
- `logging.sinks.file.path` when the file sink is enabled and configured

### Timeout Constraints

`validate_timeout` requires values between `1` and `86400000` milliseconds inclusive:

- `process.pg_rewind_timeout_ms`
- `process.bootstrap_timeout_ms`
- `process.fencing_timeout_ms`
- `logging.postgres.poll_interval_ms`

### Integer and String Constraints

| Field | Constraint |
|-------|------------|
| `postgres.listen_port` | Greater than zero |
| `dcs.endpoints` | At least one entry and no empty trimmed strings |
| `dcs.scope` | Non-empty after trimming |
| `ha.loop_interval_ms` | Greater than zero |
| `ha.lease_ttl_ms` | Greater than zero and greater than `ha.loop_interval_ms` |

### API Auth Constraints

- `api.security.auth.role_tokens.read_token` must be non-empty when configured
- `api.security.auth.role_tokens.admin_token` must be non-empty when configured
- `ApiAuthConfig::RoleTokens` requires at least one of `read_token` or `admin_token`

### Role and Identity Constraints

- `postgres.local_conn_identity.user` must equal `postgres.roles.superuser.username`
- `postgres.rewind_conn_identity.user` must equal `postgres.roles.rewinder.username`
- `RoleAuthConfig::Tls` is rejected for postgres roles because PostgreSQL role TLS client auth is not implemented
- When `postgres.tls.mode` is `disabled`, `ssl_mode` values `Require`, `VerifyCa`, or `VerifyFull` are rejected

### TLS Configuration

| Condition | Requirement |
|-----------|-------------|
| `tls.mode = disabled` | Validation succeeds immediately |
| `tls.mode = optional` or `required` | Requires `identity` with non-empty `cert_chain` and `private_key` |
| `tls.mode = disabled` | `client_auth` is rejected |
| `tls.mode = optional` or `required` | `client_auth` requires non-empty `client_ca` |

### Secret and Source Constraints

- `postgres.pg_hba.source` and `postgres.pg_ident.source` must be non-empty `InlineOrPath` values
- Password-backed postgres role secrets must be non-empty in either path or inline form

### Logging Constraints

When `logging.postgres.cleanup.enabled` is `true`:

- `max_files` must be greater than zero
- `max_age_seconds` must be greater than zero
- `protect_recent_seconds` must be greater than zero

`logging.sinks.file.enabled` requires `logging.sinks.file.path` to be configured.

`validate_logging_path_ownership_invariants` rejects `logging.sinks.file.path` when it equals `postgres.log_file`, equals the effective `logging.postgres.pg_ctl_log_file`, or is inside `logging.postgres.log_dir`.

### DCS Initialization

When `dcs.init` is present, `validate_dcs_init_config` requires `payload_json` to be non-empty, valid JSON, and decodable as a `RuntimeConfig` JSON document.

## Sample Bundled Config Artifacts

### `docker/configs/common/pg_hba.conf`

```text
local   all             all                                     trust
host    all             all             127.0.0.1/32            trust
host    all             all             ::1/128                 trust
host    all             all             0.0.0.0/0               trust
host    replication     all             127.0.0.1/32            trust
host    replication     all             0.0.0.0/0               trust
```

### `docker/configs/common/pg_ident.conf`

```text
# empty
```

## Verified Behaviors

Tests in `src/config/parser.rs` verify:

- Missing `config_version` returns an actionable validation message
- `config_version = "v1"` is rejected with a migration hint
- Unknown fields in `config_version = "v2"` are rejected
- V2 defaults apply when fields are omitted
- Missing required secure fields return actionable validation messages
- Missing `process.binaries` entries are rejected
- Missing password secrets for password auth are rejected
- Missing postgres role blocks and role fields are rejected
- Role identity mismatches are rejected
- Blank password secrets are rejected
- TLS required without identity is rejected
- Client auth with TLS disabled is rejected
- Postgres role TLS auth is rejected
- SSL modes requiring postgres TLS while postgres TLS is disabled are rejected

Tests in `tests/cli_binary.rs` verify the node binary prints the explicit `config_version = "v2"` migration hint when `config_version` is missing.
