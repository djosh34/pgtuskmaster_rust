# Runtime configuration deep source summary

This file summarizes the runtime configuration surface strictly from:

- `src/config/schema.rs`
- `src/config/defaults.rs`
- `src/config/parser.rs`
- `docker/configs/cluster/node-a/runtime.toml`
- `docker/configs/single/node-a/runtime.toml`

## Supported config version

- `load_runtime_config` in `src/config/parser.rs` first deserializes a small envelope that only reads `config_version`.
- Missing `config_version` is rejected with a validation error that tells the operator to set `config_version = "v2"`.
- `config_version = "v1"` is explicitly rejected. The parser keeps a legacy v1 probing path only for diagnostics, but it always returns a validation error saying v1 is no longer supported.
- The only accepted version is `config_version = "v2"`.

## Top-level runtime config shape

The final normalized `RuntimeConfig` in `src/config/schema.rs` has these required top-level sections:

- `cluster`
- `postgres`
- `dcs`
- `ha`
- `process`
- `logging`
- `api`
- `debug`

The v2 input schema in `RuntimeConfigV2Input` requires:

- `config_version`
- `cluster`
- `postgres`
- `dcs`
- `ha`
- `process`
- `api`

The v2 input schema allows these top-level sections to be omitted and filled by defaults:

- `logging`
- `debug`

## `cluster`

`ClusterConfig` contains:

- `name: String`
- `member_id: String`

Validation from `validate_runtime_config`:

- `cluster.name` must not be empty
- `cluster.member_id` must not be empty

Both example configs set:

- `name` to either `docker-cluster` or `docker-single`
- `member_id = "node-a"`

## `postgres`

The normalized `PostgresConfig` contains:

- `data_dir: PathBuf`
- `connect_timeout_s: u32`
- `listen_host: String`
- `listen_port: u16`
- `socket_dir: PathBuf`
- `log_file: PathBuf`
- `local_conn_identity`
- `rewind_conn_identity`
- `tls`
- `roles`
- `pg_hba`
- `pg_ident`
- `extra_gucs: BTreeMap<String, String>`

### `postgres` fields that must be present in v2 input

`PostgresConfigV2Input` requires these fields in TOML:

- `data_dir`
- `listen_host`
- `listen_port`
- `socket_dir`
- `log_file`

These are represented as required Rust fields rather than `Option`, so TOML parsing itself requires them.

These blocks are required by normalization and produce explicit validation errors if missing:

- `local_conn_identity`
- `rewind_conn_identity`
- `tls`
- `roles`
- `pg_hba`
- `pg_ident`

`extra_gucs` is optional and defaults to an empty map.

### `postgres.connect_timeout_s`

- Optional in v2 input.
- Defaults to `5` seconds from `DEFAULT_PG_CONNECT_TIMEOUT_S`.
- Validation requires it to be greater than zero.

### `postgres.local_conn_identity` and `postgres.rewind_conn_identity`

Each identity becomes `PostgresConnIdentityConfig` with:

- `user`
- `dbname`
- `ssl_mode`

In v2 input, all three subfields are required by normalization:

- missing block returns validation error on `postgres.local_conn_identity` or `postgres.rewind_conn_identity`
- missing `user`, `dbname`, or `ssl_mode` returns field-specific validation errors

Validation also requires:

- `user` must not be empty
- `dbname` must not be empty
- if `postgres.tls.mode = "disabled"`, the connection identity `ssl_mode` must be compatible with disabled TLS

Both example configs use:

- `local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "disable" }`
- `rewind_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "disable" }`

### `postgres.tls`

`TlsServerConfig` contains:

- `mode`
- `identity`
- `client_auth`

`mode` uses the `ApiTlsMode` enum, aliased as `TlsMode`, with supported values:

- `disabled`
- `optional`
- `required`

Normalization requires the `postgres.tls` block and also requires `mode`.

Additional validation from parser rules:

- if TLS mode is `optional` or `required`, `identity` must be present
- `identity` requires `cert_chain` and `private_key`
- `client_auth`, when present, requires `client_ca`
- `require_client_cert` is a required boolean inside `TlsClientAuthConfig`

`InlineOrPath` supports three TOML shapes:

- a bare path value
- `{ path = "/some/file" }`
- `{ content = "inline material" }`

For secret-like inputs, inline content is accepted but redacted in `Debug`.

Both example configs use `tls = { mode = "disabled" }`.

### `postgres.roles`

`PostgresRolesConfig` contains required role blocks:

- `superuser`
- `replicator`
- `rewinder`

Each `PostgresRoleConfig` contains:

- `username`
- `auth`

Normalization requires all three role blocks in v2.

`auth` supports:

- `type = "tls"`
- `type = "password"` with nested `password`

Validation details:

- `username` must not be empty
- password auth requires a non-empty secret source
- the parser has a dedicated test that rejects TLS auth for postgres roles with an actionable validation error, so operators should treat password auth as the supported role-auth path here

Both example configs use password auth for all three postgres roles and source each password from a path under `/run/secrets/`.

### `postgres.pg_hba` and `postgres.pg_ident`

Each block contains one required field:

- `source`

Normalization requires both blocks and the `source` subfield.

Validation requires the source content/path to be non-empty. The parser passes `allow_inline_empty = false` when validating both sources.

Both example configs use:

- `pg_hba = { source = { path = "/etc/pgtuskmaster/pg_hba.conf" } }`
- `pg_ident = { source = { path = "/etc/pgtuskmaster/pg_ident.conf" } }`

### `postgres.extra_gucs`

- Optional map from string key to string value.
- Defaults to an empty map.
- Each entry is validated through `validate_extra_guc_entry`.
- Validation can fail if the key/value is invalid, reserved by pgtuskmaster, or violates replica slot naming constraints.

### Other `postgres` validation

`validate_runtime_config` also enforces:

- `postgres.data_dir` must be non-empty and absolute
- `postgres.listen_host` must not be empty
- `postgres.listen_port` must be greater than zero
- `postgres.socket_dir` must be non-empty and absolute
- `postgres.log_file` must be non-empty and absolute

## `dcs`

`DcsConfig` contains:

- `endpoints: Vec<String>`
- `scope: String`
- `init: Option<DcsInitConfig>`

`DcsInitConfig` contains:

- `payload_json: String`
- `write_on_bootstrap: bool`

Validation requires:

- `dcs.endpoints` must contain at least one endpoint
- endpoints must not be empty strings
- `dcs.scope` must not be empty
- when `dcs.init` is present, parser validation checks its payload and related invariants

The example configs set:

- `endpoints = ["http://etcd:2379"]`
- `scope` to match the cluster name

## `ha`

`HaConfig` contains:

- `loop_interval_ms: u64`
- `lease_ttl_ms: u64`

Validation requires:

- `ha.loop_interval_ms > 0`
- `ha.lease_ttl_ms > 0`
- `ha.lease_ttl_ms > ha.loop_interval_ms`

Both example configs set:

- `loop_interval_ms = 1000`
- `lease_ttl_ms = 10000`

## `process`

The normalized `ProcessConfig` contains:

- `pg_rewind_timeout_ms`
- `bootstrap_timeout_ms`
- `fencing_timeout_ms`
- `binaries`

### Process defaults

If omitted in v2 input, the timeout defaults are:

- `pg_rewind_timeout_ms = 120000`
- `bootstrap_timeout_ms = 300000`
- `fencing_timeout_ms = 30000`

These come from `src/config/defaults.rs`.

### `process.binaries`

The block is required in v2 input. Missing `process.binaries` or any individual binary path returns a validation error mentioning `missing required secure field for config_version=v2`.

`BinaryPaths` contains required absolute paths for:

- `postgres`
- `pg_ctl`
- `pg_rewind`
- `initdb`
- `pg_basebackup`
- `psql`

Validation requires every path to be:

- non-empty
- absolute

The example configs explicitly set all six binaries under `/usr/lib/postgresql/16/bin/`.

### Process timeout validation

`validate_timeout` applies to all process timeout fields and to `logging.postgres.poll_interval_ms`.

Allowed range:

- minimum `1`
- maximum `86_400_000`

The unit is milliseconds.

## `logging`

`logging` can be omitted in v2 input. If omitted, `default_logging_config()` is used.

Normalized `LoggingConfig` contains:

- `level`
- `capture_subprocess_output`
- `postgres`
- `sinks`

### Logging defaults

Defaults from `src/config/defaults.rs`:

- `level = "info"`
- `capture_subprocess_output = true`
- `logging.postgres.enabled = true`
- `logging.postgres.pg_ctl_log_file = None`
- `logging.postgres.log_dir = None`
- `logging.postgres.poll_interval_ms = 200`
- `logging.postgres.cleanup.enabled = true`
- `logging.postgres.cleanup.max_files = 50`
- `logging.postgres.cleanup.max_age_seconds = 604800`
- `logging.postgres.cleanup.protect_recent_seconds = 300`
- `logging.sinks.stderr.enabled = true`
- `logging.sinks.file.enabled = false`
- `logging.sinks.file.path = None`
- `logging.sinks.file.mode = "append"`

### Logging enums and sub-objects

`LogLevel` supports:

- `trace`
- `debug`
- `info`
- `warn`
- `error`
- `fatal`

`FileSinkMode` supports:

- `append`
- `truncate`

`PostgresLoggingConfig` contains:

- `enabled`
- `pg_ctl_log_file`
- `log_dir`
- `poll_interval_ms`
- `cleanup`

`LogCleanupConfig` contains:

- `enabled`
- `max_files`
- `max_age_seconds`
- `protect_recent_seconds`

`LoggingSinksConfig` contains:

- `stderr`
- `file`

### Logging validation

Validation requires:

- `logging.postgres.poll_interval_ms` within the shared timeout range `1..=86_400_000`
- if `logging.postgres.pg_ctl_log_file` is present, it must be non-empty and absolute
- if `logging.postgres.log_dir` is present, it must be non-empty and absolute
- when cleanup is enabled, `max_files`, `max_age_seconds`, and `protect_recent_seconds` must each be greater than zero
- if `logging.sinks.file.enabled = true`, `logging.sinks.file.path` must be configured
- when a file sink path is configured and enabled, it must be absolute
- the runtime log sink path must not equal `postgres.log_file`
- the runtime log sink path must not equal the effective pg_ctl log file
- the runtime log sink path must not be placed inside `logging.postgres.log_dir`, because that would cause self-ingest

The example configs do not rely on logging defaults because they set:

- `level = "info"`
- `capture_subprocess_output = true`
- `logging.postgres.enabled = true`
- `logging.postgres.poll_interval_ms = 200`
- `logging.postgres.cleanup = { enabled = true, max_files = 20, max_age_seconds = 86400, protect_recent_seconds = 300 }`
- `logging.sinks.stderr.enabled = true`
- `logging.sinks.file.enabled = true`
- `logging.sinks.file.path = "/var/log/pgtuskmaster/runtime.jsonl"`
- `logging.sinks.file.mode = "append"`

## `api`

`api` is required in v2 input, but `listen_addr` and `security` are normalized from a partially optional input structure.

Normalized `ApiConfig` contains:

- `listen_addr: String`
- `security`

`ApiSecurityConfig` contains:

- `tls`
- `auth`

`ApiAuthConfig` supports:

- `type = "disabled"`
- `type = "role_tokens"`

`ApiRoleTokensConfig` contains:

- `read_token: Option<String>`
- `admin_token: Option<String>`

### API defaults and requirements

- `api.listen_addr` defaults to `"127.0.0.1:8080"` if omitted
- `api.security` is required by normalization
- `api.security.auth` is required by normalization
- `api.security.tls` is required by normalization

### API validation

Validation requires:

- `api.listen_addr` must not be empty
- if auth mode is `role_tokens`, at least one of `read_token` or `admin_token` must be configured
- if `read_token` or `admin_token` is configured, it must not be empty
- TLS validation rules for `api.security.tls` match the general `TlsServerConfig` rules described above

Both example configs use:

- `listen_addr = "0.0.0.0:8080"`
- `security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }`

## `debug`

`debug` can be omitted in v2 input.

If omitted, `default_debug_config()` sets:

- `enabled = false`

Both example configs override the default and set:

- `enabled = true`

## Unknown fields and parsing model

- Many schema structs are annotated with `#[serde(deny_unknown_fields)]`.
- The parser test suite includes a test that rejects unknown fields in v2 input.
- As a result, operators should expect undocumented or misspelled fields to fail parsing rather than being ignored.

## Environment variable overrides

- The inspected config loader and defaults modules do not expose an environment-variable override layer for runtime configuration fields.
- The requested source files in this batch do not show any environment-driven override path for runtime TOML values.
- The examples use literal values and file/inline config sources only.

## Example config comparison

The inspected `docker/configs/cluster/node-a/runtime.toml` and `docker/configs/single/node-a/runtime.toml` are structurally identical except for:

- `cluster.name`
- `dcs.scope`

The cluster and single-node examples both declare `config_version = "v2"` and include all main sections explicitly rather than relying on the optional `logging` or `debug` defaults.
