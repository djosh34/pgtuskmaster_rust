# How to check cluster health

This guide shows you how to inspect the runtime health of a PGTuskMaster cluster using the administrative CLI.

## Prerequisites

- The `pgtm` CLI is available to run.
- At least one cluster node is running with an accessible API endpoint.
- You know the base URL for the node you want to inspect. The CLI default is `http://127.0.0.1:8080`.

If you are using the local Docker HA cluster, you can first print the current endpoints and topology with:

```bash
tools/docker/cluster.sh status --env-file .env.docker.example
```

Or, when your local stack is configured through `.env.docker`:

```bash
make docker-status-cluster
```

## Inspect current HA state

Run the HA state command against a node:

```bash
pgtm status
```

For a specific node or non-default port:

```bash
pgtm --base-url http://127.0.0.1:18081 status
```

## Choose output format

The CLI supports two output formats:

- `--output json` for machine-readable output. This is the default.
- `--output text` for newline-delimited key-value output.

Examples:

```bash
pgtm --output json status
```

```bash
pgtm --output text status
```

## Interpret the output

The `status` response includes these fields:

- `cluster_name`: cluster name from runtime configuration.
- `scope`: DCS scope for the cluster.
- `self_member_id`: member ID of the node you queried.
- `leader`: current leader member ID, or `<none>` in text output when no leader is present.
- `switchover_pending`: `true` when a switchover request is currently pending, otherwise `false`.
- `switchover_to`: pending target member ID for a targeted switchover, or `<none>` in text output when the request is generic or absent.
- `member_count`: count of cached DCS members in the API response.
- `dcs_trust`: DCS trust state. The response values are `full_quorum`, `fail_safe`, and `not_trusted`.
- `ha_phase`: current HA phase such as `primary`, `replica`, `candidate_leader`, `rewinding`, `fencing`, or another transition phase.
- `ha_tick`: current HA worker tick.
- `ha_decision`: current HA decision such as `no_change`, `follow_leader(...)`, `become_primary(...)`, or another decision variant.
- `snapshot_sequence`: sequence number of the system snapshot behind the response.

## Check more than one node

A single successful request only tells you about one node. For cluster-level checks, run `status` against multiple nodes and compare:

- whether they agree on the same `leader`
- whether more than one node reports `ha_phase=primary`
- whether `member_count` is consistent
- whether any node reports degraded `dcs_trust`
- whether nodes are stuck in transition phases instead of stabilizing

The HA observer support used in tests treats multiple primaries as a split-brain condition and also treats too little successful sampling as insufficient evidence. That is a useful model for operator checks: sample repeatedly and compare nodes rather than trusting one snapshot.

## Troubleshoot connectivity

If the CLI reports a `transport error`, verify:

- the base URL is correct and reachable
- the node API is listening on the configured port
- network access from the host running `pgtm`

## Text output shape

The text formatter emits newline-delimited key-value lines in this order:

```text
cluster_name=...
scope=...
self_member_id=...
leader=...
switchover_pending=...
switchover_to=...
member_count=...
dcs_trust=...
ha_phase=...
ha_tick=...
ha_decision=...
snapshot_sequence=...
```
That shape is reconstructed from the CLI formatter source in `src/cli/output.rs`.
