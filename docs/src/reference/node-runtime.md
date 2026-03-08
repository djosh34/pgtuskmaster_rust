# Node Runtime Reference

The node runtime in `src/runtime/node.rs` loads runtime config, plans and executes startup, wires worker tasks, and binds the API listener for one member process.

## Entrypoints And Errors

| Function | Responsibility |
|---|---|
| `run_node_from_config_path(path: &Path)` | Loads config with `load_runtime_config(path)?` and delegates to `run_node_from_config(cfg).await`. |
| `run_node_from_config(cfg: RuntimeConfig)` | Validates config, bootstraps logging, emits `runtime.startup.entered`, computes `ProcessDispatchDefaults`, plans startup, executes startup, and runs workers. |

`RuntimeError` variants:

| Variant | Payload |
|---|---|
| `Config` | `ConfigError` |
| `StartupPlanning` | `String` |
| `StartupExecution` | `String` |
| `ApiBind` | `listen_addr: String`, `message: String` |
| `Worker` | `String` |
| `Time` | `String` |

`RuntimeError::Time` is used for system clock and millisecond-conversion failures in runtime time helpers.

## Startup Planning

### Data Directory Classification

`inspect_data_dir` returns:

| Result | Condition |
|---|---|
| `Missing` | The path is absent. |
| `Existing` | `PG_VERSION` exists in the directory. |
| `Empty` | The path exists as a directory and has no entries. |
| `RuntimeError::StartupPlanning` | The path is not a directory, metadata or directory reads fail, or the directory is non-empty without `PG_VERSION`. |

### DCS Probe Behavior

`plan_startup` delegates to `plan_startup_with_probe(..., probe_dcs_cache)`.

`probe_dcs_cache`:

- connects `EtcdDcsStore`
- drains watch events
- seeds an empty `DcsCache` with `config: cfg.clone()`
- applies `refresh_from_etcd_watch`
- returns the resulting cache

If the DCS probe fails, planning emits `runtime.startup.dcs_cache_probe` with a failed result and continues without a cache.

### Startup Modes

| Mode | Fields |
|---|---|
| `InitializePrimary` | `start_intent: ManagedPostgresStartIntent` |
| `CloneReplica` | `leader_member_id: MemberId`, `source: ReplicatorSourceConn`, `start_intent: ManagedPostgresStartIntent` |
| `ResumeExisting` | `start_intent: ManagedPostgresStartIntent` |

`select_startup_mode` returns:

- `ResumeExisting` for `Existing` data-dir state
- `CloneReplica` for `Missing` or `Empty` data-dir state when a foreign healthy primary is available from the leader key or, when `init_lock` exists, from member records
- `InitializePrimary` for `Missing` or `Empty` data-dir state when no leader evidence exists and no init lock is present
- `RuntimeError::StartupPlanning` for `Missing` or `Empty` data-dir state when init lock is present but no healthy primary is available for basebackup

`select_resume_start_intent`:

- reads existing managed replica state with `postgres_managed::read_existing_replica_start_intent(data_dir)`
- returns primary when no managed replica state exists and no DCS cache is available
- returns `RuntimeError::StartupPlanning` when managed replica state exists but no authoritative DCS cache is available
- returns primary when the local member holds the leader key
- returns primary when a healthy local primary member exists
- returns replica intent rebuilt from a foreign healthy leader or foreign healthy primary member
- returns `RuntimeError::StartupPlanning` when managed replica state exists but no healthy primary is available in DCS

## Startup Execution

### Startup Actions

| Startup mode | Ordered actions |
|---|---|
| `InitializePrimary` | `ClaimInitLockAndSeedConfig`, `RunJob(Bootstrap)`, `StartPostgres` |
| `CloneReplica` | `RunJob(BaseBackup)`, `StartPostgres` |
| `ResumeExisting` with `postmaster.pid` present | none |
| `ResumeExisting` without `postmaster.pid` | `StartPostgres` |

### Path Preparation And Action Behavior

`ensure_start_paths`:

- creates the parent directory of `postgres.data_dir` when present
- creates `postgres.data_dir`
- sets unix mode `0700` on `postgres.data_dir`
- creates the socket directory
- creates the parent directory of the PostgreSQL log file when present

`ClaimInitLockAndSeedConfig`:

- connects to etcd
- writes `/{scope}/init` with `put_path_if_absent`
- returns an error if the init lock already exists
- may seed `/{scope}/config` from `cfg.dcs.init.payload_json` when `write_on_bootstrap` is true

`run_start_job`:

- materializes managed PostgreSQL config
- runs `ProcessJobKind::StartPostgres` with:

| Field | Value |
|---|---|
| `data_dir` | `cfg.postgres.data_dir.clone()` |
| `config_file` | managed `postgresql.conf` path |
| `log_file` | `process_defaults.log_file.clone()` |
| `wait_seconds` | `Some(30)` |
| `timeout_ms` | `Some(cfg.process.bootstrap_timeout_ms)` |

`run_startup_job`:

- builds the command with `build_command`
- spawns with `TokioCommandRunner`
- drains subprocess output up to `256 * 1024` bytes per loop
- emits subprocess lines as `LogProducer::PgTool` with origin `startup`
- polls every `20 ms`
- returns success on `ProcessExit::Success`
- returns `RuntimeError::StartupExecution` on command-build failure, spawn failure, output-drain failure, poll failure, unsuccessful exit, or timeout cancellation failure

`execute_startup`:

- emits `runtime.startup.actions_planned`
- for each action emits `runtime.startup.action` with `started`, executes the action, then emits `runtime.startup.action` with `ok` or `failed`
- emits `runtime.startup.phase` through target `startup` immediately before a `StartPostgres` action with:

| Field | Value |
|---|---|
| `startup.phase` | `start` |
| `startup.detail` | `start postgres with managed config` |

## Worker Wiring And Initial State

`run_workers` creates state channels for config, pg, dcs, process, ha, and debug snapshot, plus a separate unbounded process inbox channel.

Initial state values:

| Channel | Initial value |
|---|---|
| `pg` | `PgInfoState::Unknown` with worker `Starting`, `SqlStatus::Unknown`, `Readiness::Unknown`, no timeline, empty `PgConfig`, and `last_refresh_at: None` |
| `dcs` | `DcsState` with worker `Starting`, `DcsTrust::NotTrusted`, empty `DcsCache` seeded with config clone, and `last_refresh_at: None` |
| `process` | `ProcessState::Idle { worker: WorkerStatus::Starting, last_outcome: None }` |
| `ha` | `HaState { worker: WorkerStatus::Starting, phase: HaPhase::Init, tick: 0, decision: HaDecision::NoChange }` |
| `debug snapshot` | Snapshot built from `AppLifecycle::Running`, the latest config/pg/dcs/process/ha snapshots, sequence `0`, and empty change and timeline slices |

Worker wiring:

| Worker | Runtime source |
|---|---|
| `pginfo::worker::run` | `cfg.ha.loop_interval_ms` |
| `dcs::worker::run` | `cfg.ha.loop_interval_ms` |
| `process::worker::run` | `PROCESS_WORKER_POLL_INTERVAL = 10 ms` |
| `logging::postgres_ingest::run` | `crate::logging::postgres_ingest::build_ctx(cfg.clone(), log.clone())` |
| `ha::worker::run` | `cfg.ha.loop_interval_ms` |
| `debug_api::worker::run` | `cfg.ha.loop_interval_ms` |
| `api::worker::run` | listener-driven |

`run_workers` also:

- creates separate `EtcdDcsStore` connections for DCS, HA, and API contexts
- binds `TcpListener` to `cfg.api.listen_addr`, returning `RuntimeError::ApiBind` on bind failure
- passes the debug snapshot subscriber into `ApiWorkerCtx` with `set_ha_snapshot_subscriber`
- builds API TLS config from `cfg.api.security.tls`
- configures TLS mode on `ApiWorkerCtx`
- sets `require_client_cert` from `cfg.api.security.tls.client_auth.require_client_cert`, or `false` when client auth is absent
- runs `pginfo`, `dcs`, `process`, PostgreSQL log ingest, `ha`, `debug_api`, and `api` workers with `tokio::try_join!`, mapping failure to `RuntimeError::Worker(err.to_string())`

## Local Helper Surfaces

`process_defaults_from_config(cfg)` builds `ProcessDispatchDefaults` from:

- PostgreSQL listen host and port
- socket directory
- PostgreSQL log file
- replicator and rewinder usernames and auth
- remote dbname and SSL mode
- connect timeout
- `ShutdownMode::Fast`

`local_postgres_conninfo` builds a local PostgreSQL connection record with:

| Field | Value |
|---|---|
| `host` | socket directory path |
| `port` | `process_defaults.postgres_port` |
| `user` | configured superuser username |
| `dbname` | `identity.dbname` |
| `application_name` | `None` |
| `connect_timeout_s` | `Some(connect_timeout_s)` |
| `ssl_mode` | `identity.ssl_mode` |
| `options` | `None` |

`initial_pg_state()` returns the unknown PostgreSQL snapshot described in the worker-initial-state table.

`now_unix_millis()` converts `SystemTime::now()` into `UnixMillis` and returns `RuntimeError::Time` for pre-epoch or millisecond-conversion failures.

## Verified Behaviors

Tests in `src/runtime/node.rs` verify:

- data-dir classification
- clone preference when a foreign healthy leader exists
- initialize when no leader evidence exists
- resume when `PGDATA` exists
- DCS authority overrides stale local replica auto-conf when rebuilding resume intent
- rejection of ambiguous partial data-dir state
- rejection when init lock exists without a healthy primary
- use of member records when init lock exists and the leader key is missing
- rejection of local replica state without DCS authority
- emission of startup data-dir, DCS probe, and mode-selection events
- role-specific local, clone, and rewind connection users
