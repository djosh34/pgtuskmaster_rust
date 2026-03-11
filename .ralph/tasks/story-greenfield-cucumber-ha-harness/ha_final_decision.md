
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

The core fix is this: the design must not only choose a local role. It must also choose the operator-visible authority view and the recovery path. The feature suite proved those are first-class state, not side effects we can hand-wave.

### 1. `types.rs` (The Epistemology)
*This file still contains zero logic. It now models the missing things explicitly: publication, switchover eligibility, recovery fallback, and fencing cutoffs.*

```rust
use std::collections::{BTreeMap, BTreeSet};

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
    pub replicas: BTreeSet<MemberId>,
    pub last_error: Option<OperatorError>,
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
    pub operator_error: Option<OperatorError>,
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
    SwitchoverRejected(SwitchoverBlocker),
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
        replicas: BTreeSet<MemberId>,
        epoch: LeaseEpoch,
    },
    PublishNoPrimary {
        reason: NoPrimaryReason,
        replicas: BTreeSet<MemberId>,
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
pub enum OperatorError {
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
    RecordOperatorError(OperatorError),
    ClearOperatorError,
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
*This module still has no side effects. The important change is that it returns a full `DesiredState`, not just a role. That is what fixes operator-visible primary, switchover rejection, and fencing-cutoff correctness.*

```rust
use crate::types::*;
use std::collections::{BTreeMap, BTreeSet};

pub fn decide(world: &WorldView, self_id: &MemberId) -> DesiredState {
    let known_members = known_members(&world.global.peers, self_id);

    if world.global.dcs_trust == DcsTrust::Degraded {
        return decide_degraded(world, known_members);
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
                        replicas: known_members,
                        fence_cutoff: Some(cutoff),
                    },
                    operator_error: None,
                };
            }
        }
    }

    match &world.global.lease {
        LeaseState::HeldByMe(epoch) => {
            decide_as_lease_holder(world, self_id, epoch.clone(), known_members)
        }
        LeaseState::HeldByPeer(epoch) => {
            let publication = PublicationGoal::PublishPrimary {
                primary: epoch.holder.clone(),
                replicas: replica_members(&known_members, &epoch.holder),
                epoch: epoch.clone(),
            };

            match &world.local.postgres {
                PostgresState::Primary { .. } => DesiredState {
                    role: TargetRole::Fenced(FenceReason::ForeignLeaderDetected),
                    publication,
                    operator_error: None,
                },
                PostgresState::Offline | PostgresState::Replica { .. } => DesiredState {
                    role: TargetRole::Follower(follow_goal(world, epoch.holder.clone())),
                    publication,
                    operator_error: None,
                },
            }
        }
        LeaseState::Unheld => decide_without_lease(world, self_id, known_members),
    }
}

fn decide_degraded(world: &WorldView, known_members: BTreeSet<MemberId>) -> DesiredState {
    match &world.local.postgres {
        PostgresState::Primary { committed_lsn } => match &world.global.lease {
            LeaseState::HeldByMe(epoch) | LeaseState::HeldByPeer(epoch) => DesiredState {
                role: TargetRole::FailSafe(FailSafeGoal::PrimaryMustStop(FenceCutoff {
                    epoch: epoch.clone(),
                    committed_lsn: *committed_lsn,
                })),
                publication: PublicationGoal::PublishNoPrimary {
                    reason: NoPrimaryReason::DcsDegraded,
                    replicas: known_members,
                    fence_cutoff: Some(FenceCutoff {
                        epoch: epoch.clone(),
                        committed_lsn: *committed_lsn,
                    }),
                },
                operator_error: None,
            },
            LeaseState::Unheld => DesiredState {
                role: TargetRole::FailSafe(FailSafeGoal::WaitForQuorum),
                publication: PublicationGoal::PublishNoPrimary {
                    reason: NoPrimaryReason::DcsDegraded,
                    replicas: known_members,
                    fence_cutoff: None,
                },
                operator_error: None,
            },
        },
        PostgresState::Replica { upstream, .. } => DesiredState {
            role: TargetRole::FailSafe(FailSafeGoal::ReplicaKeepFollowing(upstream.clone())),
            publication: PublicationGoal::PublishNoPrimary {
                reason: NoPrimaryReason::DcsDegraded,
                replicas: known_members,
                fence_cutoff: None,
            },
            operator_error: None,
        },
        PostgresState::Offline => DesiredState {
            role: TargetRole::FailSafe(FailSafeGoal::WaitForQuorum),
            publication: PublicationGoal::PublishNoPrimary {
                reason: NoPrimaryReason::DcsDegraded,
                replicas: known_members,
                fence_cutoff: None,
            },
            operator_error: None,
        },
    }
}

fn decide_as_lease_holder(
    world: &WorldView,
    self_id: &MemberId,
    epoch: LeaseEpoch,
    known_members: BTreeSet<MemberId>,
) -> DesiredState {
    match resolve_switchover(world, self_id) {
        ResolvedSwitchover::NotRequested => DesiredState {
            role: TargetRole::Leader(epoch.clone()),
            publication: PublicationGoal::PublishPrimary {
                primary: self_id.clone(),
                replicas: replica_members(&known_members, self_id),
                epoch,
            },
            operator_error: None,
        },
        ResolvedSwitchover::Proceed(target) if target == *self_id => DesiredState {
            role: TargetRole::Leader(epoch.clone()),
            publication: PublicationGoal::PublishPrimary {
                primary: self_id.clone(),
                replicas: replica_members(&known_members, self_id),
                epoch,
            },
            operator_error: None,
        },
        ResolvedSwitchover::Proceed(target) => DesiredState {
            role: TargetRole::DemotingForSwitchover(target),
            publication: PublicationGoal::KeepCurrent,
            operator_error: None,
        },
        ResolvedSwitchover::Rejected(blocker) => DesiredState {
            role: TargetRole::Leader(epoch.clone()),
            publication: PublicationGoal::PublishPrimary {
                primary: self_id.clone(),
                replicas: replica_members(&known_members, self_id),
                epoch,
            },
            operator_error: Some(OperatorError::SwitchoverRejected(blocker)),
        },
    }
}

fn decide_without_lease(
    world: &WorldView,
    self_id: &MemberId,
    known_members: BTreeSet<MemberId>,
) -> DesiredState {
    match resolve_switchover(world, self_id) {
        ResolvedSwitchover::Proceed(target) if target == *self_id => DesiredState {
            role: TargetRole::Candidate(Candidacy::TargetedSwitchover(target)),
            publication: PublicationGoal::PublishNoPrimary {
                reason: NoPrimaryReason::LeaseOpen,
                replicas: known_members,
                fence_cutoff: None,
            },
            operator_error: None,
        },
        ResolvedSwitchover::Proceed(target) => DesiredState {
            role: TargetRole::Idle(IdleReason::AwaitingTarget(target)),
            publication: PublicationGoal::PublishNoPrimary {
                reason: NoPrimaryReason::LeaseOpen,
                replicas: known_members,
                fence_cutoff: None,
            },
            operator_error: None,
        },
        ResolvedSwitchover::Rejected(blocker) => DesiredState {
            role: TargetRole::Idle(IdleReason::SwitchoverRejected(blocker.clone())),
            publication: PublicationGoal::PublishNoPrimary {
                reason: NoPrimaryReason::SwitchoverRejected(blocker.clone()),
                replicas: known_members,
                fence_cutoff: None,
            },
            operator_error: Some(OperatorError::SwitchoverRejected(blocker)),
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
                    replicas: known_members,
                    fence_cutoff: None,
                },
                operator_error: None,
            },
            Some(_) | None => DesiredState {
                role: TargetRole::Idle(IdleReason::AwaitingLeader),
                publication: PublicationGoal::PublishNoPrimary {
                    reason: NoPrimaryReason::LeaseOpen,
                    replicas: known_members,
                    fence_cutoff: None,
                },
                operator_error: None,
            },
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
    Rejected(SwitchoverBlocker),
}

fn resolve_switchover(world: &WorldView, self_id: &MemberId) -> ResolvedSwitchover {
    match &world.global.switchover {
        SwitchoverState::None => ResolvedSwitchover::NotRequested,
        SwitchoverState::Requested(request) => match &request.target {
            SwitchoverTarget::Specific(target) => {
                if target == self_id {
                    match classify_candidate(&world.global.self_peer) {
                        Some(()) => ResolvedSwitchover::Proceed(target.clone()),
                        None => ResolvedSwitchover::Rejected(SwitchoverBlocker::TargetIneligible(
                            ineligible_reason(&world.global.self_peer),
                        )),
                    }
                } else {
                    match world.global.peers.get(target) {
                        Some(peer) => match classify_candidate(peer) {
                            Some(()) => ResolvedSwitchover::Proceed(target.clone()),
                            None => ResolvedSwitchover::Rejected(SwitchoverBlocker::TargetIneligible(
                                ineligible_reason(peer),
                            )),
                        },
                        None => ResolvedSwitchover::Rejected(SwitchoverBlocker::TargetMissing),
                    }
                }
            }
            SwitchoverTarget::AnyHealthyReplica => match find_best_candidate(
                &world.global.peers,
                &world.global.self_peer,
                self_id,
            ) {
                Some(best) => ResolvedSwitchover::Proceed(best),
                None => ResolvedSwitchover::Rejected(SwitchoverBlocker::TargetMissing),
            },
        },
    }
}

fn known_members(peers: &BTreeMap<MemberId, PeerKnowledge>, self_id: &MemberId) -> BTreeSet<MemberId> {
    std::iter::once(self_id.clone())
        .chain(peers.keys().cloned())
        .collect()
}

fn replica_members(all_members: &BTreeSet<MemberId>, primary: &MemberId) -> BTreeSet<MemberId> {
    all_members.iter()
        .filter(|member_id| *member_id != primary)
        .cloned()
        .collect()
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

    let role_action = match &world.local.process {
        ProcessState::Running(_) => None,
        ProcessState::Idle | ProcessState::Failed(_) => reconcile_role(world, &desired.role),
    };

    publication_actions
        .into_iter()
        .chain(role_action)
        .collect()
}

fn reconcile_publication(
    current: &PublicationState,
    desired: &DesiredState,
) -> Vec<ReconcileAction> {
    let publish_action = match (&current.authority, &desired.publication) {
        (AuthorityView::Unknown, PublicationGoal::KeepCurrent) => None,
        (_, PublicationGoal::KeepCurrent) => None,
        (
            AuthorityView::Primary {
                member: current_member,
                epoch: current_epoch,
            },
            PublicationGoal::PublishPrimary {
                primary,
                epoch,
                ..
            },
        ) if current_member == primary && current_epoch == epoch => None,
        (
            AuthorityView::NoPrimary(current_reason),
            PublicationGoal::PublishNoPrimary { reason, .. },
        ) if current_reason == reason => None,
        (_, publication) => Some(ReconcileAction::Publish(publication.clone())),
    };

    let error_action = match (&current.last_error, &desired.operator_error) {
        (None, None) => None,
        (Some(_), None) => Some(ReconcileAction::ClearOperatorError),
        (Some(current_error), Some(desired_error)) if current_error == desired_error => None,
        (None, Some(desired_error)) => Some(ReconcileAction::RecordOperatorError(desired_error.clone())),
        (Some(_), Some(desired_error)) => Some(ReconcileAction::RecordOperatorError(desired_error.clone())),
    };

    [publish_action, error_action]
        .into_iter()
        .flatten()
        .collect()
}

fn reconcile_role(world: &WorldView, target: &TargetRole) -> Option<ReconcileAction> {
    match target {
        TargetRole::Leader(_) => match (&world.local.data_dir, &world.local.postgres) {
            (DataDirState::Missing, _) => Some(ReconcileAction::InitDb),
            (DataDirState::Initialized(LocalDataState::BootstrapEmpty), _) => Some(ReconcileAction::InitDb),
            (DataDirState::Initialized(_), PostgresState::Offline) => Some(ReconcileAction::StartPrimary),
            (DataDirState::Initialized(_), PostgresState::Replica { .. }) => Some(ReconcileAction::Promote),
            (DataDirState::Initialized(_), PostgresState::Primary { .. }) => {
                match &world.global.switchover {
                    SwitchoverState::None => None,
                    SwitchoverState::Requested(_) => Some(ReconcileAction::ClearSwitchover),
                }
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
            ctx.publisher.publish_cluster_view(&ctx.scope, goal).await
        }
        ReconcileAction::RecordOperatorError(error) => {
            ctx.publisher.publish_operator_error(&ctx.scope, error).await
        }
        ReconcileAction::ClearOperatorError => {
            ctx.publisher.clear_operator_error(&ctx.scope).await
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
```

---

### Why this revised design fixes the flaws while keeping the same spirit
1. **The core architecture still works.** We still have a pure typed kernel with one loop. What changed is not the philosophy. What changed is the shape of the policy output: local role alone was too small.
2. **Targeted switchover is now exclusive.** When a specific target is valid, only that target can become `Candidate`. Everyone else becomes `Idle(AwaitingTarget(...))`.
3. **Targeted switchover rejection is now first-class.** An invalid target yields `OperatorError::SwitchoverRejected(...)` and `PublicationGoal::PublishNoPrimary` only where appropriate. The leader no longer blindly demotes for a bad request.
4. **Bootstrap and restore are now deterministic.** `find_best_candidate` includes `self`, distinguishes `BootstrapEligible` from `PromoteEligible`, and gives deterministic tie-breaking.
5. **Rejoin fallback is now typed.** `JobFailure` plus `FailureRecovery` makes `pg_rewind -> basebackup` an explicit transition instead of hidden retry folklore.
6. **Operator-visible primary is now designed, not implied.** `PublicationGoal` is part of the pure decision, so `pgtm` and "no operator-visible primary" are part of the model.
7. **Fencing cutoff is now explicit.** `FenceCutoff` is carried through fail-safe publication, which is what the concurrent-write features need.
8. **Wedged primary behavior is now modeled.** `StorageState::Stalled` lets the pure policy fence a hung leader without pretending it is a normal crash.

### What enums and matches changed from the first pass
- `TargetRole` is larger: `WaitingForLeader` became `Idle(IdleReason)`, `Candidate` is now `Candidate(Candidacy)`, and switchover demotion became its own `DemotingForSwitchover(MemberId)` state.
- `decide()` no longer returns just `TargetRole`. It returns `DesiredState { role, publication, operator_error }`.
- `ReconcileAction` gained `Publish`, `RecordOperatorError`, and `ClearOperatorError`.
- `LeaseToken` became `LeaseEpoch` because generation matters for fencing and publication.
- `PgStatus` became `PostgresState`, and `SyncState` became `ReplicationState`, so stalled/catching-up replica cases are representable.
- `LocalKnowledge` gained `ProcessState`, `StorageState`, and `PublicationState`.
- `GlobalKnowledge` gained `SwitchoverState`, `PeerKnowledge`, and an explicit `self_peer`.
- `reconcile()` no longer returns one optional action. It returns an ordered `Vec<ReconcileAction>` so publication and local PG actions can both be explicit.

### Final judgement
Yes, the core of the design absolutely still works.

The correct move is not to abandon the pure-kernel architecture. The correct move is to enlarge the types until the feature suite fits inside them naturally. The first pass had the right shape, but it was still missing three categories of truth:
- operator-visible authority
- recovery outcome and fallback
- explicit switchover eligibility

With those added, the design remains elegant, testable, and compiler-driven, while now matching the real HA scenarios we actually care about.
