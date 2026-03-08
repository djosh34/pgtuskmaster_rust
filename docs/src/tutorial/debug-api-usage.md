# Debug API Usage

This tutorial shows how to read the debug API as an operator-facing observation surface. The goal is not to change cluster state, but to learn how to inspect the current snapshot and how to poll incrementally with `since=`.

## Prerequisites

Complete [First HA Cluster](first-ha-cluster.md) and keep the cluster running.

The shipped cluster runtime config enables the debug API, so node-a exposes it on the same HTTP listener as the normal API.

## Step 1: Read the current verbose snapshot

Start with node-a:

```bash
curl --fail --silent http://127.0.0.1:18081/debug/verbose | jq .
```

This response contains the current system snapshot plus retained history. The most useful top-level sections to start with are:

- `meta`
- `config`
- `pginfo`
- `dcs`
- `process`
- `ha`
- `changes`
- `timeline`

## Step 2: Identify the fields you will poll

For cluster observation, these are the fastest fields to read:

```bash
curl --fail --silent http://127.0.0.1:18081/debug/verbose | jq '{
  sequence: .meta.sequence,
  trust: .dcs.trust,
  member_count: .dcs.member_count,
  leader: .dcs.leader,
  phase: .ha.phase,
  decision: .ha.decision
}'
```

Those values let you answer:

- whether the cluster is trusted
- which member currently leads
- which HA phase the local node is in
- which HA decision the node currently wants

## Step 3: Learn the `since=` polling model

The debug worker keeps bounded in-memory history for `changes` and `timeline`. The current default limit is `300` entries for each stream.

The `since` query parameter does not remove the current snapshot. It only filters the retained history arrays so that they include entries with `sequence > since`.

Capture the current sequence:

```bash
seq=$(curl --fail --silent http://127.0.0.1:18081/debug/verbose | jq '.meta.sequence')
echo "$seq"
```

Then ask for only newer events:

```bash
curl --fail --silent "http://127.0.0.1:18081/debug/verbose?since=${seq}" | jq '{
  sequence: .meta.sequence,
  changes: .changes,
  timeline: .timeline
}'
```

If nothing changed, the snapshot still reports the latest `meta.sequence`, but `changes` and `timeline` may both be empty.

## Step 4: Watch only the HA and DCS signals

You usually do not need the full payload on every poll. This view narrows the response to the parts most relevant during cluster movement:

```bash
curl --fail --silent "http://127.0.0.1:18081/debug/verbose?since=0" | jq '{
  trust: .dcs.trust,
  leader: .dcs.leader,
  phase: .ha.phase,
  decision: .ha.decision,
  changes: [.changes[] | select(.domain == "Dcs" or .domain == "Ha")],
  timeline: [.timeline[] | select(.category == "Dcs" or .category == "Ha")]
}'
```

That is a practical pattern for:

- leader transitions
- trust degradation
- fail-safe entry
- recovery progress

## Step 5: Build a simple incremental poll loop

Use the latest sequence number as your next cursor:

```bash
last_seq=0

while true; do
  payload=$(curl --fail --silent "http://127.0.0.1:18081/debug/verbose?since=${last_seq}")
  echo "$payload" | jq '{sequence: .meta.sequence, changes: .changes, timeline: .timeline}'
  last_seq=$(echo "$payload" | jq '.meta.sequence')
  sleep 2
done
```

This loop is useful when you want:

- the current snapshot every time
- only new history entries
- a stable cursor taken from `meta.sequence`

## Step 6: Understand the availability rules

The debug endpoints live on the normal API listener and follow its auth and TLS posture.

That means:

- if `debug.enabled` is `false`, `/debug/snapshot`, `/debug/verbose`, and `/debug/ui` return `404`
- if API auth is disabled, the debug endpoints are reachable without bearer tokens
- if role tokens are configured, debug routes are read endpoints and require read access

The shipped docker cluster config disables API auth, so the local tutorial commands do not require tokens.

## What you learned

You now know how to:

- read the current debug snapshot
- extract high-signal HA and DCS fields
- use `meta.sequence` as an incremental polling cursor
- treat `since=` as a history filter rather than a delta-only snapshot format
