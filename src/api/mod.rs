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
    pub ha_phase: HaPhaseResponse,
    pub ha_tick: u64,
    pub ha_decision: HaDecisionResponse,
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
#[serde(rename_all = "snake_case")]
pub enum HaPhaseResponse {
    Init,
    WaitingPostgresReachable,
    WaitingDcsTrusted,
    WaitingSwitchoverSuccessor,
    Replica,
    CandidateLeader,
    Primary,
    Rewinding,
    Bootstrapping,
    Fencing,
    FailSafe,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum HaDecisionResponse {
    NoChange,
    WaitForPostgres {
        start_requested: bool,
        leader_member_id: Option<String>,
    },
    WaitForDcsTrust,
    WaitForPromotionSafety {
        blocker: PromotionSafetyBlockerResponse,
    },
    AttemptLeadership,
    FollowLeader {
        leader_member_id: String,
    },
    BecomePrimary {
        promote: bool,
    },
    CompleteSwitchover,
    StepDown {
        reason: StepDownReasonResponse,
        release_leader_lease: bool,
        fence: bool,
    },
    RecoverReplica {
        strategy: RecoveryStrategyResponse,
    },
    FenceNode,
    ReleaseLeaderLease {
        reason: LeaseReleaseReasonResponse,
    },
    EnterFailSafe {
        release_leader_lease: bool,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StepDownReasonResponse {
    Switchover,
    ForeignLeaderDetected { leader_member_id: String },
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RecoveryStrategyResponse {
    Rewind { leader_member_id: String },
    BaseBackup { leader_member_id: String },
    Bootstrap,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeaseReleaseReasonResponse {
    FencingComplete,
    PostgresUnreachable,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PromotionSafetyBlockerResponse {
    NotHealthyReplica,
    MissingLocalTimeline,
    MissingLocalReplayLsn,
    HigherFreshTimeline {
        required_timeline: u64,
        source_member_id: String,
    },
    LaggingFreshWal {
        timeline: u64,
        required_lsn: u64,
        local_replay_lsn: u64,
        source_member_id: String,
    },
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

impl HaPhaseResponse {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Init => "init",
            Self::WaitingPostgresReachable => "waiting_postgres_reachable",
            Self::WaitingDcsTrusted => "waiting_dcs_trusted",
            Self::WaitingSwitchoverSuccessor => "waiting_switchover_successor",
            Self::Replica => "replica",
            Self::CandidateLeader => "candidate_leader",
            Self::Primary => "primary",
            Self::Rewinding => "rewinding",
            Self::Bootstrapping => "bootstrapping",
            Self::Fencing => "fencing",
            Self::FailSafe => "fail_safe",
        }
    }

    fn legacy_label(&self) -> &'static str {
        match self {
            Self::Init => "Init",
            Self::WaitingPostgresReachable => "WaitingPostgresReachable",
            Self::WaitingDcsTrusted => "WaitingDcsTrusted",
            Self::WaitingSwitchoverSuccessor => "WaitingSwitchoverSuccessor",
            Self::Replica => "Replica",
            Self::CandidateLeader => "CandidateLeader",
            Self::Primary => "Primary",
            Self::Rewinding => "Rewinding",
            Self::Bootstrapping => "Bootstrapping",
            Self::Fencing => "Fencing",
            Self::FailSafe => "FailSafe",
        }
    }
}

impl fmt::Display for DcsTrustResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Display for HaPhaseResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Display for StepDownReasonResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Switchover => f.write_str("switchover"),
            Self::ForeignLeaderDetected { leader_member_id } => {
                write!(f, "foreign_leader_detected({leader_member_id})")
            }
        }
    }
}

impl fmt::Display for RecoveryStrategyResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rewind { leader_member_id } => write!(f, "rewind({leader_member_id})"),
            Self::BaseBackup { leader_member_id } => {
                write!(f, "base_backup({leader_member_id})")
            }
            Self::Bootstrap => f.write_str("bootstrap"),
        }
    }
}

impl fmt::Display for LeaseReleaseReasonResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FencingComplete => f.write_str("fencing_complete"),
            Self::PostgresUnreachable => f.write_str("postgres_unreachable"),
        }
    }
}

impl fmt::Display for PromotionSafetyBlockerResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotHealthyReplica => f.write_str("not_healthy_replica"),
            Self::MissingLocalTimeline => f.write_str("missing_local_timeline"),
            Self::MissingLocalReplayLsn => f.write_str("missing_local_replay_lsn"),
            Self::HigherFreshTimeline {
                required_timeline,
                source_member_id,
            } => write!(
                f,
                "higher_fresh_timeline(required_timeline={required_timeline}, source_member_id={source_member_id})"
            ),
            Self::LaggingFreshWal {
                timeline,
                required_lsn,
                local_replay_lsn,
                source_member_id,
            } => write!(
                f,
                "lagging_fresh_wal(timeline={timeline}, required_lsn={required_lsn}, local_replay_lsn={local_replay_lsn}, source_member_id={source_member_id})"
            ),
        }
    }
}

impl fmt::Display for MemberRoleResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown => f.write_str("unknown"),
            Self::Primary => f.write_str("primary"),
            Self::Replica => f.write_str("replica"),
        }
    }
}

impl fmt::Display for SqlStatusResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown => f.write_str("unknown"),
            Self::Healthy => f.write_str("healthy"),
            Self::Unreachable => f.write_str("unreachable"),
        }
    }
}

impl fmt::Display for ReadinessResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown => f.write_str("unknown"),
            Self::Ready => f.write_str("ready"),
            Self::NotReady => f.write_str("not_ready"),
        }
    }
}

impl PartialEq<&str> for HaPhaseResponse {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other || self.legacy_label() == *other
    }
}
