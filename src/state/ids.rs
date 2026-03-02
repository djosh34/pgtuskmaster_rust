#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MemberId(pub String);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ClusterName(pub String);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SwitchoverRequestId(pub String);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct JobId(pub String);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WalLsn(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TimelineId(pub u32);
