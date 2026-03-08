# Handle Primary Failure

This guide shows you how to detect, assess, and respond to a PostgreSQL primary node failure in a pgtuskmaster cluster.

## Prerequisites

- A running pgtuskmaster cluster with more than one node
- Access to the HTTP API
- Access to PostgreSQL on the cluster nodes

## Detect Primary Failure

Monitor primary health through `/ha/state`.

Parse these fields from the response:

- `ha_phase`: Current operational phase (string)
- `dcs_trust`: Trust level state (`full_quorum`, `fail_safe`, or `not_trusted`)
- `leader`: Member ID of the current leader (optional string)

A primary failure can surface through HA phase and trust changes such as:

```json
{
  "ha_phase": "rewinding",
  "dcs_trust": "fail_safe",
  "leader": null
}
```

If etcd is unreachable, `dcs_trust` becomes `not_trusted`. If member records are stale, it becomes `fail_safe`. These states are part of the safety model that prevents split-brain.

## Assess Cluster State

Query all reachable nodes to build a complete picture.

Look for:

- **Exactly one** node in `primary` phase (or `rewinding` if recently failed)
- **All other** nodes in `replica`, `candidate_leader`, or `rewinding`
- `dcs_trust: "full_quorum"` on the majority of nodes

Use the observer pattern from test tooling to programmatically assert no split-brain:

- Collect `/ha/state` responses from all nodes
- Fail if more than one node reports `ha_phase: "primary"` at any sample
- The observer requires a minimum sample count to assert safety

The Docker cluster example uses `ha.lease_ttl_ms = 10000` and `ha.loop_interval_ms = 1000`. Those values bound freshness checks and influence when stale member or leader state is treated as unsafe.

## Respond to Primary Failure

No manual intervention is required for most failures. The HA decision engine automatically:

1. Detects PostgreSQL unreachability
2. Releases the leader lease if held by the failed node
3. Transitions to `rewinding` phase
4. Selects a recovery target from healthy members
5. Executes `pg_rewind` or base backup to rejoin the cluster

To monitor automated recovery, keep sampling `/ha/state` across the cluster and compare leader identity, trust state, phase, and decision.

### If automation stalls

If decisions remain unchanged and trust stays at `fail_safe` or `not_trusted`, resolve the underlying etcd, network, or PostgreSQL problem before expecting promotion to proceed.

## Verify Recovery

Once a new primary is visible in HA state, verify data consistency:

1. Confirm exactly one primary via SQL:

```bash
for node in node-a node-b node-c; do
  psql -h ${node} -U postgres -c "SELECT pg_is_in_recovery();"
done
```

Exactly one node should return `false`.

2. Check replication lag on replicas:

```bash
psql -h <replica-ip> -U postgres -c "SELECT now() - pg_last_xact_replay_timestamp();"
```

3. Validate DCS trust restoration through the `dcs_trust` field in `/ha/state`.

## Troubleshoot Common Scenarios

### All Nodes in `fail_safe` Phase

**Cause**: etcd unreachable or most member records stale.  
**Action**: Restore etcd cluster health first.

### New Primary Elected but Replicas Stuck in `rewinding`

**Cause**: `pg_rewind` fails due to timeline divergence or WAL gaps.  
**Action**: The node can escalate from rewind-related recovery to bootstrap or base-backup paths automatically.

### Split-Brain Detected

**Cause**: Network partition created multiple primaries. The observer flags this when samples show `max_concurrent_primaries > 1`.  
**Action**: pgtuskmaster can enter fencing when a foreign leader is detected.

### Leader Lease Release Stalls

**Cause**: The previous primary cannot reach etcd to release its lease.  
`ha.lease_ttl_ms` bounds member-record freshness and therefore affects how stale leader or member state is treated.

## Verification Checklist

- [ ] Exactly one node in `primary` phase
- [ ] All other nodes in `replica` phase
- [ ] `dcs_trust: "full_quorum"` on majority
- [ ] `pg_is_in_recovery()` returns `false` on one node only
- [ ] Replication lag on replicas is acceptable
- [ ] No split-brain warnings in observer logs
- [ ] Application traffic can write to the new primary
