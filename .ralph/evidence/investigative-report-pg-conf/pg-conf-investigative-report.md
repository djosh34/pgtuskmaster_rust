# Investigative report: postgres conf and PGDATA mutation paths

Status: complete

## Step 1: direct production mutation nodes identified

### Node A: `ProcessRuntimePlan::ensure_start_paths`
Path: `src/process/state.rs`

Snippet:
```rust
if let Some(parent) = data_dir.parent() {
    fs::create_dir_all(parent)?;
}

fs::create_dir_all(data_dir)?;
fs::set_permissions(data_dir, fs::Permissions::from_mode(0o700))?;
fs::create_dir_all(&self.postgres.paths.socket_dir)?;

if let Some(log_parent) = self.postgres.paths.log_file.parent() {
    fs::create_dir_all(log_parent)?;
}
```

Why it matters:
- This is the earliest production `PGDATA` filesystem mutation.
- It creates the data-dir parent, the data dir itself, the socket dir, and the postgres log dir.
- It does not write postgres config contents, but it mutates the filesystem layout that later config materialization depends on.

### Node B: `wipe_data_dir`
Path: `src/process/worker.rs`

Snippet:
```rust
if data_dir.exists() {
    wipe_data_dir_contents(data_dir)?;
} else {
    fs::create_dir_all(data_dir)?;
}

#[cfg(unix)]
fs::set_permissions(data_dir, fs::Permissions::from_mode(0o700))?;
```

And inside `wipe_data_dir_contents`:
```rust
if file_type.is_dir() {
    fs::remove_dir_all(entry_path.as_path())?;
} else {
    fs::remove_file(entry_path.as_path())?;
}
```

Why it matters:
- This is the destructive `PGDATA` mutation node.
- It deletes every child entry under the configured postgres data dir and recreates/permissions the directory if needed.
- It is invoked before `Bootstrap` and before `ProvisionReplica(BaseBackup)`.

### Node C: `materialize_managed_postgres_config`
Path: `src/postgres_managed.rs`

Snippet:
```rust
write_atomic(&managed_hba, hba_contents.as_bytes(), Some(0o644))?;
write_atomic(&managed_ident, ident_contents.as_bytes(), Some(0o644))?;
...
write_atomic(&managed_postgresql_conf, rendered_conf.as_bytes(), Some(0o644))?;

quarantine_postgresql_auto_conf(&postgresql_auto_conf, &quarantined_postgresql_auto_conf)?;
materialize_recovery_signal_files(
    normalized_start_intent.recovery_signal(),
    &standby_signal,
    &recovery_signal,
)?;
```

Why it matters:
- This is the authoritative managed-config materialization node.
- It writes managed `pg_hba.conf`, `pg_ident.conf`, `pgtm.postgresql.conf`, standby passfile when needed, TLS runtime files when needed, quarantines `postgresql.auto.conf`, and creates/removes recovery signal files in `PGDATA`.

### Node D: helper mutations under `materialize_managed_postgres_config`
Path: `src/postgres_managed.rs`

Helper effects:
- `materialize_tls_files`: writes `pgtm.server.crt`, `pgtm.server.key`, and optional `pgtm.ca.crt`
- `materialize_managed_standby_passfile`: writes or removes `pgtm.standby.passfile`
- `materialize_recovery_signal_files`: creates/removes `standby.signal` and `recovery.signal`
- `quarantine_postgresql_auto_conf`: renames `postgresql.auto.conf` to `pgtm.unmanaged.postgresql.auto.conf` when present
- `write_atomic`: parent-dir creation, temporary-file write, chmod, and rename into final path

Current conclusion:
- The production mutation surface relevant to the PO request is concentrated in three code areas:
  - `src/process/state.rs`
  - `src/process/worker.rs`
  - `src/postgres_managed.rs`
- The next step is caller expansion: who reaches each node in production, then who reaches those callers, including tests.

## Step 2: caller expansion for each mutation node

### Node A caller tree: `ProcessRuntimePlan::ensure_start_paths`
Leaf mutation node:
- `src/process/state.rs`

Direct caller:
- `src/runtime/node.rs`

Snippet:
```rust
let process_plan = ProcessRuntimePlan::from_config(&cfg);
process_plan.ensure_start_paths()?;
run_workers(cfg, process_plan, log, worker).await
```

Production root chain:
1. `src/bin/pgtuskmaster.rs`
   - CLI root calls `pgtuskmaster_rust::runtime::run_node_from_config_path(config.as_path())`
2. `src/runtime/node.rs`
   - `run_node_from_config_path` loads TOML into `RuntimeConfig`
   - `run_node_from_config` validates config, builds `ProcessRuntimePlan`, then calls `ensure_start_paths`
3. `src/process/state.rs`
   - `ProcessRuntimePlan::ensure_start_paths` creates `PGDATA`, socket, and log directories

Observations:
- This path happens before workers start, so it is unconditional process bootstrap preparation for a node run.
- This is the only production caller of `ensure_start_paths`.

Tests that reach or validate this area:
- No direct unit test for `ensure_start_paths` was found in this sweep.
- Indirect runtime/process tests rely on `ProcessRuntimePlan::from_config` and later worker startup, but they do not call `ensure_start_paths` directly in the files inspected here.

### Node B caller tree: `wipe_data_dir`
Leaf mutation node:
- `src/process/worker.rs`

Direct caller:
- `src/process/worker.rs::materialize_execution_request`

Snippet:
```rust
match &request.intent {
    ProcessIntent::Bootstrap => {
        wipe_data_dir(runtime_config.postgres.paths.data_dir.as_path())?;
        ...
    }
    ProcessIntent::ProvisionReplica(ReplicaProvisionIntent::BaseBackup { leader }) => {
        wipe_data_dir(runtime_config.postgres.paths.data_dir.as_path())?;
        ...
    }
```

Next caller outward:
- `src/process/worker.rs::start_job`

Snippet:
```rust
let execution_request = match materialize_execution_request(ctx, &request) {
    Ok(materialized) => materialized,
    Err(error) => { ... }
};
```

Next caller outward:
- `src/process/worker.rs::step_once`

Snippet:
```rust
match ctx.control.inbox.try_recv() {
    Ok(request) => {
        start_job(ctx, request).await?;
    }
    ...
}
```

Next callers outward:
- `src/process/worker.rs::run`
- `src/process/startup.rs::ProcessWorker::run`
- `src/process/startup.rs::bootstrap`
- `src/runtime/node.rs::run_workers`

Production root chain:
1. `src/bin/pgtuskmaster.rs`
2. `src/runtime/node.rs::run_node_from_config_path`
3. `src/runtime/node.rs::run_node_from_config`
4. `src/runtime/node.rs::run_workers`
5. `src/process/startup.rs::bootstrap`
6. `src/process/worker.rs::run`
7. `src/process/worker.rs::step_once`
8. `src/process/worker.rs::start_job`
9. `src/process/worker.rs::materialize_execution_request`
10. `src/process/worker.rs::wipe_data_dir`

Who can enqueue the `Bootstrap` / `BaseBackup` intents that reach this node?
- Production HA path:
  1. `src/ha/worker.rs::step_once`
  2. `src/ha/decide.rs`
  3. `src/ha/reconcile.rs`
  4. `src/ha/worker.rs::execute_plan`
  5. `src/ha/worker.rs::execute_process_action`
  6. `src/ha/process_dispatch.rs::dispatch_process_action`
  7. `ProcessIntentRequest` sent into the process worker inbox

Intent-specific upstream production conditions:
- `Bootstrap`
  - `src/ha/reconcile.rs`
  - Emitted when target role is leader and local data dir is `Missing` or `BootstrapEmpty`
- `ProvisionReplica(BaseBackup)`
  - `src/ha/decide.rs::follow_goal` chooses `RecoveryPlan::Basebackup`
  - `src/ha/reconcile.rs::reconcile_follow_role` maps that to `ProcessIntent::ProvisionReplica(ReplicaProvisionIntent::BaseBackup { leader })`

Tests that reach this path:
- `src/logging/postgres_ingest.rs`
  - Sends `ProcessIntent::Bootstrap` through a process worker inbox
  - Sends `ProcessIntent::ProvisionReplica(ReplicaProvisionIntent::BaseBackup { leader })` through a process worker inbox
- These are process-worker integration style tests, not direct `wipe_data_dir` unit tests.

### Node C caller tree: `materialize_managed_postgres_config`
Leaf mutation node:
- `src/postgres_managed.rs`

Direct production caller:
- `src/process/worker.rs::materialize_start_postgres`

Snippet:
```rust
let managed =
    materialize_managed_postgres_config(runtime_config, start_intent)?;
Ok(ProcessExecutionKind::StartPostgres(StartPostgresSpec {
    config_file: managed.postgresql_conf_path,
    ...
}))
```

Direct callers of `materialize_start_postgres`:
- `src/process/worker.rs::materialize_execution_request`

Intent fan-in:
- `ProcessIntent::Start(PostgresStartIntent::Primary)`
- `ProcessIntent::Start(PostgresStartIntent::DetachedStandby)`
- `ProcessIntent::Start(PostgresStartIntent::Replica { leader })`

Important pre-step nodes before materialization:
- `primary_start_intent`
  - Rejects primary start if managed standby/recovery signal files are already present
- `replica_start_intent`
  - Resolves leader member from DCS
  - Builds leader conninfo via `basebackup_source_from_member`
  - Converts source auth into managed standby auth with a passfile path under `PGDATA`

Production root chain:
1. `src/bin/pgtuskmaster.rs`
2. `src/runtime/node.rs::run_node_from_config_path`
3. `src/runtime/node.rs::run_node_from_config`
4. `src/runtime/node.rs::run_workers`
5. `src/process/startup.rs::bootstrap`
6. `src/process/worker.rs::run`
7. `src/process/worker.rs::step_once`
8. `src/process/worker.rs::start_job`
9. `src/process/worker.rs::materialize_execution_request`
10. `src/process/worker.rs::materialize_start_postgres`
11. `src/postgres_managed.rs::materialize_managed_postgres_config`

Who can enqueue the start intents that reach this node?
- `Start Primary`
  - `src/ha/reconcile.rs`
  - Target role is leader, local data dir is initialized, local postgres is offline
- `Start Replica`
  - `src/ha/decide.rs::follow_goal` chooses `RecoveryPlan::StartStreaming`
  - `src/ha/reconcile.rs::reconcile_follow_role` maps that to `ProcessIntent::Start(PostgresStartIntent::Replica { leader })`
- `Start DetachedStandby`
  - `src/ha/decide.rs`
    - No usable leader yet, or peer leader is not primary-ready, or lease is open and this node is idle
  - `src/ha/reconcile.rs::reconcile_idle_role`
    - If postgres is offline and data dir exists, map idle role to `Start(PostgresStartIntent::DetachedStandby)`

Preflight caveat:
- `src/process/worker.rs::start_job` can short-circuit start intents before materialization.
- If `start_postgres_preflight_is_already_running` returns true, the worker reports success and does not call `materialize_execution_request`.
- This means not every `Start*` intent actually reaches `materialize_managed_postgres_config`.

Direct non-production callers/tests:
- `src/postgres_managed.rs`
  - Unit and real-binary tests call `materialize_managed_postgres_config` directly
- `src/dev_support/runtime_config.rs`
  - Builder test calls `materialize_managed_postgres_config` directly

### Step 2 interim conclusion
- There are two distinct top-level mutation paths:
  - runtime startup path for directory creation only
  - HA-driven process intent path for destructive `PGDATA` wipes and authoritative managed-config materialization
- The HA-driven path is the only production path that writes or removes postgres-managed config artifacts under `PGDATA`.

## Step 3: exhaustive source-input trace for managed postgres conf

### Render target
The final rendered file is `pgtm.postgresql.conf`, produced by:
- `src/postgres_managed.rs::materialize_managed_postgres_config`
- `src/postgres_managed_conf.rs::render_managed_postgres_conf`

The render function owns these setting families:
- fixed managed settings
  - `listen_addresses`
  - `port`
  - `unix_socket_directories`
  - `hba_file`
  - `ident_file`
  - `logging_collector = on`
  - `log_destination = 'jsonlog,stderr'`
- TLS settings
  - `ssl`
  - `ssl_cert_file`
  - `ssl_key_file`
  - optional `ssl_ca_file`
- role/startup settings
  - `hot_standby`
  - optional `primary_conninfo`
  - optional `primary_slot_name`
- user-supplied `postgres.extra_gucs`, after reserved-key validation

### Field-by-field input mapping into `ManagedPostgresConf`

#### `listen_addresses`
Source:
- `RuntimeConfig.postgres.network.listen_host`

Path:
1. `src/config/parser.rs`
   - `load_runtime_config` reads and validates TOML
2. `src/config/schema.rs`
   - `RuntimeConfig.postgres.network.listen_host`
3. `src/postgres_managed.rs`
   - copied directly into `ManagedPostgresConf.listen_addresses`
4. `src/postgres_managed_conf.rs`
   - rendered as `listen_addresses = '...'`

Merge/edit:
- no merge beyond direct copy

#### `port`
Source:
- `RuntimeConfig.postgres.network.listen_port`

Path:
1. config TOML
2. `RuntimeConfig.postgres.network.listen_port`
3. `ManagedPostgresConf.port`
4. rendered by `render_managed_postgres_conf`

Merge/edit:
- no merge beyond direct copy

#### `unix_socket_directories`
Source inputs:
- `RuntimeConfig.postgres.paths.socket_dir`, if configured
- otherwise `RuntimeConfig.process.working_root.join("socket")`

Path:
1. config TOML
2. `src/config/schema.rs::RuntimeConfig::postgres_socket_dir`
3. `src/postgres_managed.rs`
   - `ManagedPostgresConf.unix_socket_directories = cfg.postgres_socket_dir()`
4. rendered by `render_managed_postgres_conf`

Merge/edit:
- this is a two-source fallback merge:
  - explicit postgres socket dir wins
  - otherwise derived runtime working-root socket dir is used

#### `hba_file` and `ident_file`
Source inputs:
- `RuntimeConfig.postgres.paths.data_dir`
- fixed managed filenames:
  - `pgtm.pg_hba.conf`
  - `pgtm.pg_ident.conf`
- contents come from:
  - `RuntimeConfig.postgres.access.hba`
  - `RuntimeConfig.postgres.access.ident`

Path:
1. config TOML
2. `src/postgres_managed.rs`
   - joins `data_dir` with managed filenames
   - loads HBA/ident contents via `resolve_inline_or_path_string`
   - writes files with `write_atomic`
3. those written paths are then stored into `ManagedPostgresConf`
4. rendered by `render_managed_postgres_conf`

Merge/edit:
- path derivation: `data_dir + fixed managed filename`
- content materialization: inline-or-path config source is resolved into bytes/string before write
- render uses the managed file path, not the original input source path

#### TLS render fields: `ssl`, `ssl_cert_file`, `ssl_key_file`, optional `ssl_ca_file`
Source inputs:
- `RuntimeConfig.postgres.tls`
- if enabled:
  - `postgres.tls.identity.cert_chain`
  - `postgres.tls.identity.private_key`
  - optional `postgres.tls.client_auth.client_ca`
- destination filenames under `PGDATA`:
  - `pgtm.server.crt`
  - `pgtm.server.key`
  - optional `pgtm.ca.crt`

Path:
1. config TOML
2. `src/postgres_managed.rs::materialize_tls_files`
   - resolves inline/path bytes
   - writes managed TLS files under `PGDATA`
   - returns `ManagedPostgresTlsConfig`
3. `src/postgres_managed.rs`
   - inserts `tls_files.managed_tls_config` into `ManagedPostgresConf.tls`
4. `src/postgres_managed_conf.rs`
   - renders `ssl=off` when disabled
   - otherwise renders `ssl=on` plus managed file paths

Merge/edit:
- source bytes may come from inline TOML content or external file references
- runtime copies them into canonical managed filenames under `PGDATA`
- render stage references only the canonical managed filenames

#### `extra_gucs`
Source:
- `RuntimeConfig.postgres.extra_gucs`

Path:
1. config TOML
2. `src/postgres_managed.rs`
   - clones directly into `ManagedPostgresConf.extra_gucs`
3. `src/postgres_managed_conf.rs`
   - validates each key/value
   - rejects reserved keys that would override pgtuskmaster-owned settings
   - appends the remaining entries after all owned settings

Merge/edit:
- validation filter plus deterministic iteration order from `BTreeMap`
- no other transformation of user-provided values beyond quoting/escaping

### Start-intent source mapping

#### Primary start path
Source inputs:
- HA desired role becomes leader
- local data dir exists and postgres is offline
- local managed recovery-signal state must be `None`

Path:
1. `src/ha/decide.rs`
   - node ends up with `TargetRole::Leader(...)`
2. `src/ha/reconcile.rs`
   - leader + initialized data dir + offline postgres => `ProcessIntent::Start(PostgresStartIntent::Primary)`
3. `src/ha/process_dispatch.rs`
   - sends intent to process worker
4. `src/process/worker.rs::primary_start_intent`
   - inspects `standby.signal` / `recovery.signal` under local `PGDATA`
   - returns `ManagedPostgresStartIntent::Primary` if clean
5. `src/postgres_managed_conf.rs`
   - renders `hot_standby = off`
   - does not render `primary_conninfo`
   - recovery signal becomes `None`

Merge/edit:
- there is no DCS leader conninfo merged here
- local filesystem state is consulted to prevent rebuilding a primary config on top of managed replica markers

#### Detached standby start path
Source inputs:
- HA desired role becomes idle while data dir is initialized and postgres is offline

Path:
1. `src/ha/decide.rs`
   - examples: awaiting leader, leader not primary-ready, or lease open
2. `src/ha/reconcile.rs::reconcile_idle_role`
   - offline postgres + initialized data dir => `Start(DetachedStandby)`
3. process dispatch into worker
4. `src/process/worker.rs`
   - directly uses `ManagedPostgresStartIntent::detached_standby()`
5. `src/postgres_managed_conf.rs`
   - renders `hot_standby = on`
   - no `primary_conninfo`
   - recovery signal becomes `Standby`

Merge/edit:
- no leader endpoint is merged
- this path produces standby signal state without upstream conninfo

#### Replica start path
This is the most important merged path because it combines runtime config, DCS cluster view, and managed passfile state.

Source inputs:
- leader member id
- leader DCS member entry
- local process runtime plan
- local runtime config

Detailed path:
1. `src/ha/decide.rs::decide_under_foreign_leadership`
   - if peer leader state is `PrimaryReady`, desired role becomes follower with `follow_goal(world, epoch.holder)`
2. `src/ha/decide.rs::follow_goal`
   - chooses `RecoveryPlan::StartStreaming`, `Basebackup`, or `Rewind` based on:
     - local data-dir state
     - local postgres state
     - prior process failure recovery state
3. `src/ha/reconcile.rs::reconcile_follow_role`
   - `RecoveryPlan::StartStreaming` + local postgres offline => `ProcessIntent::Start(PostgresStartIntent::Replica { leader })`
4. `src/ha/process_dispatch.rs`
   - sends `ProcessIntentRequest` to process worker
5. `src/process/worker.rs::replica_start_intent`
   - calls `resolve_source_member(dcs, leader)`
   - calls `basebackup_source_from_member(&ctx.identity.member_id, &ctx.plan, source_member_id, source_member)`
   - converts returned source auth into `ManagedStandbyAuth::PasswordPassfile { path = data_dir/pgtm.standby.passfile }`
   - returns `ManagedPostgresStartIntent::Replica(source.conninfo, managed_passfile_auth, None)`
6. `src/postgres_managed.rs`
   - normalizes standby auth path
   - resolves replicator password from config secret source
   - writes passfile under `PGDATA`
   - builds `ManagedPostgresConf.start_intent`
7. `src/postgres_managed_conf.rs`
   - renders `hot_standby = on`
   - renders `primary_conninfo = '...'`
   - would render `primary_slot_name` if set, but current production path passes `None`

##### Where does the leader member entry come from?
1. `src/runtime/node.rs`
   - bootstraps DCS worker with `DcsAdvertisedEndpoints::from_config(&cfg)`
2. `src/dcs/startup.rs`
   - advertisement endpoint is built from:
     - `cfg.postgres.network.listen_host`
     - `cfg.postgres.network.advertise_port.unwrap_or(cfg.postgres.network.listen_port)`
3. `src/dcs/worker.rs`
   - `build_local_member_state(advertisement, pg_snapshot)`
   - serializes that state into etcd
4. `src/dcs/state.rs`
   - `DcsMemberState` contains:
     - `postgres_endpoint`
     - `postgres` (`PgInfoState`)
5. `src/process/worker.rs::resolve_source_member`
   - reads that `DcsView` quorum member back out on the follower side

##### What fields from DCS are actually used for replica conninfo?
Used by `src/process/source.rs::basebackup_source_from_member`:
- `member.postgres_target()`
  - becomes `PgConnInfo.endpoint`
- `member.postgres()`
  - only used as a health/type gate: must be `Primary { .. }`

Not used for rendered `primary_conninfo`:
- remote `primary_conninfo`
- remote `primary_slot_name`
- remote extra GUCs

##### What fields from local runtime config are merged into `primary_conninfo`?
Via `ProcessRuntimePlan::from_config` and `process/source.rs::remote_conninfo`:
- user
  - `cfg.postgres.roles.mandatory.replicator.username`
- auth secret
  - `cfg.postgres.roles.mandatory.replicator.auth`
- dbname
  - `cfg.postgres.rewind.database`
- ssl mode
  - `cfg.postgres.rewind.transport.ssl_mode`
- ssl root cert path
  - `cfg.postgres.rewind.transport.ca_cert`, but only when provided as a path source
- connect timeout
  - `cfg.postgres.connect_timeout_s`
- endpoint host/port
  - from remote leader DCS advertisement

##### Final merge into rendered `primary_conninfo`
1. `process/source.rs::remote_conninfo`
   - constructs `PgConnInfo` from leader endpoint + local replica-access settings
2. `postgres_managed.rs::materialize_managed_standby_passfile`
   - resolves local replicator password secret
   - writes `pgtm.standby.passfile`
3. `postgres_managed_conf.rs::render_managed_primary_conninfo`
   - renders libpq conninfo string
   - appends `passfile=<managed-passfile-path>`

Result:
- rendered `primary_conninfo` is a merged artifact built from:
  - remote leader endpoint from DCS
  - local configured replicator role name + auth
  - local configured rewind transport db/ssl/timeout settings
  - local managed passfile path under `PGDATA`

### Non-rendered but related `PGDATA` mutation inputs

#### `postgresql.auto.conf` quarantine
Source inputs:
- existence of `PGDATA/postgresql.auto.conf`
- fixed target path `PGDATA/pgtm.unmanaged.postgresql.auto.conf`

Path:
1. `materialize_managed_postgres_config`
2. `quarantine_postgresql_auto_conf`
3. rename old auto-conf to quarantined path, replacing older quarantined file if necessary

#### recovery signal files
Source input:
- `ManagedPostgresStartIntent.recovery_signal()`

Mapping:
- `Primary` => remove both signal files
- `DetachedStandby` and `Replica` => create `standby.signal`, remove `recovery.signal`
- `Recovery` => create `recovery.signal`, remove `standby.signal`

Exhaustive search note:
- current production code paths create `Primary`, `DetachedStandby`, and `Replica`
- `ManagedPostgresStartIntent::Recovery` appears in definitions and tests, but no production caller was found in the codebase search for this investigation

### Step 3 interim conclusion
- Managed postgres conf generation is not fed by a single source.
- It is a layered merge of:
  - runtime TOML config
  - derived runtime defaults (`socket_dir`, `log_file`)
  - current local filesystem state (`standby.signal`, `recovery.signal`, existing `postgresql.auto.conf`)
  - for replica starts only, DCS leader/member advertisement data plus local replica-access settings

## Step 4: tests and reachability notes

### Direct tests of `materialize_managed_postgres_config`
File:
- `src/postgres_managed.rs`

Direct coverage found:
- creates authoritative managed config
- uses managed config file path for startup
- creates/removes `standby.signal`
- creates/removes `recovery.signal`
- writes managed standby passfile
- removes stale standby passfile on primary start
- quarantines `postgresql.auto.conf`
- rejects reserved extra GUCs
- writes managed TLS runtime files
- real clone start quarantines auto-conf and stale signal
- inspects managed recovery state and conflicting signal files

Meaning:
- the low-level managed-config mutation engine is heavily tested in isolation

### Process-worker tests relevant to start/materialization reachability
Files:
- `src/process/worker.rs`
- `src/logging/postgres_ingest.rs`

Coverage found:
- `src/process/worker.rs`
  - `start_postgres_noop_preserves_existing_standby_passfile`
  - proves start preflight can short-circuit before config materialization
- `src/logging/postgres_ingest.rs`
  - sends `Bootstrap`
  - sends `Start(Primary)`
  - sends `ProvisionReplica(BaseBackup)`
  - exercises the process worker through inbox-driven requests

Meaning:
- there is direct evidence for both:
  - the normal worker path where intents are consumed from the inbox
  - the preflight short-circuit where a start intent does not reach config materialization

### HA reconciliation tests relevant to upstream intent selection
File:
- `src/ha/reconcile.rs`

Coverage found:
- detached-standby start while idle with initialized data dir
- missing data dir not starting detached standby
- follower restart to follow authoritative leader
- demoting-for-switchover transition behavior

Meaning:
- the tests cover key intent-selection states, but they do not themselves perform the final `PGDATA` writes; they validate the plan stage that feeds the process worker

## Final conclusions

### Production mutation sites
- Production `PGDATA` creation/setup happens in `ProcessRuntimePlan::ensure_start_paths`
- Production destructive `PGDATA` wiping happens only in `process/worker.rs::wipe_data_dir`
- Production managed postgres config and side-file authoring happens only in `postgres_managed.rs::materialize_managed_postgres_config` and its helpers

### Production roots
- Directory creation root:
  - `pgtuskmaster` CLI startup -> runtime node bootstrap
- Config and destructive `PGDATA` mutation root:
  - HA worker decision/reconcile -> process dispatch -> process worker job materialization

### Exhaustive live start-intent set
The production code paths traced in this report can currently materialize only:
- `Primary`
- `DetachedStandby`
- `Replica`

Not currently reached in production call paths found here:
- `ManagedPostgresStartIntent::Recovery`
- `primary_slot_name = Some(...)`

### Most important merge point
The densest merge for rendered config is replica startup:
- remote leader endpoint and primary-health proof come from DCS member state
- local role/auth/rewind transport settings come from runtime config
- local secret resolution writes the standby passfile under `PGDATA`
- the renderer then emits the final `primary_conninfo` string plus all local managed settings
