# Verbose Extra Context For `docs/src/explanation/trust-model.md`

This note exists to give K2 exhaustive raw context about the current trust model after the DCS rewrite. It is intentionally factual and implementation-centered.

## Current naming and public model

The old `FullQuorum` naming is no longer the current implementation. The public DCS coordination surface is now:

- `DcsView::NotTrusted(NotTrustedView)`
- `DcsView::Degraded(ClusterView)`
- `DcsView::Coordinated(ClusterView)`

The corresponding mode enum is:

- `DcsMode::NotTrusted`
- `DcsMode::Degraded`
- `DcsMode::Coordinated`

This matters for docs wording. The current code does not model "full quorum mathematics". The implementation uses simpler coordination-mode gates that are driven by etcd reachability, self visibility in the DCS member set, and a minimal member-count rule.

## Exact trust / coordination evaluation logic

The core logic is in `src/dcs/state.rs` in `evaluate_mode(etcd_reachable, cache, self_id)`.

The current decision order is:

1. If etcd is not reachable, return `DcsMode::NotTrusted`.
2. If the local node does not see its own member record in the DCS cache, return `DcsMode::Degraded`.
3. If the cache fails the member-count rule, return `DcsMode::Degraded`.
4. Otherwise return `DcsMode::Coordinated`.

So the current model is intentionally conservative but also intentionally simple:

- `NotTrusted` means the DCS transport itself is not currently trusted.
- `Degraded` means etcd is reachable, but the node does not have enough confidence in the cluster view to treat it as fully coordinated.
- `Coordinated` means etcd is reachable, the local member record is present, and the minimal member rule passes.

## Exact member-count rule

The member-count rule is implemented by `has_member_quorum(cache)` in `src/dcs/state.rs`.

The exact rule is:

- if there is 1 or fewer visible members, only exactly 1 member counts as coordinated
- if there are 2 or more visible members, at least 2 members are required for coordinated mode

This is not majority quorum math. It is a small safety rule that distinguishes:

- single-node deployments, where one visible member is enough
- multi-member deployments, where seeing only one member is treated as degraded

The docs should explain that clearly and should not imply the system is calculating a mathematical majority across arbitrary cluster sizes in this code path.

## What "fresh" means in practice

The raw config knobs are in `src/config/schema.rs`:

- `ha.loop_interval_ms`
- `ha.lease_ttl_ms`

The current defaults are defined in `src/config/defaults.rs`:

- `DEFAULT_HA_LOOP_INTERVAL_MS = 1_000`
- `DEFAULT_HA_LEASE_TTL_MS = 10_000`

These values are used as follows:

- `runtime/node.rs` converts `cfg.ha.loop_interval_ms` into the worker poll interval.
- `runtime/node.rs` passes `cfg.ha.lease_ttl_ms` into DCS as `DcsCadence.member_ttl_ms`.
- `dcs/worker.rs` uses `member_ttl_ms` as the staleness cutoff for the local PostgreSQL observation during member publication.

The exact local publication freshness check in `dcs/worker.rs` is:

- read the latest PostgreSQL observation timestamp with `pg_snapshot.last_refresh_at()`
- compare `now - last_refresh_at` against `member_ttl_ms`
- if that age is greater than `member_ttl_ms`, the local member entry is deleted from etcd and any locally held leadership lease is released

That means "fresh" is operationally defined as:

- the node has a recent PostgreSQL observation
- the age of that observation is not older than `ha.lease_ttl_ms`

When the observation is older than that threshold, the node stops advertising itself as a valid DCS member.

## Why self-member visibility matters

`evaluate_mode()` explicitly requires that the local node can still see its own member record in the DCS cache. This gives an important safety property:

- if the node can talk to etcd but does not see itself in the authoritative member set, it does not treat the cluster as fully coordinated

This is separate from transport reachability:

- etcd reachable but self missing => `Degraded`
- etcd unreachable => `NotTrusted`

That distinction is important for explaining why the node may still have some cluster information visible while refusing to act with full coordination authority.

## What `NotTrusted` still exposes

Although `NotTrusted` is the most conservative mode, the current implementation still retains the last observed cluster snapshot internally and exposes it through `DcsView::cluster()`. It also preserves the last observed leadership through `NotTrustedView`.

That behavior exists so the system can keep publishing conservative operator-visible information and fail-safe fencing context during DCS outages instead of collapsing the view to an empty cluster.

Docs should describe this carefully:

- `NotTrusted` does not mean "no information exists"
- it means "current DCS coordination cannot be trusted enough for authoritative decisions"

## HA decision boundary

`src/ha/decide.rs` currently uses a very hard gate:

- if `world.global.coordination.mode != DcsMode::Coordinated`, HA goes through `decide_degraded(world)`

So from the HA policy perspective, the meaningful split is:

- `Coordinated`: normal leader/follower decision logic is allowed
- `Degraded` or `NotTrusted`: the system takes fail-safe behavior

The current degraded-path outcomes include:

- a primary may be forced into `FailSafeGoal::PrimaryMustStop(...)`
- replicas may use `FailSafeGoal::ReplicaKeepFollowing(...)`
- nodes may wait for quorum with `FailSafeGoal::WaitForQuorum`
- public no-primary projections use `NoPrimaryProjection::DcsDegraded`

This is the critical docs point: trust mode is not cosmetic status text. It directly gates whether HA is allowed to do normal coordinated failover behavior.

## Example scenario requested by K2

The feature `tests/ha/features/ha_dcs_quorum_lost_enters_failsafe/ha_dcs_quorum_lost_enters_failsafe.feature` is a good concrete narrative because it shows:

- a healthy cluster starting with one stable primary
- loss of a DCS quorum majority
- disappearance of the operator-visible primary
- every running node reporting `fail_safe`
- no dual-primary evidence during the transition
- restoration of DCS quorum
- recovery back to one stable primary

That feature is useful because it demonstrates the operator-facing consequence of the trust model:

- trust loss is intentionally conservative
- visibility of authority is withdrawn before the system is willing to claim a primary again

## One wording constraint for the final doc

Please avoid describing the current model as:

- "full quorum"
- "strict quorum calculation"
- "majority math"

Those phrases do not match the current code. The documentation should instead explain the actual implemented safety gate:

- etcd reachability
- self-member visibility
- minimal member-count rule
- HA fail-safe gating when the mode is not `Coordinated`
