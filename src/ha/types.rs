use serde::{Deserialize, Serialize};

use crate::{
    dcs::DcsSnapshot,
    pginfo::state::PgInfoState,
    process::{
        jobs::{ProcessIntent, ShutdownMode},
        state::ProcessState,
    },
    state::{LeaseEpoch, MemberId, ObservedWalPosition},
};

pub use crate::state::SwitchoverState;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FenceCutoff {
    pub epoch: LeaseEpoch,
    pub committed_lsn: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocalDataState {
    Missing,
    BootstrapEmpty,
    ConsistentReplica,
    DivergedRewind,
    DivergedBasebackup,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CandidateState {
    Ineligible,
    Bootstrap,
    Promote(ObservedWalPosition),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadyPrimary {
    pub member: MemberId,
    pub timeline: Option<u64>,
    pub system_identifier: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HaObservation {
    pub pg: PgInfoState,
    pub process: ProcessState,
    pub dcs: DcsSnapshot,
    pub publication: PublicationState,
    pub managed_roles_reconciled: bool,
    pub local_data: LocalDataState,
    pub resolved_upstream: Option<MemberId>,
    pub self_candidate: CandidateState,
    pub storage_stalled: bool,
    pub ready_primary: Option<ReadyPrimary>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HaDecision {
    pub mode: HaMode,
    pub publication: Option<AuthorityProjection>,
    pub clear_switchover: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HaMode {
    Lead(LeaseEpoch),
    AcquireLease(LeaseClaim),
    Follow {
        leader: MemberId,
        recovery: FollowRecovery,
    },
    FailsafeStop {
        shutdown: ShutdownMode,
        cutoff: Option<FenceCutoff>,
    },
    FailsafeKeepFollowing {
        leader: Option<MemberId>,
    },
    WaitForQuorum,
    WaitForLeader,
    WaitForTarget(MemberId),
    DemoteForSwitchover(MemberId),
    Fence {
        release_lease: bool,
        shutdown: Option<ShutdownMode>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeaseClaim {
    Bootstrap,
    Failover,
    ResumeAfterOutage,
    TargetedSwitchover(MemberId),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FollowRecovery {
    None,
    StartStreaming,
    Rewind,
    Basebackup,
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
    NoQuorum {
        fence: NoPrimaryFence,
    },
    LeaseOpen,
    Recovering {
        epoch: Option<LeaseEpoch>,
        fence: NoPrimaryFence,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NoPrimaryFence {
    None,
    Cutoff(FenceCutoff),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HaStep {
    Publish(AuthorityProjection),
    AcquireLease(LeaseClaim),
    ReleaseLease,
    ClearSwitchover,
    ReconcileManagedRoles,
    RunProcess(ProcessIntent),
}

pub type HaPlan = Vec<HaStep>;

impl CandidateState {
    pub(crate) fn is_eligible(&self) -> bool {
        !matches!(self, Self::Ineligible)
    }
}

impl HaObservation {
    pub(crate) fn initial() -> Self {
        Self {
            pg: PgInfoState::starting(),
            process: ProcessState::starting(),
            dcs: DcsSnapshot::starting(),
            publication: PublicationState::unknown(),
            managed_roles_reconciled: false,
            local_data: LocalDataState::Missing,
            resolved_upstream: None,
            self_candidate: CandidateState::Ineligible,
            storage_stalled: false,
            ready_primary: None,
        }
    }
}

impl HaDecision {
    pub(crate) fn initial() -> Self {
        Self {
            mode: HaMode::WaitForLeader,
            publication: None,
            clear_switchover: false,
        }
    }
}

impl PublicationState {
    pub(crate) fn unknown() -> Self {
        Self::Unknown
    }
}
