# Debug API Usage

This tutorial shows how to inspect the stable verbose debug payload through `pgtm`. The focus is observation: learn the human summary first, then switch to JSON only when you need the full stable payload or incremental polling.

## Prerequisites

Complete [First HA Cluster](first-ha-cluster.md) and keep the cluster running.

If you want to extract `meta.sequence` from saved JSON exactly as shown below, install `jq`.

The examples below use [`docs/examples/docker-cluster-node-a.toml`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/examples/docker-cluster-node-a.toml), which mirrors the shipped docker runtime config and adds an operator-facing `[pgtm].api_url` for the host-mapped API port.

## Step 1: Start with the human summary

The normal operator entry point is:

```bash
pgtm -c docs/examples/docker-cluster-node-a.toml debug verbose
```

That one command summarizes:

- the current sequence
- PostgreSQL role and readiness
- DCS trust and leader
- HA phase and decision
- recent `changes`
- recent `timeline`

## Step 2: Save the raw stable payload only when you need it

When you want the full machine-readable document, keep the CLI but switch to JSON:

```bash
pgtm -c docs/examples/docker-cluster-node-a.toml --json debug verbose > debug-node-a.json
```

The saved file is the same stable payload the API serves. Use it when you need:

- a durable incident artifact
- automation against exact field names
- the full retained `changes` and `timeline` arrays

## Step 3: Learn the `since` polling model

The debug worker keeps bounded in-memory history for `changes` and `timeline`. The current default limit is `300` entries for each stream.

The `since` cursor does not remove the current snapshot. It only filters the retained history arrays so that they include entries with `sequence > since`.

Extract the current sequence from the saved JSON, then ask for only newer events:

```bash
seq=$(jq -r '.meta.sequence' debug-node-a.json)
pgtm -c docs/examples/docker-cluster-node-a.toml --json debug verbose --since "${seq}" > debug-node-a-since.json
```

If nothing changed, the new snapshot still reports the latest `meta.sequence`, but `changes` and `timeline` may both be empty.

## Step 4: Watch the HA and DCS signals

You usually do not need every field on every poll. A practical operator loop is:

1. Start with `pgtm ... debug verbose` for the current summary.
2. Save `--json debug verbose` only when you need an artifact.
3. Re-run `--json debug verbose --since <sequence>` while you are following an incident.

That is enough to reconstruct:

- leader transitions
- trust degradation
- fail-safe entry
- recovery progress

## Step 5: Drop to raw HTTP only for protocol work

Routine operator inspection should stay in `pgtm`. If you need the underlying endpoint contract itself, use the [Debug API reference](../reference/debug-api.md).

## Step 6: Understand availability rules

The debug endpoints live on the normal API listener and follow its auth and TLS posture.

That means:

- if `debug.enabled` is `false`, `status -v` reports `debug=disabled`
- if API auth is disabled, the debug endpoints are reachable without tokens
- if role tokens are configured, debug reads need read access

The shipped docker cluster config disables API auth, so the local tutorial commands do not require separate token flags.

## What you learned

You now know how to:

- use `pgtm debug verbose` as the normal operator entry point
- save the raw stable payload with `--json`
- use `meta.sequence` as an incremental polling cursor
- treat `since` as a history filter rather than a delta-only snapshot format
