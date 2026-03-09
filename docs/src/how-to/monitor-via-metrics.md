# Monitor via CLI Signals

This guide shows how to monitor a running cluster with `pgtm` as the primary observability surface. The CLI already exposes the operator signals you normally need: cluster health, trust, leadership, debug availability, and retained debug history.

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
- `jq` if you want to extract `meta.sequence` from saved JSON exactly as shown below
- `[debug] enabled = true` on the nodes where you want richer debug history

## Step 1: Poll `pgtm status`

Start with the cluster summary:

```bash
pgtm -c /etc/pgtuskmaster/config.toml status
```

When you want structured output for a poller or incident artifact, save the JSON form:

```bash
pgtm -c /etc/pgtuskmaster/config.toml --json status > status.json
```

The synthesized status view includes:

- `leader`
- `trust`
- `phase`
- `decision`
- peer-sampling warnings
- the seed node recorded in `queried_via`

When you want richer per-node investigation without leaving the CLI, use:

```bash
pgtm -c /etc/pgtuskmaster/config.toml status -v
```

That adds:

- `PGINFO`
- `READINESS`
- `PROCESS`
- explicit `DEBUG` availability
- per-node debug detail blocks

## Step 2: Watch the cluster continuously

For a live operator console, use watch mode:

```bash
pgtm -c /etc/pgtuskmaster/config.toml status --watch
```

For a noisier but deeper view during an incident, use:

```bash
pgtm -c /etc/pgtuskmaster/config.toml status -v --watch
```

## Step 3: Alert on leadership and trust anomalies

The fastest high-signal checks are:

- `health: degraded`
- warning lines about peer sampling failures or disagreement
- more than one sampled `ROLE=primary`
- any sampled `TRUST=fail_safe` or `TRUST=not_trusted`

If you need machine-readable checks, read the saved `status.json` artifact rather than scraping the table output.

## Step 4: Collect rich state with `pgtm debug verbose`

For one node, the stable rich inspection surface is:

```bash
pgtm -c /etc/pgtuskmaster/config.toml debug verbose
```

Use it when you need more than the coarse cluster summary:

- PostgreSQL role and readiness via `pginfo`
- DCS trust and leader cache via `dcs`
- background job activity via `process`
- decision detail and planned action count via `ha`
- recent history via `changes` and `timeline`

If you want the raw stable payload, save the JSON form:

```bash
pgtm -c /etc/pgtuskmaster/config.toml --json debug verbose > debug.json
```

## Step 5: Poll incrementally with `--since`

The CLI preserves the debug endpoint's incremental history model:

```bash
seq=$(jq -r '.meta.sequence' debug.json)
pgtm -c /etc/pgtuskmaster/config.toml --json debug verbose --since "${seq}" > debug-since.json
```

Use these fields to manage your poller:

- `meta.sequence`
- `debug.last_sequence`
- `debug.history_changes`
- `debug.history_timeline`

The current in-memory retention limit is `300` entries for `changes` and `timeline`, so incremental polling is the safest way to keep event history without missing rollovers.

## Step 6: Watch decision kinds, not only phases

`phase` tells you where the node is. `decision` tells you what it wants to do next.

Useful decision kinds to alert on:

- `enter_fail_safe`
- `fence_node`
- `release_leader_lease`
- `step_down`
- `recover_replica`

`pgtm debug verbose` surfaces those decisions directly in human output and preserves the full payload in `debug.json` when you need to inspect the exact field values.

## Step 7: Archive recent history during incidents

When an incident starts, capture the raw verbose payload for later analysis:

```bash
stamp=$(date -u +%Y%m%dT%H%M%SZ)
pgtm -c /etc/pgtuskmaster/config.toml --json debug verbose > "/var/log/pgtuskmaster/debug-${stamp}.json"
```

The retained `timeline` and `changes` sections are especially useful for reconstructing:

- when trust degraded
- when leadership changed
- when recovery started
- when fencing or fail-safe behavior appeared
