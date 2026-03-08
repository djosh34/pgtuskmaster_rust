# HA State Machine Reference

The HA state machine implements a deterministic decision loop that translates observed cluster state into coordinated DCS and local process actions. The pipeline is `observe -> decide -> lower -> dispatch`.

## Overview

`HaWorkerCtx` contains the worker context.

| Group | Fields |
|---|---|
| timing and state | `poll_interval`, `state` |
| subscribers and publisher | `publisher`, `config_subscriber`, `pg_subscriber`, `dcs_subscriber`, `process_subscriber` |
| process and store handles | `process_inbox`, `dcs_store` |
| identity | `scope`, `self_id` |
| dispatch defaults | `process_defaults` |
| runtime hooks | `now`, `log` |

`ProcessDispatchDefaults` contains process configuration.

| Group | Fields |
|---|---|
| local PostgreSQL endpoint | `postgres_host`, `postgres_port`, `socket_dir`, `log_file` |
| replication role credentials | `replicator_username`, `replicator_auth`, `rewinder_username`, `rewinder_auth` |
| remote connection defaults | `remote_dbname`, `remote_ssl_mode`, `connect_timeout_s` |
| process shutdown | `shutdown_mode` |

`HaWorkerCtx::contract_stub` initializes `poll_interval = 10 ms`, `worker = Starting`, `phase = Init`, `tick = 0`, `decision = NoChange`, stub process defaults, and `now = UnixMillis(0)`.

`ha::worker::run` waits on:

- `pg_subscriber.changed()`
- `dcs_subscriber.changed()`
- `process_subscriber.changed()`
- `config_subscriber.changed()`
- `interval.tick()`

Then it calls `step_once`.

### step_once

1. builds `WorldSnapshot` from the latest `config`, `pg`, `dcs`, and `process`
2. calls `decide(DecideInput { current, world })`
3. lowers the selected decision into `HaEffectPlan`
4. emits decision-selected and effect-plan-selected events
5. publishes the next state with `worker = Running`
6. emits phase-transition and role-transition events when labels change
7. dispatches the effect plan unless redundant process dispatch should be skipped
8. republishes the same `phase`, `tick`, and `decision` with `worker = Faulted(...)` if dispatch errors were collected

`HaState` fields: `worker`, `phase`, `tick`, `decision`.

`tick` increments with `saturating_add(1)` each decision step.

`WorldSnapshot` fields: `config`, `pg`, `dcs`, `process`.

`ha_role_label` maps `Primary` to `primary`, `Replica` to `replica`, and all other phases to `unknown`.

Redundant process dispatch is skipped only when:

- `current.phase == next.phase`
- `current.decision == next.decision`
- `next.decision` is `WaitForPostgres { start_requested: true, .. }`, `RecoverReplica { .. }`, or `FenceNode`

## Phase Model

### HaPhase

Variants:

- `Init`
- `WaitingPostgresReachable`
- `WaitingDcsTrusted`
- `WaitingSwitchoverSuccessor`
- `Replica`
- `CandidateLeader`
- `Primary`
- `Rewinding`
- `Bootstrapping`
- `Fencing`
- `FailSafe`

## World Snapshot and Decision Inputs

### WorldSnapshot

| Field | Type |
|---|---|
| `config` | `Versioned<RuntimeConfig>` |
| `pg` | `Versioned<PgInfoState>` |
| `dcs` | `Versioned<DcsState>` |
| `process` | `Versioned<ProcessState>` |

### DecisionFacts

`DecisionFacts` captures observed cluster state.

| Group | Fields |
|---|---|
| identity and trust | `self_member_id`, `trust` |
| local PostgreSQL state | `postgres_reachable`, `postgres_primary` |
| leader observations | `leader_member_id`, `active_leader_member_id`, `available_primary_member_id` |
| switchover and leadership flags | `switchover_requested_by`, `i_am_leader`, `has_other_leader_record`, `has_available_other_leader` |
| recovery and process state | `rewind_required`, `process_state` |

`DecisionFacts::from_world` derives:

- `active_leader_member_id` from the leader record after `is_available_primary_leader` filtering
- `available_primary_member_id` from the active leader or from another healthy primary member
- `rewind_required` from `should_rewind_from_leader(world, leader_id)` when an available primary exists

`is_available_primary_leader` preserves the current leader record when corresponding member metadata has not been observed. When member metadata is present, the leader is active only if that member is a healthy primary.

Helper methods on `DecisionFacts`:

- `start_postgres_can_be_requested()`
- `rewind_activity()`
- `bootstrap_activity()`
- `fencing_activity()`

`ProcessActivity` values: `Running`, `IdleNoOutcome`, `IdleSuccess`, `IdleFailure`.

## Decision Variants

### HaDecision

Variants:

- `NoChange`
- `WaitForPostgres { start_requested: bool, leader_member_id: Option<MemberId> }`
- `WaitForDcsTrust`
- `AttemptLeadership`
- `FollowLeader { leader_member_id: MemberId }`
- `BecomePrimary { promote: bool }`
- `StepDown(StepDownPlan { reason: StepDownReason, release_leader_lease: bool, clear_switchover: bool, fence: bool })`
- `RecoverReplica { strategy: RecoveryStrategy }`
- `FenceNode`
- `ReleaseLeaderLease { reason: LeaseReleaseReason }`
- `EnterFailSafe { release_leader_lease: bool }`

Supporting enums:

- `StepDownReason`: `Switchover`, `ForeignLeaderDetected { leader_member_id: MemberId }`
- `RecoveryStrategy`: `Rewind { leader_member_id: MemberId }`, `BaseBackup { leader_member_id: MemberId }`, `Bootstrap`
- `LeaseReleaseReason`: `FencingComplete`, `PostgresUnreachable`

### Labels and Details

`HaDecision::label()` returns snake_case labels:

| Decision | Label |
|---|---|
| `NoChange` | `no_change` |
| `WaitForPostgres { .. }` | `wait_for_postgres` |
| `WaitForDcsTrust` | `wait_for_dcs_trust` |
| `AttemptLeadership` | `attempt_leadership` |
| `FollowLeader { .. }` | `follow_leader` |
| `BecomePrimary { .. }` | `become_primary` |
| `StepDown(_)` | `step_down` |
| `RecoverReplica { .. }` | `recover_replica` |
| `FenceNode` | `fence_node` |
| `ReleaseLeaderLease { .. }` | `release_leader_lease` |
| `EnterFailSafe { .. }` | `enter_fail_safe` |

`HaDecision::detail()` returns:

- `None` for `NoChange`, `WaitForDcsTrust`, `AttemptLeadership`, and `FenceNode`
- Descriptive strings for other variants

## Phase Transition Matrix

### Global Trust Override

At the top of `decide_phase`:

- if `trust != FullQuorum` and local PostgreSQL is primary: returns `FailSafe + EnterFailSafe { release_leader_lease: false }`
- if `trust != FullQuorum` and local PostgreSQL is not primary: returns `FailSafe + NoChange`

### Init

Returns `WaitingPostgresReachable + WaitForPostgres { start_requested: false, leader_member_id: None }`.

### WaitingPostgresReachable

- reachable PostgreSQL → `WaitingDcsTrusted + WaitForDcsTrust`
- completed `StartPostgres` job → `WaitingDcsTrusted + WaitForDcsTrust`
- otherwise → `wait_for_postgres(facts)`

### WaitingDcsTrusted

- unreachable PostgreSQL after `ReleaseLeaderLease { reason: FencingComplete }`
  - if a recovery leader or other leader record exists → `Bootstrapping + RecoverReplica(BaseBackup { leader_member_id })`
  - otherwise → `WaitingDcsTrusted + WaitForDcsTrust`
- other unreachable cases → `wait_for_postgres(facts)`
- active leader is self → `Primary + BecomePrimary { promote: false }`
- follow target exists → `Replica + FollowLeader`
- no follow target and local PostgreSQL is not primary → `WaitingDcsTrusted + WaitForDcsTrust`
- otherwise → `CandidateLeader + AttemptLeadership`

### WaitingSwitchoverSuccessor

- no leader or leader is self → `WaitingSwitchoverSuccessor + WaitForDcsTrust`
- unreachable PostgreSQL → `wait_for_postgres(facts)`
- follow target exists → `Replica + FollowLeader`
- otherwise → `WaitingSwitchoverSuccessor + WaitForDcsTrust`

### Replica

- unreachable PostgreSQL → `wait_for_postgres(facts)`
- switchover requested and active leader is self → `Replica + NoChange`
- active leader is self → `Primary + BecomePrimary { promote: true }`
- other active leader and `rewind_required` → `Rewinding + RecoverReplica(Rewind { leader_member_id })`
- other active leader without `rewind_required` → `Replica + FollowLeader`
- no active leader → `CandidateLeader + AttemptLeadership`

### CandidateLeader

- unreachable PostgreSQL → `wait_for_postgres(facts)`
- `i_am_leader` → `Primary + BecomePrimary { promote: !postgres_primary }`
- follow target exists → `Replica + FollowLeader`
- otherwise → `CandidateLeader + AttemptLeadership`

### Primary

- switchover requested and `i_am_leader` → `WaitingSwitchoverSuccessor + StepDown { reason: Switchover, release_leader_lease: true, clear_switchover: true, fence: false }`
- unreachable PostgreSQL and `i_am_leader` → `Rewinding + ReleaseLeaderLease { reason: PostgresUnreachable }`
- unreachable PostgreSQL and not leader
  - if a recovery leader exists → `Rewinding + RecoverReplica(Rewind { leader_member_id })`
  - otherwise → `Rewinding + NoChange`
- other active leader → `Fencing + StepDown { reason: ForeignLeaderDetected { leader_member_id }, release_leader_lease: true, clear_switchover: false, fence: true }`
- otherwise
  - if `i_am_leader` → `Primary + NoChange`
  - else → `Primary + AttemptLeadership`

### Rewinding

- `Running` → `Rewinding + NoChange`
- `IdleSuccess`
  - if follow target exists → `Replica + FollowLeader`
  - otherwise → `Replica + NoChange`
- `IdleFailure`
  - if a recovery leader exists → `Bootstrapping + RecoverReplica(BaseBackup { leader_member_id })`
  - otherwise → `Rewinding + NoChange`
- `IdleNoOutcome`
  - if a recovery leader exists → `Rewinding + RecoverReplica(Rewind { leader_member_id })`
  - otherwise → `Rewinding + NoChange`

### Bootstrapping

- `Running` → `Bootstrapping + NoChange`
- `IdleSuccess` → `wait_for_postgres(facts)`
- `IdleFailure` → `Fencing + FenceNode`
- `IdleNoOutcome`
  - if a recovery leader exists → `Bootstrapping + RecoverReplica(BaseBackup { leader_member_id })`
  - otherwise → `Bootstrapping + NoChange`

### Fencing

- `Running` → `Fencing + NoChange`
- `IdleSuccess` → `WaitingDcsTrusted + ReleaseLeaderLease { reason: FencingComplete }`
- `IdleFailure` → `FailSafe + EnterFailSafe { release_leader_lease: false }`
- `IdleNoOutcome` → `Fencing + FenceNode`

### FailSafe

- `Running` → `FailSafe + NoChange`
- otherwise if local PostgreSQL is primary → routes through the `Primary` decision path
- otherwise if `i_am_leader` → `FailSafe + ReleaseLeaderLease { reason: FencingComplete }`
- otherwise → `WaitingDcsTrusted + WaitForDcsTrust`

### wait_for_postgres(facts)

Returns `WaitingPostgresReachable + WaitForPostgres { start_requested: facts.start_postgres_can_be_requested(), leader_member_id: recovery_leader_member_id(facts).or_else(|| other_leader_record(facts)) }`.

## Effect and Action Lowering

### Effect Buckets

`HaEffectPlan` organizes effects into five buckets:

- `lease`
- `switchover`
- `replication`
- `postgres`
- `safety`

### Bucket Variants

| Bucket | Variants |
|---|---|
| `LeaseEffect` | `None`, `AcquireLeader`, `ReleaseLeader` |
| `SwitchoverEffect` | `None`, `ClearRequest` |
| `ReplicationEffect` | `None`, `FollowLeader { leader_member_id }`, `RecoverReplica { strategy }` |
| `PostgresEffect` | `None`, `Start`, `Promote`, `Demote` |
| `SafetyEffect` | `None`, `FenceNode`, `SignalFailSafe` |

### Lowering Rules

| Decision | Lowered Effects |
|---|---|
| `NoChange` and `WaitForDcsTrust` | empty plan |
| `WaitForPostgres` with `start_requested = true` | postgres `Start` |
| `AttemptLeadership` | lease `AcquireLeader` |
| `FollowLeader` | replication `FollowLeader` |
| `BecomePrimary { promote: true }` | postgres `Promote` |
| `BecomePrimary { promote: false }` | no postgres effect |
| `StepDown` | postgres `Demote` plus optional lease release, optional switchover clear, and optional safety `FenceNode` |
| `RecoverReplica` | replication `RecoverReplica` |
| `FenceNode` | safety `FenceNode` |
| `ReleaseLeaderLease` | lease `ReleaseLeader` |
| `EnterFailSafe` | safety `FenceNode` plus optional lease `ReleaseLeader` |

### Dispatch Order

`apply_effect_plan` dispatches in order: `postgres`, `lease`, `switchover`, `replication`, `safety`.

### Step Counts

`HaEffectPlan::dispatch_step_count()` sums bucket step counts:

| Operation | Count |
|---|---|
| lease acquire or release | 1 |
| switchover clear | 1 |
| follow-leader | 1 |
| rewind recovery | 1 |
| base-backup recovery | 2 |
| bootstrap recovery | 2 |
| postgres start, promote, or demote | 1 |
| safety fence or signal fail-safe | 1 |

## Selected Invariants from Tests

### Core State Transitions

- `src/ha/worker.rs` tests verify `step_once` consumes the latest subscriber snapshots and publishes the next HA state with incremented tick and `WorkerStatus::Running`.
- Tests verify duplicate `StartPostgres` dispatch is suppressed while PostgreSQL remains unreachable in `WaitingPostgresReachable`.
- Tests verify `step_once` matches the output of `decide(...)` for the same world snapshot.
- Tests verify a primary that loses quorum enters `FailSafe` with `EnterFailSafe { release_leader_lease: false }` and enqueues fencing without deleting the leader lease.
- Tests verify a fail-safe primary with restored quorum attempts leadership by writing `/{scope}/leader` without deleting that path first.
- Tests verify a primary outage with another available primary enqueues only recovery dispatch and moves to `Rewinding`.

### Effect Application and Loop Behavior

- `src/ha/worker.rs` tests verify `apply_effect_plan` maps lease and process effects to DCS writes, DCS deletes, and process job requests.
- Tests verify `apply_effect_plan` is best-effort and reports typed dispatch errors.
- Tests verify `apply_effect_plan` clears the switchover key when the plan includes `SwitchoverEffect::ClearRequest`.
- Tests verify leader lease conflicts surface as a typed `ActionDispatchError::DcsWrite`.
- Tests verify `run` reacts to both interval ticks and watcher changes.
- Tests verify the initial buffered updates emitted by `run` match the explicit two-step prefix produced by direct `step_once` calls.

### Integration Coverage

- `src/ha/worker.rs` integration tests exercise replica-to-candidate-to-primary-to-fail-safe transitions.
- Integration tests exercise primary-outage rewind recovery back to replica.
- Integration tests exercise split-brain fencing, demotion, lease release, and post-process feedback transition back to `WaitingDcsTrusted`.
- Integration tests exercise start-postgres dispatch and the resulting process-state version changes.
- `tests/ha_multi_node_failover.rs` covers unassisted failover, stress planned switchover, stress failover under concurrent SQL, custom PostgreSQL role names through bootstrap and rewind, and no-quorum fail-safe and fencing behavior.
- `tests/ha_partition_isolation.rs` covers minority isolation, primary isolation failover, API-path isolation, and mixed-fault healing without split brain.
