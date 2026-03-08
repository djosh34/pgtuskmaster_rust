# Remove a Cluster Node

This guide shows how to remove a node from a running pgtuskmaster cluster. The process differs depending on whether you are gracefully scaling down or recovering from a node failure.

## Before you begin

- You have shell access to all cluster nodes
- You have the `pgtuskmasterctl` CLI installed
- You know the scope name of your cluster
- You have identified which node to remove by its member ID

## Graceful removal: stop and wait

// todo: The requested sources do not establish a built-in graceful remove-node procedure. Treat this whole sequence as tentative until aligned to the actual supported control paths.

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

5. Verify that a primary is still healthy and that no dual-primary condition exists:
   ```bash
   pgtuskmasterctl --base-url http://<remaining-node>:8000 ha check
   ```

// todo: `ha check` was not established in the requested sources.

6. Check etcd health to confirm the DCS still has quorum:
   ```bash
   etcdctl endpoint health --cluster
   ```

7. If you removed a replica, confirm replication continues from the remaining replicas:
   ```bash
   psql -h <primary-host> -c "SELECT * FROM pg_stat_replication;"
   ```

## Forced removal: after failure

If a node fails unexpectedly, pgtuskmaster automatically fences the failed primary and promotes a replica. No manual intervention is required to maintain availability.

// todo: This sentence overstates what the requested sources prove for every failure mode. Keep the claim narrower and grounded in the observed HA reaction paths.

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

pgtuskmaster does not expose a dedicated CLI command to delete member records. Member entries under `/{scope}/member/{member_id}` are informational and do not affect HA decisions once the node stops heartbeating.

// todo: The requested sources support the absence of a dedicated removal command and the member key pattern, but they do not justify the broad claim that stale member entries never affect HA decisions. Rephrase carefully.

If you must remove stale member records and understand the risks:

- The DCS key path follows the pattern: `/{scope}/member/{member_id}`
- Missing source support: The codebase provides generic `delete_path` primitives but no tested operator-facing workflow for member key deletion. Direct DCS surgery is not shown as the standard path.
- Manual deletion may be safe only after confirming the node is permanently decommissioned and the cluster has fully converged without it.

Use etcdctl only if your operational policy requires purging obsolete keys:
```bash
etcdctl del /{scope}/member/{member_id}
```

Replace `{scope}` and `{member_id}` with your actual values.

// todo: This manual deletion command is not presented as a tested supported operator workflow in the requested sources. If kept, it must remain explicitly caveated as unsupported/manual.

[diagram about graceful node removal showing the sequence: operator stops pgtuskmaster and postgres, member stops heartbeating, remaining nodes detect loss via DCS watch, HA decision engine absorbs topology change, cluster converges to new stable state without manual DCS edits]

## Verify removal completeness

1. Query the HA state from any remaining node. The removed member should not appear:
   ```bash
   pgtuskmasterctl --base-url http://<any-node>:8000 ha state
   ```

// todo: The requested sources do not establish that `ha state` output includes a full member listing suitable for this exact verification step.

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
