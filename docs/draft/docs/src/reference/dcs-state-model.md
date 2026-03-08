# DCS State Model

## Overview

The DCS (Distributed Coordination Service) state model defines the data structures that persist in etcd and their in-memory representation. Three trust levels govern whether the DCS cache can be used for consensus decisions: `FullQuorum`, `FailSafe`, and `NotTrusted`.

[diagram showing DcsState struct containing DcsCache, DcsTrust, and WorkerStatus, with arrows to member records, leader record, and config]

## Core Structures

### DcsTrust

Enumeration of trust levels for DCS quorum decisions.

| Variant | Description |
|---------|-------------|
| `FullQuorum` | Sufficient members are fresh; leader and self are present and valid. Safe for consensus operations. |
| `FailSafe` | Some records may be stale or missing. Safe for read-only decisions. |
| `NotTrusted` | DCS store is unhealthy or local member cannot publish. All DCS-derived decisions are blocked. |

### MemberRole

Enumeration of PostgreSQL replication roles reported by each member.

| Variant | Description |
|---------|-------------|
| `Unknown` | Member state is indeterminate; PostgreSQL not yet probed or in transition. |
| `Primary` | Member reports itself as a read-write primary with WAL generation. |
| `Replica` | Member reports itself as a read-only replica with WAL replay progress. |

### MemberRecord

Per-member state published by each node to `/{scope}/member/{member_id}`.

| Field | Type | Description |
|-------|------|-------------|
| `member_id` | MemberId | Unique identifier for the cluster member. |
| `postgres_host` | String | Listen address the member advertises for client connections. |
| `postgres_port` | u16 | Listen port for client connections. |
| `role` | MemberRole | Last observed replication role. |
| `sql` | SqlStatus | SQL health probe result: Unknown, Healthy, or Unhealthy. |
| `readiness` | Readiness | Readiness probe result: Unknown, Starting, Recovering, or Ready. |
| `timeline` | Option<TimelineId> | Current timeline if known. |
| `write_lsn` | Option<WalLsn> | Last generated LSN if role is Primary. |
| `replay_lsn` | Option<WalLsn> | Last replayed LSN if role is Replica. |
| `updated_at` | UnixMillis | Milliseconds since epoch when this record was written. |
| `pg_version` | Version | PostgreSQL major version number (e.g., 16). |

All fields except `timeline`, `write_lsn`, and `replay_lsn` are required.

### LeaderRecord

Singleton record at `/{scope}/leader` indicating the current elected primary.

| Field | Type | Description |
|-------|------|-------------|
| `member_id` | MemberId | Identifier of the member acting as primary. |

### SwitchoverRequest

Singleton record at `/{scope}/switchover` requesting a planned primary change.

| Field | Type | Description |
|-------------|------|-------------|
| `requested_by` | MemberId | Member that created the request. |

### InitLockRecord

Singleton record at `/{scope}/init` held during one-time cluster initialization.

| Field | Type | Description |
|-------------|------|-------------|
| `holder` | MemberId | Member performing initdb or bootstrapping. |

### DcsCache

In-memory snapshot of all persisted keys.

| Field | Type | Description |
|-------|------|-------------|
| `members` | BTreeMap<MemberId, MemberRecord> | Map of all member records, indexed by member ID. |
| `leader` | Option<LeaderRecord> | Current leader if any. |
| `switchover` | Option<SwitchoverRequest> | Pending switchover if any. |
| `config` | RuntimeConfig | Full HA and DCS configuration used for lease interpretation. |
| `init_lock` | Option<InitLockRecord> | Init lock if held. |

### DcsState

Published by the DCS worker each polling interval.

| Field | Type | Description |
|-------|------|-------------|
| `worker` | WorkerStatus | DCS worker health: Starting, Running, or Faulted. |
| `trust` | DcsTrust | Computed trust level for this interval. |
| `cache` | DcsCache | Snapshot of keys and config. |
| `last_refresh_at` | Option<UnixMillis> | Timestamp when this snapshot was assembled. |

## Key Layout

All keys are prefixed by the configured scope. The scope is defined in `runtime.toml` under `[dcs] scope = "..."`.

```
/{scope}/
├── member/{member_id}  -> MemberRecord (JSON)
├── leader             -> LeaderRecord (JSON)
├── switchover         -> SwitchoverRequest (JSON)
├── config             -> RuntimeConfig
└── init               -> InitLockRecord (JSON)
```

// todo: The requested sources establish strict key parsing, but this sentence mixes path parsing with payload decode behavior too loosely. Rephrase with separate claims for path parsing and watch-update handling.

Key parsing is strict; unknown paths or malformed payloads are not accepted as valid state updates.

[diagram showing etcd key hierarchy under a scope, with JSON payloads for each key type]

## Trust Evaluation Rules

Trust is recomputed each polling interval based on store health, member presence, and record freshness.

### Preconditions

- If the etcd client reports unhealthy, trust is `NotTrusted`.

// todo: The requested sources support store-unhealthy leading to `NotTrusted`, but the exact phrasing here about every local-member write failure needs tighter grounding in worker behavior.

### Member Freshness

A member record is fresh when:

```
(now - updated_at) <= cache.config.ha.lease_ttl_ms
```

The lease TTL is defined in `runtime.toml` under `[ha] lease_ttl_ms`.

### Evaluation Steps

1. **Self Check**: If the local member ID is absent from the cache, trust is `FailSafe`.
2. **Self Freshness**: If the local member record is stale, trust is `FailSafe`.
3. **Leader Validity**: If a leader record exists but the corresponding member is missing or stale, trust is `FailSafe`.
4. **Quorum Size**: If total member count exceeds one and fewer than two members are fresh, trust is `FailSafe`.
5. **Full Quorum**: Otherwise trust is `FullQuorum`.

[flowchart showing decision tree from store health through each check to final trust level]

The `config` embedded in `DcsCache` provides `lease_ttl_ms` and other HA parameters that affect trust.

## Record Publication and Updates

Each node publishes its own `MemberRecord` to `/{scope}/member/{self_id}` every polling interval, overwriting the previous value. The record reflects the latest PostgreSQL state snapshot and includes the PostgreSQL-state version to detect changes.

The DCS worker also watches the etcd keyspace and applies remote changes to its local cache via `apply_watch_update`. Deletes remove keys from the cache; puts insert or replace them.

## Time and Versioning

- `UnixMillis` is milliseconds since Unix epoch.
- `Version` is a generic version type used throughout state publication.
- `WalLsn` and `TimelineId` follow PostgreSQL wire protocol types.
- Timestamps and versions enable staleness detection and conflict resolution.

// todo: The original sentence describing `Version` as specifically the PostgreSQL major version was too strong for every use in these structures.

## Related Documentation

- [HTTP API](http-api.md) for reading DCS state via `/status`.
// todo: The `/status` path reference above is not established by the requested sources; verify or replace with the actual documented endpoint.
- [Debug API](debug-api.md) for snapshots of the internal cache.
- [HA Decisions](ha-decisions.md) for how trust influences failover logic.
