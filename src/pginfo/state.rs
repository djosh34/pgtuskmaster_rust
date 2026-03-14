use std::{path::PathBuf, time::Duration};

use serde::{Deserialize, Serialize};

pub(crate) use super::conninfo::render_pg_conninfo;
pub use super::conninfo::{PgConnInfo, PgSslMode};
use super::query::PgPollData;
use crate::state::StatePublisher;
use crate::state::{
    MemberId, SystemIdentifier, TimelineId, UnixMillis, WalLsn, WorkerError, WorkerStatus,
};
use crate::{config::RuntimeConfig, logging::LogHandle, process::state::ProcessRuntimePlan};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

    pub(crate) fn starting() -> Self {
        Self::Unknown {
            common: PgInfoCommon {
                worker: WorkerStatus::Starting,
                sql: SqlStatus::Unknown,
                readiness: Readiness::Unknown,
                timeline: None,
                system_identifier: None,
                pg_config: PgConfig {
                    port: None,
                    hot_standby: None,
                    primary_conninfo: None,
                    primary_slot_name: None,
                    extra: std::collections::BTreeMap::new(),
                },
                last_refresh_at: None,
            },
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct PgInfoWorkerCtx {
    pub(crate) identity: PgNodeIdentity,
    pub(crate) probe: PgProbeTarget,
    pub(crate) cadence: PgInfoCadence,
    pub(crate) state_channel: PgInfoStateChannel,
    pub(crate) runtime: PgInfoRuntime,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PgNodeIdentity {
    pub(crate) self_id: MemberId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PgLocalProbeTarget {
    pub(crate) socket_dir: PathBuf,
    pub(crate) port: u16,
    pub(crate) user: String,
    pub(crate) dbname: String,
    pub(crate) application_name: Option<String>,
    pub(crate) connect_timeout_s: Option<u32>,
    pub(crate) ssl_mode: PgSslMode,
    pub(crate) ssl_root_cert: Option<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PgProbeTarget {
    Local(PgLocalProbeTarget),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PgInfoCadence {
    pub(crate) poll_interval: Duration,
}

#[derive(Clone, Debug)]
pub(crate) struct PgInfoStateChannel {
    pub(crate) publisher: StatePublisher<PgInfoState>,
    pub(crate) last_emitted_sql_status: Option<SqlStatus>,
}

#[derive(Clone, Debug)]
pub(crate) struct PgInfoRuntime {
    pub(crate) log: LogHandle,
}

pub(crate) struct PgInfoWorkerBootstrap {
    pub(crate) identity: PgNodeIdentity,
    pub(crate) probe: PgProbeTarget,
    pub(crate) cadence: PgInfoCadence,
    pub(crate) state_channel: PgInfoStateChannel,
    pub(crate) runtime: PgInfoRuntime,
}

impl PgInfoWorkerCtx {
    pub(crate) fn new(bootstrap: PgInfoWorkerBootstrap) -> Self {
        let PgInfoWorkerBootstrap {
            identity,
            probe,
            cadence,
            state_channel,
            runtime,
        } = bootstrap;
        Self {
            identity,
            probe,
            cadence,
            state_channel,
            runtime,
        }
    }
}

impl PgProbeTarget {
    pub(crate) fn local_from_config(
        cfg: &RuntimeConfig,
        process_plan: &ProcessRuntimePlan,
    ) -> Self {
        Self::Local(PgLocalProbeTarget {
            socket_dir: process_plan.postgres.paths.socket_dir.clone(),
            port: process_plan.postgres.port,
            user: cfg.postgres.roles.superuser.username.clone(),
            dbname: cfg.postgres.local_database.clone(),
            application_name: None,
            connect_timeout_s: Some(cfg.postgres.connect_timeout_s),
            ssl_mode: crate::config::defaults::default_pg_ssl_mode(),
            ssl_root_cert: None,
        })
    }

    pub(crate) fn to_conninfo(&self) -> PgConnInfo {
        match self {
            Self::Local(target) => PgConnInfo {
                host: target.socket_dir.display().to_string(),
                port: target.port,
                user: target.user.clone(),
                dbname: target.dbname.clone(),
                application_name: target.application_name.clone(),
                connect_timeout_s: target.connect_timeout_s,
                ssl_mode: target.ssl_mode,
                ssl_root_cert: target.ssl_root_cert.clone(),
                options: None,
            },
        }
    }
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
) -> Result<PgInfoState, WorkerError> {
    let readiness_signal = poll.as_ref().map(|value| value.is_ready).unwrap_or(false);
    let timeline = poll.as_ref().and_then(|value| value.timeline);
    let system_identifier = poll.as_ref().and_then(|value| value.system_identifier);
    let primary_conninfo = poll
        .as_ref()
        .and_then(|value| value.primary_conninfo.as_deref())
        .map(super::conninfo::parse_pg_conninfo)
        .transpose()
        .map_err(|err| WorkerError::Message(format!("primary_conninfo parse failed: {err}")))?;
    let common = PgInfoCommon {
        worker: worker_status,
        sql: sql_status.clone(),
        readiness: derive_readiness(&sql_status, readiness_signal),
        timeline,
        system_identifier,
        pg_config: PgConfig {
            port: None,
            hot_standby: None,
            primary_conninfo,
            primary_slot_name: poll
                .as_ref()
                .and_then(|value| value.primary_slot_name.clone()),
            extra: std::collections::BTreeMap::new(),
        },
        last_refresh_at: Some(polled_at),
    };

    let Some(polled) = poll else {
        return Ok(PgInfoState::Unknown { common });
    };

    if polled.in_recovery {
        return Ok(PgInfoState::Replica {
            common,
            replay_lsn: polled
                .replay_lsn
                .or(polled.receive_lsn)
                .unwrap_or(WalLsn(0)),
            follow_lsn: polled.receive_lsn,
            upstream: None,
        });
    }

    if let Some(wal_lsn) = polled.current_wal_lsn {
        return Ok(PgInfoState::Primary {
            common,
            wal_lsn,
            slots: polled
                .slot_names
                .into_iter()
                .map(|name| ReplicationSlotInfo { name })
                .collect(),
        });
    }

    Ok(PgInfoState::Unknown { common })
}

#[cfg(test)]
mod tests {
    use crate::state::{SystemIdentifier, UnixMillis, WalLsn, WorkerStatus};

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
            system_identifier: Some(SystemIdentifier(11)),
            current_wal_lsn: Some(WalLsn(42)),
            replay_lsn: None,
            receive_lsn: None,
            primary_conninfo: None,
            primary_slot_name: None,
            slot_names: vec!["slot_a".to_string(), "slot_b".to_string()],
        };
        let state = to_member_status(
            WorkerStatus::Running,
            SqlStatus::Healthy,
            UnixMillis(100),
            Some(poll),
        );
        assert!(state.is_ok(), "unexpected error: {state:?}");
        let mut matched_primary = false;
        if let Ok(PgInfoState::Primary {
            wal_lsn,
            slots,
            common,
            ..
        }) = state
        {
            matched_primary = true;
            assert_eq!(wal_lsn, WalLsn(42));
            assert_eq!(slots.len(), 2);
            assert_eq!(common.readiness, Readiness::Ready);
            assert_eq!(common.system_identifier, Some(SystemIdentifier(11)));
        }
        assert!(matched_primary, "expected primary state");
    }

    #[test]
    fn to_member_status_maps_replica_snapshot() {
        let poll = PgPollData {
            in_recovery: true,
            is_ready: true,
            timeline: Some(TimelineId(8)),
            system_identifier: Some(SystemIdentifier(17)),
            current_wal_lsn: None,
            replay_lsn: Some(WalLsn(11)),
            receive_lsn: Some(WalLsn(12)),
            primary_conninfo: None,
            primary_slot_name: None,
            slot_names: Vec::new(),
        };
        let state = to_member_status(
            WorkerStatus::Running,
            SqlStatus::Healthy,
            UnixMillis(100),
            Some(poll),
        );
        assert!(state.is_ok(), "unexpected error: {state:?}");
        let mut matched_replica = false;
        if let Ok(PgInfoState::Replica {
            replay_lsn,
            follow_lsn,
            common,
            ..
        }) = state
        {
            matched_replica = true;
            assert_eq!(replay_lsn, WalLsn(11));
            assert_eq!(follow_lsn, Some(WalLsn(12)));
            assert_eq!(common.readiness, Readiness::Ready);
            assert_eq!(common.system_identifier, Some(SystemIdentifier(17)));
        }
        assert!(matched_replica, "expected replica state");
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
                system_identifier: Some(SystemIdentifier(23)),
                current_wal_lsn: None,
                replay_lsn: None,
                receive_lsn: None,
                primary_conninfo: None,
                primary_slot_name: None,
                slot_names: Vec::new(),
            }),
        );
        assert!(state.is_ok(), "unexpected error: {state:?}");
        let mut matched_replica = false;
        if let Ok(PgInfoState::Replica {
            replay_lsn,
            follow_lsn,
            common,
            ..
        }) = state
        {
            matched_replica = true;
            assert_eq!(replay_lsn, WalLsn(0));
            assert_eq!(follow_lsn, None);
            assert_eq!(common.readiness, Readiness::NotReady);
        }
        assert!(matched_replica, "expected replica state");
    }
}
