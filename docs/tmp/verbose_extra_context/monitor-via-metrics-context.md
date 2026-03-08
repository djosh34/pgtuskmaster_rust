# Verbose extra context for docs/src/how-to/monitor-via-metrics.md

This note is intentionally exhaustive and source-first. It is for a how-to page, so it focuses on what an operator can actually poll and infer from the currently implemented surfaces.

## Important scope fact

- I do not see a dedicated metrics exporter module or any Prometheus/StatsD-specific surface in the requested files.
- The evidence in the requested files points to JSON-oriented observability through the API and CLI, not to a native scrape exporter.
- Treat that as an evidence-based inference from the loaded files, not as a universal claim about every possible future build.

## Observable surfaces relevant to monitoring

- `GET /ha/state` is the smallest stable machine-readable HA summary.
- `/debug/verbose` is the richest structured diagnostic surface.
- `/debug/snapshot` is a debug snapshot surface, but `/debug/verbose` is the structured endpoint described with a stable JSON schema.
- `/debug/ui` is an HTML viewer over `/debug/verbose`, not a separate metrics source.
- `pgtuskmasterctl ... ha state --output json` is relevant because the CLI is an API client and can be used by shell-based monitoring hooks where direct HTTP access is inconvenient.

## What `GET /ha/state` gives operators

- Cluster name and scope
- Self member id
- Current leader id or `null`
- Pending switchover requester or `null`
- Member count
- DCS trust
- HA phase
- HA tick
- HA decision
- Snapshot sequence

## Monitoring signals available from `GET /ha/state`

- Leader changes: compare the `leader` field over time.
- Trust degradation: alert when `dcs_trust` moves away from `full_quorum`.
- Fail-safe entry: alert when `ha_phase` becomes `fail_safe`.
- Unexpected fencing or demotion behavior: alert on `ha_decision` variants like `fence_node`, `release_leader_lease`, `step_down`, or `enter_fail_safe`.
- Topology shrinkage: watch `member_count`.
- Pending operator action: watch `switchover_requested_by`.

## What `/debug/verbose` adds

- `meta` provides generation timestamp, channel timestamp/version, lifecycle, and sequence.
- `config` exposes cluster/member/scope plus `debug_enabled` and `tls_enabled`.
- `pginfo` exposes role-ish information via `variant`, SQL health, readiness, optional timeline, and a compact summary string.
- `dcs` exposes trust, member count, current leader, and whether a switchover request exists.
- `process` exposes whether the process worker is idle or running, active job id, and last outcome.
- `ha` exposes phase, decision label, optional decision detail, and the count of planned actions.
- `changes` is an incremental event list.
- `timeline` is an incremental message timeline.

## Incremental polling model

- `/debug/verbose` accepts `?since=<sequence>`.
- The server returns only `changes` and `timeline` entries with a sequence greater than the provided cutoff.
- The payload also returns `debug.last_sequence`, so pollers can store the latest sequence and request only deltas next time.
- `debug.history_changes` and `debug.history_timeline` expose the retained history depth in memory.
- `src/debug_api/worker.rs` currently bounds retained history to 300 entries by default.

## High-value alert ideas grounded in the sources

- Alert when `dcs.trust` is not healthy enough for normal operation:
- `full_quorum` is the normal case.
- `fail_safe` means the store is reachable but freshness/coverage is insufficient.
- `not_trusted` means the backing store itself is unhealthy or writes failed.
- Alert when the whole cluster or a node remains in `fail_safe` unexpectedly.
- Alert on repeated `ha.decision` values that imply churn or danger:
- `enter_fail_safe`
- `release_leader_lease`
- `step_down`
- `recover_replica`
- `fence_node`
- Alert when `process.state` stays `Running` with the same job id for too long if your environment expects bounded completion for rewind/bootstrap/fencing jobs.
- Alert when `pginfo.variant` or readiness unexpectedly changes on a leader node.

## Observer logic already embodied in tests

- `tests/ha/support/observer.rs` is valuable because it shows what the project itself considers operational invariants.
- The observer samples `HaStateResponse` values over time and tracks:
- sample count
- API error count
- maximum concurrent primaries
- leader change count
- fail-safe sample count
- recent samples
- It explicitly errors when more than one primary is observed in the same sample window.
- That means a monitoring guide can legitimately recommend alerts around:
- more than one node reporting `ha_phase = primary`
- repeated observation gaps
- elevated API error counts during critical events
- unusual leader churn

## Partition test cues that are useful for monitoring

- `tests/ha/support/partition.rs` runs scenarios around etcd isolation, API path isolation, healing, and convergence.
- Those tests record timelines and repeatedly query cluster HA state.
- They wait for conditions like stable primary convergence and no-dual-primary observation windows.
- This suggests practical monitoring patterns:
- poll all nodes, not just one
- detect disagreements across nodes, not just local state changes
- keep a short ring of recent observations for incident diagnosis

## Built-in metric format answer

- Evidence from the requested files suggests there is no built-in Prometheus or StatsD exporter today.
- The implemented machine-readable sources are JSON over HTTP and JSON output from the CLI.
- If the page needs to say this plainly, phrase it as:
- "At the time of writing, the implemented observability surfaces in this repo are JSON API/CLI outputs rather than a dedicated metrics exporter."

## Practical operator framing for the how-to

- This page should tell the operator how to:
- poll `/ha/state` on every node for coarse health
- use `/debug/verbose?since=` when deeper event history is needed
- treat trust transitions and dual-primary evidence as high-severity alerts
- retain recent raw JSON snapshots or reduced summaries for incident forensics
