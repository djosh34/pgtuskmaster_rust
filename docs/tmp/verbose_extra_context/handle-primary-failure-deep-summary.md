# Handle primary failure deep summary

This support note is only raw factual context for `docs/src/how-to/handle-primary-failure.md`.
Do not claim exact runtime log lines or per-phase `/ha/state` wire payloads unless the source files support them directly.

Primary-failure and failover control flow from `src/ha/decide.rs`:

- There is a top-level safety gate before most phase logic.
- If DCS trust is not `FullQuorum` and local PostgreSQL is still primary, HA enters `FailSafe` with `EnterFailSafe { release_leader_lease: false }`.
- If DCS trust is not `FullQuorum` and local PostgreSQL is not primary, HA still enters `FailSafe` but with `NoChange`.
- This makes degraded trust a first-class incident response branch.

Primary branch behavior when PostgreSQL becomes unreachable:

- While in `Primary`, if PostgreSQL is unreachable and the node still holds leadership, HA moves toward replica recovery by emitting `ReleaseLeaderLease { reason: PostgresUnreachable }` and transitioning to `Rewinding`.
- If PostgreSQL is unreachable but the node no longer has a self-leader record, HA still moves to `Rewinding`.
- From there it either:
  - selects `RecoverReplica::Rewind` toward another available primary
  - or remains in `Rewinding` with `NoChange` when no recovery target exists yet

Split-brain avoidance path:

- If a primary sees another active leader, HA moves to `Fencing`.
- The decision is `StepDown` with:
  - `reason = ForeignLeaderDetected`
  - `release_leader_lease = true`
  - `fence = true`
- This is the source-backed split-brain avoidance path.

Recovery and fencing behavior:

- `Rewinding` is an intermediate phase.
- Rewind success returns the node to replica-follow behavior.
- Rewind failure can escalate to `Bootstrapping` with base-backup recovery when a leader exists.
- `Bootstrapping` failure leads to `Fencing` with `FenceNode`.
- `Fencing` success releases the leader lease with reason `FencingComplete` and moves back toward trusted waiting.
- `Fencing` failure enters `FailSafe`.
- A node already in `FailSafe` returns to normal HA flow once quorum is restored.

DCS trust rules that should shape the operator doc:

- `NotTrusted` if etcd is unhealthy.
- `FailSafe` if the local member record is missing or stale.
- `FailSafe` if the leader record is missing or stale.
- `FailSafe` in multi-member clusters when fewer than two fresh member records remain.
- `FullQuorum` otherwise.
- Freshness is bounded by `ha.lease_ttl_ms`.
- The Docker cluster example sets `ha.lease_ttl_ms = 10000` and `ha.loop_interval_ms = 1000`, so those timings matter for detection speed.

Source-backed `/ha/state` structure that is safe to mention:

- `src/api/mod.rs` defines `HaStateResponse` directly.
- Fields are:
  - `cluster_name`
  - `scope`
  - `self_member_id`
  - `leader`
  - `switchover_requested_by`
  - `member_count`
  - `dcs_trust`
  - `ha_phase`
  - `ha_tick`
  - `ha_decision`
  - `snapshot_sequence`
- `DcsTrustResponse` values are:
  - `full_quorum`
  - `fail_safe`
  - `not_trusted`
- `HaPhaseResponse` values are:
  - `init`
  - `waiting_postgres_reachable`
  - `waiting_dcs_trusted`
  - `waiting_switchover_successor`
  - `replica`
  - `candidate_leader`
  - `primary`
  - `rewinding`
  - `bootstrapping`
  - `fencing`
  - `fail_safe`
- `HaDecisionResponse` carries structured variants that mirror the HA decision engine.

What is not source-backed strongly enough and must be treated carefully:

- Do not claim a concrete `/ha/state` example body for every failure phase unless the doc clearly frames it as derived from the response type definitions rather than captured runtime output.
- Do not invent exact runtime log messages for primary failure handling.
- Allowed files show log destinations and observer/test diagnostics, but not authoritative runtime message text for every failure phase.

Observer and testing facts useful for incident verification language:

- `tests/ha/support/observer.rs` treats more than one primary in API or SQL samples as split-brain and fails immediately.
- It also has helpers that recognize fail-safe by requiring all observed states to report `FailSafe`.
- This supports operator guidance about confirming:
  - exactly one primary
  - leader identity
  - trust state
  - current HA phase and decision
