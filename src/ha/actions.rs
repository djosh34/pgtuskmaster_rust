use crate::state::MemberId;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ActionId {
    AcquireLeaderLease,
    ReleaseLeaderLease,
    ClearSwitchover,
    FollowLeader(String),
    StartRewind,
    StartBaseBackup,
    RunBootstrap,
    FenceNode,
    WipeDataDir,
    SignalFailSafe,
    StartPostgres,
    PromoteToPrimary,
    DemoteToReplica,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum HaAction {
    AcquireLeaderLease,
    ReleaseLeaderLease,
    ClearSwitchover,
    FollowLeader { leader_member_id: String },
    StartRewind { leader_member_id: MemberId },
    StartBaseBackup { leader_member_id: MemberId },
    RunBootstrap,
    FenceNode,
    WipeDataDir,
    SignalFailSafe,
    StartPostgres,
    PromoteToPrimary,
    DemoteToReplica,
}

impl HaAction {
    pub(crate) fn id(&self) -> ActionId {
        match self {
            Self::AcquireLeaderLease => ActionId::AcquireLeaderLease,
            Self::ReleaseLeaderLease => ActionId::ReleaseLeaderLease,
            Self::ClearSwitchover => ActionId::ClearSwitchover,
            Self::FollowLeader { leader_member_id } => {
                ActionId::FollowLeader(leader_member_id.clone())
            }
            Self::StartRewind { .. } => ActionId::StartRewind,
            Self::StartBaseBackup { .. } => ActionId::StartBaseBackup,
            Self::RunBootstrap => ActionId::RunBootstrap,
            Self::FenceNode => ActionId::FenceNode,
            Self::WipeDataDir => ActionId::WipeDataDir,
            Self::SignalFailSafe => ActionId::SignalFailSafe,
            Self::StartPostgres => ActionId::StartPostgres,
            Self::PromoteToPrimary => ActionId::PromoteToPrimary,
            Self::DemoteToReplica => ActionId::DemoteToReplica,
        }
    }
}

impl ActionId {
    pub(crate) fn label(&self) -> String {
        match self {
            Self::AcquireLeaderLease => "acquire_leader_lease".to_string(),
            Self::ReleaseLeaderLease => "release_leader_lease".to_string(),
            Self::ClearSwitchover => "clear_switchover".to_string(),
            Self::FollowLeader(leader) => format!("follow_leader_{leader}"),
            Self::StartRewind => "start_rewind".to_string(),
            Self::StartBaseBackup => "start_basebackup".to_string(),
            Self::RunBootstrap => "run_bootstrap".to_string(),
            Self::FenceNode => "fence_node".to_string(),
            Self::WipeDataDir => "wipe_data_dir".to_string(),
            Self::SignalFailSafe => "signal_failsafe".to_string(),
            Self::StartPostgres => "start_postgres".to_string(),
            Self::PromoteToPrimary => "promote_to_primary".to_string(),
            Self::DemoteToReplica => "demote_to_replica".to_string(),
        }
    }
}
