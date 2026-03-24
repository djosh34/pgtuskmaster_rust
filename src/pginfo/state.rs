use std::fmt;

use pgtm_log_derive::LogValue;
use serde::{Deserialize, Serialize};

pub use super::conninfo::{PgConnInfo, PgSslMode};
use crate::config_v2::RuntimeConfigV2;
use crate::logging::LogSender;
use crate::state::StatePublisher;
use crate::state::{
    MemberId, ObservedWalPosition, SystemIdentifier, TimelineId, UnixMillis, WalLsn, WorkerStatus,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, LogValue)]
#[log_value(rename_all = "snake_case")]
pub enum SqlStatus {
    Unknown,
    Healthy,
    Unreachable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Readiness {
    Unknown,
    Ready,
    NotReady,
}

impl fmt::Display for Readiness {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Unknown => "unknown",
            Self::Ready => "ready",
            Self::NotReady => "not_ready",
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PgConfig {
    pub port: Option<u16>,
    pub hot_standby: Option<bool>,
    pub primary_conninfo: Option<PgConnInfo>,
    pub primary_slot_name: Option<String>,
    pub extra: std::collections::BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplicationSlotInfo {
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpstreamInfo {
    pub member_id: MemberId,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PgInfoCommon {
    pub worker: WorkerStatus,
    pub sql: SqlStatus,
    pub readiness: Readiness,
    pub timeline: Option<TimelineId>,
    pub system_identifier: Option<SystemIdentifier>,
    pub pg_config: PgConfig,
    pub last_refresh_at: Option<UnixMillis>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PgInfoState {
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

impl PgInfoState {
    pub(crate) fn common(&self) -> &PgInfoCommon {
        match self {
            Self::Unknown { common }
            | Self::Primary { common, .. }
            | Self::Replica { common, .. } => common,
        }
    }

    pub(crate) fn last_refresh_at(&self) -> Option<UnixMillis> {
        self.common().last_refresh_at
    }

    pub(crate) fn readiness(&self) -> Readiness {
        self.common().readiness.clone()
    }

    pub(crate) fn system_identifier(&self) -> Option<SystemIdentifier> {
        self.common().system_identifier
    }

    pub(crate) fn timeline(&self) -> Option<TimelineId> {
        self.common().timeline
    }

    pub(crate) fn is_primary(&self) -> bool {
        matches!(self, Self::Primary { .. })
    }

    pub(crate) fn is_ready_replica(&self) -> bool {
        matches!(
            self,
            Self::Replica {
                common: PgInfoCommon {
                    readiness: Readiness::Ready,
                    ..
                },
                ..
            }
        )
    }

    pub(crate) fn committed_wal(&self) -> Option<ObservedWalPosition> {
        match self {
            Self::Primary {
                common, wal_lsn, ..
            } => Some(ObservedWalPosition {
                timeline: common.timeline,
                lsn: *wal_lsn,
            }),
            Self::Unknown { .. } | Self::Replica { .. } => None,
        }
    }

    pub(crate) fn replay_wal(&self) -> Option<ObservedWalPosition> {
        match self {
            Self::Replica {
                common, replay_lsn, ..
            } => Some(ObservedWalPosition {
                timeline: common.timeline,
                lsn: *replay_lsn,
            }),
            Self::Unknown { .. } | Self::Primary { .. } => None,
        }
    }

    pub(crate) fn follow_wal(&self) -> Option<ObservedWalPosition> {
        match self {
            Self::Replica {
                common, follow_lsn, ..
            } => follow_lsn.map(|lsn| ObservedWalPosition {
                timeline: common.timeline,
                lsn,
            }),
            Self::Unknown { .. } | Self::Primary { .. } => None,
        }
    }

    pub(crate) fn upstream(&self) -> Option<&MemberId> {
        match self {
            Self::Replica {
                upstream: Some(upstream),
                ..
            } => Some(&upstream.member_id),
            Self::Unknown { .. } | Self::Primary { .. } | Self::Replica { upstream: None, .. } => {
                None
            }
        }
    }

    pub(crate) fn starting() -> Self {
        Self::unknown(WorkerStatus::Starting, SqlStatus::Unknown, None)
    }

    pub(crate) fn unknown(
        worker: WorkerStatus,
        sql: SqlStatus,
        last_refresh_at: Option<UnixMillis>,
    ) -> Self {
        Self::Unknown {
            common: PgInfoCommon {
                worker,
                sql,
                readiness: derive_readiness(&sql, false),
                timeline: None,
                system_identifier: None,
                pg_config: PgConfig {
                    port: None,
                    hot_standby: None,
                    primary_conninfo: None,
                    primary_slot_name: None,
                    extra: std::collections::BTreeMap::new(),
                },
                last_refresh_at,
            },
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct PgInfoWorkerCtx<'a> {
    pub(crate) cfg: &'a RuntimeConfigV2,
    pub(crate) state_channel: PgInfoStateChannel,
    pub(crate) runtime: PgInfoRuntime,
}

#[derive(Clone, Debug)]
pub(crate) struct PgInfoStateChannel {
    pub(crate) publisher: StatePublisher<PgInfoState>,
    pub(crate) last_emitted_sql_status: Option<SqlStatus>,
}

#[derive(Clone, Debug)]
pub(crate) struct PgInfoRuntime {
    pub(crate) log: LogSender,
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

#[cfg(test)]
mod tests {
    use crate::state::{UnixMillis, WorkerStatus};

    use super::{derive_readiness, PgInfoState, Readiness, SqlStatus};

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
    fn unknown_state_tracks_sql_and_refresh_time() {
        let state = PgInfoState::unknown(
            WorkerStatus::Running,
            SqlStatus::Unreachable,
            Some(UnixMillis(100)),
        );

        let mut matched_unknown = false;
        if let PgInfoState::Unknown { common } = state {
            matched_unknown = true;
            assert_eq!(common.sql, SqlStatus::Unreachable);
            assert_eq!(common.readiness, Readiness::NotReady);
            assert_eq!(common.last_refresh_at, Some(UnixMillis(100)));
        }
        assert!(matched_unknown, "expected unknown state");
    }
}
