# Pgtuskmaster Rust System Harness Plan

## Design Goals
- Compiler-driven development: define complete types + function signatures first.
- Private-by-default module APIs; expose only minimal `pub(crate)` surfaces needed for wiring/tests.
- Worker communication through typed `tokio::watch` state channels.
- HA decision engine is pure and deterministic.
- Zero `unwrap()` policy in production code.
- Runtime behavior controlled by config only.

## Strict Lint Policy
- Runtime crate builds deny `clippy::unwrap_used`, `clippy::expect_used`, `clippy::panic`, `clippy::todo`, and `clippy::unimplemented` at the crate root.
- `make lint` runs two passes:
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo clippy --lib --all-features -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::todo -D clippy::unimplemented`
- Test harness helpers use narrowly scoped lint allowances in `src/test_harness/mod.rs` so test fixture ergonomics do not weaken runtime lint guarantees.

## Real-Binary Test Prerequisites
- Real-system tests resolve binaries from repository-local paths:
  - PostgreSQL 16 tools: `.tools/postgres16/bin/{postgres,pg_ctl,pg_rewind,initdb,psql,pg_basebackup}`
  - etcd: `.tools/etcd/bin/etcd`
- Required default flow: `make test`
  - Missing real-test binaries now fail fast with explicit prerequisite errors.
- Focused real-only flow: `make test-real`
  - Runs the real-binary suites directly and also fails fast if prerequisites are missing.
- CI should install/copy binaries into those `.tools/...` paths before running `make test` or `make test-real`.
- Example package sources (distribution-dependent):
  - PostgreSQL 16 binaries from `postgresql-16` / official PostgreSQL apt/yum packages.
  - etcd binary from `etcd-server` package or official etcd release artifact.

## Visibility Policy (`pub` vs private)
- In Rust, `pub` means visible outside the current module scope (not automatically mutable).
- Mutation still requires mutable access (`&mut`) or interior mutability.
- Plan policy:
  - `pub` only for crate root app entrypoints and wire types that must cross module boundaries.
  - `pub(crate)` for internal cross-module contracts.
  - private (`fn`, `struct` fields without `pub`) for worker internals.

## Thread/Worker Model
- Each worker owns one authoritative state.
- Each worker publishes that state via one `watch::Sender<Versioned<State>>`.
- Consumers subscribe via `watch::Receiver`.
- No per-field event enums.
- HA loop waits on `changed()` for multiple watchers plus periodic tick.

## Module Layout
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
    parser.rs
  state/
    mod.rs
    ids.rs
    time.rs
    watch_state.rs
    errors.rs
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
    store.rs
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
    tls.rs
    auth.rs
```

## Shared Core Types

```rust
pub(crate) struct MemberId(String);
pub(crate) struct ClusterName(String);
pub(crate) struct SwitchoverRequestId(String);
pub(crate) struct JobId(String);
pub(crate) struct WalLsn(u64);
pub(crate) struct TimelineId(u32);
pub(crate) struct UnixMillis(u64);
pub(crate) struct Version(u64);
```

```rust
pub(crate) enum WorkerStatus {
    Starting,
    Running,
    Stopping,
    Stopped,
    Faulted(WorkerError),
}
```

```rust
pub(crate) struct Versioned<T> {
    version: Version,
    updated_at: UnixMillis,
    value: T,
}
```

## `watch` Wrapper (Typed, Restricted)

```rust
pub(crate) struct StatePublisher<T> {
    tx: tokio::sync::watch::Sender<Versioned<T>>,
}

pub(crate) struct StateSubscriber<T> {
    rx: tokio::sync::watch::Receiver<Versioned<T>>,
}
```

```rust
pub(crate) fn new_state_channel<T: Clone>(
    initial: T,
    now: UnixMillis,
) -> (StatePublisher<T>, StateSubscriber<T>);

impl<T: Clone> StatePublisher<T> {
    pub(crate) fn publish(&self, next: T, now: UnixMillis) -> Result<Version, StatePublishError>;
    pub(crate) fn latest(&self) -> Versioned<T>;
}

impl<T: Clone> StateSubscriber<T> {
    pub(crate) fn latest(&self) -> Versioned<T>;
    pub(crate) async fn changed(&mut self) -> Result<Versioned<T>, StateRecvError>;
}
```

## Config Model (All Runtime Inputs Typed)

```rust
pub(crate) struct RuntimeConfig {
    cluster: ClusterConfig,
    postgres: PostgresConfig,
    dcs: DcsConfig,
    ha: HaConfig,
    process: ProcessConfig,
    api: ApiConfig,
    debug: DebugConfig,
    security: SecurityConfig,
}
```

```rust
pub(crate) struct ProcessConfig {
    pg_rewind_timeout_ms: u64,
    bootstrap_timeout_ms: u64,
    fencing_timeout_ms: u64,
    binaries: BinaryPaths,
}

pub(crate) struct BinaryPaths {
    postgres: std::path::PathBuf,
    pg_ctl: std::path::PathBuf,
    pg_rewind: std::path::PathBuf,
    initdb: std::path::PathBuf,
    psql: std::path::PathBuf,
}
```

```rust
pub(crate) fn load_runtime_config(path: &std::path::Path) -> Result<RuntimeConfig, ConfigError>;
pub(crate) fn apply_defaults(raw: PartialRuntimeConfig) -> RuntimeConfig;
pub(crate) fn validate_runtime_config(cfg: &RuntimeConfig) -> Result<(), ConfigError>;
```

## PgInfo Worker

### State Shape With Extracted Common Fields
```rust
pub(crate) struct PgInfoCommon {
    worker: WorkerStatus,
    sql: SqlStatus,
    readiness: Readiness,
    timeline: Option<TimelineId>,
    pg_config: PgConfig,
    last_refresh_at: Option<UnixMillis>,
}

pub(crate) enum PgInfoState {
    Unknown { common: PgInfoCommon },
    Primary { common: PgInfoCommon, wal_lsn: WalLsn, slots: Vec<ReplicationSlotInfo> },
    Replica { common: PgInfoCommon, replay_lsn: WalLsn, follow_lsn: Option<WalLsn>, upstream: Option<UpstreamInfo> },
}
```

### Single Poll Query Requirement
- PgInfo worker uses one SQL query returning all required HA fields each cycle.

```rust
pub(crate) const PGINFO_POLL_SQL: &str = "...single query text...";
```

### Function Signatures
```rust
pub(crate) async fn run(ctx: PgInfoWorkerCtx) -> Result<(), WorkerError>;
pub(crate) async fn step_once(ctx: &mut PgInfoWorkerCtx) -> Result<(), WorkerError>; // test-focused inner loop

fn poll_once(client: &mut dyn PgSqlClient, cfg: &PostgresConfig) -> Result<PgInfoState, PgPollError>;
fn derive_readiness(info: &PgInfoState, cfg: &PostgresConfig) -> Readiness;
fn to_member_status(info: &PgInfoState, self_id: &MemberId) -> MemberStatus;
```

## DCS Worker (Owns etcd3 + Member Publishing)

### Trust Only (No Separate Mode)
```rust
pub(crate) enum DcsTrust {
    FullQuorum,
    FailSafe,
    NotTrusted,
}
```

### State
```rust
pub(crate) struct DcsState {
    worker: WorkerStatus,
    trust: DcsTrust,
    cache: DcsCache,
    last_refresh_at: Option<UnixMillis>,
}

pub(crate) struct DcsCache {
    members: std::collections::BTreeMap<MemberId, MemberRecord>,
    leader: Option<LeaderRecord>,
    switchover: Option<SwitchoverRequest>,
    config: DcsConfigRecord,
    init_lock: Option<InitLockRecord>,
}
```

### Ownership Rules
- DCS worker subscribes to `watch<PgInfoState>` directly.
- On PgInfo version change, DCS worker computes local member record and writes `/member/{self_id}`.
- No external `upsert_member(...)` API exposed to other modules.
- DCS key parsing is strongly typed and internal.

### Function Signatures
```rust
pub(crate) async fn run(ctx: DcsWorkerCtx) -> Result<(), WorkerError>;
pub(crate) async fn step_once(ctx: &mut DcsWorkerCtx) -> Result<(), WorkerError>; // test-focused inner loop

fn evaluate_trust(etcd: &EtcdStatus, quorum: QuorumStatus) -> DcsTrust;
fn build_local_member_record(pg: &PgInfoState, self_id: &MemberId) -> MemberRecord;
fn apply_watch_update(cache: &mut DcsCache, update: DcsWatchUpdate) -> Result<(), DcsStateError>;
fn key_from_path(path: &str) -> Result<DcsKey, DcsKeyParseError>;

async fn write_local_member(ctx: &mut DcsWorkerCtx, rec: MemberRecord) -> Result<(), DcsWriteError>;
async fn refresh_from_etcd_watch(ctx: &mut DcsWorkerCtx) -> Result<(), DcsWatchError>;
```

## Process Worker (Single Active Long-Running Job)

### No Queue, No History
- Only one active job at a time.
- HA decides next action; Process worker executes at most one.
- State publishes active/idle + latest outcome.

```rust
pub(crate) enum ProcessState {
    Idle { worker: WorkerStatus, last_outcome: Option<JobOutcome> },
    Running { worker: WorkerStatus, active: ActiveJob },
}
```

```rust
pub(crate) enum ProcessJobKind {
    Bootstrap(BootstrapSpec),
    PgRewind(PgRewindSpec),
    Promote(PromoteSpec),
    Demote(DemoteSpec),
    StartPostgres(StartPostgresSpec),
    StopPostgres(StopPostgresSpec),
    RestartPostgres(RestartPostgresSpec),
    Fencing(FencingSpec),
}
```

```rust
pub(crate) enum JobOutcome {
    Success { id: JobId, finished_at: UnixMillis },
    Failure { id: JobId, error: ProcessError, finished_at: UnixMillis },
    Timeout { id: JobId, finished_at: UnixMillis },
    Cancelled { id: JobId, finished_at: UnixMillis },
}
```

### Function Signatures
```rust
pub(crate) async fn run(ctx: ProcessWorkerCtx) -> Result<(), WorkerError>;
pub(crate) async fn step_once(ctx: &mut ProcessWorkerCtx) -> Result<(), WorkerError>; // test-focused inner loop

fn can_accept_job(state: &ProcessState) -> bool;
async fn start_job(ctx: &mut ProcessWorkerCtx, job: ProcessJobKind) -> Result<(), ProcessError>;
async fn tick_active_job(ctx: &mut ProcessWorkerCtx) -> Result<Option<JobOutcome>, ProcessError>;
async fn cancel_active_job(ctx: &mut ProcessWorkerCtx, reason: CancelReason) -> Result<JobOutcome, ProcessError>;
```

## HA Worker (Pure Decision + Dispatch)

### State
```rust
pub(crate) enum HaPhase {
    Init,
    WaitingPostgresReachable,
    WaitingDcsTrusted,
    Replica,
    CandidateLeader,
    Primary,
    Rewinding,
    Bootstrapping,
    Fencing,
    FailSafe,
}

pub(crate) struct HaState {
    worker: WorkerStatus,
    phase: HaPhase,
    tick: u64,
    pending: Vec<HaAction>,
    recent_action_ids: std::collections::BTreeSet<ActionId>, // idempotency dedupe
}
```

### Snapshot Input (No ApiState)
- HA decisions read only:
  - config
  - pginfo
  - dcs
  - process
- API effects appear through DCS (`/switchover_request`) and config.

```rust
pub(crate) struct WorldSnapshot {
    config: Versioned<RuntimeConfig>,
    pg: Versioned<PgInfoState>,
    dcs: Versioned<DcsState>,
    process: Versioned<ProcessState>,
}
```

### Decision + Dispatch
```rust
pub(crate) struct DecideInput {
    current: HaState,
    world: WorldSnapshot,
}

pub(crate) struct DecideOutput {
    next: HaState,
    actions: Vec<HaAction>,
}

pub(crate) fn decide(input: DecideInput) -> Result<DecideOutput, DecideError>;
pub(crate) async fn run(ctx: HaWorkerCtx) -> Result<(), WorkerError>;
pub(crate) async fn step_once(ctx: &mut HaWorkerCtx) -> Result<(), WorkerError>; // test-focused inner loop

async fn dispatch_actions(ctx: &mut HaWorkerCtx, actions: Vec<HaAction>) -> Result<(), ActionError>;
```

### Why No WakeReason
- HA uses `tokio::select!` over:
  - `pg_rx.changed()`
  - `dcs_rx.changed()`
  - `process_rx.changed()`
  - `config_rx.changed()`
  - periodic tick interval
- This removes extra wake buses and duplicate signal paths.

## API Worker
- API worker handles HTTP only.
- Controller endpoint writes typed `SwitchoverRequest` to DCS via internal DCS client adapter.
- Fallback API endpoints are scoped and typed.

```rust
pub(crate) async fn run(ctx: ApiWorkerCtx) -> Result<(), WorkerError>;
pub(crate) async fn step_once(ctx: &mut ApiWorkerCtx) -> Result<(), WorkerError>; // test-focused inner loop

async fn post_switchover(ctx: &ApiCtx, req: SwitchoverRequestInput) -> Result<AcceptedResponse, ApiError>;
async fn get_fallback_cluster(ctx: &ApiCtx) -> Result<FallbackClusterView, ApiError>;
async fn post_fallback_heartbeat(ctx: &ApiCtx, req: FallbackHeartbeatInput) -> Result<AcceptedResponse, ApiError>;
```

## Debug API Worker
- Debug API reads current snapshots from all watchers.
- Allowed to clone state for visibility.

```rust
pub(crate) struct SystemSnapshot {
    app: AppLifecycle,
    config: Versioned<RuntimeConfig>,
    pg: Versioned<PgInfoState>,
    dcs: Versioned<DcsState>,
    process: Versioned<ProcessState>,
    ha: Versioned<HaState>,
}
```

```rust
pub(crate) async fn run(ctx: DebugApiCtx) -> Result<(), WorkerError>;
pub(crate) fn build_snapshot(ctx: &DebugSnapshotCtx, now: UnixMillis) -> SystemSnapshot;
```

## Typed PG Config (No Raw `String` for Decisive Fields)

```rust
pub(crate) struct PgConfig {
    synchronous_standby_names: SynchronousStandbyNames,
    max_wal_senders: MaxWalSenders,
    wal_level: WalLevel,
    hot_standby: HotStandby,
    primary_conninfo: Option<PgConnInfo>,
    replication_slots: Vec<SlotConfig>,
    extra: std::collections::BTreeMap<String, String>,
}
```

```rust
pub(crate) struct PgConnInfo {
    hosts: Vec<HostPort>,
    user: PgUser,
    dbname: PgDatabase,
    ssl_mode: PgSslMode,
    application_name: Option<String>,
    connect_timeout_s: Option<u32>,
}
```

```rust
fn parse_pg_conninfo(raw: &str) -> Result<PgConnInfo, PgConnInfoError>;
fn render_pg_conninfo(ci: &PgConnInfo) -> String;
```

## Cross-Module Contracts (Minimal)
- Workers expose only:
  - `run(ctx)` for production runtime.
  - `step_once(&mut ctx)` for deterministic tests.
- All other helpers remain private.
- `step_once` is `pub(crate)` only.

## Test Strategy (Expanded, Real-System Heavy)

## 1. Unit Tests
- Exhaustive `ha::decide` transition matrix.
- Action idempotency and dedupe.
- DCS trust evaluation.
- DCS key parsing and cache application.
- Pg config parsing/rendering roundtrip.
- Conninfo parser strict validation.

## 2. PgInfo Real PG16 Tests
- Single polling query against real PG16 primary.
- Same query against real PG16 replica.
- Transition tests: startup unavailable -> available.
- WAL movement and slot fields verification.
- Role switch validation.

## 3. Action + Process Real Tests
- Each HA action tested end-to-end with real PG16/etcd3.
- Each process job kind real execution tests:
  - bootstrap
  - rewind
  - promote/demote
  - postgres start/stop/restart
  - fencing flow

## 4. HA Loop Integration Tests
- Real watchers + injected IO adapters.
- No mocks required for state channels.
- Deterministic stepping via `step_once`.
- Assert state transitions and dispatched actions.

## 5. E2E Scenario Matrix
- Cluster bootstrap from empty DCS.
- Leader promotion path.
- Controlled switchover.
- Unplanned primary failure failover.
- No quorum -> fail-safe behavior.
- Rejoin old primary via rewind.
- Fencing required before promotion.
- Split-brain prevention checks.

## 6. Security/Auth/TLS Tests
- Node-to-node auth validation.
- Client-to-server auth validation.
- TLS required mode tests.
- TLS optional/disabled mode tests.
- Invalid cert / expired cert / wrong CA failures.
- Role permission tests for API endpoints.

## 7. Parallel-Safe Harness Requirements
- Every test gets unique namespace id.
- Unique etcd data dir per test.
- Unique PGDATA dir per node per test.
- Dynamic free-port allocation per test.
- No shared socket directories.
- Cleanup on success and failure.

```rust
pub(crate) fn create_namespace(test_name: &str) -> Result<TestNamespace, HarnessError>;
pub(crate) fn cleanup_namespace(ns: TestNamespace) -> Result<(), HarnessError>;
pub(crate) fn allocate_ports(count: usize) -> Result<AllocatedPorts, HarnessError>;
pub(crate) fn prepare_pgdata_dir(ns: &TestNamespace, node_name: &str) -> Result<std::path::PathBuf, HarnessError>;
pub(crate) fn prepare_etcd_data_dir(ns: &TestNamespace) -> Result<std::path::PathBuf, HarnessError>;
pub(crate) async fn spawn_pg16(spec: PgInstanceSpec) -> Result<PgHandle, HarnessError>;
pub(crate) async fn spawn_etcd3(spec: EtcdInstanceSpec) -> Result<EtcdHandle, HarnessError>;
```

## Build Order
1. Core IDs/errors/time/watch wrapper.
2. Runtime config schema/defaults/validation.
3. Worker state enums and context structs.
4. Worker `run` + `step_once` signatures.
5. HA `decide` signature and exhaustive phase enum.
6. Wiring/bootstrap scaffold.
7. Unit tests for pure logic.
8. Real PG/etcd harness.
9. Integration + e2e + auth/TLS suites.



# More Future TODOS: create tasks
- Have opentelemetry ready logging system
  - All structured logs
  - Includes auto reader/parser of postgresql logs from its log dir and converts into structured logs
  - all structured logs include the member/host and where the logs came from (such as from pgtuskmaster, postgres, pg_rewind) etc..
  - log config inside app config: write to json files/write to opentelemetry only/write to stderr only in jsonl (default is always stderr)
  - tests includes full files of json postgres logs that are live parsed (since the parser reads them live and not directly all at once, but per flush of pg)
- Setup docker
  - all tests run in docker, no exceptions
  - docker builds are smart to reuse cache from previous builds

## HA Admin CLI (`pgtuskmasterctl`)

The project now includes a simple API-driven HA admin CLI binary:

- read state:
  - `pgtuskmasterctl ha state`
- set leader lease:
  - `pgtuskmasterctl --admin-token "$PGTUSKMASTER_ADMIN_TOKEN" ha leader set --member-id node-a`
- clear leader lease:
  - `pgtuskmasterctl --admin-token "$PGTUSKMASTER_ADMIN_TOKEN" ha leader clear`
- request switchover:
  - `pgtuskmasterctl --admin-token "$PGTUSKMASTER_ADMIN_TOKEN" ha switchover request --requested-by node-b`
- clear switchover:
  - `pgtuskmasterctl --admin-token "$PGTUSKMASTER_ADMIN_TOKEN" ha switchover clear`

Common global flags:

- `--base-url` (default `http://127.0.0.1:8008`)
- `--read-token` (or env `PGTUSKMASTER_READ_TOKEN`)
- `--admin-token` (or env `PGTUSKMASTER_ADMIN_TOKEN`)
- `--timeout-ms` (default `5000`)
- `--output json|text` (default `json`)

Exit codes:

- `0` success
- `2` usage/argument parsing failure
- `3` transport/timeouts
- `4` API non-2xx status
- `5` response decode/output serialization failure
