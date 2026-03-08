# `pginfo` module reference

The `pginfo` subsystem parses PostgreSQL conninfo strings, polls PostgreSQL state via SQL, and publishes typed state snapshots.

## Module surface

| Path | Visibility |
|---|---|
| `src/pginfo/mod.rs` | `pub mod conninfo` |
| `src/pginfo/conninfo.rs` | `pub(crate)` items |
| `src/pginfo/query.rs` | `pub(crate)` items |
| `src/pginfo/state.rs` | `pub(crate)` items |
| `src/pginfo/worker.rs` | `pub(crate)` items |

The public module `conninfo` is re-exported. All other modules and their contents are crate-visible.

## Conninfo types and parsing

### `PgSslMode`

| Variant | String |
|---|---|
| `Disable` | `disable` |
| `Allow` | `allow` |
| `Prefer` | `prefer` |
| `Require` | `require` |
| `VerifyCa` | `verify-ca` |
| `VerifyFull` | `verify-full` |

`PgSslMode::parse` returns `None` for any string not listed above. `Deserialize` for `PgSslMode` rejects unsupported values with error message `unsupported sslmode \`<value>\``.

### `PgConnInfo`

All fields are `pub(crate)`.

| Field | Type |
|---|---|
| `host` | `String` |
| `port` | `u16` |
| `user` | `String` |
| `dbname` | `String` |
| `application_name` | `Option<String>` |
| `connect_timeout_s` | `Option<u32>` |
| `ssl_mode` | `PgSslMode` |
| `options` | `Option<String>` |

### `parse_pg_conninfo`

Accepts keys `host`, `port`, `user`, `dbname`, `application_name`, `connect_timeout`, `sslmode`, and `options`. Requires `host`, `port`, `user`, and `dbname`. Defaults `ssl_mode` to `PgSslMode::Prefer` when omitted. Rejects:

- whitespace before `=`
- empty keys
- duplicate keys
- unsupported keys
- unsupported `sslmode` values
- invalid numeric values

Returns `ConnInfoParseError` on failure.

### `render_pg_conninfo`

Always renders `host`, `port`, `user`, `dbname`, and `sslmode`. Conditionally renders `application_name`, `connect_timeout`, and `options` when present. Quotes values that are empty or contain whitespace, `'`, or `\`. Escapes `'` and `\` inside quoted values.

### `ConnInfoParseError`

| Variant | Display message |
|---|---|
| `Syntax` | `conninfo syntax error: {0}` |
| `MissingRequiredKey` | `missing required conninfo key \`{0}\`` |
| `DuplicateKey` | `duplicate conninfo key \`{0}\`` |
| `UnsupportedKey` | `unsupported conninfo key \`{0}\`` |
| `UnsupportedSslMode` | `unsupported conninfo sslmode \`{0}\`` |
| `InvalidValue` | `invalid conninfo value for \`{key}\`: {message}` |

## Poll query and decoded payload

### `PGINFO_POLL_SQL`

```sql
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
```

The statement uses exactly one semicolon.

### `PgPollData`

| Field | Type |
|---|---|
| `in_recovery` | `bool` |
| `is_ready` | `bool` |
| `timeline` | `Option<TimelineId>` |
| `current_wal_lsn` | `Option<WalLsn>` |
| `replay_lsn` | `Option<WalLsn>` |
| `receive_lsn` | `Option<WalLsn>` |
| `slot_names` | `Vec<String>` |

### `poll_once`

Renders a DSN from `PgConnInfo`, connects with `tokio_postgres::connect` using `NoTls`, spawns the connection task, runs `query_one(PGINFO_POLL_SQL, &[])`, drops the client, waits for the connection task, decodes columns, parses timeline and LSN values, and returns `PgPollData`.

Connect, query, connection task join, post-query connection error, decode, timeline parse, and LSN parse failures map to `WorkerError::Message`.

### `parse_wal_lsn`

Requires `X/Y` hexadecimal format. Parses each half as hexadecimal, left-shifts the high segment by `32` bits, adds the low segment, and returns `WalLsn`. Errors on malformed format, invalid hex, high-segment overflow, or combined-value overflow.

### `parse_timeline`

Rejects negative `i64` values and converts non-negative values to `TimelineId(u32)`.

## Published state model

### `SqlStatus`

| Variant |
|---|
| `Unknown` |
| `Healthy` |
| `Unreachable` |

### `Readiness`

| Variant |
|---|
| `Unknown` |
| `Ready` |
| `NotReady` |

### `PgConfig`

| Field | Type |
|---|---|
| `port` | `Option<u16>` |
| `hot_standby` | `Option<bool>` |
| `primary_conninfo` | `Option<PgConnInfo>` |
| `primary_slot_name` | `Option<String>` |
| `extra` | `BTreeMap<String, String>` |

### `ReplicationSlotInfo`

| Field | Type |
|---|---|
| `name` | `String` |

### `UpstreamInfo`

| Field | Type |
|---|---|
| `member_id` | `MemberId` |

### `PgInfoCommon`

| Field | Type |
|---|---|
| `worker` | `WorkerStatus` |
| `sql` | `SqlStatus` |
| `readiness` | `Readiness` |
| `timeline` | `Option<TimelineId>` |
| `pg_config` | `PgConfig` |
| `last_refresh_at` | `Option<UnixMillis>` |

### `PgInfoState`

| Variant | Fields |
|---|---|
| `Unknown` | `common: PgInfoCommon` |
| `Primary` | `common: PgInfoCommon`, `wal_lsn: WalLsn`, `slots: Vec<ReplicationSlotInfo>` |
| `Replica` | `common: PgInfoCommon`, `replay_lsn: WalLsn`, `follow_lsn: Option<WalLsn>`, `upstream: Option<UpstreamInfo>` |

### `derive_readiness`

Maps `(SqlStatus, is_ready)` to `Readiness`:

| `SqlStatus` | `is_ready` | `Readiness` |
|---|---|---|
| `Healthy` | `true` | `Ready` |
| `Healthy` | `false` | `NotReady` |
| `Unknown` | any | `Unknown` |
| `Unreachable` | any | `NotReady` |

### `to_member_status`

Builds `PgInfoCommon` with the supplied worker status and SQL status, readiness from `derive_readiness`, timeline from the poll result, empty optional `PgConfig` fields plus empty `extra`, and `last_refresh_at` set to the poll timestamp. Returns:

- `Unknown` when poll data is absent
- `Replica` when `in_recovery = true` and `replay_lsn` is present, with `follow_lsn` from `receive_lsn` and `upstream: None`
- `Unknown` when `in_recovery = true` and `replay_lsn` is absent
- `Primary` when `in_recovery = false` and `current_wal_lsn` is present, with slots mapped from `slot_names`
- `Unknown` for all other cases

## Worker loop and emitted events

### `PgInfoWorkerCtx`

| Field | Type |
|---|---|
| `self_id` | `MemberId` |
| `postgres_conninfo` | `PgConnInfo` |
| `poll_interval` | `Duration` |
| `publisher` | `StatePublisher<PgInfoState>` |
| `log` | `LogHandle` |
| `last_emitted_sql_status` | `Option<SqlStatus>` |

### `run`

Loops forever, calling `step_once`, then sleeping for `poll_interval`.

### `step_once`

Gets current Unix milliseconds, calls `poll_once`, maps success to `WorkerStatus::Running` and `SqlStatus::Healthy`, maps failure to `WorkerStatus::Running` and `SqlStatus::Unreachable`, and publishes the resulting `PgInfoState`.

On poll failure, emits an app event with severity `Warn`, message `pginfo poll failed`, name `pginfo.poll_failed`, domain `pginfo`, result `failed`, and fields `member_id` and `error`.

When SQL status changes, emits `pginfo.sql_transition`:

| Transition | Severity | Result | Fields |
|---|---|---|---|
| `Healthy -> Unreachable` | `Warn` | `failed` | `member_id`, `sql_status_prev`, `sql_status_next` |
| `Unreachable -> Healthy` | `Info` | `recovered` | `member_id`, `sql_status_prev`, `sql_status_next` |
| other changes | `Debug` | `ok` | `member_id`, `sql_status_prev`, `sql_status_next` |

`step_once` publishes the next state with the current timestamp. Publish failures map to `WorkerError::Message("pginfo publish failed for {:?}: {err}")`.

`now_unix_millis` returns `WorkerError` when the system clock is before the Unix epoch or the millisecond conversion fails.

## Verified behaviors from direct tests

### `src/pginfo/conninfo.rs`

- Accepts minimal valid input with required keys only
- Accepts quoted values containing whitespace and escaped `'` and `\`
- Rejects syntax errors: whitespace before `=`, missing `=`, missing token boundaries
- Rejects missing required keys
- Rejects duplicate keys
- Rejects unknown keys
- Rejects unsupported `sslmode` values
- Rejects invalid numeric ranges for `port` and `connect_timeout`
- Renders canonical key order: host, port, user, dbname, application_name, connect_timeout, sslmode, options
- Parse-render roundtrip is stable

### `src/pginfo/query.rs`

- `parse_wal_lsn` accepts `X/Y` hexadecimal format and computes combined `WalLsn`
- `parse_wal_lsn` rejects invalid formats, invalid hex, and overflow cases
- `PGINFO_POLL_SQL` selects all expected columns and contains exactly one semicolon

### `src/pginfo/state.rs`

- `derive_readiness` maps all combinations of `SqlStatus` and `is_ready` to correct `Readiness`
- `to_member_status` maps primary snapshot to `PgInfoState::Primary` with LSN and replication slots
- `to_member_status` maps replica snapshot to `PgInfoState::Replica` with LSN values and `upstream: None`

### `src/pginfo/worker.rs`

Real PostgreSQL flows verify:

- `step_once` transitions from unreachable to primary and tracks WAL and slots
- WAL LSN advances after writes on primary
- Replication slot creation appears in subsequent poll
- Replica convergence with upstream WAL positions
- `pginfo.sql_transition` events emit on SQL status changes
