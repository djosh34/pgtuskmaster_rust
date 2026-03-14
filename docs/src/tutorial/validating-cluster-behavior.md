# Validating Cluster Behavior

In this tutorial we will run two validation exercises on a running three-node cluster. First we will stop a replica and confirm the primary stays stable. Then we will kill the primary and verify safe failover plus clean rejoin.

## Prerequisites

Keep the cluster from [First HA Cluster](first-ha-cluster.md) running. You need the three Docker services and the docs-provided operator configs.

The operator configs seed the three host-mapped HTTPS APIs and carry the shared TLS and token material:
- [`docs/examples/docker-cluster-node-a.toml`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/examples/docker-cluster-node-a.toml)
- [`docs/examples/docker-cluster-node-b.toml`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/examples/docker-cluster-node-b.toml)
- [`docs/examples/docker-cluster-node-c.toml`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/examples/docker-cluster-node-c.toml)

## Exercise 1: Replica outage keeps primary stable

### Step 1: Confirm initial health and identify the primary

```bash
pgtm -c docs/examples/docker-cluster-node-a.toml primary --tls --json
```

The output should show a connection view with exactly one target. Record that target's `member_id` so you can compare it after the fault.

Now check the cluster view:

```bash
pgtm -c docs/examples/docker-cluster-node-a.toml status --json
```

Notice the `warnings` array is empty and cluster `health` is healthy with all nodes showing `"role"` and `"trust"` values.

### Step 2: Record a proof row on the primary

Connect to the primary and insert a marker through the TLS-aware DSN that `pgtm` prints:

```bash
PGPASSWORD=$(cat docker/secrets/postgres-superuser-password) \
  psql "$(pgtm -c docs/examples/docker-cluster-node-a.toml primary --tls)" \
  -U postgres \
  -c "CREATE TABLE IF NOT EXISTS proof (id int); INSERT INTO proof VALUES (1);"
```

The command should return `INSERT 0 1`.

### Step 3: Stop a replica

Stop one replica. The shipped compose stack uses the service name `node-c`:

```bash
docker compose -f docker/compose.yml stop node-c
```

### Step 4: Validate primary remains unchanged

Check the primary again:

```bash
pgtm -c docs/examples/docker-cluster-node-a.toml primary --tls --json
```

The same target `member_id` should still be returned. The `warnings` array may mention the stopped replica.

Check the cluster status:

```bash
pgtm -c docs/examples/docker-cluster-node-a.toml status -v
```

Look for:
- The primary role unchanged
- `node-c` showing degraded status
- Cluster trust level still indicating quorum

Verify the proof row is still visible on the current primary:

```bash
PGPASSWORD=$(cat docker/secrets/postgres-superuser-password) \
  psql "$(pgtm -c docs/examples/docker-cluster-node-a.toml primary --tls)" \
  -U postgres \
  -c "SELECT * FROM proof;"
```

You should see the row `id = 1`.

### Step 5: Restart the replica

```bash
docker compose -f docker/compose.yml start node-c
```

### Step 6: Confirm replica rejoins and catches up

Watch the cluster status until `node-c` returns to replica behavior:

```bash
pgtm -c docs/examples/docker-cluster-node-a.toml status -v --watch
```

Press **Ctrl-C** to exit the watch. The final view should show all three nodes healthy with one primary.

## Exercise 2: Primary failover and rejoin

### Step 1: Check initial state and record a proof row

Identify the current primary:

```bash
pgtm -c docs/examples/docker-cluster-node-a.toml primary --tls --json
```

Record which node is primary. Then insert a new proof row through the TLS-aware DSN:

```bash
PGPASSWORD=$(cat docker/secrets/postgres-superuser-password) \
  psql "$(pgtm -c docs/examples/docker-cluster-node-a.toml primary --tls)" \
  -U postgres \
  -c "INSERT INTO proof VALUES (2);"
```

### Step 2: Kill the primary container

Use the current primary's `member_id` from the previous step. In the shipped compose stack, that `member_id` matches the compose service name:

```bash
docker compose -f docker/compose.yml kill <current_primary_member_id>
```

### Step 3: Watch trust degrade and new primary emerge

Monitor the cluster state:

```bash
pgtm -c docs/examples/docker-cluster-node-b.toml status -v --watch
```

Watch for:
1. Cluster trust moving to degraded states
2. Surviving nodes showing election-related status
3. One node transitioning to primary
4. Trust returning to full quorum

Press **Ctrl-C** once you see a stable new primary.

### Step 4: Verify the new primary is writable

Check which node is now primary:

```bash
pgtm -c docs/examples/docker-cluster-node-b.toml primary --tls --json
```

The output should show a different target `member_id` than the one you recorded before the failure.

Connect to the new primary and verify both proof rows:

```bash
PGPASSWORD=$(cat docker/secrets/postgres-superuser-password) \
  psql "$(pgtm -c docs/examples/docker-cluster-node-b.toml primary --tls)" \
  -U postgres \
  -c "SELECT * FROM proof ORDER BY id;"
```

You should see rows `1` and `2`, confirming data consistency.

### Step 5: Restart the old primary

```bash
docker compose -f docker/compose.yml start <old_primary_member_id>
```

### Step 6: Confirm old primary rejoins as replica

Watch the cluster status:

```bash
pgtm -c docs/examples/docker-cluster-node-b.toml status -v --watch
```

Eventually the old primary should appear as `ROLE: replica` and `SQL: Healthy`.

Check the replicas view:

```bash
pgtm -c docs/examples/docker-cluster-node-b.toml replicas --tls --json
```

The old primary should appear in the returned replica target list.

### Step 7: Final verification

Exit the watch and run a final cluster check:

```bash
pgtm -c docs/examples/docker-cluster-node-b.toml status -v
```

Confirm:
- Exactly one node with `ROLE: primary`
- Healthy reachability for the nodes that are back online
- Cluster trust at full quorum after recovery
- No lingering failover warnings once the old primary has rejoined

## What you learned

You can now validate HA behavior through operator-facing surfaces. You observed that replica outages do not trigger failover, while primary failures result in safe election and promotion. You confirmed that the old primary rejoins cleanly as a replica and that data remains consistent throughout.
