# DCS State Model

The cluster state exposed by the distributed coordination service is a two-state algebraic data type. A node either has quorum and can publish authoritative cluster data, or it has no quorum and must withhold that data.

## Type hierarchy

```text
DcsSnapshot
- NoQuorum
- Quorum(DcsQuorumState)

DcsQuorumState
- leadership: Option<LeaseEpoch>
- switchover: SwitchoverState
- members: BTreeMap<MemberId, DcsMemberState>

DcsMemberState
- postgres_endpoint: PgEndpoint
- postgres: PgInfoState

DcsAuthority
- NoQuorum
- Quorum
```

## Authority and public data visibility

`DcsAuthority` mirrors the active `DcsSnapshot` variant.

### NoQuorum

```text
DcsSnapshot::NoQuorum
```

Public accessors expose no cluster payload:

- `mode_label()` returns `"no_quorum"`
- `authority()` returns `DcsAuthority::NoQuorum`
- `is_quorum()` returns `false`
- `leadership()` returns `None`
- `switchover()` returns `None`
- `members()` iterates zero items
- `member_ids()` iterates zero items
- `member_count()` returns `0`
- `member(_)` returns `None`
- `quorum_state()` returns `None`

This is the main safety guarantee of the current model: outside authoritative quorum, the public DCS surface does not expose stale members, stale leadership, or stale switchover state.

### Quorum

```text
DcsSnapshot::Quorum(DcsQuorumState {
    leadership: Option<LeaseEpoch>,
    switchover: SwitchoverState,
    members: BTreeMap<MemberId, DcsMemberState>,
})
```

Public accessors expose the authoritative cluster payload:

- `mode_label()` returns `"quorum"`
- `authority()` returns `DcsAuthority::Quorum`
- `is_quorum()` returns `true`
- `leadership()` returns `Some(&LeaseEpoch)` or `None` when the lease is open
- `switchover()` returns `Some(&SwitchoverState)`; the no-request case is represented by `SwitchoverState::None`
- `members()` iterates `(&MemberId, &DcsMemberState)` pairs
- `member_ids()` iterates `&MemberId` values
- `member_count()` returns the number of quorum-visible members
- `member(member_id)` returns `Some(&DcsMemberState)` when the member exists
- `quorum_state()` returns `Some(&DcsQuorumState)`

## Member payload

`DcsMemberState` is the public per-member payload inside a quorum snapshot.

```text
pub struct DcsMemberState {
    pub postgres_endpoint: PgEndpoint,
    pub postgres: PgInfoState,
}
```

- `postgres_endpoint` is the advertised PostgreSQL endpoint for the member
- `postgres` is the current PostgreSQL observation for that member

These fields are public and serializable as part of the published snapshot and API surface. The etcd-backed store still uses internal DCS record types rather than writing `DcsMemberState` directly.

## Constructing snapshots

### Initial state

`DcsSnapshot::starting()` returns `NoQuorum`.

### Quorum snapshot from parts

```text
DcsSnapshot::quorum(
    leadership: Option<LeaseEpoch>,
    switchover: SwitchoverState,
    members: BTreeMap<MemberId, DcsMemberState>,
) -> DcsSnapshot
```

## Quorum logic

`current_snapshot(...)` computes the published DCS view.

It returns `NoQuorum` when either of these conditions is true:

- the etcd client is not reachable
- `has_member_quorum(members)` is false

`has_member_quorum(...)` uses the current minimal rule:

- zero members -> false
- one member -> true
- two or more members -> require at least two members

This is not majority math over the whole deployment. It is the current rule used to decide whether the public snapshot may expose cluster data at all.

## Authority transitions

The worker records `DcsLogEvent::CoordinationModeTransition` when the published authority changes. The event includes the previous and next labels, which makes `no_quorum <-> quorum` transitions visible in logs.

The worker also has a dedicated `LeaderLeaseExpired` path. When the node's own leader keepalive reports an expired lease, the worker drops only that owned leader lease, reloads the authoritative snapshot, and republishes based on the reloaded state instead of forcing a full disconnected `NoQuorum` transition first.

## Internal state versus published state

The DCS runtime still keeps internal in-memory fields for:

- `members`
- `leadership`
- `switchover`

Those internal values can exist even when the published snapshot is `NoQuorum`. They are implementation detail, not public authority. The public contract is that cluster topology is withheld until quorum is restored.

## Downstream consumption

The HA layer mirrors this split with:

- `CoordinationState::NoQuorum`
- `CoordinationState::Quorum(Box<QuorumCoordinationState>)`

When the published DCS view is `NoQuorum`, HA routes into fail-safe and projects `NoPrimaryProjection::NoQuorum`.

The CLI status command derives warnings from that same published view:

- `degraded_dcs_mode` when `!state.dcs.is_quorum()`
- `no_members` when `member_count() == 0`
- `no_primary` when there is no authoritative primary projection

That means operator-facing command health can still be degraded while the DCS snapshot itself remains a strict two-state model.

## Example sequence

1. The worker connects to etcd and loads a valid snapshot -> publishes `Quorum`.
2. The node loses etcd reachability or member quorum -> publishes `NoQuorum`.
3. HA withdraws the operator-visible primary and enters fail-safe behavior.
4. Etcd connectivity and member quorum recover -> the worker republishes `Quorum`.

During step 2 and step 3, public DCS accessors intentionally yield empty results instead of stale cluster state.
