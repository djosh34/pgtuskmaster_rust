# Debug API Usage

This tutorial shows how to inspect the stable verbose debug payload as an operator-facing observation surface. The goal is not to change cluster state, but to learn how to inspect the current snapshot and how to poll incrementally with `since`.

## Prerequisites

Complete [First HA Cluster](first-ha-cluster.md) and keep the cluster running.

The shipped cluster runtime config enables the debug API, so node-a exposes it on the same HTTP listener as the normal API.

## Step 1: Start with the CLI wrapper

The normal operator entry point is:

```bash
cargo run --bin pgtm -- --base-url http://127.0.0.1:18081 debug verbose
```

That summary already tells you:

- the current sequence
- PostgreSQL role and readiness
- DCS trust and leader
- HA phase and decision
- recent `changes` and `timeline`

## Step 2: Read the raw stable payload through the CLI

When you want the full machine-readable document, keep the CLI but switch to JSON:

```bash
cargo run --bin pgtm -- --base-url http://127.0.0.1:18081 --json debug verbose | jq '{
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

## Step 3: Learn the `since` polling model

The debug worker keeps bounded in-memory history for `changes` and `timeline`. The current default limit is `300` entries for each stream.

The `since` cursor does not remove the current snapshot. It only filters the retained history arrays so that they include entries with `sequence > since`.

Capture the current sequence:

```bash
seq=$(cargo run --bin pgtm -- --base-url http://127.0.0.1:18081 --json debug verbose | jq '.meta.sequence')
echo "$seq"
```

Then ask for only newer events:

```bash
cargo run --bin pgtm -- --base-url http://127.0.0.1:18081 --json debug verbose --since "${seq}" | jq '{
  sequence: .meta.sequence,
  changes: .changes,
  timeline: .timeline
}'
```

If nothing changed, the snapshot still reports the latest `meta.sequence`, but `changes` and `timeline` may both be empty.

## Step 4: Watch only the HA and DCS signals

You usually do not need the full payload on every poll. This view narrows the response to the parts most relevant during cluster movement:

```bash
cargo run --bin pgtm -- --base-url http://127.0.0.1:18081 --json debug verbose --since 0 | jq '{
  trust: .dcs.trust,
  leader: .dcs.leader,
  phase: .ha.phase,
  decision: .ha.decision,
  changes: [.changes[] | select(.domain == "dcs" or .domain == "ha")],
  timeline: [.timeline[] | select(.category == "dcs" or .category == "ha")]
}'
```

That is a practical pattern for:

- leader transitions
- trust degradation
- fail-safe entry
- recovery progress

## Step 5: Fall back to raw HTTP only when you need the protocol directly

The CLI reads the same stable payload you can read directly:

```bash
curl --fail --silent http://127.0.0.1:18081/debug/verbose | jq .
curl --fail --silent "http://127.0.0.1:18081/debug/verbose?since=${seq}" | jq .
```

Use the raw HTTP form when you are testing the protocol itself, another client implementation, or auth/TLS behavior outside `pgtm`.

## Step 6: Understand the availability rules

The debug endpoints live on the normal API listener and follow its auth and TLS posture.

That means:

- if `debug.enabled` is `false`, the CLI will surface `debug=disabled` from `status -v`, and direct `/debug/verbose` reads return `404`
- if API auth is disabled, the debug endpoints are reachable without bearer tokens
- if role tokens are configured, debug routes are read endpoints and require read access

The shipped docker cluster config disables API auth, so the local tutorial commands do not require tokens.

## What you learned

You now know how to:

- use `pgtm debug verbose` as the normal operator entry point
- read the raw stable payload with `--json`
- use `meta.sequence` as an incremental polling cursor
- treat `since` as a history filter rather than a delta-only snapshot format
