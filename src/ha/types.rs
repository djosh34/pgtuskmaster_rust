use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    dcs::state::DcsTrust,
    process::{
        jobs::{ActiveJobKind, ShutdownMode as ProcessShutdownMode},
        state::{JobOutcome, ProcessState as WorkerProcessState},
    },
    state::{MemberId, TimelineId, UnixMillis, WalLsn},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct LeaseEpoch {
    pub(crate) holder: MemberId,
    pub(crate) generation: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FenceCutoff {
    pub(crate) epoch: LeaseEpoch,
    pub(crate) committed_lsn: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WorldView {
    pub(crate) local: LocalKnowledge,
    pub(crate) global: GlobalKnowledge,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LocalKnowledge {
    pub(crate) data_dir: DataDirState,
    pub(crate) postgres: PostgresState,
    pub(crate) process: ProcessState,
    pub(crate) storage: StorageState,
    pub(crate) publication: PublicationState,
    pub(crate) observation: ObservationState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ObservationState {
    pub(crate) pg_observed_at: UnixMillis,
    pub(crate) last_start_success_at: Option<UnixMillis>,
    pub(crate) last_promote_success_at: Option<UnixMillis>,
    pub(crate) last_demote_success_at: Option<UnixMillis>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum DataDirState {
    Missing,
    Initialized(LocalDataState),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum LocalDataState {
    BootstrapEmpty,
    ConsistentReplica,
    Diverged(DivergenceState),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum DivergenceState {
    RewindPossible,
    BasebackupRequired,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum PostgresState {
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
pub(crate) enum ReplicationState {
    Streaming(WalPosition),
    CatchingUp(WalPosition),
    Stalled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum ProcessState {
    Idle,
    Running(JobKind),
    Failed(JobFailure),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct JobFailure {
    pub(crate) job: JobKind,
    pub(crate) recovery: FailureRecovery,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum FailureRecovery {
    RetrySameJob,
    FallbackToBasebackup,
    WaitForOperator,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum StorageState {
    Healthy,
    Stalled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct PublicationState {
    pub(crate) authority: AuthorityView,
    pub(crate) fence_cutoff: Option<FenceCutoff>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum AuthorityView {
    Primary { member: MemberId, epoch: LeaseEpoch },
    NoPrimary(NoPrimaryReason),
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GlobalKnowledge {
    pub(crate) dcs_trust: DcsTrust,
    pub(crate) lease: LeaseState,
    pub(crate) observed_lease: Option<LeaseEpoch>,
    pub(crate) observed_primary: Option<MemberId>,
    pub(crate) switchover: SwitchoverState,
    pub(crate) peers: BTreeMap<MemberId, PeerKnowledge>,
    pub(crate) self_peer: PeerKnowledge,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum LeaseState {
    HeldByMe(LeaseEpoch),
    HeldByPeer(LeaseEpoch),
    Unheld,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum SwitchoverState {
    None,
    Requested(SwitchoverRequest),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SwitchoverRequest {
    pub(crate) target: SwitchoverTarget,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum SwitchoverTarget {
    AnyHealthyReplica,
    Specific(MemberId),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PeerKnowledge {
    pub(crate) election: ElectionEligibility,
    pub(crate) api: ApiVisibility,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum ElectionEligibility {
    BootstrapEligible,
    PromoteEligible(WalPosition),
    Ineligible(IneligibleReason),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum IneligibleReason {
    NotReady,
    Lagging,
    Partitioned,
    ApiUnavailable,
    StartingUp,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum ApiVisibility {
    Reachable,
    Unreachable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct DesiredState {
    pub(crate) role: TargetRole,
    pub(crate) publication: PublicationGoal,
    pub(crate) clear_switchover: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum TargetRole {
    Leader(LeaseEpoch),
    Candidate(Candidacy),
    Follower(FollowGoal),
    FailSafe(FailSafeGoal),
    DemotingForSwitchover(MemberId),
    Fenced(FenceReason),
    Idle(IdleReason),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum Candidacy {
    Bootstrap,
    Failover,
    ResumeAfterOutage,
    TargetedSwitchover(MemberId),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FollowGoal {
    pub(crate) leader: MemberId,
    pub(crate) recovery: RecoveryPlan,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum RecoveryPlan {
    None,
    StartStreaming,
    Rewind,
    Basebackup,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum FailSafeGoal {
    PrimaryMustStop(FenceCutoff),
    ReplicaKeepFollowing(Option<MemberId>),
    WaitForQuorum,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum IdleReason {
    AwaitingLeader,
    AwaitingTarget(MemberId),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum FenceReason {
    ForeignLeaderDetected,
    StorageStalled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum PublicationGoal {
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum NoPrimaryReason {
    DcsDegraded,
    LeaseOpen,
    Recovering,
    SwitchoverRejected(SwitchoverBlocker),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum SwitchoverBlocker {
    TargetMissing,
    TargetIneligible(IneligibleReason),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum ReconcileAction {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum ShutdownMode {
    Fast,
    Immediate,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum JobKind {
    InitDb,
    BaseBackup,
    PgRewind,
    StartPrimary,
    StartReplica,
    Promote,
    Demote,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub(crate) struct WalPosition {
    pub(crate) timeline: u64,
    pub(crate) lsn: u64,
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
        Self {
            authority: AuthorityView::Unknown,
            fence_cutoff: None,
        }
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

impl ReconcileAction {
    pub(crate) fn label(&self) -> &'static str {
        match self {
            Self::InitDb => "init_db",
            Self::BaseBackup(_) => "basebackup",
            Self::PgRewind(_) => "pg_rewind",
            Self::StartPrimary => "start_primary",
            Self::StartReplica(_) => "start_replica",
            Self::Promote => "promote",
            Self::Demote(_) => "demote",
            Self::AcquireLease(_) => "acquire_lease",
            Self::ReleaseLease => "release_lease",
            Self::Publish(_) => "publish",
            Self::ClearSwitchover => "clear_switchover",
        }
    }
}

impl ShutdownMode {
    pub(crate) fn to_process_mode(self) -> ProcessShutdownMode {
        match self {
            Self::Fast => ProcessShutdownMode::Fast,
            Self::Immediate => ProcessShutdownMode::Immediate,
        }
    }
}

impl From<&WorkerProcessState> for ProcessState {
    fn from(value: &WorkerProcessState) -> Self {
        match value {
            WorkerProcessState::Running { active, .. } => {
                Self::Running(job_kind_from_active(&active.kind))
            }
            WorkerProcessState::Idle {
                last_outcome: Some(JobOutcome::Failure { job_kind, .. }),
                ..
            }
            | WorkerProcessState::Idle {
                last_outcome: Some(JobOutcome::Timeout { job_kind, .. }),
                ..
            } => Self::Failed(JobFailure {
                job: job_kind_from_active(job_kind),
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

fn job_kind_from_active(value: &ActiveJobKind) -> JobKind {
    match value {
        ActiveJobKind::Bootstrap => JobKind::InitDb,
        ActiveJobKind::BaseBackup => JobKind::BaseBackup,
        ActiveJobKind::PgRewind => JobKind::PgRewind,
        ActiveJobKind::Promote => JobKind::Promote,
        ActiveJobKind::Demote => JobKind::Demote,
        ActiveJobKind::StartPostgres => JobKind::StartPrimary,
    }
}

fn failure_recovery_from_job(value: &ActiveJobKind) -> FailureRecovery {
    match value {
        ActiveJobKind::PgRewind => FailureRecovery::FallbackToBasebackup,
        ActiveJobKind::BaseBackup | ActiveJobKind::Bootstrap => FailureRecovery::WaitForOperator,
        ActiveJobKind::Promote | ActiveJobKind::Demote | ActiveJobKind::StartPostgres => {
            FailureRecovery::RetrySameJob
        }
    }
}
