# Observing a Failover Event

In this tutorial, you will watch a real high-availability failover unfold in a three-node cluster. The focus is observation: use the debug API and HA state surfaces to see how trust, leadership, and recovery change during a primary outage.

## Prerequisites

Complete the [First HA Cluster](first-ha-cluster.md) tutorial and keep the cluster running. You will need:

- Docker and docker-compose
- curl or any HTTP client
- The cluster from the previous tutorial

## Step 1: Check initial cluster health

Start from the topology defined by `docker/compose/docker-compose.cluster.yml`: one `etcd` service and three pgtuskmaster nodes. Each node exposes an API port and a PostgreSQL port, and each node has its own data and log volumes.

## Step 2: Query the debug snapshot

The debug API exposes a `SystemSnapshot` that carries app lifecycle plus versioned `config`, `pg`, `dcs`, `process`, and `ha` state, along with a monotonic `sequence`, a `changes` list, and a `timeline` list.

The response will contain nested state information. Focus on these fields:

```text
dcs.trust        <-- FullQuorum, FailSafe, or NotTrusted
dcs.cache.leader <-- which member is currently leader
dcs.cache.members[].role    <-- Primary or Replica per node
dcs.cache.members[].sql   <-- Healthy, Unknown, etc.
ha.decision.kind        <-- no_change, wait_for_postgres, become_primary, etc.
ha.decision.detail      <-- additional parameters
```

Initially, `dcs.trust` should be `FullQuorum`, `ha.decision.kind` should be `no_change`, and exactly one member should show `role: Primary`.

## Step 3: Introduce a primary outage

Create a failure in the current primary using the fault method available in your environment. The source-backed test harness uses two families of faults:

- immediate PostgreSQL stop helpers
- connectivity faults created through TCP proxy links in front of etcd, API, and PostgreSQL endpoints

## Step 4: Watch trust degrade

Inspect the debug state again after the fault. Pay attention to how trust and member freshness change.

Trust evaluation follows these rules:
- `NotTrusted` if etcd is unhealthy
- `FailSafe` if self member or leader member is missing/stale
- `FailSafe` if fewer than two members are fresh
- `FullQuorum` otherwise

## Step 5: Observe the election decision

Repeatedly inspect the debug state and watch the HA decision evolve.

1. `wait_for_dcs_trust` while the system stabilizes
2. `attempt_leadership` as a replica tries to acquire leadership
3. `become_primary { promote: true }` for the winning node
4. `no_change` once promotion is complete

During this window, you will see `ha.decision.kind` change and `ha.decision.detail` provide parameters such as `promote=true`.

## Step 6: Verify the new leader

After promotion completes, the DCS cache should show a new leader record and updated member roles.

At this point you should see:

- a new leader in the DCS cache
- one member reporting `role: Primary`
- the previously failed member showing degraded SQL health until it recovers
- `dcs.trust` returning to `FullQuorum` once the cluster stabilizes

## Step 7: Restore the failed node

Restore the failed node using the inverse of the fault you introduced.

## Step 8: Watch node recovery

The snapshot should show the failed node rejoin as a replica. The HA decision may temporarily display `recover_replica { strategy: rewind }` or `recover_replica { strategy: base_backup }` depending on timeline divergence. Once caught up, that node appears as a replica with healthy SQL state.

## Step 9: Final verification

Query the debug snapshot one final time to confirm:

- Three members in the cache
- Exactly one primary (the new leader)
- Trust level restored to `FullQuorum`
- HA decision back to `no_change`

You have now observed a complete failover cycle: detection, election, promotion, and recovery.

## What you learned

Through direct observation, you saw how pgtuskmaster evaluates DCS trust, makes HA decisions, and orchestrates failover automatically. The debug API provides real-time visibility into the state machines that keep your cluster available.
