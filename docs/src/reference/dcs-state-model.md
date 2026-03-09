# DCS State Model

This page documents the DCS-backed state structures used by the runtime and the key layout used under the configured cluster scope.

## Trust Model

`DcsTrust` has three variants:

- `FreshQuorum`
- `NoFreshQuorum`
- `NotTrusted`

Trust evaluation follows this order:

1. If the backing store is unhealthy, trust is `NotTrusted`.
2. If the local member is missing from the cache, trust is `NoFreshQuorum`.
3. If the local member record is stale, trust is `NoFreshQuorum`.
4. If the observed cache has a multi-member view and fewer than two members are fresh, trust is `NoFreshQuorum`.
5. Otherwise trust is `FreshQuorum`.

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

When PostgreSQL state is unknown, member publication stays intentionally conservative:

- `timeline` is kept from current PostgreSQL state when available, otherwise from the previously published member record
- the previously published `write_lsn` is retained
- the previously published `replay_lsn` is retained

This means a temporarily unreachable node can still contribute its last known WAL evidence to promotion-safety checks while its member record remains fresh.

### `LeaderRecord`

`LeaderRecord` contains:

- `member_id`

`/{scope}/leader` is not a plain persistent key. In the etcd-backed store it is attached to an etcd lease whose TTL is derived from `ha.lease_ttl_ms`.

- while the owner keeps renewing the lease, the key stays present
- if the owner releases leadership explicitly, it revokes its own lease and etcd deletes the key
- if the owner dies hard and stops renewing, etcd expires the lease and deletes the key automatically

That means a missing leader member record does not itself force `NoFreshQuorum`. The authoritative signal for dead leadership is the disappearance of the lease-backed leader key from the watched DCS cache.

### `SwitchoverRequest`

`SwitchoverRequest` contains:

- `switchover_to: Option<MemberId>`

When `switchover_to` is `None`, the record means a generic switchover request is pending. When it is set, the record captures the requested target member for a targeted switchover.

The runtime keeps this record in DCS for the full handoff window. During a targeted switchover, non-target replicas continue to treat the request as blocking leadership until the requested successor becomes the observed primary. The record is cleared only after a safe success observer, normally the new primary, confirms the switchover completed.

### `ClusterInitializedRecord`

`ClusterInitializedRecord` contains:

- `initialized_by`
- `initialized_at`

### `ClusterIdentityRecord`

`ClusterIdentityRecord` contains:

- `system_identifier`
- `bootstrapped_by`
- `bootstrapped_at`

### `BootstrapLockRecord`

`BootstrapLockRecord` contains:

- `holder`

### `DcsView`

`DcsView` contains:

- `members: BTreeMap<MemberId, MemberRecord>`
- `leader: Option<LeaderRecord>`
- `switchover: Option<SwitchoverRequest>`
- `config: RuntimeConfig`
- `cluster_initialized: Option<ClusterInitializedRecord>`
- `cluster_identity: Option<ClusterIdentityRecord>`
- `bootstrap_lock: Option<BootstrapLockRecord>`

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
/{scope}/bootstrap
/{scope}/switchover
/{scope}/config
/{scope}/cluster/initialized
/{scope}/cluster/identity
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
- bootstrap-lock puts and deletes update `cache.bootstrap_lock`
- cluster-initialized puts and deletes update `cache.cluster_initialized`
- cluster-identity puts and deletes update `cache.cluster_identity`
- config puts replace `cache.config`

The local worker also republishes its own member record from current PostgreSQL state while the store is healthy.

For the etcd-backed implementation, lease expiry is visible through the normal watch path. When etcd deletes `/{scope}/leader` because the lease expired or was revoked, the watch-fed cache removes `cache.leader` and the HA loop sees that update through normal DCS state publication.

## Runtime Fields That Affect DCS Meaning

The DCS state model is not independent of runtime config. In particular:

- `dcs.endpoints` choose the coordination backend endpoints
- `dcs.scope` determines the prefix for all keys
- `ha.lease_ttl_ms` determines member freshness and the etcd leader-lease TTL

In the shipped docker cluster config, `ha.lease_ttl_ms` is `10000`.

## Operator Reading Notes

The DCS state model answers a different question from the debug API and the HA API:

- this page defines what the runtime stores and caches
- the debug API shows current snapshot views of that state
- the HA API reports a smaller operator-facing subset
