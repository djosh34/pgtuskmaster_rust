# Debug Cluster Issues

This guide shows how to investigate cluster incidents with the operator CLI first and the raw debug API second. Use it when the cluster is unhealthy, a node is stuck in the wrong phase, or you need to understand why HA decisions changed.

## Goal

Answer four questions quickly:

1. What does the cluster view look like right now?
2. Does each sampled node trust that view enough to act?
3. What HA phase and decision is each node currently in?
4. What changed immediately before the incident?

## Prerequisites

- the affected nodes expose the API listener
- you can run `pgtm`
- `[debug] enabled = true` on the nodes where you want richer debug detail

## Step 1: Start with `pgtm status -v`

Use the CLI summary before you drop into raw JSON:

```bash
pgtm --base-url http://node-a:8080 -v status
```

Focus on:

- `TRUST`
- `PHASE`
- `LEADER`
- `DECISION`
- `DEBUG`
- warning lines above the table

That one command tells you whether the cluster sample is healthy, whether peers agree on one leader, and whether `/debug/verbose` was available on the sampled nodes.

The `DEBUG` column is intentionally explicit:

- `available`: the CLI fetched `/debug/verbose`
- `disabled`: the node returned `404`, which usually means `debug.enabled = false`
- `auth_failed`: the node rejected the debug read with `401` or `403`
- `not_ready`: the node returned `503`
- `transport_failed`, `decode_failed`, or `api_status_failed`: the seed status read worked, but the debug read failed for the specific reason shown in the detail block

## Step 2: Interpret trust first

The DCS worker classifies trust into three states:

- `full_quorum`
- `fail_safe`
- `not_trusted`

Read them as follows:

- `full_quorum`: the node has enough fresh DCS information to behave normally
- `fail_safe`: etcd is reachable, but freshness or coverage is not good enough for ordinary leadership behavior
- `not_trusted`: etcd itself is unhealthy or unreachable, so the node does not trust DCS state at all

If trust is degraded, fix that before you expect normal promotions or switchovers.

## Step 3: Inspect one node with `pgtm debug verbose`

When the table is not enough, inspect the affected node directly:

```bash
pgtm --base-url http://node-a:8080 debug verbose
```

The default output summarizes the high-signal sections:

- `pginfo`
- `dcs`
- `ha`
- `process`
- recent `changes`
- recent `timeline`

Use it to answer:

- whether PostgreSQL is reachable and ready
- which leader the node currently believes in
- which HA decision is active
- whether the process worker is still running a job or has already failed one

If you need the full stable payload for automation or a saved incident artifact, switch to JSON:

```bash
pgtm --base-url http://node-a:8080 --json debug verbose > debug-node-a.json
```

## Step 4: Poll incrementally with `--since`

Use the retained sequence cursor when you are following an incident live:

```bash
seq=$(pgtm --base-url http://node-a:8080 --json debug verbose | jq '.meta.sequence')
pgtm --base-url http://node-a:8080 --json debug verbose --since "${seq}" | jq '{
  sequence: .meta.sequence,
  changes: .changes,
  timeline: .timeline
}'
```

`--since` filters only `changes` and `timeline`. The other sections still describe the current snapshot.

## Step 5: Fall back to raw HTTP only when you need protocol-level debugging

The CLI reads the same stable debug endpoint that you can read directly:

```bash
curl --fail --silent http://node-a:8080/debug/verbose | jq .
curl --fail --silent "http://node-a:8080/debug/verbose?since=42" | jq .
```

Use the raw endpoint when you are debugging the HTTP contract itself, reproducing auth/TLS behavior with another client, or comparing CLI output to the underlying payload.

## Common Investigation Patterns

### The node never leaves a waiting phase

Check these in order:

1. `pginfo.sql`
2. `dcs.trust`
3. `ha.decision`
4. `process.state` and `last_outcome`

That sequence tells you whether the blocker is database reachability, DCS trust, decision logic, or an already-failed background job.

### The cluster is stuck in fail-safe behavior

Check:

- etcd health
- member freshness
- whether the local node is present in the DCS cache
- whether the recorded leader is stale

`fail_safe` means the node still has some DCS visibility, but not enough to behave as if the cluster view were fully trustworthy.

### You suspect a split-brain or dual-primary window

Run `pgtm status -v` from more than one seed node and compare the answers. Treat any sustained view with more than one sampled primary as critical.

At the same time, inspect `timeline` and `changes` on the most suspicious node to see whether the cluster was stepping down, fencing, or recovering around the same moment.

### A switchover request seems stuck

Check:

- `switchover_pending`
- `switchover_to`
- `dcs_trust`
- `ha_phase`
- `ha_decision`
- `leader`

If `switchover_pending=true` but trust is degraded, fix trust first. The runtime will not complete the switchover path safely until it has a healthy cluster view.

## Next Step

Once you understand the immediate failure mode, move to the relevant operator action:

- trust problem: restore etcd reachability or freshness
- PostgreSQL problem: inspect PostgreSQL process status and logs
- leadership transition problem: keep watching `pgtm status -v` and `pgtm debug verbose` until the node converges or fails cleanly
