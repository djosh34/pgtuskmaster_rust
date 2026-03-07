# Draft: About DCS Trust and HA Gating

Compass classification: `cognition + acquisition` because this page explains why trust exists as a separate concept in the runtime and how that shapes HA behavior.

`pgtuskmaster` does not treat "the DCS is reachable" as equivalent to "HA decisions are safe." The runtime inserts a trust layer between raw DCS connectivity and high-availability action so that the node can distinguish between healthy coordination, degraded coordination, and no trustworthy coordination at all.

## Why trust is separate from store health

The DCS worker continuously builds a typed cache of member records, the leader record, switchover state, runtime config, and the init lock. It then evaluates trust from the store health signal plus the freshness and completeness of that cache.

That produces three states:

- `full_quorum` when the store is healthy and the cache is fresh enough to support coordination
- `fail_safe` when the store is reachable but the cache is incomplete or stale for quorum purposes
- `not_trusted` when the store itself is unhealthy or the local member record could not be published

This distinction matters because a cluster can be connected to etcd and still be missing the evidence needed for safe leader or replica decisions.

## What HA does with trust

The HA state machine checks trust before it considers its normal phase logic. If trust is anything other than `full_quorum`, HA does not continue through ordinary replica, candidate-leader, or primary decision paths.

Instead:

- a node that is not primary stays in `fail_safe` without introducing new role changes
- a node that is primary enters fail-safe with an explicit safety decision

That design reveals the intention behind trust gating: the system would rather stop making ambitious cluster changes than act on a partial or stale view of leadership.

## Why freshness is tied to lease TTL

Freshness in the DCS cache is measured against `ha.lease_ttl_ms`. The same timing window that bounds leader lease expectations also bounds how old member evidence may be before trust is downgraded.

This keeps lease semantics and coordination confidence tied together. The code is not asking only, "Do I have a record?" It is asking, "Do I have a record fresh enough to support the lease-based view of this cluster?"

## Fail-safe is a coordination posture, not an implementation detail

`fail_safe` is easy to misread as a temporary error bucket, but the surrounding HA logic shows that it is a deliberate operating posture. When trust drops, the node narrows its behavior until trustworthy coordination returns. That posture is especially visible when a primary loses trust: the system favors fencing-oriented safety behavior instead of assuming that the last known leader view is still good enough.

## Why this page is explanation rather than reference

The reference page can list the three trust states and their conditions. The explanation is why the trust layer exists at all: DCS reachability alone is not the same thing as safe coordination, and HA is designed to defer cluster-changing action until it has both store health and sufficiently fresh cluster evidence.

See also:

- [DCS Reference](../src/reference/dcs.md)
- [HA Reference](../src/reference/ha.md)
