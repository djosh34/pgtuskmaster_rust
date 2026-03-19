path: /home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/worker.rs 337-389

- I found smell 1
since it looks like
```rust
fn build_postgres_state(pg: &PgInfoState) -> PostgresState {
    match pg {
        PgInfoState::Unknown { common } if common.sql != SqlStatus::Healthy => {
            PostgresState::Offline
        }
        PgInfoState::Unknown { .. } => PostgresState::Offline,
        PgInfoState::Primary {
            common, wal_lsn, ..
        } if common.sql == SqlStatus::Healthy => PostgresState::Primary {
            committed_lsn: wal_lsn.0,
        },
        PgInfoState::Primary { .. } => PostgresState::Offline,
        PgInfoState::Replica {
            common,
            replay_lsn,
            follow_lsn,
            upstream,
        } if common.sql == SqlStatus::Healthy => PostgresState::Replica {
            upstream: upstream.as_ref().map(|value| value.member_id.clone()),
            replication: build_replication_state(common.timeline, *replay_lsn, *follow_lsn),
        },
        PgInfoState::Replica { .. } => PostgresState::Offline,
    }
}

fn build_local_postgres_state(pg: &PgInfoState, dcs: &DcsView) -> PostgresState {
    match build_postgres_state(pg) {
        PostgresState::Replica {
            upstream,
            replication,
        } => PostgresState::Replica {
            upstream: upstream.or_else(|| resolve_replica_upstream(pg, dcs)),
            replication,
        },
        state => state,
    }
}
```
