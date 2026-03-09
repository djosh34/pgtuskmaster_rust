# Monitor via API and CLI Signals

This guide shows how to monitor a running cluster with the stable CLI outputs and the same JSON payloads that sit underneath them. In this repository, the observability surfaces are API and CLI JSON rather than a dedicated Prometheus or StatsD exporter.

## Goal

Track:

- leader changes
- trust degradation
- fail-safe entry
- recovery and fencing activity
- recent state-change history

## Prerequisites

- access to each node's API listener
- `pgtm`
- `jq` if you want to filter CLI JSON
- `[debug] enabled = true` on the nodes where you want richer debug history

## Step 1: Poll `pgtm status`

Use the cluster summary first.

Human output:

```bash
pgtm -c /etc/pgtuskmaster/config.toml status
```

Machine-readable output:

```bash
pgtm -c /etc/pgtuskmaster/config.toml --json status
```

The synthesized status view includes:

- `leader`
- `dcs_trust`
- `ha_phase`
- `decision`
- peer-sampling warnings
- the seed node recorded in `queried_via`

When you want the richer per-node investigation surface without leaving the CLI, use:

```bash
pgtm -c /etc/pgtuskmaster/config.toml -v status
```

That adds:

- `PGINFO`
- `READINESS`
- `PROCESS`
- explicit `DEBUG` availability
- per-node debug detail blocks

## Step 2: Alert on leader and trust anomalies

Sample all nodes repeatedly and compare them.

### Leader disagreement

```bash
for node in node-a node-b node-c; do
  pgtm --base-url "http://${node}:8080" --json status | jq -r '
    .queried_via.member_id as $seed
    | .nodes[]
    | select(.is_self)
    | "\($seed) leader=\(.leader // "none") phase=\(.phase)"'
done
```

Alert if nodes disagree for more than a brief transition window.

### Dual-primary evidence

```bash
primary_count=0
for node in node-a node-b node-c; do
  phase=$(pgtm --base-url "http://${node}:8080" --json status | jq -r '.nodes[] | select(.is_self) | .phase')
  if [ "${phase}" = "primary" ]; then
    primary_count=$((primary_count + 1))
  fi
done
printf 'primary_count=%s\n' "${primary_count}"
```

Treat any sustained value greater than `1` as critical.

### Trust degradation

```bash
for node in node-a node-b node-c; do
  pgtm --base-url "http://${node}:8080" --json status | jq -r '
    .queried_via.member_id as $seed
    | .nodes[]
    | select(.is_self)
    | "\($seed) trust=\(.trust)"'
done
```

Alert when any node reports:

- `fail_safe`
- `not_trusted`

## Step 3: Collect rich state with `pgtm debug verbose`

For a single node, the stable rich inspection surface is:

```bash
pgtm --base-url http://127.0.0.1:8080 debug verbose
```

Use it when you need more than the coarse cluster summary:

- PostgreSQL role and readiness via `pginfo`
- DCS trust and leader cache via `dcs`
- background job activity via `process`
- decision detail and planned action count via `ha`
- recent history via `changes` and `timeline`

If you want the raw stable payload, use JSON:

```bash
pgtm --base-url http://127.0.0.1:8080 --json debug verbose | jq .
```

## Step 4: Poll incrementally with `--since`

The CLI preserves the debug endpoint's incremental history model:

```bash
last_seq=$(pgtm --base-url http://127.0.0.1:8080 --json debug verbose | jq '.meta.sequence')
pgtm --base-url http://127.0.0.1:8080 --json debug verbose --since "${last_seq}" | jq .
```

Use these fields to manage your poller:

- `meta.sequence`
- `debug.last_sequence`
- `debug.history_changes`
- `debug.history_timeline`

The current in-memory retention limit is `300` entries for `changes` and `timeline`, so incremental polling is the safest way to keep event history without missing rollovers.

## Step 5: Watch decision kinds, not only phases

`phase` tells you where the node is. `decision` tells you what it wants to do next.

Useful decision kinds to alert on:

- `enter_fail_safe`
- `fence_node`
- `release_leader_lease`
- `step_down`
- `recover_replica`

```bash
pgtm --base-url http://127.0.0.1:8080 --json debug verbose | jq '.ha'
```

## Step 6: Archive recent history during incidents

When an incident starts, capture the raw verbose payload for later analysis.

```bash
stamp=$(date -u +%Y%m%dT%H%M%SZ)
pgtm --base-url http://127.0.0.1:8080 --json debug verbose > "/var/log/pgtuskmaster/debug-${stamp}.json"
```

The `timeline` and `changes` sections are especially useful for reconstructing:

- when trust degraded
- when leadership changed
- when recovery started
- when fencing or fail-safe behavior appeared

If `status -v` reports `debug=disabled`, `auth_failed`, or `transport_failed`, alert on that separately. Missing debug data is different from "no history happened."
