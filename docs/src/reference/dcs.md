# DCS Reference

The `dcs` subsystem maintains cluster coordination state, a local cache of that state, and the worker loop that keeps the cache synchronized with the backing store.

## Module layout

| Module | Surface |
| --- | --- |
| `dcs::store` | Store trait, watch event types, cache refresh helpers, and local member writes |
| `dcs::keys` | Parsing of scope-relative key paths into typed keys |
| `dcs::state` | Cached records, trust evaluation, and worker context/state types |
| `dcs::worker` | Periodic synchronization loop and watch update application |
| `dcs::etcd_store` | etcd-backed store implementation |

## Key namespace

`key_from_path` accepts only these scope-relative keys:

| Path | Meaning |
| --- | --- |
| `/<scope>/member/<member_id>` | Member record for one node |
| `/<scope>/leader` | Current leader lease holder |
| `/<scope>/switchover` | Pending switchover request |
| `/<scope>/config` | Runtime configuration document |
| `/<scope>/init` | Bootstrap init lock |

Other paths are rejected as malformed, missing a member id, outside the configured scope, or unknown.

## Store and watch surface

`DcsStore` defines the backing-store contract:

| Method | Meaning |
| --- | --- |
| `healthy` | Current health signal for the store connection |
| `read_path` | Read a single path |
| `write_path` | Unconditional write |
| `put_path_if_absent` | Conditional create |
| `delete_path` | Delete one path |
| `drain_watch_events` | Return queued watch events for this polling step |

`WatchEvent` carries an operation, full path, optional value, and revision. `WatchOp` contains `Put`, `Delete`, and `Reset`. `Reset` is synthesized during reconnect or resnapshot and instructs the cache refresh logic to discard previously cached scope state before applying the following snapshot.

## Cached state

`DcsCache` contains:

| Field | Meaning |
| --- | --- |
| `members` | `member_id -> MemberRecord` map |
| `leader` | Current `LeaderRecord`, if present |
| `switchover` | Current `SwitchoverRequest`, if present |
| `config` | Current runtime configuration snapshot |
| `init_lock` | Current `InitLockRecord`, if present |

`MemberRecord` stores the member id, PostgreSQL host and port, observed role, SQL status, readiness, timeline, write and replay LSNs, `updated_at`, and PostgreSQL version.

`DcsState` combines the worker status, trust level, cache, and `last_refresh_at` timestamp.

## Trust evaluation

`evaluate_trust` returns one of three states:

| State | Conditions |
| --- | --- |
| `full_quorum` | Store is healthy, the local member record is present and fresh, the leader record is fresh when present, and multi-member scopes have at least two fresh members |
| `fail_safe` | Store is healthy, but the cache is incomplete or stale for quorum purposes |
| `not_trusted` | Store health is false or local member publication failed |

Freshness is measured against `cache.config.ha.lease_ttl_ms`.

## Worker step behavior

`dcs::worker::step_once` performs one polling iteration:

1. Read the latest PostgreSQL state and build a local `MemberRecord`.
2. Write that record into `/<scope>/member/<self_id>` when the store is healthy.
3. Drain watch events from the store.
4. Apply watch updates into the local `DcsCache`.
5. Recompute trust and publish the next `DcsState`.
6. Emit log events for store-health transitions, trust transitions, write failures, and refresh failures.

`refresh_from_etcd_watch` applies typed `Put` and `Delete` updates for members, leader lease, switchover request, config, and init lock. `write_local_member` serializes a `MemberRecord` to JSON and writes it under the member path for the configured scope.

## Error surface

`DcsStoreError` covers:

| Variant | Meaning |
| --- | --- |
| `MissingValue` | A `Put` watch event arrived without a value |
| `InvalidKey` | A path could not be parsed into a supported DCS key |
| `Decode` | JSON decoding failed for a typed record |
| `AlreadyExists` | Conditional leader-lease creation found an existing path |
| `Io` | Store I/O failed |
