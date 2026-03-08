# DCS State Model

This page documents the DCS-backed state structures used by the runtime and the key layout used under the configured cluster scope.

## Trust Model

`DcsTrust` has three variants:

- `FullQuorum`
- `FailSafe`
- `NotTrusted`

Trust evaluation follows this order:

1. If the backing store is unhealthy, trust is `NotTrusted`.
2. If the local member is missing from the cache, trust is `FailSafe`.
3. If the local member record is stale, trust is `FailSafe`.
4. If a leader record exists but that member record is missing or stale, trust is `FailSafe`.
5. If the cache contains more than one member and fewer than two members are fresh, trust is `FailSafe`.
6. Otherwise trust is `FullQuorum`.

Freshness is evaluated from:

```text
now - updated_at <= cache.config.ha.lease_ttl_ms
```

## Core Types

### `MemberRole`

- `Unknown`
- `Primary`
- `Replica`

### `MemberRecord`

`MemberRecord` contains:

- `member_id`
- `postgres_host`
- `postgres_port`
- `role`
- `sql`
- `readiness`
- `timeline`
- `write_lsn`
- `replay_lsn`
- `updated_at`
- `pg_version`

Optional fields:

- `timeline`
- `write_lsn`
- `replay_lsn`

The worker builds member records from the latest PostgreSQL state:

- unknown PostgreSQL state publishes `role = Unknown`
- primary PostgreSQL state publishes `role = Primary` and `write_lsn`
- replica PostgreSQL state publishes `role = Replica` and `replay_lsn`

### `LeaderRecord`

`LeaderRecord` contains:

- `member_id`

### `SwitchoverRequest`

`SwitchoverRequest` contains:

- no fields; the record is a marker that a generic switchover request is pending

### `InitLockRecord`

`InitLockRecord` contains:

- `holder`

### `DcsCache`

`DcsCache` contains:

- `members: BTreeMap<MemberId, MemberRecord>`
- `leader: Option<LeaderRecord>`
- `switchover: Option<SwitchoverRequest>`
- `config: RuntimeConfig`
- `init_lock: Option<InitLockRecord>`

### `DcsState`

`DcsState` contains:

- `worker`
- `trust`
- `cache`
- `last_refresh_at`

`last_refresh_at` is optional.

## Key Layout

All DCS keys are scoped under the configured cluster scope:

```text
/{scope}/leader
/{scope}/switchover
/{scope}/config
/{scope}/init
/{scope}/member/{member_id}
```

`key_from_path(...)` accepts exactly those shapes and rejects:

- wrong scope prefixes
- malformed paths
- missing member IDs
- unknown extra segments

## Watch and Cache Updates

The DCS worker applies parsed updates into the cache:

- member puts and deletes update `cache.members`
- leader puts and deletes update `cache.leader`
- switchover puts and deletes update `cache.switchover`
- init-lock puts and deletes update `cache.init_lock`
- config puts replace `cache.config`

The local worker also republishes its own member record from current PostgreSQL state while the store is healthy.

## Runtime Fields That Affect DCS Meaning

The DCS state model is not independent of runtime config. In particular:

- `dcs.endpoints` choose the coordination backend endpoints
- `dcs.scope` determines the prefix for all keys
- `ha.lease_ttl_ms` determines freshness and therefore trust

In the shipped docker cluster config, `ha.lease_ttl_ms` is `10000`.

## Operator Reading Notes

The DCS state model answers a different question from the debug API and the HA API:

- this page defines what the runtime stores and caches
- the debug API shows current snapshot views of that state
- the HA API reports a smaller operator-facing subset
