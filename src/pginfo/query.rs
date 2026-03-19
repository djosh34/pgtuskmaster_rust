use crate::{
    pginfo::conninfo::parse_pg_conninfo,
    state::{SystemIdentifier, TimelineId, UnixMillis, WalLsn, WorkerError, WorkerStatus},
};

use super::state::{
    derive_readiness, PgConfig, PgConnInfo, PgInfoCommon, PgInfoState, ReplicationSlotInfo,
    SqlStatus,
};

pub(crate) const PGINFO_POLL_SQL: &str = r#"
SELECT
    s.in_recovery,
    s.is_ready,
    s.timeline_id,
    s.system_identifier,
    s.current_wal_lsn,
    s.replay_lsn,
    s.receive_lsn,
    s.primary_conninfo,
    s.primary_slot_name,
    COALESCE(r.slot_names, '{}'::text[]) AS slot_names
FROM (
    SELECT
        pg_is_in_recovery() AS in_recovery,
        CASE
            WHEN pg_is_in_recovery() THEN pg_last_wal_replay_lsn() IS NOT NULL OR pg_last_wal_receive_lsn() IS NOT NULL
            ELSE TRUE
        END AS is_ready,
        (pg_control_checkpoint()).timeline_id::bigint AS timeline_id,
        (pg_control_system()).system_identifier::text AS system_identifier,
        CASE
            WHEN pg_is_in_recovery() THEN NULL
            ELSE pg_current_wal_lsn()::text
        END AS current_wal_lsn,
        pg_last_wal_replay_lsn()::text AS replay_lsn,
        pg_last_wal_receive_lsn()::text AS receive_lsn,
        NULLIF(current_setting('primary_conninfo', true), '') AS primary_conninfo,
        NULLIF(current_setting('primary_slot_name', true), '') AS primary_slot_name
) AS s
CROSS JOIN (
    SELECT COALESCE(array_remove(array_agg(slot_name ORDER BY slot_name), NULL), '{}'::text[]) AS slot_names
    FROM pg_replication_slots
) AS r;
"#;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PgPollData {
    pub(crate) in_recovery: bool,
    pub(crate) is_ready: bool,
    pub(crate) timeline: Option<TimelineId>,
    pub(crate) system_identifier: Option<SystemIdentifier>,
    pub(crate) current_wal_lsn: Option<WalLsn>,
    pub(crate) replay_lsn: Option<WalLsn>,
    pub(crate) receive_lsn: Option<WalLsn>,
    pub(crate) primary_conninfo: Option<PgConnInfo>,
    pub(crate) primary_slot_name: Option<String>,
    pub(crate) slot_names: Vec<String>,
}

pub(super) async fn poll_state_once(
    postgres_conninfo: &PgConnInfo,
    worker_status: WorkerStatus,
    sql_status: SqlStatus,
    polled_at: UnixMillis,
) -> Result<PgInfoState, WorkerError> {
    let postgres_dsn = postgres_conninfo.to_string();
    let (client, connection) = tokio_postgres::connect(&postgres_dsn, tokio_postgres::NoTls)
        .await
        .map_err(|err| WorkerError::Message(format!("postgres connect failed: {err}")))?;

    let connection_task = tokio::spawn(connection);

    let row = client
        .query_one(PGINFO_POLL_SQL, &[])
        .await
        .map_err(|err| WorkerError::Message(format!("pginfo poll query failed: {err}")))?;

    drop(client);

    let connection_result = connection_task.await.map_err(|err| {
        WorkerError::Message(format!("postgres connection task join failed: {err}"))
    })?;
    if let Err(err) = connection_result {
        return Err(WorkerError::Message(format!(
            "postgres connection error after poll: {err}"
        )));
    }

    let timeline_raw: Option<i64> = row
        .try_get("timeline_id")
        .map_err(|err| WorkerError::Message(format!("timeline decode failed: {err}")))?;
    let timeline = parse_timeline(timeline_raw)?;
    let system_identifier =
        parse_system_identifier(row.try_get("system_identifier").map_err(|err| {
            WorkerError::Message(format!("system_identifier decode failed: {err}"))
        })?)?;

    let current_wal_lsn =
        parse_optional_lsn(row.try_get("current_wal_lsn").map_err(|err| {
            WorkerError::Message(format!("current_wal_lsn decode failed: {err}"))
        })?)?;
    let replay_lsn = parse_optional_lsn(
        row.try_get("replay_lsn")
            .map_err(|err| WorkerError::Message(format!("replay_lsn decode failed: {err}")))?,
    )?;
    let receive_lsn = parse_optional_lsn(
        row.try_get("receive_lsn")
            .map_err(|err| WorkerError::Message(format!("receive_lsn decode failed: {err}")))?,
    )?;

    let slot_names: Vec<String> = row
        .try_get("slot_names")
        .map_err(|err| WorkerError::Message(format!("slot_names decode failed: {err}")))?;
    let primary_conninfo_raw: Option<String> = row
        .try_get("primary_conninfo")
        .map_err(|err| WorkerError::Message(format!("primary_conninfo decode failed: {err}")))?;
    let primary_conninfo = primary_conninfo_raw
        .as_deref()
        .map(parse_pg_conninfo)
        .transpose()
        .map_err(|err| WorkerError::Message(format!("primary_conninfo parse failed: {err}")))?;
    let primary_slot_name: Option<String> = row
        .try_get("primary_slot_name")
        .map_err(|err| WorkerError::Message(format!("primary_slot_name decode failed: {err}")))?;

    let in_recovery: bool = row
        .try_get("in_recovery")
        .map_err(|err| WorkerError::Message(format!("in_recovery decode failed: {err}")))?;
    let is_ready: bool = row
        .try_get("is_ready")
        .map_err(|err| WorkerError::Message(format!("is_ready decode failed: {err}")))?;

    Ok(PgPollData {
        in_recovery,
        is_ready,
        timeline,
        system_identifier,
        current_wal_lsn,
        replay_lsn,
        receive_lsn,
        primary_conninfo,
        primary_slot_name,
        slot_names,
    }
    .into_state(worker_status, sql_status, polled_at))
}

pub(crate) fn parse_wal_lsn(raw: &str) -> Result<WalLsn, WorkerError> {
    let trimmed = raw.trim();
    let Some((left, right)) = trimmed.split_once('/') else {
        return Err(WorkerError::Message(format!(
            "invalid LSN '{trimmed}': expected X/Y format"
        )));
    };

    let left_num = u64::from_str_radix(left, 16).map_err(|err| {
        WorkerError::Message(format!(
            "invalid LSN '{trimmed}': high segment parse failed: {err}"
        ))
    })?;
    let right_num = u64::from_str_radix(right, 16).map_err(|err| {
        WorkerError::Message(format!(
            "invalid LSN '{trimmed}': low segment parse failed: {err}"
        ))
    })?;

    let shifted = left_num.checked_shl(32).ok_or_else(|| {
        WorkerError::Message(format!("invalid LSN '{trimmed}': high segment overflow"))
    })?;
    let combined = shifted.checked_add(right_num).ok_or_else(|| {
        WorkerError::Message(format!("invalid LSN '{trimmed}': combined value overflow"))
    })?;
    Ok(WalLsn(combined))
}

fn parse_optional_lsn(raw: Option<String>) -> Result<Option<WalLsn>, WorkerError> {
    match raw {
        Some(value) => parse_wal_lsn(&value).map(Some),
        None => Ok(None),
    }
}

fn parse_timeline(raw: Option<i64>) -> Result<Option<TimelineId>, WorkerError> {
    match raw {
        Some(value) => {
            if value < 0 {
                return Err(WorkerError::Message(format!(
                    "timeline must be non-negative, got {value}"
                )));
            }
            let as_u32 = u32::try_from(value)
                .map_err(|err| WorkerError::Message(format!("timeline out of range: {err}")))?;
            Ok(Some(TimelineId(as_u32)))
        }
        None => Ok(None),
    }
}

fn parse_system_identifier(raw: Option<String>) -> Result<Option<SystemIdentifier>, WorkerError> {
    match raw {
        Some(value) => value
            .parse::<u64>()
            .map(SystemIdentifier)
            .map(Some)
            .map_err(|err| {
                WorkerError::Message(format!(
                    "system_identifier parse failed for '{value}': {err}"
                ))
            }),
        None => Ok(None),
    }
}

impl PgPollData {
    fn into_state(
        self,
        worker_status: WorkerStatus,
        sql_status: SqlStatus,
        polled_at: UnixMillis,
    ) -> PgInfoState {
        let Self {
            in_recovery,
            is_ready,
            timeline,
            system_identifier,
            current_wal_lsn,
            replay_lsn,
            receive_lsn,
            primary_conninfo,
            primary_slot_name,
            slot_names,
        } = self;

        let common = PgInfoCommon {
            worker: worker_status,
            sql: sql_status,
            readiness: derive_readiness(&sql_status, is_ready),
            timeline,
            system_identifier,
            pg_config: PgConfig {
                port: None,
                hot_standby: None,
                primary_conninfo,
                primary_slot_name,
                extra: std::collections::BTreeMap::new(),
            },
            last_refresh_at: Some(polled_at),
        };

        if in_recovery {
            return PgInfoState::Replica {
                common,
                replay_lsn: replay_lsn.or(receive_lsn).unwrap_or(WalLsn(0)),
                follow_lsn: receive_lsn,
                upstream: None,
            };
        }

        if let Some(wal_lsn) = current_wal_lsn {
            return PgInfoState::Primary {
                common,
                wal_lsn,
                slots: slot_names
                    .into_iter()
                    .map(|name| ReplicationSlotInfo { name })
                    .collect(),
            };
        }

        PgInfoState::Unknown { common }
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_wal_lsn, PgPollData, PGINFO_POLL_SQL};
    use crate::{
        pginfo::state::{PgInfoState, Readiness, SqlStatus},
        state::{SystemIdentifier, TimelineId, UnixMillis, WalLsn, WorkerStatus},
    };

    #[test]
    fn parse_wal_lsn_accepts_valid_hex_format() {
        let parsed = parse_wal_lsn("16/B374D848");
        assert!(parsed.is_ok());
        if let Ok(lsn) = parsed {
            assert_eq!(lsn.0, 0x16_0000_0000 + 0xB374D848);
        }
    }

    #[test]
    fn parse_wal_lsn_rejects_invalid_formats() {
        assert!(parse_wal_lsn("16").is_err());
        assert!(parse_wal_lsn("G/10").is_err());
        assert!(parse_wal_lsn("10/XYZ").is_err());
    }

    #[test]
    fn poll_sql_selects_expected_fields() {
        assert!(PGINFO_POLL_SQL.contains("in_recovery"));
        assert!(PGINFO_POLL_SQL.contains("timeline_id"));
        assert!(PGINFO_POLL_SQL.contains("system_identifier"));
        assert!(PGINFO_POLL_SQL.contains("current_wal_lsn"));
        assert!(PGINFO_POLL_SQL.contains("replay_lsn"));
        assert!(PGINFO_POLL_SQL.contains("receive_lsn"));
        assert!(PGINFO_POLL_SQL.contains("primary_conninfo"));
        assert!(PGINFO_POLL_SQL.contains("primary_slot_name"));
        assert!(PGINFO_POLL_SQL.contains("slot_names"));
        assert_eq!(PGINFO_POLL_SQL.matches(';').count(), 1);
    }

    #[test]
    fn poll_data_into_state_maps_primary_snapshot() {
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
        let state =
            poll.into_state(WorkerStatus::Running, SqlStatus::Healthy, UnixMillis(100));

        let mut matched_primary = false;
        if let PgInfoState::Primary {
            wal_lsn,
            slots,
            common,
            ..
        } = state
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
    fn poll_data_into_state_maps_replica_snapshot() {
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
        let state =
            poll.into_state(WorkerStatus::Running, SqlStatus::Healthy, UnixMillis(100));

        let mut matched_replica = false;
        if let PgInfoState::Replica {
            replay_lsn,
            follow_lsn,
            common,
            ..
        } = state
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
    fn poll_data_into_state_maps_replica_without_replay_lsn() {
        let state = PgPollData {
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
        }
        .into_state(WorkerStatus::Running, SqlStatus::Healthy, UnixMillis(100));

        let mut matched_replica = false;
        if let PgInfoState::Replica {
            replay_lsn,
            follow_lsn,
            common,
            ..
        } = state
        {
            matched_replica = true;
            assert_eq!(replay_lsn, WalLsn(0));
            assert_eq!(follow_lsn, None);
            assert_eq!(common.readiness, Readiness::NotReady);
        }
        assert!(matched_replica, "expected replica state");
    }
}
