path: /home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/pginfo/state.rs 273-304

- I found smell 6
since it looks like
```rust
pub(crate) fn to_member_status(
    worker_status: WorkerStatus,
    sql_status: SqlStatus,
    polled_at: UnixMillis,
    poll: Option<PgPollData>,
) -> Result<PgInfoState, WorkerError> {
    let primary_conninfo = poll
        .as_ref()
        .and_then(|value| value.primary_conninfo.as_deref())
        .map(super::conninfo::parse_pg_conninfo)
        .transpose()
        .map_err(|err| WorkerError::Message(format!("primary_conninfo parse failed: {err}")))?;
    let common = PgInfoCommon {
        worker: worker_status,
        sql: sql_status,
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
}
```
