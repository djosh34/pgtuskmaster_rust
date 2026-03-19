## Smell Set: ha-flat-observation-and-decision-collapse <status>completed</status> <passes>true</passes>

Please refer to skill `improve-code-boundaries` to see what smells there are.

Inside dirs:
- `src/ha`

Solve each smell:

---
- [x] Smell 6, raw DTO data not converted once into one shared flat type
HA currently reads `PgInfoState`, `DcsSnapshot`, `ProcessState`, and retained HA state, then projects them into a second HA-only universe before `decide.rs` and `reconcile.rs` immediately unpack that universe again. This is the main boundary failure in HA.

The projection starts in `src/ha/worker.rs::observe(...)` and fans out through:
- `build_data_dir_state`
- `build_storage_state`
- `build_global_knowledge`
- `build_peer_knowledge_from_member`
- `build_self_peer`
- `self_replica_position`
- `observed_primary_member`
- `member_timeline`
- `member_system_identifier`
- `pg_timeline`
- `pg_system_identifier`
- `pg_timeline_id`

The resulting tree is then consumed mostly by `src/ha/decide.rs` and `src/ha/reconcile.rs`, which proves the current HA-local types are acting as a lossy mirror, not as a stable shared boundary.

Mandatory completion threshold for this smell set:
- at least 70% of the listed HA-local mirror/projection types must disappear entirely through deletion or merge
- more removal is better; 70% is only the minimum acceptable outcome
- if that threshold is not reached, the implementer must assume the flattening was not pushed far enough and rethink the diagnosis instead of preserving the extra shapes

Types that must overwhelmingly go away through deletion or merge into a flatter representation:
- `WorldView`
- `LocalKnowledge`
- `GlobalKnowledge`
- `ObservationState`
- `DataDirState`
- `LocalDataState`
- `DivergenceState`
- `PostgresState`
- `ReplicationState`
- `ProcessAssessment`
- `JobFailure`
- `FailureRecovery`
- `StorageState`
- `CoordinationState`
- `QuorumCoordinationState`
- `LeadershipView`
- `PeerLeaderState`
- `PrimaryObservation`
- `ObservedPrimary`
- `PeerKnowledge`
- `ElectionEligibility`
- `IneligibleReason`
- `ApiVisibility`
- `WalPosition`

The flattened replacement should reuse owning-domain types directly instead of copying them into HA:
- `PgInfoState`
- `DcsSnapshot`
- `DcsQuorumState`
- `DcsMemberState`
- `ProcessState`
- `LeaseEpoch`
- `ObservedWalPosition`
- `SwitchoverState`

The target is one flatter HA observation representation owned by HA itself, not a second parallel domain model. If a tiny HA-local fact is truly needed, add that fact once to the flat representation instead of introducing another enum tree.

code:
```rust
// src/ha/types.rs
pub struct WorldView {
    pub local: LocalKnowledge,
    pub global: GlobalKnowledge,
}

pub struct LocalKnowledge {
    pub data_dir: DataDirState,
    pub postgres: PostgresState,
    pub process: ProcessAssessment,
    pub storage: StorageState,
    pub managed_roles_reconciled: bool,
    pub publication: PublicationState,
    pub observation: ObservationState,
}

pub struct GlobalKnowledge {
    pub coordination: CoordinationState,
    pub self_peer: PeerKnowledge,
}
```

```rust
// src/ha/worker.rs
fn observe(ctx: &HaRuntimeCtx, now: crate::state::UnixMillis) -> Result<WorldView, WorkerError> {
    let config = ctx.observed.config.latest();
    let pg = ctx.observed.pg.latest();
    let dcs = ctx.observed.dcs.latest();
    let process = ctx.observed.process.latest();
    ...
    let local = LocalKnowledge {
        data_dir: build_data_dir_state(...),
        postgres: match &pg { ... },
        process: process_assessment,
        storage: build_storage_state(...),
        managed_roles_reconciled: ctx.state_channel.current.managed_roles_reconciled,
        publication: ctx.state_channel.current.publication.clone(),
        observation,
    };
    let global = build_global_knowledge(&dcs, &pg, &local.data_dir, &ctx.identity.member_id);

    Ok(WorldView { local, global })
}
```

```rust
// src/ha/decide.rs
pub(crate) fn decide(world: &WorldView, self_id: &MemberId) -> DesiredState { ... }
```

```rust
// src/ha/reconcile.rs
pub(crate) fn reconcile(world: &WorldView, desired: &DesiredState) -> PlannedActions { ... }
```

---
- [x] Smell 3, wrong place-ism: HA owns process timings and retained identity history that belong to process/pginfo
HA currently owns `pg_observed_at`, `last_start_success_at`, `last_basebackup_success_at`, `last_promote_success_at`, `last_demote_success_at`, `last_local_timeline`, and `last_local_system_identifier`. That means HA is not just consuming process/pginfo state; it is reconstructing and remembering process freshness rules and identity fallback rules on its own.

This violates the user constraint directly: process timings must live in `process`, and HA must trust that process-owned state instead of rebuilding it.

Specific HA-owned timing/history logic that must go completely:
- `ObservationState`
- `last_success_at`
- `last_start_success_at`
- `ObservationState::waiting_for_fresh_pg_after`
- `ObservationState::basebackup_completed_awaiting_start`
- `local_member_identity_fallback`
- `retained_local_identity_fallback`
- all `last_*_success_at` and `last_local_*` fields in tests under `src/ha/decide.rs`, `src/ha/reconcile.rs`, and `src/ha/worker.rs`

Required replacement direction:
- `process` owns start/basebackup/promote/demote freshness and recovery progress
- `pginfo` owns timeline/system-identifier observation facts
- HA consumes one process-owned progress/readiness fact instead of comparing raw timestamps
- HA stops retaining identity history across ticks

Mandatory completion threshold for this smell:
- at least 70% of the timing/history types and fields listed here must disappear entirely from HA
- more removal is better; 70% is only the minimum acceptable outcome
- if that threshold is not reached, the implementer must assume the timing boundary was not actually fixed

code:
```rust
// src/ha/types.rs
pub struct ObservationState {
    pub pg_observed_at: UnixMillis,
    pub last_start_success_at: Option<UnixMillis>,
    pub last_basebackup_success_at: Option<UnixMillis>,
    pub last_promote_success_at: Option<UnixMillis>,
    pub last_demote_success_at: Option<UnixMillis>,
    pub last_local_timeline: Option<u64>,
    pub last_local_system_identifier: Option<u64>,
}
```

```rust
// src/ha/worker.rs
let observation = ObservationState {
    pg_observed_at: pg.last_refresh_at().unwrap_or(now),
    last_start_success_at: last_start_success_at(&process),
    last_basebackup_success_at: last_success_at(&process, ActiveJobKind::BaseBackup),
    last_promote_success_at: last_success_at(&process, ActiveJobKind::Promote),
    last_demote_success_at: last_success_at(&process, ActiveJobKind::Demote),
    last_local_timeline: current_local_timeline.or(previous_observation.last_local_timeline),
    last_local_system_identifier: current_local_system_identifier
        .or(previous_observation.last_local_system_identifier),
};
```

```rust
// src/ha/worker.rs
fn local_member_identity_fallback(
    dcs: &DcsSnapshot,
    self_id: &MemberId,
    observation: &ObservationState,
) -> (Option<u64>, Option<u64>) { ... }

fn retained_local_identity_fallback(observation: &ObservationState) -> (Option<u64>, Option<u64>) { ... }
```

```rust
// src/ha/reconcile.rs
world.local.observation.waiting_for_fresh_pg_after(ActiveJobKind::StartPrimary)
world.local.observation.waiting_for_fresh_pg_after(ActiveJobKind::Promote)
world.local.observation.waiting_for_fresh_pg_after(ActiveJobKind::Demote)
world.local.observation.waiting_for_fresh_pg_after(ActiveJobKind::StartReplica)
```

---
- [x] Smell 1, useless overabstraction and overnesting across decide/reconcile transit types
After the observation mirror tree is removed, keep collapsing. HA still has a second type corridor for decision output and reconciliation input:
- `DesiredState`
- `TargetRole`
- `Candidacy`
- `FollowGoal`
- `RecoveryPlan`
- `FailSafeGoal`
- `IdleReason`
- `FenceReason`
- `PublicationGoal`
- `PublicationState`
- `AuthorityProjection`
- `NoPrimaryProjection`
- `NoPrimaryFence`
- `FenceCutoff`
- `SwitchoverBlocker`
- `PlannedActions`
- `CoordinationAction`
- `LocalAction`
- `PublicationAction`

This is currently nested enum soup rather than one direct decision/reconcile flow. The most obvious bad cases are:
- `PublicationState::Projected(AuthorityProjection::NoPrimary(NoPrimaryProjection::...))`
- `TargetRole::Idle(IdleReason::...)`
- `PlannedActions` holding four separate optional action slots only to be read back sequentially in `step_once`

The task is not allowed to stop after flattening only the input world. It must keep pushing until the decision/result boundary becomes one flatter readable intent representation or direct match-driven flow.

Mandatory completion threshold for this smell:
- at least 70% of the listed transit types must disappear entirely through deletion or merge
- more removal is better; 70% is only the minimum acceptable outcome
- if that threshold is not reached, the implementer must assume the nesting diagnosis was still correct and continue collapsing

code:
```rust
// src/ha/types.rs
pub struct DesiredState {
    pub role: TargetRole,
    pub publication: PublicationGoal,
    pub clear_switchover: bool,
}

pub enum TargetRole {
    Leader(LeaseEpoch),
    Candidate(Candidacy),
    Follower(FollowGoal),
    FailSafe(FailSafeGoal),
    DemotingForSwitchover(MemberId),
    Fenced(FenceReason),
    Idle(IdleReason),
}
```

```rust
// src/ha/types.rs
pub enum PublicationState {
    Unknown,
    Projected(AuthorityProjection),
}

pub enum AuthorityProjection {
    Primary(LeaseEpoch),
    NoPrimary(NoPrimaryProjection),
}
```

```rust
// src/ha/types.rs
pub struct PlannedActions {
    pub publication: Option<PublicationAction>,
    pub coordination: Option<CoordinationAction>,
    pub local: Option<LocalAction>,
    pub process: Option<ProcessIntent>,
}
```

---
- [x] Smell 10, remove the damn helpers in `src/ha/worker.rs` and `src/ha/decide.rs`
`src/ha/decide.rs` and `src/ha/worker.rs` are dominated by private one-file helper chains. Most of these helpers only exist to move small matches away from the owning workflow.

Helper groups in `src/ha/decide.rs` that should be collapsed into one readable decision pipeline:
- `decide_no_quorum`
- `decide_under_foreign_leadership`
- `decide_as_lease_holder`
- `decide_without_lease`
- `leader_publication`
- `follow_goal`
- `rewind_failed_and_requires_basebackup`
- `candidacy_kind`
- `active_epoch`
- `primary_publication`
- `no_primary_publication`
- `lease_open_publication`
- `publication_epoch`
- `no_quorum_fence`
- `resolve_switchover`
- `best_switchover_target`
- `best_failover_candidate`
- `switchover_target_is_valid`
- `compare_candidate_eligibility`

Helper groups in `src/ha/worker.rs` that should be collapsed into one readable observation pipeline:
- `map_process_dispatch_error`
- `build_next_state`
- `next_managed_roles_reconciled`
- `apply_publication_goal`
- `build_data_dir_state`
- `build_storage_state`
- `build_global_knowledge`
- `build_peer_knowledge_from_member`
- `build_self_peer`
- `self_replica_position`
- `observed_primary_member`
- `member_timeline`
- `member_system_identifier`
- `pg_timeline`
- `pg_system_identifier`
- `pg_timeline_id`

Hard reduction target for this smell set:
- at least 90% of the listed helper functions in `src/ha/decide.rs` must disappear entirely through inlining, merge, or collapse into one owning flow
- at least 90% of the listed helper functions in `src/ha/worker.rs` must disappear entirely through inlining, merge, or collapse into one owning flow
- more removal is better; 90% is only the minimum acceptable outcome
- if that threshold is not reached, the implementer must assume the smell report was still under-scoped and continue reducing functions instead of keeping them

code:
```rust
// src/ha/decide.rs
pub(crate) fn decide(world: &WorldView, self_id: &MemberId) -> DesiredState {
    let Some(coordination) = world.global.coordination.as_quorum() else {
        return decide_no_quorum(world);
    };
    ...
    match &coordination.leadership {
        LeadershipView::HeldBySelf(epoch) => decide_as_lease_holder(world, self_id, epoch.clone()),
        LeadershipView::HeldByPeer { epoch, state } => {
            decide_under_foreign_leadership(world, epoch.clone(), state)
        }
        LeadershipView::Open => decide_without_lease(world, coordination, self_id),
    }
}
```

```rust
// src/ha/worker.rs
fn observe(ctx: &HaRuntimeCtx, now: crate::state::UnixMillis) -> Result<WorldView, WorkerError> {
    ...
    let local = LocalKnowledge {
        data_dir: build_data_dir_state(...),
        postgres: match &pg { ... },
        process: process_assessment,
        storage: build_storage_state(...),
        ...
    };
    let global = build_global_knowledge(&dcs, &pg, &local.data_dir, &ctx.identity.member_id);
    Ok(WorldView { local, global })
}
```

<plan>
- [x] Replace the current `WorldView` mirror tree in [`src/ha/types.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/types.rs) with one flat HA-owned observation shape that embeds owning-domain state directly. The new observation should hold `PgInfoState`, `ProcessState`, `DcsSnapshot`, the current `PublicationState`, `managed_roles_reconciled`, and only the irreducible HA-derived facts that are not already modeled elsewhere: local data-dir classification, self candidate status, observed ready primary identity, and storage stall status. Delete or merge the current HA-local mirror shapes rather than translating them forward.
- [x] Remove HA-owned timing and retained identity history from the observation boundary. `ObservationState`, `last_success_at`, `last_start_success_at`, `local_member_identity_fallback`, `retained_local_identity_fallback`, and the `last_local_*` fallback state must disappear from HA. Before repairing all compile fallout, add the missing process-owned readiness/progress API in [`src/process/state.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/process/state.rs) and consume `PgInfoState` identity fields directly so HA stops rebuilding freshness and timeline/system-identifier history on its own.
- [x] Flatten the observation pipeline inside [`src/ha/worker.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/worker.rs) around one owning `observe(...)` flow. Inline or merge the helper chain so the worker computes the flat observation in one readable pass from `config`, `pg`, `dcs`, and `process`, reusing `DcsSnapshot`/`DcsQuorumState`/`DcsMemberState` and `ObservedWalPosition` directly instead of building `GlobalKnowledge`, `PeerKnowledge`, `LeadershipView`, `PrimaryObservation`, `ObservedPrimary`, or `WalPosition`.
- [x] Collapse the decision boundary in [`src/ha/decide.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/decide.rs) from nested transit ADTs into one flatter intent shape. Replace `DesiredState`, `TargetRole`, `Candidacy`, `FollowGoal`, `RecoveryPlan`, `FailSafeGoal`, `IdleReason`, `FenceReason`, and the publication projection nesting with one direct HA intent enum or similarly flat match-driven output that carries only the data reconcile actually needs. While doing this, inline the current one-caller helper chain so only real ranking or ordering logic survives.
- [x] Collapse reconciliation in [`src/ha/reconcile.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/reconcile.rs) to consume the flatter intent directly and emit one ordered action list or one direct step flow instead of `PlannedActions` plus separate `CoordinationAction`/`LocalAction`/`PublicationAction` corridors. The final `step_once` path in [`src/ha/worker.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/worker.rs) should read as observe -> decide -> reconcile -> execute without rewrapping the same facts into another nested enum tree.
- [x] Rewrite HA tests after the type collapse so fixtures stop constructing the deleted mirror types and timestamp-history fields. Tests in [`src/ha/decide.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/decide.rs), [`src/ha/reconcile.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/reconcile.rs), and [`src/ha/worker.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/worker.rs) should assert against the new flat observation and intent/action boundaries, not rebuild deleted helper-only state.
- [x] After the type collapse compiles, run the required validation gates in repo order: `make check`, `make lint`, `make test`, and `make test-long`. If execution shows the flat observation or intent boundary is still wrong, switch this task back to `TO BE VERIFIED`, explain the exact type leak that remains in this file, and stop immediately.
NOW EXECUTE
</plan>

### Execution notes
1. Start by changing types, not by editing helper bodies in place. The current HA problem is the duplicated ADT corridor, so the observation and intent shapes must be replaced before compile-fallout cleanup.
2. Reuse existing domain facts before creating any new HA enum or struct. `PgInfoState`, `ProcessState`, `DcsSnapshot`, `DcsQuorumState`, `DcsMemberState`, `LeaseEpoch`, `ObservedWalPosition`, and `SwitchoverState` are the preferred source of truth.
3. Keep HA-local additions minimal and flat. If one HA-only fact is still needed, add it once to the flat observation or flat intent instead of introducing another nested subtree.
4. The helper-reduction thresholds in this smell file are real completion criteria. If a helper still exists after the rewrite, it should have a clear multi-caller invariant, not just a convenience wrapper.
