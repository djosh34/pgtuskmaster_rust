# Observing a Failover Event

In this tutorial, you will watch a real failover unfold in a three-node cluster. Start with the cluster-wide `pgtm` view, then inspect one node more deeply only when you need the retained history.

## Prerequisites

Complete the [First HA Cluster](first-ha-cluster.md) tutorial and keep the cluster running.

Use these docs-owned operator configs while you follow the event:

- [`docs/examples/docker-cluster-node-a.toml`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/examples/docker-cluster-node-a.toml)
- [`docs/examples/docker-cluster-node-b.toml`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/examples/docker-cluster-node-b.toml)
- [`docs/examples/docker-cluster-node-c.toml`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/examples/docker-cluster-node-c.toml)

Each file mirrors the shipped docker runtime config and adds `[pgtm].api_url` for the corresponding host-mapped API port.

## Step 1: Check initial cluster health

Start from one seed node:

```bash
pgtm -c docs/examples/docker-cluster-node-a.toml status -v
```

Initially:

- trust should be healthy
- exactly one member should appear as `primary`
- the `DEBUG` column should be `available` on nodes with debug enabled
- the detail block should show `decision=no_change`

## Step 2: Capture a deeper baseline on the current leader

Pick the current leader from the status output. Then inspect that node directly by using the matching docs example config:

```bash
pgtm -c docs/examples/docker-cluster-node-a.toml debug verbose
```

If another node is leader, swap in `docker-cluster-node-b.toml` or `docker-cluster-node-c.toml`.

If you want to archive the exact payload before the fault, save the JSON form:

```bash
pgtm -c docs/examples/docker-cluster-node-a.toml --json debug verbose > before-failover.json
```

## Step 3: Introduce a primary outage

Create a failure in the current primary using the fault method available in your environment. The source-backed test harness exercises both immediate PostgreSQL stops and network faults created through proxy links.

## Step 4: Watch trust and leadership move

Use repeated cluster-wide status while the fault is active:

```bash
pgtm -c docs/examples/docker-cluster-node-a.toml status -v --watch
```

Watch for:

- trust degradation
- changes in `LEADER`
- one node moving toward candidacy or primary behavior
- `DECISION` changing from wait states into leadership actions

## Step 5: Inspect the election on the winning node

Once a likely winner appears, inspect that node directly:

```bash
pgtm -c docs/examples/docker-cluster-node-b.toml debug verbose
```

During the election window, you should see the decision evolve roughly like this:

1. `wait_for_dcs_trust`
2. `attempt_leadership`
3. `become_primary`
4. `no_change`

The retained `changes` and `timeline` sections are the fastest way to reconstruct the order of those transitions.

## Step 6: Verify the new leader

After promotion completes, run the cluster view again:

```bash
pgtm -c docs/examples/docker-cluster-node-a.toml status -v
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
pgtm -c docs/examples/docker-cluster-node-c.toml debug verbose
```

The recovering node may temporarily show `recover_replica` behavior. Once caught up, it should return to replica behavior with healthy SQL state.

## Step 9: Final verification

Run one final cluster summary:

```bash
pgtm -c docs/examples/docker-cluster-node-a.toml status -v
```

Confirm:

- all expected members are present
- exactly one primary remains
- trust is back to `full_quorum`
- the cluster has returned to steady-state decisions

## What you learned

You have now observed a complete failover cycle: detection, election, promotion, and recovery. `pgtm status -v` gives the cluster-wide picture, and `pgtm debug verbose` gives the single-node retained history behind it.
