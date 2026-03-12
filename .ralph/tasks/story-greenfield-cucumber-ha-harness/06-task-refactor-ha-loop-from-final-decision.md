
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

This keeps the same spirit as the first pass:
- `observe -> decide -> reconcile -> act` remains the whole loop
- `decide.rs` is still pure
- `reconcile.rs` is still pure
- the worker is still thin
- startup, failover, switchover, rejoin, and quorum-restore are still one unified state machine

The core fix is this: the design must not only choose a local role. It must also choose the local operator-facing authority projection and the recovery path. The feature suite proved those are first-class state, not side effects we can hand-wave.

Important clarification: "publish" here does **not** mean "tell peer nodes what to do" or "replicate a cluster replica list into the control plane". Every node still runs the loop independently. Peer nodes react only to the real coordination primitives: leader lease, switchover request, and per-member DCS records. Publication is only this node updating its own operator-facing authority projection.


<po_note>
# IMPORTANT! Drop your current work IMMEDIATELY and fix these before doing anything else!

There have been concerning reports you have been overcomplicating the design and following the underlying design.
Please fix the source code first to follow the design VERY closely, before continueing to hotpatch the code:

- [ ] Original bootstrap code was NOT removed! This is very bad, as the goal was to 'unify' the logic. 

To Remove for bootstrap:
- startup is still separate in `src/runtime/node.rs`. There is still a distinct startup planner with `StartupMode::InitializePrimary`, `StartupMode::CloneReplica`, and `StartupMode::ResumeExisting`.
- that means the task-6 promise that “startup, failover, switchover, rejoin, and quorum-restore are one unified state machine” is still false in the current tree.
- startup still does its own DCS probe, start-intent rebuilding, and action planning before the HA worker starts. (NO MORE DCS OUTSIDE THE MAIN WAY!)


- [ ] Simplify Dcs member state

  Make member in dcs simpler:
- DCS/member modelling is still more operational and less elegant than the task-6 epistemology wanted: member records are still timestamped JSON records with freshness heuristics, not a cleaner authority model with stronger physical/offline facts carrying the whole startup/rejoin story.
- What i want is all that junk removed, all that logic removed, and instead replaced by elegant (lease based + always exactly fresh what pginfo workers/whatever said) member key
  - dcs exposes simple set own member endpoint towards pginfo (or where pg data is obtained)
  - only that one always writes the full current state of pg: be it is it ready, pginfo (or none if none could be queried), etc,etc: all of it MUST BE 100% fresh, no lies, no caching, never old data, no exceptions. the member's key purpose IS having the latest valid true data of a member. Member keys Must be made impossible to be written by other member, using the typesystem of rust guarantees
  - Remove updated_at crap, and other shit like that, and FULLY replace with member/{member_id} based lease. This lease info MUST BE only in the dcs worker, AND NOT anywhere poluted elsewhere.

to quote
'''
Why the code got complicated
- leader liveness is authoritative through the lease-backed `/leader` key,
- but member freshness is only inferred from periodic member writes and watch-fed cache state,
- so the two can drift apart under watch/snapshot timing, stale cache state, reconnect behavior, or partial observation.

The current cleanup in `src/dcs/worker.rs` is therefore a defensive invariant repair, not proof that the model is intrinsically simple.
'''


- [ ] Api output and debug api output is somehow messy/cached/stale/late and that must go

- Api must just like the ha loop, also obtain all FRESH data and logic, and NEVER any stale data (none is much better than old)
- This is easily possible from other workers, IT MUST reuse the same data collection logic. It is encouraged to alter the api schema to be closer to the real underlying data
- Instead of 'publishing' new ha state, the api stuff should 'listen' to the new api state, in the same way ha loop here listens to pginfo and dcs

- [ ] DEEP INVESTIGATION must be done to find all current ways 'stale' data is entering the system: I just don't get it, how is this possible? why is  any stale data entering at all? why is old shit copied to new if known to not be always true?
- verify deeply

- [ ] DEEP INVESTIGATION must be done on .ralph/progress/189.jsonl, to find any other weird shit behaviour just like i found in the code.
- read logs
- decide on hacky behaviour that was added, but really lost the spirit of this task
- verify deeply


## Found hacky behaviour
- (list them here)


Even though many things have been done already, I put the task back into 'TO BE VERIFIED', to make sure CDD/ADT based programming is followed.
Please remove this and the above line, after setting NOW EXECUTE


</po_note>

### 1. `types.rs` (The Epistemology)
*This file still contains zero logic. It now models the missing things explicitly: operator-facing authority projection, switchover eligibility, recovery fallback, and fencing cutoffs.*

```rust
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemberId(pub String);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct WalPosition {
    pub timeline: u64,
    pub lsn: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeaseEpoch {
    pub holder: MemberId,
    pub generation: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FenceCutoff {
    pub epoch: LeaseEpoch,
    pub committed_lsn: u64,
}

// --- WHAT IS TRUE (EPISTEMOLOGY) ---

pub struct WorldView {
    pub local: LocalKnowledge,
    pub global: GlobalKnowledge,
}

pub struct LocalKnowledge {
    pub data_dir: DataDirState,
    pub postgres: PostgresState,
    pub process: ProcessState,
    pub storage: StorageState,
    pub publication: PublicationState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DataDirState {
    Missing,
    Initialized(LocalDataState),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LocalDataState {
    BootstrapEmpty,
    ConsistentReplica,
    Diverged(DivergenceState),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DivergenceState {
    RewindPossible,
    BasebackupRequired,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PostgresState {
    Offline,
    Primary { committed_lsn: u64 },
    Replica {
        upstream: Option<MemberId>,
        replication: ReplicationState,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReplicationState {
    Streaming(WalPosition),
    CatchingUp(WalPosition),
    Stalled,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProcessState {
    Idle,
    Running(JobKind),
    Failed(JobFailure),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JobFailure {
    pub job: JobKind,
    pub recovery: FailureRecovery,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FailureRecovery {
    RetrySameJob,
    FallbackToBasebackup,
    WaitForOperator,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StorageState {
    Healthy,
    Stalled,
}

pub struct PublicationState {
    pub authority: AuthorityView,
    pub fence_cutoff: Option<FenceCutoff>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuthorityView {
    Primary {
        member: MemberId,
        epoch: LeaseEpoch,
    },
    NoPrimary(NoPrimaryReason),
    Unknown,
}

pub struct GlobalKnowledge {
    pub dcs_trust: DcsTrust,
    pub lease: LeaseState,
    pub switchover: SwitchoverState,
    pub peers: BTreeMap<MemberId, PeerKnowledge>,
    pub self_peer: PeerKnowledge,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DcsTrust {
    FullQuorum,
    Degraded,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LeaseState {
    HeldByMe(LeaseEpoch),
    HeldByPeer(LeaseEpoch),
    Unheld,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SwitchoverState {
    None,
    Requested(SwitchoverRequest),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SwitchoverRequest {
    pub target: SwitchoverTarget,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SwitchoverTarget {
    AnyHealthyReplica,
    Specific(MemberId),
}

pub struct PeerKnowledge {
    pub election: ElectionEligibility,
    pub api: ApiVisibility,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ElectionEligibility {
    BootstrapEligible,
    PromoteEligible(WalPosition),
    Ineligible(IneligibleReason),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IneligibleReason {
    NotReady,
    Lagging,
    Partitioned,
    ApiUnavailable,
    StartingUp,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ApiVisibility {
    Reachable,
    Unreachable,
}

// --- WHAT I SHOULD BE (POLICY OUTPUT) ---

pub struct DesiredState {
    pub role: TargetRole,
    pub publication: PublicationGoal,
    pub clear_switchover: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TargetRole {
    Leader(LeaseEpoch),
    Candidate(Candidacy),
    Follower(FollowGoal),
    FailSafe(FailSafeGoal),
    DemotingForSwitchover(MemberId),
    Fenced(FenceReason),
    Idle(IdleReason),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Candidacy {
    Bootstrap,
    Failover,
    ResumeAfterOutage,
    TargetedSwitchover(MemberId),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FollowGoal {
    pub leader: MemberId,
    pub recovery: RecoveryPlan,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecoveryPlan {
    None,
    StartStreaming,
    Rewind,
    Basebackup,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FailSafeGoal {
    PrimaryMustStop(FenceCutoff),
    ReplicaKeepFollowing(Option<MemberId>),
    WaitForQuorum,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IdleReason {
    AwaitingLeader,
    AwaitingTarget(MemberId),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FenceReason {
    ForeignLeaderDetected,
    StorageStalled,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PublicationGoal {
    KeepCurrent,
    PublishPrimary {
        primary: MemberId,
        epoch: LeaseEpoch,
    },
    PublishNoPrimary {
        reason: NoPrimaryReason,
        fence_cutoff: Option<FenceCutoff>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NoPrimaryReason {
    DcsDegraded,
    LeaseOpen,
    Recovering,
    SwitchoverRejected(SwitchoverBlocker),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SwitchoverBlocker {
    TargetMissing,
    TargetIneligible(IneligibleReason),
}

// --- HOW I GET THERE (ACTION PLAN) ---

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReconcileAction {
    InitDb,
    BaseBackup(MemberId),
    PgRewind(MemberId),
    StartPrimary,
    StartReplica(MemberId),
    Promote,
    Demote(ShutdownMode),
    AcquireLease(Candidacy),
    ReleaseLease,
    Publish(PublicationGoal),
    ClearSwitchover,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShutdownMode {
    Fast,
    Immediate,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JobKind {
    InitDb,
    BaseBackup,
    PgRewind,
    StartPrimary,
    StartReplica,
    Promote,
    Demote,
}
```

---

### 2. `decide.rs` (The Pure Policy Engine)
*This module still has no side effects. The important change is that it returns a full `DesiredState`, not just a role. That is what fixes operator-facing authority projection, switchover exclusivity, fencing-cutoff correctness, and auto-clearing stale switchover intent.*

```rust
use crate::types::*;
use std::collections::BTreeMap;

pub fn decide(world: &WorldView, self_id: &MemberId) -> DesiredState {
    if world.global.dcs_trust == DcsTrust::Degraded {
        return decide_degraded(world);
    }

    if world.local.storage == StorageState::Stalled {
        if let PostgresState::Primary { committed_lsn } = &world.local.postgres {
            if let LeaseState::HeldByMe(epoch) = &world.global.lease {
                let cutoff = FenceCutoff {
                    epoch: epoch.clone(),
                    committed_lsn: *committed_lsn,
                };

                return DesiredState {
                    role: TargetRole::Fenced(FenceReason::StorageStalled),
                    publication: PublicationGoal::PublishNoPrimary {
                        reason: NoPrimaryReason::Recovering,
                        fence_cutoff: Some(cutoff),
                    },
                    clear_switchover: false,
                };
            }
        }
    }

    match &world.global.lease {
        LeaseState::HeldByMe(epoch) => decide_as_lease_holder(world, self_id, epoch.clone()),
        LeaseState::HeldByPeer(epoch) => {
            let publication = PublicationGoal::PublishPrimary {
                primary: epoch.holder.clone(),
                epoch: epoch.clone(),
            };

            match &world.local.postgres {
                PostgresState::Primary { .. } => DesiredState {
                    role: TargetRole::Fenced(FenceReason::ForeignLeaderDetected),
                    publication,
                    clear_switchover: false,
                },
                PostgresState::Offline | PostgresState::Replica { .. } => DesiredState {
                    role: TargetRole::Follower(follow_goal(world, epoch.holder.clone())),
                    publication,
                    clear_switchover: false,
                },
            }
        }
        LeaseState::Unheld => decide_without_lease(world, self_id),
    }
}

fn decide_degraded(world: &WorldView) -> DesiredState {
    match &world.local.postgres {
        PostgresState::Primary { committed_lsn } => match &world.global.lease {
            LeaseState::HeldByMe(epoch) | LeaseState::HeldByPeer(epoch) => DesiredState {
                role: TargetRole::FailSafe(FailSafeGoal::PrimaryMustStop(FenceCutoff {
                    epoch: epoch.clone(),
                    committed_lsn: *committed_lsn,
                })),
                publication: PublicationGoal::PublishNoPrimary {
                    reason: NoPrimaryReason::DcsDegraded,
                    fence_cutoff: Some(FenceCutoff {
                        epoch: epoch.clone(),
                        committed_lsn: *committed_lsn,
                    }),
                },
                clear_switchover: false,
            },
            LeaseState::Unheld => DesiredState {
                role: TargetRole::FailSafe(FailSafeGoal::WaitForQuorum),
                publication: PublicationGoal::PublishNoPrimary {
                    reason: NoPrimaryReason::DcsDegraded,
                    fence_cutoff: None,
                },
                clear_switchover: false,
            },
        },
        PostgresState::Replica { upstream, .. } => DesiredState {
            role: TargetRole::FailSafe(FailSafeGoal::ReplicaKeepFollowing(upstream.clone())),
            publication: PublicationGoal::PublishNoPrimary {
                reason: NoPrimaryReason::DcsDegraded,
                fence_cutoff: None,
            },
            clear_switchover: false,
        },
        PostgresState::Offline => DesiredState {
            role: TargetRole::FailSafe(FailSafeGoal::WaitForQuorum),
            publication: PublicationGoal::PublishNoPrimary {
                reason: NoPrimaryReason::DcsDegraded,
                fence_cutoff: None,
            },
            clear_switchover: false,
        },
    }
}

fn decide_as_lease_holder(
    world: &WorldView,
    self_id: &MemberId,
    epoch: LeaseEpoch,
) -> DesiredState {
    let publication = leader_publication(world, self_id, &epoch);

    match resolve_switchover(world, self_id) {
        ResolvedSwitchover::NotRequested => DesiredState {
            role: TargetRole::Leader(epoch.clone()),
            publication: publication.clone(),
            clear_switchover: false,
        },
        ResolvedSwitchover::Proceed(target) if target == *self_id => DesiredState {
            role: TargetRole::Leader(epoch.clone()),
            publication: publication.clone(),
            clear_switchover: true,
        },
        ResolvedSwitchover::Proceed(target) => DesiredState {
            role: TargetRole::DemotingForSwitchover(target),
            publication: PublicationGoal::KeepCurrent,
            clear_switchover: false,
        },
        ResolvedSwitchover::Abandon => DesiredState {
            role: TargetRole::Leader(epoch.clone()),
            publication,
            clear_switchover: true,
        },
    }
}

fn decide_without_lease(
    world: &WorldView,
    self_id: &MemberId,
) -> DesiredState {
    match resolve_switchover(world, self_id) {
        ResolvedSwitchover::Proceed(target) if target == *self_id => DesiredState {
            role: TargetRole::Candidate(Candidacy::TargetedSwitchover(target)),
            publication: PublicationGoal::PublishNoPrimary {
                reason: NoPrimaryReason::LeaseOpen,
                fence_cutoff: None,
            },
            clear_switchover: false,
        },
        ResolvedSwitchover::Proceed(target) => DesiredState {
            role: TargetRole::Idle(IdleReason::AwaitingTarget(target)),
            publication: PublicationGoal::PublishNoPrimary {
                reason: NoPrimaryReason::LeaseOpen,
                fence_cutoff: None,
            },
            clear_switchover: false,
        },
        ResolvedSwitchover::Abandon => DesiredState {
            role: TargetRole::Idle(IdleReason::AwaitingLeader),
            publication: PublicationGoal::PublishNoPrimary {
                reason: NoPrimaryReason::LeaseOpen,
                fence_cutoff: None,
            },
            clear_switchover: true,
        },
        ResolvedSwitchover::NotRequested => match find_best_candidate(
            &world.global.peers,
            &world.global.self_peer,
            self_id,
        ) {
            Some(best) if best == *self_id => DesiredState {
                role: TargetRole::Candidate(candidacy_kind(world)),
                publication: PublicationGoal::PublishNoPrimary {
                    reason: NoPrimaryReason::LeaseOpen,
                    fence_cutoff: None,
                },
                clear_switchover: false,
            },
            Some(_) | None => DesiredState {
                role: TargetRole::Idle(IdleReason::AwaitingLeader),
                publication: PublicationGoal::PublishNoPrimary {
                    reason: NoPrimaryReason::LeaseOpen,
                    fence_cutoff: None,
                },
                clear_switchover: false,
            },
        },
    }
}

fn leader_publication(
    world: &WorldView,
    self_id: &MemberId,
    epoch: &LeaseEpoch,
) -> PublicationGoal {
    match &world.local.postgres {
        PostgresState::Primary { .. } => PublicationGoal::PublishPrimary {
            primary: self_id.clone(),
            epoch: epoch.clone(),
        },
        PostgresState::Offline | PostgresState::Replica { .. } => PublicationGoal::PublishNoPrimary {
            reason: NoPrimaryReason::Recovering,
            fence_cutoff: None,
        },
    }
}

fn follow_goal(world: &WorldView, leader: MemberId) -> FollowGoal {
    let recovery = match (&world.local.data_dir, &world.local.process) {
        (DataDirState::Missing, _) => RecoveryPlan::Basebackup,
        (DataDirState::Initialized(LocalDataState::BootstrapEmpty), _) => RecoveryPlan::Basebackup,
        (
            DataDirState::Initialized(LocalDataState::Diverged(DivergenceState::BasebackupRequired)),
            _,
        ) => RecoveryPlan::Basebackup,
        (
            DataDirState::Initialized(LocalDataState::Diverged(DivergenceState::RewindPossible)),
            ProcessState::Failed(JobFailure {
                recovery: FailureRecovery::FallbackToBasebackup,
                ..
            }),
        ) => RecoveryPlan::Basebackup,
        (
            DataDirState::Initialized(LocalDataState::Diverged(DivergenceState::RewindPossible)),
            ProcessState::Failed(JobFailure {
                recovery: FailureRecovery::WaitForOperator,
                ..
            }),
        ) => RecoveryPlan::None,
        (
            DataDirState::Initialized(LocalDataState::Diverged(DivergenceState::RewindPossible)),
            ProcessState::Idle,
        ) => RecoveryPlan::Rewind,
        (
            DataDirState::Initialized(LocalDataState::Diverged(DivergenceState::RewindPossible)),
            ProcessState::Failed(JobFailure {
                recovery: FailureRecovery::RetrySameJob,
                ..
            }),
        ) => RecoveryPlan::Rewind,
        (DataDirState::Initialized(LocalDataState::ConsistentReplica), _) => RecoveryPlan::StartStreaming,
    };

    FollowGoal { leader, recovery }
}

fn candidacy_kind(world: &WorldView) -> Candidacy {
    match &world.local.data_dir {
        DataDirState::Missing => Candidacy::Bootstrap,
        DataDirState::Initialized(LocalDataState::BootstrapEmpty) => Candidacy::Bootstrap,
        DataDirState::Initialized(LocalDataState::ConsistentReplica) => Candidacy::Failover,
        DataDirState::Initialized(LocalDataState::Diverged(_)) => Candidacy::ResumeAfterOutage,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ResolvedSwitchover {
    NotRequested,
    Proceed(MemberId),
    Abandon,
}

fn resolve_switchover(world: &WorldView, self_id: &MemberId) -> ResolvedSwitchover {
    match &world.global.switchover {
        SwitchoverState::None => ResolvedSwitchover::NotRequested,
        SwitchoverState::Requested(request) => match &request.target {
            SwitchoverTarget::Specific(target) => {
                if target == self_id {
                    match classify_candidate(&world.global.self_peer) {
                        Some(()) => ResolvedSwitchover::Proceed(target.clone()),
                        None => ResolvedSwitchover::Abandon,
                    }
                } else {
                    match world.global.peers.get(target) {
                        Some(peer) => match classify_candidate(peer) {
                            Some(()) => ResolvedSwitchover::Proceed(target.clone()),
                            None => ResolvedSwitchover::Abandon,
                        },
                        None => ResolvedSwitchover::Abandon,
                    }
                }
            }
            SwitchoverTarget::AnyHealthyReplica => match find_best_candidate(
                &world.global.peers,
                &world.global.self_peer,
                self_id,
            ) {
                Some(best) => {
                    let target = match &world.global.lease {
                        LeaseState::HeldByMe(_) => find_best_peer_candidate(&world.global.peers),
                        LeaseState::HeldByPeer(_) | LeaseState::Unheld => Some(best),
                    };
                    match target {
                        Some(target) => ResolvedSwitchover::Proceed(target),
                        None => ResolvedSwitchover::Abandon,
                    }
                }
                None => ResolvedSwitchover::Abandon,
            },
        },
    }
}

fn find_best_peer_candidate(peers: &BTreeMap<MemberId, PeerKnowledge>) -> Option<MemberId> {
    peers.iter()
        .filter_map(|(member_id, peer)| {
            candidate_rank(peer).map(|rank| (member_id.clone(), rank))
        })
        .max_by(|(left_id, left_rank), (right_id, right_rank)| {
            compare_candidate_rank(left_id, left_rank, right_id, right_rank)
        })
        .map(|(member_id, _)| member_id)
}

fn find_best_candidate(
    peers: &BTreeMap<MemberId, PeerKnowledge>,
    self_peer: &PeerKnowledge,
    self_id: &MemberId,
) -> Option<MemberId> {
    let self_candidate = candidate_rank(self_peer).map(|rank| (self_id.clone(), rank));
    let peer_candidates = peers.iter().filter_map(|(member_id, peer)| {
        candidate_rank(peer).map(|rank| (member_id.clone(), rank))
    });

    self_candidate
        .into_iter()
        .chain(peer_candidates)
        .max_by(|(left_id, left_rank), (right_id, right_rank)| {
            compare_candidate_rank(left_id, left_rank, right_id, right_rank)
        })
        .map(|(member_id, _)| member_id)
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum CandidateRank {
    Bootstrap,
    Promote(WalPosition),
}

fn candidate_rank(peer: &PeerKnowledge) -> Option<CandidateRank> {
    match &peer.election {
        ElectionEligibility::BootstrapEligible => Some(CandidateRank::Bootstrap),
        ElectionEligibility::PromoteEligible(wal) => Some(CandidateRank::Promote(wal.clone())),
        ElectionEligibility::Ineligible(_) => None,
    }
}

fn compare_candidate_rank(
    left_id: &MemberId,
    left_rank: &CandidateRank,
    right_id: &MemberId,
    right_rank: &CandidateRank,
) -> std::cmp::Ordering {
    match (left_rank, right_rank) {
        (CandidateRank::Promote(left), CandidateRank::Promote(right)) => {
            left.timeline.cmp(&right.timeline)
                .then(left.lsn.cmp(&right.lsn))
                .then(right_id.cmp(left_id))
        }
        (CandidateRank::Promote(_), CandidateRank::Bootstrap) => std::cmp::Ordering::Greater,
        (CandidateRank::Bootstrap, CandidateRank::Promote(_)) => std::cmp::Ordering::Less,
        (CandidateRank::Bootstrap, CandidateRank::Bootstrap) => right_id.cmp(left_id),
    }
}

fn classify_candidate(peer: &PeerKnowledge) -> Option<()> {
    match &peer.election {
        ElectionEligibility::BootstrapEligible => Some(()),
        ElectionEligibility::PromoteEligible(_) => Some(()),
        ElectionEligibility::Ineligible(_) => None,
    }
}

fn ineligible_reason(peer: &PeerKnowledge) -> IneligibleReason {
    match &peer.election {
        ElectionEligibility::BootstrapEligible => IneligibleReason::StartingUp,
        ElectionEligibility::PromoteEligible(_) => IneligibleReason::NotReady,
        ElectionEligibility::Ineligible(reason) => reason.clone(),
    }
}
```

---

### 3. `reconcile.rs` (The Pure State Machine)
*This function is still pure. The important change is that it returns an ordered action plan, not one action. That is the minimum needed to keep publication, error recording, and local PG control explicit without smearing them back into the worker.*

```rust
use crate::types::*;

pub fn reconcile(world: &WorldView, desired: &DesiredState) -> Vec<ReconcileAction> {
    let publication_actions = reconcile_publication(&world.local.publication, desired);
    let switchover_actions = reconcile_switchover(world, desired);

    let role_action = match &world.local.process {
        ProcessState::Running(_) => None,
        ProcessState::Idle | ProcessState::Failed(_) => reconcile_role(world, &desired.role),
    };

    publication_actions
        .into_iter()
        .chain(switchover_actions)
        .chain(role_action)
        .collect()
}

fn reconcile_publication(
    current: &PublicationState,
    desired: &DesiredState,
) -> Vec<ReconcileAction> {
    let publish_action = match (
        &current.authority,
        &current.fence_cutoff,
        &desired.publication,
    ) {
        (_, _, PublicationGoal::KeepCurrent) => None,
        (
            AuthorityView::Primary {
                member: current_member,
                epoch: current_epoch,
            },
            current_cutoff,
            PublicationGoal::PublishPrimary {
                primary,
                epoch,
            },
        ) if current_member == primary && current_epoch == epoch && current_cutoff.is_none() => None,
        (
            AuthorityView::NoPrimary(current_reason),
            current_cutoff,
            PublicationGoal::PublishNoPrimary { reason, fence_cutoff },
        ) if current_reason == reason && current_cutoff == fence_cutoff => None,
        (_, _, publication) => Some(ReconcileAction::Publish(publication.clone())),
    };

    publish_action.into_iter().collect()
}

fn reconcile_switchover(world: &WorldView, desired: &DesiredState) -> Vec<ReconcileAction> {
    match (&world.global.switchover, desired.clear_switchover) {
        (SwitchoverState::Requested(_), true) => vec![ReconcileAction::ClearSwitchover],
        (SwitchoverState::None, _) | (_, false) => Vec::new(),
    }
}

fn reconcile_role(world: &WorldView, target: &TargetRole) -> Option<ReconcileAction> {
    match target {
        TargetRole::Leader(_) => match (&world.local.data_dir, &world.local.postgres) {
            (DataDirState::Missing, _) => Some(ReconcileAction::InitDb),
            (DataDirState::Initialized(LocalDataState::BootstrapEmpty), _) => Some(ReconcileAction::InitDb),
            (DataDirState::Initialized(_), PostgresState::Offline) => Some(ReconcileAction::StartPrimary),
            (DataDirState::Initialized(_), PostgresState::Replica { .. }) => Some(ReconcileAction::Promote),
            (DataDirState::Initialized(_), PostgresState::Primary { .. }) => {
                None
            }
        },

        TargetRole::Candidate(kind) => Some(ReconcileAction::AcquireLease(kind.clone())),

        TargetRole::Follower(goal) => match goal.recovery {
            RecoveryPlan::None => None,
            RecoveryPlan::Basebackup => Some(ReconcileAction::BaseBackup(goal.leader.clone())),
            RecoveryPlan::Rewind => Some(ReconcileAction::PgRewind(goal.leader.clone())),
            RecoveryPlan::StartStreaming => match &world.local.postgres {
                PostgresState::Offline => Some(ReconcileAction::StartReplica(goal.leader.clone())),
                PostgresState::Primary { .. } => Some(ReconcileAction::Demote(ShutdownMode::Fast)),
                PostgresState::Replica { upstream, .. } => match upstream {
                    Some(current_upstream) if current_upstream == &goal.leader => None,
                    Some(_) | None => Some(ReconcileAction::Demote(ShutdownMode::Fast)),
                },
            },
        },

        TargetRole::FailSafe(fail_safe) => match fail_safe {
            FailSafeGoal::PrimaryMustStop(_) => Some(ReconcileAction::Demote(ShutdownMode::Immediate)),
            FailSafeGoal::ReplicaKeepFollowing(_) => None,
            FailSafeGoal::WaitForQuorum => None,
        },

        TargetRole::DemotingForSwitchover(_) => match &world.local.postgres {
            PostgresState::Primary { .. } | PostgresState::Replica { .. } => {
                Some(ReconcileAction::Demote(ShutdownMode::Fast))
            }
            PostgresState::Offline => match &world.global.lease {
                LeaseState::HeldByMe(_) => Some(ReconcileAction::ReleaseLease),
                LeaseState::HeldByPeer(_) | LeaseState::Unheld => None,
            },
        },

        TargetRole::Fenced(FenceReason::ForeignLeaderDetected) => {
            match &world.local.postgres {
                PostgresState::Primary { .. } | PostgresState::Replica { .. } => {
                    Some(ReconcileAction::Demote(ShutdownMode::Immediate))
                }
                PostgresState::Offline => None,
            }
        }

        TargetRole::Fenced(FenceReason::StorageStalled) => {
            match &world.local.postgres {
                PostgresState::Primary { .. } | PostgresState::Replica { .. } => {
                    Some(ReconcileAction::Demote(ShutdownMode::Immediate))
                }
                PostgresState::Offline => None,
            }
        }

        TargetRole::Idle(_) => match &world.local.postgres {
            PostgresState::Primary { .. } => Some(ReconcileAction::Demote(ShutdownMode::Fast)),
            PostgresState::Offline | PostgresState::Replica { .. } => None,
        },
    }
}
```

---

### 4. `worker.rs` (The IO Boundary)
*The worker is still small. It now executes an ordered plan and it refuses to fabricate unknown state. No empty `MemberId`, no fake default job kind, no silent downgrade from malformed runtime state into "probably fine".*

```rust
use crate::decide::decide;
use crate::reconcile::reconcile;
use crate::types::*;
use crate::ha::state::HaWorkerCtx;
use crate::state::WorkerError;

pub(crate) async fn step_once(ctx: &HaWorkerCtx) -> Result<(), WorkerError> {
    let world = observe(ctx).await?;
    let desired = decide(&world, &ctx.self_id);

    if ctx.state.desired != desired {
        emit_state_transition_log(&ctx.state.desired, &desired);
    }

    let actions = reconcile(&world, &desired);

    for action in actions {
        emit_action_intent_log(&action);
        execute_action(ctx, action).await?;
    }

    Ok(())
}

async fn observe(ctx: &HaWorkerCtx) -> Result<WorldView, WorkerError> {
    Ok(WorldView {
        local: build_local_knowledge(ctx).await?,
        global: build_global_knowledge(ctx).await?,
    })
}

async fn execute_action(ctx: &HaWorkerCtx, action: ReconcileAction) -> Result<(), WorkerError> {
    match action {
        ReconcileAction::AcquireLease(kind) => {
            ctx.dcs_store.acquire_leader_lease(&ctx.scope, &ctx.self_id, kind).await
        }
        ReconcileAction::ReleaseLease => {
            ctx.dcs_store.release_leader_lease(&ctx.scope, &ctx.self_id).await
        }
        ReconcileAction::Publish(goal) => {
            ctx.publisher.publish_authority_projection(&ctx.scope, goal).await
        }
        ReconcileAction::ClearSwitchover => {
            ctx.dcs_store.clear_switchover(&ctx.scope).await
        }
        process_action => {
            let request = build_process_job_request(process_action)?;
            ctx.process_inbox.send(request).await
        }
    }
}

// The adapters are strict:
// - unknown pg/process states become WorkerError::StateMapping
// - missing upstream is `Option<MemberId>`, never an empty string sentinel
// - process failures are carried as typed `JobFailure`
// - publication state is observed just like PG and DCS state, not reconstructed ad hoc
// - member/replica inventory still comes from per-member DCS records and API sampling, not from this authority projection
// - invalid targeted switchover requests are rejected synchronously at the API boundary before they ever reach this loop
// - rejected requests and auto-cleared stale requests are still emitted to structured logs / retained debug events for operator auditability
```

---

### Clarification: what "publish" means in practice
- Every node still runs this loop independently.
- `Publish(...)` is not a peer-consumed control-plane message. Other nodes do not change behavior because this node published a replica list or authority answer.
- Peer behavior is still driven by DCS lease ownership, switchover state, and observed per-member records.
- The thing being published here is only this node's operator-facing authority projection, so local `/ha/state`, retained debug output, and operator tooling can fail closed without reconstructing special-case logic ad hoc.
- Replica membership for `pgtm replicas` should stay derived from sampled member records. It should not be authored by the HA decision loop as a separate cluster-truth object.
- If an already-accepted targeted switchover later becomes unsafe or impossible, the loop clears that request and returns to normal HA policy instead of keeping zombie operator intent around forever.
- That rejection/auto-clear path should still be logged with the target and blocker/cause, but it should be an audit event, not steady-state HA data.

---

### Why this revised design fixes the flaws while keeping the same spirit
1. **The core architecture still works.** We still have a pure typed kernel with one loop. What changed is not the philosophy. What changed is the shape of the policy output: local role alone was too small.
2. **Generic switchover can no longer self-select the old leader.** When the current lease holder is resolving `AnyHealthyReplica`, it must choose from peers only. That is required by the planned switchover features.
3. **Targeted switchover is now exclusive.** When a specific target is valid, only that target can become `Candidate`. Everyone else becomes `Idle(AwaitingTarget(...))`.
4. **Invalid targeted requests are API concerns, not HA-loop concerns.** The current feature suite captures synchronous request rejection. The HA loop should only see already accepted requests, but both API rejection and later auto-clear should still be logged for auditability.
5. **Accepted switchover intent is not sticky forever.** If the requested target later becomes unsafe or impossible, the loop auto-clears the switchover request, logs why, and resumes ordinary HA policy.
6. **Bootstrap and restore are now deterministic.** `find_best_candidate` includes `self` for ordinary failover/bootstrap, distinguishes `BootstrapEligible` from `PromoteEligible`, and gives deterministic tie-breaking.
7. **Rejoin fallback is now typed.** `JobFailure` plus `FailureRecovery` makes `pg_rewind -> basebackup` an explicit transition instead of hidden retry folklore.
8. **Operator-visible authority is now designed, not implied.** `PublicationGoal` is part of the pure decision, but it is intentionally narrow: it projects primary/no-primary authority only, while member and replica inventory stays derived from observed member records.
9. **Fencing cutoff is now explicit and diffed correctly.** `FenceCutoff` is carried through fail-safe publication, and reconcile must compare the cutoff itself rather than only the high-level reason.
10. **Wedged primary behavior is now modeled.** `StorageState::Stalled` lets the pure policy fence a hung leader without pretending it is a normal crash.

### What enums and matches changed from the first pass
- `TargetRole` is larger: `WaitingForLeader` became `Idle(IdleReason)`, `Candidate` is now `Candidate(Candidacy)`, and switchover demotion became its own `DemotingForSwitchover(MemberId)` state.
- `decide()` no longer returns just `TargetRole`. It returns `DesiredState { role, publication, clear_switchover }`.
- `ReconcileAction` gained `Publish` and `ClearSwitchover`.
- `LeaseToken` became `LeaseEpoch` because generation matters for fencing and publication.
- `PgStatus` became `PostgresState`, and `SyncState` became `ReplicationState`, so stalled/catching-up replica cases are representable.
- `LocalKnowledge` gained `ProcessState`, `StorageState`, and `PublicationState`.
- `GlobalKnowledge` gained `SwitchoverState`, `PeerKnowledge`, and an explicit `self_peer`.
- `PublicationGoal` no longer carries a replica set. Replica inventory is derived from member records rather than "published" as a second source of cluster truth.
- `reconcile()` no longer returns one optional action. It returns an ordered `Vec<ReconcileAction>` so publication and local PG actions can both be explicit.

### Final judgement
Yes, the core of the design absolutely still works.

The correct move is not to abandon the pure-kernel architecture. The correct move is to enlarge the types until the feature suite fits inside them naturally. The first pass had the right shape, but it was still missing three categories of truth:
- operator-visible authority suppression/projection
- recovery outcome and fallback
- explicit switchover eligibility

With those added, the design remains elegant, testable, and compiler-driven, while now matching the real HA scenarios we actually care about.

<acceptance_criteria>
- [ ] Code should skip og bootstrap and go directly into this ha loop, all original bootstrap logic + startup states must have been cleaned up.
- [ ] Simplify Dcs member state, as specified by po comment
- [ ] All instances of api stale data must go, as specified by po comment
- [ ] DEEP INVESTIGATION must be done to find all current ways 'stale' data is entering the system, as specified by po comment
- [ ] DEEP INVESTIGATION must be done on .ralph/progress/189.jsonl, to find any other weird shit behaviour, all those findings must have listed on the place designated for that in this task
- [ ] Implement the full design as much as stated as possibly, with those structs, those enums, those functions, only altering them slightly to either fit the code or fix edge cases
- [ ] The implementation remains in the spirit of the original design-based request: strong use of Rust's type system, maintainable structure, conceptual simplicity, and net code reduction rather than more incidental machinery.
- [ ] All old code, old source paths, old structs, old assumptions, and other stale ha loop design leftovers that conflict with this task are fully cleaned out and stripped rather than kept around beside the refactor.
- [ ] All unit tests that assumed the old behavior are updated to align with the `.feature` files first and then with this task's instructions, so the lower-level tests validate the same HA contract as the feature suite.
- [ ] The whole codebase is verified to follow the new design defined in this task, and any design drift or half-migrated logic discovered during the work is removed or brought into alignment before the task is considered done.
- [ ] Also clean up stale tests/pieces of code that do not help the task and/or grander goal: making those make test-long bdd tests pass.
- [ ] The work is executed in the required order: refactor the code first, then only if implementation proves the design is still incomplete, tune the design in the same spirit afterward instead of redesigning first.
- [ ] `make check` passes cleanly.
- [ ] `make test` passes cleanly.
- [ ] `make test-long` passes cleanly.
- [ ] `make lint` passes cleanly.
- [ ] `<passes>true</passes>` is not set until every required acceptance criterion is complete and the required verification commands have actually passed.
</acceptance_criteria>

Plan review notes before execution:
- Keep external DCS trust compatibility where needed, but the HA policy itself may collapse any non-`FullQuorum` trust into the degraded/fail-safe branch.
- Add an explicit leader lease generation/epoch to the DCS leader record so fencing cutoffs and operator-facing authority projection can refer to a concrete lease instance instead of only a member id.
- Replace the operator-facing `/ha/state` contract and matching docs/tests so it reflects the new authority/publication-oriented HA state; keep richer internal decision/action detail in debug surfaces instead of preserving the old public phase/decision shape.

TO BE VERIFIED



