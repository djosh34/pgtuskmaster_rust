# Monitor Cluster Health via API

This guide shows how to poll the HTTP API and CLI to monitor a running pgtuskmaster cluster, track leader changes, detect trust degradation, and collect forensic data. No external metrics exporter is required.

## Prerequisites

- `curl` and `jq` installed on your monitoring host
- `pgtuskmasterctl` binary in PATH
// todo: unsupported default port. Loaded sources and existing docs use `http://127.0.0.1:8080` as the CLI/API default, not `8008`.
- Network access to each node's API port (default 8008)

## Step 1: Poll the minimal HA state endpoint

Query `GET /ha/state` on every node to collect a coarse health snapshot.

```bash
for node in node-a node-b node-c; do
// todo: unsupported scheme/port example. Use source-backed examples that match the documented default listener shape or clearly mark them as cluster-specific examples.
  curl -s https://$node:8008/ha/state | jq .
done
```

Expected output per node:

```json
{
  "cluster_name": "prod-cluster",
  "scope": "pgtusk/prod",
  "self_member_id": "node-a",
  "leader": "node-a",
  "switchover_pending": false,
  "member_count": 3,
  "dcs_trust": "full_quorum",
  "ha_phase": "primary",
  "ha_tick": 42,
// todo: unsupported response shape. `ha_decision` is a tagged object in the API, not a plain string.
  "ha_decision": "no_change",
  "snapshot_sequence": 120
}
```

Watch these fields:

- `leader`: Changes indicate a failover or switchover.
- `dcs_trust`: Values other than `full_quorum` signal degraded store health.
- `ha_phase`: `fail_safe` means the node entered protective mode.
- `ha_decision`: Values `step_down`, `fence_node`, `release_leader_lease`, or `enter_fail_safe` indicate dangerous transitions.

## Step 2: Fetch a full verbose diagnostic snapshot

Use `/debug/verbose` to obtain deep state from a single node.

```bash
// todo: unsupported scheme/port example. Align with source-backed API defaults or mark them as operator-specific values.
curl -s https://node-a:8008/debug/verbose | jq .
```

The response contains these top-level sections:

- `meta`: snapshot timestamp, lifecycle, and sequence number
- `config`: cluster name, member id, scope, TLS status
- `pginfo`: PostgreSQL role, SQL health, readiness, timeline
- `dcs`: trust level, member count, leader, switchover request flag
- `process`: idle or running, active job id, last outcome
- `ha`: phase, tick, decision label, detail, planned action count
- `changes`: incremental event list since startup
- `timeline`: chronological message list

Inspect `ha.decision` and `ha.planned_actions` to see what logic the node will execute next.

## Step 3: Poll incrementally to capture deltas

Append `?since=<sequence>` to `/debug/verbose` to receive only new entries.

```bash
# Store the last sequence after each poll
// todo: unsupported scheme/port example. Align with source-backed API defaults or mark them as operator-specific values.
LAST_SEQ=$(curl -s https://node-a:8008/debug/verbose | jq '.meta.sequence')

# Next poll fetches only changes since that sequence
// todo: unsupported scheme/port example. Align with source-backed API defaults or mark them as operator-specific values.
curl -s "https://node-a:8008/debug/verbose?since=$LAST_SEQ" | jq .
```

The server returns `changes` and `timeline` filtered to sequence numbers greater than your cutoff. Use `debug.history_changes` and `debug.history_timeline` to gauge how much history remains in memory (default 300 entries).

## Step 4: Alert on dual-primary detection

Poll all nodes in parallel and count primaries.

```bash
mapfile -t phases < <(
  for node in node-a node-b node-c; do
// todo: unsupported scheme/port example. Align with source-backed API defaults or mark them as operator-specific values.
    curl -s https://$node:8008/ha/state | jq -r '.ha_phase'
  done
)

primary_count=$(printf '%s\n' "${phases[@]}" | grep -c '^primary$')
if (( primary_count > 1 )); then
  echo "ALERT: $primary_count nodes report primary phase"
fi
```

Alert if `primary_count > 1`. This matches the invariant tested in `tests/ha/support/observer.rs`.

## Step 5: Use CLI when HTTP access is inconvenient

Run `pgtuskmasterctl` on any host that can reach the API.

```bash
// todo: unsupported CLI flag. The loaded CLI sources use `--base-url`, not `--endpoint`.
pgtuskmasterctl --endpoint https://node-a:8008 ha state --output json
```

The CLI output mirrors `GET /ha/state`. Wrap it in a monitoring plugin:

```bash
#!/bin/bash
// todo: unsupported CLI flag. The loaded CLI sources use `--base-url`, not `--endpoint`.
state=$(pgtuskmasterctl --endpoint "$1" ha state --output json)
phase=$(echo "$state" | jq -r '.ha_phase')
trust=$(echo "$state" | jq -r '.dcs_trust')

if [[ "$phase" == "fail_safe" ]]; then
  echo "CRITICAL: node $1 is in fail_safe"
  exit 2
fi
```

## Step 6: Retain snapshots for forensics

Archive verbose payloads after anomalies.

```bash
timestamp=$(date -u +%Y%m%dT%H%M%SZ)
// todo: unsupported scheme/port example. Align with source-backed API defaults or mark them as operator-specific values.
curl -s https://node-a:8008/debug/verbose > "/var/log/pgtusk/snapshots/node-a-$timestamp.json"
```

Rotate old files with a scheduled job. The incremental `?since=` parameter reduces bandwidth when polling continuously.

---

**Note**: At the time of writing, pgtuskmaster exposes JSON over HTTP and CLI outputs rather than native Prometheus or StatsD exporters.
