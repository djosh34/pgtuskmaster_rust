## Task: Follow The Direct-`cfg` Reduction Loop One Root At A Time <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Define the exact reduction loop that every config-v2 migration task in this story must follow. The higher-order goal is to stop vague “migrate to RuntimeConfigV2” work and replace it with one strict invasive loop:

1. find one root-level place that first uses `cfg`
2. change that root to `RuntimeConfigV2`
3. accept the compiler errors
4. inspect every field on every nested type reached from that root
5. if any nested field is really a config field in disguise, delete it from that type
6. fix the resulting errors by reading directly from `self.cfg`
7. when the package is valid again, QUIT IMMEDIATELY
8. next iteration continues deeper

This task is a playbook task for the rest of the story. It does not replace the package tasks; it defines the algorithm they must follow.

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
    member_ttl_ms: u64,
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
            inputs: DcsWorkerInputs {
                keys: DcsKeySpace::new(identity.scope.as_str()),
                identity,
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
