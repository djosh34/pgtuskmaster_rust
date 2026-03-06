# HA Decision and Effect-Plan Pipeline

The HA pipeline is the architectural core: it is where the system decides **what role this node should be in**, and **which side effects must happen next** to converge to a safe lifecycle state.

This chapter is intentionally concrete. It describes:

- what inputs HA reads (and from where)
- what the decision function produces (phase + domain decision)
- how the domain decision lowers into a typed effect plan
- how effect buckets are dispatched (DCS writes/deletes vs process job requests)
- how outcomes feed back into the next decision tick.

## Decision path

The canonical loop lives across a small HA pipeline:

1. `src/ha/worker.rs` collects a **world snapshot** from input state channels (config/pg/dcs/process).
2. `src/ha/decide.rs` runs pure decision logic: `ha::decide::decide(DecideInput { current, world })`.
3. `src/ha/lower.rs` lowers the chosen `HaDecision` into a typed `HaEffectPlan`.
4. `src/ha/apply.rs` dispatches the effect buckets in deterministic order.
5. `src/ha/worker.rs` publishes the updated `HaState` (phase + tick + decision + worker status) and emits phase/role transition events.

The decision function itself is in `src/ha/decide.rs`. It is intentionally structured to be testable as a pure function: given “current state” and “world view”, it returns a `PhaseOutcome { next_phase, decision }`. Effect-plan lowering lives separately in `src/ha/lower.rs`, `src/ha/apply.rs` owns effect application, `src/ha/process_dispatch.rs` owns local process request construction plus managed-config/filesystem preparation, and `src/ha/events.rs` owns the repetitive HA event payload construction.

Contributor test guidance follows the same split:

- `src/ha/decide.rs` should prove exact `DecisionFacts -> PhaseOutcome` mappings and lowered-plan invariants with immutable test builders.
- `src/ha/worker.rs` should prove `step_once(...)` publishes the same decision-selected state and dispatches the matching side effects for a given snapshot.
- `tests/ha_multi_node_*.rs` and `tests/ha_partition_*.rs` should keep scenario driving separate from continuous invariant observation; the observer window in `tests/ha/support/observer.rs` must cover the fault sequence, not only the final convergence point.

## Inputs: what HA reads

HA does not probe the world directly. It reads the latest snapshots:

- local Postgres health and role evidence from `PgInfoState` (`src/pginfo/state.rs`)
- DCS cache + trust from `DcsState` (`src/dcs/state.rs`)
- the outcome of the last process job from `ProcessState` (`src/process/state.rs`)
- runtime config from `RuntimeConfig` (`src/config/schema.rs` and `src/config/parser.rs`).

That separation matters: when HA is wrong, you can usually locate the failure to one of:

- an incorrect observation (pginfo)
- an incorrect cache/trust view (dcs)
- a decision bug (ha/decide)
- an apply/dispatch bug (`ha/apply.rs` or `ha/process_dispatch.rs`)
- a process execution bug (process).

## Outputs: `HaState`, `HaDecision`, and the lowered effect plan

The HA worker publishes a `HaState` which includes:

- `phase`: the lifecycle phase (state machine node)
- `tick`: an incrementing counter
- `decision`: the domain-level HA decision selected for this tick

Before dispatch, `HaDecision::lower()` converts that decision into a fixed-shape `HaEffectPlan` with explicit buckets for:

- lease effects
- switchover cleanup
- replication/recovery work
- Postgres lifecycle
- safety/fencing.

The worker dispatches those buckets in deterministic order rather than authoring an append-only action vector. There is intentionally **no cross-tick suppression**. If the world snapshot does not change (for example Postgres remains unreachable), the same effect plan can be re-issued on every tick until it succeeds.

If you change decision or lowering semantics, update both layers and tests together. The invariants are covered by:

- decision-level tests in `src/ha/decide.rs`
- lowering tests in `src/ha/lower.rs`
- apply-layer tests in `src/ha/apply.rs`, `src/ha/process_dispatch.rs`, and `src/ha/worker.rs`
- continuous-observer HA scenario tests in `tests/ha_multi_node_*.rs`, `tests/ha_partition_*.rs`, and `tests/ha/support/observer.rs`

## The core phases (what they mean)

The phase enum is defined in `src/ha/state.rs` and transitions are decided in `src/ha/decide.rs`.

In the current implementation, the phases cover:

- `Init` → `WaitingPostgresReachable` → `WaitingDcsTrusted`: early convergence before the node can make HA decisions.
- `Replica`, `CandidateLeader`, `Primary`: the steady-state leader election and follow/promotion loop.
- `Rewinding`, `Bootstrapping`: recovery paths when Postgres is unhealthy or history diverged.
- `Fencing`: deliberate shutdown/fencing when split-brain is detected.
- `FailSafe`: “no quorum” mode (do not attempt to act as primary without full trust).

The exact transition conditions are encoded directly in `decide(...)`; contributor docs should reference the code rather than inventing extra semantics.

## How a decision becomes side effects

There are two side-effect boundaries:

1. **DCS writes/deletes**: done directly by HA via a DCS store handle.
2. **Local process jobs**: enqueued to the process worker via an inbox channel.

This split is enforced by `src/ha/apply.rs`:

- `apply_effect_plan(...)` owns bucket sequencing and DCS coordination calls.
- `process_dispatch.rs` owns local process job construction and filesystem preparation.
- `events.rs` owns decision/plan/action/lease event payload helpers.

### Effect-plan dispatch matrix

The table below is a practical “what will happen if HA chooses effect X?” guide. It matches the current dispatch implementation.

| Effect bucket | Effect variant | Side effect category | Dispatch path |
|---|---|---|
| lease | `AcquireLeader` | DCS write | write leader lease to `/{scope}/leader` |
| lease | `ReleaseLeader` | DCS delete | delete `/{scope}/leader` |
| switchover | `ClearRequest` | DCS delete | delete `/{scope}/switchover` |
| postgres | `Start` | process job | enqueue `ProcessJobKind::StartPostgres` after materializing `PGDATA/pgtm.postgresql.conf` and the managed side files |
| postgres | `Promote` | process job | enqueue `ProcessJobKind::Promote` |
| postgres | `Demote` | process job | enqueue `ProcessJobKind::Demote` |
| replication | `RecoverReplica::Rewind { .. }` | process job | enqueue `ProcessJobKind::PgRewind` |
| replication | `RecoverReplica::BaseBackup { .. }` | filesystem + process job | wipe data dir, then enqueue `ProcessJobKind::BaseBackup` |
| replication | `RecoverReplica::Bootstrap` | filesystem + process job | wipe data dir, then enqueue `ProcessJobKind::Bootstrap` |
| replication | `FollowLeader { .. }` | coordination-only (current task) | no process job; state remains explicit in the plan/logs |
| safety | `FenceNode` | process job | enqueue `ProcessJobKind::Fencing` |
| safety | `SignalFailSafe` | coordination-only (current task) | no process job; emits fail-safe intent in logs/state |

Notes:

- The leader and switchover keys are scoped to `cfg.dcs.scope` and rendered as `/{scope}/leader` and `/{scope}/switchover`.
- `StartPostgres` dispatch calls `postgres_managed::materialize_managed_postgres_config(...)` before enqueuing the job, so the authoritative managed config file `PGDATA/pgtm.postgresql.conf`, the managed signal-file set (`standby.signal`, `recovery.signal`, or neither), and the managed side files are regenerated consistently in both startup and HA-driven starts.
- The same managed-postgres surface also owns quarantining `PGDATA/postgresql.auto.conf` to `PGDATA/pgtm.unmanaged.postgresql.auto.conf` on managed starts. HA and startup planning do not treat `postgresql.auto.conf` as authoritative configuration.
- When HA needs to preserve previously managed replica follow state without a fresh DCS leader hint, it reuses the shared managed-state reader from `postgres_managed` instead of parsing ad hoc local PostgreSQL files in multiple places.
- The current dispatch order is fixed by concern inside `apply_effect_plan(...)`: Postgres lifecycle, then lease effects, then switchover cleanup, then replication/recovery, then safety. That order is what enforces “demote before release” and “release before clear/signal” without preserving a generic action vector boundary.

## Phase transitions: the “story” paths contributors care about

This section describes the main “stories” that show up in debugging and tests.

### Bootstrap path (first node initializes)

Bootstrap is split across startup and HA:

- Startup (`runtime/node.rs`) decides whether this node should initialize based on:
  - data dir state (`inspect_data_dir`)
  - DCS probe snapshot (`probe_dcs_cache`)
  - init lock presence (`/{scope}/init`).
- If it is the first node, startup claims the init lock and can seed config (if configured), then runs a bootstrap process job before starting Postgres.

After workers start, HA converges from `Init` through “waiting” phases into `CandidateLeader` / `Primary` depending on leader lease outcomes.

### Election and promotion path

In the steady-state path, HA:

- waits for DCS trust (`DcsTrust::FullQuorum`) and Postgres reachability
- if there is no available leader, enters `CandidateLeader` and lowers to `LeaseEffect::AcquireLeader`
- if it observes itself as leader, enters `Primary` and lowers to `PostgresEffect::Promote`.

The “is a leader available?” check is intentionally conservative: it considers both the leader key and member metadata health.

### Switchover request path (API → DCS → HA)

The operator-facing switchover request is an intent written by the API into `/{scope}/switchover`.

The DCS worker observes that key and publishes it as part of `DcsCache`.

HA reads the cache and, if it is currently primary and sees the request, it transitions to replica and emits:

- `PostgresEffect::Demote`
- `LeaseEffect::ReleaseLeader`
- `SwitchoverEffect::ClearRequest`.

This is a good example of the “write intent, then react via read model” pattern that keeps control flow explicit. Publication-time phase and role transition logs still stay in `worker.rs` because they describe the published state boundary, not the apply boundary.

### Fencing / split-brain path

Split-brain signal in the current implementation is: this node believes it is primary, but the DCS leader record points to another member.

In that case, HA transitions into `Fencing` and emits:

- `PostgresEffect::Demote` (stop being primary)
- `LeaseEffect::ReleaseLeader` (stop advertising leadership)
- `SafetyEffect::FenceNode` (enforce a fail-safe local shutdown/fence action).

### Rewind and recovery path

When the node is primary but Postgres becomes unreachable, HA moves to `Rewinding` and lowers to `ReplicationEffect::RecoverReplica { strategy: Rewind { .. } }`.

Subsequent phase transitions depend on the process worker’s last outcome:

- rewind success → go to `Replica` and (optionally) `FollowLeader`
- rewind failure → attempt `Bootstrapping` as a recovery mechanism
- repeated failures can lead to `Fencing` or `FailSafe` depending on trust.

## Failure behavior and observability

Two failure classes matter:

- **Decision failure**: `decide(...)` returns an error → the HA worker returns a `WorkerError` and the whole node runtime can fail (because runtime uses `try_join`).
- **Dispatch failure**: `decide(...)` succeeded but side effects could not be dispatched → HA still publishes a next state but marks `worker = Faulted(...)`.

This is intentional: dispatch failures are “soft errors” that should be observable and recoverable without panicking the whole process.

## Adjacent subsystem connections

The HA loop is only meaningful in context:

- Read [Worker Wiring and State Flow](./worker-wiring.md) for the upstream snapshot channels HA consumes and the downstream feedback loop from process outcomes.
- Read [API and Debug Contracts](./api-debug-contracts.md) for how external intent (switchover) is written and how client-visible state is derived from the debug snapshot.
- Read [Harness Internals](./harness-internals.md) for how real-binary e2e tests construct multi-node scenarios that exercise these transitions.
