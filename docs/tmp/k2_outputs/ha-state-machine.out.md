# HA State Machine Reference

The HA state machine implements a deterministic decision loop that translates observed cluster state into coordinated DCS and local process actions. The pipeline is `observe -> decide -> lower -> dispatch`.

## Phase model

| Variant | Description |
|---|---|
| `Init` | Initial state before first observation |
| `WaitingPostgresReachable` | Waiting for PostgreSQL to become reachable |
| `WaitingDcsTrusted` | Waiting for DCS trust to be established |
| `WaitingSwitchoverSuccessor` | Waiting for a switchover successor to be identified |
| `Replica` | Operating as a replica following a leader |
| `CandidateLeader` | Attempting to become leader |
| `Primary` | Operating as the primary leader |
| `Rewinding` | Performing rewind recovery |
| `Bootstrapping` | Performing base backup or bootstrap recovery |
| `Fencing` | Executing fencing actions |
| `FailSafe` | Operating in fail-safe mode due to lost quorum |

## World snapshot and decision inputs

**`WorldSnapshot`**

| Group | Fields |
|---|---|
| Configuration | `config: Versioned<RuntimeConfig>` |
| PostgreSQL state | `pg: Versioned<PgInfoState>` |
| DCS state | `dcs: Versioned<DcsState>` |
| Process state | `process: Versioned<ProcessState>` |

**`DecideInput`**

| Field | Type |
|---|---|
| Current HA state | `current: HaState` |
| World observation | `world: WorldSnapshot` |

**`DecisionFacts`**

| Group | Fields |
|---|---|
| Identity and trust | `self_member_id`, `trust` |
| Local PostgreSQL state | `postgres_reachable`, `postgres_primary` |
| Leader observations | `leader_member_id`, `active_leader_member_id`, `available_primary_member_id` |
| Switchover and leadership flags | `switchover_requested_by`, `i_am_leader`, `has_other_leader_record`, `has_available_other_leader` |
| Recovery and process state | `rewind_required`, `process_state` |

**`DecisionFacts::from_world`** derives:
- `active_leader_member_id` from the leader record after `is_available_primary_leader` filtering
- `available_primary_member_id` from that active leader or from another healthy primary member
- `rewind_required` from `should_rewind_from_leader(world, leader_id)` when an available primary exists

**Process activity helpers:**
- `start_postgres_can_be_requested()`
- `rewind_activity()`
- `bootstrap_activity()`
- `fencing_activity()`

**`ProcessActivity` values:** `Running`, `IdleNoOutcome`, `IdleSuccess`, `IdleFailure`

## Decision variants

**`HaDecision`**

| Variant | Fields |
|---|---|
| `NoChange` | |
| `WaitForPostgres` | `start_requested: bool`, `leader_member_id: Option<MemberId>` |
| `WaitForDcsTrust` | |
| `AttemptLeadership` | |
| `FollowLeader` | `leader_member_id: MemberId` |
| `BecomePrimary` | `promote: bool` |
| `StepDown` | `StepDownPlan` |
| `RecoverReplica` | `strategy: RecoveryStrategy` |
| `FenceNode` | |
| `ReleaseLeaderLease` | `reason: LeaseReleaseReason` |
| `EnterFailSafe` | `release_leader_lease: bool` |

**Supporting enums**

**`StepDownReason`**
- `Switchover`
- `ForeignLeaderDetected { leader_member_id: MemberId }`

**`RecoveryStrategy`**
- `Rewind { leader_member_id: MemberId }`
- `BaseBackup { leader_member_id: MemberId }`
- `Bootstrap`

**`LeaseReleaseReason`**
- `FencingComplete`
- `PostgresUnreachable`

**`HaDecision::label()`** returns snake_case labels.

**`HaDecision::detail()`** returns `None` for `NoChange`, `WaitForDcsTrust`, `AttemptLeadership`, and `FenceNode`. Other variants return descriptive strings.

## Effect and action lowering

**Effect buckets**

`HaEffectPlan` organizes effects into five buckets: `lease`, `switchover`, `replication`, `postgres`, `safety`.

| Bucket | Variants |
|---|---|
| `LeaseEffect` | `None`, `AcquireLeader`, `ReleaseLeader` |
| `SwitchoverEffect` | `None`, `ClearRequest` |
| `ReplicationEffect` | `None`, `FollowLeader { leader_member_id }`, `RecoverReplica { strategy }` |
| `PostgresEffect` | `None`, `Start`, `Promote`, `Demote` |
| `SafetyEffect` | `None`, `FenceNode`, `SignalFailSafe` |

**Lowering rules**

| Decision | Effects |
|---|---|
| `NoChange`, `WaitForDcsTrust` | Empty plan |
| `WaitForPostgres` with `start_requested = true` | postgres `Start` |
| `AttemptLeadership` | lease `AcquireLeader` |
| `FollowLeader` | replication `FollowLeader` |
| `BecomePrimary { promote: true }` | postgres `Promote` |
| `BecomePrimary { promote: false }` | No postgres effect |
| `StepDown` | postgres `Demote` plus optional lease release, optional switchover clear, and optional safety `FenceNode` |
| `RecoverReplica` | replication `RecoverReplica` |
| `FenceNode` | safety `FenceNode` |
| `ReleaseLeaderLease` | lease `ReleaseLeader` |
| `EnterFailSafe` | safety `FenceNode` plus optional lease `ReleaseLeader` |

**Dispatch order**

`apply_effect_plan` dispatches in order: `postgres`, `lease`, `switchover`, `replication`, `safety`.

**Step counts**

| Effect | Steps |
|---|---|
| Lease acquire or release | 1 |
| Switchover clear | 1 |
| Follow-leader | 1 |
| Rewind recovery | 1 |
| Base-backup recovery | 2 |
| Bootstrap recovery | 2 |
| PostgreSQL start, promote, or demote | 1 |
| Safety fence or signal fail-safe | 1 |

## Worker loop behavior

**`HaWorkerCtx` fields**

| Group | Fields |
|---|---|
| Timing and state | `poll_interval`, `state` |
| Subscribers and publisher | `publisher`, `config_subscriber`, `pg_subscriber`, `dcs_subscriber`, `process_subscriber` |
| Process and store handles | `process_inbox`, `dcs_store` |
| Identity | `scope`, `self_id` |
| Dispatch defaults | `process_defaults` |
| Runtime hooks | `now`, `log` |

**`ProcessDispatchDefaults` fields**

| Group | Fields |
|---|---|
| Local PostgreSQL endpoint | `postgres_host`, `postgres_port`, `socket_dir`, `log_file` |
| Replication role credentials | `replicator_username`, `replicator_auth`, `rewinder_username`, `rewinder_auth` |
| Remote connection defaults | `remote_dbname`, `remote_ssl_mode`, `connect_timeout_s` |
| Process shutdown | `shutdown_mode` |

**`HaWorkerCtx::contract_stub`** initializes:
- `poll_interval = 10 ms`
- `worker = Starting`, `phase = Init`, `tick = 0`
- `decision = NoChange`
- Stub process defaults
- `now = UnixMillis(0)`

**`ha::worker::run`** waits on:
- `pg_subscriber.changed()`
- `dcs_subscriber.changed()`
- `process_subscriber.changed()`
- `config_subscriber.changed()`
- `interval.tick()`

Then calls `step_once`.

**`step_once`**

1. Builds `WorldSnapshot` from the latest `config`, `pg`, `dcs`, and `process`
2. Calls `decide(DecideInput { current, world })`
3. Lowers the selected decision into `HaEffectPlan`
4. Emits decision-selected and effect-plan-selected events
5. Publishes the next state with `worker = Running`
6. Emits phase-transition and role-transition events when labels change
7. Dispatches the effect plan unless redundant process dispatch should be skipped
8. Republishes the same `phase`, `tick`, and `decision` with `worker = Faulted(...)` if dispatch errors were collected

**`HaState` fields:** `worker`, `phase`, `tick`, `decision`

**`tick`** increments with `saturating_add(1)` each decision step

**`ha_role_label`** maps `Primary` to `primary`, `Replica` to `replica`, and all other phases to `unknown`

**Redundant process dispatch is skipped only when:**
- `current.phase == next.phase`
- `current.decision == next.decision`
- `next.decision` is `WaitForPostgres { start_requested: true, .. }`, `RecoverReplica { .. }`, or `FenceNode`

## Phase transition matrix

**Global trust override:** At the top of `decide_phase`, if `trust != FullQuorum`:
- If local PostgreSQL is primary: `FailSafe + EnterFailSafe { release_leader_lease: false }`
- If local PostgreSQL is not primary: `FailSafe + NoChange`

**`Init`** returns `WaitingPostgresReachable + WaitForPostgres { start_requested: false, leader_member_id: None }`

**`WaitingPostgresReachable`**
- Reachable PostgreSQL → `WaitingDcsTrusted + WaitForDcsTrust`
- Completed `StartPostgres` job → `WaitingDcsTrusted + WaitForDcsTrust`
- Otherwise → `wait_for_postgres(facts)`

**`WaitingDcsTrusted`**
- Unreachable PostgreSQL after `ReleaseLeaderLease { reason: FencingComplete }`
    - If a recovery leader or other leader record exists → `Bootstrapping + RecoverReplica(BaseBackup { leader_member_id })`
    - Otherwise → `WaitingDcsTrusted + WaitForDcsTrust`
- Other unreachable cases → `wait_for_postgres(facts)`
- Active leader is self → `Primary + BecomePrimary { promote: false }`
- Follow target exists → `Replica + FollowLeader`
- No follow target and local PostgreSQL is not primary → `WaitingDcsTrusted + WaitForDcsTrust`
- Otherwise → `CandidateLeader + AttemptLeadership`

**`WaitingSwitchoverSuccessor`**
- No leader or leader is self → `WaitingSwitchoverSuccessor + WaitForDcsTrust`
- Unreachable PostgreSQL → `wait_for_postgres(facts)`
- Follow target exists → `Replica + FollowLeader`
- Otherwise → `WaitingSwitchoverSuccessor + WaitForDcsTrust`

**`Replica`**
- Unreachable PostgreSQL → `wait_for_postgres(facts)`
- Switchover requested and active leader is self → `Replica + NoChange`
- Active leader is self → `Primary + BecomePrimary { promote: true }`
- Other active leader and `rewind_required` → `Rewinding + RecoverReplica(Rewind { leader_member_id })`
- Other active leader without `rewind_required` → `Replica + FollowLeader`
- No active leader → `CandidateLeader + AttemptLeadership`

**`CandidateLeader`**
- Unreachable PostgreSQL → `wait_for_postgres(facts)`
- `i_am_leader` → `Primary + BecomePrimary { promote: !postgres_primary }`
- Follow target exists → `Replica + FollowLeader`
- Otherwise → `CandidateLeader + AttemptLeadership`

**`Primary`**
- Switchover requested and `i_am_leader` → `WaitingSwitchoverSuccessor + StepDown { reason: Switchover, release_leader_lease: true, clear_switchover: true, fence: false }`
- Unreachable PostgreSQL and `i_am_leader` → `Rewinding + ReleaseLeaderLease { reason: PostgresUnreachable }`
- Unreachable PostgreSQL and not leader
    - If a recovery leader exists → `Rewinding + RecoverReplica(Rewind { leader_member_id })`
    - Otherwise → `Rewinding + NoChange`
- Other active leader → `Fencing + StepDown { reason: ForeignLeaderDetected { leader_member_id }, release_leader_lease: true, clear_switchover: false, fence: true }`
- Otherwise
    - If `i_am_leader` → `Primary + NoChange`
    - Else → `Primary + AttemptLeadership`

**`Rewinding`**
- `Running` → `Rewinding + NoChange`
- `IdleSuccess`
    - If follow target exists → `Replica + FollowLeader`
    - Otherwise → `Replica + NoChange`
- `IdleFailure`
    - If a recovery leader exists → `Bootstrapping + RecoverReplica(BaseBackup { leader_member_id })`
    - Otherwise → `Rewinding + NoChange`
- `IdleNoOutcome`
    - If a recovery leader exists → `Rewinding + RecoverReplica(Rewind { leader_member_id })`
    - Otherwise → `Rewinding + NoChange`

**`Bootstrapping`**
- `Running` → `Bootstrapping + NoChange`
- `IdleSuccess` → `wait_for_postgres(facts)`
- `IdleFailure` → `Fencing + FenceNode`
- `IdleNoOutcome`
    - If a recovery leader exists → `Bootstrapping + RecoverReplica(BaseBackup { leader_member_id })`
    - Otherwise → `Bootstrapping + NoChange`

**`Fencing`**
- `Running` → `Fencing + NoChange`
- `IdleSuccess` → `WaitingDcsTrusted + ReleaseLeaderLease { reason: FencingComplete }`
- `IdleFailure` → `FailSafe + EnterFailSafe { release_leader_lease: false }`
- `IdleNoOutcome` → `Fencing + FenceNode`

**`FailSafe`**
- `Running` → `FailSafe + NoChange`
- Otherwise if local PostgreSQL is primary → route through the `Primary` decision path
- Otherwise if `i_am_leader` → `FailSafe + ReleaseLeaderLease { reason: FencingComplete }`
- Otherwise → `WaitingDcsTrusted + WaitForDcsTrust`

**`wait_for_postgres(facts)`** returns:
`WaitingPostgresReachable + WaitForPostgres { start_requested: facts.start_postgres_can_be_requested(), leader_member_id: recovery_leader_member_id(facts).or_else(|| other_leader_record(facts)) }`

## Selected invariants from tests

**Core state transitions**
- `src/ha/worker.rs` tests verify `step_once` consumes the latest subscriber snapshots and publishes the next HA state with incremented tick and `WorkerStatus::Running`
- `src/ha/worker.rs` tests verify duplicate `StartPostgres` dispatch is suppressed while PostgreSQL remains unreachable in `WaitingPostgresReachable`
- `src/ha/worker.rs` tests verify `step_once` matches the output of `decide(...)` for the same world snapshot
- `src/ha/worker.rs` tests verify a primary that loses quorum enters `FailSafe` with `EnterFailSafe { release_leader_lease: false }` and enqueues fencing without deleting the leader lease
- `src/ha/worker.rs` tests verify a fail-safe primary with restored quorum attempts leadership by writing `/{scope}/leader` without deleting that path first
- `src/ha/worker.rs` tests verify a primary outage with another available primary enqueues only recovery dispatch and moves to `Rewinding`

**Effect application and loop behavior**
- `src/ha/worker.rs` tests verify `apply_effect_plan` maps lease and process effects to DCS writes, DCS deletes, and process job requests
- `src/ha/worker.rs` tests verify `apply_effect_plan` is best-effort and reports typed dispatch errors
- `src/ha/worker.rs` tests verify `apply_effect_plan` clears the switchover key when the plan includes `SwitchoverEffect::ClearRequest`
- `src/ha/worker.rs` tests verify leader lease conflicts surface as a typed `ActionDispatchError::DcsWrite`
- `src/ha/worker.rs` tests verify `run` reacts to both interval ticks and watcher changes
- `src/ha/worker.rs` tests verify the initial buffered updates emitted by `run` match the explicit two-step prefix produced by direct `step_once` calls

**Integration coverage**
- `src/ha/worker.rs` integration tests exercise replica-to-candidate-to-primary-to-fail-safe transitions
- `src/ha/worker.rs` integration tests exercise primary-outage rewind recovery back to replica
- `src/ha/worker.rs` integration tests exercise split-brain fencing, demotion, lease release, and post-process feedback transition back to `WaitingDcsTrusted`
- `src/ha/worker.rs` integration tests exercise start-postgres dispatch and the resulting process-state version changes
- `tests/ha_multi_node_failover.rs` covers unassisted failover, stress planned switchover, stress failover under concurrent SQL, custom PostgreSQL role names through bootstrap and rewind, and no-quorum fail-safe and fencing behavior
- `tests/ha_partition_isolation.rs` covers minority isolation, primary isolation failover, API-path isolation, and mixed-fault healing without split brain
