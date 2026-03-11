# Greenfield Story Task 5 Context Report

## Introduction

Executive summary.

Task 5 in the greenfield HA story did not change product code. Task 5 stopped after the new cucumber suite exposed a broad failure set and produced three refactor directions instead of one bug-by-bug patch plan. The three directions exist because the current HA implementation has one cluster of architectural mismatches, not one isolated defect. The current code can often explain why a scenario is red, but the current code does not yet give one clean place where quorum, authority, ranking, startup role choice, recovery visibility, and repeated-effect handling all agree with each other.

The report below is written from the current repository state, the task-5 review artifacts under `.ralph/tasks/story-greenfield-cucumber-ha-harness/artifacts/post-greenfield-ha-refactor-option-review/`, the runtime and HA source files, and the cucumber feature files plus step definitions. The goal is to explain the existing code first, then explain why task 5 proposed multiple options, then explain how each option changes the current logic.

One detail matters up front. The task-5 evidence inventory says the live `make test-long` run for the review stopped with `26 tests run: 11 passed, 15 failed`. Some older bug-task artifacts describe scenario families that are broader than the current 15 live failures. The report keeps both views visible. The live failing set tells you what is red now. The preserved bug tasks tell you why task 5 treated the problems as one design review rather than a narrow repair session.

## What Task 5 Actually Produced

Executive summary.

Task 5 produced a planning stop. Task 5 did not choose one final implementation. Task 5 wrote one evidence inventory, three option documents, and one comparison matrix. Task 5 recommended Option B, named "Unified authority snapshot and ranker", as the best balance between correctness and implementation risk. Task 5 recommended Option A, named "Authority layer convergence", as the second-best option when change size must stay smaller.

The important result from task 5 is not only "Option B is preferred". The important result is the diagnosis behind the options. The evidence inventory says the failures share six design pressures:

1. Quorum is still computed from the observed DCS member set instead of configured cluster membership.
2. Authority and trust are too coarse. Current code largely treats any state outside `FullQuorum` as one blunt fall into fail-safe behavior.
3. Leader and switchover target eligibility are not fully durability-ranked.
4. Startup role selection in `src/runtime/node.rs` does not fully reuse the same authority rules as steady-state HA.
5. Recovery and rejoin can become operator-visible before full queryable convergence.
6. `src/ha/worker.rs` still contains a worker-local dedup shortcut instead of making repeated effect dispatch safe through authoritative state.

Task 5 therefore did not present three unrelated solution ideas. Task 5 presented three ways to repair one broken authority model.

## From `pgtuskmaster` Process Start To The HA Loop

Executive summary.

The binary entrypoint is tiny. Almost all interesting behavior starts after configuration is loaded and `runtime::run_node_from_config_path` takes over. The runtime code has two big jobs. The first job is one-shot startup planning and startup execution. The second job is launching the long-running worker mesh. The HA loop lives in the second part, not in the first part.

The call chain at process start is short:

```rust
main
  -> run_node(cli)
  -> runtime::run_node_from_config_path(config_path)
  -> runtime::run_node_from_config(cfg)
  -> plan_startup(...)
  -> execute_startup(...)
  -> run_workers(...)
```

### The Binary Entrypoint

`src/bin/pgtuskmaster.rs` parses `--config`, builds a Tokio runtime, and calls `pgtuskmaster_rust::runtime::run_node_from_config_path`. If `--config` is missing, the binary exits with code `2`. If Tokio runtime creation fails or the async runtime returns an error, the binary exits with code `1`.

No HA logic lives in `src/bin/pgtuskmaster.rs`. `src/bin/pgtuskmaster.rs` is only the process launcher.

### Runtime Configuration Loading And Validation

`src/runtime/node.rs` starts by calling `load_runtime_config`, then `validate_runtime_config`, then logging bootstrap. Configuration parsing and validation live under `src/config/`. The runtime layer does not hand-parse TOML. The runtime layer assumes config parsing has already built a typed `RuntimeConfig`.

The runtime layer then derives `ProcessDispatchDefaults`. `ProcessDispatchDefaults` is a small bundle of recurring connection and process defaults used later by startup and by the HA process-dispatch layer. The bundle includes PostgreSQL host and port, socket directory, log file, replicator and rewinder usernames and auth settings, remote database name, remote SSL configuration, and shutdown mode.

### Startup Planning

Startup planning in `src/runtime/node.rs` happens before any long-running workers start. Startup planning asks one question: "given the current data directory and the DCS snapshot that is visible right now, what starting role intent is safe?"

Startup planning has three important internal concepts.

The first concept is `DataDirState`:

```rust
enum DataDirState {
    Missing,
    Empty,
    Existing,
}
```

`inspect_data_dir` decides which state applies. `Existing` means `PG_VERSION` is present. `Empty` means the directory exists and has no entries. `Missing` means the path does not exist. A non-empty directory without `PG_VERSION` is treated as an error, because the runtime cannot safely decide whether the directory is a valid cluster.

The second concept is the startup DCS probe. `probe_dcs_cache` connects to etcd, drains watch events, and reconstructs a `DcsCache`. The startup planner does not start long-running DCS watching here. The startup planner only tries to build one immediate cache snapshot.

The third concept is `StartupMode`:

```rust
enum StartupMode {
    InitializePrimary { start_intent },
    CloneReplica { leader_member_id, source, start_intent },
    ResumeExisting { start_intent },
}
```

`select_startup_mode` chooses among the three modes.

When the data directory is `Existing`, `select_startup_mode` calls `select_resume_start_intent`. `select_resume_start_intent` inspects the local managed recovery state on disk and compares the local member with the startup DCS cache. A local node can resume as primary, or resume as replica, or fail startup if local replica residue exists but authoritative replica source information is not available.

When the data directory is `Missing` or `Empty`, `select_startup_mode` first checks whether the cluster is already initialized. The DCS init lock controls the difference. If the init lock is absent and no leader is found, the runtime chooses `InitializePrimary`. If the init lock is present and a foreign healthy primary can be found, the runtime chooses `CloneReplica`. If the init lock is present and no healthy primary can be found, the runtime returns an error because cloning would have no source and bootstrapping would risk split cluster initialization.

The current startup planner does not call `ha::decide`. The current startup planner uses local helper logic in `src/runtime/node.rs`, such as `leader_from_leader_key`, `foreign_healthy_primary_member`, `resume_replica_source_member`, and `relaxed_leader_from_leader_key`. Task 5 called that difference out as one reason startup authority and steady-state authority drift apart.

### Startup Execution

After `plan_startup` picks one `StartupMode`, `execute_startup` turns the mode into a sequence of concrete actions.

The startup action enum is:

```rust
enum StartupAction {
    ClaimInitLockAndSeedConfig,
    RunJob(Box<ProcessJobKind>),
    StartPostgres(Box<ManagedPostgresStartIntent>),
    EnsureRequiredRoles,
}
```

`build_startup_actions` chooses the sequence.

For `InitializePrimary`, the sequence is:

1. claim the init lock and optionally seed DCS config
2. run `Bootstrap`
3. start PostgreSQL as primary
4. ensure required PostgreSQL roles exist

For `CloneReplica`, the sequence is:

1. run `BaseBackup`
2. start PostgreSQL as replica with managed recovery config

For `ResumeExisting`, the sequence is:

1. nothing, when `postmaster.pid` already exists
2. otherwise only `StartPostgres`

`run_startup_job` uses the process command builder from `src/process/worker.rs` and the job kinds from `src/process/jobs.rs`. Startup does not reuse the long-running process worker inbox. Startup runs jobs directly before the worker mesh exists.

`run_start_job` deserves special attention. `run_start_job` first checks `start_postgres_preflight_is_already_running`, then materializes managed PostgreSQL config with `crate::postgres_managed::materialize_managed_postgres_config`, then runs a `ProcessJobKind::StartPostgres` job. Replica start intent contains replicated connection info, auth, and managed recovery configuration. Primary start intent does not.

### The Worker Mesh

After startup succeeds, `run_workers` creates state channels and launches the long-running workers.

`run_workers` creates one versioned channel each for:

1. config
2. PostgreSQL observation state
3. DCS state
4. process state
5. HA state
6. debug snapshot state

The initial state values are conservative. PostgreSQL begins as `Unknown`, DCS begins as `NotTrusted`, process begins as `Idle` with no outcome, HA begins at `Init`, and debug snapshot begins with `AppLifecycle::Running` plus the current versions of the other channels.

`run_workers` then launches seven async workers together with `tokio::try_join!`:

1. `pginfo::worker::run`
2. `dcs::worker::run`
3. `process::worker::run`
4. `logging::postgres_ingest::run`
5. `ha::worker::run`
6. `debug_api::worker::run`
7. `api::worker::run`

The worker mesh is the real steady-state application. Startup is only preparation.

## The Existing Runtime Modules, In Plain Language

Executive summary.

The codebase is split by responsibility, and the split is mostly clean. `runtime` owns process boot and worker wiring. `pginfo` owns local PostgreSQL observation. `dcs` owns etcd-backed cluster cache and trust. `process` owns subprocess execution. `ha` owns decision logic. `debug_api` owns a combined snapshot. `api` owns the operator-facing HTTP surface. The important design gap is not confusion about module ownership. The important design gap is that some of the rules about authority are duplicated across the modules instead of being represented once.

### `src/runtime/node.rs`

`src/runtime/node.rs` is the orchestration root. `src/runtime/node.rs` loads config, validates config, boots logging, plans startup mode, executes startup actions, builds state channels, constructs worker contexts, and launches the workers.

`src/runtime/node.rs` also contains several authority-bearing helper functions. `leader_from_leader_key` trusts the DCS leader key only when the referenced member is a healthy primary. `foreign_healthy_primary_member` searches the cache for any healthy primary other than self. `resume_replica_source_member` is looser. `resume_replica_source_member` first tries a relaxed leader key, then falls back to the newest member record by `updated_at`. That fallback is one reason task 5 said startup authority and steady-state authority do not fully agree.

### `src/pginfo/state.rs`, `src/pginfo/query.rs`, and `src/pginfo/worker.rs`

The `pginfo` package turns local SQL polling into a typed local database state.

`src/pginfo/query.rs` performs the actual PostgreSQL poll. `src/pginfo/state.rs` defines the state types. `src/pginfo/worker.rs` runs the polling loop and publishes each new state.

`PgInfoState` has three variants:

```rust
enum PgInfoState {
    Unknown { common },
    Primary { common, wal_lsn, slots },
    Replica { common, replay_lsn, follow_lsn, upstream },
}
```

The shared `common` block carries worker status, SQL reachability, readiness, timeline, local config observations, and last refresh time. `SqlStatus` is `Unknown`, `Healthy`, or `Unreachable`. `Readiness` is `Unknown`, `Ready`, or `NotReady`.

The HA worker depends heavily on `pginfo` output. `DecisionFacts::from_world` asks whether PostgreSQL is reachable, whether the local node is primary, and whether WAL or replay position is available.

### `src/dcs/state.rs`, `src/dcs/store.rs`, `src/dcs/etcd_store.rs`, and `src/dcs/worker.rs`

The `dcs` package turns etcd watch activity plus local member publishing into a cluster cache and a trust result.

`DcsCache` contains member records, the leader record, an optional switchover request, the runtime config snapshot, and an optional init lock. `MemberRecord` is the cluster-facing projection of a node. `MemberRecord` contains member ID, PostgreSQL host and port, optional API URL, role, SQL status, readiness, timeline, WAL position, `updated_at`, and PostgreSQL version.

`DcsTrust` has three states:

```rust
enum DcsTrust {
    FullQuorum,
    FailSafe,
    NotTrusted,
}
```

`evaluate_trust` is the trust gate. `evaluate_trust` returns `NotTrusted` when the etcd store looks unhealthy. `evaluate_trust` returns `FailSafe` when etcd is reachable but self is missing, self is stale, or fresh quorum is missing. `evaluate_trust` returns `FullQuorum` only when the DCS view is healthy enough for ordinary HA behavior.

The critical detail is the fresh quorum rule in `has_fresh_quorum`. Current code uses a shortcut:

```rust
if cache.members.len() <= 1 {
    fresh_members == 1
} else {
    fresh_members >= 2
}
```

The shortcut means the quorum rule is based on observed DCS membership count, not configured cluster membership count. For a three-node configured cluster, a one-node observed view can still look like "single-member quorum" in some startup or degraded windows. Task 5 repeatedly pointed at this shortcut.

### `src/process/jobs.rs`, `src/process/state.rs`, and `src/process/worker.rs`

The `process` package is the subprocess engine. The HA layer does not call `pg_ctl`, `initdb`, `pg_basebackup`, or `pg_rewind` directly. The HA layer sends process job requests, and the process worker runs the external programs.

`ProcessJobKind` includes `Bootstrap`, `BaseBackup`, `PgRewind`, `Promote`, `Demote`, `StartPostgres`, and `Fencing`.

`ProcessState` is small:

```rust
enum ProcessState {
    Idle { worker, last_outcome },
    Running { worker, active },
}
```

`last_outcome` can be success, failure, or timeout, tagged with job kind and timestamps. The HA decision layer reads `ProcessState` to decide whether rewind is still running, whether bootstrap finished, whether fencing completed, and whether a recent start-postgres job succeeded.

`src/process/worker.rs` accepts one job at a time. If a job is already running, a new request is rejected as busy. `src/process/worker.rs` also contains preflight no-op logic for `StartPostgres` and `Fencing`. For example, starting PostgreSQL while PostgreSQL is already running becomes a no-op instead of a failing subprocess call.

### `src/ha/state.rs`

`src/ha/state.rs` defines the HA layer data model.

`HaPhase` is the main lifecycle enum:

```rust
Init
WaitingPostgresReachable
WaitingDcsTrusted
WaitingSwitchoverSuccessor
Replica
CandidateLeader
Primary
Rewinding
Bootstrapping
Fencing
FailSafe
```

`HaState` contains worker status, current phase, tick count, and the most recently chosen `HaDecision`.

`WorldSnapshot` packages the latest `RuntimeConfig`, `PgInfoState`, `DcsState`, and `ProcessState`. `DecideInput` is simply `{ current, world }`. `DecideOutput` is `{ next, outcome }`.

### `src/ha/decision.rs`

`src/ha/decision.rs` is not the phase transition machine. `src/ha/decision.rs` is the fact derivation and decision vocabulary layer.

`DecisionFacts::from_world` derives a normalized view from the `WorldSnapshot`. `DecisionFacts::from_world` computes:

1. self member ID
2. DCS trust
3. local PostgreSQL reachability
4. whether local PostgreSQL is primary
5. the last PostgreSQL observation time
6. raw leader member ID from the DCS leader key
7. active leader member ID, only when the leader is a fresh healthy primary
8. followable member ID, which falls back to any fresh healthy ready primary
9. whether a switchover is pending
10. the pending switchover target, if the request is targeted
11. the eligible switchover target set
12. whether self holds the leader key
13. whether some other leader record exists
14. whether some other leader is available
15. whether rewind is required
16. the current `ProcessState`

The decision vocabulary also lives here. `HaDecision` includes `NoChange`, `WaitForPostgres`, `WaitForDcsTrust`, `AttemptLeadership`, `FollowLeader`, `BecomePrimary`, `CompleteSwitchover`, `StepDown`, `RecoverReplica`, `FenceNode`, `ReleaseLeaderLease`, and `EnterFailSafe`.

`StepDownPlan`, `RecoveryStrategy`, and `LeaseReleaseReason` also live here. Those types are the payloads that drive lower layers.

### `src/ha/decide.rs`

`src/ha/decide.rs` is the pure phase transition engine.

The top-level `decide` function does only three things:

```rust
let facts = DecisionFacts::from_world(&input.world);
let outcome = decide_phase(&current, &facts);
let next = HaState { phase: outcome.next_phase, decision: outcome.decision, tick: current.tick + 1, ... };
```

`decide_phase` begins with the trust gate. If trust is not `FullQuorum`, current code does not run ordinary phase handlers. Current code immediately routes the node into `FailSafe`. A local primary gets `EnterFailSafe { release_leader_lease: false }`. A non-primary gets `NoChange` while the phase becomes `FailSafe`.

After the trust gate, `decide_phase` dispatches by phase:

1. `Init` moves to `WaitingPostgresReachable`.
2. `WaitingPostgresReachable` waits for PostgreSQL, or requests a PostgreSQL start when safe.
3. `WaitingDcsTrusted` decides whether to follow a leader, become primary without promotion, or attempt leadership.
4. `WaitingSwitchoverSuccessor` holds a stepping-down primary in place until a successor appears.
5. `Replica` follows the active leader, rewinds when needed, or attempts leadership.
6. `CandidateLeader` acquires leadership when no better follow target exists.
7. `Primary` may complete switchover, step down, release the lease because local PostgreSQL became unreachable, or keep trying to acquire leadership.
8. `Rewinding` reacts to rewind process outcomes.
9. `Bootstrapping` reacts to bootstrap or basebackup outcomes.
10. `Fencing` waits for fencing completion or enters fail-safe on fencing failure.
11. `FailSafe` can return toward primary or waiting states once trust and local authority look safe again.

The decision engine is pure and fairly readable. The problem is not that the phase code is impossible to follow. The problem is that the inputs to the phase code are not yet rich enough to capture the design that the new tests demand.

### `src/ha/lower.rs`

`src/ha/lower.rs` converts a semantic `HaDecision` into a bucketed `HaEffectPlan`.

The plan has five effect buckets:

```rust
struct HaEffectPlan {
    lease,
    switchover,
    replication,
    postgres,
    safety,
}
```

For example, `AttemptLeadership` becomes `lease: AcquireLeader`. `FollowLeader` becomes `replication: FollowLeader`. `BecomePrimary { promote: true }` becomes `postgres: Promote`. `StepDown` becomes a combination of demotion, optional lease release, and optional fencing.

`src/ha/lower.rs` is the point where intent stops being abstract and starts becoming executable.

### `src/ha/actions.rs`

`src/ha/actions.rs` names the executable action types. `HaAction` includes DCS actions, process actions, and a small amount of immediate filesystem work such as `WipeDataDir`.

`ActionId` gives each action a stable label. The process-dispatch layer uses the label when it builds job IDs such as `ha-scope-node-a-17-0-start_postgres`.

### `src/ha/process_dispatch.rs`

`src/ha/process_dispatch.rs` translates `HaAction` into process jobs or immediate filesystem mutations.

The important mappings are:

1. `StartPostgres` -> materialize managed PostgreSQL config -> `ProcessJobKind::StartPostgres`
2. `PromoteToPrimary` -> `ProcessJobKind::Promote`
3. `DemoteToReplica` -> `ProcessJobKind::Demote`
4. `StartRewind` -> validate rewind source -> `ProcessJobKind::PgRewind`
5. `StartBaseBackup` -> validate basebackup source -> `ProcessJobKind::BaseBackup`
6. `RunBootstrap` -> `ProcessJobKind::Bootstrap`
7. `FenceNode` -> `ProcessJobKind::Fencing`
8. `WipeDataDir` -> immediate directory delete and recreate

`src/ha/process_dispatch.rs` also owns source-member validation. `validate_rewind_source` and `validate_basebackup_source` resolve the leader member in the DCS cache, then call `src/ha/source_conn.rs` to build connection info.

`FollowLeader` and `SignalFailSafe` are explicitly skipped at the process layer. Those choices matter because not every HA decision implies a subprocess.

### `src/ha/source_conn.rs`

`src/ha/source_conn.rs` builds rewind and basebackup source connection info from `MemberRecord` plus `ProcessDispatchDefaults`.

`basebackup_source_from_member` and `rewind_source_from_member` both reject self-targeting and empty remote host values. Strict source selection also requires the remote member to be a healthy primary. Resume source selection is slightly looser because restarting a replica from an existing data directory sometimes needs source reconstruction without a fully healthy live primary.

### `src/ha/apply.rs`

`src/ha/apply.rs` applies a `HaEffectPlan` in a fixed order:

1. PostgreSQL effects
2. lease effects
3. switchover effects
4. replication effects
5. safety effects

The order matters. `apply_effect_plan` first emits intent and dispatch events, then routes DCS actions directly to DCS store operations and routes process actions to `dispatch_process_action`.

If any action dispatch fails, `apply_effect_plan` returns a list of `ActionDispatchError` values. The HA worker then republishes HA state with `WorkerStatus::Faulted`.

### `src/ha/events.rs`

`src/ha/events.rs` is the structured logging surface for the HA worker. The file emits decision-selected, plan-selected, action-intent, action-dispatch, action-result, phase-transition, role-transition, and lease-transition events.

`src/ha/events.rs` does not change authority behavior. `src/ha/events.rs` exists so the debug stream can explain what the HA loop thought it was doing at each tick.

### `src/ha/worker.rs`

`src/ha/worker.rs` is the steady-state orchestrator for the HA layer.

The long-running `run` function waits on any of four subscriber changes or on the poll interval. Every wake-up calls `step_once`.

`step_once` does five important things:

1. build a `WorldSnapshot`
2. call pure `decide`
3. lower the chosen `HaDecision` into a `HaEffectPlan`
4. publish the next `HaState`
5. apply the effect plan, unless the worker-local dedup rule suppresses process dispatch

The criticized dedup logic is:

```rust
current.phase == next.phase
    && current.decision == next.decision
    && decision_is_already_active(&next.decision, process_state)
```

The check only recognizes a small set of active jobs: `StartPostgres`, `PgRewind`, `BaseBackup`, `Bootstrap`, and `Fencing`. The check does not represent effect progress as HA state. The check is only a worker-local optimization branch. Task 5 wants repeated effect dispatch to be safe because the state model itself says an effect is already in flight, not because `src/ha/worker.rs` quietly short-circuits the apply path.

## The Steady-State State Model And The States The System Can Reach

Executive summary.

The system does not have one single global state enum. The system has several layers of state, each owned by a different worker. Understanding the code is much easier when the layers are kept separate. Local PostgreSQL state is not DCS trust. DCS trust is not HA phase. HA phase is not process state. Debug API snapshots simply bundle the layers into one view.

### Startup States

Startup does not publish a long-lived startup enum. Startup uses the private enums `DataDirState` and `StartupMode`, then runs a vector of `StartupAction` values.

The practical startup states are:

1. local data directory missing, empty, or existing
2. cluster not initialized, cluster initialized with healthy primary, or cluster initialized without healthy primary
3. start as new primary, clone as replica, or resume existing data directory

Startup also emits log phases such as `start`, but startup phases are not published into the HA state channel.

### PostgreSQL Observation States

`PgInfoState` says what the local PostgreSQL instance appears to be:

1. `Unknown`
2. `Primary`
3. `Replica`

`SqlStatus` says whether polling succeeded:

1. `Unknown`
2. `Healthy`
3. `Unreachable`

`Readiness` says whether the database is ready for operator use:

1. `Unknown`
2. `Ready`
3. `NotReady`

The HA layer reads these values as facts about the local node.

### DCS Trust States

`DcsTrust` has three values:

1. `FullQuorum`
2. `FailSafe`
3. `NotTrusted`

Current HA code treats only `FullQuorum` as normal operating authority. Every other trust state falls into `FailSafe` handling at the top of `decide_phase`.

### Process States

`ProcessState` has two values:

1. `Idle`
2. `Running`

When the process worker is `Idle`, the state can still remember the most recent outcome. The most recent outcome can be success, failure, or timeout, and the HA decision layer depends on the outcome kind. Rewind logic, bootstrap logic, fencing logic, and post-start waiting logic all read `last_outcome`.

### HA Phase States

The HA loop itself can be in these phases:

```text
Init
WaitingPostgresReachable
WaitingDcsTrusted
WaitingSwitchoverSuccessor
Replica
CandidateLeader
Primary
Rewinding
Bootstrapping
Fencing
FailSafe
```

The easiest way to read the phase meanings is:

`Init` means the worker has just started.

`WaitingPostgresReachable` means the HA loop does not yet trust local PostgreSQL enough to move on. The phase may request a PostgreSQL start.

`WaitingDcsTrusted` means local PostgreSQL is up enough to continue, but cluster authority is not yet resolved into follow or leader behavior.

`WaitingSwitchoverSuccessor` means a current primary is stepping down for switchover and is waiting for the successor to take over.

`Replica` means the node should follow a leader and remain non-authoritative.

`CandidateLeader` means the node sees no better follow target and is trying to acquire leadership.

`Primary` means the node believes it is, or should be, the authoritative primary.

`Rewinding` means the node is repairing a divergent data directory so the node can return as replica.

`Bootstrapping` means the node is rebuilding a data directory through bootstrap or basebackup.

`Fencing` means the node is forcing local PostgreSQL down because leadership is unsafe.

`FailSafe` means authority is withdrawn because trust is insufficient or local role safety is unclear.

### HA Decisions

A phase is not a decision. A phase says where the loop is. A decision says what the loop wants next.

The semantic decisions are:

```text
NoChange
WaitForPostgres
WaitForDcsTrust
AttemptLeadership
FollowLeader
BecomePrimary
CompleteSwitchover
StepDown
RecoverReplica
FenceNode
ReleaseLeaderLease
EnterFailSafe
```

The decision layer is what task 5 wants to preserve. All three task-5 options keep the `ha_loop` plus pure `decide()` direction. None of the options recommends replacing the functional decision shape with a mutable controller.

### Effect Buckets And Actions

After `HaDecision`, the next states are not more HA phases. The next layer is an effect plan. The effect plan splits into lease, switchover, replication, PostgreSQL, and safety buckets. The action layer then becomes:

```text
AcquireLeaderLease
ReleaseLeaderLease
ClearSwitchover
FollowLeader
StartRewind
StartBaseBackup
RunBootstrap
FenceNode
WipeDataDir
SignalFailSafe
StartPostgres
PromoteToPrimary
DemoteToReplica
```

Only some actions become subprocess jobs. Some actions go to DCS. Some actions are immediate filesystem work.

### Operator-Facing Snapshot States

The operator-facing API and debug API do not invent new authority rules. `src/api/controller.rs` maps the current debug snapshot into `HaStateResponse`. `src/debug_api/snapshot.rs` builds a `SystemSnapshot` containing app lifecycle, config, PostgreSQL state, DCS state, process state, and HA state.

The API therefore exposes, rather than computes, cluster state. The API can validate a targeted switchover request against the current snapshot, but the API does not own leader election.

## The Current HA Logic, Written As One Narrative

Executive summary.

A single HA tick begins with observed world state and ends with side effects. The HA worker does not directly scan Docker, PostgreSQL, or etcd. The HA worker reads the latest values already published by the other workers. The HA worker then runs one pure decision function, lowers the result, publishes the next HA state, and dispatches effects.

A rough tick looks like this:

```rust
world = latest(config, pg, dcs, process)
facts = DecisionFacts::from_world(world)
outcome = decide_phase(current_ha_state, facts)
plan = lower(outcome.decision)
publish(next_ha_state)
apply(plan)
```

The first large branch is trust. If DCS trust is not `FullQuorum`, the current code does not continue normal leader or replica reasoning. A local primary moves toward `FailSafe` with `EnterFailSafe`. A non-primary enters `FailSafe` with `NoChange`.

When trust is `FullQuorum`, phase logic begins. A node starts in `Init`, moves to `WaitingPostgresReachable`, then moves to `WaitingDcsTrusted` once PostgreSQL is reachable or a recent start-postgres job succeeded. From `WaitingDcsTrusted`, one of three broad paths opens.

When the DCS leader key already points at self, the node becomes `Primary` without running a promotion. The code assumes local PostgreSQL was already primary in that case.

When a follow target exists, the node becomes `Replica` and chooses `FollowLeader { leader_member_id }`.

When no follow target exists and local PostgreSQL is primary, the node becomes `CandidateLeader` and chooses `AttemptLeadership`.

Once inside `Replica`, the node can stay following the leader, rewind toward the leader, or attempt leadership if no usable leader exists. Once inside `CandidateLeader`, the node either becomes `Primary` after lease acquisition, falls back to following a leader that appeared, or continues trying to acquire leadership.

Once inside `Primary`, the node can stay primary, step down for switchover, fence because another leader has appeared, or release leadership because local PostgreSQL became unreachable. Recovery then flows through `Rewinding`, `Bootstrapping`, and back toward `Replica` or `WaitingPostgresReachable`.

The current code is therefore not random. The current code is structured. The problem exposed by greenfield HA is that the current structure still uses the wrong model of authority in a few crucial places.

## Scenario Inventory: Every Feature Scenario, One By One

Executive summary.

The repository currently contains twenty-six HA feature scenarios under `cucumber_tests/ha/features`. The task-5 evidence inventory says fifteen scenarios were red in the live `make test-long` run used for the design review. Eleven scenarios were green in the same run. A scenario being green does not mean every historical architectural concern vanished. A scenario being red does not always mean only one step can fail. Several red scenarios exposed more than one distinct symptom during task-5 investigation. In the red cases below, the report names the closest failing step from the current evidence.

### 1. `no_quorum_enters_failsafe`

Original scenario: `losing DCS quorum removes the operator-visible primary and exposes fail-safe behavior`

The scenario cuts DCS quorum while all three PostgreSQL nodes are still running. The scenario expects `pgtm primary` to stop resolving any writable primary and expects every running node's debug output to contain `fail_safe`.

The task-5 live run marked the scenario red. The task-5 evidence says the earliest visible failure has historically been `Then there is no operator-visible primary across 3 online node`. The deeper current failure class is often `And every running node reports fail_safe in debug output`. The pair of failures matches the current design gap in `src/dcs/state.rs` and the blunt trust gate in `src/ha/decide.rs`.

### 2. `replica_flap_keeps_primary_stable`

Original scenario: `repeatedly flapping a replica keeps the same primary`

The scenario repeatedly kills and restarts one replica while continuing writes through the original primary. The scenario expects no failover, the same primary every time, and eventual proof-row convergence across all three nodes.

The task-5 live run did not list the scenario in the failing set. The scenario currently looks green in the task-5 evidence run.

### 3. `custom_postgres_roles_survive_failover_and_rejoin`

Original scenario: `non-default replicator and rewinder roles survive failover and rejoin`

The scenario runs the same broad failover-and-rejoin story as a standard primary crash, but under a harness that uses non-default PostgreSQL role names. The scenario is validating wiring for role provisioning, managed recovery configuration, and reconnection behavior.

The task-5 live run did not list the scenario in the failing set. The scenario currently looks green.

### 4. `minority_old_primary_rejoins_safely_after_majority_failover`

Original scenario: `an old primary isolated into the minority rejoins only as a replica after the majority fails over`

The scenario isolates the old primary from the rest of the cluster, waits for the two-node majority to elect a new primary, writes on the majority, heals the network, and then expects the old primary to return only as a replica while primary history never includes the old primary after failover.

The task-5 live run marked the scenario red. The closest failing step from the current evidence is `Then exactly one primary exists across 2 running nodes as "majority_primary"`, because the evidence inventory says healthy two-node majorities can fail to elect a new primary. If majority election passes in some runs, later failure can move to `Then the node named "old_primary" rejoins as a replica`.

### 5. `two_node_loss_one_good_return_one_broken_return_recovers_service`

Original scenario: `one healthy return restores service even while another node stays broken`

The scenario kills both replicas, expects the lone survivor not to remain operator-visible as writable primary, restarts one healthy replica, intentionally blocks startup on the other replica, and expects degraded but operational two-node service before final full convergence.

The task-5 live run marked the scenario red. The closest failing step is one of the quorum-loss assertions, especially `Then the lone online node is not treated as a writable primary`. The same scenario can also slide into later failure at `Then exactly one primary exists across 2 running nodes as "restored_primary"` or at final proof-row convergence, but the first authority check is the sharpest match to the current evidence.

### 6. `primary_storage_stall_replaced_by_new_primary`

Original scenario: `a wedged primary is replaced without becoming authoritative again`

The scenario wedges the current primary, expects a different stable primary to appear, expects the wedged node never to regain primary status after the wedge marker, then expects the old primary to rejoin as a replica after the wedge is removed.

The task-5 live run marked the scenario red. The closest failing step is `Then I wait for a different stable primary than "initial_primary" as "final_primary"`. The evidence inventory explicitly says a wedged primary can stay authoritative instead of being replaced.

### 7. `full_partition_majority_survives_old_replica_isolated`

Original scenario: `an isolated replica does not self-promote while the majority preserves a single primary`

The scenario isolates one replica from the rest of the cluster while the original primary remains on the two-node majority. The scenario checks that the isolated replica does not self-promote and that proof rows still converge after healing.

The task-5 live run did not list the scenario in the failing set. The scenario currently looks green.

### 8. `full_partition_majority_survives_old_primary_isolated`

Original scenario: `a primary isolated into the minority is not accepted while the majority elects a new primary`

The scenario cuts the old primary into a one-node minority, tracks primary history, waits for a two-node majority primary, writes on the majority, heals the fault, and expects the old primary to rejoin as replica.

The task-5 live run marked the scenario red. The closest failing step is `Then exactly one primary exists across 2 running nodes as "majority_primary"`. The evidence inventory and preserved bug task both say a healthy majority can remain without any elected surviving primary.

### 9. `api_path_isolation_preserves_primary`

Original scenario: `observer api isolation of a non-primary does not trigger a failover`

The scenario isolates a non-primary from observer API access only. The scenario is validating that observer visibility loss for a replica does not look like a cluster leadership problem.

The task-5 live run did not list the scenario in the failing set. The scenario currently looks green.

### 10. `clone_failure_recovers_after_blocker_removed`

Original scenario: `a blocked basebackup clone path recovers after the blocker is removed`

The scenario blocks `pg_basebackup`, forces a recovery path that needs cloning, expects visible blocker evidence, then removes the blocker and expects the affected node to rejoin and converge.

The task-5 live run did not list the scenario in the failing set. The scenario currently looks green in the current live review run, even though task-5 evidence preserved an older bug task about clone paths becoming operator-visible before true queryability.

### 11. `rewind_failure_falls_back_to_basebackup`

Original scenario: `a rewind failure still allows the old primary to rejoin as a replica`

The scenario enables a `pg_rewind` blocker on the old primary, forces failover, then heals the old primary and expects blocker evidence for `pg_rewind` followed by successful replica rejoin and proof-row convergence.

The task-5 live run marked the scenario red. The closest failing step is `Then the node named "old_primary" emitted blocker evidence for "pg_rewind"`, because the evidence inventory says recovery can bypass the expected `pg_rewind` path entirely. If blocker evidence is present, failure can move to `And the node named "old_primary" rejoins as a replica`.

### 12. `targeted_switchover_rejects_ineligible_member`

Original scenario: `a targeted switchover request to a degraded replica is rejected`

The scenario fully isolates one replica from the cluster, attempts a targeted switchover to the isolated replica, expects an operator-visible error, expects the original primary to remain primary, then expects post-heal proof-row convergence.

The task-5 live run marked the scenario red. The closest failing step is `And I attempt a targeted switchover to "ineligible_replica" and capture the operator-visible error`. The step itself fails if the request unexpectedly succeeds. The evidence inventory explicitly says targeted switchover can still accept a fully isolated ineligible target.

### 13. `stress_planned_switchover_concurrent_sql`

Original scenario: `a planned switchover preserves single-primary behavior under concurrent writes`

The scenario combines a planned switchover with a bounded concurrent write workload. The scenario expects no dual-primary evidence, no split-brain write evidence, and final row convergence after switchover.

The task-5 live run did not list the scenario in the failing set. The scenario currently looks green.

### 14. `full_cluster_outage_restore_quorum_then_converge`

Original scenario: `the cluster comes back with two fixed nodes first, then converges after the final node returns`

The scenario kills all database nodes, restarts two fixed nodes, expects one restored primary across those two nodes, writes through the restored primary, then restarts the third node and expects the third node to rejoin as replica while `pgtm primary` continues to point at the restored primary.

The task-5 live run marked the scenario red. The closest failing step is `Then the node named "node-c" rejoins as a replica`. The evidence inventory says full-cluster restore can claim availability or authority before queryable convergence. In runs where replica identity appears first, the failure can slide to `And the 3 online nodes contain exactly the recorded proof rows`.

### 15. `no_quorum_fencing_blocks_post_cutoff_commits`

Original scenario: `fail-safe fencing eventually rejects post-cutoff writes and preserves pre-cutoff commits`

The scenario runs a concurrent workload, cuts DCS quorum, expects no operator-visible primary and explicit fail-safe reporting, restores DCS quorum, stops the workload, and then expects the workload evidence to show a clean rejection cutoff after which no writes committed.

The task-5 live run marked the scenario red. The earliest likely failing steps are `Then there is no operator-visible primary across 3 online node` and `And every running node reports fail_safe in debug output`, matching the same no-quorum authority issues as the simpler no-quorum scenario. If those pass, the later failing step becomes `Then the recorded workload evidence establishes a fencing cutoff with no later commits`.

### 16. `targeted_switchover_promotes_requested_replica`

Original scenario: `a targeted switchover promotes the chosen replica and not the other one`

The scenario chooses one replica as the requested target, asks for a targeted switchover, expects the requested replica to become the only primary, expects the other replica never to appear in primary history, and expects correct `pgtm primary` and `pgtm replicas` output.

The task-5 live run did not list the scenario in the failing set. The scenario currently looks green.

### 17. `lagging_replica_is_not_promoted`

Original scenario: `a degraded replica is not promoted during failover`

The scenario degrades one replica, forces failover, and expects the healthy replica to win while the degraded replica never appears in primary history.

The task-5 live run did not list the scenario in the failing set. The scenario currently looks green in the live review run, even though the task-5 evidence inventory preserved an older bug task that linked the scenario to durability-aware leader ranking problems.

### 18. `broken_replica_rejoin_does_not_block_healthy_quorum`

Original scenario: `a broken rejoin attempt does not destabilize the healthy primary`

The scenario kills one replica, records a marker, enables a `rejoin` blocker, restarts the broken replica while keeping the node marked unavailable, continues writes through the healthy primary, then removes the blocker and expects full three-node proof-row convergence.

The task-5 live run marked the scenario red. The closest failing step is the final convergence step, `Then the 3 online nodes contain exactly the recorded proof rows`. The evidence inventory says the cluster can remain unable to return to three online nodes even after the blocker is removed and the node is restarted.

### 19. `replica_outage_keeps_primary_stable`

Original scenario: `a replica outage keeps the current primary stable`

The scenario stops one replica without flapping it repeatedly. The scenario expects the primary to remain stable, continued write availability on the primary, and successful replica rejoin plus row convergence after restart.

The task-5 live run did not list the scenario in the failing set. The scenario currently looks green.

### 20. `repeated_leadership_changes_preserve_single_primary`

Original scenario: `repeated failovers preserve a single primary and distinct leaders when topology allows`

The scenario kills the first primary, waits for a second primary, restarts the first node as replica, cuts the first node off from DCS, kills the second primary, and then expects a third distinct primary with no dual-primary evidence.

The task-5 live run marked the scenario red. The closest failing step is `Then exactly one primary exists across 2 running nodes as "primary_c"`. The evidence inventory says repeated failovers can stall on stale leader state or stale leader lease handling. The later distinct-alias assertion can also fail when leadership churn reuses the wrong member.

### 21. `two_node_outage_one_return_restores_quorum`

Original scenario: `two replicas stop, then one returns and restores quorum`

The scenario kills both replicas, expects no operator-visible primary across the lone remaining node, restarts one replica, expects one restored primary across two nodes, then restarts the last replica and expects proof-row convergence.

The task-5 live run marked the scenario red. The closest failing step is again the no-quorum authority check, `Then there is no operator-visible primary across 1 online node`, with `And the lone online node is not treated as a writable primary` immediately behind it. The same design gap appears here as in the no-quorum scenarios.

### 22. `stress_failover_concurrent_sql`

Original scenario: `a forced failover preserves single-primary behavior under concurrent writes`

The scenario starts a workload, kills the old primary, waits for a new primary, stops the workload, checks for no dual-primary or split-brain write evidence, writes a post-failover proof row, and finally expects row convergence after the old primary rejoins as replica.

The task-5 live run marked the scenario red. The closest failing step is the final proof-row convergence assertion, first at `Then the 2 online nodes contain exactly the recorded proof rows` and sometimes later at `And the 3 online nodes contain exactly the recorded proof rows`. The evidence inventory says concurrent failover can lose acknowledged rows on surviving nodes.

### 23. `primary_crash_rejoin`

Original scenario: `a killed primary fails over and later rejoins as a replica`

The scenario is the original greenfield harness bootstrap scenario. The scenario validates primary failover, write continuity on the new primary, and safe replica rejoin for the old primary.

The task-5 live run did not list the scenario in the failing set. The scenario currently looks green.

### 24. `planned_switchover_changes_primary_cleanly`

Original scenario: `a planned switchover moves leadership to a different primary`

The scenario records the current `pgtm` view, requests a planned switchover, expects a different stable primary, expects the old primary to remain online as replica, expects `pgtm primary` and `pgtm replicas` to show the right members, then checks proof-row convergence.

The task-5 live run marked the scenario red. The closest failing step is `And the node named "old_primary" remains online as a replica`. The evidence inventory ties the scenario to the preserved bug "old primary stays unknown after planned switchover". In some runs the failure can move to `And pgtm replicas list every cluster member except "new_primary"`.

### 25. `postgres_path_isolation_replicas_catch_up_after_heal`

Original scenario: `replicas lag during replication-path isolation and catch up after heal`

The scenario isolates the primary from both replicas only on the PostgreSQL path, writes through the primary, verifies the isolated replicas do not yet have the new row, heals the fault, and expects final convergence.

The task-5 live run did not list the scenario in the failing set. The scenario currently looks green.

### 26. `mixed_network_faults_heal_converges`

Original scenario: `combined dcs and api faults still converge safely after heal`

The scenario cuts the current primary off from DCS, isolates a different replica from observer API access, expects the DCS-cut primary either to enter fail-safe or to lose primary authority safely, then heals all faults and expects exactly one final primary plus full proof-row convergence.

The task-5 live run marked the scenario red. The earliest failing step is often `Then the node named "initial_primary" enters fail-safe or loses primary authority safely`. The task-5 evidence also says later failure can appear at `Then exactly one primary exists across 3 running nodes as "final_primary"` when healed members disagree on the leader, or even later at proof-row convergence when the chosen primary is not truly queryable.

## Why The Existing Logic Produces Several Options Instead Of One Obvious Patch

Executive summary.

The current code already has a nice shape: pure decision function, effect lowering, separate worker responsibilities, and typed state. The current code does not need a total architectural replacement. The current code needs a better authority model. The three task-5 options differ mainly in how aggressively the authority model is centralized and typed.

Two examples show the problem clearly.

The first example is no-quorum handling. `src/dcs/state.rs` computes trust with the observed-member shortcut, while `src/ha/decide.rs` collapses every trust state outside `FullQuorum` into fail-safe behavior. Greenfield scenarios need more nuance than "trust fell, therefore fail-safe". Some nodes should keep following. Some nodes should keep enough orientation data to recover. Some nodes must fence and stop write authority immediately. Current code has only part of that vocabulary.

The second example is startup versus steady-state authority. Startup mode selection in `src/runtime/node.rs` uses helper-specific leader and source selection. Steady-state HA uses `DecisionFacts::from_world` plus the phase machine. When the two rule sets differ, startup can choose a role or source that the steady-state loop would not have chosen from the same facts. That mismatch shows up in rejoin, clone, and full-outage restore scenarios.

The three options from task 5 are therefore three levels of centralization:

1. centralize shared predicates
2. centralize a typed authority snapshot
3. centralize one larger authority machine that covers startup and steady state together

## Option A: Authority Layer Convergence

Executive summary.

Option A keeps the current package shape and the current broad HA phase structure. Option A mostly extracts shared authority predicates and shared ranking helpers so that `src/dcs/state.rs`, `src/runtime/node.rs`, `src/ha/decision.rs`, `src/ha/decide.rs`, and switchover validation all stop disagreeing with one another.

Under Option A, the startup planner still exists as a separate planner in `src/runtime/node.rs`. The HA phase machine still exists as a separate steady-state machine in `src/ha/decide.rs`. The main change is that both layers call the same authority helpers instead of carrying slightly different role and source rules.

Option A should change the logic in five places.

First, `src/dcs/state.rs` should stop using observed-member-count quorum and should compute majority from configured membership. For the three-node harness, the result becomes simple: authority for normal leader behavior needs two fresh configured members, not merely "one when the view size is one, otherwise two".

Second, `src/ha/decision.rs` should derive leader eligibility and switchover eligibility through one durability-aware ranking helper. Current `active_leader_member_id`, `followable_member_id`, and `eligible_switchover_targets` are mostly freshness, health, and readiness based. Option A should add stronger timeline and WAL-position reasoning without forcing a total state-model rewrite.

Third, `src/runtime/node.rs` should reuse the same authority helpers when selecting `InitializePrimary`, `CloneReplica`, and `ResumeExisting`. `resume_replica_source_member` and the relaxed leader fallback are exactly the kind of helper paths that task 5 wanted to tighten.

Fourth, `src/ha/decide.rs` should stop flattening every non-`FullQuorum` state into the same fail-safe response. Option A does not need a giant new state machine, but Option A does need at least one intermediate non-authoritative state that still preserves recovery orientation.

Fifth, `src/ha/worker.rs` should delete the worker-local dedup branch and move "same effect already in progress" reasoning into explicit plan or process-progress state. Option A still allows a small change here. Option A does not need the full Option B snapshot to remove the shortcut.

Scenario impact under Option A is broad but uneven. No-quorum scenarios such as `no_quorum_enters_failsafe`, `no_quorum_fencing_blocks_post_cutoff_commits`, `two_node_outage_one_return_restores_quorum`, and `two_node_loss_one_good_return_one_broken_return_recovers_service` should improve because majority semantics and richer trust facts are shared. Majority-partition scenarios such as `full_partition_majority_survives_old_primary_isolated` and `minority_old_primary_rejoins_safely_after_majority_failover` should improve because leader eligibility and followability are shared. Switchover scenarios such as `planned_switchover_changes_primary_cleanly` and `targeted_switchover_rejects_ineligible_member` should improve because the same ranking helper now decides both automatic and manual leadership changes.

Option A is weakest on the mixed-fault and deep recovery scenarios. `mixed_network_faults_heal_converges`, `full_cluster_outage_restore_quorum_then_converge`, `broken_replica_rejoin_does_not_block_healthy_quorum`, and `stress_failover_concurrent_sql` need more than shared predicates. Those scenarios need better explicit modeling of uncertainty, integration state, and in-flight action identity. Option A can help, but Option A can still leave conceptual duplication behind.

## Option B: Unified Authority Snapshot And Ranker

Executive summary.

Option B keeps the existing `ha_loop` and pure `decide()` shape, but Option B introduces one explicit immutable authority snapshot that both startup and steady-state code consume. Task 5 recommended Option B because Option B changes the inputs to the HA logic rather than only cleaning up helper functions around the edges.

Under Option B, the runtime would no longer decide from raw `DcsCache`, raw `PgInfoState`, and raw `ProcessState` in several slightly different ways. Instead, a new authority snapshot would normalize the facts first. The authority snapshot would say which members are authoritative, followable, recovering, ineligible, or unsafe. The authority snapshot would carry majority requirements, ranked candidates, recovery integration state, and effect-progress identity.

Option B should change the logic in the following way.

`src/dcs/state.rs` still publishes raw cluster cache and trust, but a new normalization step would project the cache into a richer authority view. `src/ha/decision.rs` would stop being only a loose fact gatherer and would become the home of the normalized authority snapshot plus the deterministic ranker. `src/runtime/node.rs` would select startup mode by asking the same authority snapshot family what kind of local role intent is safe. `src/ha/decide.rs` would then be simpler, because phase handlers would depend on already-normalized authority categories instead of reconstructing subtle rules each tick. `src/ha/worker.rs` would remove the dedup shortcut because the authority snapshot or adjacent HA state would explicitly remember effect identity and in-flight progress.

Option B directly addresses the current red scenarios more cleanly than Option A.

For `no_quorum_enters_failsafe` and `no_quorum_fencing_blocks_post_cutoff_commits`, Option B would separate "cannot grant write authority" from "must hard-fence immediately" and from "can still follow and converge". The separation is what the scenarios are asking for. Greenfield no-quorum scenarios want authority withdrawal, explicit fail-safe visibility when required, and clean recovery afterward.

For majority partition scenarios such as `full_partition_majority_survives_old_primary_isolated`, `minority_old_primary_rejoins_safely_after_majority_failover`, and `repeated_leadership_changes_preserve_single_primary`, Option B would use one durability-aware ranker for automatic failover and for post-heal reintegration. The ranker would not only choose a primary candidate. The ranker would also classify the old primary as "must rejoin behind the winner" rather than "still plausible authority".

For switchover scenarios such as `planned_switchover_changes_primary_cleanly`, `targeted_switchover_rejects_ineligible_member`, and `targeted_switchover_promotes_requested_replica`, Option B would use one shared eligibility view for both operator-requested and automatic leadership moves. Current code already partially does that through `eligible_switchover_targets`, but Option B would make the relation first-class.

For recovery scenarios such as `full_cluster_outage_restore_quorum_then_converge`, `broken_replica_rejoin_does_not_block_healthy_quorum`, `rewind_failure_falls_back_to_basebackup`, and `clone_failure_recovers_after_blocker_removed`, Option B would explicitly model integration state. A node would not become operator-visible as rejoined merely because a process finished or a member record reappeared. A node would become integrated only after readiness and queryability checks passed against the authority winner.

For concurrency and data safety scenarios such as `stress_failover_concurrent_sql` and `stress_planned_switchover_concurrent_sql`, Option B would make in-flight effect identity and rank reasoning explicit. The HA loop could then avoid both duplicate effect dispatch and stale-leader churn without leaning on a worker-local shortcut.

Option B is larger than Option A, but Option B is still incremental enough to fit the existing codebase. The phase machine stays. The worker split stays. The pure decision style stays. The improvement is in the data model, not in a wholesale mutation-heavy rewrite.

## Option C: Single HA Authority Machine

Executive summary.

Option C goes furthest. Option C turns startup, steady-state authority, recovery, and rejoin into one larger authority machine. Startup would no longer live as a separate one-shot planner in `src/runtime/node.rs`. Startup would become the earliest part of the HA authority lifecycle.

Option C should change the logic most radically.

`src/runtime/node.rs` would stop owning startup role choice. `src/ha/state.rs` would grow a larger phase vocabulary that includes startup and integration stages. `src/ha/decide.rs` would own transitions across bootstrapping, startup-resume, follow, lead, fence, and rejoin, all inside one typed transition system. `src/ha/worker.rs` would no longer need the dedup shortcut because in-flight action state would be part of the machine. `src/ha/lower.rs` and `src/ha/apply.rs` would remain as effect layers, but the machine would decide more of the lifecycle directly.

Option C would probably produce the cleanest final design. All of the red scenarios from task 5 would have one natural home because there would be one lifecycle model for authority from the first boot through final steady-state. Mixed-fault handling, rejoin visibility, stale leader churn, and startup-versus-steady-state drift would all be addressed by construction.

The downside is migration cost. Option C touches the largest number of files, creates the most test churn, and raises the highest short-term risk of destabilizing currently green scenarios such as `primary_crash_rejoin`, `replica_outage_keeps_primary_stable`, `postgres_path_isolation_replicas_catch_up_after_heal`, and `targeted_switchover_promotes_requested_replica`. Option C is architecturally credible, but Option C is the easiest option to overbuild.

## What Will Change In The Current Logic Under Each Option

Executive summary.

The easiest way to compare the options is to follow the current code path from process start into the HA loop and ask what each option changes at each stage.

### Startup Selection In `src/runtime/node.rs`

Option A keeps startup selection where it is and replaces the helper rules underneath it. `select_startup_mode`, `select_resume_start_intent`, and `resume_replica_source_member` would keep existing shape but would call a shared authority helper library.

Option B keeps startup selection in `src/runtime/node.rs`, but startup selection would read from a normalized authority snapshot instead of raw cache heuristics. Startup mode would become much closer to a pure projection of the same authority model that steady-state HA uses.

Option C removes startup role choice as a separate logic island. Startup becomes part of the HA machine.

### DCS Trust In `src/dcs/state.rs`

Option A changes quorum semantics and adds richer trust predicates, but still leaves trust as a DCS-owned concept that the HA layer interprets.

Option B keeps `DcsTrust` but treats `DcsTrust` as only one input to a richer authority snapshot. The richer authority snapshot distinguishes "non-authoritative but still orientable" from "must fail-safe".

Option C still needs trust facts, but the HA machine consumes trust as transition guards instead of as a coarse gate outside the machine.

### Fact Derivation In `src/ha/decision.rs`

Option A expands shared predicate helpers and ranking helpers around the current `DecisionFacts`.

Option B replaces `DecisionFacts` with a more explicit authority snapshot and candidate ranker.

Option C likely absorbs most of `DecisionFacts` into a much larger phase-transition domain model.

### Phase Logic In `src/ha/decide.rs`

Option A mostly edits branches. Option A teaches the current phase machine better answers using stronger shared helpers.

Option B simplifies many branches because the branch inputs become richer. Current code often asks several small questions like "is there an active leader", "is there a followable member", and "is rewind required". Option B would package those facts into stronger authority categories before branching.

Option C changes the branch structure itself because startup and rejoin would become first-class machine phases.

### Effect Dispatch In `src/ha/worker.rs`, `src/ha/lower.rs`, `src/ha/apply.rs`, and `src/ha/process_dispatch.rs`

Option A can delete the worker-local dedup shortcut by introducing explicit effect identity near apply or process-progress tracking.

Option B deletes the worker-local dedup shortcut more naturally because the authority snapshot or adjacent HA state can explicitly say "the same effect is already in flight". Repeated ticks then stay pure and safe.

Option C deletes the worker-local dedup shortcut as a natural consequence of machine-owned in-flight action state.

## Which Option Best Matches The Scenarios

Executive summary.

Option A is strong when the main problem is drift between helpers. Option B is strong when the main problem is the shape of the facts that the code reasons over. Option C is strongest in theory, but Option C is largest in practice. The live task-5 failure set mostly points at Option B, because the failures are spread across startup, majority election, switchover, mixed-fault handling, rejoin visibility, and repeated leadership churn at the same time.

Option A is likely good enough for the simpler no-quorum cases, some switchover validation, and some majority-election fixes. Option A is less convincing for `mixed_network_faults_heal_converges`, `full_cluster_outage_restore_quorum_then_converge`, `broken_replica_rejoin_does_not_block_healthy_quorum`, and `stress_failover_concurrent_sql`.

Option B most directly covers the whole red set because the whole red set is really one request: "make authority explicit, make candidate ranking deterministic, make startup and steady-state agree, and do not report recovered service too early." That request is almost a definition of Option B.

Option C is the cleanest theoretical destination, but Option C creates the biggest chance of breaking currently green flows while the machine is being reshaped. Option C makes more sense as either a longer follow-on target or as a later evolution if Option B still leaves too much fragmentation behind.

## Plain-Language Recommendation

Executive summary.

The task-5 comparison matrix recommended Option B first and Option A second. The current code reading supports the same conclusion.

Option B should be the preferred direction when the team wants the new greenfield feature set to become the normal way of thinking about HA behavior. Option B preserves the current strengths of the codebase. The current strengths are the worker split, the pure `decide()` style, and the explicit effect-lowering path. Option B fixes the weakest part, which is the authority model flowing into those layers.

Option A is still worth keeping in mind as a lower-risk entry path when implementation time or review bandwidth is tight. Option A can produce useful improvements quickly. Option A simply leaves more pressure for later.

Option C is real, but Option C is a larger commitment than the current evidence requires for a first landing.

## Final Summary

Executive summary.

`pgtuskmaster` starts as a simple CLI binary, hands off to `src/runtime/node.rs`, chooses a startup mode from local disk state plus a startup DCS probe, performs one-shot startup actions, then launches a mesh of long-running workers. Inside the steady-state mesh, `pginfo` describes the local PostgreSQL instance, `dcs` describes cluster membership and trust, `process` owns subprocesses, `ha` owns the phase machine and side-effect intent, `debug_api` bundles the full state, and `api` exposes operator control and status.

The HA code already has a disciplined shape. `ha::worker` builds a world snapshot, `ha::decision` derives normalized facts, `ha::decide` selects a semantic decision, `ha::lower` turns the decision into effect buckets, and `ha::apply` dispatches actions. The new greenfield feature files are difficult to grok mainly because the existing code spreads authority rules across startup, DCS trust, steady-state election, switchover validation, recovery visibility, and a small worker-local dedup shortcut. Task 5 produced multiple options because the feature failures are symptoms of the same spread, not because the task could not decide what problem it was looking at.

If you keep one picture in mind while reading the source, keep the following picture in mind:

```text
process start
  -> runtime startup mode selection
  -> startup subprocess actions
  -> worker mesh starts
  -> pginfo publishes local SQL state
  -> dcs publishes cluster cache and trust
  -> process publishes active job state
  -> ha builds world snapshot
  -> decide() chooses next HA phase and decision
  -> lower() builds effect plan
  -> apply() dispatches DCS/process/filesystem actions
  -> api/debug expose the result
```

The three task-5 options are three ways to make the middle of that pipeline speak one consistent authority language.
