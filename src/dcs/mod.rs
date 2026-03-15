// Stub implementation – see DESIGN.md for the planned interface.
#![allow(clippy::unimplemented)]

pub(crate) mod startup;

use serde::{Deserialize, Serialize};

use crate::{
    pginfo::state::Readiness,
    state::{
        LeaseEpoch, MemberId, ObservedWalPosition, PgTcpTarget, SwitchoverTarget,
        SystemIdentifier, TimelineId,
    },
};

// ---------------------------------------------------------------------------
// DcsMode
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DcsMode {
    NotTrusted,
    Degraded,
    Coordinated,
}

// ---------------------------------------------------------------------------
// DcsView  (interface-only – no inner data)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DcsView {
    NotTrusted,
    Degraded,
    Coordinated,
}

impl DcsView {
    pub fn mode(&self) -> DcsMode {
        unimplemented!("dcs module not implemented")
    }

    pub fn observed_leadership(&self) -> Option<&LeaseEpoch> {
        unimplemented!("dcs module not implemented")
    }

    pub fn cluster(&self) -> Option<&ClusterView> {
        unimplemented!("dcs module not implemented")
    }

    pub fn is_coordinated(&self) -> bool {
        unimplemented!("dcs module not implemented")
    }

    #[cfg(any(test, feature = "internal-test-support"))]
    pub(crate) fn starting() -> Self {
        Self::NotTrusted
    }
}

// ---------------------------------------------------------------------------
// DcsHandle  (interface-only – no content)
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub(crate) struct DcsHandle;

#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum DcsHandleError {
    #[error("dcs command channel closed")]
    ChannelClosed,
}

impl DcsHandle {
    #[cfg(any(test, feature = "internal-test-support"))]
    pub(crate) fn closed() -> Self {
        Self
    }

    pub(crate) fn acquire_leadership(&self) -> Result<(), DcsHandleError> {
        unimplemented!("dcs module not implemented")
    }

    pub(crate) fn release_leadership(&self) -> Result<(), DcsHandleError> {
        unimplemented!("dcs module not implemented")
    }

    pub(crate) fn publish_switchover(
        &self,
        _target: SwitchoverTarget,
    ) -> Result<(), DcsHandleError> {
        unimplemented!("dcs module not implemented")
    }

    pub(crate) fn clear_switchover(&self) -> Result<(), DcsHandleError> {
        unimplemented!("dcs module not implemented")
    }
}

// ---------------------------------------------------------------------------
// ClusterView  (empty struct – methods stubbed)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterView;

impl ClusterView {
    pub fn members(&self) -> impl Iterator<Item = (&MemberId, &ClusterMemberView)> {
        std::iter::empty()
    }

    pub fn member_ids(&self) -> impl Iterator<Item = &MemberId> {
        std::iter::empty()
    }

    pub fn member_count(&self) -> usize {
        unimplemented!("dcs module not implemented")
    }

    pub fn member(&self, _member_id: &MemberId) -> Option<&ClusterMemberView> {
        unimplemented!("dcs module not implemented")
    }

    pub fn leadership(&self) -> &LeadershipObservation {
        unimplemented!("dcs module not implemented")
    }

    pub fn switchover(&self) -> &SwitchoverView {
        unimplemented!("dcs module not implemented")
    }
}

// ---------------------------------------------------------------------------
// ClusterMemberView  (empty struct – methods stubbed)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterMemberView;

impl ClusterMemberView {
    pub fn postgres_target(&self) -> &PgTcpTarget {
        unimplemented!("dcs module not implemented")
    }

    pub fn postgres(&self) -> &MemberPostgresView {
        unimplemented!("dcs module not implemented")
    }
}

// ---------------------------------------------------------------------------
// MemberPostgresView  (variants kept – externally pattern-matched)
// ---------------------------------------------------------------------------

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
    pub fn readiness(&self) -> Readiness {
        unimplemented!("dcs module not implemented")
    }

    pub fn system_identifier(&self) -> Option<SystemIdentifier> {
        unimplemented!("dcs module not implemented")
    }

    pub fn timeline(&self) -> Option<TimelineId> {
        unimplemented!("dcs module not implemented")
    }

    pub fn is_primary(&self) -> bool {
        unimplemented!("dcs module not implemented")
    }

    pub fn is_ready_replica(&self) -> bool {
        unimplemented!("dcs module not implemented")
    }

    pub fn is_ready_non_primary(&self) -> bool {
        unimplemented!("dcs module not implemented")
    }

    pub fn committed_wal(&self) -> Option<&ObservedWalPosition> {
        unimplemented!("dcs module not implemented")
    }

    pub fn replay_wal(&self) -> Option<&ObservedWalPosition> {
        unimplemented!("dcs module not implemented")
    }

    pub fn follow_wal(&self) -> Option<&ObservedWalPosition> {
        unimplemented!("dcs module not implemented")
    }

    pub fn upstream(&self) -> Option<&MemberId> {
        unimplemented!("dcs module not implemented")
    }
}

// ---------------------------------------------------------------------------
// LeadershipObservation
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeadershipObservation {
    Open,
    Held(LeaseEpoch),
}

impl LeadershipObservation {
    pub fn held(&self) -> Option<&LeaseEpoch> {
        unimplemented!("dcs module not implemented")
    }
}

// ---------------------------------------------------------------------------
// SwitchoverView  (variants kept – externally pattern-matched)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "state", content = "target")]
pub enum SwitchoverView {
    None,
    Requested(SwitchoverTarget),
}
