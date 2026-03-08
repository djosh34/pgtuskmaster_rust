Answer using only the information in this prompt.
Do not ask to inspect files, browse, run tools, or fetch more context.
If required information is missing, say exactly what is missing.

You are writing prose for an mdBook documentation page.
Return only markdown for the page body requested.
Use ASCII punctuation only.
Do not use em dashes.
Do not invent facts.

[Task]
- Revise an existing reference page so it stays strictly in Diataxis reference form.

[Page path]
- docs/src/reference/dcs.md

[Page goal]
- Reference the DCS keyspace, cached state, trust model, watch refresh behavior, worker loop, and etcd-backed store implementation.

[Audience]
- Operators and contributors who need accurate repo-backed facts while working with pgtuskmaster.

[User need]
- Consult the machinery surface, data model, constraints, constants, and behavior without being taught procedures or background explanations.

[mdBook context]
- This is an mdBook page under docs/src/reference/.
- Keep headings and lists suitable for mdBook.
- Do not add verification notes, scratch notes, or commentary about how the page was produced.

[Diataxis guidance]
- This page must stay in the reference quadrant: cognition plus application.
- Describe and only describe.
- Structure the page to mirror the machinery, not a guessed workflow.
- Use neutral, technical language.
- Examples are allowed only when they illustrate the surface concisely.
- Do not include step-by-step operations, recommendations, rationale, or explanations of why the design exists.
- If action or explanation seems necessary, keep the page neutral and mention the boundary without turning the page into a how-to or explanation article.

[Required structure]
- Overview\n- Module surface\n- Keyspace and record types\n- Cache and trust model\n- Store and watch surface\n- Worker loop\n- Etcd-backed implementation constants and reconnect behavior

[Output requirements]
- Preserve or improve the title so it matches the machinery.
- Prefer compact tables where they clarify enums, fields, constants, routes, or error variants.
- Include only facts supported by the supplied source excerpts.
- If the current page contains unsupported claims, remove or correct them rather than hedging.

[Existing page to revise]

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

### `DcsStore for EtcdDcsStore`

- `read_path`, `write_path`, `put_path_if_absent`, and `delete_path` proxy through command channels bounded by `COMMAND_TIMEOUT`
- send failures and receive-timeout failures mark the store unhealthy
- `drain_watch_events` drains the internal queue under a mutex

### Drop Behavior

`Drop for EtcdDcsStore` sends `Shutdown` and joins the worker thread when present.

[Repo facts and source excerpts]

--- BEGIN FILE: src/dcs/mod.rs ---
pub(crate) mod etcd_store;
pub mod keys;
pub(crate) mod state;
pub mod store;
pub(crate) mod worker;

--- END FILE: src/dcs/mod.rs ---

--- BEGIN FILE: src/dcs/keys.rs ---
use thiserror::Error;

use crate::state::MemberId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DcsKey {
    Member(MemberId),
    Leader,
    Switchover,
    Config,
    InitLock,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum DcsKeyParseError {
    #[error("path `{path}` does not start with scope prefix `{scope_prefix}`")]
    InvalidScopePrefix { path: String, scope_prefix: String },
    #[error("path `{0}` is malformed")]
    MalformedPath(String),
    #[error("member id segment is missing in path `{0}`")]
    MissingMemberId(String),
    #[error("unknown key path `{0}`")]
    UnknownKey(String),
}

pub(crate) fn key_from_path(scope: &str, full_path: &str) -> Result<DcsKey, DcsKeyParseError> {
    let scope = scope.trim_matches('/');
    let expected_prefix = format!("/{scope}/");
    if !full_path.starts_with(&expected_prefix) {
        return Err(DcsKeyParseError::InvalidScopePrefix {
            path: full_path.to_string(),
            scope_prefix: expected_prefix,
        });
    }

    let suffix = &full_path[expected_prefix.len()..];
    let parts: Vec<&str> = suffix.split('/').collect();
    match parts.as_slice() {
        ["leader"] => Ok(DcsKey::Leader),
        ["switchover"] => Ok(DcsKey::Switchover),
        ["config"] => Ok(DcsKey::Config),
        ["init"] => Ok(DcsKey::InitLock),
        ["member", member_id] => {
            if member_id.is_empty() {
                return Err(DcsKeyParseError::MissingMemberId(full_path.to_string()));
            }
            Ok(DcsKey::Member(MemberId((*member_id).to_string())))
        }
        [] | [""] => Err(DcsKeyParseError::MalformedPath(full_path.to_string())),
        ["member"] => Err(DcsKeyParseError::MissingMemberId(full_path.to_string())),
        _ => Err(DcsKeyParseError::UnknownKey(full_path.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::{key_from_path, DcsKey, DcsKeyParseError};
    use crate::state::MemberId;

    #[test]
    fn parses_supported_paths() {
        assert_eq!(
            key_from_path("scope-a", "/scope-a/member/node-a"),
            Ok(DcsKey::Member(MemberId("node-a".to_string())))
        );
        assert_eq!(
            key_from_path("scope-a", "/scope-a/leader"),
            Ok(DcsKey::Leader)
        );
        assert_eq!(
            key_from_path("scope-a", "/scope-a/switchover"),
            Ok(DcsKey::Switchover)
        );
        assert_eq!(
            key_from_path("scope-a", "/scope-a/config"),
            Ok(DcsKey::Config)
        );
        assert_eq!(
            key_from_path("scope-a", "/scope-a/init"),
            Ok(DcsKey::InitLock)
        );
    }

    #[test]
    fn rejects_wrong_scope() {
        let parsed = key_from_path("scope-a", "/scope-b/leader");
        assert!(matches!(
            parsed,
            Err(DcsKeyParseError::InvalidScopePrefix { .. })
        ));
    }

    #[test]
    fn rejects_missing_member_id() {
        let parsed = key_from_path("scope-a", "/scope-a/member/");
        assert_eq!(
            parsed,
            Err(DcsKeyParseError::MissingMemberId(
                "/scope-a/member/".to_string()
            ))
        );
    }

    #[test]
    fn rejects_unknown_and_extra_segments() {
        assert_eq!(
            key_from_path("scope-a", "/scope-a/nope"),
            Err(DcsKeyParseError::UnknownKey("/scope-a/nope".to_string()))
        );
        assert_eq!(
            key_from_path("scope-a", "/scope-a/leader/extra"),
            Err(DcsKeyParseError::UnknownKey(
                "/scope-a/leader/extra".to_string()
            ))
        );
    }
}

--- END FILE: src/dcs/keys.rs ---

--- BEGIN FILE: src/dcs/state.rs ---
use std::collections::BTreeMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{
    config::RuntimeConfig,
    logging::LogHandle,
    pginfo::state::{PgInfoState, Readiness, SqlStatus},
    state::{
        MemberId, StatePublisher, StateSubscriber, TimelineId, UnixMillis, Version, WalLsn,
        WorkerStatus,
    },
};

use super::store::DcsStore;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DcsTrust {
    FullQuorum,
    FailSafe,
    NotTrusted,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum MemberRole {
    Unknown,
    Primary,
    Replica,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct MemberRecord {
    pub(crate) member_id: MemberId,
    pub(crate) postgres_host: String,
    pub(crate) postgres_port: u16,
    pub(crate) role: MemberRole,
    pub(crate) sql: SqlStatus,
    pub(crate) readiness: Readiness,
    pub(crate) timeline: Option<TimelineId>,
    pub(crate) write_lsn: Option<WalLsn>,
    pub(crate) replay_lsn: Option<WalLsn>,
    pub(crate) updated_at: UnixMillis,
    pub(crate) pg_version: Version,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct LeaderRecord {
    pub(crate) member_id: MemberId,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SwitchoverRequest {
    pub(crate) requested_by: MemberId,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct InitLockRecord {
    pub(crate) holder: MemberId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DcsCache {
    pub(crate) members: BTreeMap<MemberId, MemberRecord>,
    pub(crate) leader: Option<LeaderRecord>,
    pub(crate) switchover: Option<SwitchoverRequest>,
    pub(crate) config: RuntimeConfig,
    pub(crate) init_lock: Option<InitLockRecord>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DcsState {
    pub(crate) worker: WorkerStatus,
    pub(crate) trust: DcsTrust,
    pub(crate) cache: DcsCache,
    pub(crate) last_refresh_at: Option<UnixMillis>,
}

pub(crate) struct DcsWorkerCtx {
    pub(crate) self_id: MemberId,
    pub(crate) scope: String,
    pub(crate) poll_interval: Duration,
    pub(crate) local_postgres_host: String,
    pub(crate) local_postgres_port: u16,
    pub(crate) pg_subscriber: StateSubscriber<PgInfoState>,
    pub(crate) publisher: StatePublisher<DcsState>,
    pub(crate) store: Box<dyn DcsStore>,
    pub(crate) log: LogHandle,
    pub(crate) cache: DcsCache,
    pub(crate) last_published_pg_version: Option<Version>,
    pub(crate) last_emitted_store_healthy: Option<bool>,
    pub(crate) last_emitted_trust: Option<DcsTrust>,
}

pub(crate) fn evaluate_trust(
    etcd_healthy: bool,
    cache: &DcsCache,
    self_id: &MemberId,
    now: UnixMillis,
) -> DcsTrust {
    if !etcd_healthy {
        return DcsTrust::NotTrusted;
    }

    let Some(self_member) = cache.members.get(self_id) else {
        return DcsTrust::FailSafe;
    };
    if !member_record_is_fresh(self_member, cache, now) {
        return DcsTrust::FailSafe;
    }

    if let Some(leader) = &cache.leader {
        let Some(leader_member) = cache.members.get(&leader.member_id) else {
            return DcsTrust::FailSafe;
        };
        if !member_record_is_fresh(leader_member, cache, now) {
            return DcsTrust::FailSafe;
        }
    }

    if cache.members.len() > 1 && fresh_member_count(cache, now) < 2 {
        return DcsTrust::FailSafe;
    }

    DcsTrust::FullQuorum
}

fn member_record_is_fresh(record: &MemberRecord, cache: &DcsCache, now: UnixMillis) -> bool {
    let max_age_ms = cache.config.ha.lease_ttl_ms;
    now.0.saturating_sub(record.updated_at.0) <= max_age_ms
}

fn fresh_member_count(cache: &DcsCache, now: UnixMillis) -> usize {
    cache
        .members
        .values()
        .filter(|record| member_record_is_fresh(record, cache, now))
        .count()
}

pub(crate) fn build_local_member_record(
    self_id: &MemberId,
    postgres_host: &str,
    postgres_port: u16,
    pg_state: &PgInfoState,
    now: UnixMillis,
    pg_version: Version,
) -> MemberRecord {
    match pg_state {
        PgInfoState::Unknown { common } => MemberRecord {
            member_id: self_id.clone(),
            postgres_host: postgres_host.to_string(),
            postgres_port,
            role: MemberRole::Unknown,
            sql: common.sql.clone(),
            readiness: common.readiness.clone(),
            timeline: common.timeline,
            write_lsn: None,
            replay_lsn: None,
            updated_at: now,
            pg_version,
        },
        PgInfoState::Primary {
            common, wal_lsn, ..
        } => MemberRecord {
            member_id: self_id.clone(),
            postgres_host: postgres_host.to_string(),
            postgres_port,
            role: MemberRole::Primary,
            sql: common.sql.clone(),
            readiness: common.readiness.clone(),
            timeline: common.timeline,
            write_lsn: Some(*wal_lsn),
            replay_lsn: None,
            updated_at: now,
            pg_version,
        },
        PgInfoState::Replica {
            common, replay_lsn, ..
        } => MemberRecord {
            member_id: self_id.clone(),
            postgres_host: postgres_host.to_string(),
            postgres_port,
            role: MemberRole::Replica,
            sql: common.sql.clone(),
            readiness: common.readiness.clone(),
            timeline: common.timeline,
            write_lsn: None,
            replay_lsn: Some(*replay_lsn),
            updated_at: now,
            pg_version,
        },
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        config::RuntimeConfig,
        pginfo::state::{PgConfig, PgInfoCommon, ReplicationSlotInfo},
        state::{Version, WorkerStatus},
    };

    use super::{
        build_local_member_record, evaluate_trust, DcsCache, DcsTrust, LeaderRecord, MemberRecord,
        MemberRole,
    };
    use crate::{
        pginfo::state::{PgInfoState, Readiness, SqlStatus},
        state::{MemberId, TimelineId, UnixMillis, WalLsn},
    };

    fn sample_runtime_config() -> RuntimeConfig {
        crate::test_harness::runtime_config::sample_runtime_config()
    }

    fn sample_cache() -> DcsCache {
        DcsCache {
            members: BTreeMap::new(),
            leader: None,
            switchover: None,
            config: sample_runtime_config(),
            init_lock: None,
        }
    }

    #[test]
    fn evaluate_trust_covers_all_outcomes() {
        let self_id = MemberId("node-a".to_string());
        let mut cache = sample_cache();

        assert_eq!(
            evaluate_trust(false, &cache, &self_id, UnixMillis(1)),
            DcsTrust::NotTrusted
        );
        assert_eq!(
            evaluate_trust(true, &cache, &self_id, UnixMillis(1)),
            DcsTrust::FailSafe
        );

        cache.members.insert(
            self_id.clone(),
            MemberRecord {
                member_id: self_id.clone(),
                postgres_host: "127.0.0.1".to_string(),
                postgres_port: 5432,
                role: MemberRole::Unknown,
                sql: SqlStatus::Unknown,
                readiness: Readiness::Unknown,
                timeline: None,
                write_lsn: None,
                replay_lsn: None,
                updated_at: UnixMillis(1),
                pg_version: Version(1),
            },
        );
        assert_eq!(
            evaluate_trust(true, &cache, &self_id, UnixMillis(1)),
            DcsTrust::FullQuorum
        );
        assert_eq!(
            evaluate_trust(true, &cache, &self_id, UnixMillis(20_000)),
            DcsTrust::FailSafe
        );

        cache.leader = Some(LeaderRecord {
            member_id: MemberId("node-b".to_string()),
        });
        assert_eq!(
            evaluate_trust(true, &cache, &self_id, UnixMillis(1)),
            DcsTrust::FailSafe
        );
    }

    fn common(sql: SqlStatus, readiness: Readiness) -> PgInfoCommon {
        PgInfoCommon {
            worker: WorkerStatus::Running,
            sql,
            readiness,
            timeline: Some(TimelineId(4)),
            pg_config: PgConfig {
                port: None,
                hot_standby: None,
                primary_conninfo: None,
                primary_slot_name: None,
                extra: BTreeMap::new(),
            },
            last_refresh_at: Some(UnixMillis(9)),
        }
    }

    #[test]
    fn build_local_member_record_maps_pg_variants() {
        let self_id = MemberId("node-a".to_string());
        let unknown = PgInfoState::Unknown {
            common: common(SqlStatus::Unknown, Readiness::Unknown),
        };
        let unknown_record = build_local_member_record(
            &self_id,
            "10.0.0.11",
            5433,
            &unknown,
            UnixMillis(10),
            Version(11),
        );
        assert_eq!(unknown_record.postgres_host, "10.0.0.11".to_string());
        assert_eq!(unknown_record.postgres_port, 5433);
        assert_eq!(unknown_record.role, MemberRole::Unknown);
        assert_eq!(unknown_record.write_lsn, None);
        assert_eq!(unknown_record.replay_lsn, None);

        let primary = PgInfoState::Primary {
            common: common(SqlStatus::Healthy, Readiness::Ready),
            wal_lsn: WalLsn(101),
            slots: vec![ReplicationSlotInfo {
                name: "slot-a".to_string(),
            }],
        };
        let primary_record = build_local_member_record(
            &self_id,
            "10.0.0.12",
            5434,
            &primary,
            UnixMillis(12),
            Version(13),
        );
        assert_eq!(primary_record.postgres_host, "10.0.0.12".to_string());
        assert_eq!(primary_record.postgres_port, 5434);
        assert_eq!(primary_record.role, MemberRole::Primary);
        assert_eq!(primary_record.write_lsn, Some(WalLsn(101)));
        assert_eq!(primary_record.replay_lsn, None);

        let replica = PgInfoState::Replica {
            common: common(SqlStatus::Healthy, Readiness::Ready),
            replay_lsn: WalLsn(22),
            follow_lsn: Some(WalLsn(23)),
            upstream: None,
        };
        let replica_record = build_local_member_record(
            &self_id,
            "10.0.0.13",
            5435,
            &replica,
            UnixMillis(14),
            Version(15),
        );
        assert_eq!(replica_record.postgres_host, "10.0.0.13".to_string());
        assert_eq!(replica_record.postgres_port, 5435);
        assert_eq!(replica_record.role, MemberRole::Replica);
        assert_eq!(replica_record.write_lsn, None);
        assert_eq!(replica_record.replay_lsn, Some(WalLsn(22)));
    }
}

--- END FILE: src/dcs/state.rs ---

--- BEGIN FILE: src/dcs/store.rs ---
use thiserror::Error;

use super::{
    keys::{key_from_path, DcsKey, DcsKeyParseError},
    state::{DcsCache, InitLockRecord, LeaderRecord, MemberRecord, SwitchoverRequest},
    worker::{apply_watch_update, DcsWatchUpdate},
};
use crate::state::MemberId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WatchOp {
    Put,
    Delete,
    /// Indicates that the watch consumer should treat the following snapshot as authoritative
    /// and reset any previously cached DCS state for this scope.
    ///
    /// This is synthesized by the etcd store during reconnect/resnapshot and does not come from
    /// etcd itself.
    Reset,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WatchEvent {
    pub op: WatchOp,
    pub path: String,
    pub value: Option<String>,
    pub revision: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RefreshResult {
    pub(crate) applied: usize,
    pub(crate) had_errors: bool,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum DcsStoreError {
    #[error("watch value missing for put event at `{0}`")]
    MissingValue(String),
    #[error("invalid key path: {0}")]
    InvalidKey(#[from] DcsKeyParseError),
    #[error("decode failed for key `{key}`: {message}")]
    Decode { key: String, message: String },
    #[error("path already exists: {0}")]
    AlreadyExists(String),
    #[error("store I/O error: {0}")]
    Io(String),
}

pub trait DcsStore: Send {
    fn healthy(&self) -> bool;
    fn read_path(&mut self, path: &str) -> Result<Option<String>, DcsStoreError>;
    fn write_path(&mut self, path: &str, value: String) -> Result<(), DcsStoreError>;
    fn put_path_if_absent(&mut self, path: &str, value: String) -> Result<bool, DcsStoreError>;
    fn delete_path(&mut self, path: &str) -> Result<(), DcsStoreError>;
    fn drain_watch_events(&mut self) -> Result<Vec<WatchEvent>, DcsStoreError>;
}

pub(crate) trait DcsHaWriter: Send {
    fn write_leader_lease(
        &mut self,
        scope: &str,
        member_id: &MemberId,
    ) -> Result<(), DcsStoreError>;
    fn delete_leader(&mut self, scope: &str) -> Result<(), DcsStoreError>;
    fn clear_switchover(&mut self, scope: &str) -> Result<(), DcsStoreError>;
}

impl<T> DcsHaWriter for T
where
    T: DcsStore + ?Sized,
{
    fn write_leader_lease(
        &mut self,
        scope: &str,
        member_id: &MemberId,
    ) -> Result<(), DcsStoreError> {
        let path = leader_path(scope);
        let encoded = serde_json::to_string(&LeaderRecord {
            member_id: member_id.clone(),
        })
        .map_err(|err| DcsStoreError::Decode {
            key: path.clone(),
            message: err.to_string(),
        })?;
        if self.put_path_if_absent(&path, encoded)? {
            Ok(())
        } else {
            Err(DcsStoreError::AlreadyExists(path))
        }
    }

    fn delete_leader(&mut self, scope: &str) -> Result<(), DcsStoreError> {
        self.delete_path(&leader_path(scope))
    }

    fn clear_switchover(&mut self, scope: &str) -> Result<(), DcsStoreError> {
        self.delete_path(&switchover_path(scope))
    }
}

fn leader_path(scope: &str) -> String {
    format!("/{}/leader", scope.trim_matches('/'))
}

fn switchover_path(scope: &str) -> String {
    format!("/{}/switchover", scope.trim_matches('/'))
}

pub(crate) fn write_local_member(
    store: &mut dyn DcsStore,
    scope: &str,
    member: &MemberRecord,
) -> Result<(), DcsStoreError> {
    let path = format!("/{}/member/{}", scope.trim_matches('/'), member.member_id.0);
    let encoded = serde_json::to_string(member).map_err(|err| DcsStoreError::Decode {
        key: path.clone(),
        message: err.to_string(),
    })?;
    store.write_path(&path, encoded)?;
    Ok(())
}

pub(crate) fn refresh_from_etcd_watch(
    scope: &str,
    cache: &mut DcsCache,
    events: Vec<WatchEvent>,
) -> Result<RefreshResult, DcsStoreError> {
    let mut applied = 0usize;
    let mut had_errors = false;

    for event in events {
        if event.op == WatchOp::Reset {
            cache.members.clear();
            cache.leader = None;
            cache.switchover = None;
            cache.init_lock = None;
            applied = applied.saturating_add(1);
            continue;
        }

        let key = match key_from_path(scope, &event.path) {
            Ok(parsed) => parsed,
            Err(err) => match err {
                DcsKeyParseError::UnknownKey(_) => {
                    had_errors = true;
                    continue;
                }
                other => return Err(DcsStoreError::InvalidKey(other)),
            },
        };

        let update = match event.op {
            WatchOp::Delete => DcsWatchUpdate::Delete { key },
            WatchOp::Put => {
                let raw_value = match event.value {
                    Some(value) => value,
                    None => return Err(DcsStoreError::MissingValue(event.path)),
                };
                let value = decode_watch_value(&key, &raw_value, &event.path)?;
                DcsWatchUpdate::Put {
                    key,
                    value: Box::new(value),
                }
            }
            WatchOp::Reset => {
                // Handled above, before key parsing.
                continue;
            }
        };

        apply_watch_update(cache, update);
        applied = applied.saturating_add(1);
    }

    Ok(RefreshResult {
        applied,
        had_errors,
    })
}

fn decode_watch_value(
    key: &DcsKey,
    raw: &str,
    path: &str,
) -> Result<super::worker::DcsValue, DcsStoreError> {
    match key {
        DcsKey::Member(_) => serde_json::from_str::<MemberRecord>(raw)
            .map(super::worker::DcsValue::Member)
            .map_err(|err| DcsStoreError::Decode {
                key: path.to_string(),
                message: err.to_string(),
            }),
        DcsKey::Leader => serde_json::from_str::<LeaderRecord>(raw)
            .map(super::worker::DcsValue::Leader)
            .map_err(|err| DcsStoreError::Decode {
                key: path.to_string(),
                message: err.to_string(),
            }),
        DcsKey::Switchover => serde_json::from_str::<SwitchoverRequest>(raw)
            .map(super::worker::DcsValue::Switchover)
            .map_err(|err| DcsStoreError::Decode {
                key: path.to_string(),
                message: err.to_string(),
            }),
        DcsKey::Config => serde_json::from_str::<crate::config::RuntimeConfig>(raw)
            .map(|cfg| super::worker::DcsValue::Config(Box::new(cfg)))
            .map_err(|err| DcsStoreError::Decode {
                key: path.to_string(),
                message: err.to_string(),
            }),
        DcsKey::InitLock => serde_json::from_str::<InitLockRecord>(raw)
            .map(super::worker::DcsValue::InitLock)
            .map_err(|err| DcsStoreError::Decode {
                key: path.to_string(),
                message: err.to_string(),
            }),
    }
}

#[cfg(test)]
use std::collections::VecDeque;

#[cfg(test)]
#[derive(Default)]
pub(crate) struct TestDcsStore {
    healthy: bool,
    events: VecDeque<WatchEvent>,
    kv: std::collections::BTreeMap<String, String>,
    writes: Vec<(String, String)>,
    deletes: Vec<String>,
}

#[cfg(test)]
impl TestDcsStore {
    pub(crate) fn new(healthy: bool) -> Self {
        Self {
            healthy,
            events: VecDeque::new(),
            kv: std::collections::BTreeMap::new(),
            writes: Vec::new(),
            deletes: Vec::new(),
        }
    }

    pub(crate) fn push_event(&mut self, event: WatchEvent) {
        self.events.push_back(event);
    }

    pub(crate) fn writes(&self) -> &[(String, String)] {
        &self.writes
    }

    pub(crate) fn deletes(&self) -> &[String] {
        &self.deletes
    }
}

#[cfg(test)]
impl DcsStore for TestDcsStore {
    fn healthy(&self) -> bool {
        self.healthy
    }

    fn read_path(&mut self, path: &str) -> Result<Option<String>, DcsStoreError> {
        Ok(self.kv.get(path).cloned())
    }

    fn write_path(&mut self, path: &str, value: String) -> Result<(), DcsStoreError> {
        self.kv.insert(path.to_string(), value.clone());
        self.writes.push((path.to_string(), value));
        Ok(())
    }

    fn put_path_if_absent(&mut self, path: &str, value: String) -> Result<bool, DcsStoreError> {
        if self.kv.contains_key(path) {
            return Ok(false);
        }
        self.kv.insert(path.to_string(), value.clone());
        self.writes.push((path.to_string(), value));
        Ok(true)
    }

    fn delete_path(&mut self, path: &str) -> Result<(), DcsStoreError> {
        self.kv.remove(path);
        self.deletes.push(path.to_string());
        Ok(())
    }

    fn drain_watch_events(&mut self) -> Result<Vec<WatchEvent>, DcsStoreError> {
        Ok(self.events.drain(..).collect())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        config::RuntimeConfig,
        dcs::{
            state::{DcsCache, MemberRecord, MemberRole},
            worker::DcsValue,
        },
        pginfo::state::{Readiness, SqlStatus},
        state::{MemberId, UnixMillis, Version},
    };

    use super::{
        refresh_from_etcd_watch, write_local_member, DcsHaWriter, DcsStore, DcsStoreError,
        RefreshResult, TestDcsStore, WatchEvent, WatchOp,
    };

    fn sample_runtime_config() -> RuntimeConfig {
        crate::test_harness::runtime_config::sample_runtime_config()
    }

    fn sample_cache() -> DcsCache {
        DcsCache {
            members: BTreeMap::new(),
            leader: None,
            switchover: None,
            config: sample_runtime_config(),
            init_lock: None,
        }
    }

    #[test]
    fn write_local_member_writes_only_member_path() {
        let mut store = TestDcsStore::new(true);
        let member = MemberRecord {
            member_id: MemberId("node-a".to_string()),
            postgres_host: "10.0.0.10".to_string(),
            postgres_port: 5432,
            role: MemberRole::Primary,
            sql: SqlStatus::Healthy,
            readiness: Readiness::Ready,
            timeline: None,
            write_lsn: None,
            replay_lsn: None,
            updated_at: UnixMillis(10),
            pg_version: Version(7),
        };
        let wrote = write_local_member(&mut store, "scope-a", &member);
        assert_eq!(wrote, Ok(()));
        assert_eq!(store.writes().len(), 1);
        assert_eq!(store.writes()[0].0, "/scope-a/member/node-a");
        assert!(store.writes()[0].1.contains("\"member_id\""));
    }

    #[test]
    fn refresh_applies_member_put_and_delete() -> Result<(), Box<dyn std::error::Error>> {
        let mut cache = sample_cache();
        let mut store = TestDcsStore::new(true);
        let encoded = serde_json::to_string(&MemberRecord {
            member_id: MemberId("node-a".to_string()),
            postgres_host: "10.0.0.11".to_string(),
            postgres_port: 5433,
            role: MemberRole::Replica,
            sql: SqlStatus::Healthy,
            readiness: Readiness::Ready,
            timeline: None,
            write_lsn: None,
            replay_lsn: None,
            updated_at: UnixMillis(10),
            pg_version: Version(1),
        })?;
        store.push_event(WatchEvent {
            op: WatchOp::Put,
            path: "/scope-a/member/node-a".to_string(),
            value: Some(encoded),
            revision: 1,
        });
        store.push_event(WatchEvent {
            op: WatchOp::Delete,
            path: "/scope-a/member/node-a".to_string(),
            value: None,
            revision: 2,
        });

        let events = store.drain_watch_events()?;
        let refreshed = refresh_from_etcd_watch("scope-a", &mut cache, events);
        assert!(refreshed.is_ok());
        assert!(cache.members.is_empty());
        Ok(())
    }

    #[test]
    fn refresh_reports_decode_error() {
        let mut cache = sample_cache();
        let result = refresh_from_etcd_watch(
            "scope-a",
            &mut cache,
            vec![WatchEvent {
                op: WatchOp::Put,
                path: "/scope-a/member/node-a".to_string(),
                value: Some("{\"bad\":1}".to_string()),
                revision: 1,
            }],
        );
        assert!(matches!(result, Err(DcsStoreError::Decode { .. })));
    }

    #[test]
    fn refresh_sets_had_errors_for_unknown_keys_and_applies_known_updates() {
        let mut cache = sample_cache();
        let result = refresh_from_etcd_watch(
            "scope-a",
            &mut cache,
            vec![
                WatchEvent {
                    op: WatchOp::Put,
                    path: "/scope-a/not-a-real-key".to_string(),
                    value: Some("{\"ignored\":true}".to_string()),
                    revision: 1,
                },
                WatchEvent {
                    op: WatchOp::Put,
                    path: "/scope-a/leader".to_string(),
                    value: Some("{\"member_id\":\"node-a\"}".to_string()),
                    revision: 2,
                },
            ],
        );

        assert!(matches!(
            result,
            Ok(RefreshResult {
                had_errors: true,
                applied: 1
            })
        ));
        assert_eq!(
            cache.leader,
            Some(crate::dcs::state::LeaderRecord {
                member_id: MemberId("node-a".to_string())
            })
        );
    }

    #[test]
    fn refresh_reset_clears_cached_records_but_preserves_config(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut cache = sample_cache();
        let preserved_config = cache.config.clone();

        cache.members.insert(
            MemberId("node-stale".to_string()),
            MemberRecord {
                member_id: MemberId("node-stale".to_string()),
                postgres_host: "10.0.0.12".to_string(),
                postgres_port: 5434,
                role: MemberRole::Replica,
                sql: SqlStatus::Healthy,
                readiness: Readiness::Ready,
                timeline: None,
                write_lsn: None,
                replay_lsn: None,
                updated_at: UnixMillis(10),
                pg_version: Version(1),
            },
        );
        cache.leader = Some(crate::dcs::state::LeaderRecord {
            member_id: MemberId("node-stale".to_string()),
        });
        cache.switchover = Some(crate::dcs::state::SwitchoverRequest {
            requested_by: MemberId("node-stale".to_string()),
        });
        cache.init_lock = Some(crate::dcs::state::InitLockRecord {
            holder: MemberId("node-stale".to_string()),
        });

        let result = refresh_from_etcd_watch(
            "scope-a",
            &mut cache,
            vec![WatchEvent {
                op: WatchOp::Reset,
                path: "/scope-a".to_string(),
                value: None,
                revision: 42,
            }],
        )?;

        assert_eq!(
            result,
            RefreshResult {
                applied: 1,
                had_errors: false
            }
        );
        assert!(cache.members.is_empty());
        assert!(cache.leader.is_none());
        assert!(cache.switchover.is_none());
        assert!(cache.init_lock.is_none());
        assert_eq!(cache.config, preserved_config);

        Ok(())
    }

    #[test]
    fn refresh_put_then_reset_then_put_keeps_only_post_reset_state(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut cache = sample_cache();

        let stale_json = serde_json::to_string(&crate::dcs::state::LeaderRecord {
            member_id: MemberId("node-stale".to_string()),
        })?;
        let fresh_json = serde_json::to_string(&crate::dcs::state::LeaderRecord {
            member_id: MemberId("node-fresh".to_string()),
        })?;

        let result = refresh_from_etcd_watch(
            "scope-a",
            &mut cache,
            vec![
                WatchEvent {
                    op: WatchOp::Put,
                    path: "/scope-a/leader".to_string(),
                    value: Some(stale_json),
                    revision: 1,
                },
                WatchEvent {
                    op: WatchOp::Reset,
                    path: "/scope-a".to_string(),
                    value: None,
                    revision: 2,
                },
                WatchEvent {
                    op: WatchOp::Put,
                    path: "/scope-a/leader".to_string(),
                    value: Some(fresh_json),
                    revision: 3,
                },
            ],
        )?;

        assert_eq!(
            result,
            RefreshResult {
                applied: 3,
                had_errors: false
            }
        );
        assert_eq!(
            cache.leader,
            Some(crate::dcs::state::LeaderRecord {
                member_id: MemberId("node-fresh".to_string())
            })
        );

        Ok(())
    }

    #[test]
    fn dcs_value_type_is_exercised_to_keep_contracts_live() {
        let _value = DcsValue::Leader(crate::dcs::state::LeaderRecord {
            member_id: MemberId("node-a".to_string()),
        });
    }

    #[test]
    fn write_leader_lease_writes_leader_path_and_payload() {
        let mut store = TestDcsStore::new(true);
        let result =
            DcsHaWriter::write_leader_lease(&mut store, "scope-a", &MemberId("node-a".to_string()));
        assert_eq!(result, Ok(()));
        assert_eq!(store.writes().len(), 1);
        assert_eq!(store.writes()[0].0, "/scope-a/leader");
        assert!(store.writes()[0].1.contains("\"member_id\":\"node-a\""));
    }

    #[test]
    fn write_leader_lease_rejects_existing_leader() {
        let mut store = TestDcsStore::new(true);
        let first =
            DcsHaWriter::write_leader_lease(&mut store, "scope-a", &MemberId("node-a".to_string()));
        let second =
            DcsHaWriter::write_leader_lease(&mut store, "scope-a", &MemberId("node-b".to_string()));

        assert_eq!(first, Ok(()));
        assert_eq!(
            second,
            Err(DcsStoreError::AlreadyExists("/scope-a/leader".to_string()))
        );
        assert_eq!(store.writes().len(), 1);
        assert!(store.writes()[0].1.contains("\"member_id\":\"node-a\""));
        assert_eq!(
            store.read_path("/scope-a/leader"),
            Ok(Some("{\"member_id\":\"node-a\"}".to_string()))
        );
    }

    #[test]
    fn delete_leader_deletes_leader_key() {
        let mut store = TestDcsStore::new(true);
        let result = DcsHaWriter::delete_leader(&mut store, "scope-a");
        assert_eq!(result, Ok(()));
        assert_eq!(store.deletes(), &["/scope-a/leader".to_string()]);
    }

    #[test]
    fn clear_switchover_deletes_switchover_key() {
        let mut store = TestDcsStore::new(true);
        let result = DcsHaWriter::clear_switchover(&mut store, "scope-a");
        assert_eq!(result, Ok(()));
        assert_eq!(store.deletes(), &["/scope-a/switchover".to_string()]);
    }
}

--- END FILE: src/dcs/store.rs ---

--- BEGIN FILE: src/dcs/worker.rs ---
use crate::{
    logging::{AppEvent, AppEventHeader, SeverityText, StructuredFields},
    state::WorkerError,
};

use super::{
    keys::DcsKey,
    state::{
        build_local_member_record, evaluate_trust, DcsCache, DcsState, DcsTrust, DcsWorkerCtx,
        InitLockRecord, LeaderRecord, MemberRecord, SwitchoverRequest,
    },
    store::{refresh_from_etcd_watch, write_local_member},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DcsValue {
    Member(MemberRecord),
    Leader(LeaderRecord),
    Switchover(SwitchoverRequest),
    Config(Box<crate::config::RuntimeConfig>),
    InitLock(InitLockRecord),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DcsWatchUpdate {
    Put { key: DcsKey, value: Box<DcsValue> },
    Delete { key: DcsKey },
}

fn dcs_append_base_fields(fields: &mut StructuredFields, ctx: &DcsWorkerCtx) {
    fields.insert("scope", ctx.scope.clone());
    fields.insert("member_id", ctx.self_id.0.clone());
}

fn dcs_event(severity: SeverityText, message: &str, name: &str, result: &str) -> AppEvent {
    AppEvent::new(severity, message, AppEventHeader::new(name, "dcs", result))
}

fn emit_dcs_event(
    ctx: &DcsWorkerCtx,
    origin: &str,
    event: AppEvent,
    error_prefix: &str,
) -> Result<(), WorkerError> {
    ctx.log
        .emit_app_event(origin, event)
        .map_err(|err| WorkerError::Message(format!("{error_prefix}: {err}")))
}

fn dcs_io_error_severity(err: &crate::dcs::store::DcsStoreError) -> SeverityText {
    match err {
        crate::dcs::store::DcsStoreError::Io(_) => SeverityText::Warn,
        _ => SeverityText::Error,
    }
}

fn dcs_refresh_error_severity(err: &crate::dcs::store::DcsStoreError) -> SeverityText {
    match err {
        crate::dcs::store::DcsStoreError::Io(_)
        | crate::dcs::store::DcsStoreError::InvalidKey(_)
        | crate::dcs::store::DcsStoreError::MissingValue(_) => SeverityText::Warn,
        _ => SeverityText::Error,
    }
}

pub(crate) async fn run(mut ctx: DcsWorkerCtx) -> Result<(), WorkerError> {
    loop {
        step_once(&mut ctx).await?;
        tokio::time::sleep(ctx.poll_interval).await;
    }
}

pub(crate) fn apply_watch_update(cache: &mut DcsCache, update: DcsWatchUpdate) {
    match update {
        DcsWatchUpdate::Put { key, value } => match (key, *value) {
            (DcsKey::Member(member_id), DcsValue::Member(record)) => {
                cache.members.insert(member_id, record);
            }
            (DcsKey::Leader, DcsValue::Leader(record)) => {
                cache.leader = Some(record);
            }
            (DcsKey::Switchover, DcsValue::Switchover(record)) => {
                cache.switchover = Some(record);
            }
            (DcsKey::Config, DcsValue::Config(config)) => {
                cache.config = *config;
            }
            (DcsKey::InitLock, DcsValue::InitLock(record)) => {
                cache.init_lock = Some(record);
            }
            _ => {}
        },
        DcsWatchUpdate::Delete { key } => match key {
            DcsKey::Member(member_id) => {
                cache.members.remove(&member_id);
            }
            DcsKey::Leader => {
                cache.leader = None;
            }
            DcsKey::Switchover => {
                cache.switchover = None;
            }
            DcsKey::Config => {}
            DcsKey::InitLock => {
                cache.init_lock = None;
            }
        },
    }
}

pub(crate) async fn step_once(ctx: &mut DcsWorkerCtx) -> Result<(), WorkerError> {
    let now = now_unix_millis()?;
    let pg_snapshot = ctx.pg_subscriber.latest();

    let mut store_healthy = ctx.store.healthy();
    let must_publish_local_member = store_healthy;
    let mut local_member_publish_succeeded = false;

    if must_publish_local_member {
        let local_member = build_local_member_record(
            &ctx.self_id,
            ctx.local_postgres_host.as_str(),
            ctx.local_postgres_port,
            &pg_snapshot.value,
            now,
            pg_snapshot.version,
        );
        match write_local_member(ctx.store.as_mut(), &ctx.scope, &local_member) {
            Ok(()) => {
                ctx.last_published_pg_version = Some(pg_snapshot.version);
                ctx.cache.members.insert(ctx.self_id.clone(), local_member);
                local_member_publish_succeeded = true;
            }
            Err(err) => {
                let mut event = dcs_event(
                    dcs_io_error_severity(&err),
                    "dcs local member write failed",
                    "dcs.local_member.write_failed",
                    "failed",
                );
                let fields = event.fields_mut();
                dcs_append_base_fields(fields, ctx);
                fields.insert("error", err.to_string());
                emit_dcs_event(
                    ctx,
                    "dcs_worker::step_once",
                    event,
                    "dcs local member write log emit failed",
                )?;
                store_healthy = false;
            }
        }
    }

    let events = match ctx.store.drain_watch_events() {
        Ok(events) => events,
        Err(err) => {
            let mut event = dcs_event(
                dcs_io_error_severity(&err),
                "dcs watch drain failed",
                "dcs.watch.drain_failed",
                "failed",
            );
            let fields = event.fields_mut();
            dcs_append_base_fields(fields, ctx);
            fields.insert("error", err.to_string());
            emit_dcs_event(
                ctx,
                "dcs_worker::step_once",
                event,
                "dcs drain log emit failed",
            )?;
            store_healthy = false;
            Vec::new()
        }
    };
    match refresh_from_etcd_watch(&ctx.scope, &mut ctx.cache, events) {
        Ok(result) => {
            if result.had_errors {
                let mut event = dcs_event(
                    SeverityText::Warn,
                    "dcs watch refresh had errors",
                    "dcs.watch.apply_had_errors",
                    "failed",
                );
                let fields = event.fields_mut();
                dcs_append_base_fields(fields, ctx);
                fields.insert("applied", result.applied);
                emit_dcs_event(
                    ctx,
                    "dcs_worker::step_once",
                    event,
                    "dcs refresh had_errors log emit failed",
                )?;
                store_healthy = false;
            }
        }
        Err(err) => {
            let mut event = dcs_event(
                dcs_refresh_error_severity(&err),
                "dcs watch refresh failed",
                "dcs.watch.refresh_failed",
                "failed",
            );
            let fields = event.fields_mut();
            dcs_append_base_fields(fields, ctx);
            fields.insert("error", err.to_string());
            emit_dcs_event(
                ctx,
                "dcs_worker::step_once",
                event,
                "dcs refresh log emit failed",
            )?;
            store_healthy = false;
        }
    }

    let trust = if local_member_publish_succeeded {
        evaluate_trust(store_healthy, &ctx.cache, &ctx.self_id, now)
    } else {
        DcsTrust::NotTrusted
    };
    let worker = if store_healthy {
        crate::state::WorkerStatus::Running
    } else {
        crate::state::WorkerStatus::Faulted(WorkerError::Message("dcs store unhealthy".to_string()))
    };

    let next = DcsState {
        worker,
        trust: if store_healthy {
            trust
        } else {
            DcsTrust::NotTrusted
        },
        cache: ctx.cache.clone(),
        last_refresh_at: Some(now),
    };
    if ctx.last_emitted_store_healthy != Some(store_healthy) {
        ctx.last_emitted_store_healthy = Some(store_healthy);
        let mut event = dcs_event(
            if store_healthy {
                SeverityText::Info
            } else {
                SeverityText::Warn
            },
            "dcs store health transition",
            "dcs.store.health_transition",
            if store_healthy { "recovered" } else { "failed" },
        );
        let fields = event.fields_mut();
        dcs_append_base_fields(fields, ctx);
        fields.insert("store_healthy", store_healthy);
        emit_dcs_event(
            ctx,
            "dcs_worker::step_once",
            event,
            "dcs health transition log emit failed",
        )?;
    }
    if ctx.last_emitted_trust.as_ref() != Some(&next.trust) {
        let prev = ctx
            .last_emitted_trust
            .as_ref()
            .map(|value| format!("{value:?}").to_lowercase())
            .unwrap_or_else(|| "unknown".to_string());
        ctx.last_emitted_trust = Some(next.trust.clone());
        let mut event = dcs_event(
            SeverityText::Info,
            "dcs trust transition",
            "dcs.trust.transition",
            "ok",
        );
        let fields = event.fields_mut();
        dcs_append_base_fields(fields, ctx);
        fields.insert("trust_prev", prev);
        fields.insert("trust_next", format!("{:?}", next.trust).to_lowercase());
        emit_dcs_event(
            ctx,
            "dcs_worker::step_once",
            event,
            "dcs trust transition log emit failed",
        )?;
    }
    ctx.publisher
        .publish(next, now)
        .map_err(|err| WorkerError::Message(format!("dcs publish failed: {err}")))?;
    Ok(())
}

fn now_unix_millis() -> Result<crate::state::UnixMillis, WorkerError> {
    let elapsed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|err| WorkerError::Message(format!("system clock before unix epoch: {err}")))?;
    let millis = u64::try_from(elapsed.as_millis())
        .map_err(|err| WorkerError::Message(format!("unix millis conversion failed: {err}")))?;
    Ok(crate::state::UnixMillis(millis))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use crate::{
        config::RuntimeConfig,
        dcs::{
            keys::DcsKey,
            state::{
                DcsCache, DcsState, DcsTrust, DcsWorkerCtx, InitLockRecord, LeaderRecord,
                MemberRecord, MemberRole, SwitchoverRequest,
            },
            store::{DcsStore, DcsStoreError, WatchEvent, WatchOp},
            worker::{apply_watch_update, DcsValue, DcsWatchUpdate},
        },
        logging::{decode_app_event, LogHandle, LogSink, SeverityText, TestSink},
        pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
        state::{new_state_channel, MemberId, UnixMillis, Version, WorkerError, WorkerStatus},
    };

    use super::step_once;

    const TEST_DCS_POLL_INTERVAL: Duration = Duration::from_millis(5);

    #[derive(Clone, Default)]
    struct RecordingStore {
        healthy: bool,
        events: Arc<Mutex<VecDeque<WatchEvent>>>,
        writes: Arc<Mutex<Vec<(String, String)>>>,
    }

    impl RecordingStore {
        fn new(healthy: bool) -> Self {
            Self {
                healthy,
                events: Arc::new(Mutex::new(VecDeque::new())),
                writes: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn push_event(&self, event: WatchEvent) {
            if let Ok(mut guard) = self.events.lock() {
                guard.push_back(event);
            }
        }

        fn write_count(&self) -> usize {
            if let Ok(guard) = self.writes.lock() {
                guard.len()
            } else {
                0
            }
        }

        fn first_write_path(&self) -> Option<String> {
            if let Ok(guard) = self.writes.lock() {
                return guard.first().map(|(path, _)| path.clone());
            }
            None
        }

        fn first_write_value(&self) -> Option<String> {
            if let Ok(guard) = self.writes.lock() {
                return guard.first().map(|(_, value)| value.clone());
            }
            None
        }
    }

    impl DcsStore for RecordingStore {
        fn healthy(&self) -> bool {
            self.healthy
        }

        fn read_path(&mut self, _path: &str) -> Result<Option<String>, DcsStoreError> {
            Ok(None)
        }

        fn write_path(&mut self, path: &str, value: String) -> Result<(), DcsStoreError> {
            let mut guard = self
                .writes
                .lock()
                .map_err(|_| DcsStoreError::Io("writes lock poisoned".to_string()))?;
            guard.push((path.to_string(), value));
            Ok(())
        }

        fn put_path_if_absent(&mut self, path: &str, value: String) -> Result<bool, DcsStoreError> {
            let mut guard = self
                .writes
                .lock()
                .map_err(|_| DcsStoreError::Io("writes lock poisoned".to_string()))?;
            guard.push((path.to_string(), value));
            Ok(true)
        }

        fn delete_path(&mut self, _path: &str) -> Result<(), DcsStoreError> {
            Ok(())
        }

        fn drain_watch_events(&mut self) -> Result<Vec<WatchEvent>, DcsStoreError> {
            let mut guard = self
                .events
                .lock()
                .map_err(|_| DcsStoreError::Io("events lock poisoned".to_string()))?;
            Ok(guard.drain(..).collect())
        }
    }

    fn test_log_handle() -> (LogHandle, Arc<TestSink>) {
        let sink = Arc::new(TestSink::default());
        let sink_dyn: Arc<dyn LogSink> = sink.clone();
        (
            LogHandle::new("host-a".to_string(), sink_dyn, SeverityText::Trace),
            sink,
        )
    }

    #[derive(Clone, Default)]
    struct FailingWriteStore {
        events: Arc<Mutex<VecDeque<WatchEvent>>>,
    }

    impl DcsStore for FailingWriteStore {
        fn healthy(&self) -> bool {
            true
        }

        fn read_path(&mut self, _path: &str) -> Result<Option<String>, DcsStoreError> {
            Ok(None)
        }

        fn write_path(&mut self, _path: &str, _value: String) -> Result<(), DcsStoreError> {
            Err(DcsStoreError::Io("boom".to_string()))
        }

        fn put_path_if_absent(
            &mut self,
            _path: &str,
            _value: String,
        ) -> Result<bool, DcsStoreError> {
            Err(DcsStoreError::Io("boom".to_string()))
        }

        fn delete_path(&mut self, _path: &str) -> Result<(), DcsStoreError> {
            Ok(())
        }

        fn drain_watch_events(&mut self) -> Result<Vec<WatchEvent>, DcsStoreError> {
            let mut guard = self
                .events
                .lock()
                .map_err(|_| DcsStoreError::Io("events lock poisoned".to_string()))?;
            Ok(guard.drain(..).collect())
        }
    }

    fn sample_runtime_config() -> RuntimeConfig {
        crate::test_harness::runtime_config::sample_runtime_config()
    }

    fn sample_pg() -> PgInfoState {
        PgInfoState::Primary {
            common: PgInfoCommon {
                worker: WorkerStatus::Running,
                sql: SqlStatus::Healthy,
                readiness: Readiness::Ready,
                timeline: None,
                pg_config: PgConfig {
                    port: None,
                    hot_standby: None,
                    primary_conninfo: None,
                    primary_slot_name: None,
                    extra: BTreeMap::new(),
                },
                last_refresh_at: Some(UnixMillis(1)),
            },
            wal_lsn: crate::state::WalLsn(42),
            slots: Vec::new(),
        }
    }

    fn sample_cache(cfg: RuntimeConfig) -> DcsCache {
        DcsCache {
            members: BTreeMap::new(),
            leader: None,
            switchover: None,
            config: cfg,
            init_lock: None,
        }
    }

    #[test]
    fn apply_watch_update_handles_put_and_delete_paths() {
        let mut cache = sample_cache(sample_runtime_config());
        let member_id = MemberId("node-a".to_string());
        apply_watch_update(
            &mut cache,
            DcsWatchUpdate::Put {
                key: DcsKey::Member(member_id.clone()),
                value: Box::new(DcsValue::Member(MemberRecord {
                    member_id: member_id.clone(),
                    postgres_host: "10.0.0.10".to_string(),
                    postgres_port: 5432,
                    role: MemberRole::Primary,
                    sql: SqlStatus::Healthy,
                    readiness: Readiness::Ready,
                    timeline: None,
                    write_lsn: None,
                    replay_lsn: None,
                    updated_at: UnixMillis(1),
                    pg_version: Version(1),
                })),
            },
        );
        assert!(cache.members.contains_key(&member_id));

        apply_watch_update(
            &mut cache,
            DcsWatchUpdate::Put {
                key: DcsKey::Leader,
                value: Box::new(DcsValue::Leader(LeaderRecord {
                    member_id: member_id.clone(),
                })),
            },
        );
        assert!(cache.leader.is_some());

        apply_watch_update(
            &mut cache,
            DcsWatchUpdate::Put {
                key: DcsKey::Switchover,
                value: Box::new(DcsValue::Switchover(SwitchoverRequest {
                    requested_by: member_id.clone(),
                })),
            },
        );
        assert!(cache.switchover.is_some());

        apply_watch_update(
            &mut cache,
            DcsWatchUpdate::Put {
                key: DcsKey::InitLock,
                value: Box::new(DcsValue::InitLock(InitLockRecord {
                    holder: member_id.clone(),
                })),
            },
        );
        assert!(cache.init_lock.is_some());

        apply_watch_update(
            &mut cache,
            DcsWatchUpdate::Delete {
                key: DcsKey::Member(member_id.clone()),
            },
        );
        apply_watch_update(
            &mut cache,
            DcsWatchUpdate::Delete {
                key: DcsKey::Leader,
            },
        );
        apply_watch_update(
            &mut cache,
            DcsWatchUpdate::Delete {
                key: DcsKey::Switchover,
            },
        );
        apply_watch_update(
            &mut cache,
            DcsWatchUpdate::Delete {
                key: DcsKey::InitLock,
            },
        );

        assert!(!cache.members.contains_key(&member_id));
        assert!(cache.leader.is_none());
        assert!(cache.switchover.is_none());
        assert!(cache.init_lock.is_none());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_publishes_and_writes_only_self_member(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let initial_pg = sample_pg();
        let (pg_publisher, pg_subscriber) = new_state_channel(initial_pg, UnixMillis(1));
        let _ = pg_publisher.publish(sample_pg(), UnixMillis(2));

        let initial_dcs = DcsState {
            worker: WorkerStatus::Starting,
            trust: DcsTrust::NotTrusted,
            cache: sample_cache(sample_runtime_config()),
            last_refresh_at: None,
        };
        let (dcs_publisher, dcs_subscriber) = new_state_channel(initial_dcs, UnixMillis(1));

        let leader_json = serde_json::to_string(&LeaderRecord {
            member_id: MemberId("node-a".to_string()),
        })?;
        let store = RecordingStore::new(true);
        store.push_event(WatchEvent {
            op: WatchOp::Put,
            path: "/scope-a/leader".to_string(),
            value: Some(leader_json),
            revision: 2,
        });
        let store_probe = store.clone();
        let mut ctx = DcsWorkerCtx {
            self_id: MemberId("node-a".to_string()),
            scope: "scope-a".to_string(),
            poll_interval: TEST_DCS_POLL_INTERVAL,
            local_postgres_host: "127.0.0.1".to_string(),
            local_postgres_port: 5432,
            pg_subscriber,
            publisher: dcs_publisher,
            store: Box::new(store),
            log: crate::logging::LogHandle::null(),
            cache: sample_cache(sample_runtime_config()),
            last_published_pg_version: None,
            last_emitted_store_healthy: None,
            last_emitted_trust: None,
        };

        let stepped = step_once(&mut ctx).await;
        assert_eq!(stepped, Ok(()));

        let latest = dcs_subscriber.latest();
        assert_eq!(latest.value.trust, DcsTrust::FullQuorum);
        assert!(latest.value.cache.leader.is_some());
        assert!(latest
            .value
            .cache
            .members
            .contains_key(&MemberId("node-a".to_string())));
        assert_eq!(store_probe.write_count(), 1);
        assert_eq!(
            store_probe.first_write_path(),
            Some("/scope-a/member/node-a".to_string())
        );
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_emits_local_member_write_failed_event_for_io_error(
    ) -> Result<(), WorkerError> {
        let initial_pg = sample_pg();
        let (_pg_publisher, pg_subscriber) = new_state_channel(initial_pg, UnixMillis(1));

        let initial_dcs = DcsState {
            worker: WorkerStatus::Starting,
            trust: DcsTrust::NotTrusted,
            cache: sample_cache(sample_runtime_config()),
            last_refresh_at: None,
        };
        let (dcs_publisher, dcs_subscriber) = new_state_channel(initial_dcs, UnixMillis(1));

        let (log, sink) = test_log_handle();
        let mut ctx = DcsWorkerCtx {
            self_id: MemberId("node-a".to_string()),
            scope: "scope-a".to_string(),
            poll_interval: TEST_DCS_POLL_INTERVAL,
            local_postgres_host: "127.0.0.1".to_string(),
            local_postgres_port: 5432,
            pg_subscriber,
            publisher: dcs_publisher,
            store: Box::new(FailingWriteStore::default()),
            log,
            cache: sample_cache(sample_runtime_config()),
            last_published_pg_version: None,
            last_emitted_store_healthy: None,
            last_emitted_trust: None,
        };

        step_once(&mut ctx).await?;

        let latest = dcs_subscriber.latest();
        assert_eq!(latest.value.trust, DcsTrust::NotTrusted);

        let failures = sink
            .take()
            .into_iter()
            .filter_map(|record| decode_app_event(&record).ok())
            .filter(|event| {
                event.header
                    == crate::logging::AppEventHeader::new(
                        "dcs.local_member.write_failed",
                        "dcs",
                        "failed",
                    )
            })
            .collect::<Vec<_>>();
        if failures.is_empty() {
            return Err(WorkerError::Message(
                "expected dcs.local_member.write_failed event".to_string(),
            ));
        }
        if !failures
            .iter()
            .any(|event| event.severity == SeverityText::Warn)
        {
            return Err(WorkerError::Message(
                "expected dcs.local_member.write_failed severity warn".to_string(),
            ));
        }
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_writes_member_on_every_tick() {
        let initial_pg = sample_pg();
        let (pg_publisher, pg_subscriber) = new_state_channel(initial_pg.clone(), UnixMillis(1));
        let initial_dcs = DcsState {
            worker: WorkerStatus::Starting,
            trust: DcsTrust::NotTrusted,
            cache: sample_cache(sample_runtime_config()),
            last_refresh_at: None,
        };
        let (dcs_publisher, _dcs_subscriber) = new_state_channel(initial_dcs, UnixMillis(1));

        let store = RecordingStore::new(true);
        let store_probe = store.clone();
        let mut ctx = DcsWorkerCtx {
            self_id: MemberId("node-a".to_string()),
            scope: "scope-a".to_string(),
            poll_interval: TEST_DCS_POLL_INTERVAL,
            local_postgres_host: "127.0.0.1".to_string(),
            local_postgres_port: 5432,
            pg_subscriber,
            publisher: dcs_publisher,
            store: Box::new(store),
            log: crate::logging::LogHandle::null(),
            cache: sample_cache(sample_runtime_config()),
            last_published_pg_version: None,
            last_emitted_store_healthy: None,
            last_emitted_trust: None,
        };

        let first = step_once(&mut ctx).await;
        assert_eq!(first, Ok(()));

        let second = step_once(&mut ctx).await;
        assert_eq!(second, Ok(()));
        assert_eq!(store_probe.write_count(), 2);

        let _ = pg_publisher.publish(initial_pg, UnixMillis(2));
        let third = step_once(&mut ctx).await;
        assert_eq!(third, Ok(()));
        assert_eq!(store_probe.write_count(), 3);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_publishes_local_endpoint_instead_of_cached_config_endpoint(
    ) -> Result<(), WorkerError> {
        let initial_pg = sample_pg();
        let (_pg_publisher, pg_subscriber) = new_state_channel(initial_pg, UnixMillis(1));
        let initial_dcs = DcsState {
            worker: WorkerStatus::Starting,
            trust: DcsTrust::NotTrusted,
            cache: sample_cache(sample_runtime_config()),
            last_refresh_at: None,
        };
        let (dcs_publisher, _dcs_subscriber) = new_state_channel(initial_dcs, UnixMillis(1));

        let store = RecordingStore::new(true);
        let store_probe = store.clone();
        let mut ctx = DcsWorkerCtx {
            self_id: MemberId("node-a".to_string()),
            scope: "scope-a".to_string(),
            poll_interval: TEST_DCS_POLL_INTERVAL,
            local_postgres_host: "127.0.0.9".to_string(),
            local_postgres_port: 6543,
            pg_subscriber,
            publisher: dcs_publisher,
            store: Box::new(store),
            log: crate::logging::LogHandle::null(),
            cache: sample_cache(sample_runtime_config()),
            last_published_pg_version: None,
            last_emitted_store_healthy: None,
            last_emitted_trust: None,
        };

        assert_eq!(step_once(&mut ctx).await, Ok(()));
        let encoded = store_probe
            .first_write_value()
            .ok_or_else(|| WorkerError::Message("expected local member write".to_string()))?;
        let record: MemberRecord = serde_json::from_str(encoded.as_str()).map_err(|err| {
            WorkerError::Message(format!("decode written member record failed: {err}"))
        })?;
        assert_eq!(record.postgres_host, "127.0.0.9".to_string());
        assert_eq!(record.postgres_port, 6543);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_republishes_member_after_unhealthy_tick_even_without_pg_change() {
        let initial_pg = sample_pg();
        let (_pg_publisher, pg_subscriber) = new_state_channel(initial_pg.clone(), UnixMillis(1));
        let initial_dcs = DcsState {
            worker: WorkerStatus::Starting,
            trust: DcsTrust::NotTrusted,
            cache: sample_cache(sample_runtime_config()),
            last_refresh_at: None,
        };
        let (dcs_publisher, _dcs_subscriber) = new_state_channel(initial_dcs, UnixMillis(1));

        let store = RecordingStore::new(true);
        let store_probe = store.clone();
        let mut ctx = DcsWorkerCtx {
            self_id: MemberId("node-a".to_string()),
            scope: "scope-a".to_string(),
            poll_interval: TEST_DCS_POLL_INTERVAL,
            local_postgres_host: "127.0.0.1".to_string(),
            local_postgres_port: 5432,
            pg_subscriber,
            publisher: dcs_publisher,
            store: Box::new(store),
            log: crate::logging::LogHandle::null(),
            cache: sample_cache(sample_runtime_config()),
            last_published_pg_version: Some(Version(1)),
            last_emitted_store_healthy: Some(false),
            last_emitted_trust: None,
        };

        let stepped = step_once(&mut ctx).await;
        assert_eq!(stepped, Ok(()));
        assert_eq!(store_probe.write_count(), 1);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_marks_store_unhealthy_when_watch_decode_fails() {
        let initial_pg = sample_pg();
        let (_pg_publisher, pg_subscriber) = new_state_channel(initial_pg, UnixMillis(1));
        let initial_dcs = DcsState {
            worker: WorkerStatus::Starting,
            trust: DcsTrust::NotTrusted,
            cache: sample_cache(sample_runtime_config()),
            last_refresh_at: None,
        };
        let (dcs_publisher, dcs_subscriber) = new_state_channel(initial_dcs, UnixMillis(1));

        let store = RecordingStore::new(true);
        store.push_event(WatchEvent {
            op: WatchOp::Put,
            path: "/scope-a/leader".to_string(),
            value: Some("{invalid-json".to_string()),
            revision: 2,
        });

        let mut ctx = DcsWorkerCtx {
            self_id: MemberId("node-a".to_string()),
            scope: "scope-a".to_string(),
            poll_interval: TEST_DCS_POLL_INTERVAL,
            local_postgres_host: "127.0.0.1".to_string(),
            local_postgres_port: 5432,
            pg_subscriber,
            publisher: dcs_publisher,
            store: Box::new(store),
            log: crate::logging::LogHandle::null(),
            cache: sample_cache(sample_runtime_config()),
            last_published_pg_version: None,
            last_emitted_store_healthy: None,
            last_emitted_trust: None,
        };

        assert_eq!(step_once(&mut ctx).await, Ok(()));
        let latest = dcs_subscriber.latest();
        assert_eq!(latest.value.trust, DcsTrust::NotTrusted);
        assert!(matches!(
            latest.value.worker,
            WorkerStatus::Faulted(WorkerError::Message(_))
        ));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_marks_store_unhealthy_when_watch_key_is_unknown() {
        let initial_pg = sample_pg();
        let (_pg_publisher, pg_subscriber) = new_state_channel(initial_pg, UnixMillis(1));
        let initial_dcs = DcsState {
            worker: WorkerStatus::Starting,
            trust: DcsTrust::NotTrusted,
            cache: sample_cache(sample_runtime_config()),
            last_refresh_at: None,
        };
        let (dcs_publisher, dcs_subscriber) = new_state_channel(initial_dcs, UnixMillis(1));

        let store = RecordingStore::new(true);
        store.push_event(WatchEvent {
            op: WatchOp::Put,
            path: "/scope-a/not-a-real-key".to_string(),
            value: Some("{\"ignored\":true}".to_string()),
            revision: 2,
        });

        let mut ctx = DcsWorkerCtx {
            self_id: MemberId("node-a".to_string()),
            scope: "scope-a".to_string(),
            poll_interval: TEST_DCS_POLL_INTERVAL,
            local_postgres_host: "127.0.0.1".to_string(),
            local_postgres_port: 5432,
            pg_subscriber,
            publisher: dcs_publisher,
            store: Box::new(store),
            log: crate::logging::LogHandle::null(),
            cache: sample_cache(sample_runtime_config()),
            last_published_pg_version: None,
            last_emitted_store_healthy: None,
            last_emitted_trust: None,
        };

        assert_eq!(step_once(&mut ctx).await, Ok(()));
        let latest = dcs_subscriber.latest();
        assert_eq!(latest.value.trust, DcsTrust::NotTrusted);
        assert!(matches!(
            latest.value.worker,
            WorkerStatus::Faulted(WorkerError::Message(_))
        ));
    }
}

--- END FILE: src/dcs/worker.rs ---

--- BEGIN FILE: src/dcs/etcd_store.rs ---
use std::{
    collections::VecDeque,
    future::Future,
    str,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use etcd_client::{
    Client, Compare, CompareOp, EventType, GetOptions, Txn, TxnOp, WatchOptions, WatchResponse,
    WatchStream, Watcher,
};
use tokio::sync::mpsc as tokio_mpsc;

use super::store::{DcsStore, DcsStoreError, WatchEvent, WatchOp};

const COMMAND_TIMEOUT: Duration = Duration::from_secs(2);
const WORKER_BOOTSTRAP_TIMEOUT: Duration = Duration::from_secs(8);
const WATCH_IDLE_INTERVAL: Duration = Duration::from_millis(100);

enum WorkerCommand {
    Read {
        path: String,
        response_tx: mpsc::Sender<Result<Option<String>, DcsStoreError>>,
    },
    Write {
        path: String,
        value: String,
        response_tx: mpsc::Sender<Result<(), DcsStoreError>>,
    },
    PutIfAbsent {
        path: String,
        value: String,
        response_tx: mpsc::Sender<Result<bool, DcsStoreError>>,
    },
    Delete {
        path: String,
        response_tx: mpsc::Sender<Result<(), DcsStoreError>>,
    },
    Shutdown,
}

pub(crate) struct EtcdDcsStore {
    healthy: Arc<AtomicBool>,
    events: Arc<Mutex<VecDeque<WatchEvent>>>,
    command_tx: tokio_mpsc::UnboundedSender<WorkerCommand>,
    worker_handle: Option<JoinHandle<()>>,
}

impl EtcdDcsStore {
    pub(crate) fn connect(endpoints: Vec<String>, scope: &str) -> Result<Self, DcsStoreError> {
        Self::connect_with_worker_bootstrap_timeout(endpoints, scope, WORKER_BOOTSTRAP_TIMEOUT)
    }

    fn connect_with_worker_bootstrap_timeout(
        endpoints: Vec<String>,
        scope: &str,
        worker_bootstrap_timeout: Duration,
    ) -> Result<Self, DcsStoreError> {
        if endpoints.is_empty() {
            return Err(DcsStoreError::Io(
                "at least one etcd endpoint is required".to_string(),
            ));
        }

        let scope_prefix = format!("/{}/", scope.trim_matches('/'));
        let healthy = Arc::new(AtomicBool::new(false));
        let events = Arc::new(Mutex::new(VecDeque::new()));
        let (command_tx, command_rx) = tokio_mpsc::unbounded_channel::<WorkerCommand>();
        let (startup_tx, startup_rx) = mpsc::channel::<Result<(), DcsStoreError>>();

        let worker_healthy = Arc::clone(&healthy);
        let worker_events = Arc::clone(&events);
        let worker_endpoints = endpoints;
        let worker_scope = scope_prefix;

        let worker_handle = thread::Builder::new()
            .name("etcd-dcs-store".to_string())
            .spawn(move || {
                run_worker_loop(
                    worker_endpoints,
                    worker_scope,
                    worker_healthy,
                    worker_events,
                    command_rx,
                    startup_tx,
                );
            })
            .map_err(|err| DcsStoreError::Io(format!("spawn etcd worker failed: {err}")))?;

        match startup_rx.recv_timeout(worker_bootstrap_timeout) {
            Ok(Ok(())) => Ok(Self {
                healthy,
                events,
                command_tx,
                worker_handle: Some(worker_handle),
            }),
            Ok(Err(err)) => {
                let _ = worker_handle.join();
                Err(err)
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // The worker might still be performing its bootstrap (connect + get + watch).
                // Request shutdown and close the command channel, but do not join here: joining
                // would turn this bounded startup timeout into an unbounded connect() call.
                let _ = command_tx.send(WorkerCommand::Shutdown);
                drop(command_tx);
                drop(worker_handle);
                Err(DcsStoreError::Io(format!(
                    "timed out waiting for etcd worker startup after {worker_bootstrap_timeout:?}"
                )))
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                let worker_panicked = worker_handle.join().is_err();
                let suffix = if worker_panicked {
                    " (worker panicked)"
                } else {
                    ""
                };
                Err(DcsStoreError::Io(format!(
                    "etcd worker exited before signaling startup{suffix}"
                )))
            }
        }
    }

    pub(crate) fn put_path_if_absent(
        &mut self,
        path: &str,
        value: String,
    ) -> Result<bool, DcsStoreError> {
        let (response_tx, response_rx) = mpsc::channel::<Result<bool, DcsStoreError>>();
        self.command_tx
            .send(WorkerCommand::PutIfAbsent {
                path: path.to_string(),
                value,
                response_tx,
            })
            .map_err(|err| {
                self.mark_unhealthy();
                DcsStoreError::Io(format!("send put-if-absent command failed: {err}"))
            })?;

        response_rx.recv_timeout(COMMAND_TIMEOUT).map_err(|err| {
            self.mark_unhealthy();
            DcsStoreError::Io(format!(
                "timed out waiting for put-if-absent response: {err}"
            ))
        })?
    }

    fn mark_unhealthy(&self) {
        self.healthy.store(false, Ordering::SeqCst);
    }
}

fn run_worker_loop(
    endpoints: Vec<String>,
    scope_prefix: String,
    healthy: Arc<AtomicBool>,
    events: Arc<Mutex<VecDeque<WatchEvent>>>,
    mut command_rx: tokio_mpsc::UnboundedReceiver<WorkerCommand>,
    startup_tx: mpsc::Sender<Result<(), DcsStoreError>>,
) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build();

    let Ok(runtime) = runtime else {
        let _ = startup_tx.send(Err(DcsStoreError::Io(
            "failed to build tokio runtime for etcd store worker".to_string(),
        )));
        return;
    };

    runtime.block_on(async move {
        let mut had_successful_session = false;

        let (mut client, mut _watcher, mut watch_stream): (
            Option<Client>,
            Option<Watcher>,
            Option<WatchStream>,
        ) = match establish_watch_session(
            &endpoints,
            &scope_prefix,
            &events,
            had_successful_session,
        )
        .await
        {
            Ok((next_client, next_watcher, next_stream)) => {
                had_successful_session = true;
                healthy.store(true, Ordering::SeqCst);
                let _ = startup_tx.send(Ok(()));
                (Some(next_client), Some(next_watcher), Some(next_stream))
            }
            Err(err) => {
                healthy.store(false, Ordering::SeqCst);
                let _ = startup_tx.send(Err(err));
                return;
            }
        };

        loop {
            if client.is_none() || watch_stream.is_none() {
                tokio::select! {
                    maybe_command = command_rx.recv() => {
                        let Some(command) = maybe_command else {
                            return;
                        };
                        if !handle_worker_command(
                            command,
                            &endpoints,
                            &healthy,
                            &events,
                            &mut client,
                            &mut _watcher,
                            &mut watch_stream,
                        ).await {
                            return;
                        }
                    }
                    _ = tokio::time::sleep(WATCH_IDLE_INTERVAL) => {
                        match establish_watch_session(
                            &endpoints,
                            &scope_prefix,
                            &events,
                            had_successful_session,
                        )
                        .await
                        {
                            Ok((next_client, next_watcher, next_stream)) => {
                                had_successful_session = true;
                                client = Some(next_client);
                                _watcher = Some(next_watcher);
                                watch_stream = Some(next_stream);
                                healthy.store(true, Ordering::SeqCst);
                            }
                            Err(_) => {
                                healthy.store(false, Ordering::SeqCst);
                            }
                        }
                    }
                }
                continue;
            }

            let Some(active_stream) = watch_stream.as_mut() else {
                tokio::time::sleep(WATCH_IDLE_INTERVAL).await;
                continue;
            };

            tokio::select! {
                maybe_command = command_rx.recv() => {
                    let Some(command) = maybe_command else {
                        return;
                    };
                    if !handle_worker_command(
                        command,
                        &endpoints,
                        &healthy,
                        &events,
                        &mut client,
                        &mut _watcher,
                        &mut watch_stream,
                    ).await {
                        return;
                    }
                }
                response = active_stream.message() => {
                    match response {
                        Ok(Some(response)) => {
                            if apply_watch_response(response, &events).is_err() {
                                if invalidate_watch_session(
                                    &healthy,
                                    &events,
                                    &mut client,
                                    &mut _watcher,
                                    &mut watch_stream,
                                )
                                .is_err()
                                {
                                    return;
                                }
                            } else {
                                healthy.store(true, Ordering::SeqCst);
                            }
                        }
                        Ok(None) | Err(_) => {
                            if invalidate_watch_session(
                                &healthy,
                                &events,
                                &mut client,
                                &mut _watcher,
                                &mut watch_stream,
                            )
                            .is_err()
                            {
                                return;
                            }
                        }
                    }
                }
                _ = tokio::time::sleep(WATCH_IDLE_INTERVAL) => {}
            }
        }
    });
}

async fn handle_worker_command(
    command: WorkerCommand,
    endpoints: &[String],
    healthy: &Arc<AtomicBool>,
    events: &Arc<Mutex<VecDeque<WatchEvent>>>,
    client: &mut Option<Client>,
    watcher: &mut Option<Watcher>,
    watch_stream: &mut Option<WatchStream>,
) -> bool {
    match command {
        WorkerCommand::Write {
            path,
            value,
            response_tx,
        } => {
            let result = execute_write(endpoints, client, healthy, &path, value).await;
            let invalidate_result = if result.is_err() {
                invalidate_watch_session(healthy, events, client, watcher, watch_stream)
            } else {
                Ok(())
            };
            let _ = response_tx.send(result);
            invalidate_result.is_ok()
        }
        WorkerCommand::Read { path, response_tx } => {
            let result = execute_read(endpoints, client, healthy, &path).await;
            let invalidate_result = if result.is_err() {
                invalidate_watch_session(healthy, events, client, watcher, watch_stream)
            } else {
                Ok(())
            };
            let _ = response_tx.send(result);
            invalidate_result.is_ok()
        }
        WorkerCommand::PutIfAbsent {
            path,
            value,
            response_tx,
        } => {
            let result = execute_put_if_absent(endpoints, client, healthy, &path, value).await;
            let invalidate_result = if result.is_err() {
                invalidate_watch_session(healthy, events, client, watcher, watch_stream)
            } else {
                Ok(())
            };
            let _ = response_tx.send(result);
            invalidate_result.is_ok()
        }
        WorkerCommand::Delete { path, response_tx } => {
            let result = execute_delete(endpoints, client, healthy, &path).await;
            let invalidate_result = if result.is_err() {
                invalidate_watch_session(healthy, events, client, watcher, watch_stream)
            } else {
                Ok(())
            };
            let _ = response_tx.send(result);
            invalidate_result.is_ok()
        }
        WorkerCommand::Shutdown => false,
    }
}

fn invalidate_watch_session(
    healthy: &Arc<AtomicBool>,
    events: &Arc<Mutex<VecDeque<WatchEvent>>>,
    client: &mut Option<Client>,
    watcher: &mut Option<Watcher>,
    watch_stream: &mut Option<WatchStream>,
) -> Result<(), DcsStoreError> {
    healthy.store(false, Ordering::SeqCst);
    *client = None;
    *watcher = None;
    *watch_stream = None;
    clear_watch_events(events)
}

async fn establish_watch_session(
    endpoints: &[String],
    scope_prefix: &str,
    events: &Arc<Mutex<VecDeque<WatchEvent>>>,
    is_reconnect: bool,
) -> Result<(Client, Watcher, WatchStream), DcsStoreError> {
    #[cfg(test)]
    apply_test_establish_delay().await;

    let mut client = connect_client(endpoints).await?;
    let snapshot_revision =
        bootstrap_snapshot(&mut client, scope_prefix, events, is_reconnect).await?;
    let start_revision = snapshot_revision.saturating_add(1);
    let (watcher, watch_stream) =
        create_watch_stream(&mut client, scope_prefix, start_revision).await?;
    Ok((client, watcher, watch_stream))
}

async fn connect_client(endpoints: &[String]) -> Result<Client, DcsStoreError> {
    timeout_etcd("etcd connect", Client::connect(endpoints.to_vec(), None)).await
}

async fn execute_write(
    endpoints: &[String],
    client: &mut Option<Client>,
    healthy: &Arc<AtomicBool>,
    path: &str,
    value: String,
) -> Result<(), DcsStoreError> {
    if client.is_none() {
        *client = Some(connect_client(endpoints).await?);
    }

    let Some(active_client) = client.as_mut() else {
        healthy.store(false, Ordering::SeqCst);
        return Err(DcsStoreError::Io(
            "etcd client unavailable for write".to_string(),
        ));
    };

    match timeout_etcd("etcd put", active_client.put(path, value, None)).await {
        Ok(_) => {
            healthy.store(true, Ordering::SeqCst);
            Ok(())
        }
        Err(err) => {
            healthy.store(false, Ordering::SeqCst);
            *client = None;
            Err(err)
        }
    }
}

async fn execute_delete(
    endpoints: &[String],
    client: &mut Option<Client>,
    healthy: &Arc<AtomicBool>,
    path: &str,
) -> Result<(), DcsStoreError> {
    if client.is_none() {
        *client = Some(connect_client(endpoints).await?);
    }

    let Some(active_client) = client.as_mut() else {
        healthy.store(false, Ordering::SeqCst);
        return Err(DcsStoreError::Io(
            "etcd client unavailable for delete".to_string(),
        ));
    };

    match timeout_etcd("etcd delete", active_client.delete(path, None)).await {
        Ok(_) => {
            healthy.store(true, Ordering::SeqCst);
            Ok(())
        }
        Err(err) => {
            healthy.store(false, Ordering::SeqCst);
            *client = None;
            Err(err)
        }
    }
}

async fn execute_put_if_absent(
    endpoints: &[String],
    client: &mut Option<Client>,
    healthy: &Arc<AtomicBool>,
    path: &str,
    value: String,
) -> Result<bool, DcsStoreError> {
    if client.is_none() {
        *client = Some(connect_client(endpoints).await?);
    }

    let Some(active_client) = client.as_mut() else {
        healthy.store(false, Ordering::SeqCst);
        return Err(DcsStoreError::Io(
            "etcd client unavailable for put-if-absent".to_string(),
        ));
    };

    let compare = Compare::version(path, CompareOp::Equal, 0);
    let then_put = TxnOp::put(path, value, None);
    let txn = Txn::new().when(vec![compare]).and_then(vec![then_put]);

    match timeout_etcd("etcd txn", active_client.txn(txn)).await {
        Ok(response) => {
            healthy.store(true, Ordering::SeqCst);
            Ok(response.succeeded())
        }
        Err(err) => {
            healthy.store(false, Ordering::SeqCst);
            *client = None;
            Err(err)
        }
    }
}

async fn bootstrap_snapshot(
    client: &mut Client,
    scope_prefix: &str,
    events: &Arc<Mutex<VecDeque<WatchEvent>>>,
    is_reconnect: bool,
) -> Result<i64, DcsStoreError> {
    let response = timeout_etcd(
        "etcd get",
        client.get(scope_prefix, Some(GetOptions::new().with_prefix())),
    )
    .await?;

    let revision = response
        .header()
        .map(|header| header.revision())
        .unwrap_or(0);

    let mut queue = VecDeque::new();
    if is_reconnect {
        queue.push_back(WatchEvent {
            op: WatchOp::Reset,
            path: scope_prefix.to_string(),
            value: None,
            revision,
        });
    }
    for kv in response.kvs() {
        let path = str::from_utf8(kv.key()).map_err(|err| DcsStoreError::Decode {
            key: "watch-key".to_string(),
            message: err.to_string(),
        })?;
        let value = str::from_utf8(kv.value()).map_err(|err| DcsStoreError::Decode {
            key: path.to_string(),
            message: err.to_string(),
        })?;

        queue.push_back(WatchEvent {
            op: WatchOp::Put,
            path: path.to_string(),
            value: Some(value.to_string()),
            revision: kv.mod_revision(),
        });
    }

    if is_reconnect {
        replace_watch_events(events, queue)?;
    } else {
        enqueue_watch_events(events, queue)?;
    }
    Ok(revision)
}

async fn create_watch_stream(
    client: &mut Client,
    scope_prefix: &str,
    start_revision: i64,
) -> Result<(Watcher, WatchStream), DcsStoreError> {
    let watch_options = WatchOptions::new()
        .with_prefix()
        .with_start_revision(start_revision);
    timeout_etcd(
        "etcd watch",
        client.watch(scope_prefix, Some(watch_options)),
    )
    .await
}

fn apply_watch_response(
    response: WatchResponse,
    events: &Arc<Mutex<VecDeque<WatchEvent>>>,
) -> Result<(), DcsStoreError> {
    if response.canceled() || response.compact_revision() > 0 {
        return Err(DcsStoreError::Io(format!(
            "etcd watch canceled: reason='{}' compact_revision={}",
            response.cancel_reason(),
            response.compact_revision()
        )));
    }

    let mut queue = VecDeque::new();
    for event in response.events() {
        let Some(kv) = event.kv() else {
            return Err(DcsStoreError::Io(
                "etcd watch event missing key-value payload".to_string(),
            ));
        };

        let path = str::from_utf8(kv.key()).map_err(|err| DcsStoreError::Decode {
            key: "watch-key".to_string(),
            message: err.to_string(),
        })?;

        match event.event_type() {
            EventType::Put => {
                let value = str::from_utf8(kv.value()).map_err(|err| DcsStoreError::Decode {
                    key: path.to_string(),
                    message: err.to_string(),
                })?;
                queue.push_back(WatchEvent {
                    op: WatchOp::Put,
                    path: path.to_string(),
                    value: Some(value.to_string()),
                    revision: kv.mod_revision(),
                });
            }
            EventType::Delete => {
                queue.push_back(WatchEvent {
                    op: WatchOp::Delete,
                    path: path.to_string(),
                    value: None,
                    revision: kv.mod_revision(),
                });
            }
        }
    }

    enqueue_watch_events(events, queue)
}

fn enqueue_watch_events(
    events: &Arc<Mutex<VecDeque<WatchEvent>>>,
    queue: VecDeque<WatchEvent>,
) -> Result<(), DcsStoreError> {
    let mut guard = events
        .lock()
        .map_err(|_| DcsStoreError::Io("events lock poisoned".to_string()))?;
    guard.extend(queue);
    Ok(())
}

fn replace_watch_events(
    events: &Arc<Mutex<VecDeque<WatchEvent>>>,
    queue: VecDeque<WatchEvent>,
) -> Result<(), DcsStoreError> {
    let mut guard = events
        .lock()
        .map_err(|_| DcsStoreError::Io("events lock poisoned".to_string()))?;
    guard.clear();
    guard.extend(queue);
    Ok(())
}

fn clear_watch_events(events: &Arc<Mutex<VecDeque<WatchEvent>>>) -> Result<(), DcsStoreError> {
    let mut guard = events
        .lock()
        .map_err(|_| DcsStoreError::Io("events lock poisoned".to_string()))?;
    guard.clear();
    Ok(())
}

async fn timeout_etcd<T, F>(operation: &str, fut: F) -> Result<T, DcsStoreError>
where
    F: Future<Output = Result<T, etcd_client::Error>>,
{
    match tokio::time::timeout(COMMAND_TIMEOUT, fut).await {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(err)) => Err(DcsStoreError::Io(format!("{operation} failed: {err}"))),
        Err(err) => Err(DcsStoreError::Io(format!("{operation} timed out: {err}"))),
    }
}

#[cfg(test)]
use std::sync::atomic::AtomicU64;

#[cfg(test)]
static TEST_ESTABLISH_DELAY_MS: AtomicU64 = AtomicU64::new(0);

#[cfg(test)]
async fn apply_test_establish_delay() {
    let delay_ms = TEST_ESTABLISH_DELAY_MS.load(Ordering::SeqCst);
    if delay_ms == 0 {
        return;
    }
    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
}

async fn execute_read(
    endpoints: &[String],
    client: &mut Option<Client>,
    healthy: &Arc<AtomicBool>,
    path: &str,
) -> Result<Option<String>, DcsStoreError> {
    if client.is_none() {
        *client = Some(connect_client(endpoints).await?);
    }

    let Some(active_client) = client.as_mut() else {
        healthy.store(false, Ordering::SeqCst);
        return Err(DcsStoreError::Io(
            "etcd client unavailable for read".to_string(),
        ));
    };

    match timeout_etcd("etcd get", active_client.get(path, None)).await {
        Ok(response) => {
            healthy.store(true, Ordering::SeqCst);
            let Some(kv) = response.kvs().first() else {
                return Ok(None);
            };
            let raw = kv.value();
            let decoded = String::from_utf8(raw.to_vec()).map_err(|err| {
                DcsStoreError::Io(format!("etcd read value not utf8 for `{path}`: {err}"))
            })?;
            Ok(Some(decoded))
        }
        Err(err) => {
            healthy.store(false, Ordering::SeqCst);
            *client = None;
            Err(err)
        }
    }
}

impl DcsStore for EtcdDcsStore {
    fn healthy(&self) -> bool {
        self.healthy.load(Ordering::SeqCst)
    }

    fn read_path(&mut self, path: &str) -> Result<Option<String>, DcsStoreError> {
        let (response_tx, response_rx) = mpsc::channel();
        self.command_tx
            .send(WorkerCommand::Read {
                path: path.to_string(),
                response_tx,
            })
            .map_err(|err| {
                self.mark_unhealthy();
                DcsStoreError::Io(format!("send read command failed: {err}"))
            })?;

        response_rx.recv_timeout(COMMAND_TIMEOUT).map_err(|err| {
            self.mark_unhealthy();
            DcsStoreError::Io(format!("timed out waiting for read command: {err}"))
        })?
    }

    fn write_path(&mut self, path: &str, value: String) -> Result<(), DcsStoreError> {
        let (response_tx, response_rx) = mpsc::channel();
        self.command_tx
            .send(WorkerCommand::Write {
                path: path.to_string(),
                value,
                response_tx,
            })
            .map_err(|err| {
                self.mark_unhealthy();
                DcsStoreError::Io(format!("send write command failed: {err}"))
            })?;

        response_rx.recv_timeout(COMMAND_TIMEOUT).map_err(|err| {
            self.mark_unhealthy();
            DcsStoreError::Io(format!("timed out waiting for write command: {err}"))
        })?
    }

    fn put_path_if_absent(&mut self, path: &str, value: String) -> Result<bool, DcsStoreError> {
        EtcdDcsStore::put_path_if_absent(self, path, value)
    }

    fn delete_path(&mut self, path: &str) -> Result<(), DcsStoreError> {
        let (response_tx, response_rx) = mpsc::channel();
        self.command_tx
            .send(WorkerCommand::Delete {
                path: path.to_string(),
                response_tx,
            })
            .map_err(|err| {
                self.mark_unhealthy();
                DcsStoreError::Io(format!("send delete command failed: {err}"))
            })?;

        response_rx.recv_timeout(COMMAND_TIMEOUT).map_err(|err| {
            self.mark_unhealthy();
            DcsStoreError::Io(format!("timed out waiting for delete command: {err}"))
        })?
    }

    fn drain_watch_events(&mut self) -> Result<Vec<WatchEvent>, DcsStoreError> {
        let mut guard = self
            .events
            .lock()
            .map_err(|_| DcsStoreError::Io("events lock poisoned".to_string()))?;
        Ok(guard.drain(..).collect())
    }
}

impl Drop for EtcdDcsStore {
    fn drop(&mut self) {
        let _ = self.command_tx.send(WorkerCommand::Shutdown);
        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        fs,
        path::PathBuf,
        time::{Duration, Instant},
    };

    use etcd_client::Client;

    use crate::{
        config::RuntimeConfig,
        dcs::{
            etcd_store::EtcdDcsStore,
            state::{
                DcsCache, DcsState, DcsTrust, DcsWorkerCtx, InitLockRecord, LeaderRecord,
                MemberRecord, MemberRole, SwitchoverRequest,
            },
            store::{refresh_from_etcd_watch, DcsStore, DcsStoreError, WatchEvent, WatchOp},
            worker::step_once,
        },
        pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
        state::{new_state_channel, MemberId, UnixMillis, WorkerError, WorkerStatus},
        test_harness::{
            binaries::require_etcd_bin_for_real_tests,
            etcd3::{prepare_etcd_data_dir, spawn_etcd3, EtcdHandle, EtcdInstanceSpec},
            namespace::NamespaceGuard,
            ports::allocate_ports,
            HarnessError,
        },
    };

    type BoxError = Box<dyn std::error::Error + Send + Sync>;
    type TestResult = Result<(), BoxError>;

    fn boxed_error(message: impl Into<String>) -> BoxError {
        Box::new(std::io::Error::other(message.into()))
    }

    struct RealEtcdFixture {
        _guard: NamespaceGuard,
        handle: EtcdHandle,
        etcd_bin: PathBuf,
        namespace_id: String,
        log_dir: PathBuf,
        peer_port: u16,
        endpoint: String,
        scope: String,
    }

    impl RealEtcdFixture {
        async fn spawn(test_name: &str, scope: &str) -> Result<Self, HarnessError> {
            let etcd_bin = require_etcd_bin_for_real_tests()?;

            let guard = NamespaceGuard::new(test_name)?;
            let namespace = guard.namespace()?;
            let namespace_id = namespace.id.clone();
            let log_dir = namespace.child_dir("logs/etcd-store");
            let data_dir = prepare_etcd_data_dir(namespace)?;

            let reservation = allocate_ports(2)?;
            let ports = reservation.as_slice();
            let client_port = ports[0];
            let peer_port = ports[1];
            drop(reservation);

            let handle = spawn_etcd3(EtcdInstanceSpec {
                etcd_bin: etcd_bin.clone(),
                namespace_id: namespace_id.clone(),
                member_name: "node-a".to_string(),
                data_dir,
                log_dir: log_dir.clone(),
                client_port,
                peer_port,
                startup_timeout: Duration::from_secs(10),
            })
            .await?;

            Ok(Self {
                _guard: guard,
                handle,
                etcd_bin,
                namespace_id,
                log_dir,
                peer_port,
                endpoint: format!("http://127.0.0.1:{client_port}"),
                scope: scope.to_string(),
            })
        }

        async fn shutdown(&mut self) -> Result<(), HarnessError> {
            self.handle.shutdown().await
        }

        async fn restart_clean(&mut self) -> Result<(), HarnessError> {
            self.handle.shutdown().await?;

            if self.handle.data_dir.exists() {
                fs::remove_dir_all(&self.handle.data_dir)?;
            }
            fs::create_dir_all(&self.handle.data_dir)?;

            let client_port = self.handle.client_port;
            let data_dir = self.handle.data_dir.clone();
            let handle = spawn_etcd3(EtcdInstanceSpec {
                etcd_bin: self.etcd_bin.clone(),
                namespace_id: self.namespace_id.clone(),
                member_name: self.handle.member_name().to_string(),
                data_dir,
                log_dir: self.log_dir.clone(),
                client_port,
                peer_port: self.peer_port,
                startup_timeout: Duration::from_secs(10),
            })
            .await?;
            self.handle = handle;
            Ok(())
        }
    }

    struct EstablishDelayGuard {
        previous_ms: u64,
    }

    impl EstablishDelayGuard {
        fn new(delay_ms: u64) -> Self {
            let previous_ms =
                super::TEST_ESTABLISH_DELAY_MS.swap(delay_ms, std::sync::atomic::Ordering::SeqCst);
            Self { previous_ms }
        }
    }

    impl Drop for EstablishDelayGuard {
        fn drop(&mut self) {
            super::TEST_ESTABLISH_DELAY_MS
                .store(self.previous_ms, std::sync::atomic::Ordering::SeqCst);
        }
    }

    fn wait_for_event(
        store: &mut dyn DcsStore,
        op: WatchOp,
        path: &str,
        timeout: Duration,
    ) -> Result<(), DcsStoreError> {
        let deadline = Instant::now() + timeout;
        loop {
            for event in store.drain_watch_events()? {
                if event.op == op && event.path == path {
                    return Ok(());
                }
            }
            if Instant::now() >= deadline {
                return Err(DcsStoreError::Io(format!(
                    "timed out waiting for event {:?} at {}",
                    op, path
                )));
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    fn sample_runtime_config(scope: &str) -> RuntimeConfig {
        crate::test_harness::runtime_config::RuntimeConfigBuilder::new()
            .with_dcs_scope(scope)
            .build()
    }

    fn sample_cache(scope: &str) -> DcsCache {
        DcsCache {
            members: BTreeMap::new(),
            leader: None,
            switchover: None,
            config: sample_runtime_config(scope),
            init_lock: None,
        }
    }

    fn sample_pg() -> PgInfoState {
        PgInfoState::Primary {
            common: PgInfoCommon {
                worker: WorkerStatus::Running,
                sql: SqlStatus::Healthy,
                readiness: Readiness::Ready,
                timeline: None,
                pg_config: PgConfig {
                    port: None,
                    hot_standby: None,
                    primary_conninfo: None,
                    primary_slot_name: None,
                    extra: BTreeMap::new(),
                },
                last_refresh_at: Some(UnixMillis(1)),
            },
            wal_lsn: crate::state::WalLsn(42),
            slots: Vec::new(),
        }
    }

    fn build_worker_ctx(
        scope: &str,
        store: EtcdDcsStore,
    ) -> (DcsWorkerCtx, crate::state::StateSubscriber<DcsState>) {
        let self_id = MemberId("node-a".to_string());
        let initial_pg = sample_pg();
        let (_pg_publisher, pg_subscriber) = new_state_channel(initial_pg, UnixMillis(1));

        let initial_dcs = DcsState {
            worker: WorkerStatus::Starting,
            trust: DcsTrust::NotTrusted,
            cache: sample_cache(scope),
            last_refresh_at: Some(UnixMillis(1)),
        };
        let (dcs_publisher, dcs_subscriber) = new_state_channel(initial_dcs, UnixMillis(1));

        (
            DcsWorkerCtx {
                self_id,
                scope: scope.to_string(),
                poll_interval: Duration::from_millis(50),
                local_postgres_host: "127.0.0.1".to_string(),
                local_postgres_port: 5432,
                pg_subscriber,
                publisher: dcs_publisher,
                store: Box::new(store),
                log: crate::logging::LogHandle::null(),
                cache: sample_cache(scope),
                last_published_pg_version: None,
                last_emitted_store_healthy: None,
                last_emitted_trust: None,
            },
            dcs_subscriber,
        )
    }

    async fn shutdown_with_result(mut fixture: RealEtcdFixture, result: TestResult) -> TestResult {
        let shutdown_result = fixture.shutdown().await;
        match result {
            Err(err) => Err(err),
            Ok(()) => match shutdown_result {
                Ok(()) => Ok(()),
                Err(err) => Err(Box::new(err)),
            },
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn etcd_store_connect_timeout_returns_and_does_not_hang() -> TestResult {
        let fixture =
            RealEtcdFixture::spawn("dcs-etcd-store-connect-timeout", "scope-timeout").await?;

        let fixture = fixture;
        let result: TestResult = async {
            let _delay_guard = EstablishDelayGuard::new(2_500);
            let endpoint = fixture.endpoint.clone();
            let scope = fixture.scope.clone();

            let handle = tokio::task::spawn_blocking(move || {
                let started_at = Instant::now();
                let store_result = EtcdDcsStore::connect_with_worker_bootstrap_timeout(
                    vec![endpoint],
                    scope.as_str(),
                    Duration::from_millis(50),
                );
                (started_at.elapsed(), store_result)
            });

            let outcome = tokio::time::timeout(Duration::from_secs(2), handle).await;
            let (elapsed, store_result) = match outcome {
                Ok(joined) => match joined {
                    Ok(out) => out,
                    Err(err) => {
                        return Err(boxed_error(format!(
                            "connect spawn_blocking join failed: {err}"
                        )));
                    }
                },
                Err(_) => {
                    return Err(boxed_error(
                        "timed out waiting for connect() to return after startup timeout",
                    ));
                }
            };

            if elapsed >= Duration::from_secs(1) {
                return Err(boxed_error(format!(
                    "expected connect() to return promptly after worker bootstrap timeout, elapsed={elapsed:?}",
                )));
            }

            match store_result {
                Ok(_) => Err(boxed_error(
                    "expected connect() to fail when worker bootstrap timeout is too small",
                )),
                Err(DcsStoreError::Io(message)) => {
                    if !message.contains("timed out waiting for etcd worker startup") {
                        return Err(boxed_error(format!(
                            "expected startup-timeout io error, got: {message}"
                        )));
                    }
                    Ok(())
                }
                Err(other) => Err(boxed_error(format!(
                    "expected io error for startup timeout, got: {other}"
                ))),
            }
        }
        .await;

        shutdown_with_result(fixture, result).await
    }

    #[tokio::test(flavor = "current_thread")]
    async fn etcd_store_reconnect_resets_cache_when_snapshot_is_empty() -> TestResult {
        let fixture =
            RealEtcdFixture::spawn("dcs-etcd-store-reconnect-reset", "scope-reconnect").await?;

        let mut fixture = fixture;
        let result: TestResult = async {
            let mut store = EtcdDcsStore::connect(vec![fixture.endpoint.clone()], &fixture.scope)?;
            let mut cache = sample_cache(&fixture.scope);

            cache.members.insert(
                MemberId("node-stale".to_string()),
                MemberRecord {
                    member_id: MemberId("node-stale".to_string()),
                    postgres_host: "10.0.0.10".to_string(),
                    postgres_port: 5432,
                    role: MemberRole::Primary,
                    sql: SqlStatus::Healthy,
                    readiness: Readiness::Ready,
                    timeline: None,
                    write_lsn: None,
                    replay_lsn: None,
                    updated_at: UnixMillis(1),
                    pg_version: crate::state::Version(1),
                },
            );
            cache.switchover = Some(SwitchoverRequest {
                requested_by: MemberId("node-stale".to_string()),
            });
            cache.init_lock = Some(InitLockRecord {
                holder: MemberId("node-stale".to_string()),
            });

            cache.leader = Some(LeaderRecord {
                member_id: MemberId("node-stale".to_string()),
            });

            let stale_leader = serde_json::to_string(&LeaderRecord {
                member_id: MemberId("node-stale".to_string()),
            })
            .map_err(|err| boxed_error(format!("encode leader json failed: {err}")))?;

            {
                let mut guard = store
                    .events
                    .lock()
                    .map_err(|_| boxed_error("events lock poisoned"))?;
                guard.push_back(WatchEvent {
                    op: WatchOp::Put,
                    path: format!("/{}/leader", fixture.scope),
                    value: Some(stale_leader),
                    revision: 1,
                });
            }

            fixture.restart_clean().await?;

            let deadline = Instant::now() + Duration::from_secs(10);
            let mut observed_reset = false;
            while Instant::now() < deadline {
                let events = store.drain_watch_events()?;
                if events.is_empty() {
                    tokio::time::sleep(Duration::from_millis(50)).await;
                    continue;
                }

                if events.iter().any(|event| event.op == WatchOp::Reset) {
                    if events.iter().any(|event| {
                        event.op == WatchOp::Put
                            && event.path == format!("/{}/leader", fixture.scope)
                    }) {
                        return Err(boxed_error(
                            "expected reconnect to replace the watch queue (dropping stale leader PUT)",
                        ));
                    }
                    refresh_from_etcd_watch(&fixture.scope, &mut cache, events)?;
                    observed_reset = true;
                    break;
                }

                if events.iter().any(|event| {
                    event.op == WatchOp::Put && event.path == format!("/{}/leader", fixture.scope)
                }) {
                    return Err(boxed_error(
                        "observed leader PUT before reconnect Reset marker; stale events must be cleared during disconnect window",
                    ));
                }
                return Err(boxed_error(format!(
                    "observed watch events before reconnect Reset marker: {events:?}"
                )));
            }

            if !observed_reset {
                return Err(boxed_error(
                    "timed out waiting for reconnect snapshot reset marker",
                ));
            }

            if cache.leader.is_some() {
                return Err(boxed_error(
                    "expected leader record to be cleared by reconnect reset",
                ));
            }
            if !cache.members.is_empty() {
                return Err(boxed_error(
                    "expected members to be cleared by reconnect reset",
                ));
            }
            if cache.switchover.is_some() {
                return Err(boxed_error(
                    "expected switchover record to be cleared by reconnect reset",
                ));
            }
            if cache.init_lock.is_some() {
                return Err(boxed_error(
                    "expected init lock record to be cleared by reconnect reset",
                ));
            }

            Ok(())
        }
        .await;

        shutdown_with_result(fixture, result).await
    }

    #[tokio::test(flavor = "current_thread")]
    async fn etcd_store_disconnect_clears_pending_queue_before_reconnect_snapshot() -> TestResult {
        let fixture = RealEtcdFixture::spawn(
            "dcs-etcd-store-disconnect-clears-queue",
            "scope-disconnect-clears-queue",
        )
        .await?;

        let mut fixture = fixture;
        let result: TestResult = async {
            let mut store = EtcdDcsStore::connect(vec![fixture.endpoint.clone()], &fixture.scope)?;

            let stale_leader = serde_json::to_string(&LeaderRecord {
                member_id: MemberId("node-stale".to_string()),
            })
            .map_err(|err| boxed_error(format!("encode leader json failed: {err}")))?;

            {
                let mut guard = store
                    .events
                    .lock()
                    .map_err(|_| boxed_error("events lock poisoned"))?;
                guard.push_back(WatchEvent {
                    op: WatchOp::Put,
                    path: format!("/{}/leader", fixture.scope),
                    value: Some(stale_leader),
                    revision: 1,
                });
            }

            {
                let _delay_guard = EstablishDelayGuard::new(1000);
                fixture.restart_clean().await?;

                let events = store.drain_watch_events()?;
                if events.iter().any(|event| event.op != WatchOp::Reset) {
                    return Err(boxed_error(format!(
                        "expected disconnect to clear queued watch events before reconnect Reset (allowing only Reset markers); observed={events:?}"
                    )));
                }
                if events.iter().any(|event| event.op == WatchOp::Reset) {
                    return Ok(());
                }
            }

            let reset_deadline = Instant::now() + Duration::from_secs(10);
            while Instant::now() < reset_deadline {
                let events = store.drain_watch_events()?;
                if events.iter().any(|event| event.op == WatchOp::Reset) {
                    return Ok(());
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
            }

            Err(boxed_error(
                "timed out waiting for reconnect Reset marker after etcd restart",
            ))
        }
        .await;

        shutdown_with_result(fixture, result).await
    }

    #[tokio::test(flavor = "current_thread")]
    async fn etcd_store_round_trips_write_delete_and_events() -> TestResult {
        let fixture = RealEtcdFixture::spawn("dcs-etcd-store-roundtrip", "scope-a").await?;

        let fixture = fixture;
        let result: TestResult = async {
            let mut store = EtcdDcsStore::connect(vec![fixture.endpoint.clone()], &fixture.scope)?;
            let path = format!("/{}/member/node-a", fixture.scope);
            let value = r#"{"member_id":"node-a","role":"Primary"}"#.to_string();

            store.write_path(path.as_str(), value)?;
            wait_for_event(
                &mut store,
                WatchOp::Put,
                path.as_str(),
                Duration::from_secs(5),
            )?;

            store.delete_path(path.as_str())?;
            wait_for_event(
                &mut store,
                WatchOp::Delete,
                path.as_str(),
                Duration::from_secs(5),
            )?;

            Ok(())
        }
        .await;

        shutdown_with_result(fixture, result).await
    }

    #[tokio::test(flavor = "current_thread")]
    async fn etcd_store_put_if_absent_claims_only_once_and_does_not_overwrite() -> TestResult {
        let fixture = RealEtcdFixture::spawn("dcs-etcd-store-put-if-absent", "scope-put").await?;

        let fixture = fixture;
        let result: TestResult = async {
            let path_init = format!("/{}/init", fixture.scope);
            let path_config = format!("/{}/config", fixture.scope);

            let mut store_a = EtcdDcsStore::connect(vec![fixture.endpoint.clone()], &fixture.scope)?;
            let mut store_b = EtcdDcsStore::connect(vec![fixture.endpoint.clone()], &fixture.scope)?;

            let claimed_a = store_a.put_path_if_absent(path_init.as_str(), "init-a".to_string())?;
            let claimed_b = store_b.put_path_if_absent(path_init.as_str(), "init-b".to_string())?;
            if claimed_a == claimed_b {
                return Err(boxed_error(format!(
                    "expected exactly one init claim to succeed, got claimed_a={claimed_a} claimed_b={claimed_b}"
                )));
            }

            let seeded = store_a.put_path_if_absent(path_config.as_str(), "config-v1".to_string())?;
            if !seeded {
                return Err(boxed_error("expected config seed to succeed on first write"));
            }
            let seeded_again =
                store_b.put_path_if_absent(path_config.as_str(), "config-v2".to_string())?;
            if seeded_again {
                return Err(boxed_error(
                    "expected config seed to be rejected when key already exists",
                ));
            }

            let mut client = Client::connect(vec![fixture.endpoint.clone()], None)
                .await
                .map_err(|err| boxed_error(format!("etcd client connect failed: {err}")))?;
            let response = client
                .get(path_config.as_str(), None)
                .await
                .map_err(|err| boxed_error(format!("etcd get config failed: {err}")))?;
            let Some(kv) = response.kvs().first() else {
                return Err(boxed_error("expected config key to exist"));
            };
            let value = std::str::from_utf8(kv.value())
                .map_err(|err| boxed_error(format!("config value not utf8: {err}")))?;
            if value != "config-v1" {
                return Err(boxed_error(format!(
                    "expected config to remain 'config-v1', got: {value:?}"
                )));
            }

            Ok(())
        }
        .await;

        shutdown_with_result(fixture, result).await
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_consumes_real_etcd_watch_path_without_mocking() -> TestResult {
        let fixture = RealEtcdFixture::spawn("dcs-etcd-store-step-once", "scope-b").await?;

        let fixture = fixture;
        let result: TestResult = async {
            let store = EtcdDcsStore::connect(vec![fixture.endpoint.clone()], &fixture.scope)?;
            let mut client = Client::connect(vec![fixture.endpoint.clone()], None)
                .await
                .map_err(|err| boxed_error(format!("etcd client connect failed: {err}")))?;

            let leader_path = format!("/{}/leader", fixture.scope);
            let leader_json = serde_json::to_string(&LeaderRecord {
                member_id: MemberId("node-b".to_string()),
            })
            .map_err(|err| boxed_error(format!("encode leader json failed: {err}")))?;

            client
                .put(leader_path.as_str(), leader_json, None)
                .await
                .map_err(|err| boxed_error(format!("put leader key failed: {err}")))?;

            let (mut ctx, dcs_subscriber) = build_worker_ctx(&fixture.scope, store);
            let self_member = MemberId("node-a".to_string());
            let expected_leader = MemberId("node-b".to_string());

            let deadline = Instant::now() + Duration::from_secs(5);
            let mut observed = false;
            while Instant::now() < deadline {
                step_once(&mut ctx)
                    .await
                    .map_err(|err| boxed_error(format!("dcs step_once failed: {err}")))?;

                let latest = dcs_subscriber.latest();
                let leader_matches = latest
                    .value
                    .cache
                    .leader
                    .as_ref()
                    .map(|leader| leader.member_id.clone())
                    == Some(expected_leader.clone());
                let self_member_written = latest.value.cache.members.contains_key(&self_member);
                if leader_matches && self_member_written {
                    observed = true;
                    break;
                }

                tokio::time::sleep(Duration::from_millis(50)).await;
            }

            if !observed {
                return Err(boxed_error(
                    "timed out waiting for step_once to publish real-etcd leader/member refresh",
                ));
            }

            let member_path = format!("/{}/member/node-a", fixture.scope);
            let member_response = client
                .get(member_path.as_str(), None)
                .await
                .map_err(|err| boxed_error(format!("get member key failed: {err}")))?;
            if member_response.kvs().is_empty() {
                return Err(boxed_error(
                    "expected member key to be persisted at /{scope}/member/{id}",
                ));
            }

            Ok(())
        }
        .await;

        shutdown_with_result(fixture, result).await
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_marks_store_unhealthy_on_real_decode_failure() -> TestResult {
        let fixture = RealEtcdFixture::spawn("dcs-etcd-store-decode-failure", "scope-c").await?;

        let fixture = fixture;
        let result: TestResult = async {
            let store = EtcdDcsStore::connect(vec![fixture.endpoint.clone()], &fixture.scope)?;
            let mut client = Client::connect(vec![fixture.endpoint.clone()], None)
                .await
                .map_err(|err| boxed_error(format!("etcd client connect failed: {err}")))?;

            let leader_path = format!("/{}/leader", fixture.scope);
            client
                .put(leader_path.as_str(), "not-json", None)
                .await
                .map_err(|err| boxed_error(format!("put malformed leader key failed: {err}")))?;

            let (mut ctx, dcs_subscriber) = build_worker_ctx(&fixture.scope, store);
            let expected_worker =
                WorkerStatus::Faulted(WorkerError::Message("dcs store unhealthy".to_string()));

            let deadline = Instant::now() + Duration::from_secs(5);
            let mut observed_fault = false;
            while Instant::now() < deadline {
                step_once(&mut ctx)
                    .await
                    .map_err(|err| boxed_error(format!("dcs step_once failed: {err}")))?;

                let latest = dcs_subscriber.latest();
                if latest.value.worker == expected_worker
                    && latest.value.trust == DcsTrust::NotTrusted
                {
                    observed_fault = true;
                    break;
                }

                tokio::time::sleep(Duration::from_millis(50)).await;
            }

            if !observed_fault {
                return Err(boxed_error(
                    "timed out waiting for decode failure to fault dcs worker state",
                ));
            }

            Ok(())
        }
        .await;

        shutdown_with_result(fixture, result).await
    }

    #[tokio::test(flavor = "current_thread")]
    async fn etcd_store_write_reports_unreachable_endpoint() -> TestResult {
        match EtcdDcsStore::connect(vec!["http://127.0.0.1:1".to_string()], "scope-a") {
            Ok(mut store) => match store.write_path("/scope-a/member/node-a", "{}".to_string()) {
                Ok(_) => Err(boxed_error(
                    "expected write against unreachable endpoint to fail",
                )),
                Err(DcsStoreError::Io(_)) => Ok(()),
                Err(other) => Err(boxed_error(format!(
                    "expected io error for unreachable endpoint write, got {other}"
                ))),
            },
            Err(DcsStoreError::Io(_)) => Ok(()),
            Err(other) => Err(boxed_error(format!(
                "expected io error for unreachable endpoint connect, got {other}"
            ))),
        }
    }
}

--- END FILE: src/dcs/etcd_store.rs ---

