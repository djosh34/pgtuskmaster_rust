use serde::{Deserialize, Serialize};

use crate::state::{MemberId, TimelineId, WalLsn};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeaseEpoch {
    pub holder: MemberId,
    pub generation: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SwitchoverState {
    None,
    AnyHealthyReplica,
    Specific(MemberId),
}

pub type SwitchoverTarget = SwitchoverState;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservedWalPosition {
    pub timeline: Option<TimelineId>,
    pub lsn: WalLsn,
}
