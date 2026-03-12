use std::fmt;

use thiserror::Error;

pub(crate) mod controller;
pub(crate) mod fallback;
pub mod worker;

#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub(crate) enum ApiError {
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("dcs store error: {0}")]
    DcsStore(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl ApiError {
    pub(crate) fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest(message.into())
    }

    pub(crate) fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }
}

pub(crate) type ApiResult<T> = Result<T, ApiError>;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AcceptedResponse {
    pub accepted: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HaStateResponse {
    pub cluster_name: String,
    pub scope: String,
    pub self_member_id: String,
    pub leader_lease_holder: Option<String>,
    pub switchover: Option<SwitchoverIntentResponse>,
    pub member_slot_count: usize,
    pub member_slots: Vec<HaClusterMemberResponse>,
    pub dcs_trust: DcsTrustResponse,
    pub authority_projection: AuthorityProjectionResponse,
    pub fence_cutoff: Option<FenceCutoffResponse>,
    pub role_intent: RoleIntentResponse,
    pub ha_tick: u64,
    pub planned_commands: Vec<HaCommandResponse>,
    pub snapshot_sequence: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HaClusterMemberResponse {
    pub member_id: String,
    pub postgres_host: String,
    pub postgres_port: u16,
    pub api_url: Option<String>,
    pub role: MemberRoleResponse,
    pub sql: SqlStatusResponse,
    pub readiness: ReadinessResponse,
    pub timeline: Option<u64>,
    pub write_lsn: Option<u64>,
    pub replay_lsn: Option<u64>,
    pub pg_version: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DcsTrustResponse {
    FullQuorum,
    Degraded,
    NotTrusted,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AuthorityProjectionResponse {
    Primary {
        member_id: String,
        epoch: LeaseEpochResponse,
    },
    NoPrimary {
        reason: NoPrimaryReasonResponse,
    },
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LeaseEpochResponse {
    pub holder: String,
    pub generation: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FenceCutoffResponse {
    pub epoch: LeaseEpochResponse,
    pub committed_lsn: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SwitchoverIntentResponse {
    AnyHealthyReplica,
    Specific { member_id: String },
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NoPrimaryReasonResponse {
    DcsDegraded,
    LeaseOpen,
    Recovering,
    SwitchoverRejected { blocker: SwitchoverBlockerResponse },
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SwitchoverBlockerResponse {
    TargetMissing,
    TargetIneligible { reason: IneligibleReasonResponse },
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RoleIntentResponse {
    Leader { epoch: LeaseEpochResponse },
    Candidate { candidacy: CandidacyResponse },
    Follower { goal: FollowGoalResponse },
    FailSafe { goal: FailSafeGoalResponse },
    DemotingForSwitchover { member_id: String },
    Fenced { reason: FenceReasonResponse },
    Idle { reason: IdleReasonResponse },
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CandidacyResponse {
    Bootstrap,
    Failover,
    ResumeAfterOutage,
    TargetedSwitchover { member_id: String },
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FollowGoalResponse {
    pub leader: String,
    pub recovery: RecoveryPlanResponse,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryPlanResponse {
    None,
    StartStreaming,
    Rewind,
    Basebackup,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FailSafeGoalResponse {
    PrimaryMustStop { cutoff: FenceCutoffResponse },
    ReplicaKeepFollowing { upstream: Option<String> },
    WaitForQuorum,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum IdleReasonResponse {
    AwaitingLeader,
    AwaitingTarget { member_id: String },
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FenceReasonResponse {
    ForeignLeaderDetected,
    StorageStalled,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum HaCommandResponse {
    InitDb,
    BaseBackup { member_id: String },
    PgRewind { member_id: String },
    StartPrimary,
    StartDetachedStandby,
    StartReplica { member_id: String },
    Promote,
    Demote { mode: ShutdownModeResponse },
    AcquireLease { candidacy: CandidacyResponse },
    ReleaseLease,
    EnsureRequiredRoles,
    Publish { projection: AuthorityProjectionResponse },
    ClearSwitchover,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShutdownModeResponse {
    Fast,
    Immediate,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IneligibleReasonResponse {
    NotReady,
    Lagging,
    Partitioned,
    ApiUnavailable,
    StartingUp,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemberRoleResponse {
    Unknown,
    Primary,
    Replica,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SqlStatusResponse {
    Unknown,
    Healthy,
    Unreachable,
}

pub type HaAuthorityResponse = AuthorityProjectionResponse;
pub type TargetRoleResponse = RoleIntentResponse;
pub type ReconcileActionResponse = HaCommandResponse;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadinessResponse {
    Unknown,
    Ready,
    NotReady,
}

impl DcsTrustResponse {
    fn as_str(&self) -> &'static str {
        match self {
            Self::FullQuorum => "full_quorum",
            Self::Degraded => "degraded",
            Self::NotTrusted => "not_trusted",
        }
    }
}

impl fmt::Display for DcsTrustResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
