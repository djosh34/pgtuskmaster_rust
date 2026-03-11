use std::time::Duration;

use serde::{Deserialize, Serialize};

pub(crate) use super::conninfo::{render_pg_conninfo, PgConnInfo, PgSslMode};
use super::query::PgPollData;
use crate::logging::LogHandle;
use crate::state::StatePublisher;
use crate::state::{MemberId, TimelineId, UnixMillis, WalLsn, WorkerStatus};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum SqlStatus {
    Unknown,
    Healthy,
    Unreachable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum Readiness {
    Unknown,
    Ready,
    NotReady,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PgConfig {
    pub(crate) port: Option<u16>,
    pub(crate) hot_standby: Option<bool>,
    pub(crate) primary_conninfo: Option<PgConnInfo>,
    pub(crate) primary_slot_name: Option<String>,
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

#[derive(Clone, Debug)]
pub(crate) struct PgInfoWorkerCtx {
    pub(crate) self_id: MemberId,
    pub(crate) postgres_conninfo: PgConnInfo,
    pub(crate) poll_interval: Duration,
    pub(crate) publisher: StatePublisher<PgInfoState>,
    pub(crate) log: LogHandle,
    pub(crate) last_emitted_sql_status: Option<SqlStatus>,
}

pub(crate) fn derive_readiness(sql: &SqlStatus, is_ready: bool) -> Readiness {
    match sql {
        SqlStatus::Healthy => {
            if is_ready {
                Readiness::Ready
            } else {
                Readiness::NotReady
            }
        }
        SqlStatus::Unknown => Readiness::Unknown,
        SqlStatus::Unreachable => Readiness::NotReady,
    }
}

pub(crate) fn to_member_status(
    worker_status: WorkerStatus,
    sql_status: SqlStatus,
    polled_at: UnixMillis,
    poll: Option<PgPollData>,
) -> PgInfoState {
    let readiness_signal = poll.as_ref().map(|value| value.is_ready).unwrap_or(false);
    let timeline = poll.as_ref().and_then(|value| value.timeline);
    let common = PgInfoCommon {
        worker: worker_status,
        sql: sql_status.clone(),
        readiness: derive_readiness(&sql_status, readiness_signal),
        timeline,
        pg_config: PgConfig {
            port: None,
            hot_standby: None,
            primary_conninfo: None,
            primary_slot_name: None,
            extra: std::collections::BTreeMap::new(),
        },
        last_refresh_at: Some(polled_at),
    };

    let Some(polled) = poll else {
        return PgInfoState::Unknown { common };
    };

    if polled.in_recovery {
        return PgInfoState::Replica {
            common,
            replay_lsn: polled
                .replay_lsn
                .or(polled.receive_lsn)
                .unwrap_or(WalLsn(0)),
            follow_lsn: polled.receive_lsn,
            upstream: None,
        };
    }

    if let Some(wal_lsn) = polled.current_wal_lsn {
        return PgInfoState::Primary {
            common,
            wal_lsn,
            slots: polled
                .slot_names
                .into_iter()
                .map(|name| ReplicationSlotInfo { name })
                .collect(),
        };
    }

    PgInfoState::Unknown { common }
}

#[cfg(test)]
mod tests {
    use crate::state::{UnixMillis, WalLsn, WorkerStatus};

    use super::{derive_readiness, to_member_status, PgInfoState, Readiness, SqlStatus};
    use crate::pginfo::query::PgPollData;
    use crate::state::TimelineId;

    #[test]
    fn derive_readiness_maps_sql_and_signal() {
        assert_eq!(
            derive_readiness(&SqlStatus::Unknown, false),
            Readiness::Unknown
        );
        assert_eq!(
            derive_readiness(&SqlStatus::Unreachable, true),
            Readiness::NotReady
        );
        assert_eq!(
            derive_readiness(&SqlStatus::Healthy, true),
            Readiness::Ready
        );
        assert_eq!(
            derive_readiness(&SqlStatus::Healthy, false),
            Readiness::NotReady
        );
    }

    #[test]
    fn to_member_status_maps_primary_snapshot() {
        let poll = PgPollData {
            in_recovery: false,
            is_ready: true,
            timeline: Some(TimelineId(3)),
            current_wal_lsn: Some(WalLsn(42)),
            replay_lsn: None,
            receive_lsn: None,
            slot_names: vec!["slot_a".to_string(), "slot_b".to_string()],
        };
        let state = to_member_status(
            WorkerStatus::Running,
            SqlStatus::Healthy,
            UnixMillis(100),
            Some(poll),
        );
        assert!(matches!(&state, PgInfoState::Primary { .. }));
        if let PgInfoState::Primary {
            wal_lsn,
            slots,
            common,
            ..
        } = &state
        {
            assert_eq!(*wal_lsn, WalLsn(42));
            assert_eq!(slots.len(), 2);
            assert_eq!(common.readiness, Readiness::Ready);
        }
    }

    #[test]
    fn to_member_status_maps_replica_snapshot() {
        let poll = PgPollData {
            in_recovery: true,
            is_ready: true,
            timeline: Some(TimelineId(8)),
            current_wal_lsn: None,
            replay_lsn: Some(WalLsn(11)),
            receive_lsn: Some(WalLsn(12)),
            slot_names: Vec::new(),
        };
        let state = to_member_status(
            WorkerStatus::Running,
            SqlStatus::Healthy,
            UnixMillis(100),
            Some(poll),
        );
        assert!(matches!(&state, PgInfoState::Replica { .. }));
        if let PgInfoState::Replica {
            replay_lsn,
            follow_lsn,
            common,
            ..
        } = &state
        {
            assert_eq!(*replay_lsn, WalLsn(11));
            assert_eq!(*follow_lsn, Some(WalLsn(12)));
            assert_eq!(common.readiness, Readiness::Ready);
        }
    }

    #[test]
    fn to_member_status_maps_replica_without_replay_lsn() {
        let state = to_member_status(
            WorkerStatus::Running,
            SqlStatus::Healthy,
            UnixMillis(100),
            Some(PgPollData {
                in_recovery: true,
                is_ready: false,
                timeline: Some(TimelineId(9)),
                current_wal_lsn: None,
                replay_lsn: None,
                receive_lsn: None,
                slot_names: Vec::new(),
            }),
        );

        assert!(matches!(&state, PgInfoState::Replica { .. }));
        if let PgInfoState::Replica {
            replay_lsn,
            follow_lsn,
            common,
            ..
        } = &state
        {
            assert_eq!(*replay_lsn, WalLsn(0));
            assert_eq!(*follow_lsn, None);
            assert_eq!(common.readiness, Readiness::NotReady);
        }
    }
}
