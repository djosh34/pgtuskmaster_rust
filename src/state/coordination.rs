use serde::{Deserialize, Serialize};

use crate::state::{MemberId, TimelineId, WalLsn};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeaseEpoch {
    pub holder: MemberId,
    pub generation: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SwitchoverTarget {
    AnyHealthyReplica,
    Specific(MemberId),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SwitchoverRequest {
    pub requested_from: LeaseEpoch,
    pub target: SwitchoverTarget,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SwitchoverState {
    None,
    Pending(SwitchoverRequest),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservedWalPosition {
    pub timeline: Option<TimelineId>,
    pub lsn: WalLsn,
}

impl SwitchoverTarget {
    pub fn member(&self) -> Option<&MemberId> {
        match self {
            Self::AnyHealthyReplica => None,
            Self::Specific(member_id) => Some(member_id),
        }
    }
}

impl SwitchoverState {
    pub fn request(&self) -> Option<&SwitchoverRequest> {
        match self {
            Self::None => None,
            Self::Pending(request) => Some(request),
        }
    }
}
