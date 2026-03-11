
## Task: Refactor The HA Loop <status>not_started</status> <passes>false</passes>

<priority>high</priority>


Refactor the 'ha-loop'

design based on best:
- best production safety, quality
- long term stability
- code reduction (least code in the end)
- best use of rust's type system guarantees to give this production safety and quality


Artifact of a design-only refactor study for the HA loop and all surrounding startup/reconciliation logic.
The user wants a full redesign plan for the HA loop because the current architecture drifted away from the intended shape. 
The user believes the implementation is now too spread out and messy, startup logic is disconnected from the decide loop, sender-side dedup slipped into the HA worker, and the current quorum/failsafe boundary is wrong.


This is the redesigned/refined/renewed/refactored ha system on high level, what rust types it will have, how it iteracts with dcs (etcd), how it makes decisions based on state.

There is really thought about that nodes themselves, only know about themselves, and what they put on dcs from others.

I had to be elegant, long lasting, make great use of the rust's typesystem guarantees, and in general just 'good'

Here is the full designed PLAN:

### 1. `types.rs` (The Epistemology)
*This file contains zero logic. It strictly defines the shapes of knowledge and intent. By avoiding wildcards (`_`) in `match` statements later, the compiler guarantees we never miss a state combination.*

```rust
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemberId(pub String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeaseToken(pub String);

// --- WHAT IS TRUE (EPISTEMOLOGY) ---

pub struct WorldView {
    pub local: LocalKnowledge,
    pub global: GlobalKnowledge,
}

pub struct LocalKnowledge {
    pub data_dir: DataDirState,
    pub pg_status: PgStatus,
    /// If Some(_), a process (Start, Demote, BaseBackup, etc.) is currently running.
    pub active_job: Option<JobKind>, 
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DataDirState {
    MissingOrEmpty,
    Initialized,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SyncState {
    InSync,
    RequiresRewind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PgStatus {
    Stopped,
    RunningPrimary,
    RunningReplica { upstream: MemberId, sync: SyncState },
}

pub struct GlobalKnowledge {
    pub dcs_trust: DcsTrust,
    pub lease: LeaseState,
    pub switchover_intent: Option<MemberId>,
    pub valid_peers: BTreeMap<MemberId, PeerData>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DcsTrust {
    /// DCS is healthy, we have visibility into the quorum majority.
    FullQuorum,
    /// DCS partitioned or we are in the minority (e.g., 1/3 nodes).
    Degraded,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LeaseState {
    HeldByMe(LeaseToken),
    HeldBy(MemberId),
    Unheld,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeerData {
    pub is_ready: bool,
    pub wal_lsn: u64,
    pub timeline: u64,
}

// --- WHAT I SHOULD BE (POLICY/GOAL) ---

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TargetRole {
    /// Actively serving writes. Must hold the LeaseToken.
    Leader(LeaseToken),
    /// Actively attempting to acquire the Lease.
    Candidate,
    /// Replicating from a known Leader.
    Follower(MemberId),
    /// DCS is healthy, but lease is unheld and we are NOT the best candidate. Wait.
    WaitingForLeader,
    /// DCS is Degraded. Replica continues following last known upstream safely.
    FailSafe(Option<MemberId>),
    /// Emergency stop. Never serve writes.
    Fenced(FenceReason),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FenceReason {
    DcsPartitioned,
    DemotedForSwitchover,
    ForeignLeaderDetected,
}

// --- HOW I GET THERE (ACTION) ---

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReconcileAction {
    InitDb,
    BaseBackup(MemberId),
    PgRewind(MemberId),
    StartPrimary,
    StartReplica(MemberId),
    Promote,
    Demote(ShutdownMode),
    AcquireLease,
    ReleaseLease,
    ClearSwitchover,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShutdownMode {
    Fast,      // Graceful (Planned Switchover)
    Immediate, // Crash-stop (Fencing / Split-brain prevention)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JobKind {
    InitDb, BaseBackup, PgRewind, Start, Promote, Demote
}
```

---

### 2. `decide.rs` (The Pure Policy Engine)
*This module maps `WorldView` to `TargetRole`. It has no side effects. It contains the logic for elections, failovers, and switchovers.*

```rust
use crate::types::*;
use std::collections::BTreeMap;

pub fn decide(world: &WorldView, self_id: &MemberId) -> TargetRole {
    // 1. SAFETY BOUNDARY: Quorum Loss
    // If we are in the minority (1/3), we lose DcsTrust.
    if world.global.dcs_trust == DcsTrust::Degraded {
        return match &world.local.pg_status {
            // A primary in the minority MUST fence itself. 
            // The 2/3 majority will elect a new leader.
            PgStatus::RunningPrimary => TargetRole::Fenced(FenceReason::DcsPartitioned),
            
            // Replicas in the minority stay up and keep trying to stream (FailSafe).
            PgStatus::RunningReplica { upstream, .. } => TargetRole::FailSafe(Some(upstream.clone())),
            PgStatus::Stopped => TargetRole::FailSafe(None),
        };
    }

    // 2. NORMAL OPERATION: We have DCS Quorum (2/3 or 3/3).
    match &world.global.lease {
        LeaseState::HeldByMe(token) => {
            // Check for Planned Switchover
            if let Some(target_id) = &world.global.switchover_intent {
                if target_id != self_id {
                    return TargetRole::Fenced(FenceReason::DemotedForSwitchover);
                }
            }
            TargetRole::Leader(token.clone())
        }
        
        LeaseState::HeldBy(leader_id) => {
            // Split-brain protection: Someone else owns the lease, but I am writing!
            if world.local.pg_status == PgStatus::RunningPrimary {
                return TargetRole::Fenced(FenceReason::ForeignLeaderDetected);
            }
            TargetRole::Follower(leader_id.clone())
        }
        
        LeaseState::Unheld => {
            // The lease is open. Do we run for election?
            let is_targeted = world.global.switchover_intent.as_ref() == Some(self_id);
            let best_candidate = find_best_candidate(&world.global.valid_peers);

            let am_i_best = best_candidate.as_ref() == Some(self_id);
            let am_i_already_primary = world.local.pg_status == PgStatus::RunningPrimary;

            if is_targeted || am_i_best || am_i_already_primary {
                TargetRole::Candidate
            } else {
                TargetRole::WaitingForLeader
            }
        }
    }
}

/// Pure function to evaluate the healthiest replica for promotion.
fn find_best_candidate(peers: &BTreeMap<MemberId, PeerData>) -> Option<MemberId> {
    peers.iter()
        .filter(|(_, data)| data.is_ready)
        .max_by(|(_, a), (_, b)| {
            // Primary sort by Timeline, secondary sort by LSN
            a.timeline.cmp(&b.timeline)
                .then(a.wal_lsn.cmp(&b.wal_lsn))
        })
        .map(|(id, _)| id.clone())
}
```

---

### 3. `reconcile.rs` (The State Machine)
*This function diffs `LocalKnowledge` against `TargetRole`. The compiler forces us to handle all 6 `TargetRole` variants against all 3 `PgStatus` variants.*

```rust
use crate::types::*;

pub fn reconcile(local: &LocalKnowledge, target: &TargetRole, world: &WorldView) -> Option<ReconcileAction> {
    // IDEMPOTENCY LOCK: Do nothing if a process (BaseBackup, Start, etc.) is running.
    if local.active_job.is_some() {
        return None;
    }

    match target {
        TargetRole::Leader(_) => {
            // Clean up switchover intent once we successfully became leader
            if world.global.switchover_intent.is_some() {
                return Some(ReconcileAction::ClearSwitchover);
            }

            match local.data_dir {
                DataDirState::MissingOrEmpty => Some(ReconcileAction::InitDb),
                DataDirState::Initialized => match &local.pg_status {
                    PgStatus::Stopped => Some(ReconcileAction::StartPrimary),
                    PgStatus::RunningReplica { .. } => Some(ReconcileAction::Promote),
                    PgStatus::RunningPrimary => None, // GOAL REACHED
                }
            }
        }
        
        TargetRole::Follower(leader_id) => match local.data_dir {
            DataDirState::MissingOrEmpty => Some(ReconcileAction::BaseBackup(leader_id.clone())),
            DataDirState::Initialized => match &local.pg_status {
                PgStatus::Stopped => Some(ReconcileAction::StartReplica(leader_id.clone())),
                PgStatus::RunningPrimary => Some(ReconcileAction::Demote(ShutdownMode::Fast)),
                PgStatus::RunningReplica { upstream, sync } => {
                    if *sync == SyncState::RequiresRewind {
                        Some(ReconcileAction::PgRewind(leader_id.clone()))
                    } else if upstream != leader_id {
                        // Pointing to the wrong upstream -> restart
                        Some(ReconcileAction::Demote(ShutdownMode::Fast))
                    } else {
                        None // GOAL REACHED
                    }
                }
            }
        },

        TargetRole::Candidate => {
            Some(ReconcileAction::AcquireLease)
        }

        TargetRole::WaitingForLeader => match &local.pg_status {
            PgStatus::RunningPrimary => Some(ReconcileAction::Demote(ShutdownMode::Fast)),
            // If stopped or replica, we just wait. We can't start a replica without knowing the leader.
            PgStatus::Stopped | PgStatus::RunningReplica { .. } => None, 
        },

        TargetRole::FailSafe(_) => match &local.pg_status {
            // FailSafe means we protect ourselves. Primary must NEVER be in FailSafe, 
            // `decide` prevents this, but defensively we crash-stop if it happens.
            PgStatus::RunningPrimary => Some(ReconcileAction::Demote(ShutdownMode::Immediate)),
            PgStatus::RunningReplica { .. } | PgStatus::Stopped => None, // Safe. Keep doing what we are doing.
        },

        TargetRole::Fenced(reason) => match &local.pg_status {
            PgStatus::RunningPrimary | PgStatus::RunningReplica { .. } => {
                let mode = match reason {
                    FenceReason::DemotedForSwitchover => ShutdownMode::Fast, // Planned maintenance
                    FenceReason::DcsPartitioned | FenceReason::ForeignLeaderDetected => ShutdownMode::Immediate,
                };
                Some(ReconcileAction::Demote(mode))
            }
            PgStatus::Stopped => {
                // IMPORTANT: Switchover Release Logic
                // If we stopped gracefully because of a switchover, we MUST release the lease 
                // so the new target can grab it.
                if *reason == FenceReason::DemotedForSwitchover && 
                   matches!(world.global.lease, LeaseState::HeldByMe(_)) {
                    Some(ReconcileAction::ReleaseLease)
                } else {
                    None // SAFELY FENCED
                }
            }
        }
    }
}
```

---

### 4. `worker.rs` (The IO Boundary)
*This is the orchestrator. It executes exactly three steps: Gather State, Decide/Reconcile, Execute Action. It is vastly smaller than the original codebase because all "if/then" logic was pushed into the pure functional layers.*

```rust
use crate::types::*;
use crate::decide::decide;
use crate::reconcile::reconcile;
use crate::state::WorkerError;
use crate::ha::state::HaWorkerCtx;

pub(crate) async fn step_once(ctx: &mut HaWorkerCtx) -> Result<(), WorkerError> {
    // 1. OBSERVE (Map dirty system state into clean Epistemological types)
    let world = WorldView {
        local: build_local_knowledge(ctx),
        global: build_global_knowledge(ctx),
    };

    // 2. DECIDE (What should I be?)
    let target_role = decide(&world, &ctx.self_id);

    // Logging state transitions prevents log spam and gives amazing debug trails
    if ctx.state.target_role != target_role {
        emit_role_transition_log(&ctx.state.target_role, &target_role);
        ctx.state.target_role = target_role.clone();
    }

    // 3. RECONCILE (Diff reality against goal)
    let next_action = reconcile(&world.local, &target_role, &world);

    // 4. ACT (Execute the singular step required to reach the goal)
    if let Some(action) = next_action {
        emit_action_intent_log(&action);

        match action {
            // DCS Execution
            ReconcileAction::AcquireLease => {
                ctx.dcs_store.acquire_leader_lease(&ctx.scope, &ctx.self_id)?;
            }
            ReconcileAction::ReleaseLease => {
                ctx.dcs_store.release_leader_lease(&ctx.scope, &ctx.self_id)?;
            }
            ReconcileAction::ClearSwitchover => {
                ctx.dcs_store.clear_switchover(&ctx.scope)?;
            }

            // Process Execution (Mapped directly to jobs)
            process_action => {
                let job_request = build_process_job_request(process_action, ctx);
                ctx.process_inbox.send(job_request).map_err(|e| {
                    WorkerError::Message(format!("Failed to dispatch process job: {e}"))
                })?;
            }
        }
    }

    Ok(())
}

// --- Adapters to map from current pginfo/dcs state to the clean types ---

fn build_local_knowledge(ctx: &HaWorkerCtx) -> LocalKnowledge {
    let pg_state = ctx.pg_subscriber.latest();
    let process_state = ctx.process_subscriber.latest();

    let data_dir = if ctx.process_defaults.data_dir_exists_and_has_pg_version() {
        DataDirState::Initialized
    } else {
        DataDirState::MissingOrEmpty
    };

    let pg_status = match &pg_state.value {
        crate::pginfo::state::PgInfoState::Primary { .. } => PgStatus::RunningPrimary,
        crate::pginfo::state::PgInfoState::Replica { upstream, follow_lsn, .. } => {
            // Map raw PG state to strict enum
            PgStatus::RunningReplica {
                upstream: upstream.as_ref().map(|u| MemberId(u.member_id.0.clone())).unwrap_or(MemberId("".into())),
                sync: if follow_lsn.is_none() { SyncState::RequiresRewind } else { SyncState::InSync },
            }
        },
        _ => PgStatus::Stopped,
    };

    let active_job = process_state.value.running_job_kind().map(|k| match k {
        crate::process::jobs::ActiveJobKind::BaseBackup => JobKind::BaseBackup,
        crate::process::jobs::ActiveJobKind::StartPostgres => JobKind::Start,
        crate::process::jobs::ActiveJobKind::Promote => JobKind::Promote,
        crate::process::jobs::ActiveJobKind::Demote => JobKind::Demote,
        crate::process::jobs::ActiveJobKind::PgRewind => JobKind::PgRewind,
        crate::process::jobs::ActiveJobKind::Bootstrap => JobKind::InitDb,
        _ => JobKind::Start, // fallback
    });

    LocalKnowledge { data_dir, pg_status, active_job }
}

fn build_global_knowledge(ctx: &HaWorkerCtx) -> GlobalKnowledge {
    let dcs = ctx.dcs_subscriber.latest();

    let dcs_trust = match dcs.value.trust {
        crate::dcs::state::DcsTrust::FullQuorum => DcsTrust::FullQuorum,
        _ => DcsTrust::Degraded,
    };

    let lease = match &dcs.value.cache.leader {
        Some(record) if record.member_id.0 == ctx.self_id.0 => {
            LeaseState::HeldByMe(LeaseToken(record.member_id.0.clone()))
        }
        Some(record) => LeaseState::HeldBy(MemberId(record.member_id.0.clone())),
        None => LeaseState::Unheld,
    };

    let switchover_intent = dcs.value.cache.switchover.as_ref()
        .and_then(|req| req.switchover_to.as_ref())
        .map(|m| MemberId(m.0.clone()));

    let mut valid_peers = BTreeMap::new();
    for (id, record) in &dcs.value.cache.members {
        valid_peers.insert(MemberId(id.0.clone()), PeerData {
            is_ready: record.readiness == crate::pginfo::state::Readiness::Ready,
            wal_lsn: record.write_lsn.map(|l| l.0).unwrap_or(0),
            timeline: record.timeline.unwrap_or(0),
        });
    }

    GlobalKnowledge { dcs_trust, lease, switchover_intent, valid_peers }
}
```

### Why this is mathematically beautiful:
1. **The "Start Up" phase vanishes.** If you turn on three empty nodes, `Local: Missing`, `Lease: Unheld`. All three evaluate `Candidate` (or `WaitingForLeader`). One wins the lease, decides `Leader`, and reconciles to `InitDb`. The other two decide `Follower`, and reconcile to `BaseBackup`. No explicit orchestration needed!
2. **Switchover gracefully flows.** The Leader drops to `Fenced`, gracefully `Demotes`, then gracefully `Releases` the lease. The Target grabs the lease, `Starts` as primary (or `Promotes`), and `Clears` the intent.
3. **Compiler guarantees safety.** If you add a new `PgStatus` (e.g., `RunningDegraded`), Rust will throw a compile error in `reconcile.rs` until you explicitly decide how a `Leader`, `Follower`, and `Fenced` node should react to it.