# Verbose extra context for docs/src/how-to/handle-network-partition.md

This note is intentionally exhaustive and source-first. It is written to support an operator how-to, not to redesign the product.

## Core safety behavior from the sources

- `src/dcs/state.rs` evaluates DCS trust using:
- backing store health
- presence of the local member record
- freshness of the local member record
- freshness of the leader member record when a leader exists
- a minimum count of fresh members when more than one member is present
- Trust outcomes are:
- `FullQuorum`
- `FailSafe`
- `NotTrusted`
- `src/ha/decide.rs` uses that trust result as a top-level gate before normal HA phase logic.
- If trust is not `FullQuorum` and the node is currently primary, the decision becomes `EnterFailSafe { release_leader_lease: false }` and the phase becomes `FailSafe`.
- If trust is not `FullQuorum` and the node is not primary, the phase becomes `FailSafe` with `NoChange`.

## Why this matters operationally

- A network partition is not handled only as a transport glitch.
- It changes the node's right to make HA decisions.
- The cluster's safety response depends on whether the node can still prove enough fresh shared-state visibility, not only on whether local PostgreSQL is up.

## Freshness inputs

- `member_record_is_fresh` compares `now - updated_at` to `cache.config.ha.lease_ttl_ms`.
- The docker runtime example uses:
- `loop_interval_ms = 1000`
- `lease_ttl_ms = 10000`
- Those values are useful operator context because they bound how quickly stale-membership decisions can emerge under connectivity loss.
- A how-to can safely say that trust transitions are not instantaneous; they are mediated by the publish cadence and the lease TTL window.

## DCS worker behavior during degraded connectivity

- `src/dcs/worker.rs` attempts to publish the local member record when the store appears healthy.
- If local member publication fails, the worker logs a warning/error event and treats the store as unhealthy for that iteration.
- If watch draining or watch refresh fails, store health also degrades.
- When the local member publish does not succeed, the worker sets trust to `NotTrusted`.
- Otherwise it evaluates trust from the cache and freshness rules.
- This means partitions can surface either as:
- `NotTrusted` when store I/O itself fails
- `FailSafe` when the store is reachable but freshness/coverage is insufficient

## Partition scenarios already exercised by tests

- `tests/ha_partition_isolation.rs` exposes four end-to-end partition scenarios:
- minority isolation with no split-brain and rejoin
- primary isolation with failover and no split-brain
- API-path isolation that preserves the primary
- mixed faults with healing and convergence
- `tests/ha/support/partition.rs` shows the test fixture can:
- block etcd connectivity for a node
- isolate the API path for a node
- heal all faults
- repeatedly fetch HA state from all nodes
- observe convergence windows and no-dual-primary windows

## Observer and split-brain detection cues

- `tests/ha/support/observer.rs` records HA samples across nodes and explicitly fails when more than one primary is observed.
- The observer also counts API errors, leader changes, and fail-safe samples, and stores recent samples for diagnosis.
- This is strong evidence for operator guidance that says:
- watch all nodes, not just the currently believed leader
- treat conflicting `ha_phase = primary` reports as a critical incident
- record consecutive samples, not single snapshots, before declaring a sustained condition

## Decision variants that matter during partitions

- `EnterFailSafe`
- `FenceNode`
- `ReleaseLeaderLease`
- `StepDown`
- `FollowLeader`
- `RecoverReplica`
- These are all present in `src/ha/decision.rs`.
- A partition how-to should mention them as states or actions operators may observe, but it should not try to restate the entire decision reference page.

## Practical operator interpretation

- If a node loses enough DCS confidence, expect it to stop behaving like a normal leadership participant and move toward fail-safe behavior.
- After healing, expect a convergence period while trust returns to `FullQuorum`, a stable leader is visible again, and replicas reattach or recover as needed.
- The test fixture's explicit post-heal waits and convergence checks are evidence that recovery is a process, not a single instant.

## Useful facts to preserve in the how-to

- Distinguish etcd-path partitions from API-only isolation.
- Mention that stale-member detection is bounded by `lease_ttl_ms`.
- Recommend checking:
- `GET /ha/state` on every node
- `dcs_trust`
- `ha_phase`
- `ha_decision`
- leader agreement across nodes
- Mention that the project's own E2E partition tests are designed around "no split brain" as the primary invariant.
