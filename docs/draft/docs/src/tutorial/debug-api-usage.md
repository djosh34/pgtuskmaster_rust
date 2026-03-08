# Using the Debug API to Observe Cluster Behavior

We will connect to the debug API to watch a running cluster's state transitions, track failover events, and learn the incremental polling pattern.

## What We Will Build

By the end of this tutorial, we will have:
- A running three-node cluster with debug mode enabled
- Live observations of the HA decision engine
- A recorded timeline of state changes
- A reusable monitoring script using incremental polling

## Prerequisites

We need Docker and Docker Compose installed. We will use the provided cluster configuration which already has debug mode enabled.

## Start the Cluster

// todo: The working directory and compose invocation here are not grounded in the requested files. The requested runtime file establishes debug enablement, but this exact `cd docker/compose` flow and log sentinel need verification.

First, we start the cluster with debug endpoints available:

```bash
cd docker/compose
docker-compose -f docker-compose.cluster.yml up -d
docker-compose -f docker-compose.cluster.yml logs -f node-a
```

Wait for the log line `app=Running` to appear. This means node-a has entered the Running lifecycle and the debug API is ready.

// todo: The requested sources do not verify that `app=Running` appears as a log line or that it should be used as the operator-facing readiness signal.

## Make the First Debug API Call

We query node-a's debug endpoint to see the current snapshot:

```bash
curl -s http://localhost:18081/debug/verbose | jq '.'
```

The response is a JSON object with five top-level fields:

// todo: This field list is inaccurate as written. The requested sources indicate the verbose payload includes more top-level state sections than the five listed here.

- `meta` - Snapshot metadata
- `changes` - Array of change events
- `timeline` - Chronological event feed
- `dcs`, `ha`, `config`, `pg`, `process` - Current domain states

For our first call, `changes` and `timeline` contain the initial six startup events (app, config, pginfo, dcs, process, ha).

[diagram about debug API snapshot structure, showing how `meta.sequence` advances and how `changes` and `timeline` accumulate while top-level state remains current]

## Observe the Sequence Counter

We extract the current sequence number:

```bash
curl -s http://localhost:18081/debug/verbose | jq '.meta.sequence'
```

This returns an integer like `6`. The sequence increments by one for each change event recorded.

## Watch the Cluster Elect a Leader

In a second terminal, watch the cluster form:

```bash
watch -n 1 'curl -s http://localhost:18081/debug/verbose | jq "{sequence: .meta.sequence, leader: .dcs.cache.leader.member_id, phase: .ha.phase, decision: .ha.decision}"'
```

Within 10-30 seconds, we will see `phase` change from `Init` to `WaitingPostgresReachable` to `Primary`, and `dcs.cache.leader.member_id` will show `node-a`.

// todo: The exact convergence timing and path above are not verified in the requested files.

Stop the watch with Ctrl+C.

## Poll Incrementally Using `since`

The debug API supports incremental polling by passing `?since=N`. This returns a full snapshot but only includes `changes` and `timeline` entries where `sequence > since`.

First, capture the baseline sequence:

```bash
SEQUENCE=$(curl -s http://localhost:18081/debug/verbose | jq '.meta.sequence')
echo $SEQUENCE
```

Now request only newer entries:

```bash
curl -s "http://localhost:18081/debug/verbose?since=$SEQUENCE" | jq '{changes: .changes, timeline: .timeline}'
```

Initially this returns empty arrays because no new events have occurred. Wait for cluster activity, then repeat the command to see only the delta.

[diagram about incremental polling pattern, showing first request with since=0, client tracking sequence, and subsequent requests with since=previous_sequence]

## Track a Specific Timeline Event

Let's find timeline entries related to HA phase changes:

```bash
curl -s "http://localhost:18081/debug/verbose?since=0" | \
jq '.timeline[] | select(.domain == "Ha") | {at: .at, message: .message}'
```

This produces output like:

// todo: The concrete example event below is not source-backed and may combine states that do not co-occur in a real cluster startup path.

```json
{
  "at": 1234567890,
  "message": "ha worker=Starting phase=Init decision=EnterFailSafe detail=release_leader_lease=false"
}
```

Notice the `phase=Init` field. As the cluster evolves, we will see `phase=Primary` and `decision=AttemptLeadership`.

## Watch Membership Changes

// todo: Adding `node-b` to an already-defined three-node cluster compose file is not established by the requested sources, and the exact member-count transition described below needs verification.

We add a second node to see the debug API record membership changes:

```bash
docker-compose -f docker-compose.cluster.yml up -d node-b
```

Watch the timeline for `Dcs` domain events:

```bash
curl -s "http://localhost:18081/debug/verbose" | jq '.changes[] | select(.domain == "Dcs")'
```

We should see a change event with `summary` showing `members=1` becoming `members=2`.

## Monitor the Decision Engine Over Time

Create a monitoring script that uses incremental polling:

```bash
cat > monitor.sh << 'EOF'
#!/bin/bash
LAST_SINCE=0

while true; do
  RESPONSE=$(curl -s "http://localhost:18081/debug/verbose?since=$LAST_SINCE")
  
  # Extract current sequence
  CURRENT_SEQUENCE=$(echo "$RESPONSE" | jq '.meta.sequence')
  
  # Print new timeline entries
  COUNT=$(echo "$RESPONSE" | jq '.timeline | length')
  if [ "$COUNT" -gt 0 ]; then
    echo "=== New events ($COUNT since $LAST_SINCE) ==="
    echo "$RESPONSE" | jq '.timeline[] | {sequence: .sequence, domain: .domain, message: .message}'
  fi
  
  # Update since for next poll
  LAST_SINCE=$CURRENT_SEQUENCE
  
  sleep 2
done
EOF

chmod +x monitor.sh
./monitor.sh
```

Let this run while we trigger a switchover in another terminal.

## Observe a Planned Switchover

In a new terminal, request a switchover from node-a to node-b:

// todo: The endpoint path `/api/switchover` is not supported by the requested sources. Switchover handling/path details need to be rederived from the actual API implementation or existing docs before publication.

```bash
curl -X POST http://localhost:18081/switchover \
  -H "Content-Type: application/json" \
  -d '{}'
```

The monitoring script will print timeline events showing:

// todo: The exact event sequence below is illustrative but not verified from the requested sources as written.

- `Dcs` domain: `switchover=Some(...)`
- `Ha` domain: `decision=StepDown` on node-a
- `Ha` domain: `decision=BecomePrimary` on node-b
- `Dcs` domain: leader change and `switchover=None`

Stop the monitoring script with Ctrl+C.

## Check History Retention

The debug API keeps the last 300 events in memory per stream. If we poll very frequently, older sequences may age out. Verify this by noting the oldest sequence in a long-running cluster:

```bash
curl -s "http://localhost:18081/debug/verbose" | \
jq '.timeline | min_by(.sequence) | .sequence'
```

If this value increases over time, it indicates old entries are being trimmed.

## Clean Up

Stop the cluster:

```bash
docker-compose -f docker-compose.cluster.yml down
rm monitor.sh
```

## Review

We have learned to:
- Start a cluster with debug mode enabled
- Read the current snapshot and extract `meta.sequence`
- Poll incrementally using `since=` to avoid transferring full history
- Watch timeline events for specific domains (Ha, Dcs)
- Monitor switchover events in real time

The debug API provides a read-only observation surface for state transitions, failover timelines, and trust changes. When used with incremental polling, it becomes a lightweight monitoring tool for tracking cluster behavior.

---

[mermaid diagram: sequence of client polling pattern]
// todo: Convert this placeholder into a valid mermaid block only if it matches the actual published payload shape and polling semantics.
sequenceDiagram
    participant Client
    participant "Debug API"
    participant "Cluster state"
    
    Client->>"Debug API": GET /debug/verbose
    "Debug API"->>"Cluster state": snapshot all domains
    "Cluster state"->>"Debug API": (snapshot, events)
    "Debug API"->>Client: {meta: {sequence: 6}, changes: [...], timeline: [...]}
    
    Note over Client: client tracks sequence = 6
    
    Client->>"Debug API": GET /debug/verbose?since=6
    "Debug API"->>"Debug API": filter events > since
    "Debug API"->>Client: {meta: {sequence: 8}, changes: [7,8], timeline: [7,8]}
    
    Note over Client: client tracks sequence = 8, only sees new events
[mermaid end]
