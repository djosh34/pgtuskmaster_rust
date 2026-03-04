# HA Decision and Action Pipeline

The HA pipeline is the architectural core: it is where the system decides **what role this node should be in**, and **which side effects must happen next** to converge to a safe lifecycle state.

This chapter is intentionally concrete. It describes:

- what inputs HA reads (and from where)
- what the decision function produces (phase + actions)
- how actions are dispatched (DCS writes/deletes vs process job requests)
- how outcomes feed back into the next decision tick.

## Decision path

The canonical loop lives in `src/ha/worker.rs`:

1. Collect a **world snapshot** from input state channels (config/pg/dcs/process).
2. Run pure decision logic: `ha::decide::decide(DecideInput { current, world })`.
3. Dispatch side effects for the chosen actions:
   - DCS writes/deletes (coordination)
   - process job requests (local side effects).
4. Publish the updated `HaState` (phase + tick + pending actions + worker status).

The decision function itself is in `src/ha/decide.rs`. It is intentionally structured to be testable as a pure function: given “current state” and “world view”, it returns “next state” and an action list.

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
- a dispatch bug (ha/worker)
- a process execution bug (process).

## Outputs: `HaState` and action idempotency

The HA worker publishes a `HaState` which includes:

- `phase`: the lifecycle phase (state machine node)
- `tick`: an incrementing counter
- `pending`: the actions selected for this tick
- `recent_action_ids`: a set used to suppress repeated actions.

Action idempotency is currently implemented as: if the action’s `ActionId` already exists in `recent_action_ids`, it is not re-emitted on future ticks. This is a deliberate “don’t spam side effects” guard, but it also means contributors must think carefully about whether an action should be:

- “fire once” (lease acquire attempt), or
- “retryable” (start postgres when unreachable).

If you change action semantics, update the idempotency behavior and tests together.

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

This split is enforced in `src/ha/worker.rs` in `dispatch_actions(...)`.

### Action-to-side-effect matrix

The table below is a practical “what will happen if HA chooses action X?” guide. It matches the current dispatch implementation.

| HA action | Side effect category | Dispatch path |
|---|---|---|
| `AcquireLeaderLease` | DCS write | write leader lease to `/{scope}/leader` |
| `ReleaseLeaderLease` | DCS delete | delete `/{scope}/leader` |
| `ClearSwitchover` | DCS delete | delete `/{scope}/switchover` |
| `StartPostgres` | process job | enqueue `ProcessJobKind::StartPostgres` (managed settings materialized first) |
| `PromoteToPrimary` | process job | enqueue `ProcessJobKind::Promote` |
| `DemoteToReplica` | process job | enqueue `ProcessJobKind::Demote` |
| `StartRewind` | process job | enqueue `ProcessJobKind::PgRewind` |
| `RunBootstrap` | process job | enqueue `ProcessJobKind::Bootstrap` |
| `FenceNode` | process job | enqueue `ProcessJobKind::Fencing` |
| `FollowLeader { .. }` | coordination-only (in this task) | no dispatch side effects in `dispatch_actions` |
| `SignalFailSafe` | coordination-only (in this task) | no dispatch side effects in `dispatch_actions` |

Notes:

- The leader and switchover keys are scoped to `cfg.dcs.scope` and rendered as `/{scope}/leader` and `/{scope}/switchover`.
- `StartPostgres` dispatch calls `postgres_managed::materialize_managed_postgres_config(...)` before enqueuing the job, so that managed `postgresql.conf`-style settings are applied consistently in both startup and HA-driven starts.

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
- if there is no available leader, enters `CandidateLeader` and emits `AcquireLeaderLease`
- if it observes itself as leader, enters `Primary` and emits `PromoteToPrimary`.

The “is a leader available?” check is intentionally conservative: it considers both the leader key and member metadata health.

### Switchover request path (API → DCS → HA)

The operator-facing switchover request is an intent written by the API into `/{scope}/switchover`.

The DCS worker observes that key and publishes it as part of `DcsCache`.

HA reads the cache and, if it is currently primary and sees the request, it transitions to replica and emits:

- `DemoteToReplica`
- `ReleaseLeaderLease`
- `ClearSwitchover`.

This is a good example of the “write intent, then react via read model” pattern that keeps control flow explicit.

### Fencing / split-brain path

Split-brain signal in the current implementation is: this node believes it is primary, but the DCS leader record points to another member.

In that case, HA transitions into `Fencing` and emits:

- `DemoteToReplica` (stop being primary)
- `ReleaseLeaderLease` (stop advertising leadership)
- `FenceNode` (enforce a fail-safe local shutdown/fence action).

### Rewind and recovery path

When the node is primary but Postgres becomes unreachable, HA moves to `Rewinding` and emits `StartRewind`.

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
