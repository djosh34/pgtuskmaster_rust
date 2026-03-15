use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    pginfo::state::Readiness,
    state::{
        LeaseEpoch, MemberId, ObservedWalPosition, PgTcpTarget, SwitchoverTarget, SystemIdentifier,
        TimelineId,
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DcsMode {
    NotTrusted,
    Degraded,
    Coordinated,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DcsView {
    NotTrusted(ClusterView),
    Degraded(ClusterView),
    Coordinated(ClusterView),
}

impl DcsView {
    pub fn mode(&self) -> DcsMode {
        match self {
            Self::NotTrusted(_) => DcsMode::NotTrusted,
            Self::Degraded(_) => DcsMode::Degraded,
            Self::Coordinated(_) => DcsMode::Coordinated,
        }
    }

    pub fn observed_leadership(&self) -> Option<&LeaseEpoch> {
        self.cluster().leadership().held()
    }

    pub fn cluster(&self) -> &ClusterView {
        match self {
            Self::NotTrusted(cluster)
            | Self::Degraded(cluster)
            | Self::Coordinated(cluster) => cluster,
        }
    }

    pub fn is_coordinated(&self) -> bool {
        matches!(self, Self::Coordinated(_))
    }

    pub(crate) fn starting() -> Self {
        Self::NotTrusted(ClusterView {
            members: BTreeMap::new(),
            leadership: LeadershipObservation::Open,
            switchover: SwitchoverView::None,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterView {
    members: BTreeMap<MemberId, ClusterMemberView>,
    leadership: LeadershipObservation,
    switchover: SwitchoverView,
}

impl ClusterView {
    pub fn members(&self) -> impl Iterator<Item = (&MemberId, &ClusterMemberView)> {
        self.members.iter()
    }

    pub fn member_ids(&self) -> impl Iterator<Item = &MemberId> {
        self.members.keys()
    }

    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    pub fn member(&self, member_id: &MemberId) -> Option<&ClusterMemberView> {
        self.members.get(member_id)
    }

    pub fn leadership(&self) -> &LeadershipObservation {
        &self.leadership
    }

    pub fn switchover(&self) -> &SwitchoverView {
        &self.switchover
    }

    #[cfg(any(test, feature = "internal-test-support"))]
    #[allow(dead_code)]
    pub(crate) fn new(
        members: BTreeMap<MemberId, ClusterMemberView>,
        leadership: LeadershipObservation,
        switchover: SwitchoverView,
    ) -> Self {
        Self {
            members,
            leadership,
            switchover,
        }
    }

    pub(super) fn from_parts(
        members: BTreeMap<MemberId, ClusterMemberView>,
        leadership: LeadershipObservation,
        switchover: SwitchoverView,
    ) -> Self {
        Self {
            members,
            leadership,
            switchover,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterMemberView {
    postgres: MemberPostgresView,
    postgres_target: PgTcpTarget,
}

impl ClusterMemberView {
    pub fn postgres_target(&self) -> &PgTcpTarget {
        &self.postgres_target
    }

    pub fn postgres(&self) -> &MemberPostgresView {
        &self.postgres
    }

    #[cfg(any(test, feature = "internal-test-support"))]
    #[allow(dead_code)]
    pub(crate) fn new(postgres: MemberPostgresView, postgres_target: PgTcpTarget) -> Self {
        Self {
            postgres,
            postgres_target,
        }
    }

    pub(super) fn from_parts(postgres: MemberPostgresView, postgres_target: PgTcpTarget) -> Self {
        Self {
            postgres,
            postgres_target,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MemberPostgresView {
    Unknown {
        readiness: Readiness,
        timeline: Option<TimelineId>,
        system_identifier: Option<SystemIdentifier>,
    },
    Primary {
        readiness: Readiness,
        system_identifier: Option<SystemIdentifier>,
        committed_wal: ObservedWalPosition,
    },
    Replica {
        readiness: Readiness,
        system_identifier: Option<SystemIdentifier>,
        upstream: Option<MemberId>,
        replay_wal: Option<ObservedWalPosition>,
        follow_wal: Option<ObservedWalPosition>,
    },
}

impl MemberPostgresView {
    pub fn readiness(&self) -> &Readiness {
        match self {
            Self::Unknown { readiness, .. }
            | Self::Primary { readiness, .. }
            | Self::Replica { readiness, .. } => readiness,
        }
    }

    pub fn system_identifier(&self) -> Option<SystemIdentifier> {
        match self {
            Self::Unknown {
                system_identifier, ..
            }
            | Self::Primary {
                system_identifier, ..
            }
            | Self::Replica {
                system_identifier, ..
            } => *system_identifier,
        }
    }

    pub fn timeline(&self) -> Option<TimelineId> {
        match self {
            Self::Unknown { timeline, .. } => *timeline,
            Self::Primary { committed_wal, .. } => committed_wal.timeline,
            Self::Replica {
                replay_wal,
                follow_wal,
                ..
            } => replay_wal
                .as_ref()
                .map(|position| position.timeline)
                .or_else(|| follow_wal.as_ref().map(|position| position.timeline))
                .flatten(),
        }
    }

    pub fn is_primary(&self) -> bool {
        matches!(self, Self::Primary { .. })
    }

    pub fn is_ready_replica(&self) -> bool {
        matches!(
            self,
            Self::Replica {
                readiness: Readiness::Ready,
                ..
            }
        )
    }

    pub fn is_ready_non_primary(&self) -> bool {
        matches!(
            self,
            Self::Unknown {
                readiness: Readiness::Ready,
                ..
            }
                | Self::Replica {
                    readiness: Readiness::Ready,
                    ..
                }
        )
    }

    pub fn committed_wal(&self) -> Option<&ObservedWalPosition> {
        match self {
            Self::Primary { committed_wal, .. } => Some(committed_wal),
            Self::Unknown { .. } | Self::Replica { .. } => None,
        }
    }

    pub fn replay_wal(&self) -> Option<&ObservedWalPosition> {
        match self {
            Self::Replica { replay_wal, .. } => replay_wal.as_ref(),
            Self::Unknown { .. } | Self::Primary { .. } => None,
        }
    }

    pub fn follow_wal(&self) -> Option<&ObservedWalPosition> {
        match self {
            Self::Replica { follow_wal, .. } => follow_wal.as_ref(),
            Self::Unknown { .. } | Self::Primary { .. } => None,
        }
    }

    pub fn upstream(&self) -> Option<&MemberId> {
        match self {
            Self::Replica { upstream, .. } => upstream.as_ref(),
            Self::Unknown { .. } | Self::Primary { .. } => None,
        }
    }

    /// Downgrade a Primary to Unknown, preserving timeline/identity.
    /// Used when a member claims primary but is not the authoritative leader.
    pub(super) fn downgrade_to_unknown(&self) -> Self {
        match self {
            Self::Primary {
                readiness,
                system_identifier,
                committed_wal,
            } => Self::Unknown {
                readiness: readiness.clone(),
                timeline: committed_wal.timeline,
                system_identifier: *system_identifier,
            },
            other => other.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeadershipObservation {
    Open,
    Held(LeaseEpoch),
}

impl LeadershipObservation {
    pub fn held(&self) -> Option<&LeaseEpoch> {
        match self {
            Self::Open => None,
            Self::Held(epoch) => Some(epoch),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "state", content = "target")]
pub enum SwitchoverView {
    None,
    Requested(SwitchoverTarget),
}
