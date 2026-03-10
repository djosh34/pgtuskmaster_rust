use crate::state::MemberId;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ActionId {
    AcquireLeaderLease,
    ReleaseLeaderLease,
    ClearSwitchover,
    StartRewind,
    StartBaseBackup,
    RunBootstrap,
    FenceNode,
    WipeDataDir,
    StartPrimary,
    StartReplica(String),
    PromoteToPrimary,
    DemoteToReplica,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum HaAction {
    AcquireLeaderLease,
    ReleaseLeaderLease,
    ClearSwitchover,
    StartRewind { leader_member_id: MemberId },
    StartBaseBackup { leader_member_id: MemberId },
    RunBootstrap,
    FenceNode,
    WipeDataDir,
    StartPrimary,
    StartReplica { leader_member_id: MemberId },
    PromoteToPrimary,
    DemoteToReplica,
}

impl HaAction {
    pub(crate) fn id(&self) -> ActionId {
        match self {
            Self::AcquireLeaderLease => ActionId::AcquireLeaderLease,
            Self::ReleaseLeaderLease => ActionId::ReleaseLeaderLease,
            Self::ClearSwitchover => ActionId::ClearSwitchover,
            Self::StartRewind { .. } => ActionId::StartRewind,
            Self::StartBaseBackup { .. } => ActionId::StartBaseBackup,
            Self::RunBootstrap => ActionId::RunBootstrap,
            Self::FenceNode => ActionId::FenceNode,
            Self::WipeDataDir => ActionId::WipeDataDir,
            Self::StartPrimary => ActionId::StartPrimary,
            Self::StartReplica { leader_member_id } => {
                ActionId::StartReplica(leader_member_id.0.clone())
            }
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
            Self::StartRewind => "start_rewind".to_string(),
            Self::StartBaseBackup => "start_basebackup".to_string(),
            Self::RunBootstrap => "run_bootstrap".to_string(),
            Self::FenceNode => "fence_node".to_string(),
            Self::WipeDataDir => "wipe_data_dir".to_string(),
            Self::StartPrimary => "start_primary".to_string(),
            Self::StartReplica(leader) => format!("start_replica_from_{leader}"),
            Self::PromoteToPrimary => "promote_to_primary".to_string(),
            Self::DemoteToReplica => "demote_to_replica".to_string(),
        }
    }
}
