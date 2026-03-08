# HA State Machine Reference

The HA state machine implements a deterministic decision loop that translates observed cluster state into coordinated DCS and local process actions. The pipeline is `observe -> decide -> lower -> dispatch`.

## Overview

`HaWorkerCtx` contains the execution context for the HA worker loop. `step_once` drives each iteration by building a `WorldSnapshot`, calling `decide`, lowering the decision into an `HaEffectPlan`, and dispatching effects.

## Phase Model

`HaPhase` enumerates the possible worker states:

| Variant | Description |
|---|---|
| `Init` | Initial startup state |
| `WaitingPostgresReachable` | Awaiting PostgreSQL connectivity |
| `WaitingDcsTrusted` | Awaiting DCS quorum trust |
| `WaitingSwitchoverSuccessor` | Awaiting switchover completion |
| `Replica` | Operating as a replica |
| `CandidateLeader` | Eligible to become leader |
| `Primary` | Operating as primary |
| `Rewinding` | Executing rewind recovery |
| `Bootstrapping` | Executing bootstrap recovery |
| `Fencing` | Executing fencing operations |
| `FailSafe` | Operating in fail-safe mode |

## World Snapshot and Decision Inputs

`WorldSnapshot` aggregates versioned subsystem states:

| Field | Type |
|---|---|
| `config` | `Versioned<RuntimeConfig>` |
| `pg` | `Versioned<PgInfoState>` |
| `dcs` | `Versioned<DcsState>` |
| `process` | `Versioned<ProcessState>` |

`DecisionFacts` derives observational truth from `WorldSnapshot`:

| Group | Fields |
|---|---|
| identity and trust | `self_member_id`, `trust` |
| local PostgreSQL state | `postgres_reachable`, `postgres_primary` |
| leader observations | `leader_member_id`, `active_leader_member_id`, `available_primary_member_id` |
| switchover and leadership flags | `switchover_requested_by`, `i_am_leader`, `has_other_leader_record`, `has_available_other_leader` |
| recovery and process state | `rewind_required`, `process_state` |

`DecideInput` contains `current: HaState` and `world: WorldSnapshot`.

## Decision Variants

`HaDecision` enumerates possible decision outcomes:

| Variant | Fields |
|---|---|
| `NoChange` | none |
| `WaitForPostgres` | `start_requested: bool`, `leader_member_id: Option<MemberId>` |
| `WaitForDcsTrust` | none |
| `AttemptLeadership` | none |
| `FollowLeader` | `leader_member_id: MemberId` |
| `BecomePrimary` | `promote: bool` |
| `StepDown` | `StepDownPlan` |
| `RecoverReplica` | `strategy: RecoveryStrategy` |
| `FenceNode` | none |
| `ReleaseLeaderLease` | `reason: LeaseReleaseReason` |
| `EnterFailSafe` | `release_leader_lease: bool` |

Supporting enums:

- `StepDownReason`: `Switchover`, `ForeignLeaderDetected { leader_member_id: MemberId }`
- `RecoveryStrategy`: `Rewind { leader_member_id: MemberId }`, `BaseBackup { leader_member_id: MemberId }`, `Bootstrap`
- `LeaseReleaseReason`: `FencingComplete`, `PostgresUnreachable`

## Effect and Action Lowering

`HaEffectPlan` organizes effects into five buckets:

| Bucket | Purpose |
|---|---|
| `lease` | Leader lease operations |
| `switchover` | Switchover request clearing |
| `replication` | Replication configuration |
| `postgres` | PostgreSQL process control |
| `safety` | Fencing and fail-safe signaling |

Bucket variants:

- `LeaseEffect`: `None`, `AcquireLeader`, `ReleaseLeader`
- `SwitchoverEffect`: `None`, `ClearRequest`
- `ReplicationEffect`: `None`, `FollowLeader { leader_member_id }`, `RecoverReplica { strategy }`
- `PostgresEffect`: `None`, `Start`, `Promote`, `Demote`
- `SafetyEffect`: `None`, `FenceNode`, `SignalFailSafe`

Lowering rules:

| Decision | Lease | Switchover | Replication | Postgres | Safety |
|---|---|---|---|---|---|
| `NoChange`, `WaitForDcsTrust` | `None` | `None` | `None` | `None` | `None` |
| `WaitForPostgres` (start) | | | | `Start` | |
| `AttemptLeadership` | `AcquireLeader` | | | | |
| `FollowLeader` | | | `FollowLeader` | | |
| `BecomePrimary` (promote) | | | | `Promote` | |
| `BecomePrimary` (no promote) | | | | `None` | |
| `StepDown` (plan) | optional release | optional clear | | `Demote` | optional `FenceNode` |
| `RecoverReplica` | | | `RecoverReplica` | | |
| `FenceNode` | | | | | `FenceNode` |
| `ReleaseLeaderLease` | `ReleaseLeader` | | | | |
| `EnterFailSafe` | optional release | | | | `SignalFailSafe` |

`apply_effect_plan` dispatches in order: `postgres`, `lease`, `switchover`, `replication`, `safety`.

## Worker Loop Behavior

`HaWorkerCtx` fields:

| Group | Fields |
|---|---|
| timing and state | `poll_interval`, `state` |
| subscribers and publisher | `publisher`, `config_subscriber`, `pg_subscriber`, `dcs_subscriber`, `process_subscriber` |
| process and store handles | `process_inbox`, `dcs_store` |
| identity | `scope`, `self_id` |
| dispatch defaults | `process_defaults` |
| runtime hooks | `now`, `log` |

`ProcessDispatchDefaults` fields:

| Group | Fields |
|---|---|
| local PostgreSQL endpoint | `postgres_host`, `postgres_port`, `socket_dir`, `log_file` |
| replication role credentials | `replicator_username`, `replicator_auth`, `rewinder_username`, `rewinder_auth` |
| remote connection defaults | `remote_dbname`, `remote_ssl_mode`, `connect_timeout_s` |
| process shutdown | `shutdown_mode` |

`ha::worker::run` waits on subscriber changes and interval ticks, then calls `step_once`.

`step_once` executes:

1. Builds `WorldSnapshot` from latest subscriber states
2. Calls `decide(DecideInput { current, world })`
3. Lowers decision into `HaEffectPlan`
4. Emits decision-selected and effect-plan-selected events
5. Publishes next state with `worker = Running` and incremented `tick`
6. Emits phase-transition and role-transition events when labels change
7. Dispatches effect plan unless redundant
8. Republishes with `worker = Faulted(...)` if dispatch errors occurred

Redundant process dispatch is skipped when `current.phase == next.phase`, `current.decision == next.decision`, and `next.decision` is `WaitForPostgres { start_requested: true, .. }`, `RecoverReplica { .. }`, or `FenceNode`.

`tick` increments with `saturating_add(1)` each decision step.

## Selected Invariants

- `step_once` consumes latest subscriber snapshots and publishes the next HA state with incremented tick and `WorkerStatus::Running`
- Duplicate `StartPostgres` dispatch is suppressed while PostgreSQL remains unreachable in `WaitingPostgresReachable`
- A primary that loses quorum enters `FailSafe` with `EnterFailSafe { release_leader_lease: false }` and enqueues fencing without deleting the leader lease
- A fail-safe primary with restored quorum attempts leadership by writing `/{scope}/leader` without deleting that path first
- A primary outage with another available primary enqueues only recovery dispatch and moves to `Rewinding`
- `apply_effect_plan` maps lease and process effects to DCS writes, DCS deletes, and process job requests
- `apply_effect_plan` is best-effort and reports typed dispatch errors
- Leader lease conflicts surface as `ActionDispatchError::DcsWrite`
- `run` reacts to interval ticks and watcher changes
- Integration tests exercise replica-to-candidate-to-primary-to-fail-safe transitions, primary-outage rewind recovery, split-brain fencing, and start-postgres dispatch
