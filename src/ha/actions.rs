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

impl HaAction {
    pub(crate) fn id(&self) -> ActionId {
        match self {
            Self::AcquireLeaderLease => ActionId::AcquireLeaderLease,
            Self::ReleaseLeaderLease => ActionId::ReleaseLeaderLease,
            Self::ClearSwitchover => ActionId::ClearSwitchover,
            Self::FollowLeader { leader_member_id } => {
                ActionId::FollowLeader(leader_member_id.clone())
            }
            Self::StartRewind => ActionId::StartRewind,
            Self::StartBaseBackup => ActionId::StartBaseBackup,
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
