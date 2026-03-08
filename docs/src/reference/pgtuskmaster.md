# pgtuskmaster Reference

Entry point for the `pgtuskmaster` node binary (`src/bin/pgtuskmaster.rs`).

## Command Surface

| Option | Value | Required | Description |
|---|---|---|---|
| `--config <PATH>` | filesystem path | Yes at runtime | Configuration file path |

The binary name is `pgtuskmaster`. The about text is `Run a pgtuskmaster node`. Clap parses `--config` as `Option<PathBuf>`, and the binary enforces its presence after parsing. No other flags, environment variables, or subcommands are defined by this binary.

## Entry-point Flow

| Step | Behavior |
|---|---|
| `main()` | Parses arguments with `Cli::parse()` and calls `run_node(cli)` |
| Config absent | Writes `missing required \`--config <PATH>\`` to stderr and exits `2` |
| Config present | Builds a Tokio multi-thread runtime with `worker_threads(4)` and `enable_all()`, then blocks on `pgtuskmaster_rust::runtime::run_node_from_config_path(config.as_path())` |
| Runtime build failure | Writes `failed to build tokio runtime: {err}` to stderr and exits `1` |
| Runtime handoff failure | Writes the returned error string to stderr and exits `1` |
| Runtime handoff success | Exits `0` |

## Runtime Delegation

`run_node_from_config_path(path)` loads configuration from `path` and delegates to `run_node_from_config(cfg)`.

`run_node_from_config(cfg)` performs these operations:

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

## Worker Setup

`run_workers(...)` initializes state channels for config, postgres, dcs, process, ha, and debug snapshot; connects etcd-backed DCS stores; builds worker contexts; configures API TLS; and enters `tokio::try_join!(...)` for the worker set.

API TLS setup sequence:

1. `build_rustls_server_config(&cfg.api.security.tls)`
2. `api_ctx.configure_tls(cfg.api.security.tls.mode, server_tls)`
3. derive `require_client_cert` from `cfg.api.security.tls.client_auth.require_client_cert`, defaulting to `false`
4. `api_ctx.set_require_client_cert(require_client_cert)`

API TLS errors map to:

- `RuntimeError::Worker("api tls config build failed: {err}")`
- `RuntimeError::Worker("api tls configure failed: {err}")`

## Exit Behavior

| Exit code | Condition | Stderr output |
|---|---|---|
| `0` | `run_node_from_config_path` returns `Ok(())` | none |
| `1` | Tokio runtime construction fails | `failed to build tokio runtime: {err}` |
| `1` | `run_node_from_config_path` returns `Err(err)` | `{err}` |
| `2` | `--config` absent | `missing required \`--config <PATH>\`` |

`tests/cli_binary.rs` exercises the command surface and validates that invalid runtime configuration reports exit code `1` with stable stderr hints for missing `config_version = "v2"`, missing `process.binaries`, unsupported `postgres.roles.superuser.auth`, and invalid `postgres.local_conn_identity.ssl_mode`.
