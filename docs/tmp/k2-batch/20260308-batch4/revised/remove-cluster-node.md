# Remove a Cluster Node

This guide shows how to remove a node from a running pgtuskmaster cluster. The process differs depending on whether you are gracefully scaling down or recovering from a node failure.

## Before you begin

- You have shell access to all cluster nodes
- You have the `pgtuskmasterctl` CLI installed
- You know the scope name of your cluster
- You have identified which node to remove by its member ID

## Graceful removal: stop and wait

**Note:** The following sequence represents common operational practice but has not been validated against official pgtuskmaster control paths. Treat as tentative until confirmed by the project's documentation.

1. SSH to the node you plan to remove.

2. Stop the pgtuskmaster process:
   ```bash
   sudo systemctl stop pgtuskmaster
   ```

3. Stop PostgreSQL on that node:
   ```bash
   sudo -u postgres pg_ctl stop -D /path/to/postgres/data -m fast
   ```

4. From a remaining node, monitor cluster state until the failed member disappears from the member list:
   ```bash
   pgtuskmasterctl --base-url http://<remaining-node>:8000 ha state
   ```

   Repeat every 10 seconds until the removed node no longer appears.

5. Verify that a primary is still healthy and that no dual-primary condition exists. The exact verification command varies by version; consult your deployment's monitoring setup.

6. Check etcd health to confirm the DCS still has quorum:
   ```bash
   etcdctl endpoint health --cluster
   ```

7. If you removed a replica, confirm replication continues from the remaining replicas:
   ```bash
   psql -h <primary-host> -c "SELECT * FROM pg_stat_replication;"
   ```

## Forced removal: after failure

If a node fails unexpectedly, pgtuskmaster's HA components monitor DCS heartbeats and can trigger automatic promotion of a standby. The exact behavior depends on failure mode and configuration.

1. Verify the cluster has converged to a new stable primary:
   ```bash
   pgtuskmasterctl --base-url http://<any-surviving-node>:8000 ha state
   ```

2. Confirm the failed node is demoted or unreachable via its API.

3. Stop the failed node if it is still running:
   ```bash
   ssh <failed-node> "sudo systemctl stop pgtuskmaster"
   ```

4. Optionally stop PostgreSQL to prevent accidental reconnection:
   ```bash
   ssh <failed-node> "sudo -u postgres pg_ctl stop -D /path/to/postgres/data -m immediate"
   ```

## Clean up DCS member records (advanced)

pgtuskmaster does not expose a dedicated CLI command to delete member records. Member entries under `/{scope}/member/{member_id}` are informational. Their impact on HA decisions once the node stops heartbeating is implementation-dependent.

**Caution:** The codebase provides generic `delete_path` primitives but no tested operator-facing workflow for member key deletion. Direct DCS surgery is not shown as the standard path.

If you must remove stale member records and understand the risks:

- The DCS key path follows the pattern: `/{scope}/member/{member_id}`
- Manual deletion may be safe only after confirming the node is permanently decommissioned and the cluster has fully converged without it.
- This workflow is not presented as a tested, supported operator path in the requested sources.

Use etcdctl only if your operational policy requires purging obsolete keys:
```bash
etcdctl del /{scope}/member/{member_id}
```

Replace `{scope}` and `{member_id}` with your actual values.

[diagram about graceful node removal showing the sequence: operator stops pgtuskmaster and postgres, member stops heartbeating, remaining nodes detect loss via DCS watch, HA decision engine absorbs topology change, cluster converges to new stable state without manual DCS edits]

## Verify removal completeness

1. Query the HA state from any remaining node. The output format and completeness of member listings may vary by version:
   ```bash
   pgtuskmasterctl --base-url http://<any-node>:8000 ha state
   ```

2. Check that PostgreSQL roles on remaining nodes show only expected primaries and replicas.

3. If you run the optional DCS cleanup, verify the key is gone:
   ```bash
   etcdctl get /{scope}/member/{member_id} --prefix
   ```
   This should return nothing.

## Next steps

- Update connection strings in applications to remove references to the decommissioned node.
- Adjust load balancer pools that pointed to the removed replica.
- If scaling down permanently, consider reducing etcd cluster size using etcd's member management tools.
