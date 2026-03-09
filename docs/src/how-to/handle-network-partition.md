# Handle a Network Partition

This guide shows how to detect, monitor, and recover from a network partition using the runtime surfaces that pgtuskmaster already exposes.

## Goal

Determine:

- whether trust has degraded
- whether the nodes still agree on a single leader
- whether the cluster has converged again after the fault is healed

## Prerequisites

- access to the API listener on each node
- `curl` and `jq`
- optional: `pgtm`

## Step 1: Check trust on every node

Start with `GET /ha/state`.

```bash
for node in node-a node-b node-c; do
  curl --fail --silent "http://${node}:8080/ha/state" | jq -r '"\(.self_member_id) trust=\(.dcs_trust) phase=\(.ha_phase)"'
done
```

Trust values are:

- `full_quorum`
- `fail_safe`
- `not_trusted`

Operationally:

- `full_quorum` means the node has a fresh enough cluster view for normal HA behavior
- `fail_safe` means the store is reachable but freshness or coverage is not good enough for normal coordination
- `not_trusted` means the store itself is unhealthy or unreachable from that node

## Step 2: Check whether the nodes still agree on one leader

```bash
for node in node-a node-b node-c; do
  curl --fail --silent "http://${node}:8080/ha/state" | jq -r '"\(.self_member_id) leader=\(.leader // "none") phase=\(.ha_phase)"'
done
```

Look for:

- disagreement about `leader`
- more than one node reporting `ha_phase = "primary"`
- nodes stuck in `fail_safe`

To count sampled primaries:

```bash
primary_count=0
for node in node-a node-b node-c; do
  phase=$(curl --fail --silent "http://${node}:8080/ha/state" | jq -r '.ha_phase')
  if [ "${phase}" = "primary" ]; then
    primary_count=$((primary_count + 1))
  fi
done
printf 'primary_count=%s\n' "${primary_count}"
```

Treat any sustained value greater than `1` as critical. The HA observer used in tests treats multiple sampled primaries as a split-brain condition.

## Step 3: Understand what the trust gate is doing

Trust evaluation is based on:

- backing-store health
- presence of the local member record
- freshness of the local member record
- freshness of the leader record when a leader exists
- the count of fresh members when the cache contains more than one member

Freshness is checked against `ha.lease_ttl_ms`. The docker example configuration uses:

- `loop_interval_ms = 1000`
- `lease_ttl_ms = 10000`

That means trust transitions are bounded by publish cadence and lease TTL, not by a single instantaneous packet loss.

```mermaid
flowchart TD
    etcd{Store healthy?}
    self{Local member fresh?}
    leader{Leader fresh when present?}
    quorum{Enough fresh members?}
    nq[not_trusted]
    fs[fail_safe]
    fq[full_quorum]

    etcd -- no --> nq
    etcd -- yes --> self
    self -- no --> fs
    self -- yes --> leader
    leader -- no --> fs
    leader -- yes --> quorum
    quorum -- no --> fs
    quorum -- yes --> fq
```

## Step 4: Inspect the current decision and recent history

If the summary state is not enough, poll `/debug/verbose` on the affected node.

```bash
curl --fail --silent http://127.0.0.1:8080/debug/verbose | jq '{dcs,ha,process,changes,timeline}'
```

Focus on:

- `dcs.trust`
- `dcs.leader`
- `ha.phase`
- `ha.decision`
- `process.state`
- recent `changes`
- recent `timeline`

Use `?since=` for repeated polling during the incident:

```bash
last_seq=$(curl --fail --silent http://127.0.0.1:8080/debug/verbose | jq '.meta.sequence')
curl --fail --silent "http://127.0.0.1:8080/debug/verbose?since=${last_seq}" | jq .
```

## Step 5: Interpret fail-safe behavior carefully

When trust is not `FullQuorum`, the node routes into `FailSafe`.

- If local PostgreSQL is primary at that moment, the decision becomes `enter_fail_safe`
- If local PostgreSQL is not primary, the node enters `fail_safe` with `no_change`

The lowered effect plan for `enter_fail_safe` includes the safety effect for fencing, and may also release the leader lease when `release_leader_lease` is true.

Do not assume that a partitioned node should be forced back into service manually while trust is degraded. The system is intentionally conservative here.

## Step 6: Heal the fault

Restore the failed network path:

- repair connectivity to the DCS endpoints
- remove proxy, firewall, or routing blocks
- restore the node-to-node paths your environment requires

The partition tests in this repo exercise several distinct cases:

- a node isolated from etcd
- the primary isolated from etcd
- API-path isolation
- mixed network faults followed by healing

That distinction matters operationally. Diagnose which path failed before assuming the cluster needs the same recovery steps every time.

## Step 7: Wait for convergence after healing

After the fault is removed, keep polling all nodes until:

- trust returns to `full_quorum`
- the nodes agree on a single leader
- replicas settle back into expected follower behavior
- no node remains stuck in `fail_safe`

Example loop:

```bash
for node in node-a node-b node-c; do
  curl --fail --silent "http://${node}:8080/ha/state" | jq -r '"\(.self_member_id) trust=\(.dcs_trust) leader=\(.leader // "none") phase=\(.ha_phase) decision=\(.ha_decision.kind)"'
done
```

## Step 8: Verify replication again

Once HA state looks stable, verify that replicas have actually converged.

This guide cannot prescribe one exact SQL verification command for every deployment, but the goal is straightforward:

- confirm there is one stable primary
- confirm replicas are back in replica behavior
- confirm recent writes from the primary are visible after replication catches up

## Quick checklist

- `dcs_trust` is back to `full_quorum` on every node
- every node reports the same `leader`
- only one node reports `ha_phase = "primary"`
- replicas are no longer stuck in `fail_safe`
- `changes` and `timeline` no longer show ongoing trust churn

## Diagram

```mermaid
flowchart LR
    fault[Partition begins]
    trust[Trust degrades]
    fs[Node enters fail_safe]
    heal[Network healed]
    recover[Trust recovers]
    converge[Leader and replicas converge]

    fault --> trust
    trust --> fs
    fs --> heal
    heal --> recover
    recover --> converge
```
