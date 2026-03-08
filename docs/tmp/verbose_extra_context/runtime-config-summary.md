# Verbose runtime configuration summary

This file exists to answer configuration overview requests using repository source only.

## Versioning and schema posture

- Runtime config loading happens in `src/config/parser.rs`.
- The file must contain `config_version`.
- Missing `config_version` is a validation error with a migration hint telling operators to set `config_version = "v2"`.
- `config_version = "v1"` is explicitly rejected.
- The supported schema is `config_version = "v2"`.

## Top-level config sections in the normalized runtime config

`RuntimeConfig` in `src/config/schema.rs` contains these required top-level sections after normalization:

- `cluster`
- `postgres`
- `dcs`
- `ha`
- `process`
- `logging`
- `api`
- `debug`

## Cluster section

`cluster` fields:

- `name: String`
- `member_id: String`

## Postgres section

`postgres` normalized fields:

- `data_dir: PathBuf`
- `connect_timeout_s: u32`
  - default from `src/config/defaults.rs`: `5`
- `listen_host: String`
- `listen_port: u16`
- `socket_dir: PathBuf`
- `log_file: PathBuf`
- `local_conn_identity`
  - required secure block in v2
  - fields: `user`, `dbname`, `ssl_mode`
- `rewind_conn_identity`
  - required secure block in v2
  - fields: `user`, `dbname`, `ssl_mode`
- `tls`
  - normalized from `TlsServerConfig`
  - mode values: `disabled`, `optional`, `required`
- `roles`
  - required secure block in v2
  - required roles: `superuser`, `replicator`, `rewinder`
  - each role has `username` plus auth
- `pg_hba`
  - required secure block in v2
- `pg_ident`
  - required secure block in v2
- `extra_gucs`
  - optional map of PostgreSQL config overrides

Role auth values:

- `tls`
- `password`
  - nested `password` uses `SecretSource`

`InlineOrPath` supports:

- bare path
- `{ path = "..." }`
- `{ content = "..." }`

## DCS section

`dcs` fields:

- `endpoints: Vec<String>`
- `scope: String`
- `init: Option<DcsInitConfig>`

Optional `dcs.init` fields:

- `payload_json: String`
- `write_on_bootstrap: bool`

## HA section

`ha` fields:

- `loop_interval_ms: u64`
- `lease_ttl_ms: u64`

## Process section

`process` fields:

- `pg_rewind_timeout_ms: u64`
  - default: `120000`
- `bootstrap_timeout_ms: u64`
  - default: `300000`
- `fencing_timeout_ms: u64`
  - default: `30000`
- `binaries`
  - required secure block in v2

`process.binaries` fields:

- `postgres`
- `pg_ctl`
- `pg_rewind`
- `initdb`
- `pg_basebackup`
- `psql`

Missing `process.binaries` or any individual binary path is a validation error reported with a stable field path such as `process.binaries`.

## Logging section

If omitted, logging is synthesized from safe defaults in `src/config/defaults.rs`.

Default logging values:

- `level = "info"`
- `capture_subprocess_output = true`
- `logging.postgres.enabled = true`
- `logging.postgres.poll_interval_ms = 200`
- `logging.postgres.cleanup.enabled = true`
- `logging.postgres.cleanup.max_files = 50`
- `logging.postgres.cleanup.max_age_seconds = 604800`
- `logging.postgres.cleanup.protect_recent_seconds = 300`
- `logging.sinks.stderr.enabled = true`
- `logging.sinks.file.enabled = false`
- `logging.sinks.file.path = None`
- `logging.sinks.file.mode = "append"`

## API section

`api` normalized fields:

- `listen_addr: String`
  - default: `127.0.0.1:8080`
- `security`

`api.security` fields:

- `tls`
- `auth`

`api.security.auth` values:

- `disabled`
- `role_tokens`
  - `read_token: Option<String>`
  - `admin_token: Option<String>`

## Debug section

`debug.enabled` exists and defaults to `false`.

## Example runtime file facts from docker config

`docker/configs/cluster/node-a/runtime.toml` shows a concrete v2 config with:

- `cluster.name = "docker-cluster"`
- `cluster.member_id = "node-a"`
- postgres listening on `node-a:5432`
- DCS endpoint `http://etcd:2379`
- `ha.loop_interval_ms = 1000`
- `ha.lease_ttl_ms = 10000`
- API listening on `0.0.0.0:8080`
- API auth disabled in that example
- debug enabled in that example

## Validation notes that are important for docs

- v2 is the only supported config version
- several secure blocks are mandatory in v2 and are not defaulted
- defaults are intentionally restricted to safe non-secret values
- parser errors aim to report stable field paths for missing secure values
