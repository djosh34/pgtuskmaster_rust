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
    pub leader: Option<String>,
    pub switchover_pending: bool,
    pub switchover_to: Option<String>,
    pub member_count: usize,
    pub members: Vec<HaClusterMemberResponse>,
    pub dcs_trust: DcsTrustResponse,
    pub cluster_mode: ClusterModeResponse,
    pub desired_state: DesiredNodeStateResponse,
    pub ha_tick: u64,
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
    pub updated_at_ms: u64,
    pub pg_version: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DcsTrustResponse {
    FreshQuorum,
    NoFreshQuorum,
    NotTrusted,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ClusterModeResponse {
    DcsUnavailable,
    UninitializedNoBootstrapOwner,
    UninitializedBootstrapInProgress { holder: String },
    InitializedLeaderPresent { leader: String },
    InitializedNoLeaderFreshQuorum,
    InitializedNoLeaderNoFreshQuorum,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DesiredNodeStateResponse {
    Bootstrap { plan: BootstrapPlanResponse },
    Primary { plan: PrimaryPlanResponse },
    Replica { plan: ReplicaPlanResponse },
    Quiescent { reason: QuiescentReasonResponse },
    Fence { plan: FencePlanResponse },
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BootstrapPlanResponse {
    InitDb,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrimaryPlanResponse {
    KeepLeader,
    AcquireLeaderThenResumePrimary,
    AcquireLeaderThenPromote,
    AcquireLeaderThenStartPrimary,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ReplicaPlanResponse {
    DirectFollow { leader_member_id: String },
    RewindThenFollow { leader_member_id: String },
    BasebackupThenFollow { leader_member_id: String },
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuiescentReasonResponse {
    WaitingForBootstrapWinner,
    WaitingForAuthoritativeLeader,
    WaitingForFreshQuorum,
    WaitingForAuthoritativeClusterState,
    WaitingForRecoveryPreconditions,
    UnsafeUninitializedPgData,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FencePlanResponse {
    StopAndStayNonWritable,
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
            Self::FreshQuorum => "fresh_quorum",
            Self::NoFreshQuorum => "no_fresh_quorum",
            Self::NotTrusted => "not_trusted",
        }
    }
}

impl ClusterModeResponse {
    fn as_str(&self) -> &'static str {
        match self {
            Self::DcsUnavailable => "dcs_unavailable",
            Self::UninitializedNoBootstrapOwner => "uninitialized_no_bootstrap_owner",
            Self::UninitializedBootstrapInProgress { .. } => "uninitialized_bootstrap_in_progress",
            Self::InitializedLeaderPresent { .. } => "initialized_leader_present",
            Self::InitializedNoLeaderFreshQuorum => "initialized_no_leader_fresh_quorum",
            Self::InitializedNoLeaderNoFreshQuorum => "initialized_no_leader_no_fresh_quorum",
        }
    }
}

impl fmt::Display for DcsTrustResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Display for ClusterModeResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
