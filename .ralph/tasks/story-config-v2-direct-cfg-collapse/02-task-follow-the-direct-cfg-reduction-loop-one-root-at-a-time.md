## Task: Follow The Direct-`cfg` Reduction Loop One Root At A Time <status>done</status> <passes>true</passes>

<description>
**Goal:** Define the exact reduction loop that every config-v2 migration task in this story must follow. The higher-order goal is to only depend on config_v2 package and 0 on config package, and LARGE SCALE CODE REDUCTION/NESTED STRUCT COLLAPSE

1. find one root-level place that first uses `cfg`
2. change that root to `RuntimeConfigV2`
3. accept the compiler errors
4. inspect every field on every nested type reached from that root
5. if any nested field is really a config field in disguise, delete it from that type
6. fix the resulting errors by reading directly from `self.cfg`
7. when the package is valid again, QUIT IMMEDIATELY
8. next iteration continues deeper

This task is a playbook task for the rest of the story. It does not replace the package tasks; it defines the algorithm they must follow.


skill .agents/skills/improve-code-boundaries shows what we mean with code reduction


PO Message:

Great work till, now. However, do make sure:

- Only Duration fields remain, and never convert them back into ms unless the call made is to 3rd party. That means in the whole usage in our code, must stay Duration until it really can't be
- I see an concerning amount of code increase instead of decrease, really aim to eliminate Structs e.g. 'DcsWorkerInputs', since they are all not essential in the end. Really move out those non-config fields and flatten the structs and REMOVING all config fields in general and let them access cfg directly
- A concerning lack of removal of unneeded structs. Really work on deleting nested structs, flattening them, removing cfg values and refer to cfg directly

**Core rule from user discussion:**
- the daemon keeps one static shared `cfg: &RuntimeConfigV2`
- `pgtm` keeps one static shared `cfg: &OperatorConfigV2`
- `cfg` never changes dynamically
- `cfg` must not be cloned
- fields from `cfg` must not be cloned into nested structs just to pass them around
- each top-level context should contain:
  - `cfg`
  - runtime state that contains zero config-like fields whatsoever
- if a nested type contains a path, host, port, timeout, TLS/auth setting, endpoint list, binary path, working root, socket dir, log file, or similar static setting, it is presumptively wrong and must be collapsed
- even if a struct looks like it has no config fields at first glance, it must still be visited, because one or two layers deeper it may still contain cfg-derived fields
- the exploration must continue until there are verifiably zero fields left to explore in the surviving runtime struct graph
- when one iteration compiles again, QUIT IMMEDIATELY; the next iteration will discover the next config corridor

**Mandatory step plan:**
- find the first root-level use of `cfg` in the package
- change that root to `RuntimeConfigV2`, accept that this causes errors
- read the types of each field in that top-level struct
- for each field type:
  - inspect all of its fields
  - if one of those fields is itself a struct or enum carrying more fields, inspect those too
  - keep descending even if an intermediate struct appears innocent
  - judge whether each field is actually in `cfg`
  - if yes, remove it from the type
- fix compiler errors by making the callers use `self.cfg` directly
- if `cfg` is hidden under another field such as `self.inputs.cfg`, that is already wrong; collapse so the owner has `self.cfg`
- once the package is valid again, QUIT IMMEDIATELY
- the package is only truly done when every field in each surviving struct verifiably has zero config fields, the types reachable from those fields also have zero config fields, and there are zero further fields left to explore

**What this task must contain and enforce:**
- show discovery of nested types and their fields
- show the judgment for each nested field:
  - `cfg-derived, must be removed`
  - `runtime state, may stay`
- show three real iterations from this repo
- show the exact code that changes in those examples
- show how the compiler-error-driven reduction proceeds
- require the implementer to stop after each clean iteration and continue later, not “finish the whole package in theory”

**Example corridor 1 from research: DCS root and the `inputs` wrapper**

Current code:

```rust
// src/dcs/mod.rs
pub(crate) fn bootstrap(
    identity: NodeIdentity,
    cfg: &RuntimeConfig,
    pg_subscriber: StateSubscriber<PgInfoState>,
    log: LogSender,
) -> Result<(StateSubscriber<DcsSnapshot>, DcsHandle, worker::DcsWorker), worker::DcsError> {
    let (publisher, state) = new_state_channel(DcsSnapshot::starting());
    let (handle, command_inbox) = command::dcs_command_channel();
    let advertise_port = cfg
        .postgres
        .network
        .advertise_port
        .unwrap_or(cfg.postgres.network.listen_port);
    let advertised_postgres =
        PgEndpoint::tcp(cfg.postgres.network.listen_host.clone(), advertise_port)
            .map_err(worker::DcsError::Io)?;
    let worker = worker::DcsWorker::new(
        identity,
        cfg.dcs.endpoints.clone(),
        cfg.dcs.client.clone(),
        std::time::Duration::from_millis(cfg.ha.loop_interval_ms),
        cfg.ha.lease_ttl_ms,
        advertised_postgres,
        pg_subscriber,
        publisher,
        command_inbox,
        log,
    );
    Ok((state, handle, worker))
}
```

```rust
// src/dcs/worker.rs
pub(crate) struct DcsWorker {
    inputs: DcsWorkerInputs,
    cluster: DcsClusterState,
    session: Option<ConnectedSession>,
}

struct DcsWorkerInputs {
    identity: NodeIdentity,
    keys: DcsKeySpace,
    endpoints: Vec<DcsEndpoint>,
    client: DcsClientConfig,
    poll_interval: Duration,
    member_ttl_ms: u64, // Still wrong, because must be Duration
    advertised_postgres: PgEndpoint,
    pg: StateSubscriber<PgInfoState>,
    publisher: StatePublisher<DcsSnapshot>,
    command_inbox: DcsCommandInbox,
    log: LogSender,
}
```

**Discovery judgment for iteration 1:**
- `identity`: not in cfg, runtime identity, may stay
- `keys`: derived from identity scope, not a cfg field by itself, may stay for now
- `endpoints`: in cfg, must be removed
- `client`: in cfg, must be removed
- `poll_interval`: in cfg, must be removed
- `member_ttl_ms`: in cfg, must be removed
- `advertised_postgres`: derived only from cfg, must be removed
- `pg`: runtime subscriber, may stay
- `publisher`: runtime state, may stay
- `command_inbox`: runtime state, may stay
- `log`: runtime state, may stay

**Iteration 1 exact code change: replace exploded cfg arguments with one cfg root**

```rust
// src/dcs/mod.rs
pub(crate) fn bootstrap(
    identity: NodeIdentity,
    cfg: &RuntimeConfigV2,
    pg_subscriber: StateSubscriber<PgInfoState>,
    log: LogSender,
) -> Result<(StateSubscriber<DcsSnapshot>, DcsHandle, worker::DcsWorker), worker::DcsError> {
    let (publisher, state) = new_state_channel(DcsSnapshot::starting());
    let (handle, command_inbox) = command::dcs_command_channel();
    let worker = worker::DcsWorker::new(
        cfg,
        identity,
        pg_subscriber,
        publisher,
        command_inbox,
        log,
    );
    Ok((state, handle, worker))
}
```

```rust
// src/dcs/worker.rs
pub(crate) struct DcsWorker<'a> {
    cfg: &'a RuntimeConfigV2,
    inputs: DcsWorkerInputs,
    cluster: DcsClusterState,
    session: Option<ConnectedSession>,
}

struct DcsWorkerInputs {
    identity: NodeIdentity,
    keys: DcsKeySpace,
    pg: StateSubscriber<PgInfoState>,
    publisher: StatePublisher<DcsSnapshot>,
    command_inbox: DcsCommandInbox,
    log: LogSender,
}

impl<'a> DcsWorker<'a> {
    pub(crate) fn new(
        cfg: &'a RuntimeConfigV2,
        identity: NodeIdentity,
        pg: StateSubscriber<PgInfoState>,
        publisher: StatePublisher<DcsSnapshot>,
        command_inbox: DcsCommandInbox,
        log: LogSender,
    ) -> Self {
        Self {
            cfg,
            inputs: DcsWorkerInputs { // not needed, is in between state
                keys: DcsKeySpace::new(identity.scope.as_str()), // this is also bad and future work, they could just get it directly from cfg
                identity, // same here
                pg,
                publisher,
                command_inbox,
                log,
            },
            cluster: DcsClusterState::new(),
            session: None,
        }
    }
}
```

**Expected compiler errors after iteration 1:**
- `self.inputs.poll_interval` no longer exists
- `self.inputs.endpoints` no longer exists
- `self.inputs.client` no longer exists
- `self.inputs.member_ttl_ms` no longer exists
- `self.inputs.advertised_postgres` no longer exists

**Correct fix direction for iteration 1:**
- do not recreate those fields elsewhere
- replace all those callsites with direct `self.cfg...` reads

**Example corridor 2 from research: next collapse, remove the unnecessary `inputs` wrapper itself**

The user explicitly rejected this shape:

```rust
self.inputs.cfg
```

That means if iteration 1 introduces `cfg` but leaves it hidden behind another wrapper, iteration 2 must collapse the wrapper.

**Discovery judgment for iteration 2:**
- `inputs` is not a domain concept
- it only groups top-level worker-owned fields
- after config-like fields are removed, the wrapper buys nothing
- therefore `inputs` must be deleted and its surviving runtime fields must move to the worker itself

**Iteration 2 exact code change: remove `inputs`, promote survivors to `self.*`, and make all config reads use `self.cfg`**

```rust
// src/dcs/worker.rs
pub(crate) struct DcsWorker<'a> {
    cfg: &'a RuntimeConfigV2,
    identity: NodeIdentity, // same here, must refer to cfg
    keys: DcsKeySpace,// same here, must refer to cfg
    pg: StateSubscriber<PgInfoState>,
    publisher: StatePublisher<DcsSnapshot>,
    command_inbox: DcsCommandInbox,
    log: LogSender,
    cluster: DcsClusterState,
    session: Option<ConnectedSession>,
}

impl<'a> DcsWorker<'a> {
    pub(crate) fn new(
        cfg: &'a RuntimeConfigV2,
        identity: NodeIdentity,
        pg: StateSubscriber<PgInfoState>,
        publisher: StatePublisher<DcsSnapshot>,
        command_inbox: DcsCommandInbox,
        log: LogSender,
    ) -> Self {
        Self {
            cfg,
            keys: DcsKeySpace::new(identity.scope.as_str()),
            identity,
            pg,
            publisher,
            command_inbox,
            log,
            cluster: DcsClusterState::new(),
            session: None,
        }
    }

    async fn run(mut self) -> Result<(), WorkerError> {
        let mut reconnect_at = Instant::now();
        let mut tick = tokio::time::interval(self.cfg.timing.ha_loop_interval);
        tick.set_missed_tick_behavior(MissedTickBehavior::Delay);
        self.publish_current_view(false)?;
        // ...
    }
}
```

```rust
// examples of the follow-up replacements inside src/dcs/worker.rs
let pg_snapshot = self.pg.latest();

let endpoints = self
    .cfg
    .dcs
    .endpoints
    .iter()
    .map(|endpoint| endpoint.to_string())
    .collect::<Vec<_>>();

let advertised_postgres = PgEndpoint::tcp(
    self.cfg.postgres.listen_host.clone(),
    self.cfg.postgres.advertise_port,
)?;
```

**Expected compiler errors after iteration 2:**
- `self.inputs.pg` no longer exists
- `self.inputs.command_inbox` no longer exists
- `self.inputs.publisher` no longer exists
- `self.inputs.identity` no longer exists
- `self.inputs.keys` no longer exists

**Correct fix direction for iteration 2:**
- replace every `self.inputs.*` with the promoted `self.*`
- do not add another wrapper such as `runtime`, `ctx`, or `static`
- once the package compiles again, QUIT IMMEDIATELY

**Example corridor 3 from research: identity can also be part of the cfg corridor**

Iteration 2 must not stop at "identity is a runtime concept" and move on blindly. The field has to be opened. If the worker only uses `identity` to recover static membership/config values that already exist in `cfg`, then that part of `identity` is also inside the cfg corridor and must be collapsed.

Current code after iteration 2:

```rust
// src/dcs/worker.rs
pub(crate) struct DcsWorker<'a> {
    cfg: &'a RuntimeConfigV2,
    identity: NodeIdentity,
    keys: DcsKeySpace,
    pg: StateSubscriber<PgInfoState>,
    publisher: StatePublisher<DcsSnapshot>,
    command_inbox: DcsCommandInbox,
    log: LogSender,
    cluster: DcsClusterState,
    session: Option<ConnectedSession>,
}

impl<'a> DcsWorker<'a> {
    async fn publish_current_view(&self, connected: bool) -> Result<(), WorkerError> {
        let member = DcsMember::new(
            self.identity.member_id.clone(),
            self.cfg.postgres.listen_host.clone(),
            self.cfg.postgres.advertise_port, // very important to mention that advertise_port should already be checked to be correct value in RuntimeConfigV2. Not a single use of unwrap_or must stay, that is a bug and must be resolved by doing that logic inside config_v2
            connected,
        );
        self.publisher.publish(DcsSnapshot::from_member(member))?;
        Ok(())
    }
}
```

**Discovery judgment for iteration 3:**
- `identity`: not automatically runtime state; it must be opened and judged by field
- `identity.scope`: if only used to build static DCS paths/namespaces already derivable from cfg, cfg-derived, must be removed from this corridor
- `identity.member_id`: if only used as the configured member name/identifier already present in cfg, cfg-derived, must be removed from this corridor
- `keys`: if it exists only because it was built from `identity.scope`, cfg-derived, must be removed
- if some surviving identity field is truly runtime-discovered and not in cfg, that field may stay, but the judgment must be explicit
- `pg`: runtime subscriber, may stay
- `publisher`: runtime state, may stay
- `command_inbox`: runtime state, may stay
- `log`: runtime state, may stay

**Iteration 3 exact code change: stop re-carrying cfg-backed identity fields through the worker**

```rust
// src/dcs/worker.rs
pub(crate) struct DcsWorker<'a> {
    cfg: &'a RuntimeConfigV2,
    pg: StateSubscriber<PgInfoState>,
    publisher: StatePublisher<DcsSnapshot>,
    command_inbox: DcsCommandInbox,
    log: LogSender,
    cluster: DcsClusterState,
    session: Option<ConnectedSession>,
}

impl<'a> DcsWorker<'a> {
    pub(crate) fn new(
        cfg: &'a RuntimeConfigV2,
        pg: StateSubscriber<PgInfoState>,
        publisher: StatePublisher<DcsSnapshot>,
        command_inbox: DcsCommandInbox,
        log: LogSender,
    ) -> Self {
        Self {
            cfg,
            pg,
            publisher,
            command_inbox,
            log,
            cluster: DcsClusterState::new(),
            session: None,
        }
    }

    async fn publish_current_view(&self, connected: bool) -> Result<(), WorkerError> {
        let member = DcsMember::new(
            self.cfg.identity.member_id.clone(), // still using self.cfg, so therefore this task is NOT done, but this will be fixed in a future iteration of this task
            self.cfg.postgres.listen_host.clone(),
            self.cfg.postgres.advertise_port,
            connected,
        );
        self.publisher.publish(DcsSnapshot::from_member(member))?;
        Ok(())
    }
}
```

**Expected compiler errors after iteration 3:**
- `self.identity.*` no longer exists
- `self.keys` no longer exists
- constructor callsites still pass `identity`
- any helper that still rebuilds DCS paths from `identity.scope`

**Correct fix direction for iteration 3:**
- replace `self.identity.*` reads with direct `self.cfg.identity.*` reads
- replace `self.keys` derivations with direct key construction from `self.cfg`
- do not keep `identity` around as a disguised cfg cache
- only preserve identity fields that are proven to be runtime state and not representable in cfg
- once the package compiles again, QUIT IMMEDIATELY

**Example corridor 4 from research: process nested mirror chain**

Current code:

```rust
// src/process/state.rs
pub(crate) struct ManagedPostgresPaths {
    pub(crate) data_dir: PathBuf,
    pub(crate) socket_dir: PathBuf,
    pub(crate) log_file: PathBuf,
}

pub(crate) struct ManagedPostgresRuntime {
    pub(crate) paths: ManagedPostgresPaths,
    pub(crate) port: u16,
}

pub(crate) struct ReplicaAccessRuntime {
    pub(crate) roles: MandatoryPostgresRuntimeRoles,
    pub(crate) dbname: String,
    pub(crate) ssl_mode: PgSslMode,
    pub(crate) ssl_root_cert: Option<PathBuf>,
    pub(crate) connect_timeout_s: u32,
}

pub(crate) struct ProcessRuntimePlan {
    pub(crate) postgres: ManagedPostgresRuntime,
    pub(crate) replica_access: ReplicaAccessRuntime,
}
```

```rust
// src/process/planner.rs
pub(crate) fn plan(
    &self,
    identity: &NodeIdentity,
    runtime: &ProcessRuntimePlan,
    observed: &ProcessObservedSnapshot,
    intent: &ProcessIntent,
) -> Result<ClusterProcessPlan, ProcessError> {
    match intent {
        ProcessIntent::Bootstrap => Ok(ClusterProcessPlan::Bootstrap(BootstrapSpec {
            data_dir: observed.runtime_config.postgres.paths.data_dir.clone(),
            superuser: observed
                .runtime_config
                .postgres
                .roles
                .mandatory
                .superuser
                .username
                .clone(),
            timeout_ms: None,
        })),
        // ...
    }
}
```

```rust
// src/process/session.rs
pub(crate) fn materialize(
    &self,
    runtime_config: &RuntimeConfig,
    runtime: &ProcessRuntimePlan,
    plan: &ClusterProcessPlan,
) -> Result<Option<PreparedManagedPostgresSession>, ProcessError> {
    runtime.ensure_start_paths()?;
    let config = materialize_managed_postgres_config(runtime_config, &start_intent)?;
    // ...
}
```

**Discovery judgment for iteration 4:**
- `ManagedPostgresPaths.data_dir`: in cfg, must be removed
- `ManagedPostgresPaths.socket_dir`: in cfg, must be removed
- `ManagedPostgresPaths.log_file`: in cfg, must be removed
- `ManagedPostgresRuntime.port`: in cfg, must be removed
- `ReplicaAccessRuntime.roles`: in cfg, must be removed
- `ReplicaAccessRuntime.dbname`: in cfg, must be removed
- `ReplicaAccessRuntime.ssl_mode`: in cfg, must be removed
- `ReplicaAccessRuntime.ssl_root_cert`: in cfg, must be removed
- `ReplicaAccessRuntime.connect_timeout_s`: in cfg, must be removed
- therefore `ManagedPostgresPaths`, `ManagedPostgresRuntime`, `ReplicaAccessRuntime`, and `ProcessRuntimePlan` are entirely config mirrors and must be deleted
- `ProcessObservedSnapshot.runtime_config`: whole cfg carried through another corridor, must be deleted
- even after those deletions, continue exploring surviving structs and spec types, because a “clean-looking” wrapper may still hide cfg-derived fields one layer deeper

**Iteration 4 exact code change: delete the mirror structs and fix callsites to use `ctx.cfg` directly**

```rust
// src/process/state.rs
pub(crate) struct ProcessWorkerCtx<'a> {
    pub(crate) cfg: &'a RuntimeConfigV2,
    pub(crate) identity: NodeIdentity,
    pub(crate) observed: ProcessObservedState,
    pub(crate) state_channel: ProcessStateChannel,
    pub(crate) control: ProcessControlPlane,
    pub(crate) runtime: ProcessRuntime,
    pub(crate) now: Box<dyn FnMut() -> Result<UnixMillis, WorkerError> + Send>,
}

#[derive(Clone, Debug)]
pub(crate) struct ProcessObservedState {
    pub(crate) dcs: StateSubscriber<DcsSnapshot>,
}

#[derive(Clone, Debug)]
pub(crate) struct ProcessObservedSnapshot {
    pub(crate) dcs: DcsSnapshot,
    pub(crate) managed_recovery_state: ManagedRecoverySignal,
}
```

```rust
// src/process/planner.rs
pub(crate) fn plan(
    &self,
    cfg: &RuntimeConfigV2,
    identity: &NodeIdentity, // still not done, but fixed in next
    observed: &ProcessObservedSnapshot,
    intent: &ProcessIntent,
) -> Result<ClusterProcessPlan, ProcessError> {
    match intent {
        ProcessIntent::Bootstrap => Ok(ClusterProcessPlan::Bootstrap(BootstrapSpec {
            superuser: cfg.postgres.superuser.username.clone(),
        })),
        ProcessIntent::ProvisionReplica(ReplicaProvisionIntent::BaseBackup { leader }) => {
            let source = basebackup_source_from_leader(cfg, &identity.member_id, &observed.dcs, leader)?;
            Ok(ClusterProcessPlan::BaseBackup(BaseBackupSpec { source }))
        }
        // ...
    }
}
```

```rust
// src/process/session.rs
pub(crate) fn materialize(
    &self,
    cfg: &RuntimeConfigV2,
    plan: &ClusterProcessPlan,
) -> Result<Option<PreparedManagedPostgresSession>, ProcessError> {
    ensure_start_paths(cfg)?;
    let config = materialize_managed_postgres_config(cfg, &start_intent)?;
    Ok(Some(PreparedManagedPostgresSession { config }))
}
```

**Expected compiler errors after iteration 4:**
- every callsite using `ctx.plan.*`
- every callsite using `observed.runtime_config.*`
- every job/spec carrying now-deleted path/timeout/port fields

**Correct fix direction for iteration 4:**
- delete the deleted-field reads
- replace them with direct `ctx.cfg...` or `cfg...`
- keep only dynamic intent-specific state in the plan/spec layer
- if a job/spec field is still static config, remove that field too
- keep descending into surviving types until there are zero fields left to explore
- once the package compiles again, QUIT IMMEDIATELY

**Reduction-loop proof requirement:**
- for each iteration, the implementer must list the inspected nested types and fields
- for each field, mark whether it is:
  - cfg-derived and removed
  - runtime state and kept
- if a surviving type appears clean, it must still be opened and its nested field types inspected until the exploration graph is exhausted
- if any surviving type still contains config-like fields after the iteration, or if there are still unexplored nested fields reachable from the root, the task is not done; it is merely paused at the next clean boundary

**Expected outcome:**
- implementers have one exact aggressive loop to follow
- package tasks stop inventing new config-mirror structs
- the migration advances by compiler-error-driven deletion and direct `self.cfg` access

</description>

<acceptance_criteria>
- [ ] The step plan in this task is followed by the package tasks in this story
- [ ] The task shows four real iterations from this repo, including the DCS `inputs` collapse, the identity-in-cfg collapse, and a deeper process mirror collapse
- [ ] The task explicitly requires judging nested fields as cfg-derived vs runtime-state
- [ ] The task explicitly requires `self.cfg`, not `self.inputs.cfg` or another wrapper
- [ ] The task explicitly requires visiting structs that initially look clean, and continuing until there are zero further fields left to explore
- [ ] The task explicitly requires stopping after each clean iteration with `QUIT IMMEDIATELY`
- [ ] The task explicitly defines done as recursive absence of config-like fields from surviving runtime structs
- [ ] As long as you still find any structs to be constructed where one of the fields uses/clones some field from cfg, you are not done with the task
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

## Current tree findings for next iteration (2026-03-20 16:49)

- Flattening `RuntimeConfigV2.node: NodeIdentity` is still the correct first type correction and must remain step zero.
- The real root is still `src/runtime/node.rs`, not `src/logging/core/runtime.rs::bootstrap` in isolation. That runtime root already owns config-v2 and immediately fans it into logging, pginfo, dcs, process, ha, api, and postgres-ingest.
- The previous handoff was still too permissive about config subscribers. The task's own core rule says cfg is static, shared once, never updated dynamically, and must not be cloned into runtime state. Therefore `new_state_channel(cfg.clone())` and every `StateSubscriber<RuntimeConfig>` / `StateSubscriber<RuntimeConfigV2>` corridor are design mistakes, not valid intermediate targets.
- This is the combined smell from improve-code-boundaries:
  - wrong config-ingestion boundary: old `crate::config::*` shapes are still escaping past config-v2
  - wrong bootstrap boundary: `runtime/node.rs` is hand-carrying config mirrors and module-private startup details
- Do not "convert the subscriber to config-v2". Delete the config subscriber corridor itself. If a long-lived worker needs config, that worker root must hold the single shared cfg reference, not a cloned channel state.
- The one place where lifetime pressure is expected is the API server/router state. If a plain stack borrow is not sufficient there, make the daemon root own one long-lived shared cfg reference and pass that exact reference everywhere. Do not reintroduce clones, `Arc<RuntimeConfigV2>`, or subscriber wrappers as a workaround.
- The first execution patch has now been applied and compiled far enough to prove the root plan is directionally right: `RuntimeConfigV2.node` was flattened, `runtime/node.rs` stopped creating a config subscriber, and `logging::bootstrap` moved to config-v2. The next blockers are not at the root anymore; they are the nested legacy helper/type corridors that were still under-specified below.

## Discovery judgment for the real next root

### Type-first correction: `RuntimeConfigV2`

- `node`: cfg-derived wrapper, must be removed
- `cluster_name`: static config field, may stay directly on `RuntimeConfigV2`
- `scope`: static config field, may stay directly on `RuntimeConfigV2`
- `member_id`: static config field, may stay directly on `RuntimeConfigV2`

### Root corridor: `src/runtime/node.rs` immediate startup graph

- `logging::bootstrap(cfg)`: direct config read, must move to `RuntimeConfigV2`
- `ProcessRuntimePlan::from_config(cfg)`: cfg-derived runtime mirror root, must move to config-v2 and stay under active reduction
- `pginfo::startup::bootstrap(identity, cfg, ...)`: direct static config read, must move to `RuntimeConfigV2`, and pginfo must not stay coupled to `ProcessRuntimePlan`
- `dcs::bootstrap(identity, cfg, ...)`: direct static config read, must move to `RuntimeConfigV2`
- `process::startup::bootstrap(identity, cfg, observed, ...)`: root bootstrap boundary, must move to `RuntimeConfigV2`
- `ha::startup::bootstrap(...)`: must stop receiving `HaObservedState.config`; HA should read the shared cfg directly from its own root context
- `api::startup::bootstrap(...)`: must stop receiving `runtime_config` subscribers; API should read the shared cfg directly from its own root context
- `logging::postgres_ingest::{build_ctx, PostgresIngestWorkerCtx}`: must stop owning a cloned config object and read from the shared cfg directly
- `new_state_channel(cfg.clone())`: cfg-derived dynamic wrapper, must be removed entirely

### Shared cfg lifetime strategy for the next execution pass

- `run_workers` must keep owning the single `RuntimeConfigV2` value for the daemon run and pass `&cfg` into each startup/bootstrap call in this corridor.
- The touched worker/context roots in this pass must become lifetime-parameterized and store `cfg: &'a RuntimeConfigV2` directly instead of cloning config, wrapping it in `Arc`, or hiding it behind subscribers.
- One local `NodeIdentity` value may still be constructed once in `runtime/node.rs` from `cfg.cluster_name`, `cfg.scope`, and `cfg.member_id` for callsites that still truly require identity as a runtime edge type during this iteration.
- That local `NodeIdentity` is not permission to keep identity embedded inside `RuntimeConfigV2`, to rebuild `RuntimeConfig`, or to pass identity/config through channels or mirror structs.

### Nested types already proven dirty

- `ProcessRuntimePlan` is not a clean runtime type yet; it clones paths, port, usernames, auth, ssl mode, root cert path, and timeout values directly from cfg. Those are cfg-derived and must remain under active inspection during the next execution slice.
- `ProcessRuntimePlan` currently has no legitimate runtime-owned fields at all; it is a startup-wide cfg snapshot. Do not preserve it under a slimmer shape just to keep `from_config` alive. Move path preparation and other static reads onto direct config-v2 access, and delete the type entirely if no true runtime state remains.
- `ProcessWorkerCtx.config: ProcessConfig` is an old-config subtree stored in runtime state. It must be removed, not translated into a new process-config mirror. Read binaries and timeouts from `ctx.cfg.binaries` and `ctx.cfg.timing` instead.
- `ProcessRuntime.capture_subprocess_output: bool` is cfg-derived runtime storage and must be removed. Read it from `ctx.cfg.logging.capture_subprocess_output`.
- `ProcessObservedState.runtime_config` and `ProcessObservedSnapshot.runtime_config` are config mirrors in runtime state and must not survive the process corridor.
- `ProcessCluster::prepare(config, capture_output)`, `ExternalToolLowerer::build_command(config, ...)`, `timeout_for_kind(..., config)`, and `resolve_process_binary(config, PostgresBinaryName)` are the same wrong boundary expressed as helper parameters. Once `ProcessWorkerCtx` owns `cfg: &RuntimeConfigV2`, those helpers must stop accepting `ProcessConfig`/`PostgresBinaryName`-driven config leaves and read `ctx.cfg.binaries`, `ctx.cfg.timing`, and `ctx.cfg.logging.capture_subprocess_output` directly or through a direct config-v2 borrow.
- `PgInfoWorkerCtx.probe_conninfo` is a config mirror chain: socket dir, port, superuser name, dbname, tls mode, and other connection fields all come from cfg or cfg-derived defaults. It must be removed, and pginfo should read those fields from its shared cfg root instead of from `ProcessRuntimePlan`.
- `PgInfoCadence.poll_interval` is cfg-derived static timing and must be removed. Pginfo should read `ctx.cfg.timing.ha_loop_interval` directly.
- `HaObservedState.config` is a fake dynamic config observer and must be removed. `ha::worker::run` must stop selecting on config changes, and HA logic must read lease/data-dir/static settings from its shared cfg root.
- `HaWorkerCadence.poll_interval` is cfg-derived static timing and must be removed. HA should read `ctx.cfg.timing.ha_loop_interval` directly.
- `ApiRuntimeCtx.runtime_config` is a cfg-derived subscriber boundary and must be removed. Resolve auth, bind, transport, and certificate reload behavior from config-v2 without rebuilding old config types.
- `ApiRuntimeCtx.bind` and `ApiRuntimeCtx.auth` are also cfg-derived mirrors once `cfg` is available on the API root. They must not survive as duplicated static state.
- `ApiReloadCertificatesHandle` and API transport/auth helpers currently read old config shapes. When moved to config-v2, delete adapters rather than rebuilding `crate::config::*` values.
- `PostgresIngestWorkerCtx.cfg` is a cloned legacy config root and must be removed. `PostgresIngestWorkerState::new` and the ingest loop must read config-v2 logging fields directly.
- `dcs::worker::DcsWorkerInputs` is already proven dirty by the earlier example corridor. Once `dcs::bootstrap` flips to config-v2/shared-cfg, do not keep endpoints/client/poll/member-ttl/advertised-postgres mirrored on the worker.
- `process::jobs::{BootstrapSpec.superuser, MandatoryRoleSourceConn.auth, ProcessEnvValue::Secret}` are still carrying old `crate::config` types (`PostgresRoleName`, `RoleAuthConfig`, `SecretSource`). Those are not runtime state; they are static config/auth material and must be collapsed to config-v2 primitives (`String`, `Secret`, and direct cfg reads) rather than translated through old config DTOs.
- `postgres_managed::materialize_managed_postgres_config` and its TLS/auth helpers still accept `crate::config::RuntimeConfig` / `TlsServerConfig` / `RoleAuthConfig`-shaped inputs. The managed-postgres materialization boundary must move to config-v2 directly; do not rebuild old config to feed it.
- `tls::{build_api_server_transport, build_api_server_config}` still depend on old `ApiTransportConfig` / `ApiTlsConfig`. Those helpers must move to `config_v2::types::{ApiTransport, TlsConfig}` directly.
- `dcs::worker::EtcdRuntime::connect_options` still depends on old `DcsClientConfig`, `DcsAuthConfig`, `DcsTlsConfig`, and inline-or-path resolvers. Once the worker owns `&RuntimeConfigV2`, this helper must read `cfg.dcs` directly and use config-v2 `Secret`/`TlsConfig` values instead.
- `logging::postgres_ingest::cleanup_log_dir` still takes an old `LogCleanupConfig` subtree. Once ingest owns `&RuntimeConfigV2`, cleanup must read the v2 logging retention fields directly or through a small v2-local shape, not through `crate::config`.
- Freshly proven design gap from execution: `RuntimeConfigV2` currently discarded static config that the remaining process corridor still legitimately needs. `postgres_roles::reconcile_managed_roles` needs `postgres.local_database`, and the managed-postgres materialization boundary still needs the authoritative `postgres.access.hba` / `postgres.access.ident` contents. Those values existed in the raw/private schema or legacy config, but not on `config_v2::types::RuntimeConfigV2`, so the direct-cfg reduction could not finish correctly until the v2 type design was corrected first.
- The correct fix is to extend the existing `config_v2::types::PostgresConfig`, not invent a new adapter. Keep the validated runtime shape flat by adding `local_database`, `pg_hba_contents`, and `pg_ident_contents` directly onto `PostgresConfig`.
- `postgres.local_database` must be validated as non-empty during config-v2 ingestion and then carried forward as a plain `String`.
- `postgres.access.{hba,ident}` are ingestion concerns, not runtime DTOs. Resolve inline-or-path to authoritative file contents during config-v2 parsing and store those contents on `PostgresConfig`; do not leak `PathOrInline`/`InlineOrPath` or create a `PostgresAccessConfigV2`.
- Freshly proven design gap from ultra-long execution after the earlier parser fixes: the v2 binary shape is still incomplete. HA fixture runtime configs legitimately use `process.binaries.overrides.postgres` and `process.binaries.overrides.psql` wrapper paths, but `config_v2::types::BinariesConfig` currently drops them and `src/config_v2/parser/load_config.rs` rejects them as unsupported. That means the process/tool/runtime corridor is not actually ready to be fully v2-only yet.
- This is the same config-boundary smell as the earlier postgres field gap: the fix is to extend the existing `config_v2::types::BinariesConfig` with the still-legitimate runtime binary paths and move remaining direct reads onto that shared v2 type. Do not reintroduce `ProcessConfig`, do not invent `ProcessConfigV2`, and do not keep the rejection in the parser while runtime/bootstrap code still materially depends on those binaries.
- The concrete readers are already proven: the HA materialized runtime configs set both wrapper paths, `logging::postgres_ingest` still needs `psql` in its process/log-ingest integration corridor, and the process/tool runtime still needs the authoritative `postgres` path. Those must become direct `cfg.binaries.{postgres,psql}` reads during execution instead of leaking back through legacy binary-resolution helpers.
- The operator-side config_v2 type graph is now corrected first: `OperatorConfigV2` carries `pgtm.api.expected_transport`, and the operator loader must preserve that field on the shared validated shape instead of rejecting it.
- Keep one shared `OperatorConfigV2` root for pgtm. Do not rebuild legacy `PgtmConfig` adapters or add another mirror DTO just to carry API transport expectations.
- Freshly proven design gap from the repaired long-suite execution is now corrected in the shared type graph: `config_v2::types::PostgresConfig` must own one validated `source_client_tls` leaf for replica-source conninfo, and `RoleConfig` must stay reduced to credentials only.
- `src/config_v2/parser/load_config.rs` and `src/dev_support/runtime_config_v2.rs` must both map the raw/legacy `postgres.rewind.transport` input directly onto that shared `cfg.postgres.source_client_tls` field, including the CA path, instead of smearing transport state onto per-role config or discarding it.
- `process::source::source_from_member` must read `cfg.postgres.source_client_tls` directly so `pg_basebackup` and `pg_rewind` share the authoritative client TLS settings without rebuilding config wrappers or abusing the server-side `TlsConfig` shape.

## Execution order for the next turn

1. Delete `RuntimeConfigV2.node` and promote `cluster_name`, `scope`, and `member_id` onto `RuntimeConfigV2` using the existing `ClusterName`, `ScopeName`, and `MemberId` wrappers.
2. Update `src/config_v2/parser/load_config.rs` and the immediate `runtime/node.rs` reads to match the flattened root. Do not add any helper that reconstructs `NodeIdentity` inside `RuntimeConfigV2`.
3. Use the corrected v2 postgres fields directly in the remaining runtime reductions: `cfg.postgres.local_database`, `cfg.postgres.pg_hba_contents`, `cfg.postgres.pg_ident_contents`, and `cfg.postgres.source_client_tls`. Do not rebuild old config or recreate source-world access enums to reach them.
4. Keep `run_workers` as the one owner of the runtime config value and pass `&cfg` into the full startup graph. The touched runtime contexts in this pass must borrow the shared cfg with lifetimes; do not solve this with clones, `Arc`, or subscribers. If a callsite still needs identity, construct one local `NodeIdentity` once from the flattened cfg root in `runtime/node.rs`.
5. Delete the config-subscriber corridor from `src/runtime/node.rs`. Do not create `new_state_channel(cfg.clone())`. Every direct startup callee reached from `runtime/node.rs` must instead take the shared config-v2 root directly.
6. In the same runtime-root pass, convert every direct callee listed above from `crate::config::RuntimeConfig` to config-v2/shared-cfg. Do not leave a mixed v1/v2 startup graph, and do not leave some modules on subscribers while others use direct cfg.
7. Accept compiler errors and inspect each nested type reached from that pass. Remove cfg-derived stored fields instead of recreating them behind wrappers.
8. The mandatory nested reductions in that same pass are:
   - `ProcessRuntimePlan`
   - `ProcessWorkerCtx.config`
   - `ProcessRuntime.capture_subprocess_output`
   - `ProcessObservedState.runtime_config`
   - `ProcessObservedSnapshot.runtime_config`
   - `ProcessCluster::prepare(config, capture_output)`
   - `ExternalToolLowerer::{build_command, resolve_process_binary}`
   - `process::worker::timeout_for_kind(..., config)`
   - `PgInfoWorkerCtx.probe_conninfo`
   - `PgInfoCadence.poll_interval`
   - `HaObservedState.config`
   - `HaWorkerCadence.poll_interval`
   - `ApiRuntimeCtx.runtime_config`
   - `ApiRuntimeCtx.bind`
   - `ApiRuntimeCtx.auth`
   - `PostgresIngestWorkerCtx.cfg`
   - `process::jobs::{BootstrapSpec.superuser, MandatoryRoleSourceConn.auth, ProcessEnvValue::Secret}`
   - `postgres_managed::materialize_managed_postgres_config` and its TLS/auth helpers
   - `tls::{build_api_server_transport, build_api_server_config}`
   - `dcs::worker::EtcdRuntime::connect_options`
   - `logging::postgres_ingest::cleanup_log_dir`'s old-config cleanup shape
   - the already-documented `dcs::worker::DcsWorkerInputs` static cfg fields once that corridor is touched
9. The v2 binary type graph is now corrected first:
   - `config_v2::types::BinariesConfig` must stay as the one shared validated binary shape, including `postgres` and `psql`
   - `src/config_v2/parser/load_config.rs` must keep ingesting those paths directly from `process.binaries.overrides`
   - the next execution pass must revisit the process/tool/HA/log-ingest corridors that still depend on those binaries and replace old binary-resolution helpers with direct `ctx.cfg.binaries` reads
   - if a caller still wants `PostgresBinaryName` or `cfg.process.binaries.resolve_binary_path(...)`, that boundary has not been reduced yet
10. With the operator-side config_v2 shape corrected, keep execution on the shared operator root:
   - use `OperatorConfigV2.expected_transport` if a pgtm/API boundary still needs the transport expectation
   - do not reintroduce legacy `PgtmConfig` adapters or a separate mirror DTO just to carry this field
11. If execution pressure tempts you to rebuild old `RuntimeConfig`, introduce `StateSubscriber<RuntimeConfigV2>`, invent a new mirror like `ProcessConfigV2`, or recreate raw access/auth DTOs under v2 names, stop immediately, rewrite this section again, and leave the tail marker as `TO BE VERIFIED`.
12. If another helper proves it still needs static config that is genuinely absent from `RuntimeConfigV2` or `OperatorConfigV2`, switch back to `TO BE VERIFIED` immediately and correct the shared v2 type graph before continuing.

## Expected compiler errors after this design is executed

- `cfg.node.*` no longer exists after the root flattening
- `new_state_channel(cfg.clone())` and all subscriber-based config fields no longer type-check
- direct signature mismatches on `bootstrap`, `build_ctx`, and `from_config`
- old logging, HA, API, postgres-ingest, DCS, and process field paths no longer exist once direct config-v2 reads replace them
- `ProcessConfig`, `RuntimeConfig`, `PostgresRoleName`, `RoleAuthConfig`, `SecretSource`, `ApiTransportConfig`, `ApiTlsConfig`, `DcsClientConfig`, and other old config types continue to appear inside runtime contexts and helper signatures until the nested reductions are finished
- callers that still hard-code `"postgres"` or still read old `cfg.postgres.access.*` / old auth DTOs will fail once their boundaries move to direct config-v2 reads

## Correct fix direction

- read `cfg.cluster_name`, `cfg.scope`, and `cfg.member_id` directly
- read `cfg.logging.stderr_enabled`, `cfg.logging.file_enabled`, `cfg.logging.file_path`, and `cfg.logging.file_mode` directly
- read other static settings from `cfg.postgres`, `cfg.api`, `cfg.dcs`, `cfg.logging`, `cfg.binaries`, and `cfg.timing` directly
- use `cfg.postgres.local_database`, `cfg.postgres.pg_hba_contents`, and `cfg.postgres.pg_ident_contents` as the source of truth for managed-role reconciliation and managed-postgres materialization
- if `ProcessRuntimePlan` becomes empty after removing cfg mirrors, delete it instead of renaming it
- keep one shared cfg root per daemon runtime, not cloned subscriber state
- do not add bridge helpers between `crate::config` and `crate::config_v2`
- do not reintroduce a config wrapper like `cfg.identity`, `cfg.static_node`, a rebuilt `RuntimeConfig`, or a config channel/subscriber façade
- do not invent new mirror types such as `ProcessConfigV2`; if a field is static config, delete it from runtime state and read from `self.cfg`
- do not keep old config leaf DTOs around under new names either; if a process/API/DCS helper still wants `RoleAuthConfig`, `SecretSource`, `ApiTlsConfig`, or `DcsClientConfig`, that helper boundary is part of the active reduction and must move to config-v2
- do not create a new `PostgresAccessConfigV2`, do not leak `PathOrInline`/`InlineOrPath` out of config-v2 parsing, and do not work around missing fields by hard-coding defaults like `"postgres"` for the local database
- do not reintroduce `process.binaries` adapter helpers now that `cfg.binaries.{postgres,psql,pg_ctl,pg_rewind,initdb,pg_basebackup}` is the shared validated shape

NOW EXECUTE
