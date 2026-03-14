# DCS State Model

This page documents the DCS data model after the single-owner rewrite. The important distinction is:

- `DcsView` is the only public read-only surface exposed outside `src/dcs`
- internal record, cache, key, and etcd-adapter types stay private to the DCS component

## Trust Model

`DcsTrust` has three variants:

- `FullQuorum`
- `Degraded`
- `NotTrusted`

Trust evaluation follows this order:

1. If the backing store is unhealthy, trust is `NotTrusted`.
2. If the local member is missing from the observed member map, trust is `Degraded`.
3. If the observed member set does not meet quorum expectations, trust is `Degraded`.
5. Otherwise trust is `FullQuorum`.

The current implementation derives freshness from the etcd-backed member keys that remain live under their own leases, not from a public `updated_at` field.

## Core Types

### Public `DcsView`

`DcsView` contains:

- `worker`
- `trust`
- `members: BTreeMap<MemberId, DcsMemberView>`
- `leader: DcsLeaderStateView`
- `switchover: DcsSwitchoverStateView`
- `last_observed_at`

### Public member view

`DcsMemberView` contains:

- `member_id`
- `lease.ttl_ms`
- `routing.postgres.host`
- `routing.postgres.port`
- optional `routing.api.url`
- `postgres`

The public PostgreSQL view is one of:

- `Unknown { readiness, timeline }`
- `Primary { readiness, committed_wal }`
- `Replica { readiness, upstream, replay_wal, follow_wal }`

These are observation shapes, not raw etcd records.

### Public leader view

`DcsLeaderStateView` is either:

- `Unheld`
- `Held { holder, generation }`

`/{scope}/leader` is not a plain persistent key. In the etcd-backed store it is attached to an etcd lease whose TTL is derived from `ha.lease_ttl_ms`.

- while the owner keeps renewing the lease, the key stays present
- if the owner releases leadership explicitly, it revokes its own lease and etcd deletes the key
- if the owner dies hard and stops renewing, etcd expires the lease and deletes the key automatically

That means a missing leader member record does not itself force `FailSafe`. The authoritative signal for dead leadership is the disappearance of the lease-backed leader key from the watched DCS cache.

`generation` turns the leader record into a lease epoch rather than just a member label. Operators and the HA API use that epoch to distinguish one leadership term from the next even when the same member regains leadership.

### Public switchover view

`DcsSwitchoverStateView` is either:

- `None`
- `Requested { target }`

`target` is one of:

- `AnyHealthyReplica`
- `Specific(MemberId)`

The request stays in DCS while the HA loop coordinates the handoff, and clears only when the system decides the request is complete or an operator clears it explicitly.

### Internal-only cache and record types

Inside `src/dcs`, the worker still maintains private `*Record` and `*Cache` types such as `MemberRecord`, `LeaderLeaseRecord`, `SwitchoverRecord`, and `DcsCache`. Those types are implementation details and are not part of the architectural boundary anymore.

## Key Layout

All DCS keys are scoped under the configured cluster scope:

```text
/{scope}/leader
/{scope}/switchover
/{scope}/config
/{scope}/init
/{scope}/member/{member_id}
```

The key parser remains internal to `src/dcs`; non-DCS modules do not construct or interpret these raw paths directly.

## Watch and Cache Updates

The DCS worker applies parsed updates into the cache:

- member puts and deletes update the internal member-record map
- leader puts and deletes update the internal leader record
- switchover puts and deletes update the internal switchover record
- init-lock puts and deletes update the internal init-lock record

The local worker also republishes its own member record from current PostgreSQL state while the store is healthy.

For the etcd-backed implementation, lease expiry is visible through the normal watch path. When etcd deletes `/{scope}/leader` because the lease expired or was revoked, the watch-fed cache removes the leader record and the HA loop sees that update through normal `DcsView` publication.

## Runtime Fields That Affect DCS Meaning

The DCS state model is not independent of runtime config. In particular:

- `dcs.endpoints` choose the coordination backend endpoints
- `dcs.scope` determines the prefix for all keys
- `ha.lease_ttl_ms` determines member freshness and the etcd leader-lease TTL

In the shipped docker cluster config, `ha.lease_ttl_ms` is `10000`.

## Operator Reading Notes

The DCS state model answers a different question from the debug API and the HA API:

- this page defines the DCS-owned storage model and the public `DcsView`
- the debug/API state surfaces show one node's current published snapshot of that view
- HA derives a separate `WorldView` from the read-only DCS snapshot rather than from raw etcd paths
