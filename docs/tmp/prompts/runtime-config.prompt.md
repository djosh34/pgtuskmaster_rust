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
- docs/src/reference/runtime-config.md

[Page goal]
- Reference the runtime configuration loader, normalized schema, defaults, and validation boundaries exposed by the config modules.

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
- Overview\n- Module surface\n- Load pipeline and version handling\n- Normalized runtime config structure\n- Defaulted fields\n- Validation rules and invariants\n- Related bundled artifacts if directly sourced from the repo

[Output requirements]
- Preserve or improve the title so it matches the machinery.
- Prefer compact tables where they clarify enums, fields, constants, routes, or error variants.
- Include only facts supported by the supplied source excerpts.
- If the current page contains unsupported claims, remove or correct them rather than hedging.

[Existing page to revise]

# Runtime Configuration Loader

## Overview

The runtime configuration loader reads versioned TOML files, normalizes them to a canonical `RuntimeConfig` struct, and validates semantic invariants.

## Module Surface

| Module | Description |
|--------|-------------|
| `src/config/parser.rs` | TOML loading, version dispatch, normalization, and validation |
| `src/config/schema.rs` | Normalized config structs, secure v2 input structs, enums, and serde shape |
| `src/config/defaults.rs` | Safe defaults and process-config normalization helpers |

## Loader Pipeline and Version Handling

`load_runtime_config(path)` in `src/config/parser.rs` reads a versioned TOML file and returns a normalized `RuntimeConfig` or a `ConfigError`.

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

### Key Nested Types

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

## Validation Rules and Invariants

### Path Constraints

- All `process.binaries` fields must be absolute paths.
- `postgres.log_file` must be an absolute path.
- `logging.postgres.pg_ctl_log_file` must be an absolute path when configured.
- `logging.postgres.log_dir` must be an absolute path when configured.
- `logging.sinks.file.path` must be an absolute path when the file sink is enabled and configured.

Paths are normalized with `normalize_path_lexical`, which removes `.` components and pops one path component for each `..` component.

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
| `dcs.endpoints` | At least one entry; no empty trimmed strings |
| `dcs.scope` | Non-empty after trimming |
| `ha.loop_interval_ms` | Greater than zero |
| `ha.lease_ttl_ms` | Greater than zero and greater than `ha.loop_interval_ms` |

### API Auth Constraints

- `api.security.auth.role_tokens.read_token` must be non-empty when configured.
- `api.security.auth.role_tokens.admin_token` must be non-empty when configured.
- `ApiAuthConfig::RoleTokens` requires at least one of `read_token` or `admin_token`.

### Role and Identity Constraints

- `postgres.local_conn_identity.user` must equal `postgres.roles.superuser.username`.
- `postgres.rewind_conn_identity.user` must equal `postgres.roles.rewinder.username`.
- `RoleAuthConfig::Tls` is rejected for postgres roles because PostgreSQL role TLS client auth is not implemented.
- When `postgres.tls.mode` is `disabled`, `ssl_mode` values `Require`, `VerifyCa`, or `VerifyFull` are rejected.

### TLS Configuration

| Condition | Requirement |
|-----------|-------------|
| `tls.mode = disabled` | Validation succeeds immediately |
| `tls.mode = optional` or `required` | Requires `identity` with non-empty `cert_chain` and `private_key` |
| `tls.mode = disabled` | `client_auth` is rejected |
| `tls.mode = optional` or `required` | `client_auth` requires non-empty `client_ca` |

### Secret and Source Constraints

- `postgres.pg_hba.source` and `postgres.pg_ident.source` must be non-empty `InlineOrPath` values.
- Password-backed postgres role secrets must be non-empty in either path or inline form.

### Logging Constraints

When `logging.postgres.cleanup.enabled` is `true`:

- `max_files` must be greater than zero
- `max_age_seconds` must be greater than zero
- `protect_recent_seconds` must be greater than zero

`logging.sinks.file.enabled` requires `logging.sinks.file.path` to be configured.

`validate_logging_path_ownership_invariants` rejects `logging.sinks.file.path` when it equals `postgres.log_file`, equals the effective `logging.postgres.pg_ctl_log_file`, or is inside `logging.postgres.log_dir`.

### DCS Initialization

When `dcs.init` is present, `validate_dcs_init_config` requires `payload_json` to be non-empty, valid JSON, and decodable as a `RuntimeConfig` JSON document.

## Related Bundled Artifacts

### docker/configs/common/pg_hba.conf

```text
local   all             all                                     trust
host    all             all             127.0.0.1/32            trust
host    all             all             ::1/128                 trust
host    all             all             0.0.0.0/0               trust
host    replication     all             127.0.0.1/32            trust
host    replication     all             0.0.0.0/0               trust
```

### docker/configs/common/pg_ident.conf

```text
# empty
```

[Repo facts and source excerpts]

--- BEGIN FILE: src/config/mod.rs ---
pub(crate) mod defaults;
pub(crate) mod parser;
pub(crate) mod schema;

pub use parser::{load_runtime_config, validate_runtime_config, ConfigError};
pub use schema::{
    ApiAuthConfig, ApiConfig, ApiRoleTokensConfig, ApiSecurityConfig, ApiTlsMode, BinaryPaths,
    ClusterConfig, ConfigVersion, DcsConfig, DcsInitConfig, DebugConfig, FileSinkConfig,
    FileSinkMode, HaConfig, InlineOrPath, LogCleanupConfig, LogLevel, LoggingConfig,
    LoggingSinksConfig, PgHbaConfig, PgIdentConfig, PostgresConfig, PostgresConnIdentityConfig,
    PostgresLoggingConfig, PostgresRoleConfig, PostgresRolesConfig, ProcessConfig, RoleAuthConfig,
    RuntimeConfig, RuntimeConfigV2Input, SecretSource, StderrSinkConfig, TlsClientAuthConfig,
    TlsServerConfig, TlsServerIdentityConfig,
};

--- END FILE: src/config/mod.rs ---

--- BEGIN FILE: src/config/schema.rs ---
use std::{collections::BTreeMap, fmt, path::PathBuf};

use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfigVersion {
    V1,
    V2,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum InlineOrPath {
    Path(PathBuf),
    PathConfig { path: PathBuf },
    Inline { content: String },
}

#[derive(Clone, PartialEq, Eq, Deserialize)]
pub struct SecretSource(pub InlineOrPath);

impl fmt::Debug for SecretSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            InlineOrPath::Path(path) => f
                .debug_tuple("SecretSource")
                .field(&format_args!("path({})", path.display()))
                .finish(),
            InlineOrPath::PathConfig { path } => f
                .debug_tuple("SecretSource")
                .field(&format_args!("path({})", path.display()))
                .finish(),
            InlineOrPath::Inline { .. } => f
                .debug_tuple("SecretSource")
                .field(&"<inline redacted>")
                .finish(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApiTlsMode {
    Disabled,
    Optional,
    Required,
}

pub type TlsMode = ApiTlsMode;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TlsServerIdentityConfig {
    pub cert_chain: InlineOrPath,
    pub private_key: InlineOrPath,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TlsClientAuthConfig {
    pub client_ca: InlineOrPath,
    pub require_client_cert: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TlsServerConfig {
    pub mode: TlsMode,
    pub identity: Option<TlsServerIdentityConfig>,
    pub client_auth: Option<TlsClientAuthConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct RuntimeConfig {
    pub cluster: ClusterConfig,
    pub postgres: PostgresConfig,
    pub dcs: DcsConfig,
    pub ha: HaConfig,
    pub process: ProcessConfig,
    pub logging: LoggingConfig,
    pub api: ApiConfig,
    pub debug: DebugConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClusterConfig {
    pub name: String,
    pub member_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresConfig {
    pub data_dir: PathBuf,
    pub connect_timeout_s: u32,
    pub listen_host: String,
    pub listen_port: u16,
    pub socket_dir: PathBuf,
    pub log_file: PathBuf,
    pub local_conn_identity: PostgresConnIdentityConfig,
    pub rewind_conn_identity: PostgresConnIdentityConfig,
    pub tls: TlsServerConfig,
    pub roles: PostgresRolesConfig,
    pub pg_hba: PgHbaConfig,
    pub pg_ident: PgIdentConfig,
    pub extra_gucs: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresConnIdentityConfig {
    pub user: String,
    pub dbname: String,
    pub ssl_mode: crate::pginfo::conninfo::PgSslMode,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum RoleAuthConfig {
    Tls,
    Password { password: SecretSource },
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresRoleConfig {
    pub username: String,
    pub auth: RoleAuthConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresRolesConfig {
    pub superuser: PostgresRoleConfig,
    pub replicator: PostgresRoleConfig,
    pub rewinder: PostgresRoleConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PgHbaConfig {
    pub source: InlineOrPath,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PgIdentConfig {
    pub source: InlineOrPath,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DcsConfig {
    pub endpoints: Vec<String>,
    pub scope: String,
    pub init: Option<DcsInitConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DcsInitConfig {
    pub payload_json: String,
    pub write_on_bootstrap: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HaConfig {
    pub loop_interval_ms: u64,
    pub lease_ttl_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessConfig {
    pub pg_rewind_timeout_ms: u64,
    pub bootstrap_timeout_ms: u64,
    pub fencing_timeout_ms: u64,
    pub binaries: BinaryPaths,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoggingConfig {
    pub level: LogLevel,
    pub capture_subprocess_output: bool,
    pub postgres: PostgresLoggingConfig,
    pub sinks: LoggingSinksConfig,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresLoggingConfig {
    pub enabled: bool,
    pub pg_ctl_log_file: Option<PathBuf>,
    pub log_dir: Option<PathBuf>,
    pub poll_interval_ms: u64,
    pub cleanup: LogCleanupConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoggingSinksConfig {
    pub stderr: StderrSinkConfig,
    pub file: FileSinkConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StderrSinkConfig {
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileSinkConfig {
    pub enabled: bool,
    pub path: Option<PathBuf>,
    pub mode: FileSinkMode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileSinkMode {
    Append,
    Truncate,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LogCleanupConfig {
    pub enabled: bool,
    pub max_files: u64,
    pub max_age_seconds: u64,
    #[serde(default = "default_log_cleanup_protect_recent_seconds")]
    pub protect_recent_seconds: u64,
}

fn default_log_cleanup_protect_recent_seconds() -> u64 {
    300
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BinaryPaths {
    pub postgres: PathBuf,
    pub pg_ctl: PathBuf,
    pub pg_rewind: PathBuf,
    pub initdb: PathBuf,
    pub pg_basebackup: PathBuf,
    pub psql: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiConfig {
    pub listen_addr: String,
    pub security: ApiSecurityConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiSecurityConfig {
    pub tls: TlsServerConfig,
    pub auth: ApiAuthConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ApiAuthConfig {
    Disabled,
    RoleTokens(ApiRoleTokensConfig),
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiRoleTokensConfig {
    pub read_token: Option<String>,
    pub admin_token: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DebugConfig {
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialRuntimeConfig {
    pub cluster: ClusterConfig,
    pub postgres: PartialPostgresConfig,
    pub dcs: DcsConfig,
    pub ha: HaConfig,
    pub process: PartialProcessConfig,
    pub logging: Option<PartialLoggingConfig>,
    pub api: Option<PartialApiConfig>,
    pub debug: Option<PartialDebugConfig>,
    pub security: Option<PartialSecurityConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialPostgresConfig {
    pub data_dir: PathBuf,
    pub connect_timeout_s: Option<u32>,
    pub listen_host: Option<String>,
    pub listen_port: Option<u16>,
    pub socket_dir: Option<PathBuf>,
    pub log_file: Option<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialProcessConfig {
    pub pg_rewind_timeout_ms: Option<u64>,
    pub bootstrap_timeout_ms: Option<u64>,
    pub fencing_timeout_ms: Option<u64>,
    pub binaries: BinaryPaths,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialLoggingConfig {
    pub level: Option<LogLevel>,
    pub capture_subprocess_output: Option<bool>,
    pub postgres: Option<PartialPostgresLoggingConfig>,
    pub sinks: Option<PartialLoggingSinksConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialPostgresLoggingConfig {
    pub enabled: Option<bool>,
    pub pg_ctl_log_file: Option<PathBuf>,
    pub log_dir: Option<PathBuf>,
    pub poll_interval_ms: Option<u64>,
    pub cleanup: Option<PartialLogCleanupConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialLogCleanupConfig {
    pub enabled: Option<bool>,
    pub max_files: Option<u64>,
    pub max_age_seconds: Option<u64>,
    pub protect_recent_seconds: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialLoggingSinksConfig {
    pub stderr: Option<PartialStderrSinkConfig>,
    pub file: Option<PartialFileSinkConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialStderrSinkConfig {
    pub enabled: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialFileSinkConfig {
    pub enabled: Option<bool>,
    pub path: Option<PathBuf>,
    pub mode: Option<FileSinkMode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialApiConfig {
    pub listen_addr: Option<String>,
    pub read_auth_token: Option<String>,
    pub admin_auth_token: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialDebugConfig {
    pub enabled: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialSecurityConfig {
    pub tls_enabled: Option<bool>,
    pub auth_token: Option<String>,
}

// -------------------------------
// v2 input schema (explicit secure)
// -------------------------------

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeConfigV2Input {
    pub config_version: ConfigVersion,
    pub cluster: ClusterConfig,
    pub postgres: PostgresConfigV2Input,
    pub dcs: DcsConfig,
    pub ha: HaConfig,
    pub process: ProcessConfigV2Input,
    pub logging: Option<LoggingConfig>,
    pub api: ApiConfigV2Input,
    pub debug: Option<DebugConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessConfigV2Input {
    pub pg_rewind_timeout_ms: Option<u64>,
    pub bootstrap_timeout_ms: Option<u64>,
    pub fencing_timeout_ms: Option<u64>,
    pub binaries: Option<BinaryPathsV2Input>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BinaryPathsV2Input {
    pub postgres: Option<PathBuf>,
    pub pg_ctl: Option<PathBuf>,
    pub pg_rewind: Option<PathBuf>,
    pub initdb: Option<PathBuf>,
    pub pg_basebackup: Option<PathBuf>,
    pub psql: Option<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiConfigV2Input {
    pub listen_addr: Option<String>,
    pub security: Option<ApiSecurityConfigV2Input>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiSecurityConfigV2Input {
    pub tls: Option<TlsServerConfigV2Input>,
    pub auth: Option<ApiAuthConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresConfigV2Input {
    pub data_dir: PathBuf,
    pub connect_timeout_s: Option<u32>,
    pub listen_host: String,
    pub listen_port: u16,
    pub socket_dir: PathBuf,
    pub log_file: PathBuf,
    pub local_conn_identity: Option<PostgresConnIdentityConfigV2Input>,
    pub rewind_conn_identity: Option<PostgresConnIdentityConfigV2Input>,
    pub tls: Option<TlsServerConfigV2Input>,
    pub roles: Option<PostgresRolesConfigV2Input>,
    pub pg_hba: Option<PgHbaConfigV2Input>,
    pub pg_ident: Option<PgIdentConfigV2Input>,
    pub extra_gucs: Option<BTreeMap<String, String>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresConnIdentityConfigV2Input {
    pub user: Option<String>,
    pub dbname: Option<String>,
    pub ssl_mode: Option<crate::pginfo::conninfo::PgSslMode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresRoleConfigV2Input {
    pub username: Option<String>,
    pub auth: Option<RoleAuthConfigV2Input>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresRolesConfigV2Input {
    pub superuser: Option<PostgresRoleConfigV2Input>,
    pub replicator: Option<PostgresRoleConfigV2Input>,
    pub rewinder: Option<PostgresRoleConfigV2Input>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PgHbaConfigV2Input {
    pub source: Option<InlineOrPath>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PgIdentConfigV2Input {
    pub source: Option<InlineOrPath>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TlsServerIdentityConfigV2Input {
    pub cert_chain: Option<InlineOrPath>,
    pub private_key: Option<InlineOrPath>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum RoleAuthConfigV2Input {
    Tls,
    Password { password: Option<SecretSource> },
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TlsServerConfigV2Input {
    pub mode: Option<TlsMode>,
    pub identity: Option<TlsServerIdentityConfigV2Input>,
    pub client_auth: Option<TlsClientAuthConfig>,
}

--- END FILE: src/config/schema.rs ---

--- BEGIN FILE: src/config/defaults.rs ---
use super::schema::{
    BinaryPaths, BinaryPathsV2Input, DebugConfig, FileSinkConfig, FileSinkMode, LogCleanupConfig,
    LogLevel, LoggingConfig, LoggingSinksConfig, PostgresLoggingConfig, ProcessConfig,
    StderrSinkConfig,
};
use super::ConfigError;

// This module is intentionally restricted to *safe* defaults only.
// It must not synthesize security-sensitive material (users/roles/auth, TLS posture, pg_hba/pg_ident).

const DEFAULT_PG_CONNECT_TIMEOUT_S: u32 = 5;
const DEFAULT_PG_REWIND_TIMEOUT_MS: u64 = 120_000;
const DEFAULT_BOOTSTRAP_TIMEOUT_MS: u64 = 300_000;
const DEFAULT_FENCING_TIMEOUT_MS: u64 = 30_000;

const DEFAULT_API_LISTEN_ADDR: &str = "127.0.0.1:8080";
const DEFAULT_DEBUG_ENABLED: bool = false;

const DEFAULT_LOGGING_LEVEL: LogLevel = LogLevel::Info;
const DEFAULT_LOGGING_CAPTURE_SUBPROCESS_OUTPUT: bool = true;
const DEFAULT_LOGGING_POSTGRES_ENABLED: bool = true;
const DEFAULT_LOGGING_POSTGRES_POLL_INTERVAL_MS: u64 = 200;
const DEFAULT_LOGGING_CLEANUP_ENABLED: bool = true;
const DEFAULT_LOGGING_CLEANUP_MAX_FILES: u64 = 50;
const DEFAULT_LOGGING_CLEANUP_MAX_AGE_SECONDS: u64 = 7 * 24 * 60 * 60;
const DEFAULT_LOGGING_CLEANUP_PROTECT_RECENT_SECONDS: u64 = 300;
const DEFAULT_LOGGING_SINK_STDERR_ENABLED: bool = true;
const DEFAULT_LOGGING_SINK_FILE_ENABLED: bool = false;
const DEFAULT_LOGGING_SINK_FILE_MODE: FileSinkMode = FileSinkMode::Append;

pub(crate) fn default_postgres_connect_timeout_s() -> u32 {
    DEFAULT_PG_CONNECT_TIMEOUT_S
}

pub(crate) fn default_api_listen_addr() -> String {
    DEFAULT_API_LISTEN_ADDR.to_string()
}

pub(crate) fn default_debug_config() -> DebugConfig {
    DebugConfig {
        enabled: DEFAULT_DEBUG_ENABLED,
    }
}

pub(crate) fn default_logging_config() -> LoggingConfig {
    LoggingConfig {
        level: DEFAULT_LOGGING_LEVEL,
        capture_subprocess_output: DEFAULT_LOGGING_CAPTURE_SUBPROCESS_OUTPUT,
        postgres: PostgresLoggingConfig {
            enabled: DEFAULT_LOGGING_POSTGRES_ENABLED,
            pg_ctl_log_file: None,
            log_dir: None,
            poll_interval_ms: DEFAULT_LOGGING_POSTGRES_POLL_INTERVAL_MS,
            cleanup: LogCleanupConfig {
                enabled: DEFAULT_LOGGING_CLEANUP_ENABLED,
                max_files: DEFAULT_LOGGING_CLEANUP_MAX_FILES,
                max_age_seconds: DEFAULT_LOGGING_CLEANUP_MAX_AGE_SECONDS,
                protect_recent_seconds: DEFAULT_LOGGING_CLEANUP_PROTECT_RECENT_SECONDS,
            },
        },
        sinks: LoggingSinksConfig {
            stderr: StderrSinkConfig {
                enabled: DEFAULT_LOGGING_SINK_STDERR_ENABLED,
            },
            file: FileSinkConfig {
                enabled: DEFAULT_LOGGING_SINK_FILE_ENABLED,
                path: None,
                mode: DEFAULT_LOGGING_SINK_FILE_MODE,
            },
        },
    }
}

pub(crate) fn normalize_process_config(
    input: super::schema::ProcessConfigV2Input,
) -> Result<ProcessConfig, ConfigError> {
    let binaries = input.binaries.ok_or_else(|| ConfigError::Validation {
        field: "process.binaries",
        message: "missing required secure field for config_version=v2".to_string(),
    })?;
    let binaries = normalize_binary_paths_v2(binaries)?;

    Ok(ProcessConfig {
        pg_rewind_timeout_ms: input
            .pg_rewind_timeout_ms
            .unwrap_or(DEFAULT_PG_REWIND_TIMEOUT_MS),
        bootstrap_timeout_ms: input
            .bootstrap_timeout_ms
            .unwrap_or(DEFAULT_BOOTSTRAP_TIMEOUT_MS),
        fencing_timeout_ms: input
            .fencing_timeout_ms
            .unwrap_or(DEFAULT_FENCING_TIMEOUT_MS),
        binaries,
    })
}

fn normalize_binary_paths_v2(input: BinaryPathsV2Input) -> Result<BinaryPaths, ConfigError> {
    Ok(BinaryPaths {
        postgres: require_binary_path("process.binaries.postgres", input.postgres)?,
        pg_ctl: require_binary_path("process.binaries.pg_ctl", input.pg_ctl)?,
        pg_rewind: require_binary_path("process.binaries.pg_rewind", input.pg_rewind)?,
        initdb: require_binary_path("process.binaries.initdb", input.initdb)?,
        pg_basebackup: require_binary_path("process.binaries.pg_basebackup", input.pg_basebackup)?,
        psql: require_binary_path("process.binaries.psql", input.psql)?,
    })
}

fn require_binary_path(
    field: &'static str,
    value: Option<std::path::PathBuf>,
) -> Result<std::path::PathBuf, ConfigError> {
    value.ok_or_else(|| ConfigError::Validation {
        field,
        message: "missing required secure field for config_version=v2".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_logging_config_is_deterministic() {
        let a = default_logging_config();
        let b = default_logging_config();
        assert_eq!(a, b);
    }
}

--- END FILE: src/config/defaults.rs ---

--- BEGIN FILE: src/config/parser.rs ---
use std::path::{Path, PathBuf};

use thiserror::Error;

use super::defaults::{
    default_api_listen_addr, default_debug_config, default_logging_config,
    default_postgres_connect_timeout_s, normalize_process_config,
};
use super::schema::{
    ApiConfig, ApiSecurityConfig, ConfigVersion, InlineOrPath, PgHbaConfig, PgIdentConfig,
    PostgresConfig, PostgresConnIdentityConfig, PostgresRoleConfig, PostgresRolesConfig,
    RoleAuthConfig, RoleAuthConfigV2Input, RuntimeConfig, RuntimeConfigV2Input, SecretSource,
    TlsServerConfig, TlsServerIdentityConfig,
};
use crate::postgres_managed_conf::{validate_extra_guc_entry, ManagedPostgresConfError};

const MIN_TIMEOUT_MS: u64 = 1;
const MAX_TIMEOUT_MS: u64 = 86_400_000;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read config file {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse config file {path}: {source}")]
    Parse {
        path: String,
        #[source]
        source: toml::de::Error,
    },
    #[error("invalid config field `{field}`: {message}")]
    Validation {
        field: &'static str,
        message: String,
    },
}

pub fn load_runtime_config(path: &Path) -> Result<RuntimeConfig, ConfigError> {
    let contents = std::fs::read_to_string(path).map_err(|source| ConfigError::Io {
        path: path.display().to_string(),
        source,
    })?;

    #[derive(serde::Deserialize)]
    struct ConfigEnvelope {
        config_version: Option<ConfigVersion>,
    }

    let envelope: ConfigEnvelope =
        toml::from_str(&contents).map_err(|source| ConfigError::Parse {
            path: path.display().to_string(),
            source,
        })?;

    let config_version = envelope.config_version.ok_or_else(|| ConfigError::Validation {
        field: "config_version",
        message: "missing required field; set config_version = \"v2\" to use the explicit secure schema".to_string(),
    })?;

    match config_version {
        ConfigVersion::V1 => {
            probe_legacy_v1_shape_for_diagnostics(&contents);
            Err(ConfigError::Validation {
                field: "config_version",
                message: "config_version = \"v1\" is no longer supported because it depends on implicit security defaults; migrate to config_version = \"v2\""
                    .to_string(),
            })
        }
        ConfigVersion::V2 => {
            let raw: RuntimeConfigV2Input =
                toml::from_str(&contents).map_err(|source| ConfigError::Parse {
                    path: path.display().to_string(),
                    source,
                })?;
            let cfg = normalize_runtime_config_v2(raw)?;
            validate_runtime_config(&cfg)?;
            Ok(cfg)
        }
    }
}

fn probe_legacy_v1_shape_for_diagnostics(contents: &str) {
    // We keep the legacy v1 deserialization surface "alive" to:
    // - avoid unused-schema drift during the transition
    // - allow future improvements that surface rich TOML diagnostics for v1 migrations
    //
    // This must never override the v1 migration guidance with a parse error.
    let parsed: Result<toml::Value, toml::de::Error> = toml::from_str(contents);
    let Ok(mut value) = parsed else {
        return;
    };

    let Some(table) = value.as_table_mut() else {
        return;
    };

    let _ = table.remove("config_version");

    let _: Result<super::schema::PartialRuntimeConfig, toml::de::Error> = value.try_into();
}

fn normalize_runtime_config_v2(input: RuntimeConfigV2Input) -> Result<RuntimeConfig, ConfigError> {
    if !matches!(input.config_version, ConfigVersion::V2) {
        return Err(ConfigError::Validation {
            field: "config_version",
            message: "expected config_version = \"v2\"".to_string(),
        });
    }

    let postgres = normalize_postgres_config_v2(input.postgres)?;
    let process = normalize_process_config(input.process)?;
    let logging = input.logging.unwrap_or_else(default_logging_config);
    let api = normalize_api_config_v2(input.api)?;
    let debug = input.debug.unwrap_or_else(default_debug_config);

    Ok(RuntimeConfig {
        cluster: input.cluster,
        postgres,
        dcs: input.dcs,
        ha: input.ha,
        process,
        logging,
        api,
        debug,
    })
}

fn normalize_postgres_config_v2(
    input: super::schema::PostgresConfigV2Input,
) -> Result<PostgresConfig, ConfigError> {
    let connect_timeout_s = input
        .connect_timeout_s
        .unwrap_or_else(default_postgres_connect_timeout_s);

    let local_conn_identity = normalize_postgres_conn_identity_v2(
        "postgres.local_conn_identity",
        input.local_conn_identity,
    )?;
    let rewind_conn_identity = normalize_postgres_conn_identity_v2(
        "postgres.rewind_conn_identity",
        input.rewind_conn_identity,
    )?;

    let tls = normalize_tls_server_config_v2("postgres.tls", input.tls)?;
    let roles = normalize_postgres_roles_v2(input.roles)?;
    let pg_hba = normalize_pg_hba_v2(input.pg_hba)?;
    let pg_ident = normalize_pg_ident_v2(input.pg_ident)?;

    Ok(PostgresConfig {
        data_dir: input.data_dir,
        connect_timeout_s,
        listen_host: input.listen_host,
        listen_port: input.listen_port,
        socket_dir: input.socket_dir,
        log_file: input.log_file,
        local_conn_identity,
        rewind_conn_identity,
        tls,
        roles,
        pg_hba,
        pg_ident,
        extra_gucs: normalize_postgres_extra_gucs_v2(input.extra_gucs)?,
    })
}

fn normalize_postgres_extra_gucs_v2(
    input: Option<std::collections::BTreeMap<String, String>>,
) -> Result<std::collections::BTreeMap<String, String>, ConfigError> {
    let extra_gucs = input.unwrap_or_default();
    for (key, value) in &extra_gucs {
        validate_extra_guc_for_config(key.as_str(), value.as_str())?;
    }
    Ok(extra_gucs)
}

fn normalize_postgres_conn_identity_v2(
    field_prefix: &'static str,
    input: Option<super::schema::PostgresConnIdentityConfigV2Input>,
) -> Result<PostgresConnIdentityConfig, ConfigError> {
    let identity = input.ok_or_else(|| ConfigError::Validation {
        field: field_prefix,
        message: "missing required secure config block for config_version=v2".to_string(),
    })?;

    let user_field = match field_prefix {
        "postgres.local_conn_identity" => "postgres.local_conn_identity.user",
        "postgres.rewind_conn_identity" => "postgres.rewind_conn_identity.user",
        _ => field_prefix,
    };
    let dbname_field = match field_prefix {
        "postgres.local_conn_identity" => "postgres.local_conn_identity.dbname",
        "postgres.rewind_conn_identity" => "postgres.rewind_conn_identity.dbname",
        _ => field_prefix,
    };
    let ssl_mode_field = match field_prefix {
        "postgres.local_conn_identity" => "postgres.local_conn_identity.ssl_mode",
        "postgres.rewind_conn_identity" => "postgres.rewind_conn_identity.ssl_mode",
        _ => field_prefix,
    };

    let user = identity.user.ok_or_else(|| ConfigError::Validation {
        field: user_field,
        message: "missing required secure field for config_version=v2".to_string(),
    })?;
    validate_non_empty(user_field, user.as_str())?;

    let dbname = identity.dbname.ok_or_else(|| ConfigError::Validation {
        field: dbname_field,
        message: "missing required secure field for config_version=v2".to_string(),
    })?;
    validate_non_empty(dbname_field, dbname.as_str())?;

    let ssl_mode = identity.ssl_mode.ok_or_else(|| ConfigError::Validation {
        field: ssl_mode_field,
        message: "missing required secure field for config_version=v2".to_string(),
    })?;

    Ok(PostgresConnIdentityConfig {
        user,
        dbname,
        ssl_mode,
    })
}

fn normalize_postgres_roles_v2(
    input: Option<super::schema::PostgresRolesConfigV2Input>,
) -> Result<PostgresRolesConfig, ConfigError> {
    let roles = input.ok_or_else(|| ConfigError::Validation {
        field: "postgres.roles",
        message: "missing required secure config block for config_version=v2".to_string(),
    })?;

    let superuser = normalize_postgres_role_v2("postgres.roles.superuser", roles.superuser)?;
    let replicator = normalize_postgres_role_v2("postgres.roles.replicator", roles.replicator)?;
    let rewinder = normalize_postgres_role_v2("postgres.roles.rewinder", roles.rewinder)?;

    Ok(PostgresRolesConfig {
        superuser,
        replicator,
        rewinder,
    })
}

fn normalize_postgres_role_v2(
    field_prefix: &'static str,
    input: Option<super::schema::PostgresRoleConfigV2Input>,
) -> Result<PostgresRoleConfig, ConfigError> {
    let role = input.ok_or_else(|| ConfigError::Validation {
        field: field_prefix,
        message: "missing required secure config block for config_version=v2".to_string(),
    })?;

    let username_field = match field_prefix {
        "postgres.roles.superuser" => "postgres.roles.superuser.username",
        "postgres.roles.replicator" => "postgres.roles.replicator.username",
        "postgres.roles.rewinder" => "postgres.roles.rewinder.username",
        _ => field_prefix,
    };
    let auth_field = match field_prefix {
        "postgres.roles.superuser" => "postgres.roles.superuser.auth",
        "postgres.roles.replicator" => "postgres.roles.replicator.auth",
        "postgres.roles.rewinder" => "postgres.roles.rewinder.auth",
        _ => field_prefix,
    };

    let username = role.username.ok_or_else(|| ConfigError::Validation {
        field: username_field,
        message: "missing required secure field for config_version=v2".to_string(),
    })?;
    validate_non_empty(username_field, username.as_str())?;

    let auth = role.auth.ok_or_else(|| ConfigError::Validation {
        field: auth_field,
        message: "missing required secure field for config_version=v2".to_string(),
    })?;

    let auth = normalize_role_auth_config_v2(auth_field, auth)?;

    Ok(PostgresRoleConfig { username, auth })
}

fn normalize_role_auth_config_v2(
    field_prefix: &'static str,
    input: RoleAuthConfigV2Input,
) -> Result<RoleAuthConfig, ConfigError> {
    match input {
        RoleAuthConfigV2Input::Tls => Ok(RoleAuthConfig::Tls),
        RoleAuthConfigV2Input::Password { password } => {
            let password_field = match field_prefix {
                "postgres.roles.superuser.auth" => "postgres.roles.superuser.auth.password",
                "postgres.roles.replicator.auth" => "postgres.roles.replicator.auth.password",
                "postgres.roles.rewinder.auth" => "postgres.roles.rewinder.auth.password",
                _ => field_prefix,
            };

            let password = password.ok_or_else(|| ConfigError::Validation {
                field: password_field,
                message: "missing required secure field for config_version=v2".to_string(),
            })?;

            Ok(RoleAuthConfig::Password { password })
        }
    }
}

fn normalize_pg_hba_v2(
    input: Option<super::schema::PgHbaConfigV2Input>,
) -> Result<PgHbaConfig, ConfigError> {
    let cfg = input.ok_or_else(|| ConfigError::Validation {
        field: "postgres.pg_hba",
        message: "missing required secure config block for config_version=v2".to_string(),
    })?;
    let source = cfg.source.ok_or_else(|| ConfigError::Validation {
        field: "postgres.pg_hba.source",
        message: "missing required secure field for config_version=v2".to_string(),
    })?;
    Ok(PgHbaConfig { source })
}

fn normalize_pg_ident_v2(
    input: Option<super::schema::PgIdentConfigV2Input>,
) -> Result<PgIdentConfig, ConfigError> {
    let cfg = input.ok_or_else(|| ConfigError::Validation {
        field: "postgres.pg_ident",
        message: "missing required secure config block for config_version=v2".to_string(),
    })?;
    let source = cfg.source.ok_or_else(|| ConfigError::Validation {
        field: "postgres.pg_ident.source",
        message: "missing required secure field for config_version=v2".to_string(),
    })?;
    Ok(PgIdentConfig { source })
}

fn normalize_api_config_v2(
    input: super::schema::ApiConfigV2Input,
) -> Result<ApiConfig, ConfigError> {
    let listen_addr = input.listen_addr.unwrap_or_else(default_api_listen_addr);

    let security = input.security.ok_or_else(|| ConfigError::Validation {
        field: "api.security",
        message: "missing required secure config block for config_version=v2".to_string(),
    })?;

    let tls = normalize_tls_server_config_v2("api.security.tls", security.tls)?;
    let auth = security.auth.ok_or_else(|| ConfigError::Validation {
        field: "api.security.auth",
        message: "missing required secure field for config_version=v2".to_string(),
    })?;

    Ok(ApiConfig {
        listen_addr,
        security: ApiSecurityConfig { tls, auth },
    })
}

fn normalize_tls_server_config_v2(
    field_prefix: &'static str,
    input: Option<super::schema::TlsServerConfigV2Input>,
) -> Result<TlsServerConfig, ConfigError> {
    let tls = input.ok_or_else(|| ConfigError::Validation {
        field: field_prefix,
        message: "missing required secure config block for config_version=v2".to_string(),
    })?;

    let mode_field = match field_prefix {
        "postgres.tls" => "postgres.tls.mode",
        "api.security.tls" => "api.security.tls.mode",
        _ => field_prefix,
    };
    let identity_field = match field_prefix {
        "postgres.tls" => "postgres.tls.identity",
        "api.security.tls" => "api.security.tls.identity",
        _ => field_prefix,
    };

    let mode = tls.mode.ok_or_else(|| ConfigError::Validation {
        field: mode_field,
        message: "missing required secure field for config_version=v2".to_string(),
    })?;

    let identity = match tls.identity {
        None => None,
        Some(identity) => Some(normalize_tls_server_identity_v2(identity_field, identity)?),
    };

    Ok(TlsServerConfig {
        mode,
        identity,
        client_auth: tls.client_auth,
    })
}

fn normalize_tls_server_identity_v2(
    field_prefix: &'static str,
    input: super::schema::TlsServerIdentityConfigV2Input,
) -> Result<TlsServerIdentityConfig, ConfigError> {
    let cert_chain_field = match field_prefix {
        "postgres.tls.identity" => "postgres.tls.identity.cert_chain",
        "api.security.tls.identity" => "api.security.tls.identity.cert_chain",
        _ => field_prefix,
    };
    let private_key_field = match field_prefix {
        "postgres.tls.identity" => "postgres.tls.identity.private_key",
        "api.security.tls.identity" => "api.security.tls.identity.private_key",
        _ => field_prefix,
    };

    let cert_chain = input.cert_chain.ok_or_else(|| ConfigError::Validation {
        field: cert_chain_field,
        message: "missing required secure field for config_version=v2".to_string(),
    })?;
    let private_key = input.private_key.ok_or_else(|| ConfigError::Validation {
        field: private_key_field,
        message: "missing required secure field for config_version=v2".to_string(),
    })?;

    Ok(TlsServerIdentityConfig {
        cert_chain,
        private_key,
    })
}

fn validate_absolute_path(field: &'static str, path: &Path) -> Result<(), ConfigError> {
    if !path.is_absolute() {
        return Err(ConfigError::Validation {
            field,
            message: "must be an absolute path".to_string(),
        });
    }
    Ok(())
}

fn normalize_path_lexical(path: &Path) -> PathBuf {
    use std::path::Component;

    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                let _ = out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

pub fn validate_runtime_config(cfg: &RuntimeConfig) -> Result<(), ConfigError> {
    validate_non_empty_path("postgres.data_dir", &cfg.postgres.data_dir)?;
    validate_non_empty("postgres.listen_host", cfg.postgres.listen_host.as_str())?;
    validate_port("postgres.listen_port", cfg.postgres.listen_port)?;
    validate_non_empty_path("postgres.socket_dir", &cfg.postgres.socket_dir)?;
    validate_non_empty_path("postgres.log_file", &cfg.postgres.log_file)?;

    validate_non_empty(
        "postgres.local_conn_identity.user",
        cfg.postgres.local_conn_identity.user.as_str(),
    )?;
    validate_non_empty(
        "postgres.local_conn_identity.dbname",
        cfg.postgres.local_conn_identity.dbname.as_str(),
    )?;
    validate_non_empty(
        "postgres.rewind_conn_identity.user",
        cfg.postgres.rewind_conn_identity.user.as_str(),
    )?;
    validate_non_empty(
        "postgres.rewind_conn_identity.dbname",
        cfg.postgres.rewind_conn_identity.dbname.as_str(),
    )?;

    validate_non_empty(
        "postgres.roles.superuser.username",
        cfg.postgres.roles.superuser.username.as_str(),
    )?;
    validate_non_empty(
        "postgres.roles.replicator.username",
        cfg.postgres.roles.replicator.username.as_str(),
    )?;
    validate_non_empty(
        "postgres.roles.rewinder.username",
        cfg.postgres.roles.rewinder.username.as_str(),
    )?;

    if cfg.postgres.local_conn_identity.user != cfg.postgres.roles.superuser.username {
        return Err(ConfigError::Validation {
            field: "postgres.local_conn_identity.user",
            message: format!(
                "must match postgres.roles.superuser.username (got `{}`, expected `{}`)",
                cfg.postgres.local_conn_identity.user, cfg.postgres.roles.superuser.username
            ),
        });
    }
    if cfg.postgres.rewind_conn_identity.user != cfg.postgres.roles.rewinder.username {
        return Err(ConfigError::Validation {
            field: "postgres.rewind_conn_identity.user",
            message: format!(
                "must match postgres.roles.rewinder.username (got `{}`, expected `{}`)",
                cfg.postgres.rewind_conn_identity.user, cfg.postgres.roles.rewinder.username
            ),
        });
    }

    validate_postgres_auth_tls_invariants(cfg)?;

    validate_role_auth(
        "postgres.roles.superuser.auth.password.path",
        "postgres.roles.superuser.auth.password.content",
        &cfg.postgres.roles.superuser.auth,
    )?;
    validate_role_auth(
        "postgres.roles.replicator.auth.password.path",
        "postgres.roles.replicator.auth.password.content",
        &cfg.postgres.roles.replicator.auth,
    )?;
    validate_role_auth(
        "postgres.roles.rewinder.auth.password.path",
        "postgres.roles.rewinder.auth.password.content",
        &cfg.postgres.roles.rewinder.auth,
    )?;

    validate_tls_server_config(
        "postgres.tls.identity",
        "postgres.tls.identity.cert_chain",
        "postgres.tls.identity.private_key",
        &cfg.postgres.tls,
    )?;
    validate_tls_client_auth_config(
        "postgres.tls.client_auth",
        "postgres.tls.client_auth.client_ca",
        &cfg.postgres.tls,
    )?;

    validate_inline_or_path_non_empty(
        "postgres.pg_hba.source",
        &cfg.postgres.pg_hba.source,
        false,
    )?;
    validate_inline_or_path_non_empty(
        "postgres.pg_ident.source",
        &cfg.postgres.pg_ident.source,
        false,
    )?;
    for (key, value) in &cfg.postgres.extra_gucs {
        validate_extra_guc_for_config(key.as_str(), value.as_str())?;
    }

    validate_non_empty_path("process.binaries.postgres", &cfg.process.binaries.postgres)?;
    validate_absolute_path("process.binaries.postgres", &cfg.process.binaries.postgres)?;
    validate_non_empty_path("process.binaries.pg_ctl", &cfg.process.binaries.pg_ctl)?;
    validate_absolute_path("process.binaries.pg_ctl", &cfg.process.binaries.pg_ctl)?;
    validate_non_empty_path(
        "process.binaries.pg_rewind",
        &cfg.process.binaries.pg_rewind,
    )?;
    validate_absolute_path(
        "process.binaries.pg_rewind",
        &cfg.process.binaries.pg_rewind,
    )?;
    validate_non_empty_path("process.binaries.initdb", &cfg.process.binaries.initdb)?;
    validate_absolute_path("process.binaries.initdb", &cfg.process.binaries.initdb)?;
    validate_non_empty_path(
        "process.binaries.pg_basebackup",
        &cfg.process.binaries.pg_basebackup,
    )?;
    validate_absolute_path(
        "process.binaries.pg_basebackup",
        &cfg.process.binaries.pg_basebackup,
    )?;
    validate_non_empty_path("process.binaries.psql", &cfg.process.binaries.psql)?;
    validate_absolute_path("process.binaries.psql", &cfg.process.binaries.psql)?;

    validate_timeout(
        "process.pg_rewind_timeout_ms",
        cfg.process.pg_rewind_timeout_ms,
    )?;
    validate_timeout(
        "process.bootstrap_timeout_ms",
        cfg.process.bootstrap_timeout_ms,
    )?;
    validate_timeout("process.fencing_timeout_ms", cfg.process.fencing_timeout_ms)?;

    validate_timeout(
        "logging.postgres.poll_interval_ms",
        cfg.logging.postgres.poll_interval_ms,
    )?;
    if let Some(path) = cfg.logging.postgres.pg_ctl_log_file.as_ref() {
        validate_non_empty_path("logging.postgres.pg_ctl_log_file", path)?;
        validate_absolute_path("logging.postgres.pg_ctl_log_file", path)?;
    }
    if let Some(path) = cfg.logging.postgres.log_dir.as_ref() {
        validate_non_empty_path("logging.postgres.log_dir", path)?;
        validate_absolute_path("logging.postgres.log_dir", path)?;
    }
    if cfg.logging.postgres.cleanup.enabled {
        if cfg.logging.postgres.cleanup.max_files == 0 {
            return Err(ConfigError::Validation {
                field: "logging.postgres.cleanup.max_files",
                message: "must be greater than zero when cleanup is enabled".to_string(),
            });
        }
        if cfg.logging.postgres.cleanup.max_age_seconds == 0 {
            return Err(ConfigError::Validation {
                field: "logging.postgres.cleanup.max_age_seconds",
                message: "must be greater than zero when cleanup is enabled".to_string(),
            });
        }
        if cfg.logging.postgres.cleanup.protect_recent_seconds == 0 {
            return Err(ConfigError::Validation {
                field: "logging.postgres.cleanup.protect_recent_seconds",
                message: "must be greater than zero when cleanup is enabled".to_string(),
            });
        }
    }

    if let Some(path) = cfg.logging.sinks.file.path.as_ref() {
        validate_non_empty_path("logging.sinks.file.path", path)?;
    }

    if cfg.logging.sinks.file.enabled && cfg.logging.sinks.file.path.is_none() {
        return Err(ConfigError::Validation {
            field: "logging.sinks.file.path",
            message: "must be configured when logging.sinks.file.enabled is true".to_string(),
        });
    }

    validate_non_empty_path("postgres.log_file", &cfg.postgres.log_file)?;
    validate_absolute_path("postgres.log_file", &cfg.postgres.log_file)?;

    if cfg.logging.sinks.file.enabled {
        if let Some(path) = cfg.logging.sinks.file.path.as_ref() {
            validate_absolute_path("logging.sinks.file.path", path)?;
        }
    }

    validate_logging_path_ownership_invariants(cfg)?;

    if cfg.dcs.endpoints.is_empty() {
        return Err(ConfigError::Validation {
            field: "dcs.endpoints",
            message: "must contain at least one endpoint".to_string(),
        });
    }

    for endpoint in &cfg.dcs.endpoints {
        if endpoint.trim().is_empty() {
            return Err(ConfigError::Validation {
                field: "dcs.endpoints",
                message: "must not contain empty endpoint values".to_string(),
            });
        }
    }

    if cfg.dcs.scope.trim().is_empty() {
        return Err(ConfigError::Validation {
            field: "dcs.scope",
            message: "must not be empty".to_string(),
        });
    }

    if cfg.ha.loop_interval_ms == 0 {
        return Err(ConfigError::Validation {
            field: "ha.loop_interval_ms",
            message: "must be greater than zero".to_string(),
        });
    }

    if cfg.ha.lease_ttl_ms == 0 {
        return Err(ConfigError::Validation {
            field: "ha.lease_ttl_ms",
            message: "must be greater than zero".to_string(),
        });
    }

    if cfg.ha.lease_ttl_ms <= cfg.ha.loop_interval_ms {
        return Err(ConfigError::Validation {
            field: "ha.lease_ttl_ms",
            message: "must be greater than ha.loop_interval_ms".to_string(),
        });
    }

    match &cfg.api.security.auth {
        crate::config::ApiAuthConfig::Disabled => {}
        crate::config::ApiAuthConfig::RoleTokens(tokens) => {
            validate_optional_non_empty(
                "api.security.auth.role_tokens.read_token",
                tokens.read_token.as_deref(),
            )?;
            validate_optional_non_empty(
                "api.security.auth.role_tokens.admin_token",
                tokens.admin_token.as_deref(),
            )?;
            if tokens.read_token.is_none() && tokens.admin_token.is_none() {
                return Err(ConfigError::Validation {
                    field: "api.security.auth.role_tokens",
                    message: "at least one of read_token or admin_token must be configured"
                        .to_string(),
                });
            }
        }
    }

    validate_tls_server_config(
        "api.security.tls.identity",
        "api.security.tls.identity.cert_chain",
        "api.security.tls.identity.private_key",
        &cfg.api.security.tls,
    )?;
    validate_tls_client_auth_config(
        "api.security.tls.client_auth",
        "api.security.tls.client_auth.client_ca",
        &cfg.api.security.tls,
    )?;

    validate_dcs_init_config(cfg)?;

    Ok(())
}

fn validate_extra_guc_for_config(key: &str, value: &str) -> Result<(), ConfigError> {
    validate_extra_guc_entry(key, value).map_err(|err| match err {
        ManagedPostgresConfError::InvalidExtraGuc { key, message } => ConfigError::Validation {
            field: "postgres.extra_gucs",
            message: format!("entry `{key}` invalid: {message}"),
        },
        ManagedPostgresConfError::ReservedExtraGuc { key } => ConfigError::Validation {
            field: "postgres.extra_gucs",
            message: format!("entry `{key}` is reserved by pgtuskmaster"),
        },
        ManagedPostgresConfError::InvalidPrimarySlotName { slot, message } => {
            ConfigError::Validation {
                field: "postgres.extra_gucs",
                message: format!(
                    "unexpected replica slot validation while checking extra gucs `{slot}`: {message}"
                ),
            }
        }
    })
}

fn validate_logging_path_ownership_invariants(cfg: &RuntimeConfig) -> Result<(), ConfigError> {
    let Some(sink_path) = cfg.logging.sinks.file.path.as_ref() else {
        return Ok(());
    };
    if !cfg.logging.sinks.file.enabled {
        return Ok(());
    }

    let effective_pg_ctl_log_file = match cfg.logging.postgres.pg_ctl_log_file.as_ref() {
        Some(path) => path,
        None => &cfg.postgres.log_file,
    };

    let sink_path = normalize_path_lexical(sink_path);
    let postgres_log_file = normalize_path_lexical(&cfg.postgres.log_file);
    let effective_pg_ctl_log_file = normalize_path_lexical(effective_pg_ctl_log_file);

    let tailed_files: [(&'static str, &PathBuf); 2] = [
        ("postgres.log_file", &postgres_log_file),
        (
            "logging.postgres.pg_ctl_log_file",
            &effective_pg_ctl_log_file,
        ),
    ];

    for (field, path) in tailed_files {
        if &sink_path == path {
            return Err(ConfigError::Validation {
                field: "logging.sinks.file.path",
                message: format!("must not equal tailed postgres input {field}"),
            });
        }
    }

    if let Some(log_dir) = cfg.logging.postgres.log_dir.as_ref() {
        let log_dir = normalize_path_lexical(log_dir);
        if sink_path.starts_with(&log_dir) {
            return Err(ConfigError::Validation {
                field: "logging.sinks.file.path",
                message: "must not be inside logging.postgres.log_dir (would self-ingest)"
                    .to_string(),
            });
        }
    }

    Ok(())
}

fn validate_non_empty_path(field: &'static str, path: &Path) -> Result<(), ConfigError> {
    if path.as_os_str().is_empty() {
        return Err(ConfigError::Validation {
            field,
            message: "must not be empty".to_string(),
        });
    }
    Ok(())
}

fn validate_timeout(field: &'static str, value: u64) -> Result<(), ConfigError> {
    if !(MIN_TIMEOUT_MS..=MAX_TIMEOUT_MS).contains(&value) {
        return Err(ConfigError::Validation {
            field,
            message: format!("must be between {MIN_TIMEOUT_MS} and {MAX_TIMEOUT_MS} ms"),
        });
    }
    Ok(())
}

fn validate_port(field: &'static str, value: u16) -> Result<(), ConfigError> {
    if value == 0 {
        return Err(ConfigError::Validation {
            field,
            message: "must be greater than zero".to_string(),
        });
    }
    Ok(())
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), ConfigError> {
    if value.trim().is_empty() {
        return Err(ConfigError::Validation {
            field,
            message: "must not be empty".to_string(),
        });
    }
    Ok(())
}

fn validate_optional_non_empty(
    field: &'static str,
    value: Option<&str>,
) -> Result<(), ConfigError> {
    if let Some(raw) = value {
        if raw.trim().is_empty() {
            return Err(ConfigError::Validation {
                field,
                message: "must not be empty when configured".to_string(),
            });
        }
    }
    Ok(())
}

fn validate_role_auth(
    password_path_field: &'static str,
    password_content_field: &'static str,
    auth: &RoleAuthConfig,
) -> Result<(), ConfigError> {
    match auth {
        RoleAuthConfig::Tls => Ok(()),
        RoleAuthConfig::Password { password } => {
            validate_secret_source_non_empty(password_path_field, password_content_field, password)
        }
    }
}

fn validate_postgres_auth_tls_invariants(cfg: &RuntimeConfig) -> Result<(), ConfigError> {
    validate_postgres_role_auth_supported(
        "postgres.roles.superuser.auth",
        &cfg.postgres.roles.superuser.auth,
    )?;
    validate_postgres_role_auth_supported(
        "postgres.roles.replicator.auth",
        &cfg.postgres.roles.replicator.auth,
    )?;
    validate_postgres_role_auth_supported(
        "postgres.roles.rewinder.auth",
        &cfg.postgres.roles.rewinder.auth,
    )?;

    validate_postgres_conn_identity_ssl_mode_supported(
        "postgres.local_conn_identity.ssl_mode",
        cfg.postgres.local_conn_identity.ssl_mode,
        cfg.postgres.tls.mode,
    )?;
    validate_postgres_conn_identity_ssl_mode_supported(
        "postgres.rewind_conn_identity.ssl_mode",
        cfg.postgres.rewind_conn_identity.ssl_mode,
        cfg.postgres.tls.mode,
    )?;

    Ok(())
}

fn validate_postgres_role_auth_supported(
    field: &'static str,
    auth: &RoleAuthConfig,
) -> Result<(), ConfigError> {
    match auth {
        RoleAuthConfig::Tls => Err(ConfigError::Validation {
            field,
            message:
                "postgresql role TLS client auth is not implemented; use type = \"password\" for now"
                    .to_string(),
        }),
        RoleAuthConfig::Password { .. } => Ok(()),
    }
}

fn validate_postgres_conn_identity_ssl_mode_supported(
    field: &'static str,
    ssl_mode: crate::pginfo::conninfo::PgSslMode,
    tls_mode: crate::config::ApiTlsMode,
) -> Result<(), ConfigError> {
    if matches!(tls_mode, crate::config::ApiTlsMode::Disabled)
        && postgres_ssl_mode_requires_server_tls(ssl_mode)
    {
        return Err(ConfigError::Validation {
            field,
            message: format!(
                "must not require server TLS when postgres.tls.mode is disabled (got `{}`)",
                ssl_mode.as_str()
            ),
        });
    }

    Ok(())
}

fn postgres_ssl_mode_requires_server_tls(ssl_mode: crate::pginfo::conninfo::PgSslMode) -> bool {
    matches!(
        ssl_mode,
        crate::pginfo::conninfo::PgSslMode::Require
            | crate::pginfo::conninfo::PgSslMode::VerifyCa
            | crate::pginfo::conninfo::PgSslMode::VerifyFull
    )
}

fn validate_tls_server_config(
    identity_field: &'static str,
    cert_chain_field: &'static str,
    private_key_field: &'static str,
    cfg: &TlsServerConfig,
) -> Result<(), ConfigError> {
    if matches!(cfg.mode, crate::config::ApiTlsMode::Disabled) {
        return Ok(());
    }

    let identity = cfg
        .identity
        .as_ref()
        .ok_or_else(|| ConfigError::Validation {
            field: identity_field,
            message: "tls identity must be configured when tls.mode is optional or required"
                .to_string(),
        })?;

    validate_inline_or_path_non_empty(cert_chain_field, &identity.cert_chain, false)?;
    validate_inline_or_path_non_empty(private_key_field, &identity.private_key, false)?;
    Ok(())
}

fn validate_tls_client_auth_config(
    client_auth_field: &'static str,
    client_ca_field: &'static str,
    cfg: &TlsServerConfig,
) -> Result<(), ConfigError> {
    let Some(client_auth) = cfg.client_auth.as_ref() else {
        return Ok(());
    };

    if matches!(cfg.mode, crate::config::ApiTlsMode::Disabled) {
        return Err(ConfigError::Validation {
            field: client_auth_field,
            message: "must not be configured when tls.mode is disabled".to_string(),
        });
    }

    validate_inline_or_path_non_empty(client_ca_field, &client_auth.client_ca, false)?;
    Ok(())
}

fn validate_dcs_init_config(cfg: &RuntimeConfig) -> Result<(), ConfigError> {
    let Some(init) = cfg.dcs.init.as_ref() else {
        return Ok(());
    };

    validate_non_empty("dcs.init.payload_json", init.payload_json.as_str())?;

    let _: serde_json::Value = serde_json::from_str(init.payload_json.as_str()).map_err(|err| {
        ConfigError::Validation {
            field: "dcs.init.payload_json",
            message: format!("must be valid JSON: {err}"),
        }
    })?;

    let _: RuntimeConfig = serde_json::from_str(init.payload_json.as_str()).map_err(|err| {
        ConfigError::Validation {
            field: "dcs.init.payload_json",
            message: format!("must decode as a RuntimeConfig JSON document: {err}"),
        }
    })?;

    Ok(())
}

fn validate_secret_source_non_empty(
    path_field: &'static str,
    content_field: &'static str,
    secret: &SecretSource,
) -> Result<(), ConfigError> {
    validate_inline_or_path_non_empty_for_secret(path_field, content_field, &secret.0)
}

fn validate_inline_or_path_non_empty_for_secret(
    path_field: &'static str,
    content_field: &'static str,
    value: &InlineOrPath,
) -> Result<(), ConfigError> {
    match value {
        InlineOrPath::Path(path) => validate_non_empty_path(path_field, path),
        InlineOrPath::PathConfig { path } => validate_non_empty_path(path_field, path),
        InlineOrPath::Inline { content } => validate_non_empty(content_field, content.as_str()),
    }
}

fn validate_inline_or_path_non_empty(
    field: &'static str,
    value: &InlineOrPath,
    allow_empty_inline: bool,
) -> Result<(), ConfigError> {
    match value {
        InlineOrPath::Path(path) => validate_non_empty_path(field, path),
        InlineOrPath::PathConfig { path } => validate_non_empty_path(field, path),
        InlineOrPath::Inline { content } => {
            if allow_empty_inline {
                Ok(())
            } else {
                validate_non_empty(field, content.as_str())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::config::schema::{
        ApiAuthConfig, ApiConfig, ApiRoleTokensConfig, ApiSecurityConfig, ApiTlsMode, BinaryPaths,
        ClusterConfig, DcsConfig, DebugConfig, FileSinkConfig, FileSinkMode, HaConfig,
        InlineOrPath, LogCleanupConfig, LogLevel, LoggingConfig, LoggingSinksConfig, PgHbaConfig,
        PgIdentConfig, PostgresConfig, PostgresConnIdentityConfig, PostgresLoggingConfig,
        PostgresRoleConfig, PostgresRolesConfig, ProcessConfig, RoleAuthConfig, RuntimeConfig,
        StderrSinkConfig, TlsServerConfig,
    };
    use crate::pginfo::conninfo::PgSslMode;

    fn sample_password_auth() -> RoleAuthConfig {
        RoleAuthConfig::Password {
            password: crate::config::SecretSource(crate::config::InlineOrPath::Inline {
                content: "secret-password".to_string(),
            }),
        }
    }

    fn expect_validation_error(
        result: Result<(), ConfigError>,
        expected_field: &'static str,
        expected_message_fragment: &str,
    ) -> Result<(), String> {
        match result {
            Err(ConfigError::Validation { field, message }) => {
                if field != expected_field {
                    return Err(format!(
                        "expected validation field {expected_field}, got {field}"
                    ));
                }
                if !message.contains(expected_message_fragment) {
                    return Err(format!(
                        "expected validation message to contain {expected_message_fragment:?}, got {message:?}"
                    ));
                }
                Ok(())
            }
            other => Err(format!(
                "expected validation error for {expected_field}, got {other:?}"
            )),
        }
    }

    fn base_runtime_config() -> RuntimeConfig {
        RuntimeConfig {
            cluster: ClusterConfig {
                name: "cluster-a".to_string(),
                member_id: "member-a".to_string(),
            },
            postgres: PostgresConfig {
                data_dir: PathBuf::from("/var/lib/postgresql/data"),
                connect_timeout_s: 5,
                listen_host: "127.0.0.1".to_string(),
                listen_port: 5432,
                socket_dir: PathBuf::from("/tmp/pgtuskmaster/socket"),
                log_file: PathBuf::from("/tmp/pgtuskmaster/postgres.log"),
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
                        auth: sample_password_auth(),
                    },
                    replicator: PostgresRoleConfig {
                        username: "replicator".to_string(),
                        auth: sample_password_auth(),
                    },
                    rewinder: PostgresRoleConfig {
                        username: "rewinder".to_string(),
                        auth: sample_password_auth(),
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
                extra_gucs: std::collections::BTreeMap::new(),
            },
            dcs: DcsConfig {
                endpoints: vec!["http://127.0.0.1:2379".to_string()],
                scope: "scope-a".to_string(),
                init: None,
            },
            ha: HaConfig {
                loop_interval_ms: 1_000,
                lease_ttl_ms: 10_000,
            },
            process: ProcessConfig {
                pg_rewind_timeout_ms: 120_000,
                bootstrap_timeout_ms: 300_000,
                fencing_timeout_ms: 30_000,
                binaries: BinaryPaths {
                    postgres: PathBuf::from("/usr/bin/postgres"),
                    pg_ctl: PathBuf::from("/usr/bin/pg_ctl"),
                    pg_rewind: PathBuf::from("/usr/bin/pg_rewind"),
                    initdb: PathBuf::from("/usr/bin/initdb"),
                    pg_basebackup: PathBuf::from("/usr/bin/pg_basebackup"),
                    psql: PathBuf::from("/usr/bin/psql"),
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
            debug: DebugConfig { enabled: false },
        }
    }

    #[test]
    fn validate_runtime_config_accepts_valid_config() {
        let cfg = base_runtime_config();
        assert!(validate_runtime_config(&cfg).is_ok());
    }

    #[test]
    fn validate_runtime_config_rejects_postgres_role_tls_auth() -> Result<(), String> {
        let mut superuser_cfg = base_runtime_config();
        superuser_cfg.postgres.roles.superuser.auth = RoleAuthConfig::Tls;
        expect_validation_error(
            validate_runtime_config(&superuser_cfg),
            "postgres.roles.superuser.auth",
            "type = \"password\"",
        )?;

        let mut replicator_cfg = base_runtime_config();
        replicator_cfg.postgres.roles.replicator.auth = RoleAuthConfig::Tls;
        expect_validation_error(
            validate_runtime_config(&replicator_cfg),
            "postgres.roles.replicator.auth",
            "type = \"password\"",
        )?;

        let mut rewinder_cfg = base_runtime_config();
        rewinder_cfg.postgres.roles.rewinder.auth = RoleAuthConfig::Tls;
        expect_validation_error(
            validate_runtime_config(&rewinder_cfg),
            "postgres.roles.rewinder.auth",
            "type = \"password\"",
        )
    }

    #[test]
    fn validate_runtime_config_rejects_local_conn_ssl_mode_requiring_tls_when_postgres_tls_disabled(
    ) -> Result<(), String> {
        let mut cfg = base_runtime_config();
        cfg.postgres.local_conn_identity.ssl_mode = PgSslMode::Require;

        expect_validation_error(
            validate_runtime_config(&cfg),
            "postgres.local_conn_identity.ssl_mode",
            "postgres.tls.mode is disabled",
        )
    }

    #[test]
    fn validate_runtime_config_rejects_rewind_conn_ssl_mode_requiring_tls_when_postgres_tls_disabled(
    ) -> Result<(), String> {
        let mut cfg = base_runtime_config();
        cfg.postgres.rewind_conn_identity.ssl_mode = PgSslMode::VerifyFull;

        expect_validation_error(
            validate_runtime_config(&cfg),
            "postgres.rewind_conn_identity.ssl_mode",
            "postgres.tls.mode is disabled",
        )
    }

    #[test]
    fn validate_runtime_config_rejects_empty_binary_path() {
        let mut cfg = base_runtime_config();
        cfg.process.binaries.pg_ctl = PathBuf::new();

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "process.binaries.pg_ctl",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_rejects_non_absolute_binary_paths() {
        let mut cfg = base_runtime_config();
        cfg.process.binaries.pg_ctl = PathBuf::from("pg_ctl");
        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "process.binaries.pg_ctl",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_rejects_bad_timeout() {
        let mut cfg = base_runtime_config();
        cfg.process.bootstrap_timeout_ms = 0;

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "process.bootstrap_timeout_ms",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_rejects_invalid_postgres_runtime_fields() {
        let mut cfg = base_runtime_config();
        cfg.postgres.listen_host = " ".to_string();
        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.listen_host",
                ..
            })
        ));

        let mut cfg = base_runtime_config();
        cfg.postgres.listen_port = 0;
        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.listen_port",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_rejects_missing_dcs_and_ha_invariants() {
        let mut cfg = base_runtime_config();
        cfg.dcs.endpoints.clear();

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "dcs.endpoints",
                ..
            })
        ));

        let mut cfg = base_runtime_config();
        cfg.ha.lease_ttl_ms = cfg.ha.loop_interval_ms;

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "ha.lease_ttl_ms",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_rejects_blank_api_tokens() {
        let mut cfg = base_runtime_config();
        cfg.api.security.auth = ApiAuthConfig::RoleTokens(ApiRoleTokensConfig {
            read_token: Some(" ".to_string()),
            admin_token: None,
        });

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "api.security.auth.role_tokens.read_token",
                ..
            })
        ));

        let mut cfg = base_runtime_config();
        cfg.api.security.auth = ApiAuthConfig::RoleTokens(ApiRoleTokensConfig {
            read_token: None,
            admin_token: Some("\t".to_string()),
        });

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "api.security.auth.role_tokens.admin_token",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_rejects_file_sink_enabled_without_path() {
        let mut cfg = base_runtime_config();
        cfg.logging.sinks.file.enabled = true;
        cfg.logging.sinks.file.path = None;

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "logging.sinks.file.path",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_rejects_file_sink_empty_path() {
        let mut cfg = base_runtime_config();
        cfg.logging.sinks.file.enabled = true;
        cfg.logging.sinks.file.path = Some(PathBuf::new());

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "logging.sinks.file.path",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_accepts_file_sink_enabled_with_path() {
        let mut cfg = base_runtime_config();
        cfg.logging.sinks.file.enabled = true;
        cfg.logging.sinks.file.path = Some(PathBuf::from("/tmp/pgtuskmaster.jsonl"));

        assert!(validate_runtime_config(&cfg).is_ok());
    }

    #[test]
    fn validate_runtime_config_rejects_file_sink_equal_to_tailed_log_via_dot_segments() {
        let mut cfg = base_runtime_config();
        cfg.logging.sinks.file.enabled = true;
        cfg.logging.sinks.file.path = Some(PathBuf::from("/tmp/pgtuskmaster/./postgres.log"));

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "logging.sinks.file.path",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_rejects_file_sink_equal_to_tailed_log_via_parent_segments() {
        let mut cfg = base_runtime_config();
        cfg.logging.sinks.file.enabled = true;
        cfg.logging.sinks.file.path = Some(PathBuf::from("/tmp/pgtuskmaster/tmp/../postgres.log"));

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "logging.sinks.file.path",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_rejects_file_sink_inside_log_dir_via_dot_segments() {
        let mut cfg = base_runtime_config();
        cfg.logging.postgres.log_dir = Some(PathBuf::from("/tmp/pgtuskmaster/log_dir"));
        cfg.logging.sinks.file.enabled = true;
        cfg.logging.sinks.file.path = Some(PathBuf::from("/tmp/pgtuskmaster/log_dir/./out.jsonl"));

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "logging.sinks.file.path",
                ..
            })
        ));
    }

    #[test]
    fn load_runtime_config_missing_config_version_is_rejected(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!("runtime-config-{unique}.toml"));

        let toml = r#"
[cluster]
name = "cluster-a"
member_id = "member-a"
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "config_version",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_config_version_v1_is_rejected() -> Result<(), Box<dyn std::error::Error>>
    {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!("runtime-config-invalid-{unique}.toml"));

        let toml = r#"
config_version = "v1"
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "config_version",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_rejects_unknown_fields_in_v2() -> Result<(), Box<dyn std::error::Error>>
    {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!("runtime-config-invalid-{unique}.toml"));

        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
connect_timeout_s = 5
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }
unknown = 10

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
pg_rewind_timeout_ms = 120000
bootstrap_timeout_ms = 300000
fencing_timeout_ms = 30000
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[logging]
level = "info"
capture_subprocess_output = true
postgres = { enabled = true, poll_interval_ms = 200, cleanup = { enabled = true, max_files = 10, max_age_seconds = 60 } }
sinks = { stderr = { enabled = true }, file = { enabled = false, mode = "append" } }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(err, Err(ConfigError::Parse { .. })));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_v2_happy_path_with_safe_defaults(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!("runtime-config-v2-{unique}.toml"));

        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;
        let cfg = load_runtime_config(&path)?;
        assert_eq!(cfg.postgres.connect_timeout_s, 5);
        assert_eq!(cfg.process.pg_rewind_timeout_ms, 120_000);
        assert_eq!(cfg.process.bootstrap_timeout_ms, 300_000);
        assert_eq!(cfg.process.fencing_timeout_ms, 30_000);
        assert_eq!(cfg.api.listen_addr, "127.0.0.1:8080");
        assert!(!cfg.debug.enabled);

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_v2_missing_secure_fields_is_actionable(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!("runtime-config-v2-missing-{unique}.toml"));

        // Intentionally omit `postgres.local_conn_identity`.
        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.local_conn_identity",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_v2_missing_process_binaries_is_actionable(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("runtime-config-v2-missing-binaries-{unique}.toml"));

        // Intentionally omit `process.binaries`.
        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
pg_rewind_timeout_ms = 120000
bootstrap_timeout_ms = 300000
fencing_timeout_ms = 30000

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "process.binaries",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_v2_password_auth_missing_password_is_actionable(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "runtime-config-v2-missing-auth-password-{unique}.toml"
        ));

        // Intentionally omit `postgres.roles.superuser.auth.password`.
        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "password" } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.roles.superuser.auth.password",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_v2_missing_postgres_roles_block_is_actionable(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("runtime-config-v2-missing-roles-{unique}.toml"));

        // Intentionally omit `postgres.roles`.
        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.roles",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_v2_missing_replicator_role_is_actionable(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "runtime-config-v2-missing-replicator-role-{unique}.toml"
        ));

        // Intentionally omit `postgres.roles.replicator`.
        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.roles.replicator",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_v2_missing_replicator_username_is_actionable(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "runtime-config-v2-missing-replicator-username-{unique}.toml"
        ));

        // Intentionally omit `postgres.roles.replicator.username`.
        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, replicator = { auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.roles.replicator.username",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_v2_missing_replicator_auth_is_actionable(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "runtime-config-v2-missing-replicator-auth-{unique}.toml"
        ));

        // Intentionally omit `postgres.roles.replicator.auth`.
        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, replicator = { username = "replicator" }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.roles.replicator.auth",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_v2_rejects_conn_identity_role_mismatch(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "runtime-config-v2-conn-identity-mismatch-{unique}.toml"
        ));

        // Intentionally set local_conn_identity.user to a different user than roles.superuser.username.
        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "not-postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.local_conn_identity.user",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_v2_rejects_blank_password_secret(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "runtime-config-v2-blank-password-secret-{unique}.toml"
        ));

        // Intentionally set password secret content to empty.
        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "" } } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.roles.superuser.auth.password.content",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_v2_rejects_tls_required_without_identity(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "runtime-config-v2-required-tls-no-identity-{unique}.toml"
        ));

        // Intentionally omit `postgres.tls.identity` while requiring TLS.
        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "required" }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.tls.identity",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_v2_rejects_client_auth_with_tls_disabled(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "runtime-config-v2-client-auth-with-tls-disabled-{unique}.toml"
        ));

        // Intentionally configure client auth while TLS is disabled.
        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled", client_auth = { client_ca = { content = "client-ca" }, require_client_cert = false } }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.tls.client_auth",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_v2_rejects_postgres_role_tls_auth_with_actionable_error(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "runtime-config-v2-postgres-role-tls-auth-{unique}.toml"
        ));

        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "tls" } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        let mapped = match err {
            Err(ConfigError::Validation { field, message }) => {
                if field != "postgres.roles.superuser.auth" {
                    Err(format!(
                        "expected validation field postgres.roles.superuser.auth, got {field}"
                    ))
                } else if !message.contains("type = \"password\"") {
                    Err(format!(
                        "expected validation message to mention password auth, got {message:?}"
                    ))
                } else {
                    Ok(())
                }
            }
            other => Err(format!("expected validation error, got {other:?}")),
        };
        mapped.map_err(std::io::Error::other)?;

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_v2_rejects_ssl_mode_requiring_tls_when_postgres_tls_disabled(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "runtime-config-v2-postgres-ssl-mode-requires-tls-{unique}.toml"
        ));

        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "require" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "verify-full" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        let mapped = match err {
            Err(ConfigError::Validation { field, message }) => {
                if field != "postgres.local_conn_identity.ssl_mode" {
                    Err(format!(
                        "expected validation field postgres.local_conn_identity.ssl_mode, got {field}"
                    ))
                } else if !message.contains("postgres.tls.mode is disabled") {
                    Err(format!(
                        "expected validation message to mention disabled postgres TLS, got {message:?}"
                    ))
                } else {
                    Ok(())
                }
            }
            other => Err(format!("expected validation error, got {other:?}")),
        };
        mapped.map_err(std::io::Error::other)?;

        std::fs::remove_file(&path)?;
        Ok(())
    }
}

--- END FILE: src/config/parser.rs ---

--- BEGIN FILE: tests/cli_binary.rs ---
use std::process::Command;

fn write_temp_config(label: &str, toml: &str) -> Result<std::path::PathBuf, String> {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|err| format!("system time error: {err}"))?
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "pgtuskmaster-cli-config-{label}-{unique}-{}",
        std::process::id()
    ));

    std::fs::write(&path, toml).map_err(|err| format!("write config failed: {err}"))?;
    Ok(path)
}

fn cli_bin_path() -> Result<std::path::PathBuf, String> {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_pgtuskmasterctl") {
        return Ok(std::path::PathBuf::from(path));
    }

    let current = std::env::current_exe().map_err(|err| format!("current_exe failed: {err}"))?;
    let debug_dir = current
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| "failed to derive target/debug directory".to_string())?;
    let mut candidate = debug_dir.join("pgtuskmasterctl");
    if cfg!(windows) {
        candidate.set_extension("exe");
    }
    if candidate.exists() {
        Ok(candidate)
    } else {
        Err(format!("cli binary not found at {}", candidate.display()))
    }
}

fn node_bin_path() -> Result<std::path::PathBuf, String> {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_pgtuskmaster") {
        return Ok(std::path::PathBuf::from(path));
    }

    let current = std::env::current_exe().map_err(|err| format!("current_exe failed: {err}"))?;
    let debug_dir = current
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| "failed to derive target/debug directory".to_string())?;
    let mut candidate = debug_dir.join("pgtuskmaster");
    if cfg!(windows) {
        candidate.set_extension("exe");
    }
    if candidate.exists() {
        Ok(candidate)
    } else {
        Err(format!("node binary not found at {}", candidate.display()))
    }
}

#[test]
fn help_exits_success() -> Result<(), String> {
    let bin = cli_bin_path()?;
    let output = Command::new(&bin)
        .arg("--help")
        .output()
        .map_err(|err| format!("failed to run --help: {err}"))?;

    assert!(
        output.status.success(),
        "--help should exit successfully, got {:?}",
        output.status.code()
    );

    let stdout = String::from_utf8(output.stdout)
        .map_err(|err| format!("stdout utf8 decode failed: {err}"))?;
    assert!(
        stdout.contains("ha"),
        "help output should include ha command"
    );
    Ok(())
}

#[test]
fn missing_required_subcommand_arg_exits_usage_code() -> Result<(), String> {
    let bin = cli_bin_path()?;
    let output = Command::new(&bin)
        .args(["ha", "leader", "set"])
        .output()
        .map_err(|err| format!("failed to run command: {err}"))?;

    assert_eq!(
        output.status.code(),
        Some(2),
        "clap usage failures should exit with code 2"
    );
    Ok(())
}

#[test]
fn state_command_maps_connection_refused_to_exit_3() -> Result<(), String> {
    let bin = cli_bin_path()?;
    let listener =
        std::net::TcpListener::bind("127.0.0.1:0").map_err(|err| format!("bind failed: {err}"))?;
    let addr = listener
        .local_addr()
        .map_err(|err| format!("local_addr failed: {err}"))?;
    drop(listener);

    let output = Command::new(&bin)
        .args([
            "--base-url",
            &format!("http://{addr}"),
            "--timeout-ms",
            "50",
            "ha",
            "state",
        ])
        .output()
        .map_err(|err| format!("failed to run state command: {err}"))?;

    assert_eq!(
        output.status.code(),
        Some(3),
        "transport errors should map to exit code 3"
    );

    let stderr = String::from_utf8(output.stderr)
        .map_err(|err| format!("stderr utf8 decode failed: {err}"))?;
    assert!(
        stderr.contains("transport error"),
        "stderr should mention transport error"
    );
    Ok(())
}

#[test]
fn node_help_exits_success() -> Result<(), String> {
    let bin = node_bin_path()?;
    let output = Command::new(&bin)
        .arg("--help")
        .output()
        .map_err(|err| format!("failed to run node --help: {err}"))?;

    assert!(
        output.status.success(),
        "--help should exit successfully, got {:?}",
        output.status.code()
    );

    let stdout = String::from_utf8(output.stdout)
        .map_err(|err| format!("stdout utf8 decode failed: {err}"))?;
    assert!(
        stdout.contains("--config"),
        "help output should include --config option"
    );
    Ok(())
}

#[test]
fn node_missing_config_version_prints_explicit_v2_migration_hint() -> Result<(), String> {
    let bin = node_bin_path()?;
    let path = write_temp_config(
        "missing-config-version",
        r#"
[cluster]
name = "cluster-a"
member_id = "member-a"
"#,
    )?;

    let output = Command::new(&bin)
        .args(["--config", path.to_string_lossy().as_ref()])
        .output()
        .map_err(|err| format!("failed to run node with missing config_version: {err}"))?;

    assert_eq!(
        output.status.code(),
        Some(1),
        "invalid configs should exit with code 1"
    );

    let stderr = String::from_utf8(output.stderr)
        .map_err(|err| format!("stderr utf8 decode failed: {err}"))?;
    assert!(
        stderr.contains("set config_version = \"v2\""),
        "stderr should include explicit v2 migration hint, got: {stderr}"
    );

    let _ = std::fs::remove_file(path);
    Ok(())
}

#[test]
fn node_missing_secure_field_prints_stable_field_path() -> Result<(), String> {
    let bin = node_bin_path()?;
    let path = write_temp_config(
        "missing-process-binaries",
        r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
pg_rewind_timeout_ms = 120000
bootstrap_timeout_ms = 300000
fencing_timeout_ms = 30000

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#,
    )?;

    let output = Command::new(&bin)
        .args(["--config", path.to_string_lossy().as_ref()])
        .output()
        .map_err(|err| format!("failed to run node with invalid config: {err}"))?;

    assert_eq!(
        output.status.code(),
        Some(1),
        "invalid configs should exit with code 1"
    );

    let stderr = String::from_utf8(output.stderr)
        .map_err(|err| format!("stderr utf8 decode failed: {err}"))?;
    assert!(
        stderr.contains("`process.binaries`"),
        "stderr should mention stable field path, got: {stderr}"
    );

    let _ = std::fs::remove_file(path);
    Ok(())
}

#[test]
fn node_rejects_postgres_role_tls_auth_with_stable_field_path() -> Result<(), String> {
    let bin = node_bin_path()?;
    let path = write_temp_config(
        "postgres-role-tls-auth",
        r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "tls" } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#,
    )?;

    let output = Command::new(&bin)
        .args(["--config", path.to_string_lossy().as_ref()])
        .output()
        .map_err(|err| format!("failed to run node with invalid config: {err}"))?;

    assert_eq!(output.status.code(), Some(1));

    let stderr = String::from_utf8(output.stderr)
        .map_err(|err| format!("stderr utf8 decode failed: {err}"))?;
    assert!(
        stderr.contains("`postgres.roles.superuser.auth`"),
        "stderr should mention stable field path, got: {stderr}"
    );

    let _ = std::fs::remove_file(path);
    Ok(())
}

#[test]
fn node_rejects_ssl_mode_requiring_tls_when_postgres_tls_disabled() -> Result<(), String> {
    let bin = node_bin_path()?;
    let path = write_temp_config(
        "postgres-ssl-mode-requires-tls",
        r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "require" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#,
    )?;

    let output = Command::new(&bin)
        .args(["--config", path.to_string_lossy().as_ref()])
        .output()
        .map_err(|err| format!("failed to run node with invalid config: {err}"))?;

    assert_eq!(output.status.code(), Some(1));

    let stderr = String::from_utf8(output.stderr)
        .map_err(|err| format!("stderr utf8 decode failed: {err}"))?;
    assert!(
        stderr.contains("`postgres.local_conn_identity.ssl_mode`"),
        "stderr should mention stable field path, got: {stderr}"
    );

    let _ = std::fs::remove_file(path);
    Ok(())
}

--- END FILE: tests/cli_binary.rs ---

