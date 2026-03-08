```markdown
# pgtuskmaster node binary

Binary name: `pgtuskmaster`

Entry point: `src/bin/pgtuskmaster.rs`

## Overview

The pgtuskmaster node binary parses command-line arguments, verifies configuration, bootstraps logging, plans and executes startup actions, then runs worker tasks until termination.

## Command surface

| Option | Type | Required | Description |
|--------|------|----------|-------------|
| `--config <PATH>` | `std::path::PathBuf` | Yes at runtime | Filesystem path to runtime configuration file. Absence causes exit code 2. |

No subcommands, environment variables, or additional flags are defined.

## Entry-point flow

`main()` → `run_node(cli)` → `ExitCode`

| Condition | Behavior |
|-----------|----------|
| Config absent | Writes `missing required \`--config <PATH>\`` to stderr; exits 2 |
| Config present | Builds Tokio multi-thread runtime with `worker_threads(4)`, `enable_all()`; blocks on `run_node_from_config_path(config.as_path())` |
| Runtime build failure | Writes `failed to build tokio runtime: {err}` to stderr; exits 1 |
| Runtime handoff failure | Writes error Display string to stderr; exits 1 |
| Runtime handoff success | Exits 0 |

## Runtime delegation

`run_node_from_config_path(path: &Path)` → `Result<(), RuntimeError>`

- Calls `load_runtime_config(path)`
- Delegates to `run_node_from_config(cfg)`

`run_node_from_config(cfg: RuntimeConfig)` → `Result<(), RuntimeError>`

Performs, in order:
- Configuration validation via `validate_runtime_config`
- Logging bootstrap via `logging::bootstrap`
- Startup event emission
- Derivation of process defaults via `process_defaults_from_config`
- Startup planning via `plan_startup`
- Startup execution via `execute_startup`
- Worker execution via `run_workers`

## Process dispatch defaults

`process_defaults_from_config(cfg: &RuntimeConfig)` → `ProcessDispatchDefaults`

Derived fields:

| Field | Source |
|-------|--------|
| `postgres_host` | `cfg.postgres.listen_host` |
| `postgres_port` | `cfg.postgres.listen_port` |
| `socket_dir` | `cfg.postgres.socket_dir` |
| `log_file` | `cfg.postgres.log_file` |
| `replicator_username` | `cfg.postgres.roles.replicator.username` |
| `replicator_auth` | `cfg.postgres.roles.replicator.auth` |
| `rewinder_username` | `cfg.postgres.roles.rewinder.username` |
| `rewinder_auth` | `cfg.postgres.roles.rewinder.auth` |
| `remote_dbname` | `cfg.postgres.rewind_conn_identity.dbname` |
| `remote_ssl_mode` | `cfg.postgres.rewind_conn_identity.ssl_mode` |
| `connect_timeout_s` | `cfg.postgres.connect_timeout_s` |
| `shutdown_mode` | `ShutdownMode::Fast` |

## Startup machinery

`plan_startup` → `StartupMode`

| Startup mode | Data dir state | DCS cache condition | Action sequence |
|--------------|----------------|---------------------|-----------------|
| `InitializePrimary` | Missing or empty; no init lock | No healthy primary in DCS | Claim init lock, bootstrap, start postgres as primary |
| `CloneReplica` | Missing or empty; init lock present | Healthy primary exists | Basebackup from primary, start postgres as replica |
| `ResumeExisting` | Existing with `PG_VERSION` | Ignored for selection | If `postmaster.pid` exists: none; else: start postgres with selected intent |

Startup actions are executed in order; each action emits events and aborts on first failure.

`execute_startup` → `Result<(), RuntimeError>`

- Ensures data dir parent, data dir (mode 0o700 on Unix), socket dir, and log dir exist
- Builds action list from startup mode
- Runs each action: claim init lock with optional config seeding, run bootstrap/basebackup job, or start postgres

Job execution:
- Spawns `ProcessJobKind`-specific command via `TokioCommandRunner`
- Polls output every 20 ms, draining up to 256 KiB
- Emits subprocess logs via `emit_startup_subprocess_line`
- Waits for exit or timeout (`cfg.process.bootstrap_timeout_ms`)
- On failure: drains remaining output, emits logs, returns error

## Worker setup boundary

`run_workers(cfg, process_defaults, log)` → `Result<(), RuntimeError>`

Initializes per-worker state channels (config, postgres, DCS, process, HA, debug snapshot) with `WorkerStatus::Starting`. Creates worker contexts:

| Worker | Key context fields |
|--------|--------------------|
| `pginfo` | Local postgres conninfo via unix socket; polls every `cfg.ha.loop_interval_ms` |
| `dcs` | `EtcdDcsStore` connection; publishes member records from postgres subscriber |
| `process` | `TokioCommandRunner`; accepts jobs via `mpsc::unbounded_channel` |
| `ha` | Receives all state subscribers; uses separate etcd store |
| `debug_api` | Snapshot builder; receives all subscribers |
| `api` | `TcpListener::bind(cfg.api.listen_addr)`; TLS via `build_rustls_server_config` |

API TLS setup sequence:

1. Build server config: `build_rustls_server_config(&cfg.api.security.tls)`
2. Configure TLS mode: `api_ctx.configure_tls(cfg.api.security.tls.mode, server_tls)`
3. Set client cert requirement: `cfg.api.security.tls.client_auth.require_client_cert` (default `false`)

API TLS errors map to:
- `RuntimeError::Worker("api tls config build failed: {err}")`
- `RuntimeError::Worker("api tls configure failed: {err}")`

Workers converge via `tokio::try_join!` on:
- `pginfo::worker::run(pg_ctx)`
- `dcs::worker::run(dcs_ctx)`
- `process::worker::run(process_ctx)`
- `logging::postgres_ingest::run(...)`
- `ha::worker::run(ha_ctx)`
- `debug_api::worker::run(debug_ctx)`
- `api::worker::run(api_ctx)`

First error cancels all tasks and returns `RuntimeError::Worker(err.to_string())`.

## Exit behavior

| Exit code | Source | Stderr output |
|-----------|--------|---------------|
| 0 | `run_node_from_config_path` returns `Ok(())` | none |
| 1 | Tokio runtime construction fails | `failed to build tokio runtime: {err}` |
| 1 | `run_node_from_config_path` returns `Err(err)` | `{err}` (Display) |
| 2 | `--config` absent | `missing required \`--config <PATH>\`` |

`RuntimeError` variants:
- `Config` – Configuration validation or loading error
- `StartupPlanning` – Data dir inspection or DCS cache probe failure
- `StartupExecution` – Logging bootstrap, path creation, job execution failure
- `ApiBind` – API listener bind failure
- `Worker` – Worker initialization or runtime failure
- `Time` – System time unavailable
```
