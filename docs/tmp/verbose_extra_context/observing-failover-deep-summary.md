# Observing failover deep summary

This support note is only raw factual context for `docs/src/tutorial/observing-failover.md`.
Do not claim runtime commands, signals, or HTTP payloads unless the allowed files support them directly.

Published environment and cluster shape:

- `docker/compose/docker-compose.cluster.yml` defines one `etcd` service and three pgtuskmaster nodes.
- Each node exposes API port `8080` and PostgreSQL port `5432`.
- The compose setup gives each node its own persistent data and log volumes.
- This is a real multi-node topology, not a mock.

What the E2E suite proves exists:

- `tests/ha_multi_node_failover.rs` includes real HA scenarios such as:
  - unassisted failover with SQL consistency
  - stress/unassisted failover with concurrent SQL
  - no-quorum and fail-safe coverage
- `tests/ha_partition_isolation.rs` includes:
  - minority isolation without split brain
  - primary isolation with failover and no split brain
  - API-path isolation preserving primary
  - mixed-fault healing convergence
- For tutorial purposes, the safe claim is that the repo already tests both ordinary failover and network/isolation scenarios.

What learners can observe from the debug subsystem:

- `src/debug_api/snapshot.rs` defines `SystemSnapshot` with these domains:
  - app lifecycle
  - versioned `config`
  - versioned `pg`
  - versioned `dcs`
  - versioned `process`
  - versioned `ha`
  - `generated_at`
  - monotonic `sequence`
  - `changes`
  - `timeline`
- `src/debug_api/mod.rs` shows the debug API is built from `snapshot`, `view`, and `worker` modules.
- `src/debug_api/view.rs` builds a verbose payload that includes sections for config, pginfo, dcs, process, ha, api, debug, changes, and timeline.

DCS observation terms the tutorial can use:

- `DcsState` contains `worker`, `trust`, `cache`, and `last_refresh_at`.
- `DcsTrust` values are `FullQuorum`, `FailSafe`, and `NotTrusted`.
- The member cache carries per-member:
  - role
  - SQL health
  - readiness
  - timeline
  - WAL positions
  - update timestamp
- The cache also carries leader and switchover records.
- Trust evaluation becomes:
  - `NotTrusted` if etcd is unhealthy
  - `FailSafe` if self member is missing/stale
  - `FailSafe` if leader is missing/stale
  - `FailSafe` if multi-member freshness drops below two fresh records
  - `FullQuorum` otherwise

HA decisions and detail strings that are source-backed:

- `src/ha/decision.rs` serializes these decision kinds:
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
- Decision detail data that is safe to mention:
  - `wait_for_postgres { start_requested, leader_member_id }`
  - `become_primary { promote }`
  - `step_down { reason, release_leader_lease, clear_switchover, fence }`
  - `recover_replica { strategy = rewind | base_backup | bootstrap }`
  - `release_leader_lease { reason = fencing_complete | postgres_unreachable }`
  - `enter_fail_safe { release_leader_lease }`
- The HA layer also tracks job activity classes for rewind, bootstrap/base-backup, and fencing as:
  - `Running`
  - `IdleNoOutcome`
  - `IdleSuccess`
  - `IdleFailure`

Harness facts that explain why failover is observable quickly:

- `src/test_harness/ha_e2e/startup.rs` sets:
  - `loop_interval_ms = 100`
  - `lease_ttl_ms = 2000`
  - explicit timeouts for rewind, bootstrap, and fencing
- The startup harness waits for the first node to become the bootstrap primary before the rest of the cluster is brought up.
- It also verifies that DCS bootstrap keys exist under `/{scope}/init` and `/{scope}/config` before proceeding.

Source-backed answer to the extra question about fault injection:

- The exact per-test command lines or Unix signals are not fully visible in the allowed files.
- What is directly visible:
  - task teardown aborts runtime tasks with `task.abort()`
  - PostgreSQL shutdown uses the helper `pg_ctl_stop_immediate(...)`
  - partition mode inserts `TcpProxyLink` proxies in front of etcd, API, and PostgreSQL endpoints
- Safe wording:
  - connectivity-fault scenarios are simulated through network proxies
  - process/cluster teardown uses task abort and immediate PostgreSQL stop helpers
- Unsafe wording to avoid unless another source proves it:
  - do not say specific Unix signals are used
  - do not say the exact `pg_ctl` command-line flags unless supported by a lower-level helper file
