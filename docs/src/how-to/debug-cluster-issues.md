# Debug Cluster Issues

This guide shows how to investigate cluster incidents with the observation surfaces the runtime already exposes.

Use it when the cluster is unhealthy, a node is stuck in the wrong phase, or you need to understand why HA decisions changed.

## Goal

Answer four questions quickly:

1. What does the node think the cluster looks like?
2. Does it trust that cluster view enough to act?
3. What HA phase and decision is it currently in?
4. What changed immediately before the incident?

## Prerequisites

- the node has `[debug] enabled = true`
- you can reach the node's API port
- you have `curl` or another HTTP client

## Step 1: Read `/ha/state` First

Start with the lightweight cluster summary:

```bash
curl --fail --silent http://node-a:8080/ha/state
```

Focus on these fields first:

- `leader`
- `member_count`
- `dcs_trust`
- `ha_phase`
- `ha_decision`
- `switchover_requested_by`

These values tell you whether the node sees a healthy leader, whether DCS is trusted, and whether it is waiting, following, promoting, rewinding, fencing, or in fail-safe behavior.

## Step 2: Interpret `dcs_trust`

The DCS worker classifies trust into three states:

- `full_quorum`
- `fail_safe`
- `not_trusted`

Read them as follows.

### `full_quorum`

The node has enough fresh DCS information to behave normally.

### `fail_safe`

etcd is reachable, but the node does not have a safe-enough cluster view for ordinary leadership behavior. In source terms, this can happen when:

- the local member record is missing
- the local or leader record is stale
- there are too few fresh members for the current cluster view

### `not_trusted`

etcd itself is unhealthy or unreachable, so the node does not trust DCS state at all.

## Step 3: Inspect `/debug/verbose`

Use the verbose endpoint when `/ha/state` is not enough:

```bash
curl --fail --silent http://node-a:8080/debug/verbose
```

For repeated polling during an incident, narrow the response with a sequence cursor:

```bash
curl --fail --silent "http://node-a:8080/debug/verbose?since=42"
```

The verbose payload is organized into sections that map directly to the runtime:

- `config`
- `pginfo`
- `dcs`
- `process`
- `ha`
- `api`
- `debug`
- `changes`
- `timeline`

## Step 4: Read The Most Useful Sections

### `pginfo`

Use this section to answer whether PostgreSQL is actually reachable and what role it appears to be in.

Important fields:

- `variant`
- `sql`
- `readiness`
- `timeline`
- `summary`

If `sql` is unhealthy or unknown, start treating the incident as a PostgreSQL reachability or readiness problem, not only as an HA logic problem.

### `dcs`

Use this section to confirm the cluster view:

- `trust`
- `member_count`
- `leader`
- `has_switchover_request`

If `trust` is degraded, fix that before expecting normal promotions or switchovers.

### `ha`

This section explains what the decision engine is doing:

- `phase`
- `tick`
- `decision`
- `decision_detail`
- `planned_actions`

The decision labels come from the HA decision engine and include values such as:

- `wait_for_postgres`
- `wait_for_dcs_trust`
- `attempt_leadership`
- `follow_leader`
- `become_primary`
- `step_down`
- `recover_replica`
- `fence_node`
- `release_leader_lease`
- `enter_fail_safe`

### `process`

When the runtime is actively doing work, this section tells you whether the process worker is currently running a job or whether the last job ended in success, failure, or timeout.

## Step 5: Use `timeline` And `changes` To Reconstruct The Incident

The `timeline` section is the most useful place to read the story of the incident in order.

Use it to answer:

- when trust degraded
- when PostgreSQL became reachable or unreachable
- when a promotion or step-down decision appeared
- whether a fencing or recovery path started

The `changes` section is complementary: it shows which domain changed version and when.

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

Query `/ha/state` on more than one node and compare the answers. The HA test observer treats more than one sampled primary as a split-brain signal, and that is the right operational mindset as well.

At the same time, inspect `timeline` and `changes` to see whether the cluster was stepping down, fencing, or recovering around the same moment.

### A switchover request seems stuck

Check:

- `switchover_requested_by`
- `dcs_trust`
- `ha_phase`
- `ha_decision`
- `leader`

If DCS trust is degraded, fix that before expecting the switchover to complete normally.

## Next Step

Once you understand the immediate failure mode, move to the relevant operator action:

- trust problem: restore etcd reachability or freshness
- PostgreSQL problem: inspect PostgreSQL process status and logs
- leadership transition problem: keep sampling `/ha/state` and `debug/verbose` while the node converges or fails
