Answer using only the information in this prompt.
Do not ask to inspect files, browse, run tools, or fetch more context.
If required information is missing, say exactly what is missing.

You are writing prose for an mdBook documentation page.
Return only markdown for the page body requested.
Use ASCII punctuation only.
Do not use em dashes.
Do not invent facts.

[Task]
- Revise an existing reference page so it stays strictly in Diataxis reference form.

[Page path]
- docs/src/reference/ha-state-machine.md

[Page goal]
- Reference the HA phases, decision model, lowered effects, and worker state transitions.

[Audience]
- Operators and contributors who need accurate repo-backed facts while working with pgtuskmaster.

[User need]
- Consult the machinery surface, data model, constraints, constants, and behavior without being taught procedures or background explanations.

[mdBook context]
- This is an mdBook page under docs/src/reference/.
- Keep headings and lists suitable for mdBook.
- Do not add verification notes, scratch notes, or commentary about how the page was produced.

[Diataxis guidance]
- This page must stay in the reference quadrant: cognition plus application.
- Describe and only describe.
- Structure the page to mirror the machinery, not a guessed workflow.
- Use neutral, technical language.
- Examples are allowed only when they illustrate the surface concisely.
- Do not include step-by-step operations, recommendations, rationale, or explanations of why the design exists.
- If action or explanation seems necessary, keep the page neutral and mention the boundary without turning the page into a how-to or explanation article.

[Required structure]
- Overview\n- Phase model\n- World snapshot and decision inputs\n- Decision variants\n- Effect and action lowering\n- Worker loop behavior\n- Selected invariants from tests when directly supported by the excerpts

[Output requirements]
- Preserve or improve the title so it matches the machinery.
- Prefer compact tables where they clarify enums, fields, constants, routes, or error variants.
- Include only facts supported by the supplied source excerpts.
- If the current page contains unsupported claims, remove or correct them rather than hedging.

[Existing page to revise]

# HA State Machine Reference

The HA state machine implements a deterministic decision loop that translates observed cluster state into coordinated DCS and local process actions. The pipeline is `observe -> decide -> lower -> dispatch`.

## Worker Loop Foundation

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

`HaWorkerCtx::contract_stub` initializes `poll_interval = 10 ms`, `worker = Starting`, `phase = Init`, `tick = 0`, `decision = NoChange`, stub process defaults, and `now = UnixMillis(0)`.

`ha::worker::run` waits on:

- `pg_subscriber.changed()`
- `dcs_subscriber.changed()`
- `process_subscriber.changed()`
- `config_subscriber.changed()`
- `interval.tick()`

Then it calls `step_once`.

### `step_once`

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

## State And Decision Model

### `HaPhase`

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

### `DecisionFacts`

`DecisionFacts` captures the observed cluster truth:

| Group | Fields |
|---|---|
| identity and trust | `self_member_id`, `trust` |
| local PostgreSQL state | `postgres_reachable`, `postgres_primary` |
| leader observations | `leader_member_id`, `active_leader_member_id`, `available_primary_member_id` |
| switchover and leadership flags | `switchover_requested_by`, `i_am_leader`, `has_other_leader_record`, `has_available_other_leader` |
| recovery and process state | `rewind_required`, `process_state` |

`DecisionFacts::from_world` derives `active_leader_member_id` from the leader record after `is_available_primary_leader` filtering, derives `available_primary_member_id` from that active leader or from another healthy primary member, and derives `rewind_required` from `should_rewind_from_leader(world, leader_id)` when an available primary exists.

`is_available_primary_leader` preserves the current leader record when the corresponding member metadata has not been observed yet. When member metadata is present, the leader is active only if that member is a healthy primary.

Helper methods:

- `start_postgres_can_be_requested()`
- `rewind_activity()`
- `bootstrap_activity()`
- `fencing_activity()`

`ProcessActivity` values: `Running`, `IdleNoOutcome`, `IdleSuccess`, `IdleFailure`.

## Phase Transition Matrix

### Global Trust Override

At the top of `decide_phase`:

- if `trust != FullQuorum` and local PostgreSQL is primary: `FailSafe + EnterFailSafe { release_leader_lease: false }`
- if `trust != FullQuorum` and local PostgreSQL is not primary: `FailSafe + NoChange`

### `Init`

`Init` returns `WaitingPostgresReachable + WaitForPostgres { start_requested: false, leader_member_id: None }`.

### `WaitingPostgresReachable`

- reachable PostgreSQL -> `WaitingDcsTrusted + WaitForDcsTrust`
- completed `StartPostgres` job -> `WaitingDcsTrusted + WaitForDcsTrust`
- otherwise -> `wait_for_postgres(facts)`

### `WaitingDcsTrusted`

- unreachable PostgreSQL after `ReleaseLeaderLease { reason: FencingComplete }`
  - if a recovery leader or other leader record exists -> `Bootstrapping + RecoverReplica(BaseBackup { leader_member_id })`
  - otherwise -> `WaitingDcsTrusted + WaitForDcsTrust`
- other unreachable cases -> `wait_for_postgres(facts)`
- active leader is self -> `Primary + BecomePrimary { promote: false }`
- follow target exists -> `Replica + FollowLeader`
- no follow target and local PostgreSQL is not primary -> `WaitingDcsTrusted + WaitForDcsTrust`
- otherwise -> `CandidateLeader + AttemptLeadership`

### `WaitingSwitchoverSuccessor`

- no leader or leader is self -> `WaitingSwitchoverSuccessor + WaitForDcsTrust`
- unreachable PostgreSQL -> `wait_for_postgres(facts)`
- follow target exists -> `Replica + FollowLeader`
- otherwise -> `WaitingSwitchoverSuccessor + WaitForDcsTrust`

### `Replica`

- unreachable PostgreSQL -> `wait_for_postgres(facts)`
- switchover requested and active leader is self -> `Replica + NoChange`
- active leader is self -> `Primary + BecomePrimary { promote: true }`
- other active leader and `rewind_required` -> `Rewinding + RecoverReplica(Rewind { leader_member_id })`
- other active leader without `rewind_required` -> `Replica + FollowLeader`
- no active leader -> `CandidateLeader + AttemptLeadership`

### `CandidateLeader`

- unreachable PostgreSQL -> `wait_for_postgres(facts)`
- `i_am_leader` -> `Primary + BecomePrimary { promote: !postgres_primary }`
- follow target exists -> `Replica + FollowLeader`
- otherwise -> `CandidateLeader + AttemptLeadership`

### `Primary`

- switchover requested and `i_am_leader` -> `WaitingSwitchoverSuccessor + StepDown { reason: Switchover, release_leader_lease: true, clear_switchover: true, fence: false }`
- unreachable PostgreSQL and `i_am_leader` -> `Rewinding + ReleaseLeaderLease { reason: PostgresUnreachable }`
- unreachable PostgreSQL and not leader
  - if a recovery leader exists -> `Rewinding + RecoverReplica(Rewind { leader_member_id })`
  - otherwise -> `Rewinding + NoChange`
- other active leader -> `Fencing + StepDown { reason: ForeignLeaderDetected { leader_member_id }, release_leader_lease: true, clear_switchover: false, fence: true }`
- otherwise
  - if `i_am_leader` -> `Primary + NoChange`
  - else -> `Primary + AttemptLeadership`

### `Rewinding`

- `Running` -> `Rewinding + NoChange`
- `IdleSuccess`
  - if follow target exists -> `Replica + FollowLeader`
  - otherwise -> `Replica + NoChange`
- `IdleFailure`
  - if a recovery leader exists -> `Bootstrapping + RecoverReplica(BaseBackup { leader_member_id })`
  - otherwise -> `Rewinding + NoChange`
- `IdleNoOutcome`
  - if a recovery leader exists -> `Rewinding + RecoverReplica(Rewind { leader_member_id })`
  - otherwise -> `Rewinding + NoChange`

### `Bootstrapping`

- `Running` -> `Bootstrapping + NoChange`
- `IdleSuccess` -> `wait_for_postgres(facts)`
- `IdleFailure` -> `Fencing + FenceNode`
- `IdleNoOutcome`
  - if a recovery leader exists -> `Bootstrapping + RecoverReplica(BaseBackup { leader_member_id })`
  - otherwise -> `Bootstrapping + NoChange`

### `Fencing`

- `Running` -> `Fencing + NoChange`
- `IdleSuccess` -> `WaitingDcsTrusted + ReleaseLeaderLease { reason: FencingComplete }`
- `IdleFailure` -> `FailSafe + EnterFailSafe { release_leader_lease: false }`
- `IdleNoOutcome` -> `Fencing + FenceNode`

### `FailSafe`

- `Running` -> `FailSafe + NoChange`
- otherwise if local PostgreSQL is primary -> route through the `Primary` decision path
- otherwise if `i_am_leader` -> `FailSafe + ReleaseLeaderLease { reason: FencingComplete }`
- otherwise -> `WaitingDcsTrusted + WaitForDcsTrust`

### `wait_for_postgres(facts)`

`wait_for_postgres(facts)` returns:

`WaitingPostgresReachable + WaitForPostgres { start_requested: facts.start_postgres_can_be_requested(), leader_member_id: recovery_leader_member_id(facts).or_else(|| other_leader_record(facts)) }`

## Decision Variants

`HaDecision` variants:

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

`HaDecision::label()` returns snake_case labels.

`HaDecision::detail()` returns `None` for `NoChange`, `WaitForDcsTrust`, `AttemptLeadership`, and `FenceNode`. Other variants return descriptive strings.

## Effect-Plan Lowering And Dispatch

### Effect Buckets

`HaEffectPlan` organizes effects into five buckets:

- `lease`
- `switchover`
- `replication`
- `postgres`
- `safety`

### Bucket Variants

- `LeaseEffect`: `None`, `AcquireLeader`, `ReleaseLeader`
- `SwitchoverEffect`: `None`, `ClearRequest`
- `ReplicationEffect`: `None`, `FollowLeader { leader_member_id }`, `RecoverReplica { strategy }`
- `PostgresEffect`: `None`, `Start`, `Promote`, `Demote`
- `SafetyEffect`: `None`, `FenceNode`, `SignalFailSafe`

### Lowering Rules

- `NoChange` and `WaitForDcsTrust` -> empty plan
- `WaitForPostgres` with `start_requested = true` -> postgres `Start`
- `AttemptLeadership` -> lease `AcquireLeader`
- `FollowLeader` -> replication `FollowLeader`
- `BecomePrimary { promote: true }` -> postgres `Promote`
- `BecomePrimary { promote: false }` -> no postgres effect
- `StepDown` -> postgres `Demote` plus optional lease release, optional switchover clear, and optional safety `FenceNode`
- `RecoverReplica` -> replication `RecoverReplica`
- `FenceNode` -> safety `FenceNode`
- `ReleaseLeaderLease` -> lease `ReleaseLeader`
- `EnterFailSafe` -> safety `FenceNode` plus optional lease `ReleaseLeader`

### Dispatch Order

`apply_effect_plan` dispatches in order: `postgres`, `lease`, `switchover`, `replication`, `safety`.

### Step Counts

`HaEffectPlan::dispatch_step_count()` sums bucket step counts:

- lease acquire or release: `1`
- switchover clear: `1`
- follow-leader: `1`
- rewind recovery: `1`
- base-backup recovery: `2`
- bootstrap recovery: `2`
- postgres start, promote, or demote: `1`
- safety fence or signal fail-safe: `1`

## Verified Behaviors

### Core State Transitions

- `src/ha/worker.rs` tests verify `step_once` consumes the latest subscriber snapshots and publishes the next HA state with incremented tick and `WorkerStatus::Running`.
- `src/ha/worker.rs` tests verify duplicate `StartPostgres` dispatch is suppressed while PostgreSQL remains unreachable in `WaitingPostgresReachable`.
- `src/ha/worker.rs` tests verify `step_once` matches the output of `decide(...)` for the same world snapshot.
- `src/ha/worker.rs` tests verify a primary that loses quorum enters `FailSafe` with `EnterFailSafe { release_leader_lease: false }` and enqueues fencing without deleting the leader lease.
- `src/ha/worker.rs` tests verify a fail-safe primary with restored quorum attempts leadership by writing `/{scope}/leader` without deleting that path first.
- `src/ha/worker.rs` tests verify a primary outage with another available primary enqueues only recovery dispatch and moves to `Rewinding`.

### Effect Application And Loop Behavior

- `src/ha/worker.rs` tests verify `apply_effect_plan` maps lease and process effects to DCS writes, DCS deletes, and process job requests.
- `src/ha/worker.rs` tests verify `apply_effect_plan` is best-effort and reports typed dispatch errors.
- `src/ha/worker.rs` tests verify `apply_effect_plan` clears the switchover key when the plan includes `SwitchoverEffect::ClearRequest`.
- `src/ha/worker.rs` tests verify leader lease conflicts surface as a typed `ActionDispatchError::DcsWrite`.
- `src/ha/worker.rs` tests verify `run` reacts to both interval ticks and watcher changes.
- `src/ha/worker.rs` tests verify the initial buffered updates emitted by `run` match the explicit two-step prefix produced by direct `step_once` calls.

### Integration Coverage

- `src/ha/worker.rs` integration tests exercise replica-to-candidate-to-primary-to-fail-safe transitions.
- `src/ha/worker.rs` integration tests exercise primary-outage rewind recovery back to replica.
- `src/ha/worker.rs` integration tests exercise split-brain fencing, demotion, lease release, and post-process feedback transition back to `WaitingDcsTrusted`.
- `src/ha/worker.rs` integration tests exercise start-postgres dispatch and the resulting process-state version changes.
- `tests/ha_multi_node_failover.rs` covers unassisted failover, stress planned switchover, stress failover under concurrent SQL, custom PostgreSQL role names through bootstrap and rewind, and no-quorum fail-safe and fencing behavior.
- `tests/ha_partition_isolation.rs` covers minority isolation, primary isolation failover, API-path isolation, and mixed-fault healing without split brain.

[Repo facts and source excerpts]

--- BEGIN FILE: src/ha/mod.rs ---
pub(crate) mod actions;
pub(crate) mod apply;
pub(crate) mod decide;
pub(crate) mod decision;
pub(crate) mod events;
pub(crate) mod lower;
pub(crate) mod process_dispatch;
pub(crate) mod source_conn;
pub(crate) mod state;
pub(crate) mod worker;

--- END FILE: src/ha/mod.rs ---

--- BEGIN FILE: src/ha/state.rs ---
use std::{path::PathBuf, time::Duration};

use crate::{
    config::{RoleAuthConfig, RuntimeConfig},
    dcs::{state::DcsState, store::DcsStore},
    logging::LogHandle,
    pginfo::state::{PgInfoState, PgSslMode},
    process::{
        jobs::ShutdownMode,
        state::{ProcessJobRequest, ProcessState},
    },
    state::{
        MemberId, StatePublisher, StateSubscriber, UnixMillis, Versioned, WorkerError, WorkerStatus,
    },
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use super::decision::{HaDecision, PhaseOutcome};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HaPhase {
    Init,
    WaitingPostgresReachable,
    WaitingDcsTrusted,
    WaitingSwitchoverSuccessor,
    Replica,
    CandidateLeader,
    Primary,
    Rewinding,
    Bootstrapping,
    Fencing,
    FailSafe,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HaState {
    pub(crate) worker: WorkerStatus,
    pub(crate) phase: HaPhase,
    pub(crate) tick: u64,
    pub(crate) decision: HaDecision,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WorldSnapshot {
    pub(crate) config: Versioned<RuntimeConfig>,
    pub(crate) pg: Versioned<PgInfoState>,
    pub(crate) dcs: Versioned<DcsState>,
    pub(crate) process: Versioned<ProcessState>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DecideInput {
    pub(crate) current: HaState,
    pub(crate) world: WorldSnapshot,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DecideOutput {
    pub(crate) next: HaState,
    pub(crate) outcome: PhaseOutcome,
}

pub(crate) struct HaWorkerCtx {
    pub(crate) poll_interval: Duration,
    pub(crate) state: HaState,
    pub(crate) publisher: StatePublisher<HaState>,
    pub(crate) config_subscriber: StateSubscriber<RuntimeConfig>,
    pub(crate) pg_subscriber: StateSubscriber<PgInfoState>,
    pub(crate) dcs_subscriber: StateSubscriber<DcsState>,
    pub(crate) process_subscriber: StateSubscriber<ProcessState>,
    pub(crate) process_inbox: UnboundedSender<ProcessJobRequest>,
    pub(crate) dcs_store: Box<dyn DcsStore>,
    pub(crate) scope: String,
    pub(crate) self_id: MemberId,
    pub(crate) process_defaults: ProcessDispatchDefaults,
    pub(crate) now: Box<dyn FnMut() -> Result<UnixMillis, WorkerError> + Send>,
    pub(crate) log: LogHandle,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessDispatchDefaults {
    pub(crate) postgres_host: String,
    pub(crate) postgres_port: u16,
    pub(crate) socket_dir: PathBuf,
    pub(crate) log_file: PathBuf,
    pub(crate) replicator_username: String,
    pub(crate) replicator_auth: RoleAuthConfig,
    pub(crate) rewinder_username: String,
    pub(crate) rewinder_auth: RoleAuthConfig,
    pub(crate) remote_dbname: String,
    pub(crate) remote_ssl_mode: PgSslMode,
    pub(crate) connect_timeout_s: u32,
    pub(crate) shutdown_mode: ShutdownMode,
}

impl ProcessDispatchDefaults {
    pub(crate) fn contract_stub() -> Self {
        Self {
            postgres_host: "127.0.0.1".to_string(),
            postgres_port: 5432,
            socket_dir: PathBuf::from("/tmp/pgtuskmaster/socket"),
            log_file: PathBuf::from("/tmp/pgtuskmaster/postgres.log"),
            replicator_username: "replicator".to_string(),
            replicator_auth: contract_stub_password_auth(),
            rewinder_username: "rewinder".to_string(),
            rewinder_auth: contract_stub_password_auth(),
            remote_dbname: "postgres".to_string(),
            remote_ssl_mode: PgSslMode::Prefer,
            connect_timeout_s: 5,
            shutdown_mode: ShutdownMode::Fast,
        }
    }
}

fn contract_stub_password_auth() -> RoleAuthConfig {
    RoleAuthConfig::Password {
        password: crate::config::SecretSource(crate::config::InlineOrPath::Inline {
            content: "secret-password".to_string(),
        }),
    }
}

pub(crate) struct HaWorkerContractStubInputs {
    pub(crate) publisher: StatePublisher<HaState>,
    pub(crate) config_subscriber: StateSubscriber<RuntimeConfig>,
    pub(crate) pg_subscriber: StateSubscriber<PgInfoState>,
    pub(crate) dcs_subscriber: StateSubscriber<DcsState>,
    pub(crate) process_subscriber: StateSubscriber<ProcessState>,
    pub(crate) process_inbox: UnboundedSender<ProcessJobRequest>,
    pub(crate) dcs_store: Box<dyn DcsStore>,
    pub(crate) scope: String,
    pub(crate) self_id: MemberId,
}

impl HaWorkerCtx {
    pub(crate) fn contract_stub(inputs: HaWorkerContractStubInputs) -> Self {
        let HaWorkerContractStubInputs {
            publisher,
            config_subscriber,
            pg_subscriber,
            dcs_subscriber,
            process_subscriber,
            process_inbox,
            dcs_store,
            scope,
            self_id,
        } = inputs;

        Self {
            poll_interval: Duration::from_millis(10),
            state: HaState {
                worker: WorkerStatus::Starting,
                phase: HaPhase::Init,
                tick: 0,
                decision: HaDecision::NoChange,
            },
            publisher,
            config_subscriber,
            pg_subscriber,
            dcs_subscriber,
            process_subscriber,
            process_inbox,
            dcs_store,
            scope,
            self_id,
            process_defaults: ProcessDispatchDefaults::contract_stub(),
            now: Box::new(|| Ok(UnixMillis(0))),
            log: LogHandle::disabled(),
        }
    }
}

--- END FILE: src/ha/state.rs ---

--- BEGIN FILE: src/ha/decision.rs ---
use serde::{Deserialize, Serialize};

use crate::{
    dcs::state::{DcsTrust, MemberRole},
    pginfo::state::{PgInfoState, SqlStatus},
    process::{
        jobs::ActiveJobKind,
        state::{JobOutcome, ProcessState},
    },
    state::{MemberId, TimelineId},
};

use super::state::{HaPhase, WorldSnapshot};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DecisionFacts {
    pub(crate) self_member_id: MemberId,
    pub(crate) trust: DcsTrust,
    pub(crate) postgres_reachable: bool,
    pub(crate) postgres_primary: bool,
    pub(crate) leader_member_id: Option<MemberId>,
    pub(crate) active_leader_member_id: Option<MemberId>,
    pub(crate) available_primary_member_id: Option<MemberId>,
    pub(crate) switchover_requested_by: Option<MemberId>,
    pub(crate) i_am_leader: bool,
    pub(crate) has_other_leader_record: bool,
    pub(crate) has_available_other_leader: bool,
    pub(crate) rewind_required: bool,
    pub(crate) process_state: ProcessState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ProcessActivity {
    Running,
    IdleNoOutcome,
    IdleSuccess,
    IdleFailure,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PhaseOutcome {
    pub(crate) next_phase: HaPhase,
    pub(crate) decision: HaDecision,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum HaDecision {
    #[default]
    NoChange,
    WaitForPostgres {
        start_requested: bool,
        leader_member_id: Option<MemberId>,
    },
    WaitForDcsTrust,
    AttemptLeadership,
    FollowLeader {
        leader_member_id: MemberId,
    },
    BecomePrimary {
        promote: bool,
    },
    StepDown(StepDownPlan),
    RecoverReplica {
        strategy: RecoveryStrategy,
    },
    FenceNode,
    ReleaseLeaderLease {
        reason: LeaseReleaseReason,
    },
    EnterFailSafe {
        release_leader_lease: bool,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct StepDownPlan {
    pub(crate) reason: StepDownReason,
    pub(crate) release_leader_lease: bool,
    pub(crate) clear_switchover: bool,
    pub(crate) fence: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum StepDownReason {
    Switchover,
    ForeignLeaderDetected { leader_member_id: MemberId },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum RecoveryStrategy {
    Rewind { leader_member_id: MemberId },
    BaseBackup { leader_member_id: MemberId },
    Bootstrap,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum LeaseReleaseReason {
    FencingComplete,
    PostgresUnreachable,
}

impl DecisionFacts {
    pub(crate) fn from_world(world: &WorldSnapshot) -> Self {
        let self_member_id = MemberId(world.config.value.cluster.member_id.clone());
        let leader_member_id = world
            .dcs
            .value
            .cache
            .leader
            .as_ref()
            .map(|record| record.member_id.clone());
        let active_leader_member_id = leader_member_id
            .clone()
            .filter(|leader_id| is_available_primary_leader(world, leader_id));
        let available_primary_member_id = active_leader_member_id.clone().or_else(|| {
            world
                .dcs
                .value
                .cache
                .members
                .values()
                .find(|member| {
                    member.member_id != self_member_id
                        && member.role == MemberRole::Primary
                        && member.sql == SqlStatus::Healthy
                })
                .map(|member| member.member_id.clone())
        });
        let i_am_leader = leader_member_id.as_ref() == Some(&self_member_id);
        let has_other_leader_record = leader_member_id
            .as_ref()
            .map(|leader_id| leader_id != &self_member_id)
            .unwrap_or(false);
        let has_available_other_leader = active_leader_member_id
            .as_ref()
            .map(|leader_id| leader_id != &self_member_id)
            .unwrap_or(false);

        Self {
            self_member_id,
            trust: world.dcs.value.trust.clone(),
            postgres_reachable: is_postgres_reachable(&world.pg.value),
            postgres_primary: is_local_primary(&world.pg.value),
            leader_member_id,
            active_leader_member_id: active_leader_member_id.clone(),
            available_primary_member_id: available_primary_member_id.clone(),
            switchover_requested_by: world
                .dcs
                .value
                .cache
                .switchover
                .as_ref()
                .map(|request| request.requested_by.clone()),
            i_am_leader,
            has_other_leader_record,
            has_available_other_leader,
            rewind_required: available_primary_member_id
                .as_ref()
                .map(|leader_id| should_rewind_from_leader(world, leader_id))
                .unwrap_or(false),
            process_state: world.process.value.clone(),
        }
    }
}

impl ProcessActivity {
    fn from_process_state(process: &ProcessState, expected_kinds: &[ActiveJobKind]) -> Self {
        match process {
            ProcessState::Running { active, .. } => {
                if expected_kinds.contains(&active.kind) {
                    Self::Running
                } else {
                    Self::IdleNoOutcome
                }
            }
            ProcessState::Idle {
                last_outcome: Some(JobOutcome::Success { job_kind, .. }),
                ..
            } => {
                if expected_kinds.contains(job_kind) {
                    Self::IdleSuccess
                } else {
                    Self::IdleNoOutcome
                }
            }
            ProcessState::Idle {
                last_outcome:
                    Some(JobOutcome::Failure { job_kind, .. } | JobOutcome::Timeout { job_kind, .. }),
                ..
            } => {
                if expected_kinds.contains(job_kind) {
                    Self::IdleFailure
                } else {
                    Self::IdleNoOutcome
                }
            }
            ProcessState::Idle {
                last_outcome: None, ..
            } => Self::IdleNoOutcome,
        }
    }
}

impl DecisionFacts {
    pub(crate) fn start_postgres_can_be_requested(&self) -> bool {
        !matches!(self.process_state, ProcessState::Running { .. })
    }

    pub(crate) fn rewind_activity(&self) -> ProcessActivity {
        ProcessActivity::from_process_state(&self.process_state, &[ActiveJobKind::PgRewind])
    }

    pub(crate) fn bootstrap_activity(&self) -> ProcessActivity {
        ProcessActivity::from_process_state(
            &self.process_state,
            &[ActiveJobKind::BaseBackup, ActiveJobKind::Bootstrap],
        )
    }

    pub(crate) fn fencing_activity(&self) -> ProcessActivity {
        ProcessActivity::from_process_state(&self.process_state, &[ActiveJobKind::Fencing])
    }
}

impl PhaseOutcome {
    pub(crate) fn new(next_phase: HaPhase, decision: HaDecision) -> Self {
        Self {
            next_phase,
            decision,
        }
    }
}

impl HaDecision {
    pub(crate) fn label(&self) -> &'static str {
        match self {
            Self::NoChange => "no_change",
            Self::WaitForPostgres { .. } => "wait_for_postgres",
            Self::WaitForDcsTrust => "wait_for_dcs_trust",
            Self::AttemptLeadership => "attempt_leadership",
            Self::FollowLeader { .. } => "follow_leader",
            Self::BecomePrimary { .. } => "become_primary",
            Self::StepDown(_) => "step_down",
            Self::RecoverReplica { .. } => "recover_replica",
            Self::FenceNode => "fence_node",
            Self::ReleaseLeaderLease { .. } => "release_leader_lease",
            Self::EnterFailSafe { .. } => "enter_fail_safe",
        }
    }

    pub(crate) fn detail(&self) -> Option<String> {
        match self {
            Self::NoChange | Self::WaitForDcsTrust | Self::AttemptLeadership | Self::FenceNode => {
                None
            }
            Self::WaitForPostgres {
                start_requested,
                leader_member_id,
            } => {
                let leader_detail = leader_member_id
                    .as_ref()
                    .map(|leader| leader.0.as_str())
                    .unwrap_or("none");
                Some(format!(
                    "start_requested={start_requested}, leader_member_id={leader_detail}"
                ))
            }
            Self::FollowLeader { leader_member_id } => Some(leader_member_id.0.clone()),
            Self::BecomePrimary { promote } => Some(format!("promote={promote}")),
            Self::StepDown(plan) => Some(format!(
                "reason={}, release_leader_lease={}, clear_switchover={}, fence={}",
                plan.reason.label(),
                plan.release_leader_lease,
                plan.clear_switchover,
                plan.fence
            )),
            Self::RecoverReplica { strategy } => Some(strategy.label()),
            Self::ReleaseLeaderLease { reason } => Some(reason.label()),
            Self::EnterFailSafe {
                release_leader_lease,
            } => Some(format!("release_leader_lease={release_leader_lease}")),
        }
    }
}

impl StepDownReason {
    fn label(&self) -> String {
        match self {
            Self::Switchover => "switchover".to_string(),
            Self::ForeignLeaderDetected { leader_member_id } => {
                format!("foreign_leader_detected:{}", leader_member_id.0)
            }
        }
    }
}

impl RecoveryStrategy {
    fn label(&self) -> String {
        match self {
            Self::Rewind { leader_member_id } => format!("rewind:{}", leader_member_id.0),
            Self::BaseBackup { leader_member_id } => {
                format!("base_backup:{}", leader_member_id.0)
            }
            Self::Bootstrap => "bootstrap".to_string(),
        }
    }
}

impl LeaseReleaseReason {
    fn label(&self) -> String {
        match self {
            Self::FencingComplete => "fencing_complete".to_string(),
            Self::PostgresUnreachable => "postgres_unreachable".to_string(),
        }
    }
}

fn is_postgres_reachable(state: &PgInfoState) -> bool {
    let sql = match state {
        PgInfoState::Unknown { common } => &common.sql,
        PgInfoState::Primary { common, .. } => &common.sql,
        PgInfoState::Replica { common, .. } => &common.sql,
    };
    matches!(sql, SqlStatus::Healthy)
}

fn is_local_primary(state: &PgInfoState) -> bool {
    matches!(
        state,
        PgInfoState::Primary {
            common,
            ..
        } if matches!(common.sql, SqlStatus::Healthy)
    )
}

fn should_rewind_from_leader(world: &WorldSnapshot, leader_member_id: &MemberId) -> bool {
    let Some(local_timeline) = pg_timeline(&world.pg.value) else {
        return false;
    };

    let leader_timeline = world
        .dcs
        .value
        .cache
        .members
        .get(leader_member_id)
        .and_then(|member| member.timeline);

    leader_timeline
        .map(|timeline| timeline != local_timeline)
        .unwrap_or(false)
}

fn pg_timeline(state: &PgInfoState) -> Option<TimelineId> {
    match state {
        PgInfoState::Unknown { common } => common.timeline,
        PgInfoState::Primary { common, .. } => common.timeline,
        PgInfoState::Replica { common, .. } => common.timeline,
    }
}

fn is_available_primary_leader(world: &WorldSnapshot, leader_member_id: &MemberId) -> bool {
    let leader_record = world.dcs.value.cache.members.get(leader_member_id);

    let Some(member) = leader_record else {
        // Preserve current behavior when leader member metadata is not yet observed.
        return true;
    };

    matches!(member.role, crate::dcs::state::MemberRole::Primary)
        && matches!(member.sql, SqlStatus::Healthy)
}

--- END FILE: src/ha/decision.rs ---

--- BEGIN FILE: src/ha/decide.rs ---
use crate::{dcs::state::DcsTrust, process::jobs::ActiveJobKind, state::MemberId};

use super::{
    decision::{
        DecisionFacts, HaDecision, LeaseReleaseReason, PhaseOutcome, ProcessActivity,
        RecoveryStrategy, StepDownPlan, StepDownReason,
    },
    state::{DecideInput, DecideOutput, HaPhase, HaState},
};

pub(crate) fn decide(input: DecideInput) -> DecideOutput {
    let facts = DecisionFacts::from_world(&input.world);
    let current = input.current;
    let outcome = decide_phase(&current, &facts);
    let next = HaState {
        worker: current.worker,
        phase: outcome.next_phase.clone(),
        tick: current.tick.saturating_add(1),
        decision: outcome.decision.clone(),
    };

    DecideOutput { next, outcome }
}

pub(crate) fn decide_phase(current: &HaState, facts: &DecisionFacts) -> PhaseOutcome {
    if !matches!(facts.trust, DcsTrust::FullQuorum) {
        if facts.postgres_primary {
            return PhaseOutcome::new(
                HaPhase::FailSafe,
                HaDecision::EnterFailSafe {
                    release_leader_lease: false,
                },
            );
        }
        return PhaseOutcome::new(HaPhase::FailSafe, HaDecision::NoChange);
    }

    match current.phase {
        HaPhase::Init => PhaseOutcome::new(
            HaPhase::WaitingPostgresReachable,
            HaDecision::WaitForPostgres {
                start_requested: false,
                leader_member_id: None,
            },
        ),
        HaPhase::WaitingPostgresReachable => decide_waiting_postgres_reachable(facts),
        HaPhase::WaitingDcsTrusted => decide_waiting_dcs_trusted(current, facts),
        HaPhase::WaitingSwitchoverSuccessor => decide_waiting_switchover_successor(facts),
        HaPhase::Replica => decide_replica(facts),
        HaPhase::CandidateLeader => decide_candidate_leader(facts),
        HaPhase::Primary => decide_primary(facts),
        HaPhase::Rewinding => decide_rewinding(facts),
        HaPhase::Bootstrapping => decide_bootstrapping(facts),
        HaPhase::Fencing => decide_fencing(facts),
        HaPhase::FailSafe => decide_fail_safe(facts),
    }
}

fn decide_waiting_postgres_reachable(facts: &DecisionFacts) -> PhaseOutcome {
    if facts.postgres_reachable {
        return PhaseOutcome::new(HaPhase::WaitingDcsTrusted, HaDecision::WaitForDcsTrust);
    }

    if completed_start_postgres(facts) {
        return PhaseOutcome::new(HaPhase::WaitingDcsTrusted, HaDecision::WaitForDcsTrust);
    }

    wait_for_postgres(facts)
}

fn decide_waiting_dcs_trusted(current: &HaState, facts: &DecisionFacts) -> PhaseOutcome {
    if !facts.postgres_reachable {
        let released_after_fencing = matches!(
            current.decision,
            HaDecision::ReleaseLeaderLease {
                reason: LeaseReleaseReason::FencingComplete,
            }
        );
        if released_after_fencing {
            if let Some(leader_member_id) =
                recovery_leader_member_id(facts).or_else(|| other_leader_record(facts))
            {
                return PhaseOutcome::new(
                    HaPhase::Bootstrapping,
                    HaDecision::RecoverReplica {
                        strategy: RecoveryStrategy::BaseBackup { leader_member_id },
                    },
                );
            }

            return PhaseOutcome::new(HaPhase::WaitingDcsTrusted, HaDecision::WaitForDcsTrust);
        }

        return wait_for_postgres(facts);
    }

    if facts.active_leader_member_id.as_ref() == Some(&facts.self_member_id) {
        return PhaseOutcome::new(
            HaPhase::Primary,
            HaDecision::BecomePrimary { promote: false },
        );
    }

    match follow_target(facts) {
        Some(leader_member_id) => PhaseOutcome::new(
            HaPhase::Replica,
            HaDecision::FollowLeader { leader_member_id },
        ),
        None if !facts.postgres_primary => {
            PhaseOutcome::new(HaPhase::WaitingDcsTrusted, HaDecision::WaitForDcsTrust)
        }
        None => PhaseOutcome::new(HaPhase::CandidateLeader, HaDecision::AttemptLeadership),
    }
}

fn decide_waiting_switchover_successor(facts: &DecisionFacts) -> PhaseOutcome {
    if facts
        .leader_member_id
        .as_ref()
        .map(|leader_member_id| leader_member_id == &facts.self_member_id)
        .unwrap_or(true)
    {
        return PhaseOutcome::new(
            HaPhase::WaitingSwitchoverSuccessor,
            HaDecision::WaitForDcsTrust,
        );
    }

    if !facts.postgres_reachable {
        return wait_for_postgres(facts);
    }

    match follow_target(facts) {
        Some(leader_member_id) => PhaseOutcome::new(
            HaPhase::Replica,
            HaDecision::FollowLeader { leader_member_id },
        ),
        None => PhaseOutcome::new(
            HaPhase::WaitingSwitchoverSuccessor,
            HaDecision::WaitForDcsTrust,
        ),
    }
}

fn decide_replica(facts: &DecisionFacts) -> PhaseOutcome {
    if !facts.postgres_reachable {
        return wait_for_postgres(facts);
    }

    if facts.switchover_requested_by.is_some()
        && facts.active_leader_member_id.as_ref() == Some(&facts.self_member_id)
    {
        return PhaseOutcome::new(HaPhase::Replica, HaDecision::NoChange);
    }

    match facts.active_leader_member_id.as_ref() {
        Some(leader_member_id) if leader_member_id == &facts.self_member_id => PhaseOutcome::new(
            HaPhase::Primary,
            HaDecision::BecomePrimary { promote: true },
        ),
        Some(leader_member_id) if facts.rewind_required => PhaseOutcome::new(
            HaPhase::Rewinding,
            HaDecision::RecoverReplica {
                strategy: RecoveryStrategy::Rewind {
                    leader_member_id: leader_member_id.clone(),
                },
            },
        ),
        Some(leader_member_id) => PhaseOutcome::new(
            HaPhase::Replica,
            HaDecision::FollowLeader {
                leader_member_id: leader_member_id.clone(),
            },
        ),
        None => PhaseOutcome::new(HaPhase::CandidateLeader, HaDecision::AttemptLeadership),
    }
}

fn decide_candidate_leader(facts: &DecisionFacts) -> PhaseOutcome {
    if !facts.postgres_reachable {
        return wait_for_postgres(facts);
    }

    if facts.i_am_leader {
        return PhaseOutcome::new(
            HaPhase::Primary,
            HaDecision::BecomePrimary {
                promote: !facts.postgres_primary,
            },
        );
    }

    if let Some(leader_member_id) = follow_target(facts) {
        return PhaseOutcome::new(
            HaPhase::Replica,
            HaDecision::FollowLeader { leader_member_id },
        );
    }

    PhaseOutcome::new(HaPhase::CandidateLeader, HaDecision::AttemptLeadership)
}

fn decide_primary(facts: &DecisionFacts) -> PhaseOutcome {
    if facts.switchover_requested_by.is_some() && facts.i_am_leader {
        return PhaseOutcome::new(
            HaPhase::WaitingSwitchoverSuccessor,
            HaDecision::StepDown(StepDownPlan {
                reason: StepDownReason::Switchover,
                release_leader_lease: true,
                clear_switchover: true,
                fence: false,
            }),
        );
    }

    if !facts.postgres_reachable {
        if facts.i_am_leader {
            return PhaseOutcome::new(
                HaPhase::Rewinding,
                HaDecision::ReleaseLeaderLease {
                    reason: LeaseReleaseReason::PostgresUnreachable,
                },
            );
        }
        return match recovery_leader_member_id(facts) {
            Some(leader_member_id) => PhaseOutcome::new(
                HaPhase::Rewinding,
                HaDecision::RecoverReplica {
                    strategy: RecoveryStrategy::Rewind { leader_member_id },
                },
            ),
            None => PhaseOutcome::new(HaPhase::Rewinding, HaDecision::NoChange),
        };
    }

    match other_active_leader(facts) {
        Some(leader_member_id) => PhaseOutcome::new(
            HaPhase::Fencing,
            HaDecision::StepDown(StepDownPlan {
                reason: StepDownReason::ForeignLeaderDetected { leader_member_id },
                release_leader_lease: true,
                clear_switchover: false,
                fence: true,
            }),
        ),
        None => {
            if facts.i_am_leader {
                PhaseOutcome::new(HaPhase::Primary, HaDecision::NoChange)
            } else {
                PhaseOutcome::new(HaPhase::Primary, HaDecision::AttemptLeadership)
            }
        }
    }
}

fn decide_rewinding(facts: &DecisionFacts) -> PhaseOutcome {
    match facts.rewind_activity() {
        ProcessActivity::Running => PhaseOutcome::new(HaPhase::Rewinding, HaDecision::NoChange),
        ProcessActivity::IdleSuccess => match follow_target(facts) {
            Some(leader_member_id) => PhaseOutcome::new(
                HaPhase::Replica,
                HaDecision::FollowLeader { leader_member_id },
            ),
            None => PhaseOutcome::new(HaPhase::Replica, HaDecision::NoChange),
        },
        ProcessActivity::IdleFailure => match recovery_after_rewind_failure(facts) {
            Some(strategy) => PhaseOutcome::new(
                HaPhase::Bootstrapping,
                HaDecision::RecoverReplica { strategy },
            ),
            None => PhaseOutcome::new(HaPhase::Rewinding, HaDecision::NoChange),
        },
        ProcessActivity::IdleNoOutcome => match recovery_leader_member_id(facts) {
            Some(leader_member_id) => PhaseOutcome::new(
                HaPhase::Rewinding,
                HaDecision::RecoverReplica {
                    strategy: RecoveryStrategy::Rewind { leader_member_id },
                },
            ),
            None => PhaseOutcome::new(HaPhase::Rewinding, HaDecision::NoChange),
        },
    }
}

fn decide_bootstrapping(facts: &DecisionFacts) -> PhaseOutcome {
    match facts.bootstrap_activity() {
        ProcessActivity::Running => PhaseOutcome::new(HaPhase::Bootstrapping, HaDecision::NoChange),
        ProcessActivity::IdleSuccess => wait_for_postgres(facts),
        ProcessActivity::IdleFailure => PhaseOutcome::new(HaPhase::Fencing, HaDecision::FenceNode),
        ProcessActivity::IdleNoOutcome => match recovery_after_rewind_failure(facts) {
            Some(strategy) => PhaseOutcome::new(
                HaPhase::Bootstrapping,
                HaDecision::RecoverReplica { strategy },
            ),
            None => PhaseOutcome::new(HaPhase::Bootstrapping, HaDecision::NoChange),
        },
    }
}

fn decide_fencing(facts: &DecisionFacts) -> PhaseOutcome {
    match facts.fencing_activity() {
        ProcessActivity::Running => PhaseOutcome::new(HaPhase::Fencing, HaDecision::NoChange),
        ProcessActivity::IdleSuccess => PhaseOutcome::new(
            HaPhase::WaitingDcsTrusted,
            HaDecision::ReleaseLeaderLease {
                reason: LeaseReleaseReason::FencingComplete,
            },
        ),
        ProcessActivity::IdleFailure => PhaseOutcome::new(
            HaPhase::FailSafe,
            HaDecision::EnterFailSafe {
                release_leader_lease: false,
            },
        ),
        ProcessActivity::IdleNoOutcome => {
            PhaseOutcome::new(HaPhase::Fencing, HaDecision::FenceNode)
        }
    }
}

fn decide_fail_safe(facts: &DecisionFacts) -> PhaseOutcome {
    match facts.fencing_activity() {
        ProcessActivity::Running => PhaseOutcome::new(HaPhase::FailSafe, HaDecision::NoChange),
        _ if facts.postgres_primary => decide_primary(facts),
        _ if facts.i_am_leader => PhaseOutcome::new(
            HaPhase::FailSafe,
            HaDecision::ReleaseLeaderLease {
                reason: LeaseReleaseReason::FencingComplete,
            },
        ),
        _ => PhaseOutcome::new(HaPhase::WaitingDcsTrusted, HaDecision::WaitForDcsTrust),
    }
}

fn wait_for_postgres(facts: &DecisionFacts) -> PhaseOutcome {
    PhaseOutcome::new(
        HaPhase::WaitingPostgresReachable,
        HaDecision::WaitForPostgres {
            start_requested: facts.start_postgres_can_be_requested(),
            leader_member_id: recovery_leader_member_id(facts)
                .or_else(|| other_leader_record(facts)),
        },
    )
}

fn recovery_after_rewind_failure(facts: &DecisionFacts) -> Option<RecoveryStrategy> {
    recovery_leader_member_id(facts)
        .map(|leader_member_id| RecoveryStrategy::BaseBackup { leader_member_id })
}

fn recovery_leader_member_id(facts: &DecisionFacts) -> Option<MemberId> {
    facts
        .available_primary_member_id
        .clone()
        .filter(|leader_member_id| leader_member_id != &facts.self_member_id)
}

fn follow_target(facts: &DecisionFacts) -> Option<MemberId> {
    facts
        .available_primary_member_id
        .clone()
        .filter(|leader_member_id| leader_member_id != &facts.self_member_id)
}

fn other_leader_record(facts: &DecisionFacts) -> Option<MemberId> {
    facts
        .leader_member_id
        .clone()
        .filter(|leader_member_id| leader_member_id != &facts.self_member_id)
}

fn other_active_leader(facts: &DecisionFacts) -> Option<MemberId> {
    facts
        .active_leader_member_id
        .clone()
        .filter(|leader_member_id| leader_member_id != &facts.self_member_id)
}

fn completed_start_postgres(facts: &DecisionFacts) -> bool {
    matches!(
        &facts.process_state,
        crate::process::state::ProcessState::Idle {
            last_outcome: Some(
                crate::process::state::JobOutcome::Success {
                    job_kind: ActiveJobKind::StartPostgres,
                    ..
                } | crate::process::state::JobOutcome::Failure {
                    job_kind: ActiveJobKind::StartPostgres,
                    ..
                } | crate::process::state::JobOutcome::Timeout {
                    job_kind: ActiveJobKind::StartPostgres,
                    ..
                }
            ),
            ..
        }
    )
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        dcs::state::{
            DcsCache, DcsState, DcsTrust, LeaderRecord, MemberRecord, MemberRole, SwitchoverRequest,
        },
        ha::{
            decision::{
                HaDecision, LeaseReleaseReason, RecoveryStrategy, StepDownPlan, StepDownReason,
            },
            lower::{
                lower_decision, HaEffectPlan, LeaseEffect, PostgresEffect, ReplicationEffect,
                SafetyEffect, SwitchoverEffect,
            },
            state::{DecideInput, HaPhase, HaState, WorldSnapshot},
        },
        pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
        process::{
            jobs::{ActiveJob, ActiveJobKind},
            state::{JobOutcome, ProcessState},
        },
        state::{JobId, MemberId, UnixMillis, Version, Versioned, WorkerStatus},
    };

    use super::decide;

    #[derive(Clone)]
    struct WorldBuilder {
        trust: DcsTrust,
        pg: PgInfoState,
        leader: Option<MemberId>,
        process: ProcessState,
        members: BTreeMap<MemberId, MemberRecord>,
        switchover_requested_by: Option<MemberId>,
    }

    impl WorldBuilder {
        fn new() -> Self {
            Self {
                trust: DcsTrust::FullQuorum,
                pg: pg_replica(SqlStatus::Healthy),
                leader: None,
                process: process_idle(None),
                members: BTreeMap::new(),
                switchover_requested_by: None,
            }
        }

        fn with_trust(self, trust: DcsTrust) -> Self {
            Self { trust, ..self }
        }

        fn with_pg(self, pg: PgInfoState) -> Self {
            Self { pg, ..self }
        }

        fn with_process(self, process: ProcessState) -> Self {
            Self { process, ..self }
        }

        fn with_leader(self, leader_member_id: &str) -> Self {
            Self {
                leader: Some(MemberId(leader_member_id.to_string())),
                ..self
            }
        }

        fn with_switchover_request(self, requested_by: &str) -> Self {
            Self {
                switchover_requested_by: Some(MemberId(requested_by.to_string())),
                ..self
            }
        }

        fn with_member(self, record: MemberRecord) -> Self {
            let members = self
                .members
                .into_iter()
                .chain(std::iter::once((record.member_id.clone(), record)))
                .collect();
            Self { members, ..self }
        }

        fn build(self) -> WorldSnapshot {
            world(
                self.trust,
                self.pg,
                self.leader,
                self.process,
                self.members,
                self.switchover_requested_by,
            )
        }
    }

    struct Case {
        name: &'static str,
        current_phase: HaPhase,
        trust: DcsTrust,
        pg: PgInfoState,
        leader: Option<&'static str>,
        process: ProcessState,
        expected_phase: HaPhase,
        expected_decision: HaDecision,
    }

    #[test]
    fn transition_matrix_cases() {
        let cases = vec![
            Case {
                name: "init moves to waiting postgres",
                current_phase: HaPhase::Init,
                trust: DcsTrust::FullQuorum,
                pg: pg_unknown(SqlStatus::Unknown),
                leader: None,
                process: process_idle(None),
                expected_phase: HaPhase::WaitingPostgresReachable,
                expected_decision: HaDecision::WaitForPostgres {
                    start_requested: false,
                    leader_member_id: None,
                },
            },
            Case {
                name: "waiting postgres emits start when unreachable",
                current_phase: HaPhase::WaitingPostgresReachable,
                trust: DcsTrust::FullQuorum,
                pg: pg_unknown(SqlStatus::Unreachable),
                leader: None,
                process: process_idle(None),
                expected_phase: HaPhase::WaitingPostgresReachable,
                expected_decision: HaDecision::WaitForPostgres {
                    start_requested: true,
                    leader_member_id: None,
                },
            },
            Case {
                name: "waiting postgres enters dcs trusted when healthy",
                current_phase: HaPhase::WaitingPostgresReachable,
                trust: DcsTrust::FullQuorum,
                pg: pg_replica(SqlStatus::Healthy),
                leader: None,
                process: process_idle(None),
                expected_phase: HaPhase::WaitingDcsTrusted,
                expected_decision: HaDecision::WaitForDcsTrust,
            },
            Case {
                name: "waiting dcs to replica with known leader",
                current_phase: HaPhase::WaitingDcsTrusted,
                trust: DcsTrust::FullQuorum,
                pg: pg_replica(SqlStatus::Healthy),
                leader: Some("node-b"),
                process: process_idle(None),
                expected_phase: HaPhase::Replica,
                expected_decision: HaDecision::FollowLeader {
                    leader_member_id: MemberId("node-b".to_string()),
                },
            },
            Case {
                name: "waiting dcs replica without leader stays waiting",
                current_phase: HaPhase::WaitingDcsTrusted,
                trust: DcsTrust::FullQuorum,
                pg: pg_replica(SqlStatus::Healthy),
                leader: None,
                process: process_idle(None),
                expected_phase: HaPhase::WaitingDcsTrusted,
                expected_decision: HaDecision::WaitForDcsTrust,
            },
            Case {
                name: "candidate becomes primary when lease self",
                current_phase: HaPhase::CandidateLeader,
                trust: DcsTrust::FullQuorum,
                pg: pg_replica(SqlStatus::Healthy),
                leader: Some("node-a"),
                process: process_idle(None),
                expected_phase: HaPhase::Primary,
                expected_decision: HaDecision::BecomePrimary { promote: true },
            },
            Case {
                name: "primary split brain fences",
                current_phase: HaPhase::Primary,
                trust: DcsTrust::FullQuorum,
                pg: pg_primary(SqlStatus::Healthy),
                leader: Some("node-b"),
                process: process_idle(None),
                expected_phase: HaPhase::Fencing,
                expected_decision: HaDecision::StepDown(StepDownPlan {
                    reason: StepDownReason::ForeignLeaderDetected {
                        leader_member_id: MemberId("node-b".to_string()),
                    },
                    release_leader_lease: true,
                    clear_switchover: false,
                    fence: true,
                }),
            },
            Case {
                name: "no quorum enters fail safe",
                current_phase: HaPhase::CandidateLeader,
                trust: DcsTrust::FailSafe,
                pg: pg_replica(SqlStatus::Healthy),
                leader: None,
                process: process_idle(None),
                expected_phase: HaPhase::FailSafe,
                expected_decision: HaDecision::NoChange,
            },
            Case {
                name: "rewinding success re-enters replica",
                current_phase: HaPhase::Rewinding,
                trust: DcsTrust::FullQuorum,
                pg: pg_replica(SqlStatus::Healthy),
                leader: Some("node-b"),
                process: process_idle(Some(JobOutcome::Success {
                    id: JobId("job-1".to_string()),
                    job_kind: ActiveJobKind::PgRewind,
                    finished_at: UnixMillis(10),
                })),
                expected_phase: HaPhase::Replica,
                expected_decision: HaDecision::FollowLeader {
                    leader_member_id: MemberId("node-b".to_string()),
                },
            },
            Case {
                name: "rewinding failure goes bootstrap",
                current_phase: HaPhase::Rewinding,
                trust: DcsTrust::FullQuorum,
                pg: pg_replica(SqlStatus::Healthy),
                leader: Some("node-b"),
                process: process_idle(Some(JobOutcome::Failure {
                    id: JobId("job-1".to_string()),
                    job_kind: ActiveJobKind::PgRewind,
                    error: crate::process::jobs::ProcessError::OperationFailed,
                    finished_at: UnixMillis(10),
                })),
                expected_phase: HaPhase::Bootstrapping,
                expected_decision: HaDecision::RecoverReplica {
                    strategy: RecoveryStrategy::BaseBackup {
                        leader_member_id: MemberId("node-b".to_string()),
                    },
                },
            },
            Case {
                name: "rewinding failure without active leader waits",
                current_phase: HaPhase::Rewinding,
                trust: DcsTrust::FullQuorum,
                pg: pg_replica(SqlStatus::Healthy),
                leader: None,
                process: process_idle(Some(JobOutcome::Failure {
                    id: JobId("job-1".to_string()),
                    job_kind: ActiveJobKind::PgRewind,
                    error: crate::process::jobs::ProcessError::OperationFailed,
                    finished_at: UnixMillis(10),
                })),
                expected_phase: HaPhase::Rewinding,
                expected_decision: HaDecision::NoChange,
            },
            Case {
                name: "bootstrap failure goes fencing",
                current_phase: HaPhase::Bootstrapping,
                trust: DcsTrust::FullQuorum,
                pg: pg_replica(SqlStatus::Healthy),
                leader: Some("node-b"),
                process: process_idle(Some(JobOutcome::Timeout {
                    id: JobId("job-1".to_string()),
                    job_kind: ActiveJobKind::Bootstrap,
                    finished_at: UnixMillis(11),
                })),
                expected_phase: HaPhase::Fencing,
                expected_decision: HaDecision::FenceNode,
            },
            Case {
                name: "bootstrapping without active leader emits nothing",
                current_phase: HaPhase::Bootstrapping,
                trust: DcsTrust::FullQuorum,
                pg: pg_replica(SqlStatus::Healthy),
                leader: None,
                process: process_idle(None),
                expected_phase: HaPhase::Bootstrapping,
                expected_decision: HaDecision::NoChange,
            },
            Case {
                name: "fencing success returns waiting dcs",
                current_phase: HaPhase::Fencing,
                trust: DcsTrust::FullQuorum,
                pg: pg_replica(SqlStatus::Healthy),
                leader: Some("node-b"),
                process: process_idle(Some(JobOutcome::Success {
                    id: JobId("job-2".to_string()),
                    job_kind: ActiveJobKind::Fencing,
                    finished_at: UnixMillis(12),
                })),
                expected_phase: HaPhase::WaitingDcsTrusted,
                expected_decision: HaDecision::ReleaseLeaderLease {
                    reason: LeaseReleaseReason::FencingComplete,
                },
            },
            Case {
                name: "fencing failure enters fail safe",
                current_phase: HaPhase::Fencing,
                trust: DcsTrust::FullQuorum,
                pg: pg_replica(SqlStatus::Healthy),
                leader: Some("node-b"),
                process: process_idle(Some(JobOutcome::Failure {
                    id: JobId("job-2".to_string()),
                    job_kind: ActiveJobKind::Fencing,
                    error: crate::process::jobs::ProcessError::OperationFailed,
                    finished_at: UnixMillis(12),
                })),
                expected_phase: HaPhase::FailSafe,
                expected_decision: HaDecision::EnterFailSafe {
                    release_leader_lease: false,
                },
            },
        ];

        for case in cases {
            let input = DecideInput {
                current: HaState {
                    worker: WorkerStatus::Running,
                    phase: case.current_phase.clone(),
                    tick: 41,
                    decision: HaDecision::NoChange,
                },
                world: WorldBuilder::new()
                    .with_trust(case.trust)
                    .with_pg(case.pg.clone())
                    .with_process(process_clone(&case.process))
                    .build_with_optional_leader(case.leader),
            };

            let output = decide(input);
            assert_eq!(
                output.next.phase, case.expected_phase,
                "case: {}",
                case.name
            );
            assert_eq!(
                output.outcome.decision, case.expected_decision,
                "case: {}",
                case.name
            );
            assert_eq!(
                output.next.decision, case.expected_decision,
                "case: {}",
                case.name
            );
            assert_eq!(output.next.tick, 42, "case: {}", case.name);
        }
    }

    #[test]
    fn actions_are_reissued_while_conditions_persist() {
        let current = HaState {
            worker: WorkerStatus::Running,
            phase: HaPhase::WaitingDcsTrusted,
            tick: 0,
            decision: HaDecision::NoChange,
        };
        let world = WorldBuilder::new()
            .with_pg(pg_primary(SqlStatus::Healthy))
            .build();

        let first = decide(DecideInput {
            current: current.clone(),
            world: world.clone(),
        });
        assert_eq!(
            lower_decision(&first.outcome.decision),
            HaEffectPlan {
                lease: LeaseEffect::AcquireLeader,
                switchover: SwitchoverEffect::None,
                replication: ReplicationEffect::None,
                postgres: PostgresEffect::None,
                safety: SafetyEffect::None,
            }
        );

        let second = decide(DecideInput {
            current: first.next,
            world,
        });
        assert_eq!(
            lower_decision(&second.outcome.decision),
            HaEffectPlan {
                lease: LeaseEffect::AcquireLeader,
                switchover: SwitchoverEffect::None,
                replication: ReplicationEffect::None,
                postgres: PostgresEffect::None,
                safety: SafetyEffect::None,
            }
        );
    }

    #[test]
    fn fail_safe_holds_without_quorum_and_exits_when_restored() {
        let start = HaState {
            worker: WorkerStatus::Running,
            phase: HaPhase::FailSafe,
            tick: 100,
            decision: HaDecision::NoChange,
        };

        let held = decide(DecideInput {
            current: start.clone(),
            world: WorldBuilder::new().with_trust(DcsTrust::NotTrusted).build(),
        });
        assert_eq!(held.next.phase, HaPhase::FailSafe);
        assert_eq!(held.outcome.decision, HaDecision::NoChange);

        let recovered = decide(DecideInput {
            current: start,
            world: WorldBuilder::new().with_trust(DcsTrust::FullQuorum).build(),
        });
        assert_eq!(recovered.next.phase, HaPhase::WaitingDcsTrusted);
        assert_eq!(recovered.outcome.decision, HaDecision::WaitForDcsTrust);
    }

    #[test]
    fn no_quorum_failsafe_with_stale_self_lease_but_stopped_postgres_stays_quiescent() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::FailSafe,
                tick: 44,
                decision: HaDecision::NoChange,
            },
            world: WorldBuilder::new()
                .with_trust(DcsTrust::NotTrusted)
                .with_leader("node-a")
                .with_pg(pg_unknown(SqlStatus::Unreachable))
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::FailSafe);
        assert_eq!(output.outcome.decision, HaDecision::NoChange);
    }

    #[test]
    fn fail_safe_with_restored_quorum_and_stale_self_lease_retries_release_without_refencing() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::FailSafe,
                tick: 17,
                decision: HaDecision::NoChange,
            },
            world: WorldBuilder::new()
                .with_trust(DcsTrust::FullQuorum)
                .with_leader("node-a")
                .with_pg(pg_unknown(SqlStatus::Unreachable))
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::FailSafe);
        assert_eq!(
            output.outcome.decision,
            HaDecision::ReleaseLeaderLease {
                reason: LeaseReleaseReason::FencingComplete,
            }
        );
        assert_eq!(
            lower_decision(&output.outcome.decision).lease,
            LeaseEffect::ReleaseLeader
        );
        assert_eq!(
            lower_decision(&output.outcome.decision).safety,
            SafetyEffect::None
        );
    }

    #[test]
    fn primary_with_switchover_demotes_releases_and_clears_request() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::Primary,
                tick: 10,
                decision: HaDecision::NoChange,
            },
            world: WorldBuilder::new()
                .with_pg(pg_primary(SqlStatus::Healthy))
                .with_leader("node-a")
                .with_switchover_request("node-b")
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::WaitingSwitchoverSuccessor);
        assert_eq!(
            lower_decision(&output.outcome.decision),
            HaEffectPlan {
                lease: LeaseEffect::ReleaseLeader,
                switchover: SwitchoverEffect::ClearRequest,
                replication: ReplicationEffect::None,
                postgres: PostgresEffect::Demote,
                safety: SafetyEffect::None,
            }
        );
    }

    #[test]
    fn waiting_switchover_successor_holds_until_new_leader_exists() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::WaitingSwitchoverSuccessor,
                tick: 11,
                decision: HaDecision::WaitForDcsTrust,
            },
            world: WorldBuilder::new()
                .with_pg(pg_replica(SqlStatus::Healthy))
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::WaitingSwitchoverSuccessor);
        assert_eq!(output.outcome.decision, HaDecision::WaitForDcsTrust);
    }

    #[test]
    fn waiting_switchover_successor_does_not_restart_while_demote_runs() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::WaitingSwitchoverSuccessor,
                tick: 12,
                decision: HaDecision::WaitForDcsTrust,
            },
            world: WorldBuilder::new()
                .with_pg(pg_unknown(SqlStatus::Unreachable))
                .with_process(process_running(ActiveJobKind::Demote))
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::WaitingSwitchoverSuccessor);
        assert_eq!(output.outcome.decision, HaDecision::WaitForDcsTrust);
    }

    #[test]
    fn waiting_switchover_successor_follows_new_leader_once_visible() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::WaitingSwitchoverSuccessor,
                tick: 13,
                decision: HaDecision::WaitForDcsTrust,
            },
            world: WorldBuilder::new()
                .with_pg(pg_replica(SqlStatus::Healthy))
                .with_leader("node-b")
                .with_member(MemberRecord {
                    member_id: MemberId("node-b".to_string()),
                    postgres_host: "10.0.0.10".to_string(),
                    postgres_port: 5432,
                    role: MemberRole::Primary,
                    sql: SqlStatus::Healthy,
                    readiness: Readiness::Ready,
                    timeline: None,
                    write_lsn: None,
                    replay_lsn: None,
                    updated_at: UnixMillis(1),
                    pg_version: Version(1),
                })
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::Replica);
        assert_eq!(
            output.outcome.decision,
            HaDecision::FollowLeader {
                leader_member_id: MemberId("node-b".to_string()),
            }
        );
    }

    #[test]
    fn waiting_postgres_reachable_with_active_demote_does_not_request_start() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::WaitingPostgresReachable,
                tick: 21,
                decision: HaDecision::NoChange,
            },
            world: WorldBuilder::new()
                .with_pg(pg_unknown(SqlStatus::Unreachable))
                .with_process(process_running(ActiveJobKind::Demote))
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::WaitingPostgresReachable);
        assert_eq!(
            output.outcome.decision,
            HaDecision::WaitForPostgres {
                start_requested: false,
                leader_member_id: None,
            }
        );
    }

    #[test]
    fn waiting_dcs_trusted_after_fencing_with_known_leader_reenters_basebackup() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::WaitingDcsTrusted,
                tick: 34,
                decision: HaDecision::ReleaseLeaderLease {
                    reason: LeaseReleaseReason::FencingComplete,
                },
            },
            world: WorldBuilder::new()
                .with_pg(pg_unknown(SqlStatus::Unreachable))
                .with_leader("node-b")
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::Bootstrapping);
        assert_eq!(
            output.outcome.decision,
            HaDecision::RecoverReplica {
                strategy: RecoveryStrategy::BaseBackup {
                    leader_member_id: MemberId("node-b".to_string()),
                },
            }
        );
    }

    #[test]
    fn waiting_dcs_trusted_with_wait_for_dcs_and_known_leader_retries_postgres() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::WaitingDcsTrusted,
                tick: 35,
                decision: HaDecision::WaitForDcsTrust,
            },
            world: WorldBuilder::new()
                .with_pg(pg_unknown(SqlStatus::Unreachable))
                .with_leader("node-b")
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::WaitingPostgresReachable);
        assert_eq!(
            output.outcome.decision,
            HaDecision::WaitForPostgres {
                start_requested: true,
                leader_member_id: Some(MemberId("node-b".to_string())),
            }
        );
    }

    #[test]
    fn bootstrapping_success_waits_for_postgres_before_becoming_replica() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::Bootstrapping,
                tick: 35,
                decision: HaDecision::RecoverReplica {
                    strategy: RecoveryStrategy::BaseBackup {
                        leader_member_id: MemberId("node-b".to_string()),
                    },
                },
            },
            world: WorldBuilder::new()
                .with_pg(pg_unknown(SqlStatus::Unreachable))
                .with_leader("node-b")
                .with_process(process_idle(Some(JobOutcome::Success {
                    id: JobId("job-basebackup".to_string()),
                    job_kind: ActiveJobKind::BaseBackup,
                    finished_at: UnixMillis(35),
                })))
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::WaitingPostgresReachable);
        assert_eq!(
            output.outcome.decision,
            HaDecision::WaitForPostgres {
                start_requested: true,
                leader_member_id: Some(MemberId("node-b".to_string())),
            }
        );
    }

    #[test]
    fn waiting_dcs_trusted_without_leader_follows_healthy_primary_member() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::WaitingDcsTrusted,
                tick: 35,
                decision: HaDecision::WaitForDcsTrust,
            },
            world: WorldBuilder::new()
                .with_pg(pg_replica(SqlStatus::Healthy))
                .with_member(MemberRecord {
                    member_id: MemberId("node-b".to_string()),
                    postgres_host: "10.0.0.20".to_string(),
                    postgres_port: 5432,
                    role: MemberRole::Primary,
                    sql: SqlStatus::Healthy,
                    readiness: Readiness::Ready,
                    timeline: None,
                    write_lsn: None,
                    replay_lsn: None,
                    updated_at: UnixMillis(1),
                    pg_version: Version(1),
                })
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::Replica);
        assert_eq!(
            output.outcome.decision,
            HaDecision::FollowLeader {
                leader_member_id: MemberId("node-b".to_string()),
            }
        );
    }

    #[test]
    fn waiting_dcs_trusted_after_fencing_without_leader_waits_for_dcs() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::WaitingDcsTrusted,
                tick: 35,
                decision: HaDecision::ReleaseLeaderLease {
                    reason: LeaseReleaseReason::FencingComplete,
                },
            },
            world: WorldBuilder::new()
                .with_pg(pg_unknown(SqlStatus::Unreachable))
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::WaitingDcsTrusted);
        assert_eq!(output.outcome.decision, HaDecision::WaitForDcsTrust);
    }

    #[test]
    fn waiting_dcs_trusted_after_fencing_uses_stale_foreign_leader_record_for_basebackup() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::WaitingDcsTrusted,
                tick: 36,
                decision: HaDecision::ReleaseLeaderLease {
                    reason: LeaseReleaseReason::FencingComplete,
                },
            },
            world: WorldBuilder::new()
                .with_pg(pg_unknown(SqlStatus::Unreachable))
                .with_member(MemberRecord {
                    member_id: MemberId("node-b".to_string()),
                    postgres_host: "10.0.0.10".to_string(),
                    postgres_port: 5432,
                    role: MemberRole::Replica,
                    sql: SqlStatus::Unreachable,
                    readiness: Readiness::NotReady,
                    timeline: None,
                    write_lsn: None,
                    replay_lsn: None,
                    updated_at: UnixMillis(1),
                    pg_version: Version(1),
                })
                .build_with_optional_leader(Some("node-b")),
        });

        assert_eq!(output.next.phase, HaPhase::Bootstrapping);
        assert_eq!(
            output.outcome.decision,
            HaDecision::RecoverReplica {
                strategy: RecoveryStrategy::BaseBackup {
                    leader_member_id: MemberId("node-b".to_string()),
                },
            }
        );
    }

    #[test]
    fn primary_without_leader_reacquires_lease_without_leaving_primary() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::Primary,
                tick: 12,
                decision: HaDecision::NoChange,
            },
            world: WorldBuilder::new()
                .with_pg(pg_primary(SqlStatus::Healthy))
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::Primary);
        assert_eq!(output.outcome.decision, HaDecision::AttemptLeadership);
        assert_eq!(
            lower_decision(&output.outcome.decision).lease,
            LeaseEffect::AcquireLeader
        );
    }

    #[test]
    fn replica_with_self_leader_and_pending_switchover_does_not_repromote() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::Replica,
                tick: 10,
                decision: HaDecision::NoChange,
            },
            world: WorldBuilder::new()
                .with_leader("node-a")
                .with_switchover_request("node-b")
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::Replica);
        assert_eq!(output.outcome.decision, HaDecision::NoChange);
        assert_eq!(
            lower_decision(&output.outcome.decision),
            HaEffectPlan::default()
        );
    }

    #[test]
    fn rewinding_while_running_emits_nothing() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::Rewinding,
                tick: 8,
                decision: HaDecision::NoChange,
            },
            world: WorldBuilder::new()
                .with_leader("node-b")
                .with_process(process_running(ActiveJobKind::PgRewind))
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::Rewinding);
        assert_eq!(lower_decision(&output.outcome.decision).len(), 0);
    }

    #[test]
    fn primary_ignores_unavailable_foreign_leader_record_and_reacquires_lease() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::Primary,
                tick: 12,
                decision: HaDecision::NoChange,
            },
            world: WorldBuilder::new()
                .with_pg(pg_primary(SqlStatus::Healthy))
                .with_member(MemberRecord {
                    member_id: MemberId("node-b".to_string()),
                    postgres_host: "10.0.0.20".to_string(),
                    postgres_port: 5432,
                    role: MemberRole::Replica,
                    sql: SqlStatus::Unreachable,
                    readiness: Readiness::NotReady,
                    timeline: None,
                    write_lsn: None,
                    replay_lsn: None,
                    updated_at: UnixMillis(1),
                    pg_version: Version(1),
                })
                .build_with_optional_leader(Some("node-b")),
        });

        assert_eq!(output.next.phase, HaPhase::Primary);
        assert_eq!(output.outcome.decision, HaDecision::AttemptLeadership);
    }

    #[test]
    fn primary_outage_without_foreign_leader_waits_in_rewinding_without_self_target() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::Primary,
                tick: 9,
                decision: HaDecision::NoChange,
            },
            world: WorldBuilder::new()
                .with_pg(pg_primary(SqlStatus::Unreachable))
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::Rewinding);
        assert_eq!(output.outcome.decision, HaDecision::NoChange);
        assert_eq!(lower_decision(&output.outcome.decision).len(), 0);
    }

    #[test]
    fn primary_outage_with_self_leader_releases_lease_before_rewinding() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::Primary,
                tick: 10,
                decision: HaDecision::NoChange,
            },
            world: WorldBuilder::new()
                .with_pg(pg_primary(SqlStatus::Unreachable))
                .with_leader("node-a")
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::Rewinding);
        assert_eq!(
            output.outcome.decision,
            HaDecision::ReleaseLeaderLease {
                reason: LeaseReleaseReason::PostgresUnreachable,
            }
        );
        assert_eq!(
            lower_decision(&output.outcome.decision).lease,
            LeaseEffect::ReleaseLeader
        );
        assert_eq!(
            lower_decision(&output.outcome.decision).replication,
            ReplicationEffect::None
        );
    }

    #[test]
    fn rewinding_without_foreign_leader_and_no_process_outcome_emits_nothing() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::Rewinding,
                tick: 10,
                decision: HaDecision::NoChange,
            },
            world: WorldBuilder::new()
                .with_pg(pg_primary(SqlStatus::Unreachable))
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::Rewinding);
        assert_eq!(output.outcome.decision, HaDecision::NoChange);
        assert_eq!(lower_decision(&output.outcome.decision).len(), 0);
    }

    #[test]
    fn rewinding_ignores_stale_start_postgres_failure_until_rewind_runs() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::Rewinding,
                tick: 11,
                decision: HaDecision::NoChange,
            },
            world: WorldBuilder::new()
                .with_leader("node-b")
                .with_process(process_idle(Some(JobOutcome::Failure {
                    id: JobId("job-start".to_string()),
                    job_kind: ActiveJobKind::StartPostgres,
                    error: crate::process::jobs::ProcessError::OperationFailed,
                    finished_at: UnixMillis(15),
                })))
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::Rewinding);
        assert_eq!(
            output.outcome.decision,
            HaDecision::RecoverReplica {
                strategy: RecoveryStrategy::Rewind {
                    leader_member_id: MemberId("node-b".to_string()),
                },
            }
        );
    }

    #[test]
    fn waiting_postgres_does_not_reissue_start_while_start_job_is_running() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::WaitingPostgresReachable,
                tick: 12,
                decision: HaDecision::NoChange,
            },
            world: WorldBuilder::new()
                .with_pg(pg_unknown(SqlStatus::Unreachable))
                .with_process(process_running(ActiveJobKind::StartPostgres))
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::WaitingPostgresReachable);
        assert_eq!(
            output.outcome.decision,
            HaDecision::WaitForPostgres {
                start_requested: false,
                leader_member_id: None,
            }
        );
    }

    #[test]
    fn waiting_postgres_after_failed_start_with_foreign_leader_waits_for_dcs() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::WaitingPostgresReachable,
                tick: 13,
                decision: HaDecision::WaitForPostgres {
                    start_requested: true,
                    leader_member_id: Some(MemberId("node-b".to_string())),
                },
            },
            world: WorldBuilder::new()
                .with_pg(pg_unknown(SqlStatus::Unreachable))
                .with_leader("node-b")
                .with_process(process_idle(Some(JobOutcome::Failure {
                    id: JobId("job-start".to_string()),
                    job_kind: ActiveJobKind::StartPostgres,
                    error: crate::process::jobs::ProcessError::OperationFailed,
                    finished_at: UnixMillis(16),
                })))
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::WaitingDcsTrusted);
        assert_eq!(output.outcome.decision, HaDecision::WaitForDcsTrust);
    }

    #[test]
    fn waiting_postgres_after_failed_start_without_foreign_leader_waits_for_dcs() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::WaitingPostgresReachable,
                tick: 14,
                decision: HaDecision::WaitForPostgres {
                    start_requested: true,
                    leader_member_id: None,
                },
            },
            world: WorldBuilder::new()
                .with_pg(pg_unknown(SqlStatus::Unreachable))
                .with_process(process_idle(Some(JobOutcome::Failure {
                    id: JobId("job-start".to_string()),
                    job_kind: ActiveJobKind::StartPostgres,
                    error: crate::process::jobs::ProcessError::OperationFailed,
                    finished_at: UnixMillis(16),
                })))
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::WaitingDcsTrusted);
        assert_eq!(output.outcome.decision, HaDecision::WaitForDcsTrust);
    }

    #[test]
    fn waiting_postgres_after_failed_start_without_leader_uses_healthy_primary_member_waits_for_dcs(
    ) {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::WaitingPostgresReachable,
                tick: 14,
                decision: HaDecision::WaitForPostgres {
                    start_requested: true,
                    leader_member_id: Some(MemberId("node-b".to_string())),
                },
            },
            world: WorldBuilder::new()
                .with_pg(pg_unknown(SqlStatus::Unreachable))
                .with_member(MemberRecord {
                    member_id: MemberId("node-b".to_string()),
                    postgres_host: "10.0.0.20".to_string(),
                    postgres_port: 5432,
                    role: MemberRole::Primary,
                    sql: SqlStatus::Healthy,
                    readiness: Readiness::Ready,
                    timeline: None,
                    write_lsn: None,
                    replay_lsn: None,
                    updated_at: UnixMillis(1),
                    pg_version: Version(1),
                })
                .with_process(process_idle(Some(JobOutcome::Failure {
                    id: JobId("job-start".to_string()),
                    job_kind: ActiveJobKind::StartPostgres,
                    error: crate::process::jobs::ProcessError::OperationFailed,
                    finished_at: UnixMillis(16),
                })))
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::WaitingDcsTrusted);
        assert_eq!(output.outcome.decision, HaDecision::WaitForDcsTrust);
    }

    #[test]
    fn waiting_postgres_after_failed_start_as_leader_waits_for_dcs() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::WaitingPostgresReachable,
                tick: 15,
                decision: HaDecision::WaitForPostgres {
                    start_requested: true,
                    leader_member_id: None,
                },
            },
            world: WorldBuilder::new()
                .with_pg(pg_unknown(SqlStatus::Unreachable))
                .with_leader("node-a")
                .with_process(process_idle(Some(JobOutcome::Failure {
                    id: JobId("job-start".to_string()),
                    job_kind: ActiveJobKind::StartPostgres,
                    error: crate::process::jobs::ProcessError::OperationFailed,
                    finished_at: UnixMillis(16),
                })))
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::WaitingDcsTrusted);
        assert_eq!(output.outcome.decision, HaDecision::WaitForDcsTrust);
    }

    #[test]
    fn waiting_postgres_after_successful_start_as_follower_waits_for_dcs() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::WaitingPostgresReachable,
                tick: 16,
                decision: HaDecision::WaitForPostgres {
                    start_requested: true,
                    leader_member_id: None,
                },
            },
            world: WorldBuilder::new()
                .with_pg(pg_unknown(SqlStatus::Unreachable))
                .with_process(process_idle(Some(JobOutcome::Success {
                    id: JobId("job-start".to_string()),
                    job_kind: ActiveJobKind::StartPostgres,
                    finished_at: UnixMillis(16),
                })))
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::WaitingDcsTrusted);
        assert_eq!(output.outcome.decision, HaDecision::WaitForDcsTrust);
    }

    #[test]
    fn waiting_postgres_after_successful_start_as_leader_waits_for_dcs() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::WaitingPostgresReachable,
                tick: 17,
                decision: HaDecision::WaitForPostgres {
                    start_requested: true,
                    leader_member_id: None,
                },
            },
            world: WorldBuilder::new()
                .with_pg(pg_unknown(SqlStatus::Unreachable))
                .with_leader("node-a")
                .with_process(process_idle(Some(JobOutcome::Success {
                    id: JobId("job-start".to_string()),
                    job_kind: ActiveJobKind::StartPostgres,
                    finished_at: UnixMillis(16),
                })))
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::WaitingDcsTrusted);
        assert_eq!(output.outcome.decision, HaDecision::WaitForDcsTrust);
    }

    #[test]
    fn replica_with_unhealthy_leader_becomes_candidate() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::Replica,
                tick: 11,
                decision: HaDecision::NoChange,
            },
            world: WorldBuilder::new()
                .with_leader("node-b")
                .with_member(MemberRecord {
                    member_id: MemberId("node-b".to_string()),
                    postgres_host: "10.0.0.10".to_string(),
                    postgres_port: 5432,
                    role: MemberRole::Unknown,
                    sql: SqlStatus::Unreachable,
                    readiness: Readiness::NotReady,
                    timeline: None,
                    write_lsn: None,
                    replay_lsn: None,
                    updated_at: UnixMillis(1),
                    pg_version: Version(1),
                })
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::CandidateLeader);
        assert_eq!(output.outcome.decision, HaDecision::AttemptLeadership);
    }

    #[test]
    fn candidate_leader_with_unhealthy_foreign_leader_keeps_attempting() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::CandidateLeader,
                tick: 12,
                decision: HaDecision::NoChange,
            },
            world: WorldBuilder::new()
                .with_leader("node-b")
                .with_member(MemberRecord {
                    member_id: MemberId("node-b".to_string()),
                    postgres_host: "10.0.0.10".to_string(),
                    postgres_port: 5432,
                    role: MemberRole::Unknown,
                    sql: SqlStatus::Unreachable,
                    readiness: Readiness::NotReady,
                    timeline: None,
                    write_lsn: None,
                    replay_lsn: None,
                    updated_at: UnixMillis(1),
                    pg_version: Version(1),
                })
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::CandidateLeader);
        assert_eq!(output.outcome.decision, HaDecision::AttemptLeadership);
    }

    #[test]
    fn candidate_leader_without_leader_follows_healthy_primary_member() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::CandidateLeader,
                tick: 12,
                decision: HaDecision::NoChange,
            },
            world: WorldBuilder::new()
                .with_pg(pg_replica(SqlStatus::Healthy))
                .with_member(MemberRecord {
                    member_id: MemberId("node-b".to_string()),
                    postgres_host: "10.0.0.20".to_string(),
                    postgres_port: 5432,
                    role: MemberRole::Primary,
                    sql: SqlStatus::Healthy,
                    readiness: Readiness::Ready,
                    timeline: None,
                    write_lsn: None,
                    replay_lsn: None,
                    updated_at: UnixMillis(1),
                    pg_version: Version(1),
                })
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::Replica);
        assert_eq!(
            output.outcome.decision,
            HaDecision::FollowLeader {
                leader_member_id: MemberId("node-b".to_string()),
            }
        );
    }

    #[test]
    fn candidate_leader_with_self_lease_and_primary_postgres_skips_promote() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::CandidateLeader,
                tick: 13,
                decision: HaDecision::NoChange,
            },
            world: WorldBuilder::new()
                .with_pg(pg_primary(SqlStatus::Healthy))
                .with_leader("node-a")
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::Primary);
        assert_eq!(
            output.outcome.decision,
            HaDecision::BecomePrimary { promote: false }
        );
    }

    #[test]
    fn decide_is_deterministic_for_identical_inputs() {
        let input = DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::Primary,
                tick: 9,
                decision: HaDecision::NoChange,
            },
            world: WorldBuilder::new()
                .with_pg(pg_primary(SqlStatus::Healthy))
                .with_leader("node-b")
                .build(),
        };

        let first = decide(input.clone());
        let second = decide(input.clone());
        let third = decide(input);

        assert_eq!(first, second);
        assert_eq!(second, third);
    }

    #[test]
    fn non_quorum_trust_always_routes_to_fail_safe() {
        struct FailSafeCase {
            name: &'static str,
            current_phase: HaPhase,
            trust: DcsTrust,
            pg: PgInfoState,
            expected_decision: HaDecision,
        }

        let cases = [
            FailSafeCase {
                name: "primary loses full quorum and fences without lease release",
                current_phase: HaPhase::Primary,
                trust: DcsTrust::NotTrusted,
                pg: pg_primary(SqlStatus::Healthy),
                expected_decision: HaDecision::EnterFailSafe {
                    release_leader_lease: false,
                },
            },
            FailSafeCase {
                name: "replica enters fail safe without extra actions",
                current_phase: HaPhase::Replica,
                trust: DcsTrust::NotTrusted,
                pg: pg_replica(SqlStatus::Healthy),
                expected_decision: HaDecision::NoChange,
            },
            FailSafeCase {
                name: "candidate leader in failsafe trust stays quiescent",
                current_phase: HaPhase::CandidateLeader,
                trust: DcsTrust::FailSafe,
                pg: pg_replica(SqlStatus::Healthy),
                expected_decision: HaDecision::NoChange,
            },
            FailSafeCase {
                name: "already failsafe replica stays quiescent",
                current_phase: HaPhase::FailSafe,
                trust: DcsTrust::FailSafe,
                pg: pg_replica(SqlStatus::Healthy),
                expected_decision: HaDecision::NoChange,
            },
        ];

        for case in cases {
            let output = decide(DecideInput {
                current: HaState {
                    worker: WorkerStatus::Running,
                    phase: case.current_phase.clone(),
                    tick: 3,
                    decision: HaDecision::NoChange,
                },
                world: WorldBuilder::new()
                    .with_trust(case.trust)
                    .with_pg(case.pg)
                    .build(),
            });

            assert_eq!(output.next.phase, HaPhase::FailSafe, "case: {}", case.name);
            assert_eq!(
                output.outcome.decision, case.expected_decision,
                "case: {}",
                case.name
            );
            assert_eq!(
                assert_plan_has_no_contradictions(&lower_decision(&output.outcome.decision)),
                Ok(()),
                "case: {}",
                case.name
            );
        }
    }

    #[test]
    fn failsafe_primary_with_full_quorum_returns_to_primary_decision_path() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::FailSafe,
                tick: 7,
                decision: HaDecision::NoChange,
            },
            world: WorldBuilder::new()
                .with_trust(DcsTrust::FullQuorum)
                .with_pg(pg_primary(SqlStatus::Healthy))
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::Primary);
        assert_eq!(output.outcome.decision, HaDecision::AttemptLeadership);
        assert_eq!(
            lower_decision(&output.outcome.decision).lease,
            LeaseEffect::AcquireLeader
        );
        assert_eq!(
            lower_decision(&output.outcome.decision).safety,
            SafetyEffect::None
        );
    }

    #[test]
    fn lowered_ha_plans_never_encode_contradictory_actions() {
        let decisions = [
            HaDecision::NoChange,
            HaDecision::WaitForPostgres {
                start_requested: false,
                leader_member_id: None,
            },
            HaDecision::WaitForPostgres {
                start_requested: true,
                leader_member_id: None,
            },
            HaDecision::WaitForDcsTrust,
            HaDecision::AttemptLeadership,
            HaDecision::FollowLeader {
                leader_member_id: MemberId("node-b".to_string()),
            },
            HaDecision::BecomePrimary { promote: false },
            HaDecision::BecomePrimary { promote: true },
            HaDecision::StepDown(StepDownPlan {
                reason: StepDownReason::Switchover,
                release_leader_lease: true,
                clear_switchover: true,
                fence: false,
            }),
            HaDecision::StepDown(StepDownPlan {
                reason: StepDownReason::ForeignLeaderDetected {
                    leader_member_id: MemberId("node-c".to_string()),
                },
                release_leader_lease: true,
                clear_switchover: false,
                fence: true,
            }),
            HaDecision::RecoverReplica {
                strategy: RecoveryStrategy::Rewind {
                    leader_member_id: MemberId("node-b".to_string()),
                },
            },
            HaDecision::RecoverReplica {
                strategy: RecoveryStrategy::BaseBackup {
                    leader_member_id: MemberId("node-b".to_string()),
                },
            },
            HaDecision::RecoverReplica {
                strategy: RecoveryStrategy::Bootstrap,
            },
            HaDecision::FenceNode,
            HaDecision::ReleaseLeaderLease {
                reason: LeaseReleaseReason::FencingComplete,
            },
            HaDecision::EnterFailSafe {
                release_leader_lease: false,
            },
            HaDecision::EnterFailSafe {
                release_leader_lease: true,
            },
        ];

        for decision in decisions {
            let plan = lower_decision(&decision);
            assert_eq!(
                assert_plan_has_no_contradictions(&plan),
                Ok(()),
                "decision: {}",
                decision.label()
            );
        }
    }

    impl WorldBuilder {
        fn build_with_optional_leader(self, leader: Option<&str>) -> WorldSnapshot {
            match leader {
                Some(leader_member_id) => self.with_leader(leader_member_id).build(),
                None => self.build(),
            }
        }
    }

    fn assert_plan_has_no_contradictions(plan: &HaEffectPlan) -> Result<(), String> {
        if matches!(plan.replication, ReplicationEffect::FollowLeader { .. })
            && matches!(plan.postgres, PostgresEffect::Promote)
        {
            return Err("plan cannot follow a leader and promote locally".to_string());
        }

        if matches!(plan.safety, SafetyEffect::SignalFailSafe)
            && (!matches!(plan.replication, ReplicationEffect::None)
                || !matches!(plan.postgres, PostgresEffect::None)
                || !matches!(plan.switchover, SwitchoverEffect::None))
        {
            return Err(
                "fail-safe plan cannot carry replication, postgres, or switchover side effects"
                    .to_string(),
            );
        }

        if matches!(plan.lease, LeaseEffect::AcquireLeader)
            && matches!(plan.postgres, PostgresEffect::Demote)
        {
            return Err("plan cannot acquire the leader lease while demoting postgres".to_string());
        }

        if matches!(plan.safety, SafetyEffect::FenceNode)
            && matches!(plan.postgres, PostgresEffect::Promote)
        {
            return Err("fence plan cannot promote postgres".to_string());
        }

        Ok(())
    }

    fn process_clone(process: &ProcessState) -> ProcessState {
        match process {
            ProcessState::Running { worker, active } => ProcessState::Running {
                worker: worker.clone(),
                active: active.clone(),
            },
            ProcessState::Idle {
                worker,
                last_outcome,
            } => ProcessState::Idle {
                worker: worker.clone(),
                last_outcome: last_outcome.clone(),
            },
        }
    }

    fn process_idle(last_outcome: Option<JobOutcome>) -> ProcessState {
        ProcessState::Idle {
            worker: WorkerStatus::Running,
            last_outcome,
        }
    }

    fn process_running(kind: ActiveJobKind) -> ProcessState {
        ProcessState::Running {
            worker: WorkerStatus::Running,
            active: ActiveJob {
                id: JobId("active-1".to_string()),
                kind,
                started_at: UnixMillis(1),
                deadline_at: UnixMillis(2),
            },
        }
    }

    fn pg_unknown(sql: SqlStatus) -> PgInfoState {
        PgInfoState::Unknown {
            common: pg_common(sql),
        }
    }

    fn pg_primary(sql: SqlStatus) -> PgInfoState {
        PgInfoState::Primary {
            common: pg_common(sql),
            wal_lsn: crate::state::WalLsn(10),
            slots: vec![],
        }
    }

    fn pg_replica(sql: SqlStatus) -> PgInfoState {
        PgInfoState::Replica {
            common: pg_common(sql),
            replay_lsn: crate::state::WalLsn(10),
            follow_lsn: None,
            upstream: None,
        }
    }

    fn pg_common(sql: SqlStatus) -> PgInfoCommon {
        PgInfoCommon {
            worker: WorkerStatus::Running,
            sql,
            readiness: Readiness::Ready,
            timeline: None,
            pg_config: PgConfig {
                port: None,
                hot_standby: None,
                primary_conninfo: None,
                primary_slot_name: None,
                extra: BTreeMap::new(),
            },
            last_refresh_at: Some(UnixMillis(1)),
        }
    }

    fn world(
        trust: DcsTrust,
        pg: PgInfoState,
        leader: Option<MemberId>,
        process: ProcessState,
        members: BTreeMap<MemberId, MemberRecord>,
        switchover_requested_by: Option<MemberId>,
    ) -> WorldSnapshot {
        let cfg = crate::test_harness::runtime_config::sample_runtime_config();

        let leader_record = leader.map(|member_id| LeaderRecord { member_id });

        WorldSnapshot {
            config: Versioned::new(Version(1), UnixMillis(1), cfg.clone()),
            pg: Versioned::new(Version(1), UnixMillis(1), pg),
            dcs: Versioned::new(
                Version(1),
                UnixMillis(1),
                DcsState {
                    worker: WorkerStatus::Running,
                    trust,
                    cache: DcsCache {
                        members,
                        leader: leader_record,
                        switchover: switchover_requested_by
                            .map(|requested_by| SwitchoverRequest { requested_by }),
                        config: cfg,
                        init_lock: None,
                    },
                    last_refresh_at: Some(UnixMillis(1)),
                },
            ),
            process: Versioned::new(Version(1), UnixMillis(1), process),
        }
    }
}

--- END FILE: src/ha/decide.rs ---

--- BEGIN FILE: src/ha/lower.rs ---
use serde::{Deserialize, Serialize};

use crate::state::MemberId;

use super::decision::{HaDecision, RecoveryStrategy, StepDownPlan};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct HaEffectPlan {
    pub(crate) lease: LeaseEffect,
    pub(crate) switchover: SwitchoverEffect,
    pub(crate) replication: ReplicationEffect,
    pub(crate) postgres: PostgresEffect,
    pub(crate) safety: SafetyEffect,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum LeaseEffect {
    #[default]
    None,
    AcquireLeader,
    ReleaseLeader,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum SwitchoverEffect {
    #[default]
    None,
    ClearRequest,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum ReplicationEffect {
    #[default]
    None,
    FollowLeader {
        leader_member_id: MemberId,
    },
    RecoverReplica {
        strategy: RecoveryStrategy,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum PostgresEffect {
    #[default]
    None,
    Start,
    Promote,
    Demote,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum SafetyEffect {
    #[default]
    None,
    FenceNode,
    SignalFailSafe,
}

impl HaDecision {
    pub(crate) fn lower(&self) -> HaEffectPlan {
        match self {
            Self::NoChange | Self::WaitForDcsTrust => HaEffectPlan::default(),
            Self::WaitForPostgres {
                start_requested, ..
            } => HaEffectPlan {
                postgres: if *start_requested {
                    PostgresEffect::Start
                } else {
                    PostgresEffect::None
                },
                ..HaEffectPlan::default()
            },
            Self::AttemptLeadership => HaEffectPlan {
                lease: LeaseEffect::AcquireLeader,
                ..HaEffectPlan::default()
            },
            Self::FollowLeader { leader_member_id } => HaEffectPlan {
                replication: ReplicationEffect::FollowLeader {
                    leader_member_id: leader_member_id.clone(),
                },
                ..HaEffectPlan::default()
            },
            Self::BecomePrimary { promote } => HaEffectPlan {
                postgres: if *promote {
                    PostgresEffect::Promote
                } else {
                    PostgresEffect::None
                },
                ..HaEffectPlan::default()
            },
            Self::StepDown(plan) => lower_step_down(plan),
            Self::RecoverReplica { strategy } => HaEffectPlan {
                replication: ReplicationEffect::RecoverReplica {
                    strategy: strategy.clone(),
                },
                ..HaEffectPlan::default()
            },
            Self::FenceNode => HaEffectPlan {
                safety: SafetyEffect::FenceNode,
                ..HaEffectPlan::default()
            },
            Self::ReleaseLeaderLease { .. } => HaEffectPlan {
                lease: LeaseEffect::ReleaseLeader,
                ..HaEffectPlan::default()
            },
            Self::EnterFailSafe {
                release_leader_lease,
            } => HaEffectPlan {
                lease: if *release_leader_lease {
                    LeaseEffect::ReleaseLeader
                } else {
                    LeaseEffect::None
                },
                safety: SafetyEffect::FenceNode,
                ..HaEffectPlan::default()
            },
        }
    }
}

pub(crate) fn lower_decision(decision: &HaDecision) -> HaEffectPlan {
    decision.lower()
}

impl HaEffectPlan {
    pub(crate) fn len(&self) -> usize {
        self.dispatch_step_count()
    }

    pub(crate) fn dispatch_step_count(&self) -> usize {
        lease_effect_step_count(&self.lease)
            + switchover_effect_step_count(&self.switchover)
            + replication_effect_step_count(&self.replication)
            + postgres_effect_step_count(&self.postgres)
            + safety_effect_step_count(&self.safety)
    }
}

fn lower_step_down(plan: &StepDownPlan) -> HaEffectPlan {
    HaEffectPlan {
        lease: if plan.release_leader_lease {
            LeaseEffect::ReleaseLeader
        } else {
            LeaseEffect::None
        },
        switchover: if plan.clear_switchover {
            SwitchoverEffect::ClearRequest
        } else {
            SwitchoverEffect::None
        },
        replication: ReplicationEffect::None,
        postgres: PostgresEffect::Demote,
        safety: if plan.fence {
            SafetyEffect::FenceNode
        } else {
            SafetyEffect::None
        },
    }
}

pub(crate) fn lease_effect_step_count(effect: &LeaseEffect) -> usize {
    match effect {
        LeaseEffect::None => 0,
        LeaseEffect::AcquireLeader | LeaseEffect::ReleaseLeader => 1,
    }
}

pub(crate) fn switchover_effect_step_count(effect: &SwitchoverEffect) -> usize {
    match effect {
        SwitchoverEffect::None => 0,
        SwitchoverEffect::ClearRequest => 1,
    }
}

pub(crate) fn replication_effect_step_count(effect: &ReplicationEffect) -> usize {
    match effect {
        ReplicationEffect::None => 0,
        ReplicationEffect::FollowLeader { .. } => 1,
        ReplicationEffect::RecoverReplica { strategy } => match strategy {
            RecoveryStrategy::Rewind { .. } => 1,
            RecoveryStrategy::BaseBackup { .. } | RecoveryStrategy::Bootstrap => 2,
        },
    }
}

pub(crate) fn postgres_effect_step_count(effect: &PostgresEffect) -> usize {
    match effect {
        PostgresEffect::None => 0,
        PostgresEffect::Start | PostgresEffect::Promote | PostgresEffect::Demote => 1,
    }
}

pub(crate) fn safety_effect_step_count(effect: &SafetyEffect) -> usize {
    match effect {
        SafetyEffect::None => 0,
        SafetyEffect::FenceNode | SafetyEffect::SignalFailSafe => 1,
    }
}

#[cfg(test)]
mod tests {
    use crate::state::MemberId;

    use super::{
        super::decision::{
            HaDecision, LeaseReleaseReason, RecoveryStrategy, StepDownPlan, StepDownReason,
        },
        HaEffectPlan, LeaseEffect, PostgresEffect, ReplicationEffect, SafetyEffect,
        SwitchoverEffect,
    };

    #[test]
    fn lowers_composite_step_down_into_bucketed_plan() {
        let decision = HaDecision::StepDown(StepDownPlan {
            reason: StepDownReason::Switchover,
            release_leader_lease: true,
            clear_switchover: true,
            fence: false,
        });

        assert_eq!(
            decision.lower(),
            HaEffectPlan {
                lease: LeaseEffect::ReleaseLeader,
                switchover: SwitchoverEffect::ClearRequest,
                replication: ReplicationEffect::None,
                postgres: PostgresEffect::Demote,
                safety: SafetyEffect::None,
            }
        );
    }

    #[test]
    fn lowers_fail_safe_primary_release_into_fencing_plan() {
        let decision = HaDecision::EnterFailSafe {
            release_leader_lease: true,
        };

        assert_eq!(
            decision.lower(),
            HaEffectPlan {
                lease: LeaseEffect::ReleaseLeader,
                switchover: SwitchoverEffect::None,
                replication: ReplicationEffect::None,
                postgres: PostgresEffect::None,
                safety: SafetyEffect::FenceNode,
            }
        );
    }

    #[test]
    fn lowers_fail_safe_without_release_into_fencing_plan() {
        let decision = HaDecision::EnterFailSafe {
            release_leader_lease: false,
        };

        assert_eq!(
            decision.lower(),
            HaEffectPlan {
                lease: LeaseEffect::None,
                switchover: SwitchoverEffect::None,
                replication: ReplicationEffect::None,
                postgres: PostgresEffect::None,
                safety: SafetyEffect::FenceNode,
            }
        );
    }

    #[test]
    fn lowers_recovery_variants() {
        let rewind = HaDecision::RecoverReplica {
            strategy: RecoveryStrategy::Rewind {
                leader_member_id: MemberId("node-b".to_string()),
            },
        };
        let basebackup = HaDecision::RecoverReplica {
            strategy: RecoveryStrategy::BaseBackup {
                leader_member_id: MemberId("node-b".to_string()),
            },
        };
        let bootstrap = HaDecision::RecoverReplica {
            strategy: RecoveryStrategy::Bootstrap,
        };

        assert_eq!(
            rewind.lower(),
            HaEffectPlan {
                lease: LeaseEffect::None,
                switchover: SwitchoverEffect::None,
                replication: ReplicationEffect::RecoverReplica {
                    strategy: RecoveryStrategy::Rewind {
                        leader_member_id: MemberId("node-b".to_string()),
                    },
                },
                postgres: PostgresEffect::None,
                safety: SafetyEffect::None,
            }
        );
        assert_eq!(basebackup.lower().dispatch_step_count(), 2);
        assert_eq!(bootstrap.lower().dispatch_step_count(), 2);
    }

    #[test]
    fn lowers_extra_release_variant() {
        let decision = HaDecision::ReleaseLeaderLease {
            reason: LeaseReleaseReason::FencingComplete,
        };

        assert_eq!(
            decision.lower(),
            HaEffectPlan {
                lease: LeaseEffect::ReleaseLeader,
                switchover: SwitchoverEffect::None,
                replication: ReplicationEffect::None,
                postgres: PostgresEffect::None,
                safety: SafetyEffect::None,
            }
        );
    }
}

--- END FILE: src/ha/lower.rs ---

--- BEGIN FILE: src/ha/actions.rs ---
use crate::state::MemberId;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ActionId {
    AcquireLeaderLease,
    ReleaseLeaderLease,
    ClearSwitchover,
    FollowLeader(String),
    StartRewind,
    StartBaseBackup,
    RunBootstrap,
    FenceNode,
    WipeDataDir,
    SignalFailSafe,
    StartPostgres,
    PromoteToPrimary,
    DemoteToReplica,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum HaAction {
    AcquireLeaderLease,
    ReleaseLeaderLease,
    ClearSwitchover,
    FollowLeader { leader_member_id: String },
    StartRewind { leader_member_id: MemberId },
    StartBaseBackup { leader_member_id: MemberId },
    RunBootstrap,
    FenceNode,
    WipeDataDir,
    SignalFailSafe,
    StartPostgres,
    PromoteToPrimary,
    DemoteToReplica,
}

impl HaAction {
    pub(crate) fn id(&self) -> ActionId {
        match self {
            Self::AcquireLeaderLease => ActionId::AcquireLeaderLease,
            Self::ReleaseLeaderLease => ActionId::ReleaseLeaderLease,
            Self::ClearSwitchover => ActionId::ClearSwitchover,
            Self::FollowLeader { leader_member_id } => {
                ActionId::FollowLeader(leader_member_id.clone())
            }
            Self::StartRewind { .. } => ActionId::StartRewind,
            Self::StartBaseBackup { .. } => ActionId::StartBaseBackup,
            Self::RunBootstrap => ActionId::RunBootstrap,
            Self::FenceNode => ActionId::FenceNode,
            Self::WipeDataDir => ActionId::WipeDataDir,
            Self::SignalFailSafe => ActionId::SignalFailSafe,
            Self::StartPostgres => ActionId::StartPostgres,
            Self::PromoteToPrimary => ActionId::PromoteToPrimary,
            Self::DemoteToReplica => ActionId::DemoteToReplica,
        }
    }
}

impl ActionId {
    pub(crate) fn label(&self) -> String {
        match self {
            Self::AcquireLeaderLease => "acquire_leader_lease".to_string(),
            Self::ReleaseLeaderLease => "release_leader_lease".to_string(),
            Self::ClearSwitchover => "clear_switchover".to_string(),
            Self::FollowLeader(leader) => format!("follow_leader_{leader}"),
            Self::StartRewind => "start_rewind".to_string(),
            Self::StartBaseBackup => "start_basebackup".to_string(),
            Self::RunBootstrap => "run_bootstrap".to_string(),
            Self::FenceNode => "fence_node".to_string(),
            Self::WipeDataDir => "wipe_data_dir".to_string(),
            Self::SignalFailSafe => "signal_failsafe".to_string(),
            Self::StartPostgres => "start_postgres".to_string(),
            Self::PromoteToPrimary => "promote_to_primary".to_string(),
            Self::DemoteToReplica => "demote_to_replica".to_string(),
        }
    }
}

--- END FILE: src/ha/actions.rs ---

--- BEGIN FILE: src/ha/apply.rs ---
use thiserror::Error;

use crate::{
    dcs::store::{DcsHaWriter, DcsStoreError},
    state::WorkerError,
};

use super::{
    actions::{ActionId, HaAction},
    events::{
        emit_ha_action_dispatch, emit_ha_action_intent, emit_ha_action_result_failed,
        emit_ha_action_result_ok, emit_ha_action_result_skipped, emit_ha_lease_transition,
    },
    lower::{
        HaEffectPlan, LeaseEffect, PostgresEffect, ReplicationEffect, SafetyEffect,
        SwitchoverEffect,
    },
    process_dispatch::{
        dispatch_process_action, validate_basebackup_source, ProcessDispatchError,
        ProcessDispatchOutcome,
    },
    state::HaWorkerCtx,
};

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(crate) enum ActionDispatchError {
    #[error("process send failed for action `{action:?}`: {message}")]
    ProcessSend { action: ActionId, message: String },
    #[error("managed config materialization failed for action `{action:?}`: {message}")]
    ManagedConfig { action: ActionId, message: String },
    #[error("filesystem operation failed for action `{action:?}`: {message}")]
    Filesystem { action: ActionId, message: String },
    #[error("dcs write failed for action `{action:?}` at `{path}`: {message}")]
    DcsWrite {
        action: ActionId,
        path: String,
        message: String,
    },
    #[error("dcs delete failed for action `{action:?}` at `{path}`: {message}")]
    DcsDelete {
        action: ActionId,
        path: String,
        message: String,
    },
}

pub(crate) fn apply_effect_plan(
    ctx: &mut HaWorkerCtx,
    ha_tick: u64,
    plan: &HaEffectPlan,
) -> Result<Vec<ActionDispatchError>, WorkerError> {
    let runtime_config = ctx.config_subscriber.latest().value;
    let mut errors = Vec::new();
    let mut action_index = 0usize;

    action_index = dispatch_postgres_effect(
        ctx,
        ha_tick,
        action_index,
        &plan.postgres,
        &runtime_config,
        &mut errors,
    )?;
    action_index = dispatch_lease_effect(
        ctx,
        ha_tick,
        action_index,
        &plan.lease,
        &runtime_config,
        &mut errors,
    )?;
    action_index = dispatch_switchover_effect(
        ctx,
        ha_tick,
        action_index,
        &plan.switchover,
        &runtime_config,
        &mut errors,
    )?;
    action_index = dispatch_replication_effect(
        ctx,
        ha_tick,
        action_index,
        &plan.replication,
        &runtime_config,
        &mut errors,
    )?;
    let _ = dispatch_safety_effect(
        ctx,
        ha_tick,
        action_index,
        &plan.safety,
        &runtime_config,
        &mut errors,
    )?;

    Ok(errors)
}

pub(crate) fn format_dispatch_errors(errors: &[ActionDispatchError]) -> String {
    let mut details = String::new();
    for (index, err) in errors.iter().enumerate() {
        if index > 0 {
            details.push_str("; ");
        }
        details.push_str(&err.to_string());
    }
    format!(
        "ha dispatch failed with {} error(s): {details}",
        errors.len()
    )
}

fn dispatch_postgres_effect(
    ctx: &mut HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    effect: &PostgresEffect,
    runtime_config: &crate::config::RuntimeConfig,
    errors: &mut Vec<ActionDispatchError>,
) -> Result<usize, WorkerError> {
    match effect {
        PostgresEffect::None => Ok(action_index),
        PostgresEffect::Start => dispatch_effect_action(
            ctx,
            ha_tick,
            action_index,
            HaAction::StartPostgres,
            runtime_config,
            errors,
        ),
        PostgresEffect::Promote => dispatch_effect_action(
            ctx,
            ha_tick,
            action_index,
            HaAction::PromoteToPrimary,
            runtime_config,
            errors,
        ),
        PostgresEffect::Demote => dispatch_effect_action(
            ctx,
            ha_tick,
            action_index,
            HaAction::DemoteToReplica,
            runtime_config,
            errors,
        ),
    }
}

fn dispatch_lease_effect(
    ctx: &mut HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    effect: &LeaseEffect,
    runtime_config: &crate::config::RuntimeConfig,
    errors: &mut Vec<ActionDispatchError>,
) -> Result<usize, WorkerError> {
    match effect {
        LeaseEffect::None => Ok(action_index),
        LeaseEffect::AcquireLeader => dispatch_effect_action(
            ctx,
            ha_tick,
            action_index,
            HaAction::AcquireLeaderLease,
            runtime_config,
            errors,
        ),
        LeaseEffect::ReleaseLeader => dispatch_effect_action(
            ctx,
            ha_tick,
            action_index,
            HaAction::ReleaseLeaderLease,
            runtime_config,
            errors,
        ),
    }
}

fn dispatch_switchover_effect(
    ctx: &mut HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    effect: &SwitchoverEffect,
    runtime_config: &crate::config::RuntimeConfig,
    errors: &mut Vec<ActionDispatchError>,
) -> Result<usize, WorkerError> {
    match effect {
        SwitchoverEffect::None => Ok(action_index),
        SwitchoverEffect::ClearRequest => dispatch_effect_action(
            ctx,
            ha_tick,
            action_index,
            HaAction::ClearSwitchover,
            runtime_config,
            errors,
        ),
    }
}

fn dispatch_replication_effect(
    ctx: &mut HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    effect: &ReplicationEffect,
    runtime_config: &crate::config::RuntimeConfig,
    errors: &mut Vec<ActionDispatchError>,
) -> Result<usize, WorkerError> {
    match effect {
        ReplicationEffect::None => Ok(action_index),
        ReplicationEffect::FollowLeader { leader_member_id } => dispatch_effect_action(
            ctx,
            ha_tick,
            action_index,
            HaAction::FollowLeader {
                leader_member_id: leader_member_id.0.clone(),
            },
            runtime_config,
            errors,
        ),
        ReplicationEffect::RecoverReplica { strategy } => match strategy {
            crate::ha::decision::RecoveryStrategy::Rewind { leader_member_id } => {
                dispatch_effect_action(
                    ctx,
                    ha_tick,
                    action_index,
                    HaAction::StartRewind {
                        leader_member_id: leader_member_id.clone(),
                    },
                    runtime_config,
                    errors,
                )
            }
            crate::ha::decision::RecoveryStrategy::BaseBackup { leader_member_id } => {
                if let Err(err) =
                    validate_basebackup_source(ctx, ActionId::StartBaseBackup, leader_member_id)
                {
                    errors.push(map_process_dispatch_error(err));
                    return Ok(action_index);
                }
                let next_index = dispatch_effect_action(
                    ctx,
                    ha_tick,
                    action_index,
                    HaAction::WipeDataDir,
                    runtime_config,
                    errors,
                )?;
                dispatch_effect_action(
                    ctx,
                    ha_tick,
                    next_index,
                    HaAction::StartBaseBackup {
                        leader_member_id: leader_member_id.clone(),
                    },
                    runtime_config,
                    errors,
                )
            }
            crate::ha::decision::RecoveryStrategy::Bootstrap => {
                let next_index = dispatch_effect_action(
                    ctx,
                    ha_tick,
                    action_index,
                    HaAction::WipeDataDir,
                    runtime_config,
                    errors,
                )?;
                dispatch_effect_action(
                    ctx,
                    ha_tick,
                    next_index,
                    HaAction::RunBootstrap,
                    runtime_config,
                    errors,
                )
            }
        },
    }
}

fn dispatch_safety_effect(
    ctx: &mut HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    effect: &SafetyEffect,
    runtime_config: &crate::config::RuntimeConfig,
    errors: &mut Vec<ActionDispatchError>,
) -> Result<usize, WorkerError> {
    match effect {
        SafetyEffect::None => Ok(action_index),
        SafetyEffect::FenceNode => dispatch_effect_action(
            ctx,
            ha_tick,
            action_index,
            HaAction::FenceNode,
            runtime_config,
            errors,
        ),
        SafetyEffect::SignalFailSafe => dispatch_effect_action(
            ctx,
            ha_tick,
            action_index,
            HaAction::SignalFailSafe,
            runtime_config,
            errors,
        ),
    }
}

fn dispatch_effect_action(
    ctx: &mut HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    action: HaAction,
    runtime_config: &crate::config::RuntimeConfig,
    errors: &mut Vec<ActionDispatchError>,
) -> Result<usize, WorkerError> {
    emit_ha_action_intent(ctx, ha_tick, action_index, &action)?;
    emit_ha_action_dispatch(ctx, ha_tick, action_index, &action)?;

    if let Some(error) = dispatch_action(ctx, ha_tick, action_index, &action, runtime_config)? {
        errors.push(error);
    }

    Ok(action_index.saturating_add(1))
}

fn dispatch_action(
    ctx: &mut HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    action: &HaAction,
    runtime_config: &crate::config::RuntimeConfig,
) -> Result<Option<ActionDispatchError>, WorkerError> {
    match action {
        HaAction::AcquireLeaderLease => {
            let dispatch_result = acquire_leader_lease(ctx);
            dcs_dispatch_result(
                ctx,
                ha_tick,
                action_index,
                action,
                leader_path(&ctx.scope),
                dispatch_result,
                true,
            )
        }
        HaAction::ReleaseLeaderLease => {
            let dispatch_result = release_leader_lease(ctx);
            dcs_dispatch_result(
                ctx,
                ha_tick,
                action_index,
                action,
                leader_path(&ctx.scope),
                dispatch_result,
                false,
            )
        }
        HaAction::ClearSwitchover => {
            let path = switchover_path(&ctx.scope);
            let result = clear_switchover_request(ctx);
            dcs_delete_result(ctx, ha_tick, action_index, action, path, result)
        }
        _ => {
            let result =
                dispatch_process_action(ctx, ha_tick, action_index, action, runtime_config);
            process_dispatch_result(ctx, ha_tick, action_index, action, result)
        }
    }
}

fn dcs_dispatch_result(
    ctx: &HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    action: &HaAction,
    path: String,
    result: Result<(), DcsStoreError>,
    acquired: bool,
) -> Result<Option<ActionDispatchError>, WorkerError> {
    match result {
        Ok(()) => {
            emit_ha_action_result_ok(ctx, ha_tick, action_index, action)?;
            emit_ha_lease_transition(ctx, ha_tick, acquired)?;
            Ok(None)
        }
        Err(err) => {
            let message = err.to_string();
            emit_ha_action_result_failed(ctx, ha_tick, action_index, action, message)?;
            let error = if acquired {
                ActionDispatchError::DcsWrite {
                    action: action.id(),
                    path,
                    message: dcs_error_message(err),
                }
            } else {
                ActionDispatchError::DcsDelete {
                    action: action.id(),
                    path,
                    message: dcs_error_message(err),
                }
            };
            Ok(Some(error))
        }
    }
}

fn dcs_delete_result(
    ctx: &HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    action: &HaAction,
    path: String,
    result: Result<(), DcsStoreError>,
) -> Result<Option<ActionDispatchError>, WorkerError> {
    match result {
        Ok(()) => {
            emit_ha_action_result_ok(ctx, ha_tick, action_index, action)?;
            Ok(None)
        }
        Err(err) => {
            let message = err.to_string();
            emit_ha_action_result_failed(ctx, ha_tick, action_index, action, message)?;
            Ok(Some(ActionDispatchError::DcsDelete {
                action: action.id(),
                path,
                message: dcs_error_message(err),
            }))
        }
    }
}

fn process_dispatch_result(
    ctx: &HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    action: &HaAction,
    result: Result<ProcessDispatchOutcome, ProcessDispatchError>,
) -> Result<Option<ActionDispatchError>, WorkerError> {
    match result {
        Ok(ProcessDispatchOutcome::Applied) => {
            emit_ha_action_result_ok(ctx, ha_tick, action_index, action)?;
            Ok(None)
        }
        Ok(ProcessDispatchOutcome::Skipped) => {
            emit_ha_action_result_skipped(ctx, ha_tick, action_index, action)?;
            Ok(None)
        }
        Err(ProcessDispatchError::UnsupportedAction { action }) => {
            Err(WorkerError::Message(format!(
                "ha apply routed unsupported process action `{}`",
                action.label()
            )))
        }
        Err(err) => {
            let message = err.to_string();
            emit_ha_action_result_failed(ctx, ha_tick, action_index, action, message)?;
            Ok(Some(map_process_dispatch_error(err)))
        }
    }
}

fn acquire_leader_lease(ctx: &mut HaWorkerCtx) -> Result<(), DcsStoreError> {
    ctx.dcs_store.write_leader_lease(&ctx.scope, &ctx.self_id)
}

fn release_leader_lease(ctx: &mut HaWorkerCtx) -> Result<(), DcsStoreError> {
    ctx.dcs_store.delete_leader(&ctx.scope)
}

fn clear_switchover_request(ctx: &mut HaWorkerCtx) -> Result<(), DcsStoreError> {
    ctx.dcs_store.clear_switchover(&ctx.scope)
}

fn leader_path(scope: &str) -> String {
    format!("/{}/leader", scope.trim_matches('/'))
}

fn switchover_path(scope: &str) -> String {
    format!("/{}/switchover", scope.trim_matches('/'))
}

fn dcs_error_message(error: DcsStoreError) -> String {
    error.to_string()
}

fn map_process_dispatch_error(error: ProcessDispatchError) -> ActionDispatchError {
    match error {
        ProcessDispatchError::ProcessSend { action, message } => {
            ActionDispatchError::ProcessSend { action, message }
        }
        ProcessDispatchError::ManagedConfig { action, message } => {
            ActionDispatchError::ManagedConfig { action, message }
        }
        ProcessDispatchError::Filesystem { action, message } => {
            ActionDispatchError::Filesystem { action, message }
        }
        ProcessDispatchError::SourceSelection { action, message } => {
            ActionDispatchError::ProcessSend { action, message }
        }
        ProcessDispatchError::UnsupportedAction { action } => ActionDispatchError::ProcessSend {
            action,
            message: "unsupported process action".to_string(),
        },
    }
}

--- END FILE: src/ha/apply.rs ---

--- BEGIN FILE: src/ha/worker.rs ---
use crate::state::{WorkerError, WorkerStatus};

use super::{
    apply::{apply_effect_plan, format_dispatch_errors},
    decide::decide,
    events::{
        emit_ha_decision_selected, emit_ha_effect_plan_selected, emit_ha_phase_transition,
        emit_ha_role_transition, ha_role_label,
    },
    state::{DecideInput, HaWorkerCtx, WorldSnapshot},
};

pub(crate) async fn run(mut ctx: HaWorkerCtx) -> Result<(), WorkerError> {
    let mut interval = tokio::time::interval(ctx.poll_interval);
    loop {
        tokio::select! {
            changed = ctx.pg_subscriber.changed() => {
                changed.map_err(|err| {
                    WorkerError::Message(format!("ha pg subscriber closed: {err}"))
                })?;
            }
            changed = ctx.dcs_subscriber.changed() => {
                changed.map_err(|err| {
                    WorkerError::Message(format!("ha dcs subscriber closed: {err}"))
                })?;
            }
            changed = ctx.process_subscriber.changed() => {
                changed.map_err(|err| {
                    WorkerError::Message(format!("ha process subscriber closed: {err}"))
                })?;
            }
            changed = ctx.config_subscriber.changed() => {
                changed.map_err(|err| {
                    WorkerError::Message(format!("ha config subscriber closed: {err}"))
                })?;
            }
            _ = interval.tick() => {}
        }
        step_once(&mut ctx).await?;
    }
}

pub(crate) async fn step_once(ctx: &mut HaWorkerCtx) -> Result<(), WorkerError> {
    let prev_phase = ctx.state.phase.clone();
    let world = world_snapshot(ctx);
    let output = decide(DecideInput {
        current: ctx.state.clone(),
        world,
    });
    let plan = output.outcome.decision.lower();
    let skip_redundant_process_dispatch =
        should_skip_redundant_process_dispatch(&ctx.state, &output.next);

    emit_ha_decision_selected(ctx, output.next.tick, &output.outcome.decision, &plan)?;
    emit_ha_effect_plan_selected(ctx, output.next.tick, &plan)?;
    let published_next = crate::ha::state::HaState {
        worker: WorkerStatus::Running,
        ..output.next.clone()
    };
    let now = (ctx.now)()?;

    ctx.publisher
        .publish(published_next.clone(), now)
        .map_err(|err| WorkerError::Message(format!("ha publish failed: {err}")))?;

    if prev_phase != published_next.phase {
        emit_ha_phase_transition(ctx, published_next.tick, &prev_phase, &published_next.phase)?;
    }

    let prev_role = ha_role_label(&prev_phase);
    let next_role = ha_role_label(&published_next.phase);
    if prev_role != next_role {
        emit_ha_role_transition(ctx, published_next.tick, prev_role, next_role)?;
    }

    ctx.state = published_next.clone();

    let dispatch_errors = if skip_redundant_process_dispatch {
        Vec::new()
    } else {
        apply_effect_plan(ctx, published_next.tick, &plan)?
    };
    if !dispatch_errors.is_empty() {
        let faulted = crate::ha::state::HaState {
            worker: WorkerStatus::Faulted(WorkerError::Message(format_dispatch_errors(
                &dispatch_errors,
            ))),
            ..published_next
        };
        let faulted_now = (ctx.now)()?;
        ctx.publisher
            .publish(faulted.clone(), faulted_now)
            .map_err(|err| WorkerError::Message(format!("ha publish failed: {err}")))?;
        ctx.state = faulted;
    }

    Ok(())
}

fn world_snapshot(ctx: &HaWorkerCtx) -> WorldSnapshot {
    WorldSnapshot {
        config: ctx.config_subscriber.latest(),
        pg: ctx.pg_subscriber.latest(),
        dcs: ctx.dcs_subscriber.latest(),
        process: ctx.process_subscriber.latest(),
    }
}

fn should_skip_redundant_process_dispatch(
    current: &crate::ha::state::HaState,
    next: &crate::ha::state::HaState,
) -> bool {
    current.phase == next.phase
        && current.decision == next.decision
        && matches!(
            next.decision,
            crate::ha::decision::HaDecision::WaitForPostgres {
                start_requested: true,
                ..
            } | crate::ha::decision::HaDecision::RecoverReplica { .. }
                | crate::ha::decision::HaDecision::FenceNode
        )
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{BTreeMap, VecDeque},
        path::PathBuf,
        sync::{
            atomic::{AtomicU64, Ordering},
            mpsc::{self, RecvTimeoutError},
            Arc, Mutex,
        },
        time::{Duration, SystemTime, UNIX_EPOCH},
    };
    use tokio::sync::mpsc::error::TryRecvError;

    use crate::{
        config::RuntimeConfig,
        dcs::{
            state::{DcsCache, DcsState, DcsTrust, LeaderRecord, MemberRecord, MemberRole},
            store::{DcsStore, DcsStoreError, WatchEvent, WatchOp},
        },
        ha::{
            actions::ActionId,
            apply::{apply_effect_plan, ActionDispatchError},
            decision::HaDecision,
            lower::{
                lower_decision, HaEffectPlan, LeaseEffect, PostgresEffect, ReplicationEffect,
                SafetyEffect, SwitchoverEffect,
            },
            state::{
                DecideInput, HaPhase, HaState, HaWorkerContractStubInputs, HaWorkerCtx,
                ProcessDispatchDefaults,
            },
            worker::{run, step_once},
        },
        pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
        process::{
            jobs::{
                ProcessCommandRunner, ProcessCommandSpec, ProcessError, ProcessExit, ProcessHandle,
            },
            state::{
                JobOutcome, ProcessJobKind, ProcessJobRequest, ProcessState, ProcessWorkerCtx,
            },
        },
        state::{new_state_channel, MemberId, UnixMillis, Version, WorkerError, WorkerStatus},
    };

    const TEST_DCS_AND_PROCESS_POLL_INTERVAL: Duration = Duration::from_millis(5);
    const TEST_HA_VERSION_WAIT_TIMEOUT: Duration = Duration::from_millis(250);

    #[derive(Clone, Default)]
    struct RecordingStore {
        fail_write: bool,
        fail_delete: bool,
        reject_put_if_absent: bool,
        writes: Arc<Mutex<Vec<(String, String)>>>,
        deletes: Arc<Mutex<Vec<String>>>,
        events: Arc<Mutex<VecDeque<WatchEvent>>>,
        delete_block_started: Option<Arc<Mutex<Option<mpsc::Sender<()>>>>>,
        delete_block_release: Option<Arc<Mutex<mpsc::Receiver<()>>>>,
    }

    impl RecordingStore {
        fn writes_len(&self) -> usize {
            if let Ok(guard) = self.writes.lock() {
                return guard.len();
            }
            0
        }

        fn has_write_path(&self, path: &str) -> bool {
            if let Ok(guard) = self.writes.lock() {
                return guard.iter().any(|(key, _)| key == path);
            }
            false
        }

        fn deletes_len(&self) -> usize {
            if let Ok(guard) = self.deletes.lock() {
                return guard.len();
            }
            0
        }

        fn has_delete_path(&self, path: &str) -> bool {
            if let Ok(guard) = self.deletes.lock() {
                return guard.iter().any(|key| key == path);
            }
            false
        }

        fn push_event(&self, event: WatchEvent) -> Result<(), WorkerError> {
            let mut guard = self
                .events
                .lock()
                .map_err(|_| WorkerError::Message("events lock poisoned".to_string()))?;
            guard.push_back(event);
            Ok(())
        }

        fn first_write_path(&self) -> Option<String> {
            if let Ok(guard) = self.writes.lock() {
                return guard.first().map(|(path, _)| path.clone());
            }
            None
        }
    }

    impl DcsStore for RecordingStore {
        fn healthy(&self) -> bool {
            true
        }

        fn read_path(&mut self, _path: &str) -> Result<Option<String>, DcsStoreError> {
            Ok(None)
        }

        fn write_path(&mut self, path: &str, value: String) -> Result<(), DcsStoreError> {
            if self.fail_write {
                return Err(DcsStoreError::Io("forced write failure".to_string()));
            }
            let mut guard = self
                .writes
                .lock()
                .map_err(|_| DcsStoreError::Io("writes lock poisoned".to_string()))?;
            guard.push((path.to_string(), value));
            Ok(())
        }

        fn put_path_if_absent(&mut self, path: &str, value: String) -> Result<bool, DcsStoreError> {
            if self.fail_write {
                return Err(DcsStoreError::Io("forced write failure".to_string()));
            }
            if self.reject_put_if_absent {
                return Ok(false);
            }
            let mut guard = self
                .writes
                .lock()
                .map_err(|_| DcsStoreError::Io("writes lock poisoned".to_string()))?;
            guard.push((path.to_string(), value));
            Ok(true)
        }

        fn delete_path(&mut self, path: &str) -> Result<(), DcsStoreError> {
            if self.fail_delete {
                return Err(DcsStoreError::Io("forced delete failure".to_string()));
            }
            if let Some(started) = &self.delete_block_started {
                let mut guard = started
                    .lock()
                    .map_err(|_| DcsStoreError::Io("delete start lock poisoned".to_string()))?;
                if let Some(tx) = guard.take() {
                    tx.send(())
                        .map_err(|_| DcsStoreError::Io("delete start signal failed".to_string()))?;
                }
            }
            if let Some(release) = &self.delete_block_release {
                let guard = release
                    .lock()
                    .map_err(|_| DcsStoreError::Io("delete release lock poisoned".to_string()))?;
                match guard.recv_timeout(Duration::from_secs(5)) {
                    Ok(()) => {}
                    Err(RecvTimeoutError::Timeout) => {
                        return Err(DcsStoreError::Io(
                            "delete release unblock timed out".to_string(),
                        ));
                    }
                    Err(RecvTimeoutError::Disconnected) => {
                        return Err(DcsStoreError::Io(
                            "delete release unblock disconnected".to_string(),
                        ));
                    }
                }
            }
            let mut guard = self
                .deletes
                .lock()
                .map_err(|_| DcsStoreError::Io("deletes lock poisoned".to_string()))?;
            guard.push(path.to_string());
            Ok(())
        }

        fn drain_watch_events(&mut self) -> Result<Vec<WatchEvent>, DcsStoreError> {
            let mut guard = self
                .events
                .lock()
                .map_err(|_| DcsStoreError::Io("events lock poisoned".to_string()))?;
            Ok(guard.drain(..).collect())
        }
    }

    static TEST_DATA_DIR_SEQ: AtomicU64 = AtomicU64::new(0);

    fn test_now_unix_millis() -> UnixMillis {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| {
                u64::try_from(duration.as_millis()).map_or(u64::MAX, |value| value)
            });
        UnixMillis(millis)
    }

    fn unique_test_data_dir(label: &str) -> PathBuf {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_millis());
        let sequence = TEST_DATA_DIR_SEQ.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "pgtuskmaster-worker-{label}-{}-{millis}-{sequence}",
            std::process::id(),
        ))
    }

    fn sample_runtime_config() -> RuntimeConfig {
        crate::test_harness::runtime_config::RuntimeConfigBuilder::new()
            .with_postgres_data_dir(unique_test_data_dir("pgdata"))
            .build()
    }

    fn sample_pg_common(sql: SqlStatus) -> PgInfoCommon {
        PgInfoCommon {
            worker: WorkerStatus::Running,
            sql,
            readiness: Readiness::Ready,
            timeline: None,
            pg_config: PgConfig {
                port: None,
                hot_standby: None,
                primary_conninfo: None,
                primary_slot_name: None,
                extra: BTreeMap::new(),
            },
            last_refresh_at: Some(UnixMillis(1)),
        }
    }

    fn sample_pg_state(sql: SqlStatus) -> PgInfoState {
        PgInfoState::Unknown {
            common: sample_pg_common(sql),
        }
    }

    fn sample_primary_pg_state(sql: SqlStatus) -> PgInfoState {
        PgInfoState::Primary {
            common: sample_pg_common(sql),
            wal_lsn: crate::state::WalLsn(1),
            slots: Vec::new(),
        }
    }

    fn sample_dcs_state(config: RuntimeConfig, trust: DcsTrust) -> DcsState {
        DcsState {
            worker: WorkerStatus::Running,
            trust,
            cache: DcsCache {
                members: BTreeMap::new(),
                leader: None,
                switchover: None,
                config,
                init_lock: None,
            },
            last_refresh_at: Some(UnixMillis(1)),
        }
    }

    fn sample_process_state() -> ProcessState {
        ProcessState::Idle {
            worker: WorkerStatus::Running,
            last_outcome: None,
        }
    }

    fn sample_ha_state() -> HaState {
        HaState {
            worker: WorkerStatus::Starting,
            phase: HaPhase::Init,
            tick: 0,
            decision: HaDecision::NoChange,
        }
    }

    fn sample_process_defaults() -> ProcessDispatchDefaults {
        ProcessDispatchDefaults::contract_stub()
    }

    fn monotonic_clock(start: u64) -> Box<dyn FnMut() -> Result<UnixMillis, WorkerError> + Send> {
        let clock = Arc::new(Mutex::new(start));
        Box::new(move || {
            let mut guard = clock
                .lock()
                .map_err(|_| WorkerError::Message("clock lock poisoned".to_string()))?;
            let now = *guard;
            *guard = guard.saturating_add(1);
            Ok(UnixMillis(now))
        })
    }

    struct BuiltContext {
        ctx: HaWorkerCtx,
        ha_subscriber: crate::state::StateSubscriber<HaState>,
        _config_publisher: crate::state::StatePublisher<RuntimeConfig>,
        pg_publisher: crate::state::StatePublisher<PgInfoState>,
        _dcs_publisher: crate::state::StatePublisher<DcsState>,
        _process_publisher: crate::state::StatePublisher<ProcessState>,
        process_rx: tokio::sync::mpsc::UnboundedReceiver<ProcessJobRequest>,
        store: RecordingStore,
    }

    #[derive(Clone)]
    struct HaWorkerTestBuilder {
        store: RecordingStore,
        poll_interval: Duration,
        dcs_trust: DcsTrust,
        initial_phase: HaPhase,
        initial_tick: u64,
        initial_decision: HaDecision,
        pg_state: PgInfoState,
        process_state: ProcessState,
    }

    impl HaWorkerTestBuilder {
        fn new() -> Self {
            Self {
                store: RecordingStore::default(),
                poll_interval: Duration::from_millis(100),
                dcs_trust: DcsTrust::FullQuorum,
                initial_phase: HaPhase::Init,
                initial_tick: 0,
                initial_decision: HaDecision::NoChange,
                pg_state: sample_pg_state(SqlStatus::Healthy),
                process_state: sample_process_state(),
            }
        }

        fn with_store(self, store: RecordingStore) -> Self {
            Self { store, ..self }
        }

        fn with_poll_interval(self, poll_interval: Duration) -> Self {
            Self {
                poll_interval,
                ..self
            }
        }

        fn with_dcs_trust(self, dcs_trust: DcsTrust) -> Self {
            Self { dcs_trust, ..self }
        }

        fn with_phase(self, initial_phase: HaPhase) -> Self {
            Self {
                initial_phase,
                ..self
            }
        }

        fn with_tick(self, initial_tick: u64) -> Self {
            Self {
                initial_tick,
                ..self
            }
        }

        fn with_pg_state(self, pg_state: PgInfoState) -> Self {
            Self { pg_state, ..self }
        }

        fn build(self) -> Result<BuiltContext, WorkerError> {
            let BuiltContext {
                mut ctx,
                ha_subscriber,
                _config_publisher,
                pg_publisher,
                _dcs_publisher,
                _process_publisher,
                process_rx,
                store,
            } = build_context(self.store, self.poll_interval, self.dcs_trust);

            pg_publisher
                .publish(self.pg_state, UnixMillis(50))
                .map_err(|err| WorkerError::Message(format!("pg publish failed: {err}")))?;
            _process_publisher
                .publish(self.process_state, UnixMillis(50))
                .map_err(|err| WorkerError::Message(format!("process publish failed: {err}")))?;
            ctx.state = HaState {
                worker: WorkerStatus::Running,
                phase: self.initial_phase,
                tick: self.initial_tick,
                decision: self.initial_decision,
            };

            Ok(BuiltContext {
                ctx,
                ha_subscriber,
                _config_publisher,
                pg_publisher,
                _dcs_publisher,
                _process_publisher,
                process_rx,
                store,
            })
        }
    }

    fn build_context(
        store: RecordingStore,
        poll_interval: Duration,
        dcs_trust: DcsTrust,
    ) -> BuiltContext {
        let runtime_config = sample_runtime_config();
        let (config_publisher, config_subscriber) =
            new_state_channel(runtime_config.clone(), UnixMillis(1));
        let (pg_publisher, pg_subscriber) =
            new_state_channel(sample_pg_state(SqlStatus::Healthy), UnixMillis(1));
        let dcs_state = sample_dcs_state(runtime_config.clone(), dcs_trust);
        let (dcs_publisher, dcs_subscriber) = new_state_channel(dcs_state, UnixMillis(1));
        let (process_publisher, process_subscriber) =
            new_state_channel(sample_process_state(), UnixMillis(1));
        let (ha_publisher, ha_subscriber) = new_state_channel(sample_ha_state(), UnixMillis(1));
        let (process_tx, process_rx) = tokio::sync::mpsc::unbounded_channel();

        let mut ctx = HaWorkerCtx::contract_stub(HaWorkerContractStubInputs {
            publisher: ha_publisher,
            config_subscriber,
            pg_subscriber,
            dcs_subscriber,
            process_subscriber,
            process_inbox: process_tx,
            dcs_store: Box::new(store.clone()),
            scope: "scope-a".to_string(),
            self_id: MemberId("node-a".to_string()),
        });
        ctx.poll_interval = poll_interval;
        ctx.state = sample_ha_state();
        ctx.process_defaults = sample_process_defaults();
        ctx.now = monotonic_clock(10);

        BuiltContext {
            ctx,
            ha_subscriber,
            _config_publisher: config_publisher,
            pg_publisher,
            _dcs_publisher: dcs_publisher,
            _process_publisher: process_publisher,
            process_rx,
            store,
        }
    }

    #[derive(Clone)]
    struct ScriptedProcess {
        polls: VecDeque<Result<Option<ProcessExit>, ProcessError>>,
        cancel_result: Result<(), ProcessError>,
    }

    struct ScriptedHandle {
        polls: VecDeque<Result<Option<ProcessExit>, ProcessError>>,
        cancel_result: Result<(), ProcessError>,
    }

    impl ProcessHandle for ScriptedHandle {
        fn poll_exit(&mut self) -> Result<Option<ProcessExit>, ProcessError> {
            match self.polls.pop_front() {
                Some(next) => next,
                None => Ok(None),
            }
        }

        fn drain_output<'a>(
            &'a mut self,
            _max_bytes: usize,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<
                        Output = Result<Vec<crate::process::jobs::ProcessOutputLine>, ProcessError>,
                    > + Send
                    + 'a,
            >,
        > {
            Box::pin(async move { Ok(Vec::new()) })
        }

        fn cancel<'a>(
            &'a mut self,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<(), ProcessError>> + Send + 'a>,
        > {
            let result = self.cancel_result.clone();
            Box::pin(async move { result })
        }
    }

    #[derive(Clone, Default)]
    struct ScriptedRunner {
        scripts: Arc<Mutex<VecDeque<Result<ScriptedProcess, ProcessError>>>>,
        spawned_specs: Arc<Mutex<Vec<ProcessCommandSpec>>>,
    }

    impl ScriptedRunner {
        fn queue_success_exit(&self) -> Result<(), WorkerError> {
            let mut scripts = self
                .scripts
                .lock()
                .map_err(|_| WorkerError::Message("scripts lock poisoned".to_string()))?;
            scripts.push_back(Ok(ScriptedProcess {
                polls: VecDeque::from(vec![Ok(Some(ProcessExit::Success))]),
                cancel_result: Ok(()),
            }));
            Ok(())
        }

        fn any_spawn_contains_arg(&self, needle: &str) -> bool {
            if let Ok(specs) = self.spawned_specs.lock() {
                return specs
                    .iter()
                    .any(|spec| spec.args.iter().any(|arg| arg == needle));
            }
            false
        }
    }

    impl ProcessCommandRunner for ScriptedRunner {
        fn spawn(
            &mut self,
            spec: ProcessCommandSpec,
        ) -> Result<Box<dyn ProcessHandle>, ProcessError> {
            {
                let mut spawned = self
                    .spawned_specs
                    .lock()
                    .map_err(|_| ProcessError::OperationFailed)?;
                spawned.push(spec);
            }
            let scripted = {
                let mut scripts = self
                    .scripts
                    .lock()
                    .map_err(|_| ProcessError::OperationFailed)?;
                match scripts.pop_front() {
                    Some(next) => next,
                    None => Err(ProcessError::InvalidSpec(
                        "scripted runner queue exhausted".to_string(),
                    )),
                }
            }?;

            Ok(Box::new(ScriptedHandle {
                polls: scripted.polls,
                cancel_result: scripted.cancel_result,
            }))
        }
    }

    struct IntegrationFixture {
        store: RecordingStore,
        runner: ScriptedRunner,
        _config_publisher: crate::state::StatePublisher<RuntimeConfig>,
        pg_publisher: crate::state::StatePublisher<PgInfoState>,
        dcs_subscriber: crate::state::StateSubscriber<DcsState>,
        process_subscriber: crate::state::StateSubscriber<ProcessState>,
        ha_subscriber: crate::state::StateSubscriber<HaState>,
        dcs_ctx: crate::dcs::state::DcsWorkerCtx,
        process_ctx: ProcessWorkerCtx,
        ha_ctx: HaWorkerCtx,
        next_revision: i64,
    }

    impl IntegrationFixture {
        fn new(initial_phase: HaPhase) -> Self {
            let runtime_config = sample_runtime_config();
            let store = RecordingStore::default();
            let runner = ScriptedRunner::default();

            let (config_publisher, config_subscriber) =
                new_state_channel(runtime_config.clone(), UnixMillis(1));
            let (pg_publisher, pg_subscriber) =
                new_state_channel(sample_pg_state(SqlStatus::Healthy), UnixMillis(1));
            let (dcs_publisher, dcs_subscriber) = new_state_channel(
                sample_dcs_state(runtime_config.clone(), DcsTrust::NotTrusted),
                UnixMillis(1),
            );
            let (process_publisher, process_subscriber) =
                new_state_channel(sample_process_state(), UnixMillis(1));
            let (ha_publisher, ha_subscriber) = new_state_channel(sample_ha_state(), UnixMillis(1));
            let (process_tx, process_rx) = tokio::sync::mpsc::unbounded_channel();

            let dcs_ctx = crate::dcs::state::DcsWorkerCtx {
                self_id: MemberId("node-a".to_string()),
                scope: "scope-a".to_string(),
                poll_interval: TEST_DCS_AND_PROCESS_POLL_INTERVAL,
                local_postgres_host: runtime_config.postgres.listen_host.clone(),
                local_postgres_port: runtime_config.postgres.listen_port,
                pg_subscriber: pg_subscriber.clone(),
                publisher: dcs_publisher,
                store: Box::new(store.clone()),
                log: crate::logging::LogHandle::null(),
                cache: DcsCache {
                    members: BTreeMap::new(),
                    leader: None,
                    switchover: None,
                    config: runtime_config.clone(),
                    init_lock: None,
                },
                last_published_pg_version: None,
                last_emitted_store_healthy: None,
                last_emitted_trust: None,
            };

            let mut process_ctx = ProcessWorkerCtx::contract_stub(
                runtime_config.process.clone(),
                process_publisher,
                process_rx,
            );
            process_ctx.poll_interval = TEST_DCS_AND_PROCESS_POLL_INTERVAL;
            process_ctx.command_runner = Box::new(runner.clone());
            process_ctx.now = monotonic_clock(100);

            let mut ha_ctx = HaWorkerCtx::contract_stub(HaWorkerContractStubInputs {
                publisher: ha_publisher,
                config_subscriber,
                pg_subscriber,
                dcs_subscriber: dcs_subscriber.clone(),
                process_subscriber: process_subscriber.clone(),
                process_inbox: process_tx,
                dcs_store: Box::new(store.clone()),
                scope: "scope-a".to_string(),
                self_id: MemberId("node-a".to_string()),
            });
            ha_ctx.poll_interval = TEST_DCS_AND_PROCESS_POLL_INTERVAL;
            ha_ctx.process_defaults = sample_process_defaults();
            ha_ctx.now = monotonic_clock(1_000);
            ha_ctx.state = HaState {
                worker: WorkerStatus::Running,
                phase: initial_phase,
                tick: 0,
                decision: HaDecision::NoChange,
            };

            Self {
                store,
                runner,
                _config_publisher: config_publisher,
                pg_publisher,
                dcs_subscriber,
                process_subscriber,
                ha_subscriber,
                dcs_ctx,
                process_ctx,
                ha_ctx,
                next_revision: 1,
            }
        }

        fn queue_process_success(&self) -> Result<(), WorkerError> {
            self.runner.queue_success_exit()
        }

        fn publish_pg_sql(&self, status: SqlStatus, now: u64) -> Result<(), WorkerError> {
            self.publish_pg_sql_state(sample_pg_state(status), now)
        }

        fn publish_pg_sql_state(&self, state: PgInfoState, now: u64) -> Result<(), WorkerError> {
            self.pg_publisher
                .publish(state, UnixMillis(now))
                .map(|_| ())
                .map_err(|err| WorkerError::Message(format!("pg publish failed: {err}")))
        }

        fn push_member_event(
            &mut self,
            member_id: &str,
            role: MemberRole,
        ) -> Result<(), WorkerError> {
            let record = sample_member_record(member_id, role);
            let value = serde_json::to_string(&record)
                .map_err(|err| WorkerError::Message(format!("member encode failed: {err}")))?;
            let event = WatchEvent {
                op: WatchOp::Put,
                path: format!("/scope-a/member/{member_id}"),
                value: Some(value),
                revision: self.take_revision(),
            };
            self.store.push_event(event)
        }

        fn push_leader_event(&mut self, member_id: &str) -> Result<(), WorkerError> {
            let record = LeaderRecord {
                member_id: MemberId(member_id.to_string()),
            };
            let value = serde_json::to_string(&record)
                .map_err(|err| WorkerError::Message(format!("leader encode failed: {err}")))?;
            let event = WatchEvent {
                op: WatchOp::Put,
                path: "/scope-a/leader".to_string(),
                value: Some(value),
                revision: self.take_revision(),
            };
            self.store.push_event(event)
        }

        fn delete_leader_event(&mut self) -> Result<(), WorkerError> {
            let event = WatchEvent {
                op: WatchOp::Delete,
                path: "/scope-a/leader".to_string(),
                value: None,
                revision: self.take_revision(),
            };
            self.store.push_event(event)
        }

        async fn step_dcs_and_ha(&mut self) -> Result<(), WorkerError> {
            crate::dcs::worker::step_once(&mut self.dcs_ctx).await?;
            step_once(&mut self.ha_ctx).await
        }

        async fn step_dcs_ha_process_ha(&mut self) -> Result<(), WorkerError> {
            crate::dcs::worker::step_once(&mut self.dcs_ctx).await?;
            step_once(&mut self.ha_ctx).await?;
            crate::process::worker::step_once(&mut self.process_ctx).await?;
            step_once(&mut self.ha_ctx).await
        }

        fn latest_ha(&self) -> HaState {
            self.ha_subscriber.latest().value
        }

        fn latest_dcs(&self) -> DcsState {
            self.dcs_subscriber.latest().value
        }

        fn latest_process(&self) -> ProcessState {
            self.process_subscriber.latest().value
        }

        fn take_revision(&mut self) -> i64 {
            let current = self.next_revision;
            self.next_revision = self.next_revision.saturating_add(1);
            current
        }
    }

    fn sample_member_record(member_id: &str, role: MemberRole) -> MemberRecord {
        MemberRecord {
            member_id: MemberId(member_id.to_string()),
            postgres_host: "10.0.0.10".to_string(),
            postgres_port: 5432,
            role,
            sql: SqlStatus::Healthy,
            readiness: Readiness::Ready,
            timeline: None,
            write_lsn: None,
            replay_lsn: None,
            updated_at: test_now_unix_millis(),
            pg_version: Version(1),
        }
    }

    async fn wait_for_ha_version(
        subscriber: &crate::state::StateSubscriber<HaState>,
        min_version: u64,
        timeout: Duration,
    ) -> bool {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            if subscriber.latest().version.0 >= min_version {
                return true;
            }
            if tokio::time::Instant::now() >= deadline {
                return false;
            }
            tokio::time::sleep(TEST_DCS_AND_PROCESS_POLL_INTERVAL).await;
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_uses_subscribers_and_publishes_next_state() -> Result<(), WorkerError> {
        let BuiltContext {
            mut ctx,
            ha_subscriber: subscriber,
            ..
        } = HaWorkerTestBuilder::new().build()?;

        let stepped = step_once(&mut ctx).await;
        assert_eq!(stepped, Ok(()));
        assert_eq!(ctx.state.phase, HaPhase::WaitingPostgresReachable);
        assert_eq!(ctx.state.tick, 1);
        assert_eq!(ctx.state.worker, WorkerStatus::Running);

        let published = subscriber.latest();
        assert_eq!(published.version, Version(1));
        assert_eq!(published.value.phase, HaPhase::WaitingPostgresReachable);
        assert_eq!(published.value.tick, 1);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_suppresses_duplicate_start_postgres_dispatch_while_unreachable(
    ) -> Result<(), WorkerError> {
        let BuiltContext {
            mut ctx,
            ha_subscriber: _ha_subscriber,
            mut process_rx,
            ..
        } = HaWorkerTestBuilder::new()
            .with_phase(HaPhase::WaitingPostgresReachable)
            .with_pg_state(sample_pg_state(SqlStatus::Unreachable))
            .build()?;

        step_once(&mut ctx).await?;
        let first = process_rx.try_recv().map_err(|err| {
            WorkerError::Message(format!(
                "expected process job request after first tick: {err}"
            ))
        })?;
        assert!(matches!(first.kind, ProcessJobKind::StartPostgres(_)));

        step_once(&mut ctx).await?;
        assert_eq!(process_rx.try_recv(), Err(TryRecvError::Empty));
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_matches_decide_output_for_same_snapshot() -> Result<(), WorkerError> {
        let BuiltContext {
            mut ctx,
            ha_subscriber,
            ..
        } = HaWorkerTestBuilder::new()
            .with_phase(HaPhase::WaitingDcsTrusted)
            .with_tick(7)
            .build()?;

        let expected = crate::ha::decide::decide(DecideInput {
            current: ctx.state.clone(),
            world: super::world_snapshot(&ctx),
        });

        step_once(&mut ctx).await?;

        assert_eq!(ctx.state.phase, expected.next.phase);
        assert_eq!(ctx.state.tick, expected.next.tick);
        assert_eq!(ctx.state.decision, expected.next.decision);
        assert_eq!(ha_subscriber.latest().value, ctx.state);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_primary_quorum_loss_enqueues_fencing_without_releasing_lease(
    ) -> Result<(), WorkerError> {
        let BuiltContext {
            mut ctx,
            ha_subscriber: _ha_subscriber,
            mut process_rx,
            store,
            ..
        } = HaWorkerTestBuilder::new()
            .with_phase(HaPhase::Primary)
            .with_dcs_trust(DcsTrust::NotTrusted)
            .with_pg_state(sample_primary_pg_state(SqlStatus::Healthy))
            .build()?;

        step_once(&mut ctx).await?;

        assert_eq!(ctx.state.phase, HaPhase::FailSafe);
        assert_eq!(
            ctx.state.decision,
            HaDecision::EnterFailSafe {
                release_leader_lease: false,
            }
        );
        assert!(!store.has_delete_path("/scope-a/leader"));
        let request = process_rx.try_recv().map_err(|err| {
            WorkerError::Message(format!("expected fencing dispatch during fail-safe: {err}"))
        })?;
        assert!(matches!(request.kind, ProcessJobKind::Fencing(_)));
        assert_eq!(process_rx.try_recv(), Err(TryRecvError::Empty));
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_failsafe_primary_with_restored_quorum_attempts_leadership(
    ) -> Result<(), WorkerError> {
        let BuiltContext {
            mut ctx,
            ha_subscriber,
            mut process_rx,
            store: store_handle,
            ..
        } = HaWorkerTestBuilder::new()
            .with_phase(HaPhase::FailSafe)
            .with_dcs_trust(DcsTrust::FullQuorum)
            .with_pg_state(sample_primary_pg_state(SqlStatus::Healthy))
            .build()?;

        step_once(&mut ctx).await?;

        let published = ha_subscriber.latest();
        assert_eq!(published.version, Version(1));
        assert_eq!(published.value.phase, HaPhase::Primary);
        assert_eq!(published.value.decision, HaDecision::AttemptLeadership);
        assert_eq!(published.value.worker, WorkerStatus::Running);
        assert_eq!(ctx.state.phase, HaPhase::Primary);
        assert_eq!(ctx.state.decision, HaDecision::AttemptLeadership);
        assert_eq!(
            store_handle.first_write_path().as_deref(),
            Some("/scope-a/leader")
        );
        assert!(!store_handle.has_delete_path("/scope-a/leader"));
        assert_eq!(process_rx.try_recv(), Err(TryRecvError::Empty));
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_primary_outage_enqueues_only_recovery_dispatch() -> Result<(), WorkerError> {
        let BuiltContext {
            mut ctx,
            ha_subscriber: _ha_subscriber,
            _dcs_publisher: dcs_publisher,
            mut process_rx,
            store,
            ..
        } = HaWorkerTestBuilder::new()
            .with_phase(HaPhase::Primary)
            .with_pg_state(sample_pg_state(SqlStatus::Unreachable))
            .build()?;
        let mut dcs_state = sample_dcs_state(sample_runtime_config(), DcsTrust::FullQuorum);
        let leader_member = sample_member_record("node-b", MemberRole::Primary);
        dcs_state
            .cache
            .members
            .insert(leader_member.member_id.clone(), leader_member.clone());
        dcs_state.cache.leader = Some(LeaderRecord {
            member_id: leader_member.member_id,
        });
        dcs_publisher
            .publish(dcs_state, UnixMillis(60))
            .map_err(|err| WorkerError::Message(format!("dcs publish failed: {err}")))?;

        step_once(&mut ctx).await?;

        let first = process_rx.try_recv().map_err(|err| {
            WorkerError::Message(format!("expected a recovery process request: {err}"))
        })?;
        assert!(matches!(first.kind, ProcessJobKind::PgRewind(_)));
        assert_eq!(process_rx.try_recv(), Err(TryRecvError::Empty));
        assert_eq!(store.writes_len(), 0);
        assert_eq!(store.deletes_len(), 0);
        assert_eq!(ctx.state.phase, HaPhase::Rewinding);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn apply_effect_plan_maps_dcs_and_process_requests() -> Result<(), WorkerError> {
        let BuiltContext {
            mut ctx,
            mut process_rx,
            store,
            ..
        } = HaWorkerTestBuilder::new().build()?;
        let plan = HaEffectPlan {
            lease: LeaseEffect::ReleaseLeader,
            switchover: SwitchoverEffect::None,
            replication: ReplicationEffect::None,
            postgres: PostgresEffect::Start,
            safety: SafetyEffect::None,
        };
        let acquire_only = HaEffectPlan {
            lease: LeaseEffect::AcquireLeader,
            switchover: SwitchoverEffect::None,
            replication: ReplicationEffect::None,
            postgres: PostgresEffect::None,
            safety: SafetyEffect::None,
        };

        let ha_tick = ctx.state.tick;
        let acquire_errors = apply_effect_plan(&mut ctx, ha_tick, &acquire_only)?;
        assert!(acquire_errors.is_empty());
        let errors = apply_effect_plan(&mut ctx, ha_tick, &plan)?;
        assert!(errors.is_empty(), "dispatch errors were: {errors:?}");
        assert_eq!(store.writes_len(), 1);
        assert_eq!(store.deletes_len(), 1);
        assert_eq!(
            store.first_write_path(),
            Some("/scope-a/leader".to_string())
        );

        let request = process_rx.try_recv();
        assert!(request.is_ok());
        if let Ok(job) = request {
            assert!(matches!(job.kind, ProcessJobKind::StartPostgres(_)));
            if let ProcessJobKind::StartPostgres(spec) = job.kind {
                assert_eq!(
                    spec.data_dir,
                    ctx.config_subscriber.latest().value.postgres.data_dir
                );
                assert_eq!(
                    spec.config_file,
                    ctx.config_subscriber
                        .latest()
                        .value
                        .postgres
                        .data_dir
                        .join("pgtm.postgresql.conf")
                );
            }
        }
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn apply_effect_plan_is_best_effort_and_reports_typed_errors() -> Result<(), WorkerError>
    {
        let store = RecordingStore {
            fail_write: true,
            ..RecordingStore::default()
        };
        let BuiltContext {
            mut ctx,
            process_rx,
            store: store_handle,
            ..
        } = HaWorkerTestBuilder::new().with_store(store).build()?;
        drop(process_rx);

        let plan = HaEffectPlan {
            lease: LeaseEffect::ReleaseLeader,
            switchover: SwitchoverEffect::None,
            replication: ReplicationEffect::None,
            postgres: PostgresEffect::Start,
            safety: SafetyEffect::None,
        };
        let acquire_only = HaEffectPlan {
            lease: LeaseEffect::AcquireLeader,
            switchover: SwitchoverEffect::None,
            replication: ReplicationEffect::None,
            postgres: PostgresEffect::None,
            safety: SafetyEffect::None,
        };
        let ha_tick = ctx.state.tick;
        let mut errors = apply_effect_plan(&mut ctx, ha_tick, &acquire_only)?;
        errors.extend(apply_effect_plan(&mut ctx, ha_tick, &plan)?);

        assert_eq!(store_handle.deletes_len(), 1);
        assert_eq!(errors.len(), 2);
        assert!(errors.iter().any(|err| matches!(
            err,
            ActionDispatchError::DcsWrite {
                action: ActionId::AcquireLeaderLease,
                ..
            }
        )));
        assert!(
            errors.iter().any(|err| matches!(
                err,
                ActionDispatchError::ProcessSend {
                    action: ActionId::StartPostgres,
                    ..
                }
            )),
            "dispatch errors were: {errors:?}"
        );
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn apply_effect_plan_clears_switchover_key() -> Result<(), WorkerError> {
        let BuiltContext { mut ctx, store, .. } = HaWorkerTestBuilder::new().build()?;

        let ha_tick = ctx.state.tick;
        let plan = HaEffectPlan {
            lease: LeaseEffect::None,
            switchover: SwitchoverEffect::ClearRequest,
            replication: ReplicationEffect::None,
            postgres: PostgresEffect::None,
            safety: SafetyEffect::None,
        };
        let errors = apply_effect_plan(&mut ctx, ha_tick, &plan)?;
        assert!(errors.is_empty());
        assert!(store.has_delete_path("/scope-a/switchover"));
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn apply_effect_plan_surfaces_leader_lease_conflict() -> Result<(), WorkerError> {
        let store = RecordingStore {
            reject_put_if_absent: true,
            ..RecordingStore::default()
        };
        let BuiltContext { mut ctx, .. } = HaWorkerTestBuilder::new()
            .with_store(store.clone())
            .build()?;

        let ha_tick = ctx.state.tick;
        let plan = HaEffectPlan {
            lease: LeaseEffect::AcquireLeader,
            switchover: SwitchoverEffect::None,
            replication: ReplicationEffect::None,
            postgres: PostgresEffect::None,
            safety: SafetyEffect::None,
        };

        let errors = apply_effect_plan(&mut ctx, ha_tick, &plan)?;

        assert_eq!(store.writes_len(), 0);
        assert_eq!(errors.len(), 1);
        assert_eq!(
            errors[0],
            ActionDispatchError::DcsWrite {
                action: ActionId::AcquireLeaderLease,
                path: "/scope-a/leader".to_string(),
                message: "path already exists: /scope-a/leader".to_string(),
            }
        );
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn integration_transitions_replica_candidate_primary_and_failsafe() {
        let mut fixture = IntegrationFixture::new(HaPhase::WaitingDcsTrusted);

        assert_eq!(fixture.publish_pg_sql(SqlStatus::Healthy, 10), Ok(()));
        assert_eq!(
            fixture.push_member_event("node-b", MemberRole::Primary),
            Ok(())
        );
        assert_eq!(fixture.push_leader_event("node-b"), Ok(()));
        assert_eq!(fixture.step_dcs_and_ha().await, Ok(()));

        let replica = fixture.latest_ha();
        assert_eq!(replica.phase, HaPhase::Replica);
        assert_eq!(
            lower_decision(&replica.decision),
            HaEffectPlan {
                lease: LeaseEffect::None,
                switchover: SwitchoverEffect::None,
                replication: ReplicationEffect::FollowLeader {
                    leader_member_id: MemberId("node-b".to_string()),
                },
                postgres: PostgresEffect::None,
                safety: SafetyEffect::None,
            }
        );
        assert_eq!(fixture.latest_dcs().trust, DcsTrust::FullQuorum);

        assert_eq!(fixture.delete_leader_event(), Ok(()));
        assert_eq!(fixture.step_dcs_and_ha().await, Ok(()));
        let candidate = fixture.latest_ha();
        assert_eq!(candidate.phase, HaPhase::CandidateLeader);
        assert_eq!(
            lower_decision(&candidate.decision),
            HaEffectPlan {
                lease: LeaseEffect::AcquireLeader,
                switchover: SwitchoverEffect::None,
                replication: ReplicationEffect::None,
                postgres: PostgresEffect::None,
                safety: SafetyEffect::None,
            }
        );
        assert!(fixture.store.has_write_path("/scope-a/leader"));

        assert_eq!(fixture.push_leader_event("node-a"), Ok(()));
        assert_eq!(fixture.step_dcs_and_ha().await, Ok(()));
        let primary = fixture.latest_ha();
        assert_eq!(primary.phase, HaPhase::Primary);
        assert_eq!(
            lower_decision(&primary.decision),
            HaEffectPlan {
                lease: LeaseEffect::None,
                switchover: SwitchoverEffect::None,
                replication: ReplicationEffect::None,
                postgres: PostgresEffect::Promote,
                safety: SafetyEffect::None,
            }
        );

        assert_eq!(
            fixture.publish_pg_sql_state(sample_primary_pg_state(SqlStatus::Healthy), 20),
            Ok(())
        );
        assert_eq!(fixture.push_leader_event("node-z"), Ok(()));
        assert_eq!(fixture.step_dcs_and_ha().await, Ok(()));
        let failsafe = fixture.latest_ha();
        assert_eq!(failsafe.phase, HaPhase::FailSafe);
        assert_eq!(
            lower_decision(&failsafe.decision).safety,
            SafetyEffect::FenceNode
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn integration_primary_unreachable_rewinds_then_returns_replica_on_success() {
        let mut fixture = IntegrationFixture::new(HaPhase::Primary);

        assert_eq!(
            fixture.push_member_event("node-b", MemberRole::Primary),
            Ok(())
        );
        assert_eq!(fixture.push_leader_event("node-b"), Ok(()));
        assert_eq!(fixture.publish_pg_sql(SqlStatus::Unreachable, 20), Ok(()));
        assert_eq!(fixture.queue_process_success(), Ok(()));
        assert_eq!(fixture.step_dcs_ha_process_ha().await, Ok(()));

        let latest_ha = fixture.latest_ha();
        assert_eq!(latest_ha.phase, HaPhase::Replica);
        assert!(fixture.runner.any_spawn_contains_arg("--target-pgdata"));
        assert!(matches!(
            fixture.latest_process(),
            ProcessState::Idle {
                last_outcome: Some(JobOutcome::Success { .. }),
                ..
            }
        ));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn integration_primary_split_brain_enters_fencing_and_process_feedback_advances() {
        let mut fixture = IntegrationFixture::new(HaPhase::Primary);

        assert_eq!(fixture.publish_pg_sql(SqlStatus::Healthy, 30), Ok(()));
        assert_eq!(
            fixture.push_member_event("node-b", MemberRole::Primary),
            Ok(())
        );
        assert_eq!(fixture.push_leader_event("node-b"), Ok(()));
        assert_eq!(fixture.step_dcs_and_ha().await, Ok(()));

        let fencing = fixture.latest_ha();
        assert_eq!(fencing.phase, HaPhase::Fencing);
        assert_eq!(
            lower_decision(&fencing.decision),
            HaEffectPlan {
                lease: LeaseEffect::ReleaseLeader,
                switchover: SwitchoverEffect::None,
                replication: ReplicationEffect::None,
                postgres: PostgresEffect::Demote,
                safety: SafetyEffect::FenceNode,
            }
        );
        assert!(fixture.store.has_delete_path("/scope-a/leader"));

        assert_eq!(fixture.queue_process_success(), Ok(()));
        assert_eq!(fixture.queue_process_success(), Ok(()));
        let process_version_before = fixture.process_subscriber.latest().version;
        assert_eq!(
            crate::process::worker::step_once(&mut fixture.process_ctx).await,
            Ok(())
        );
        assert_eq!(step_once(&mut fixture.ha_ctx).await, Ok(()));
        assert_eq!(fixture.latest_ha().phase, HaPhase::Fencing);

        let process_version_mid = fixture.process_subscriber.latest().version;
        assert_eq!(
            process_version_mid.0,
            process_version_before.0.saturating_add(2)
        );

        assert_eq!(
            crate::process::worker::step_once(&mut fixture.process_ctx).await,
            Ok(())
        );
        assert_eq!(step_once(&mut fixture.ha_ctx).await, Ok(()));

        let process_version_after = fixture.process_subscriber.latest().version;
        assert!(process_version_after.0 > process_version_mid.0);
        assert!(matches!(
            fixture.latest_process(),
            ProcessState::Idle {
                last_outcome: Some(JobOutcome::Success { .. }),
                ..
            }
        ));
        assert_eq!(fixture.latest_ha().phase, HaPhase::WaitingDcsTrusted);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn integration_start_postgres_dispatch_updates_process_state_versions() {
        let mut fixture = IntegrationFixture::new(HaPhase::WaitingPostgresReachable);

        assert_eq!(fixture.publish_pg_sql(SqlStatus::Unreachable, 40), Ok(()));
        assert_eq!(fixture.step_dcs_and_ha().await, Ok(()));
        let waiting = fixture.latest_ha();
        assert_eq!(waiting.phase, HaPhase::WaitingPostgresReachable);
        assert_eq!(
            lower_decision(&waiting.decision),
            HaEffectPlan {
                lease: LeaseEffect::None,
                switchover: SwitchoverEffect::None,
                replication: ReplicationEffect::None,
                postgres: PostgresEffect::Start,
                safety: SafetyEffect::None,
            }
        );

        assert_eq!(fixture.queue_process_success(), Ok(()));
        let process_version_before = fixture.process_subscriber.latest().version;
        assert_eq!(
            crate::process::worker::step_once(&mut fixture.process_ctx).await,
            Ok(())
        );
        let process_version_after = fixture.process_subscriber.latest().version;
        assert_eq!(
            process_version_after.0,
            process_version_before.0.saturating_add(2)
        );
        assert!(matches!(
            fixture.latest_process(),
            ProcessState::Idle {
                last_outcome: Some(JobOutcome::Success { .. }),
                ..
            }
        ));
        assert!(fixture.runner.any_spawn_contains_arg("start"));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn run_reacts_to_interval_tick_and_watcher_change() -> Result<(), WorkerError> {
        let BuiltContext {
            ctx,
            ha_subscriber: subscriber,
            _config_publisher,
            pg_publisher,
            _dcs_publisher,
            _process_publisher,
            ..
        } = HaWorkerTestBuilder::new()
            .with_poll_interval(Duration::from_millis(20))
            .build()?;

        let handle = tokio::spawn(async move { run(ctx).await });

        let first_advanced =
            wait_for_ha_version(&subscriber, 1, TEST_HA_VERSION_WAIT_TIMEOUT).await;
        assert!(first_advanced);

        let publish_result =
            pg_publisher.publish(sample_pg_state(SqlStatus::Unreachable), UnixMillis(50));
        assert!(publish_result.is_ok());
        let second_advanced =
            wait_for_ha_version(&subscriber, 2, TEST_HA_VERSION_WAIT_TIMEOUT).await;
        assert!(second_advanced);

        handle.abort();
        let _ = handle.await;
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn run_initial_buffered_updates_match_explicit_buffered_prefix() -> Result<(), WorkerError>
    {
        let BuiltContext {
            ctx: mut step_ctx,
            ha_subscriber: _ha_subscriber,
            ..
        } = HaWorkerTestBuilder::new()
            .with_poll_interval(Duration::from_secs(1))
            .build()?;

        let stepped = step_once(&mut step_ctx).await;
        assert_eq!(stepped, Ok(()));
        let expected_after_first = step_ctx.state.clone();
        let stepped = step_once(&mut step_ctx).await;
        assert_eq!(stepped, Ok(()));
        let expected_after_second = step_ctx.state.clone();

        let BuiltContext {
            ctx: run_ctx,
            ha_subscriber: run_subscriber,
            _config_publisher,
            _dcs_publisher,
            _process_publisher,
            ..
        } = HaWorkerTestBuilder::new()
            .with_poll_interval(Duration::from_secs(1))
            .build()?;
        let handle = tokio::spawn(async move { run(run_ctx).await });

        let advanced = wait_for_ha_version(&run_subscriber, 1, TEST_HA_VERSION_WAIT_TIMEOUT).await;
        assert!(advanced);
        let observed = run_subscriber.latest().value;
        assert!(
            observed == expected_after_first || observed == expected_after_second,
            "observed buffered run prefix did not match either explicit prefix: observed={observed:?} first={expected_after_first:?} second={expected_after_second:?}"
        );

        handle.abort();
        let _ = handle.await;
        Ok(())
    }
}

--- END FILE: src/ha/worker.rs ---

