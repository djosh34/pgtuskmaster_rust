# Validating Cluster Behavior

// todo: The draft scope is good, but several concrete commands and expected outputs are not source-backed yet. Please correct facts without changing the overall tutorial shape.
// todo: `pgtm primary --tls --json` returns a connection view with `targets`, not a status table and not per-member `"role": "primary"` fields. Any expected-output text that claims otherwise should be rewritten against the real connection-view shape.
// todo: `pgtm status --json` exposes cluster `health`, `warnings`, and per-node `role` / `trust` strings. The draft currently mixes in unsupported field names like `sql_status`, a cluster-level `trust`, and old phase names such as `waiting_postgres_reachable` and `candidate_leader`.
// todo: The docs source set used here does not prove the literal Docker container names `pgtuskmaster_node-a_1` / `_node-b_1` / `_node-c_1`. Either replace them with source-backed names or describe the stop/kill/start step in a way that does not hard-code unsupported container IDs.
// todo: `status --watch` exits via Ctrl-C in the current CLI implementation; the draft's `Press q to exit` instruction is not source-backed.
// todo: The forwarded PostgreSQL ports used in the `psql` examples are not proven by the gathered source set. Replace them with source-backed ports or instruct the reader to use the actual mapped port from their running Docker cluster.
// todo: Exercise 2 currently hard-codes `docker kill pgtuskmaster_node-a_1` for the primary-failure step. The tutorial needs source-backed branching based on whichever node `pgtm primary` resolved, or it should reuse the current primary-failure how-to wording instead of hard-coding one node.
// todo: The "watch for trust degrade / return to FullQuorum" narrative is directionally right, but the exact field names and transition labels should be aligned with current `pgtm status -v` output and the existing `observing-failover` tutorial, not with removed older terminology.

In this tutorial we will run two validation exercises on a running three-node cluster. First we will stop a replica and confirm the primary stays stable. Then we will kill the primary and verify safe failover plus clean rejoin.

## Prerequisites

Keep the cluster from [First HA Cluster](first-ha-cluster.md) running. You need the three Docker containers `node-a`, `node-b`, and `node-c` and the docs-provided operator configs.

The operator configs add `[pgtm].api_url` for operator-reachable ports:
- [`docs/examples/docker-cluster-node-a.toml`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/examples/docker-cluster-node-a.toml)
- [`docs/examples/docker-cluster-node-b.toml`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/examples/docker-cluster-node-b.toml)
- [`docs/examples/docker-cluster-node-c.toml`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/examples/docker-cluster-node-c.toml)

## Exercise 1: Replica outage keeps primary stable

### Step 1: Confirm initial health and identify the primary

```bash
pgtm -c docs/examples/docker-cluster-node-a.toml primary --tls --json
```

The output should show exactly one node with `"role": "primary"` and all members reporting `sql_status: "Healthy"`.

Now check the cluster view:

```bash
pgtm -c docs/examples/docker-cluster-node-a.toml status --json
```

Notice the `warnings` array is empty and `trust` is `FullQuorum`.

### Step 2: Record a proof row on the primary

Connect to the primary and insert a marker:

```bash
psql -h localhost -p 15432 -U postgres -c "CREATE TABLE IF NOT EXISTS proof (id int); INSERT INTO proof VALUES (1);"
```

The command should return `INSERT 0 1`.

### Step 3: Stop a replica

Stop `node-c`:

```bash
docker stop pgtuskmaster_node-c_1
```

### Step 4: Validate primary remains unchanged

Check the primary again:

```bash
pgtm -c docs/examples/docker-cluster-node-a.toml primary --tls --json
```

The same node should still report `"role": "primary"`. The `warnings` array may mention the stopped replica.

Check the cluster status:

```bash
pgtm -c docs/examples/docker-cluster-node-a.toml status -v
```

Look for:
- The primary role unchanged
- `node-c` showing `PHASE: waiting_postgres_reachable`
- `TRUST` still `FullQuorum`

Verify the proof row is still visible:

```bash
psql -h localhost -p 15432 -U postgres -c "SELECT * FROM proof;"
```

You should see the row `id = 1`.

### Step 5: Restart the replica

```bash
docker start pgtuskmaster_node-c_1
```

### Step 6: Confirm replica rejoins and catches up

Watch the cluster status until `node-c` returns to replica behavior:

```bash
pgtm -c docs/examples/docker-cluster-node-a.toml status -v --watch
```

Press `q` to exit the watch. The final view should show all three nodes healthy with one primary.

## Exercise 2: Primary failover and rejoin

### Step 1: Check initial state and record a proof row

Identify the current primary:

```bash
pgtm -c docs/examples/docker-cluster-node-a.toml primary --tls --json
```

Record which node is primary. Then insert a new proof row:

```bash
psql -h localhost -p 15432 -U postgres -c "INSERT INTO proof VALUES (2);"
```

### Step 2: Kill the primary container

If the primary is `node-a`, run:

```bash
docker kill pgtuskmaster_node-a_1
```

### Step 3: Watch trust degrade and new primary emerge

Monitor the cluster state:

```bash
pgtm -c docs/examples/docker-cluster-node-b.toml status -v --watch
```

Watch for:
1. `TRUST` moving to `FailSafe`
2. `PHASE` on surviving nodes showing `candidate_leader`
3. One node transitioning to `primary`
4. `TRUST` returning to `FullQuorum`

Press `q` once you see a stable new primary.

### Step 4: Verify the new primary is writable

Check which node is now primary:

```bash
pgtm -c docs/examples/docker-cluster-node-b.toml primary --tls --json
```

The output should show a different node with `"role": "primary"`.

Connect to the new primary's forwarded port (15433 for node-b or 15434 for node-c) and verify both proof rows:

```bash
psql -h localhost -p 15433 -U postgres -c "SELECT * FROM proof ORDER BY id;"
```

You should see rows `1` and `2`, confirming data consistency.

### Step 5: Restart the old primary

```bash
docker start pgtuskmaster_node-a_1
```

### Step 6: Confirm old primary rejoins as replica

Watch the cluster status:

```bash
pgtm -c docs/examples/docker-cluster-node-b.toml status -v --watch
```

Eventually `node-a` should appear as `ROLE: replica` and `SQL: Healthy`.

Check the replicas view:

```bash
pgtm -c docs/examples/docker-cluster-node-b.toml replicas --tls --json
```

`node-a` should be listed with `"role": "replica"` and `sql_status: "Healthy"`.

### Step 7: Final verification

Exit the watch and run a final cluster check:

```bash
pgtm -c docs/examples/docker-cluster-node-b.toml status -v
```

Confirm:
- Exactly one node with `ROLE: primary`
- All nodes `SQL: Healthy`
- `TRUST: FullQuorum`
- `warnings` array empty

## What you learned

You can now validate HA behavior through operator-facing surfaces. You observed that replica outages do not trigger failover, while primary failures result in safe election and promotion. You confirmed that the old primary rejoins cleanly as a replica and that data remains consistent throughout.
