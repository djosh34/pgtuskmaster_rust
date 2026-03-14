use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    dcs::DcsTrust,
    process::{
        jobs::{ActiveJobKind, ShutdownMode as ProcessShutdownMode},
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
    Running(JobKind),
    Failed(JobFailure),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobFailure {
    pub job: JobKind,
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
pub struct PublicationState {
    pub authority: AuthorityView,
    pub fence_cutoff: Option<FenceCutoff>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthorityView {
    Primary { member: MemberId, epoch: LeaseEpoch },
    NoPrimary(NoPrimaryReason),
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalKnowledge {
    pub dcs_trust: DcsTrust,
    pub lease: LeaseState,
    pub observed_lease: Option<LeaseEpoch>,
    pub observed_primary: Option<MemberId>,
    pub coordination: CoordinationView,
    pub switchover: SwitchoverState,
    pub peers: BTreeMap<MemberId, PeerKnowledge>,
    pub self_peer: PeerKnowledge,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoordinationView {
    pub trust: DcsTrust,
    pub leader: LeaseState,
    pub sampled_primary: Option<MemberId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeaseState {
    HeldByMe(LeaseEpoch),
    HeldByPeer(LeaseEpoch),
    Unheld,
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReconcileAction {
    InitDb,
    BaseBackup(MemberId),
    PgRewind(MemberId),
    StartPrimary,
    StartDetachedStandby,
    StartReplica(MemberId),
    Promote,
    Demote(ShutdownMode),
    AcquireLease(Candidacy),
    ReleaseLease,
    EnsureRequiredRoles,
    Publish(PublicationGoal),
    ClearSwitchover,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShutdownMode {
    Fast,
    Immediate,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobKind {
    InitDb,
    BaseBackup,
    PgRewind,
    StartPrimary,
    StartReplica,
    Promote,
    Demote,
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
                dcs_trust: DcsTrust::NotTrusted,
                lease: LeaseState::Unheld,
                observed_lease: None,
                observed_primary: None,
                coordination: CoordinationView {
                    trust: DcsTrust::NotTrusted,
                    leader: LeaseState::Unheld,
                    sampled_primary: None,
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
            Self::StartDetachedStandby => "start_detached_standby",
            Self::StartReplica(_) => "start_replica",
            Self::Promote => "promote",
            Self::Demote(_) => "demote",
            Self::AcquireLease(_) => "acquire_lease",
            Self::ReleaseLease => "release_lease",
            Self::EnsureRequiredRoles => "ensure_required_roles",
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
