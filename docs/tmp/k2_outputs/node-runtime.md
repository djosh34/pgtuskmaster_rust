# Node Runtime Reference

The node runtime in `src/runtime/node.rs` loads runtime config, plans and executes startup, wires worker tasks, and binds the API listener for one member process.

## Public entrypoints and binary contract

The `pgtuskmaster` binary accepts a mandatory `--config <PATH>` argument and calls `run_node_from_config_path`.

| Entrypoint | Behavior |
|------------|----------|
| `run_node_from_config_path(path: &Path)` | Loads config with `load_runtime_config(path)?` and delegates to `run_node_from_config(cfg).await`. |
| `run_node_from_config(cfg: RuntimeConfig)` | Validates config, bootstraps logging, emits `runtime.startup.entered`, computes `ProcessDispatchDefaults`, plans startup, executes startup, and runs workers. |

## Startup planning

### Data directory classification

`inspect_data_dir` inspects `cfg.postgres.data_dir` and returns:

| Result | Condition |
|--------|-----------|
| `Missing` | Path is absent. |
| `Empty` | Path exists as directory and has no entries. |
| `Existing` | `PG_VERSION` exists in the directory. |
| `RuntimeError::StartupPlanning` | Path is not a directory, metadata or directory reads fail, or directory is non-empty without `PG_VERSION`. |

### DCS probe

`plan_startup` probes etcd before mode selection:

- Connects `EtcdDcsStore` to `cfg.dcs.endpoints` with scope `cfg.dcs.scope`.
- Drains watch events.
- Seeds empty `DcsCache` with `config: cfg.clone()`.
- Applies `refresh_from_etcd_watch` to populate members, leader, switchover, and init lock.
- Returns cache or logs warning and continues without cache on error.

`plan_startup_with_probe` emits `runtime.startup.dcs_cache_probe` with result `ok` or `failed`.

### Startup mode selection

`select_startup_mode` returns one of three modes based on data-dir state and DCS contents:

| Mode | Fields |
|------|--------|
| `InitializePrimary` | `start_intent: ManagedPostgresStartIntent` |
| `CloneReplica` | `leader_member_id: MemberId`, `source: ReplicatorSourceConn`, `start_intent: ManagedPostgresStartIntent` |
| `ResumeExisting` | `start_intent: ManagedPostgresStartIntent` |

Selection rules:

- `ResumeExisting` for `Existing` data-dir state.
- `CloneReplica` for `Missing` or `Empty` data-dir state when a foreign healthy primary is available from the leader key or, when init lock exists, from member records.
- `InitializePrimary` for `Missing` or `Empty` data-dir state when no leader evidence exists and no init lock is present.
- `RuntimeError::StartupPlanning` for `Missing` or `Empty` data-dir state when init lock is present but no healthy primary is available for basebackup.

### Resume intent reconstruction

`select_resume_start_intent` reads existing managed replica state with `postgres_managed::read_existing_replica_start_intent(data_dir)` and applies these rules:

- Returns primary intent when no managed replica state exists and no DCS cache is available.
- Returns `RuntimeError::StartupPlanning` when managed replica state exists but no authoritative DCS cache is available.
- Returns primary intent when the local member holds the leader key.
- Returns primary intent when a healthy local primary member exists.
- Returns replica intent rebuilt from a foreign healthy leader or foreign healthy primary member.
- Returns `RuntimeError::StartupPlanning` when managed replica state exists but no healthy primary is available in DCS.

## Startup execution

### Path preparation

`ensure_start_paths` creates directories and sets permissions:

| Path | Action |
|------|--------|
| Parent of `postgres.data_dir` | `create_dir_all` if present. |
| `postgres.data_dir` | `create_dir_all`, then Unix mode `0700` on Unix platforms. |
| `process_defaults.socket_dir` | `create_dir_all`. |
| Parent of PostgreSQL log file | `create_dir_all` if present. |

### Startup action matrix

`build_startup_actions` returns ordered actions per mode:

| Startup mode | Actions |
|--------------|---------|
| `InitializePrimary` | `ClaimInitLockAndSeedConfig`, `RunJob(Bootstrap)`, `StartPostgres` |
| `CloneReplica` | `RunJob(BaseBackup)`, `StartPostgres` |
| `ResumeExisting` with `postmaster.pid` present | none |
| `ResumeExisting` without `postmaster.pid` | `StartPostgres` |

### Action details

`ClaimInitLockAndSeedConfig`:

- Connects to etcd.
- Writes `/{scope}/init` with `put_path_if_absent`.
- Returns error if init lock already exists.
- Seeds `/{scope}/config` from `cfg.dcs.init.payload_json` when `write_on_bootstrap` is true.

`RunJob(job)`:

- Materializes managed PostgreSQL config with `postgres_managed::materialize_managed_postgres_config`.
- Builds command with `build_command`.
- Spawns with `TokioCommandRunner`.
- Drains subprocess output up to `STARTUP_OUTPUT_DRAIN_MAX_BYTES` per loop.
- Emits subprocess lines as `LogProducer::PgTool` with origin `startup`.
- Polls every `STARTUP_JOB_POLL_INTERVAL` (20 ms).
- Returns success on `ProcessExit::Success`; else returns `RuntimeError::StartupExecution`.

`StartPostgres`:

- Uses `ProcessJobKind::StartPostgres` with fields:
  - `data_dir: cfg.postgres.data_dir.clone()`
  - `config_file: managed postgresql.conf path`
  - `log_file: process_defaults.log_file.clone()`
  - `wait_seconds: Some(30)`
  - `timeout_ms: Some(cfg.process.bootstrap_timeout_ms)`

`execute_startup` emits events:

- `runtime.startup.actions_planned` before execution.
- `runtime.startup.action` with `started` before each action.
- `runtime.startup.action` with `ok` or `failed` after each action.
- `runtime.startup.phase` through target `startup` before `StartPostgres` action with fields `startup.phase: start` and `startup.detail: start postgres with managed config`.

## Worker startup and channel wiring

`run_workers` initializes state channels and spawns workers.

### Initial state values

| Channel | Initial value |
|---------|---------------|
| `pg` | `PgInfoState::Unknown` with worker `Starting`, `SqlStatus::Unknown`, `Readiness::Unknown`, no timeline, empty `PgConfig`, `last_refresh_at: None` |
| `dcs` | `DcsState` with worker `Starting`, `DcsTrust::NotTrusted`, empty `DcsCache` seeded with config clone, `last_refresh_at: None` |
| `process` | `ProcessState::Idle { worker: WorkerStatus::Starting, last_outcome: None }` |
| `ha` | `HaState { worker: WorkerStatus::Starting, phase: HaPhase::Init, tick: 0, decision: HaDecision::NoChange }` |
| `debug snapshot` | Snapshot built from `AppLifecycle::Running`, latest config/pg/dcs/process/ha snapshots, sequence `0`, empty change and timeline slices |

### Worker runtime sources

| Worker | Source |
|--------|--------|
| `pginfo::worker::run` | `cfg.ha.loop_interval_ms` |
| `dcs::worker::run` | `cfg.ha.loop_interval_ms` |
| `process::worker::run` | `PROCESS_WORKER_POLL_INTERVAL = 10 ms` |
| `logging::postgres_ingest::run` | `crate::logging::postgres_ingest::build_ctx(cfg.clone(), log.clone())` |
| `ha::worker::run` | `cfg.ha.loop_interval_ms` |
| `debug_api::worker::run` | `cfg.ha.loop_interval_ms` |
| `api::worker::run` | listener-driven |

### Additional wiring

- Creates separate `EtcdDcsStore` connections for DCS, HA, and API contexts.
- Binds `TcpListener` to `cfg.api.listen_addr`, returning `RuntimeError::ApiBind` on bind failure.
- Injects debug snapshot subscriber into `ApiWorkerCtx` with `set_ha_snapshot_subscriber`.
- Builds API TLS config from `cfg.api.security.tls`.
- Configures TLS mode on `ApiWorkerCtx`.
- Sets `require_client_cert` from `cfg.api.security.tls.client_auth.require_client_cert`, or `false` when client auth absent.
- Runs all workers with `tokio::try_join!`, mapping failure to `RuntimeError::Worker(err.to_string())`.

## Runtime error variants and constants

### RuntimeError

| Variant | Payload | Usage |
|---------|---------|-------|
| `Config` | `ConfigError` | Config validation or load failure. |
| `StartupPlanning` | `String` | Data-dir inspection, DCS probe, mode selection, or resume intent reconstruction failure. |
| `StartupExecution` | `String` | Path creation, init lock claim, command build, spawn, output drain, poll, or timeout failure. |
| `ApiBind` | `listen_addr: String`, `message: String` | TCP listener bind failure. |
| `Worker` | `String` | Any worker task panic or permanent error. |
| `Time` | `String` | System clock or millisecond-conversion failure in runtime time helpers. |

### Key constants

| Constant | Value |
|----------|-------|
| `STARTUP_OUTPUT_DRAIN_MAX_BYTES` | 262144 (256 * 1024) |
| `STARTUP_JOB_POLL_INTERVAL` | 20 ms |
| `PROCESS_WORKER_POLL_INTERVAL` | 10 ms |
| `STARTUP_POSTGRES_WAIT_SECONDS` | 30 |

### Helper surfaces

`process_defaults_from_config` builds `ProcessDispatchDefaults` from:

- `postgres_host: cfg.postgres.listen_host`
- `postgres_port: cfg.postgres.listen_port`
- `socket_dir: cfg.postgres.socket_dir`
- `log_file: cfg.postgres.log_file`
- `replicator_username/auth`, `rewinder_username/auth`
- `remote_dbname/ssl_mode` from `cfg.postgres.rewind_conn_identity`
- `connect_timeout_s: cfg.postgres.connect_timeout_s`
- `shutdown_mode: ShutdownMode::Fast`

`local_postgres_conninfo` builds local connection record with socket host, `process_defaults.postgres_port`, superuser username, `identity.dbname`, `identity.ssl_mode`, and `connect_timeout_s`.

`now_unix_millis` converts `SystemTime::now()` to `UnixMillis` and returns `RuntimeError::Time` for pre-epoch or conversion failures.
