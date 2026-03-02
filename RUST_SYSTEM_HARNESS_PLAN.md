# Pgtuskmaster Rust Harness Plan (Full System, Compiler-Driven)

## Intent
- Define the full system contract now: states, enums, function signatures, thread responsibilities, and module boundaries.
- Defer real IO internals until interfaces compile.
- Keep HA decisions pure and testable with injected dependencies.
- Use `tokio::watch` state channels as primary state propagation mechanism.
- Use minimal wake events only to avoid polling loops.

## Non-Negotiables Captured
- Zero `unwrap()` policy in production code.
- All runtime behavior controlled by config (including binary paths and timeouts), not env vars.
- Fields used by HA decisions must be strongly typed.
- DCS/PG integration tests must run concurrently on same host with no collisions.
- E2E scenario tests are first-class.

## High-Level Architecture
```text
PgInfo Worker  --publishes--> watch<PgInfoState> ----\
DCS Worker     --publishes--> watch<DcsState> ------- \
Process Worker --publishes--> watch<ProcessState> ---- > HA Worker (pure decide + action dispatch)
API Worker     --publishes--> watch<ApiState> ------- /
Config Worker  --publishes--> watch<RuntimeConfig> --/

HA Worker -> sends typed commands to workers (mpsc)
Workers -> send WakeReason::StateUpdated(Source) to HA (mpsc) after successful watch publish

Debug API reads all watchers and queue gauges to expose snapshots.
```

## Why `watch` + Wake Signal
- `watch` carries full authoritative state snapshots.
- Wake signal only means: "new version exists".
- HA never depends on fragile per-field events.
- Avoids bug class: forgetting to emit a specific event variant.

## Module Plan
```text
src/
  app/
    mod.rs
    bootstrap.rs
    wiring.rs
  config/
    mod.rs
    schema.rs
    defaults.rs
    source.rs
  state/
    mod.rs
    ids.rs
    time.rs
    watch_state.rs
    error.rs
  pginfo/
    mod.rs
    state.rs
    query.rs
    worker.rs
  dcs/
    mod.rs
    keys.rs
    state.rs
    worker.rs
    etcd_client.rs
  process/
    mod.rs
    jobs.rs
    state.rs
    worker.rs
  ha/
    mod.rs
    state.rs
    decide.rs
    actions.rs
    worker.rs
  api/
    mod.rs
    controller.rs
    fallback.rs
    worker.rs
    state.rs
  debug_api/
    mod.rs
    snapshot.rs
    worker.rs
  test_harness/
    mod.rs
    namespace.rs
    ports.rs
    pg16.rs
    etcd3.rs
```

## Shared Foundation Types

### IDs and Scalars
```rust
pub struct MemberId(pub String);
pub struct ClusterName(pub String);
pub struct SwitchoverRequestId(pub String);
pub struct JobId(pub String);
pub struct TimelineId(pub u32);
pub struct WalLsn(pub u64);
pub struct SlotName(pub String);
pub struct Version(pub u64);
pub struct UnixMillis(pub u64);
```

### App and Worker Lifecycle
```rust
pub enum AppLifecycle {
    Booting,
    Starting,
    Running,
    Draining,
    Stopped,
    Faulted(AppError),
}

pub enum WorkerStatus {
    Starting,
    Running,
    Stopping,
    Stopped,
    Faulted(WorkerError),
}
```

Note: no `Panic` variant. Any thread task failure is captured as `Faulted(...)` and bubbled through supervisors.

### Typed Errors (No unwrap flow)
```rust
pub enum AppError { Config(ConfigError), Wiring(WiringError), Runtime(RuntimeError) }
pub enum WorkerError { Io(String), Timeout(String), Protocol(String), Invariant(String) }
pub enum DecideError { InvalidInput(String) }
pub enum ActionError { Dispatch(String), Validation(String) }
```

## Config Model (All Behavior From Config)
```rust
pub struct RuntimeConfig {
    pub cluster: ClusterConfig,
    pub pg: PgRuntimeConfig,
    pub dcs: DcsRuntimeConfig,
    pub ha: HaRuntimeConfig,
    pub process: ProcessRuntimeConfig,
    pub api: ApiRuntimeConfig,
    pub debug_api: DebugApiRuntimeConfig,
}

pub struct ProcessRuntimeConfig {
    pub pg_rewind_timeout_ms: u64,
    pub bootstrap_timeout_ms: u64,
    pub process_poll_interval_ms: u64,
    pub binaries: BinaryPaths,
}

pub struct BinaryPaths {
    pub postgres: String,
    pub pg_ctl: String,
    pub pg_rewind: String,
    pub initdb: String,
}
```

Required functions:
```rust
pub fn load_runtime_config(path: &std::path::Path) -> Result<RuntimeConfig, ConfigError>;
pub fn validate_runtime_config(cfg: &RuntimeConfig) -> Result<(), ConfigError>;
pub fn apply_defaults(cfg: PartialRuntimeConfig) -> RuntimeConfig;
```

## `watch` Wrapper Contract
```rust
pub struct Versioned<T> {
    pub version: Version,
    pub updated_at: UnixMillis,
    pub value: T,
}

pub struct StateTx<T> { tx: tokio::sync::watch::Sender<Versioned<T>> }
pub struct StateRx<T> { rx: tokio::sync::watch::Receiver<Versioned<T>> }
```

Functions:
```rust
pub fn new_state_channel<T: Clone>(initial: T, now: UnixMillis) -> (StateTx<T>, StateRx<T>);

impl<T: Clone> StateTx<T> {
    pub fn publish(&self, next: T, now: UnixMillis) -> Result<Version, StatePublishError>;
    pub fn current(&self) -> Versioned<T>;
}

impl<T: Clone> StateRx<T> {
    pub fn latest(&self) -> Versioned<T>;
    pub async fn changed(&mut self) -> Result<Versioned<T>, StateRecvError>;
}
```

## PgInfo Worker

### `PgInfo` Shape Decision: Variant Types
Use variant payloads so primary/replica-only fields are encoded at type level.

```rust
pub enum PgInfoState {
    Unknown(PgUnknownInfo),
    Primary(PrimaryPgInfo),
    Replica(ReplicaPgInfo),
}

pub struct PgUnknownInfo {
    pub worker: WorkerStatus,
    pub readiness: Readiness,
    pub sql: SqlStatus,
    pub last_refresh_at: Option<UnixMillis>,
}

pub struct PrimaryPgInfo {
    pub worker: WorkerStatus,
    pub readiness: Readiness,
    pub sql: SqlStatus,
    pub wal_lsn: WalLsn,
    pub slots: Vec<ReplicationSlotInfo>,
    pub pg_config: TypedPgConfig,
    pub last_refresh_at: UnixMillis,
}

pub struct ReplicaPgInfo {
    pub worker: WorkerStatus,
    pub readiness: Readiness,
    pub sql: SqlStatus,
    pub replay_lsn: WalLsn,
    pub follow_lsn: Option<WalLsn>,
    pub upstream: Option<UpstreamIdentity>,
    pub pg_config: TypedPgConfig,
    pub last_refresh_at: UnixMillis,
}
```

`last_poll` vs `last_change` decision:
- Keep only `last_refresh_at` in state.
- Change detection is already encoded by `Versioned<T>.version` from `watch`.
- This removes duplicated time semantics.

Functions:
```rust
pub async fn run_pginfo_worker(ctx: PgInfoWorkerCtx) -> Result<(), WorkerError>;
pub async fn poll_pginfo(client: &mut dyn PgSqlClient, cfg: &PgRuntimeConfig) -> Result<PgInfoState, PgPollError>;
pub fn derive_readiness(info: &PgInfoState, cfg: &PgRuntimeConfig) -> Readiness;
pub fn build_member_record(info: &PgInfoState, member_id: &MemberId) -> MemberRecord;
```

## DCS Worker (etcd3 owner)

### Trust and Mode
```rust
pub enum DcsTrust {
    FullQuorum,
    FailSafeNoQuorum,
    NotTrusted,
}

pub enum DcsMode {
    EtcdAuthoritative,
    FailSafe,
    FallbackConsensusBacklog, // modeled, not implemented
}
```

### DCS State
```rust
pub struct DcsState {
    pub worker: WorkerStatus,
    pub trust: DcsTrust,
    pub mode: DcsMode,
    pub etcd: EtcdStatus,
    pub cache: DcsCache,
    pub last_refresh_at: Option<UnixMillis>,
}

pub struct DcsCache {
    pub members: std::collections::BTreeMap<MemberId, MemberRecord>,
    pub leader: Option<LeaderRecord>,
    pub switchover: Option<SwitchoverRequest>,
    pub config: DcsConfigRecord,
    pub init_lock: Option<InitLockRecord>,
}
```

Functions:
```rust
pub async fn run_dcs_worker(ctx: DcsWorkerCtx) -> Result<(), WorkerError>;
pub async fn upsert_member(client: &mut dyn DcsClient, member: MemberRecord) -> Result<(), DcsWriteError>;
pub async fn refresh_cache_from_watch(client: &mut dyn DcsClient, prev: &DcsCache) -> Result<DcsCache, DcsWatchError>;
pub fn evaluate_trust(etcd: &EtcdStatus, quorum: QuorumStatus) -> DcsTrust;
pub fn map_key_to_enum(path: &str) -> Option<DcsKey>;
```

## Process Worker (Long-Running Operations)

State:
```rust
pub struct ProcessState {
    pub worker: WorkerStatus,
    pub active: Option<ActiveJob>,
    pub queued: std::collections::VecDeque<ProcessJob>,
    pub history: Vec<JobResult>,
}

pub enum ProcessJob {
    PgRewind(PgRewindSpec),
    Bootstrap(BootstrapSpec),
}

pub enum JobResult {
    Succeeded { id: JobId, finished_at: UnixMillis },
    Failed { id: JobId, error: ProcessError, finished_at: UnixMillis },
    TimedOut { id: JobId, finished_at: UnixMillis },
    Cancelled { id: JobId, finished_at: UnixMillis },
}
```

Functions:
```rust
pub async fn run_process_worker(ctx: ProcessWorkerCtx) -> Result<(), WorkerError>;
pub async fn start_job(deps: &ProcessDeps, cfg: &RuntimeConfig, job: ProcessJob) -> Result<ActiveJob, ProcessError>;
pub async fn tick_active_job(deps: &ProcessDeps, cfg: &RuntimeConfig, active: &mut ActiveJob) -> Result<JobTickOutcome, ProcessError>;
pub async fn stop_active_job(deps: &ProcessDeps, active: ActiveJob, reason: StopReason) -> Result<JobResult, ProcessError>;
```

## HA Worker (Main Loop)

State:
```rust
pub struct HaState {
    pub worker: WorkerStatus,
    pub phase: HaPhase,
    pub tick: u64,
    pub pending: Vec<HaAction>,
    pub last_error: Option<DecideError>,
}

pub enum HaPhase {
    Init,
    WaitingPgReady,
    WaitingDcsTrusted,
    Replica,
    CandidateLeader,
    Primary,
    Rewinding,
    Bootstrapping,
    FailSafe,
}
```

No `DecisionSummary` struct needed initially; decision trace can be derived in debug builds from input + outcome.

Decision and action interfaces:
```rust
pub struct WorldSnapshot {
    pub config: Versioned<RuntimeConfig>,
    pub pg: Versioned<PgInfoState>,
    pub dcs: Versioned<DcsState>,
    pub process: Versioned<ProcessState>,
    pub api: Versioned<ApiState>,
}

pub enum WakeReason {
    StateUpdated(StateSource),
    Tick,
    Shutdown,
}

pub enum StateSource { Config, PgInfo, Dcs, Process, Api }

pub struct DecideInput {
    pub current: HaState,
    pub world: WorldSnapshot,
    pub wake: WakeReason,
}

pub struct DecideOutput {
    pub next: HaState,
    pub actions: Vec<HaAction>,
}

pub fn decide(input: DecideInput) -> Result<DecideOutput, DecideError>;
pub async fn run_ha_worker(ctx: HaWorkerCtx) -> Result<(), WorkerError>;
pub async fn dispatch_action(action: HaAction, deps: &HaDispatchDeps) -> Result<(), ActionError>;
```

Idempotency requirement:
```rust
pub struct ActionId(pub String);

pub enum HaAction {
    PublishMember { id: ActionId },
    ReconcilePgConfig { id: ActionId },
    PromoteToLeader { id: ActionId },
    DemoteFromPrimary { id: ActionId },
    StartPgRewind { id: ActionId, spec: PgRewindSpec },
    StartBootstrap { id: ActionId, spec: BootstrapSpec },
    EnterFailSafe { id: ActionId, reason: String },
}
```

`ActionId` dedupe cache should exist in HA worker state or dispatcher.

## API Worker (Controller + Fallback)

State:
```rust
pub struct ApiState {
    pub worker: WorkerStatus,
    pub controller: ControllerApiState,
    pub fallback: FallbackApiState,
}
```

Functions:
```rust
pub async fn run_api_worker(ctx: ApiWorkerCtx) -> Result<(), WorkerError>;
pub async fn handle_post_switchover(req: SwitchoverRequestInput, deps: &ApiDeps) -> Result<AcceptedResponse, ApiError>;
pub async fn handle_fallback_get_cluster(deps: &ApiDeps) -> Result<FallbackClusterView, ApiError>;
pub async fn handle_fallback_post_heartbeat(req: HeartbeatInput, deps: &ApiDeps) -> Result<AcceptedResponse, ApiError>;
```

`SwitchoverRequest` includes request id:
```rust
pub struct SwitchoverRequest {
    pub id: SwitchoverRequestId,
    pub requested_at: UnixMillis,
    pub requested_by: String,
    pub target: Option<MemberId>,
    pub reason: Option<String>,
}
```

## Debug API Worker

State and snapshot:
```rust
pub struct DebugApiState {
    pub worker: WorkerStatus,
    pub last_snapshot_at: Option<UnixMillis>,
}

pub struct SystemSnapshot {
    pub app: AppLifecycle,
    pub config: Versioned<RuntimeConfig>,
    pub pg: Versioned<PgInfoState>,
    pub dcs: Versioned<DcsState>,
    pub process: Versioned<ProcessState>,
    pub ha: Versioned<HaState>,
    pub api: Versioned<ApiState>,
    pub queue_metrics: QueueMetrics,
}
```

Functions:
```rust
pub async fn run_debug_api_worker(ctx: DebugApiCtx) -> Result<(), WorkerError>;
pub fn build_system_snapshot(deps: &DebugSnapshotDeps, now: UnixMillis) -> SystemSnapshot;
```

Debug requirement choice captured:
- v1 returns full state snapshots + queue counts.
- Full queued payload dumping is optional and can be gated behind debug flag.

## Inter-Thread Connections (Explicit)

### Commands (HA -> Workers)
```rust
pub enum PgInfoCommand { PollNow }
pub enum DcsCommand { UpsertLocalMember(MemberRecord), SetLeader(LeaderRecord), SetSwitchover(SwitchoverRequest) }
pub enum ProcessCommand { Enqueue(ProcessJob), Cancel(JobId) }
pub enum ApiCommand { PublishSwitchoverAccepted(SwitchoverRequestId) }
```

### Wake Bus (Workers -> HA)
```rust
pub type WakeTx = tokio::sync::mpsc::Sender<WakeReason>;
pub type WakeRx = tokio::sync::mpsc::Receiver<WakeReason>;
```

Contract:
- Worker updates own state via `StateTx.publish`.
- If publish success and version changed, worker sends `WakeReason::StateUpdated(source)` to HA.
- HA reads latest state from all watchers before every `decide`.

## Leader Semantics Captured
- Authoritative intent key: `/leader`.
- Node promotion condition (candidate node):
  - local member id == `/leader` id
  - local PG role currently replica
  - no conflicting primary leader evidence in `/members`
  - DCS trust is not `NotTrusted`
- Node demotion condition (current primary):
  - local member id != `/leader` id
  - local PG role currently primary
  - action: `DemoteFromPrimary`

Represent as pure functions:
```rust
pub fn should_promote(world: &WorldSnapshot, self_id: &MemberId) -> bool;
pub fn should_demote(world: &WorldSnapshot, self_id: &MemberId) -> bool;
```

## Typed PG Config Requirement

Approach:
- `TypedPgConfig` has explicit fields used by HA.
- `extra: BTreeMap<String, String>` stores non-HA fields.

```rust
pub struct TypedPgConfig {
    pub synchronous_standby_names: Option<String>,
    pub max_wal_senders: u32,
    pub wal_level: WalLevel,
    pub hot_standby: bool,
    pub primary_conninfo: Option<String>,
    pub extra: std::collections::BTreeMap<String, String>,
}
```

This satisfies:
- HA decision fields are always typed and matched by compiler.
- Long-tail keys are still represented and replicated.

## Test Strategy (Detailed)

### 1. Unit Tests (Pure and Fast)
- `ha::decide` transition tests with table-driven inputs.
- Action dedupe logic tests.
- PgInfo parsing/derivation tests from query rows.
- DCS trust evaluation tests.
- Config validation/defaulting tests.

### 2. Integration Tests (Real PG16 + Real etcd3)
- Test each action against real dependencies.
- Test each long-running process (`pg_rewind`, `bootstrap`) against real dependencies.
- Test HA loop with dependency injection where adapters are real process-backed clients.

### 3. E2E Scenario Tests
- Full multi-worker app started in-process.
- Scenarios:
  - bootstrap new cluster
  - leader crash and failover
  - switchover request path
  - no quorum enters fail-safe
  - rewind then rejoin as replica

## Parallel-Safe Test Harness Design

### Namespace Isolation
Every test acquires a unique namespace:
```rust
pub struct TestNamespace {
    pub id: String,               // e.g. testname + random suffix
    pub root_dir: std::path::PathBuf,
    pub port_pool: AllocatedPorts,
}
```

Functions:
```rust
pub fn create_test_namespace(test_name: &str) -> Result<TestNamespace, HarnessError>;
pub fn cleanup_test_namespace(ns: TestNamespace) -> Result<(), HarnessError>;
```

### Port Allocation
- Allocate free ports dynamically per test process.
- Persist selected ports in namespace state to avoid reuse race.

Functions:
```rust
pub fn allocate_ports(count: usize) -> Result<AllocatedPorts, HarnessError>;
pub fn reserve_port(port: u16) -> Result<(), HarnessError>;
```

### Filesystem Isolation
- Unique PGDATA per node per test namespace.
- Unique etcd data dir per test namespace.
- Unique unix socket dirs where needed.

Functions:
```rust
pub fn prepare_pgdata_dir(ns: &TestNamespace, node: &str) -> Result<std::path::PathBuf, HarnessError>;
pub fn prepare_etcd_data_dir(ns: &TestNamespace) -> Result<std::path::PathBuf, HarnessError>;
```

### Runtime Spawners
```rust
pub async fn spawn_pg16_instance(spec: PgInstanceSpec) -> Result<PgInstanceHandle, HarnessError>;
pub async fn spawn_etcd3_instance(spec: EtcdInstanceSpec) -> Result<EtcdInstanceHandle, HarnessError>;
pub async fn wait_pg_ready(handle: &PgInstanceHandle, timeout_ms: u64) -> Result<(), HarnessError>;
pub async fn wait_etcd_ready(handle: &EtcdInstanceHandle, timeout_ms: u64) -> Result<(), HarnessError>;
```

## Compiler-Driven Build Order
1. Create `state/*` foundational types and watch wrapper.
2. Create `config/schema.rs` + defaults + validation.
3. Add worker state structs and command enums for all modules.
4. Add `ha::decide` signature and minimal exhaustive transition scaffold.
5. Add worker context structs and `run_*_worker` loop signatures.
6. Add wiring/bootstrap to instantiate channels/watchers/contexts.
7. Add debug snapshot assembler.
8. Add unit tests for pure functions.
9. Add test harness namespace + real PG/etcd spawners.
10. Add integration and e2e scenarios.

## Immediate Output of This Plan
After approval, next task should generate compile-only skeleton code for:
- all enums/structs above,
- all function signatures above returning `todo!()` or typed placeholder errors,
- all module files wired into `mod.rs`,
- a first compile pass (`cargo check`) to force missing type closure.
