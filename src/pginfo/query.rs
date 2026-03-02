use crate::state::{TimelineId, WalLsn, WorkerError};

pub(crate) const PGINFO_POLL_SQL: &str = r#"
SELECT
    s.in_recovery,
    s.is_ready,
    s.timeline_id,
    s.current_wal_lsn,
    s.replay_lsn,
    s.receive_lsn,
    COALESCE(r.slot_names, '{}'::text[]) AS slot_names
FROM (
    SELECT
        pg_is_in_recovery() AS in_recovery,
        CASE
            WHEN pg_is_in_recovery() THEN pg_last_wal_replay_lsn() IS NOT NULL
            ELSE TRUE
        END AS is_ready,
        (pg_control_checkpoint()).timeline_id::bigint AS timeline_id,
        CASE
            WHEN pg_is_in_recovery() THEN NULL
            ELSE pg_current_wal_lsn()::text
        END AS current_wal_lsn,
        pg_last_wal_replay_lsn()::text AS replay_lsn,
        pg_last_wal_receive_lsn()::text AS receive_lsn
) AS s
CROSS JOIN (
    SELECT COALESCE(array_remove(array_agg(slot_name ORDER BY slot_name), NULL), '{}'::text[]) AS slot_names
    FROM pg_replication_slots
) AS r;
"#;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PgPollData {
    pub(crate) in_recovery: bool,
    pub(crate) is_ready: bool,
    pub(crate) timeline: Option<TimelineId>,
    pub(crate) current_wal_lsn: Option<WalLsn>,
    pub(crate) replay_lsn: Option<WalLsn>,
    pub(crate) receive_lsn: Option<WalLsn>,
    pub(crate) slot_names: Vec<String>,
}

pub(crate) async fn poll_once(postgres_dsn: &str) -> Result<PgPollData, WorkerError> {
    let (client, connection) = tokio_postgres::connect(postgres_dsn, tokio_postgres::NoTls)
        .await
        .map_err(|err| WorkerError::Message(format!("postgres connect failed: {err}")))?;

    let connection_task = tokio::spawn(connection);

    let row = client
        .query_one(PGINFO_POLL_SQL, &[])
        .await
        .map_err(|err| WorkerError::Message(format!("pginfo poll query failed: {err}")))?;

    drop(client);

    let connection_result = connection_task
        .await
        .map_err(|err| WorkerError::Message(format!("postgres connection task join failed: {err}")))?;
    if let Err(err) = connection_result {
        return Err(WorkerError::Message(format!(
            "postgres connection error after poll: {err}"
        )));
    }

    let timeline_raw: Option<i64> = row
        .try_get("timeline_id")
        .map_err(|err| WorkerError::Message(format!("timeline decode failed: {err}")))?;
    let timeline = parse_timeline(timeline_raw)?;

    let current_wal_lsn = parse_optional_lsn(row.try_get("current_wal_lsn").map_err(|err| {
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
        current_wal_lsn,
        replay_lsn,
        receive_lsn,
        slot_names,
    })
}

pub(crate) fn parse_wal_lsn(raw: &str) -> Result<WalLsn, WorkerError> {
    let trimmed = raw.trim();
    let Some((left, right)) = trimmed.split_once('/') else {
        return Err(WorkerError::Message(format!(
            "invalid LSN '{trimmed}': expected X/Y format"
        )));
    };

    let left_num = u64::from_str_radix(left, 16).map_err(|err| {
        WorkerError::Message(format!("invalid LSN '{trimmed}': high segment parse failed: {err}"))
    })?;
    let right_num = u64::from_str_radix(right, 16).map_err(|err| {
        WorkerError::Message(format!("invalid LSN '{trimmed}': low segment parse failed: {err}"))
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

#[cfg(test)]
mod tests {
    use super::{parse_wal_lsn, PGINFO_POLL_SQL};

    #[test]
    fn parse_wal_lsn_accepts_valid_hex_format() {
        let parsed = match parse_wal_lsn("16/B374D848") {
            Ok(lsn) => lsn,
            Err(err) => panic!("expected valid LSN parse, got error: {err}"),
        };
        assert_eq!(parsed.0, 0x16_0000_0000 + 0xB374D848);
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
        assert!(PGINFO_POLL_SQL.contains("current_wal_lsn"));
        assert!(PGINFO_POLL_SQL.contains("replay_lsn"));
        assert!(PGINFO_POLL_SQL.contains("receive_lsn"));
        assert!(PGINFO_POLL_SQL.contains("slot_names"));
        assert_eq!(PGINFO_POLL_SQL.matches(';').count(), 1);
    }
}
