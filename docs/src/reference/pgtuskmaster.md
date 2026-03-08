# pgtuskmaster Reference

pgtuskmaster node binary entry point.

## Command Surface

| Key | Value |
|---|---|
| Name | `pgtuskmaster` |
| About | `Run a pgtuskmaster node` |

| Option | Type | Value name |
|---|---|---|
| `--config <PATH>` | `Option<PathBuf>` | `PATH` |

The binary enforces `--config` presence at runtime; clap treats it as optional.

## Entry-point Flow

| Step | Behavior |
|---|---|
| `main()` | parses arguments with `Cli::parse()` and calls `run_node(cli)` |
| `run_node(cli)` with missing config | writes `missing required \`--config <PATH>\`` to stderr and exits `2` |
| `run_node(cli)` with config present | builds a Tokio runtime and blocks on `pgtuskmaster_rust::runtime::run_node_from_config_path(config.as_path())` |

Tokio runtime construction uses `Builder::new_multi_thread().worker_threads(4).enable_all().build()`.

Runtime-construction failure writes `failed to build tokio runtime: {err}` to stderr and exits `1`.

## Runtime Delegation Boundary

`run_node_from_config_path(path)` loads configuration from `path` and delegates to `run_node_from_config(cfg)`.

`run_node_from_config(cfg)` performs:

- configuration validation
- logging bootstrap
- startup event emission
- process defaults derivation
- startup planning and execution
- worker execution

`process_defaults_from_config(cfg)` derives:

- PostgreSQL listen host and port
- socket directory and log file path
- replicator and rewinder usernames and auth config
- remote database name and remote SSL mode
- connection timeout
- `ShutdownMode::Fast`

`run_workers(...)` initializes config, pg, dcs, process, ha, and debug-snapshot state channels, connects the etcd-backed DCS stores, builds each worker context, and then enters `tokio::try_join!(...)` for the worker set.

For API TLS setup, `run_workers(...)` applies this sequence:

1. `build_rustls_server_config(&cfg.api.security.tls)`
2. `api_ctx.configure_tls(cfg.api.security.tls.mode, server_tls)`
3. derive `require_client_cert` from `cfg.api.security.tls.client_auth.require_client_cert`, defaulting to `false`
4. `api_ctx.set_require_client_cert(require_client_cert)`

API TLS builder failures map to `RuntimeError::Worker("api tls config build failed: {err}")`.

API TLS configure failures map to `RuntimeError::Worker("api tls configure failed: {err}")`.

## Exit Behavior

| Code | Trigger | Stderr output |
|---|---|---|
| `0` | `run_node_from_config_path` returns `Ok(())` | none |
| `1` | Tokio runtime construction fails | `failed to build tokio runtime: {err}` |
| `1` | `run_node_from_config_path` returns `Err(err)` | `{err}` |
| `2` | `--config` is absent | `missing required \`--config <PATH>\`` |

## Verified Behaviors

Tests in `tests/cli_binary.rs` verify:

- `--help` succeeds with `--config` in stdout
- missing `config_version = "v2"` exits `1` with stderr mentioning that value
- missing `process.binaries` exits `1` with stderr mentioning that key
- `postgres.roles.superuser.auth` type `tls` exits `1` with stderr mentioning that key
- `postgres.local_conn_identity.ssl_mode=require` while `postgres.tls.mode=disabled` exits `1` with stderr mentioning that key
