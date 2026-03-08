# pgtuskmaster Reference

Entry point for the `pgtuskmaster` node binary (`src/bin/pgtuskmaster.rs`).

## Command Surface

**Name:** `pgtuskmaster`  
**About:** Run a pgtuskmaster node

| Option | Type | Value name |
|---|---|---|
| `--config <PATH>` | `Option<PathBuf>` | `PATH` |

Runtime enforces `--config` presence; clap treats it as optional. No other flags, environment variables, or subcommands are defined by this binary.

## Entry-point Flow

| Step | Behavior |
|---|---|
| `main()` | Parses arguments with `Cli::parse()` and calls `run_node(cli)` |
| Config absent | Writes `missing required \`--config <PATH>\`` to stderr and exits `2` |
| Config present | Builds a Tokio runtime and blocks on `pgtuskmaster_rust::runtime::run_node_from_config_path(config.as_path())` |

Tokio runtime: `Builder::new_multi_thread().worker_threads(4).enable_all().build()`. Runtime construction failure writes `failed to build tokio runtime: {err}` to stderr and exits `1`.

## Runtime Delegation

`run_node_from_config_path(path)` loads configuration from `path` and delegates to `run_node_from_config(cfg)`, which:

- validates configuration
- bootstraps logging
- emits startup events
- derives process defaults
- plans and executes startup
- runs workers

`process_defaults_from_config(cfg)` derives:

- PostgreSQL listen host and port
- socket directory and log file path
- replicator username and auth
- rewinder username and auth
- remote database name
- remote SSL mode
- connection timeout seconds
- `ShutdownMode::Fast`

`run_workers(...)` initializes state channels for config, postgres, dcs, process, ha, and debug snapshot; connects etcd-backed DCS stores; builds worker contexts; configures API TLS; and enters `tokio::try_join!(...)` for the worker set.

API TLS setup sequence:

1. `build_rustls_server_config(&cfg.api.security.tls)`
2. `api_ctx.configure_tls(cfg.api.security.tls.mode, server_tls)`
3. derive `require_client_cert` from `cfg.api.security.tls.client_auth.require_client_cert`, defaulting to `false`
4. `api_ctx.set_require_client_cert(require_client_cert)`

API TLS errors map to:

- `RuntimeError::Worker("api tls config build failed: {err}")`
- `RuntimeError::Worker("api tls configure failed: {err}")`

## Exit Codes

| Code | Trigger | Stderr output |
|---|---|---|
| `0` | `run_node_from_config_path` returns `Ok(())` | none |
| `1` | Tokio runtime construction fails | `failed to build tokio runtime: {err}` |
| `1` | `run_node_from_config_path` returns `Err(err)` | `{err}` |
| `2` | `--config` absent | `missing required \`--config <PATH>\`` |

## Verified Behaviors

`tests/cli_binary.rs` verifies:

- `--help` includes `--config`
- missing `config_version = "v2"` exits `1` with stderr mentioning that hint
- missing `process.binaries` exits `1` with stderr mentioning that field path
- TLS auth under `postgres.roles.superuser.auth` exits `1` with stderr mentioning that field path
- invalid `postgres.local_conn_identity.ssl_mode` exits `1` with stderr mentioning that field path
