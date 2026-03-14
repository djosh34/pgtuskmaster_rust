use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    dcs::DcsTrust,
    process::{
        jobs::{ActiveJobKind, ProcessIntent},
        state::{JobOutcome, ProcessState as WorkerProcessState},
    },
    state::{MemberId, TimelineId, UnixMillis, WalLsn},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeaseEpoch {
    pub holder: MemberId,
    pub generation: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FenceCutoff {
    pub epoch: LeaseEpoch,
    pub committed_lsn: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldView {
    pub local: LocalKnowledge,
    pub global: GlobalKnowledge,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalKnowledge {
    pub data_dir: DataDirState,
    pub postgres: PostgresState,
    pub process: ProcessState,
    pub storage: StorageState,
    pub required_roles_ready: bool,
    pub publication: PublicationState,
    pub observation: ObservationState,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservationState {
    pub pg_observed_at: UnixMillis,
    pub last_start_success_at: Option<UnixMillis>,
    pub last_promote_success_at: Option<UnixMillis>,
    pub last_demote_success_at: Option<UnixMillis>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataDirState {
    Missing,
    Initialized(LocalDataState),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocalDataState {
    BootstrapEmpty,
    ConsistentReplica,
    Diverged(DivergenceState),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DivergenceState {
    RewindPossible,
    BasebackupRequired,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PostgresState {
    Offline,
    Primary {
        committed_lsn: u64,
    },
    Replica {
        upstream: Option<MemberId>,
        replication: ReplicationState,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplicationState {
    Streaming(WalPosition),
    CatchingUp(WalPosition),
    Stalled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessState {
    Idle,
    Running(ActiveJobKind),
    Failed(JobFailure),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobFailure {
    pub job: ActiveJobKind,
    pub recovery: FailureRecovery,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailureRecovery {
    RetrySameJob,
    FallbackToBasebackup,
    WaitForOperator,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageState {
    Healthy,
    Stalled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PublicationState {
    Unknown,
    Projected(AuthorityProjection),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthorityProjection {
    Primary(LeaseEpoch),
    NoPrimary(NoPrimaryProjection),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NoPrimaryProjection {
    LeaseOpen,
    Recovering {
        epoch: Option<LeaseEpoch>,
        fence: NoPrimaryFence,
    },
    DcsDegraded {
        fence: NoPrimaryFence,
    },
    StaleObservedLease {
        epoch: LeaseEpoch,
        reason: StaleLeaseReason,
    },
    SwitchoverRejected(SwitchoverBlocker),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NoPrimaryFence {
    None,
    Cutoff(FenceCutoff),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalKnowledge {
    pub coordination: CoordinationState,
    pub switchover: SwitchoverState,
    pub peers: BTreeMap<MemberId, PeerKnowledge>,
    pub self_peer: PeerKnowledge,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoordinationState {
    pub trust: DcsTrust,
    pub leadership: LeadershipView,
    pub primary: PrimaryObservation,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeadershipView {
    Open,
    HeldBySelf(LeaseEpoch),
    HeldByPeer {
        epoch: LeaseEpoch,
        state: PeerLeaderState,
    },
    StaleObservedLease {
        epoch: LeaseEpoch,
        reason: StaleLeaseReason,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PeerLeaderState {
    PrimaryReady,
    Recovering,
    Unreachable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StaleLeaseReason {
    HolderMissing,
    HolderNotPrimary,
    HolderNotReady,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimaryObservation {
    Absent,
    Observed(ObservedPrimary),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservedPrimary {
    pub member: MemberId,
    pub timeline: Option<u64>,
    pub system_identifier: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SwitchoverState {
    None,
    Requested(SwitchoverRequest),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SwitchoverRequest {
    pub target: SwitchoverTarget,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SwitchoverTarget {
    AnyHealthyReplica,
    Specific(MemberId),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerKnowledge {
    pub eligibility: ElectionEligibility,
    pub api: ApiVisibility,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElectionEligibility {
    BootstrapEligible,
    PromoteEligible(WalPosition),
    Ineligible(IneligibleReason),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum IneligibleReason {
    NotReady,
    Lagging,
    Partitioned,
    ApiUnavailable,
    StartingUp,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApiVisibility {
    Reachable,
    Unreachable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DesiredState {
    pub role: TargetRole,
    pub publication: PublicationGoal,
    pub clear_switchover: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetRole {
    Leader(LeaseEpoch),
    Candidate(Candidacy),
    Follower(FollowGoal),
    FailSafe(FailSafeGoal),
    DemotingForSwitchover(MemberId),
    Fenced(FenceReason),
    Idle(IdleReason),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Candidacy {
    Bootstrap,
    Failover,
    ResumeAfterOutage,
    TargetedSwitchover(MemberId),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FollowGoal {
    pub leader: MemberId,
    pub recovery: RecoveryPlan,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryPlan {
    None,
    StartStreaming,
    Rewind,
    Basebackup,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailSafeGoal {
    PrimaryMustStop(FenceCutoff),
    ReplicaKeepFollowing(Option<MemberId>),
    WaitForQuorum,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdleReason {
    AwaitingLeader,
    AwaitingTarget(MemberId),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FenceReason {
    ForeignLeaderDetected,
    StorageStalled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PublicationGoal {
    KeepCurrent,
    Publish(AuthorityProjection),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NoPrimaryReason {
    DcsDegraded,
    LeaseOpen,
    Recovering,
    SwitchoverRejected(SwitchoverBlocker),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SwitchoverBlocker {
    TargetMissing,
    TargetIneligible(IneligibleReason),
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannedActions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication: Option<PublicationAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coordination: Option<CoordinationAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local: Option<LocalAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub process: Option<ProcessIntent>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ReconcilePlan {
    pub(crate) publication: Option<PublicationAction>,
    pub(crate) coordination: Option<CoordinationAction>,
    pub(crate) local: Option<LocalAction>,
    pub(crate) process: Option<ProcessIntent>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoordinationAction {
    AcquireLease(Candidacy),
    ReleaseLease,
    ClearSwitchover,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocalAction {
    EnsureRequiredRoles,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PublicationAction {
    Publish(PublicationGoal),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct WalPosition {
    pub timeline: u64,
    pub lsn: u64,
}

pub type AuthorityProjectionState = PublicationState;

impl WorldView {
    pub(crate) fn initial() -> Self {
        Self {
            local: LocalKnowledge {
                data_dir: DataDirState::Missing,
                postgres: PostgresState::Offline,
                process: ProcessState::Idle,
                storage: StorageState::Healthy,
                required_roles_ready: false,
                publication: PublicationState::unknown(),
                observation: ObservationState {
                    pg_observed_at: UnixMillis(0),
                    last_start_success_at: None,
                    last_promote_success_at: None,
                    last_demote_success_at: None,
                },
            },
            global: GlobalKnowledge {
                coordination: CoordinationState {
                    trust: DcsTrust::NotTrusted,
                    leadership: LeadershipView::Open,
                    primary: PrimaryObservation::Absent,
                },
                switchover: SwitchoverState::None,
                peers: BTreeMap::new(),
                self_peer: PeerKnowledge {
                    eligibility: ElectionEligibility::Ineligible(IneligibleReason::StartingUp),
                    api: ApiVisibility::Reachable,
                },
            },
        }
    }
}

impl ObservationState {
    pub(crate) fn waiting_for_fresh_pg_after_start(&self) -> bool {
        self.last_start_success_at
            .map(|finished_at| finished_at.0 >= self.pg_observed_at.0)
            .unwrap_or(false)
    }

    pub(crate) fn waiting_for_fresh_pg_after_promote(&self) -> bool {
        self.last_promote_success_at
            .map(|finished_at| finished_at.0 >= self.pg_observed_at.0)
            .unwrap_or(false)
    }

    pub(crate) fn waiting_for_fresh_pg_after_demote(&self) -> bool {
        self.last_demote_success_at
            .map(|finished_at| finished_at.0 >= self.pg_observed_at.0)
            .unwrap_or(false)
    }
}

impl PublicationState {
    pub(crate) fn unknown() -> Self {
        Self::Unknown
    }
}

impl TargetRole {
    pub(crate) fn label(&self) -> &'static str {
        match self {
            Self::Leader(_) => "leader",
            Self::Candidate(_) => "candidate",
            Self::Follower(_) => "follower",
            Self::FailSafe(_) => "fail_safe",
            Self::DemotingForSwitchover(_) => "demoting_for_switchover",
            Self::Fenced(_) => "fenced",
            Self::Idle(_) => "idle",
        }
    }
}

impl PlannedActions {
    pub(crate) fn from_plan(value: &ReconcilePlan) -> Self {
        Self {
            publication: value.publication.clone(),
            coordination: value.coordination.clone(),
            local: value.local.clone(),
            process: value.process.clone(),
        }
    }
}

impl ReconcilePlan {
    pub(crate) fn process(intent: ProcessIntent) -> Self {
        Self {
            process: Some(intent),
            ..Self::default()
        }
    }

    pub(crate) fn coordination(action: CoordinationAction) -> Self {
        Self {
            coordination: Some(action),
            ..Self::default()
        }
    }

    pub(crate) fn local(action: LocalAction) -> Self {
        Self {
            local: Some(action),
            ..Self::default()
        }
    }

    pub(crate) fn publication(action: PublicationAction) -> Self {
        Self {
            publication: Some(action),
            ..Self::default()
        }
    }

    pub(crate) fn merge(self, other: Self) -> Self {
        Self {
            publication: self.publication.or(other.publication),
            coordination: self.coordination.or(other.coordination),
            local: self.local.or(other.local),
            process: self.process.or(other.process),
        }
    }

    #[cfg(test)]
    pub(crate) fn is_empty(&self) -> bool {
        self.publication.is_none()
            && self.coordination.is_none()
            && self.local.is_none()
            && self.process.is_none()
    }
}

impl From<&WorkerProcessState> for ProcessState {
    fn from(value: &WorkerProcessState) -> Self {
        match value {
            WorkerProcessState::Running { active, .. } => Self::Running(active.kind.clone()),
            WorkerProcessState::Idle {
                last_outcome: Some(JobOutcome::Failure { job_kind, .. }),
                ..
            }
            | WorkerProcessState::Idle {
                last_outcome: Some(JobOutcome::Timeout { job_kind, .. }),
                ..
            } => Self::Failed(JobFailure {
                job: job_kind.clone(),
                recovery: failure_recovery_from_job(job_kind),
            }),
            WorkerProcessState::Idle { .. } => Self::Idle,
        }
    }
}

pub(crate) fn last_success_at(
    value: &WorkerProcessState,
    expected: ActiveJobKind,
) -> Option<UnixMillis> {
    match value {
        WorkerProcessState::Idle {
            last_outcome:
                Some(JobOutcome::Success {
                    job_kind,
                    finished_at,
                    ..
                }),
            ..
        } if *job_kind == expected => Some(*finished_at),
        _ => None,
    }
}

pub(crate) fn last_start_success_at(value: &WorkerProcessState) -> Option<UnixMillis> {
    match value {
        WorkerProcessState::Idle {
            last_outcome:
                Some(JobOutcome::Success {
                    job_kind:
                        ActiveJobKind::StartPrimary
                        | ActiveJobKind::StartDetachedStandby
                        | ActiveJobKind::StartReplica,
                    finished_at,
                    ..
                }),
            ..
        } => Some(*finished_at),
        _ => None,
    }
}

pub(crate) fn wal_position(
    timeline: Option<TimelineId>,
    lsn: Option<WalLsn>,
) -> Option<WalPosition> {
    match (timeline, lsn) {
        (Some(timeline), Some(lsn)) => Some(WalPosition {
            timeline: u64::from(timeline.0),
            lsn: lsn.0,
        }),
        _ => None,
    }
}

fn failure_recovery_from_job(value: &ActiveJobKind) -> FailureRecovery {
    match value {
        ActiveJobKind::PgRewind => FailureRecovery::FallbackToBasebackup,
        ActiveJobKind::BaseBackup | ActiveJobKind::Bootstrap => FailureRecovery::WaitForOperator,
        ActiveJobKind::Promote
        | ActiveJobKind::Demote
        | ActiveJobKind::StartPrimary
        | ActiveJobKind::StartDetachedStandby
        | ActiveJobKind::StartReplica => {
            FailureRecovery::RetrySameJob
        }
    }
}
