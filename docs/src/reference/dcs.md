# DCS Layer Reference

The `src/dcs` module provides the distributed consensus store layer used by the HA and API workers. It covers scoped key parsing, cached DCS state, trust evaluation, watch-driven refresh, and the etcd-backed store implementation.

## Module Surface

`src/dcs` contains:

| Module | Purpose |
|---|---|
| `etcd_store` | etcd client and watch worker |
| `keys` | scoped key parsing |
| `state` | cache, trust evaluation, and state containers |
| `store` | store trait, HA writer, and watch refresh logic |
| `worker` | polling loop and step execution |

## Keyspace And Record Types

### `DcsKey`

| Variant | Path Pattern | Record Type |
|---|---|---|
| `Member(MemberId)` | `/{scope}/member/{member_id}` | `MemberRecord` |
| `Leader` | `/{scope}/leader` | `LeaderRecord` |
| `Switchover` | `/{scope}/switchover` | `SwitchoverRequest` |
| `Config` | `/{scope}/config` | `RuntimeConfig` |
| `InitLock` | `/{scope}/init` | `InitLockRecord` |

### Path Parsing

`key_from_path(scope, full_path)`:

- trims outer slashes from `scope`
- requires the prefix `/{scope}/`
- parses `leader`, `switchover`, `config`, `init`, and `member/{member_id}`
- rejects empty member ids
- rejects empty or malformed suffixes
- rejects unsupported extra path segments

### `DcsKeyParseError`

| Variant | Condition |
|---|---|
| `InvalidScopePrefix { path, scope_prefix }` | path does not begin with the expected scoped prefix |
| `MalformedPath(String)` | the suffix after the scope prefix is empty or malformed |
| `MissingMemberId(String)` | the member key path omits the member id segment |
| `UnknownKey(String)` | the path does not match a supported key kind |

### Record Types

| Record | Fields |
|---|---|
| `MemberRecord` | `member_id`, `postgres_host`, `postgres_port`, `role`, `sql`, `readiness`, `timeline`, `write_lsn`, `replay_lsn`, `updated_at`, `pg_version` |
| `LeaderRecord` | `member_id` |
| `SwitchoverRequest` | `requested_by` |
| `InitLockRecord` | `holder` |

### `MemberRole`

Values: `Unknown`, `Primary`, `Replica`.

`build_local_member_record` maps:

- `PgInfoState::Unknown` to role `Unknown` with no LSNs
- `PgInfoState::Primary` to role `Primary` with `write_lsn`
- `PgInfoState::Replica` to role `Replica` with `replay_lsn`

## Cache And Trust Model

### State Containers

| Type | Fields |
|---|---|
| `DcsCache` | `members`, `leader`, `switchover`, `config`, `init_lock` |
| `DcsState` | `worker`, `trust`, `cache`, `last_refresh_at` |

### `DcsTrust`

Values: `FullQuorum`, `FailSafe`, `NotTrusted`. Serde uses `snake_case`.

### Trust Evaluation

`evaluate_trust(etcd_healthy, cache, self_id, now)` returns:

- `NotTrusted` when etcd health is false
- `FailSafe` when:
  - the local member record is missing or stale
  - a leader exists but its member record is missing or stale
  - more than one cached member exists and fewer than two are fresh
- `FullQuorum` otherwise

Member freshness uses `now.0.saturating_sub(record.updated_at.0) <= cache.config.ha.lease_ttl_ms`.

## Store And Watch Surface

### `DcsStore`

| Method | Description |
|---|---|
| `healthy` | reports whether the store currently considers itself healthy |
| `read_path` | reads one path value |
| `write_path` | writes one path value |
| `put_path_if_absent` | writes one path only when it does not already exist |
| `delete_path` | deletes one path |
| `drain_watch_events` | returns the queued watch events for the current scope |

### `DcsHaWriter`

The blanket `DcsHaWriter` implementation applies to any `DcsStore`.

| Method | Behavior |
|---|---|
| `write_leader_lease` | Serializes `LeaderRecord` to `/{scope}/leader` with `put_path_if_absent`; returns `DcsStoreError::AlreadyExists(path)` when the path already exists |
| `delete_leader` | Deletes `/{scope}/leader` |
| `clear_switchover` | Deletes `/{scope}/switchover` |

### Member Writes

`write_local_member` serializes `MemberRecord` as JSON and writes it unconditionally to `/{scope}/member/{member_id}`.

### Watch Types

| Type | Values Or Fields |
|---|---|
| `WatchOp` | `Put`, `Delete`, `Reset` |
| `WatchEvent` | `op`, `path`, `value`, `revision` |
| `RefreshResult` | `applied`, `had_errors` |

`Reset` is synthesized by the etcd store during reconnect or resnapshot so the consumer can clear cached scope records before applying the authoritative snapshot.

### `DcsStoreError`

| Variant | Meaning |
|---|---|
| `MissingValue(String)` | a `Put` watch event had no value payload |
| `InvalidKey(DcsKeyParseError)` | a non-ignorable path failed key parsing |
| `Decode { key, message }` | stored JSON or watch payload decoding failed |
| `AlreadyExists(String)` | a conditional write found an existing path |
| `Io(String)` | etcd transport, timeout, or worker I/O failure |

### `refresh_from_etcd_watch`

`refresh_from_etcd_watch(scope, cache, events)`:

- applies events in order
- clears `members`, `leader`, `switchover`, and `init_lock` on `Reset`
- preserves `cache.config` across `Reset`
- increments `applied` for each processed event, including `Reset`
- skips unknown keyed paths and sets `had_errors = true`
- returns `DcsStoreError::InvalidKey` for invalid scope prefixes or malformed key paths
- returns `DcsStoreError::MissingValue` for `Put` without a value
- decodes `Put` payloads into `MemberRecord`, `LeaderRecord`, `SwitchoverRequest`, `RuntimeConfig`, or `InitLockRecord` according to key kind
- applies config puts by replacing `cache.config`
- treats config deletes as a no-op

## Worker Loop

`dcs::worker::run` loops forever, calling `step_once(&mut ctx).await?` and then sleeping for `poll_interval`.

### `DcsWorkerCtx`

| Field | Description |
|---|---|
| `self_id` | local member identifier |
| `scope` | DCS key prefix |
| `poll_interval` | duration between ticks |
| `local_postgres_host`, `local_postgres_port` | local PostgreSQL endpoint |
| `pg_subscriber` | PostgreSQL state subscriber |
| `publisher` | DCS state publisher |
| `store` | `DcsStore` implementation |
| `log` | log handle |
| `cache` | current `DcsCache` |
| `last_published_pg_version` | version of the last published PostgreSQL snapshot |
| `last_emitted_store_healthy`, `last_emitted_trust` | last transition values emitted by the worker |

### `step_once`

`step_once`:

1. reads the latest PostgreSQL snapshot
2. checks store health
3. publishes the local member record only when the store is healthy at the start of the step
4. drains watch events
5. refreshes the cache from those events
6. computes trust
7. derives worker status
8. publishes the next `DcsState`
9. emits store-health and trust transition events when those values change

The step marks the store unhealthy when local member publication fails, watch draining fails, watch refresh fails, or watch refresh reports `had_errors`.

Trust is computed with `evaluate_trust` only when local member publication succeeded. Otherwise trust is `NotTrusted`.

When the store is unhealthy, worker status becomes `Faulted(WorkerError::Message("dcs store unhealthy"))` and the published trust is forced to `NotTrusted`.

## Etcd-Backed Implementation

### `EtcdDcsStore::connect`

`EtcdDcsStore::connect(endpoints, scope)`:

- requires at least one endpoint
- builds the scope prefix `/{scope}/`
- spawns a background thread named `etcd-dcs-store`
- builds a current-thread Tokio runtime in that thread
- establishes an initial get plus watch session before reporting startup success

| Constant | Value |
|---|---|
| `COMMAND_TIMEOUT` | `2 seconds` |
| `WORKER_BOOTSTRAP_TIMEOUT` | `8 seconds` |
| `WATCH_IDLE_INTERVAL` | `100 ms` |

On startup timeout, `connect` sends `Shutdown`, drops the command channel, drops the worker handle without joining, and returns an I/O error.

### Reconnect And Watch Handling

- The worker loop reconnects when the client or watch stream is missing, sleeping for `WATCH_IDLE_INTERVAL` between retries
- On reconnect or resnapshot, the store prepends `Reset` and replaces the queued events with the authoritative snapshot
- `apply_watch_response` converts etcd `Put` and `Delete` events into `WatchEvent` values and rejects canceled or compacted watches with `DcsStoreError::Io`

## Verified Behaviors

Tests in `src/dcs/store.rs` verify:

- `write_local_member` writes only `/{scope}/member/{member_id}` and encodes a `MemberRecord` payload
- `write_leader_lease` writes `/{scope}/leader` and returns `DcsStoreError::AlreadyExists(path)` when the leader key already exists
- `delete_leader` deletes `/{scope}/leader`
- `clear_switchover` deletes `/{scope}/switchover`
- `refresh_from_etcd_watch` applies member `Put` and `Delete` events
- decode failures return `DcsStoreError::Decode`
- unknown scoped keys set `had_errors = true` while allowing known updates in the same batch
- `Reset` clears `members`, `leader`, `switchover`, and `init_lock` while preserving `cache.config`

Tests in `src/dcs/worker.rs` verify:

- `step_once` can publish the local member record, consume an observed leader record, and publish `DcsTrust::FullQuorum`
- local member write I/O failure emits `dcs.local_member.write_failed` and forces the published trust to `NotTrusted`
- the worker writes the local member record on every tick, not only when the PostgreSQL version changes
- local member publication uses `local_postgres_host` and `local_postgres_port` from `DcsWorkerCtx` rather than a cached config endpoint
- the worker republishes the local member record after an unhealthy tick even when PostgreSQL state has not changed
- watch decode failure or unknown keyed watch input faults the worker and forces published trust to `NotTrusted`

Tests in `src/dcs/etcd_store.rs` verify:

- reconnect prepends `Reset` and clears stale queued events before the replacement snapshot is applied
- write and delete operations round-trip through real etcd watch events
- `put_path_if_absent` can claim a key only once and does not overwrite the existing value
- a real etcd-backed `step_once` run can observe a leader update and publish the local member record
- malformed leader JSON in real etcd input faults the worker and forces trust to `NotTrusted`

### `DcsStore for EtcdDcsStore`

- `read_path`, `write_path`, `put_path_if_absent`, and `delete_path` proxy through command channels bounded by `COMMAND_TIMEOUT`
- send failures and receive-timeout failures mark the store unhealthy
- `drain_watch_events` drains the internal queue under a mutex

### Drop Behavior

`Drop for EtcdDcsStore` sends `Shutdown` and joins the worker thread when present.
