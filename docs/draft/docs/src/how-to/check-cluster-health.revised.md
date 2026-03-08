```md
# How to check cluster health

This guide shows you how to inspect the runtime health of a PGTuskMaster cluster using the administrative CLI.

## Prerequisites

- The `pgtuskmasterctl` CLI binary is available to run (in PATH or via direct execution like `cargo run --bin pgtuskmasterctl`)
- At least one cluster node is running with API endpoint accessible
- You know the base URL (default: `http://127.0.0.1:8080`)

## Inspect current HA state

```bash
pgtuskmasterctl ha state
```

For a specific node or non-default port:

```bash
pgtuskmasterctl --base-url http://127.0.0.1:18081 ha state
```

## Choose output format

Add `--output text` for human-readable key-value lines, or `--output json` for machine-readable output (default: `json`).

## Interpret the output

Key fields for health assessment:

- `leader`: current leader member ID. A stable cluster shows one consistent leader across all nodes. Missing leader renders as `<none>`.
- `ha_phase`: current node role. `primary`, `replica`, or transitional states like `candidate_leader`, `rewinding`, `fencing`.
- `dcs_trust`: distributed store trust state. `full_quorum`, `fail_safe`, and `not_trusted` come directly from the API response.
- `member_count`: count of cached DCS members in the API response. Compare across all nodes.
- `ha_decision`: recent automation decision. `no_change` is steady-state; other values explain transitions.

## Identify common issues

- **No leader**: If all nodes show `leader=<none>`, the cluster has no active primary.
- **Multiple primaries**: If different nodes show different leaders, or if `primary` count exceeds one across the cluster, split-brain risk exists.
- **DCS trust degraded**: `fail_safe` or `not_trusted` means the node cannot trust cluster state; investigate network or storage.

## Troubleshoot connectivity

If you see `transport error`, verify:
- Base URL is correct and reachable
- Node API is listening on the configured port
- Network allows connections from your admin host

## Illustrative text output shape

```text
cluster_name=docker-cluster
scope=docker-cluster
self_member_id=node-a
leader=node-a
switchover_requested_by=<none>
member_count=3
dcs_trust=full_quorum
ha_phase=primary
ha_tick=45
ha_decision=no_change
snapshot_sequence=123
```

The block above shows the text output shape reconstructed from the CLI formatter. No successful runtime `ha state` response was captured during this documentation cycle.
```
