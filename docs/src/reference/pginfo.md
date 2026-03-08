# PostgreSQL Observation Reference

The `pginfo` subsystem parses PostgreSQL conninfo strings, polls PostgreSQL state via SQL, and publishes typed state snapshots.

## Module Surface

| Path | Visibility |
|---|---|
| `src/pginfo/mod.rs` | module definition, public `conninfo` |
| `src/pginfo/conninfo.rs` | public module |
| `src/pginfo/query.rs` | crate-visible |
| `src/pginfo/state.rs` | crate-visible |
| `src/pginfo/worker.rs` | crate-visible |

## Conninfo Parsing And Rendering

### `PgConnInfo`

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

### `PgSslMode`

| Variant | String |
|---|---|
| `Disable` | `disable` |
| `Allow` | `allow` |
| `Prefer` | `prefer` |
| `Require` | `require` |
| `VerifyCa` | `verify-ca` |
| `VerifyFull` | `verify-full` |

`PgSslMode::parse` returns `None` for any string not listed above. `Deserialize` for `PgSslMode` rejects unsupported values with message `unsupported sslmode \`<value>\``.

### `parse_pg_conninfo`

Accepts keys `host`, `port`, `user`, `dbname`, `application_name`, `connect_timeout`, `sslmode`, and `options`. Requires `host`, `port`, `user`, and `dbname`. Defaults `ssl_mode` to `PgSslMode::Prefer` when omitted. Rejects:

- whitespace before `=`
- empty keys
- duplicate keys
- unsupported keys
- unsupported `sslmode` values
- invalid numeric values

### `render_pg_conninfo`

Always renders `host`, `port`, `user`, `dbname`, and `sslmode`. Conditionally renders `application_name`, `connect_timeout`, and `options` when present. Quotes values that are empty or contain whitespace, `'`, or `\`. Escapes `'` and `\` inside quoted values.

### `ConnInfoParseError`

| Variant | Trigger |
|---|---|
| `Syntax(String)` | whitespace before `=` or empty key |
| `MissingRequiredKey(&'static str)` | missing required key |
| `DuplicateKey(String)` | repeated key |
| `UnsupportedKey(String)` | unsupported key |
| `UnsupportedSslMode(String)` | unsupported `sslmode` value |
| `InvalidValue { key, message }` | invalid numeric value |

## Poll Query And Polling Function

### `PGINFO_POLL_SQL`

| Column | Expression |
|---|---|
| `in_recovery` | `pg_is_in_recovery()` |
| `is_ready` | `TRUE` on primaries; on replicas `TRUE` only when `pg_last_wal_replay_lsn()` is not `NULL` |
| `timeline_id` | `(pg_control_checkpoint()).timeline_id::bigint` |
| `current_wal_lsn` | `pg_current_wal_lsn()::text` when not in recovery, otherwise `NULL` |
| `replay_lsn` | `pg_last_wal_replay_lsn()` |
| `receive_lsn` | `pg_last_wal_receive_lsn()` |
| `slot_names` | `COALESCE(array_remove(array_agg(slot_name ORDER BY slot_name), NULL), '{}'::text[])` |

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

### `parse_wal_lsn`

Requires `X/Y` hexadecimal format. Parses each half as hexadecimal, left-shifts the high segment by `32` bits, adds the low segment, and returns `WalLsn`. Errors on malformed format, invalid hex, high-segment overflow, or combined-value overflow.

### `parse_timeline`

Rejects negative `i64` values and converts non-negative values to `TimelineId(u32)`.

### `poll_once`

Renders a DSN from `PgConnInfo`, connects with `tokio_postgres::connect` using `NoTls`, spawns the connection task, runs `query_one(PGINFO_POLL_SQL, &[])`, drops the client, waits for the connection task, decodes columns, parses timeline and LSN values, and returns `PgPollData`. Connect, query, connection task join, post-query connection error, decode, timeline parse, and LSN parse failures map to `WorkerError::Message`.

## Published State Model

### `SqlStatus`

Variants: `Unknown`, `Healthy`, `Unreachable`.

### `Readiness`

Variants: `Unknown`, `Ready`, `NotReady`.

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

## Worker Loop And Events

### `PgInfoWorkerCtx`

| Field | Type |
|---|---|
| `self_id` | `MemberId` |
| `postgres_conninfo` | `PgConnInfo` |
| `poll_interval` | `Duration` |
| `publisher` | `StatePublisher<PgInfoState>` |
| `log` | `LogHandle` |
| `last_emitted_sql_status` | `Option<SqlStatus>` |

### `worker::run`

Loops forever, calling `step_once`, then sleeping for `poll_interval`.

### `worker::step_once`

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

## Verified Behaviors

- `src/pginfo/query.rs`: validates `parse_wal_lsn` for valid and invalid formats; verifies `PGINFO_POLL_SQL` selects expected fields with one semicolon
- `src/pginfo/state.rs`: validates readiness mapping; verifies primary and replica state derivation
- `src/pginfo/worker.rs`: real PostgreSQL flows verify unreachable-to-primary transition, WAL and slot tracking on primary, replica convergence, and emitted SQL transition events
