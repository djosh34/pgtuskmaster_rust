# Failure modes deep summary

This support note is only raw factual context for `docs/src/explanation/failure-modes.md`.
Explain why the system behaves this way, but do not overclaim unsupported timers, retries, or operator outcomes.

High-level failure model:

- Failure handling is split first by DCS trust, then by local PostgreSQL reachability.
- `DcsTrust` values are `FullQuorum`, `FailSafe`, and `NotTrusted`.
- Trust is evaluated from etcd health and the freshness of cached member records.

Trust evaluation details from `src/dcs/state.rs`:

- `NotTrusted` if etcd itself is unhealthy.
- `FailSafe` if the local member record is missing.
- `FailSafe` if the local member record is stale.
- `FailSafe` if a recorded leader points to a missing member.
- `FailSafe` if a recorded leader points to a stale member.
- `FailSafe` in clusters with more than one member when fewer than two fresh member records remain.
- `FullQuorum` only when those checks all pass.
- Freshness is bounded by `cache.config.ha.lease_ttl_ms`.
- Local member records are built directly from local PostgreSQL state, so an unreachable local server can still remain represented in DCS with `role: Unknown` rather than disappearing immediately.

HA behavior when trust is degraded:

- As soon as HA sees anything other than `FullQuorum`, it routes to `FailSafe`.
- If local PostgreSQL is still a healthy primary, the emitted decision is `EnterFailSafe { release_leader_lease: false }`.
- Otherwise the node enters `FailSafe` with `NoChange`.
- Tests in `src/ha/decide.rs` explicitly assert this degraded-trust behavior.
- Once already in `FailSafe`, the node remains there while trust is degraded.
- On restored quorum, the node exits toward trusted-waiting flow rather than using a separate ad hoc recovery mode.

PostgreSQL reachability behavior:

- The code treats PostgreSQL as reachable only when `SqlStatus::Healthy`.
- `Unknown` and `Unreachable` both count as not reachable.
- Non-primary phases fall back to `WaitingPostgresReachable` and keep emitting `WaitForPostgres { start_requested, leader_member_id }`.
- The node may keep waiting here without a source-backed timeout escalation if DCS is still healthy but PostgreSQL remains unavailable.

Source-backed answer to the requested extra question:

- When DCS is healthy but local PostgreSQL is unreachable for an extended period:
  - if the node is not acting as primary, HA continues returning to `WaitingPostgresReachable` and emitting `WaitForPostgres`
  - there is no allowed-source evidence of a timeout that escalates this case to fail-safe or fencing purely because the outage is long-lived
  - if the node is in `Primary` and still holds the leader lease, it releases the leader lease with reason `PostgresUnreachable` and transitions to `Rewinding`
- After entering `Rewinding`, the node either:
  - starts rewind toward a healthy primary
  - falls back to base backup after rewind failure
  - or remains in `Rewinding` with `NoChange` until a usable recovery leader exists

Actions and side effects visible in the HA/action surface:

- `StartPostgres`
- `ReleaseLeaderLease`
- `StartRewind`
- `StartBaseBackup`
- `FenceNode`
- `SignalFailSafe`
- These action ids align with the decision engine and are useful conceptual anchors for the explanation page.

etcd-backed store failure behavior from `src/dcs/etcd_store.rs`:

- The etcd store marks itself unhealthy on command send failures, response failures, watch failures, and decode errors.
- When that happens it can clear the current watch session.
- A real-etcd test demonstrates malformed leader JSON driving the DCS worker into a faulted state with trust downgraded to `NotTrusted`.

Observer behavior that frames split-brain claims:

- The HA observer counts “all sampled nodes are in fail-safe” as a concrete condition.
- It tracks leader changes across samples.
- It fails immediately if more than one primary is observed.
- It also fails closed when there are too few successful samples to support a “no dual primary” assertion.
