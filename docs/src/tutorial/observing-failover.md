# Observing a Failover Event

In this tutorial, you will watch a real high-availability failover unfold in a three-node cluster. The focus is observation: start with the cluster-wide CLI view, then inspect a single node more deeply when you need the retained history.

## Prerequisites

Complete the [First HA Cluster](first-ha-cluster.md) tutorial and keep the cluster running. You will need:

- Docker and docker-compose
- the cluster from the previous tutorial
- `cargo` so you can run `pgtm` from the repo checkout

## Step 1: Check initial cluster health

Start from the topology defined by `docker/compose/docker-compose.cluster.yml`: one `etcd` service and three pgtuskmaster nodes. Each node exposes an API port and a PostgreSQL port, and each node has its own data and log volumes.

Use the CLI summary from one node:

```bash
cargo run --bin pgtm -- --base-url http://127.0.0.1:18081 -v status
```

Initially:

- trust should be healthy
- exactly one member should appear as `primary`
- the `DEBUG` column should be `available` on nodes with debug enabled
- the detail block should show `decision=no_change`

## Step 2: Capture a deeper baseline on the current leader

Pick the current leader from the status output and inspect its stable verbose snapshot:

```bash
cargo run --bin pgtm -- --base-url http://127.0.0.1:18081 debug verbose
```

Focus on:

- `dcs.trust`
- `dcs.leader`
- `ha.phase`
- `ha.decision`
- recent `changes`
- recent `timeline`

If you want to archive the exact payload before the fault, use:

```bash
cargo run --bin pgtm -- --base-url http://127.0.0.1:18081 --json debug verbose > before-failover.json
```

## Step 3: Introduce a primary outage

Create a failure in the current primary using the fault method available in your environment. The source-backed test harness uses two families of faults:

- immediate PostgreSQL stop helpers
- connectivity faults created through TCP proxy links in front of etcd, API, and PostgreSQL endpoints

## Step 4: Watch trust and leadership move

Run the verbose status command repeatedly after the fault:

```bash
cargo run --bin pgtm -- --base-url http://127.0.0.1:18081 -v status
```

Watch for:

- trust degradation
- changes in `LEADER`
- one node moving toward candidacy or primary behavior
- `DECISION` changing from wait states into leadership actions

Trust evaluation follows these rules:

- `not_trusted` if etcd is unhealthy
- `fail_safe` if self or leader freshness is not good enough
- `full_quorum` otherwise

## Step 5: Inspect the election on the winning node

Once a likely winner appears, inspect that node directly:

```bash
cargo run --bin pgtm -- --base-url http://127.0.0.1:18082 debug verbose
```

During the election window, you should see the decision evolve roughly like this:

1. `wait_for_dcs_trust` while the system stabilizes
2. `attempt_leadership` as a replica tries to acquire leadership
3. `become_primary` for the winning node
4. `no_change` once promotion is complete

The retained `changes` and `timeline` sections are the fastest way to reconstruct the order of those transitions.

## Step 6: Verify the new leader

After promotion completes, run the cluster view again:

```bash
cargo run --bin pgtm -- --base-url http://127.0.0.1:18081 -v status
```

At this point you should see:

- a new leader
- one member reporting `primary`
- the previously failed member showing degraded SQL health until it recovers
- trust returning to `full_quorum` once the cluster stabilizes

## Step 7: Restore the failed node

Restore the failed node using the inverse of the fault you introduced.

## Step 8: Watch node recovery

Keep watching the cluster summary and, when needed, inspect the recovering node:

```bash
cargo run --bin pgtm -- --base-url http://127.0.0.1:18083 debug verbose
```

The recovering node may temporarily show `recover_replica` behavior. Once caught up, it should return to replica behavior with healthy SQL state.

## Step 9: Final verification

Run one final cluster summary:

```bash
cargo run --bin pgtm -- --base-url http://127.0.0.1:18081 -v status
```

Confirm:

- all expected members are present
- exactly one primary remains
- trust is back to `full_quorum`
- the cluster has returned to steady-state decisions

You have now observed a complete failover cycle: detection, election, promotion, and recovery.

## What you learned

Through direct observation, you saw how pgtuskmaster evaluates DCS trust, makes HA decisions, and orchestrates failover automatically. `pgtm status -v` gives the cluster-wide picture, and `pgtm debug verbose` gives the single-node retained history behind it.
