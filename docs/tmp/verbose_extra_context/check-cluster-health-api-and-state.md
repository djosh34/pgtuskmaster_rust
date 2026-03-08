# Verbose API and HA-state semantics for cluster health checks

This note is intentionally verbose and grounded in the requested raw files.

## What the API controller exposes from `src/api/controller.rs`

- `post_switchover(scope, store, input)` validates `requested_by` and writes a switchover request into `/{scope}/switchover`.
- `delete_switchover(scope, store)` clears the switchover request.
- `get_ha_state(snapshot)` builds the HA status payload used by the CLI and HTTP API.

The controller file itself does not enumerate every route registration. It does show the payload mapping for the HA state response. Separate code search in `src/api/worker.rs` shows these relevant HTTP routes:

- `GET /ha/state`
- `DELETE /ha/switchover`
- `POST /switchover`

For a health-check how-to, the read endpoint is `GET /ha/state`.

## Exact `HaStateResponse` fields populated by `get_ha_state`

- `cluster_name`: from `snapshot.value.config.value.cluster.name`
- `scope`: from `snapshot.value.config.value.dcs.scope`
- `self_member_id`: from `snapshot.value.config.value.cluster.member_id`
- `leader`: optional; present only when the DCS cache currently has a leader record
- `switchover_requested_by`: optional; present only when the DCS cache currently has a switchover record
- `member_count`: count of cached members in DCS
- `dcs_trust`: derived from DCS trust
- `ha_phase`: derived from the HA worker phase
- `ha_tick`: current HA worker tick
- `ha_decision`: derived from the current HA decision
- `snapshot_sequence`: system snapshot sequence number

## DCS trust states mapped by the controller

- `full_quorum`
- `fail_safe`
- `not_trusted`

These are health-significant because they say whether the node considers the distributed coordination layer fully trusted, degraded into fail-safe, or not trusted.

## HA phases mapped by the controller and defined in `src/ha/state.rs`

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

For documentation purposes, these phases should be treated as state observations, not user promises. The files requested do not define a single canonical health verdict like "healthy" or "unhealthy"; they expose underlying state that operators must interpret.

## HA decision variants exposed in the response

- `no_change`
- `wait_for_postgres`
- `wait_for_dcs_trust`
- `attempt_leadership`
- `follow_leader`
- `become_primary`
- `step_down`
- `recover_replica`
- `fence_node`
- `release_leader_lease`
- `enter_fail_safe`

These help explain why the node is in its current condition, especially when `ha_phase` alone is not enough.

## Runtime configuration details from `docker/configs/cluster/node-a/runtime.toml`

- Cluster name: `docker-cluster`
- Member id: `node-a`
- DCS scope: `docker-cluster`
- API listen address: `0.0.0.0:8080`
- HA loop interval: `1000ms`
- Lease TTL: `10000ms`
- Debug API support: `[debug] enabled = true`

Important documentation consequence:

- The requested runtime evidence command uses base URL `http://127.0.0.1:18081`.
- The node-a runtime file itself says the API listens on container port `8080`.
- A how-to should avoid asserting that `18081` is intrinsic to the application. It may be a host-mapped port from docker-compose, but that mapping is not defined by `runtime.toml`.

## What the observer test support measures from `tests/ha/support/observer.rs`

The observer code shows the kinds of cluster-health invariants the test harness cares about:

- sample count and API error count
- maximum concurrent primaries
- leader change count
- fail-safe sample count
- recent sample ring buffer with per-node state fragments
- transport errors and observation gaps

The observer treats these as evidence sources:

- API-reported `HaStateResponse` samples
- SQL role samples
- transport errors

The strongest invariant check shown in this file is the absence of dual primaries:

- if more than one node reports `primary`, the observer returns an error
- if there are too few successful samples, it also returns an error because evidence is insufficient

This is useful for operator documentation because it suggests that "cluster health" in this repo is not just "one request succeeded". It includes repeated sampling and checking for contradictory leadership.
