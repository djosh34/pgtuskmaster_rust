# Handle Primary Failure

This guide shows how to detect, assess, and respond to a PostgreSQL primary node failure with `pgtm` as the primary operator interface.

## Prerequisites

- a running pgtuskmaster cluster with more than one node
- access to at least one reachable node API
- access to PostgreSQL on the cluster nodes
- an operator-facing config that either sets `[pgtm].api_url` or derives an operator-reachable URL from `api.listen_addr`

## Detect primary failure

Start with the cluster-wide view:

```bash
pgtm -c config.toml status -v
```

Focus on:

- `ROLE`
- `TRUST`
- `PHASE`
- `LEADER`
- `DECISION`
- warning lines about unreachable peers or disagreement

A primary failure usually surfaces as:

- no stable primary in the sampled view
- trust degrading to `fail_safe` or `not_trusted`
- one or more nodes moving through `candidate_leader`, `rewinding`, or other recovery-related phases

If the cluster view is degraded, repeat the same command from another seed config before you conclude the failure scope.

## Assess cluster state

Look for:

- exactly one node in `ROLE=primary` once the cluster settles
- all other nodes in replica-oriented behavior
- `TRUST=full_quorum` on the healthy view

The Docker cluster example uses `ha.lease_ttl_ms = 10000` and `ha.loop_interval_ms = 1000`. Those values bound member freshness checks and also define the etcd leader-lease TTL. In abrupt-node-loss cases, the old leader is invalidated when etcd expires that lease and the watched DCS cache drops `/{scope}/leader`.

If the status table is not enough, inspect the most suspicious node directly:

```bash
pgtm -c config.toml debug verbose
```

That gives you the current:

- PostgreSQL state
- DCS trust and leader cache
- HA phase and decision
- retained `changes` and `timeline`

## Respond to primary failure

No manual intervention is required for most failures. The HA decision engine automatically:

1. Detects PostgreSQL unreachability.
2. Releases the local leader lease when the primary can still step down cleanly.
3. Otherwise waits for etcd to expire the dead primary's lease-backed leader key.
4. Moves through recovery and election logic on the surviving majority.
5. Selects a recovery target from healthy members.
6. Executes rewind or base-backup recovery so the failed node can rejoin safely.

To monitor automated recovery, keep watching cluster status:

```bash
pgtm -c config.toml status -v --watch
```

### If automation stalls

If decisions remain unchanged and trust stays at `fail_safe` or `not_trusted`, resolve the underlying etcd, network, or PostgreSQL problem before expecting promotion to proceed. A healthy 2-of-3 majority should not remain stuck behind a dead primary's stale leader metadata once the old leader lease has actually expired.

## Verify recovery

Once a new primary is visible in cluster status, verify data consistency:

1. Confirm exactly one primary via SQL:

   ```bash
   for node in node-a node-b node-c; do
     psql -h "${node}" -U postgres -c "SELECT pg_is_in_recovery();"
   done
   ```

   Exactly one node should return `false`.

2. Check replication lag on replicas:

   ```bash
   psql -h <replica-ip> -U postgres -c "SELECT now() - pg_last_xact_replay_timestamp();"
   ```

3. Confirm `pgtm status -v` has returned to one primary, healthy trust, and no warning lines.

## Troubleshoot common scenarios

### All nodes show `TRUST=fail_safe`

Cause: etcd unreachable or most member records stale.  
Action: restore etcd cluster health first.

### New primary elected but replicas stay in recovery work

Cause: rewind or catch-up still running, or recovery escalated because WAL continuity was not sufficient.  
Action: keep watching `status -v` and inspect `debug verbose` on the affected replica.

### You suspect split-brain

Cause: network partition or stale observations.  
Action: run `pgtm status -v` from more than one seed config and treat any sustained multi-primary view as critical.

### Leader lease release stalls

Cause: the previous primary cannot reach etcd to revoke its own lease cleanly.  
Action: wait for lease expiry on the etcd side, then verify that `LEADER` clears from `pgtm status -v` and that the surviving majority returns to `TRUST=full_quorum`. `ha.lease_ttl_ms` bounds both member freshness and the leader-lease TTL.

## Verification checklist

- [ ] Exactly one node reports `ROLE=primary`
- [ ] All other nodes report replica-oriented behavior
- [ ] `TRUST=full_quorum` on the healthy view
- [ ] `pg_is_in_recovery()` returns `false` on one node only
- [ ] Replication lag on replicas is acceptable
- [ ] No sustained split-brain evidence appears in repeated status samples
- [ ] Application traffic can write to the new primary
