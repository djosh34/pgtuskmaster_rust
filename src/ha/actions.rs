#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ActionId {
    AcquireLeaderLease,
    ReleaseLeaderLease,
    FollowLeader(String),
    StartRewind,
    RunBootstrap,
    FenceNode,
    SignalFailSafe,
    StartPostgres,
    PromoteToPrimary,
    DemoteToReplica,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum HaAction {
    AcquireLeaderLease,
    ReleaseLeaderLease,
    FollowLeader { leader_member_id: String },
    StartRewind,
    RunBootstrap,
    FenceNode,
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
            Self::FollowLeader { leader_member_id } => {
                ActionId::FollowLeader(leader_member_id.clone())
            }
            Self::StartRewind => ActionId::StartRewind,
            Self::RunBootstrap => ActionId::RunBootstrap,
            Self::FenceNode => ActionId::FenceNode,
            Self::SignalFailSafe => ActionId::SignalFailSafe,
            Self::StartPostgres => ActionId::StartPostgres,
            Self::PromoteToPrimary => ActionId::PromoteToPrimary,
            Self::DemoteToReplica => ActionId::DemoteToReplica,
        }
    }
}
