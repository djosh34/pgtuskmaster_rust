# Runtime configuration deep summary

This note is a source-backed summary for `docs/src/reference/runtime-configuration.md`.

The runtime config entrypoint is `load_runtime_config(path)` in `src/config/parser.rs`.
It reads TOML text, parses a small envelope to extract `config_version`, and then only accepts `config_version = "v2"`.
If `config_version` is missing, parsing returns a validation error for `config_version` with guidance to set it to `v2`.
If `config_version = "v1"`, parsing rejects it explicitly and says v1 is no longer supported.

The top-level runtime config shape is `RuntimeConfig` in `src/config/schema.rs`.
It requires these top-level sections:
- `cluster`
- `postgres`
- `dcs`
- `ha`
- `process`
- `logging`
- `api`
- `debug`

For v2 input, some sections are structurally present but still allow defaults during normalization:
- `logging` is optional in `RuntimeConfigV2Input` and defaults via `default_logging_config()`
- `debug` is optional in `RuntimeConfigV2Input` and defaults via `default_debug_config()`
- `api.listen_addr` defaults to `127.0.0.1:8080`
- `postgres.connect_timeout_s` defaults to `5`
- `process.pg_rewind_timeout_ms` defaults to `120000`
- `process.bootstrap_timeout_ms` defaults to `300000`
- `process.fencing_timeout_ms` defaults to `30000`

Important secure-schema rule:
The parser intentionally requires explicit secure config blocks for fields like `postgres.local_conn_identity`, `postgres.roles`, `postgres.pg_hba`, `postgres.pg_ident`, `postgres.tls`, and `api.security`.
Missing these blocks yields `ConfigError::Validation` with field-specific messages saying the secure field or block is required for `config_version=v2`.

Top-level sections and important nested fields from `src/config/schema.rs`:

`cluster`
- `name: String`
- `member_id: String`

`postgres`
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

`postgres.local_conn_identity` and `postgres.rewind_conn_identity`
- `user`
- `dbname`
- `ssl_mode`

`postgres.roles`
- `superuser`
- `replicator`
- `rewinder`

Each role entry has:
- `username`
- `auth`

`auth` is tagged by `type` and currently supports:
- `tls`
- `password`

For `password`, the secret source is `SecretSource(InlineOrPath)`.
`InlineOrPath` supports:
- bare path
- `{ path = ... }`
- `{ content = ... }`

`postgres.tls` and `api.security.tls` use `TlsServerConfig`:
- `mode`
- `identity`
- `client_auth`

TLS mode enum values:
- `disabled`
- `optional`
- `required`

`dcs`
- `endpoints: Vec<String>`
- `scope: String`
- `init: Option<DcsInitConfig>`

`dcs.init`
- `payload_json`
- `write_on_bootstrap`

`ha`
- `loop_interval_ms`
- `lease_ttl_ms`

`process`
- `pg_rewind_timeout_ms`
- `bootstrap_timeout_ms`
- `fencing_timeout_ms`
- `binaries`

`process.binaries`
- `postgres`
- `pg_ctl`
- `pg_rewind`
- `initdb`
- `pg_basebackup`
- `psql`

`logging`
- `level`
- `capture_subprocess_output`
- `postgres`
- `sinks`

`logging.level` enum values:
- `trace`
- `debug`
- `info`
- `warn`
- `error`
- `fatal`

`logging.postgres`
- `enabled`
- `pg_ctl_log_file`
- `log_dir`
- `poll_interval_ms`
- `cleanup`

`logging.postgres.cleanup`
- `enabled`
- `max_files`
- `max_age_seconds`
- `protect_recent_seconds`

`logging.sinks.stderr`
- `enabled`

`logging.sinks.file`
- `enabled`
- `path`
- `mode`

`logging.sinks.file.mode` enum values:
- `append`
- `truncate`

`api`
- `listen_addr`
- `security`

`api.security.auth` supports:
- `disabled`
- `role_tokens`

When `role_tokens` is used, the config carries:
- `read_token`
- `admin_token`

`debug`
- `enabled`

Validation behavior from `validate_runtime_config(cfg)` and related helpers in `src/config/parser.rs`:
- path fields like `process.binaries.*` must be non-empty absolute paths
- timeout fields are checked with minimum and maximum bounds
- port fields are validated
- many string fields must be non-empty after trimming
- `postgres.local_conn_identity.user` must match `postgres.roles.superuser.username`
- `postgres.rewind_conn_identity.user` must match `postgres.roles.rewinder.username`
- postgres role auth and TLS settings are checked for supported combinations
- TLS identity and client-auth blocks are validated when TLS modes require them
- `postgres.pg_hba.source` and `postgres.pg_ident.source` must be non-empty
- `extra_gucs` keys and values are validated through `validate_extra_guc_entry`
- file sink and postgres logging paths are checked for ownership and overlap invariants
- DCS init JSON is parsed and validated

Concrete example observations from Docker runtime config examples:
- both `docker/configs/cluster/node-a/runtime.toml` and `docker/configs/single/node-a/runtime.toml` set `config_version = "v2"`
- both examples include every major section explicitly
- the examples set postgres TLS mode to `disabled`
- password-bearing role auth uses `{ path = "/run/secrets/..." }`
- both examples configure `logging.sinks.file.enabled = true`
- both examples set `api.security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }`
- both examples override `api.listen_addr` to `0.0.0.0:8080`, which is broader than the parser default

CLI interaction evidence from `cargo run --bin pgtuskmaster -- --help`:
- the daemon exposes `--config <PATH>`
- help text says it is the path to the runtime config file

Use this note as exhaustive factual support. Avoid inventing defaults or claiming a field is optional unless the parser or v2 input schema proves it.
