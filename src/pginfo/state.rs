use crate::state::{MemberId, TimelineId, UnixMillis, WalLsn, WorkerStatus};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SqlStatus {
    Unknown,
    Healthy,
    Unreachable,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Readiness {
    Unknown,
    Ready,
    NotReady,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PgConfig {
    pub(crate) extra: std::collections::BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ReplicationSlotInfo {
    pub(crate) name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UpstreamInfo {
    pub(crate) member_id: MemberId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PgInfoCommon {
    pub(crate) worker: WorkerStatus,
    pub(crate) sql: SqlStatus,
    pub(crate) readiness: Readiness,
    pub(crate) timeline: Option<TimelineId>,
    pub(crate) pg_config: PgConfig,
    pub(crate) last_refresh_at: Option<UnixMillis>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PgInfoState {
    Unknown {
        common: PgInfoCommon,
    },
    Primary {
        common: PgInfoCommon,
        wal_lsn: WalLsn,
        slots: Vec<ReplicationSlotInfo>,
    },
    Replica {
        common: PgInfoCommon,
        replay_lsn: WalLsn,
        follow_lsn: Option<WalLsn>,
        upstream: Option<UpstreamInfo>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PgInfoWorkerCtx {
    pub(crate) self_id: MemberId,
}
