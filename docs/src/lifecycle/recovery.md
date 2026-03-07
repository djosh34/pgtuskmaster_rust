# Recovery and Rejoin

After failover or fencing events, nodes may need recovery work before they can safely follow or become eligible again.

Common recovery paths:

- rewind when divergence is recoverable
- bootstrap when rewind is unsafe or unavailable
- rejoin as replica after data and coordination state are coherent

For managed starts after rewind, bootstrap, or rejoin work, `pgtuskmaster` rebuilds the authoritative PostgreSQL startup surface itself. That means:

- recovery and follow settings live in `PGDATA/pgtm.postgresql.conf`
- managed signal files are normalized to the single expected state before PostgreSQL starts
- any active `PGDATA/postgresql.auto.conf` is quarantined out of the live startup path

The runtime does not treat leftover PostgreSQL side-effect files as authoritative recovery instructions. Recovery posture must be reconstructible from DCS and runtime state plus previously managed artifacts; otherwise startup fails instead of guessing.

## Recovery paths

A node that was previously primary can carry divergent history, so rejoin is not automatic. The usual recovery paths are:

- rewind when divergence is recoverable
- basebackup or bootstrap when rewind is unsafe or unavailable
- replica rejoin after the data and coordination state line up again

## What to diagnose first

If a node repeatedly fails to rejoin, treat these as first-class checks:

- rewind identity and password wiring
- replication authentication and HBA rules
- connectivity to the current leader's advertised PostgreSQL endpoint
- process job results and timeouts for rewind or bootstrap work

Do not force a node back into normal eligibility until the recovery preconditions are satisfied and visible in the runtime state.

## The recovery decision branches

Recovery is driven by what the runtime can still trust about the current leader and about local history. If a believable external primary is available and the local node appears to have diverged, the HA loop can choose a rewind strategy. If rewind completes successfully, the node can move back toward following that leader. If rewind fails but a believable leader still exists, the runtime can escalate to a base-backup style rebuild path instead of leaving the node permanently half-recovered.

If no believable recovery leader is available, the runtime becomes much more cautious. A node without a trusted source cannot safely reinvent its history. In that case, waiting, fencing, or bootstrap-style recovery may be safer than trying to rejoin immediately.

## Rewinding versus bootstrapping

### Rewinding

Rewind is the preferred recovery path when the node's local state is still close enough to a visible leader that `pg_rewind` can repair divergence. Operationally, this is attractive because it preserves more of the local data directory and is often faster than a full rebuild. The runtime only chooses it when there is a believable leader source to rewind from.

### Base-backup or bootstrap-style recovery

When rewind is not possible or has already failed, the runtime can fall back to a more destructive recovery path. In the decision model this shows up as bootstrapping work driven by an explicit recovery strategy rather than as a silent side effect. That distinction matters because it tells operators that the node is no longer preserving local history opportunistically; it is deliberately rebuilding into a new coherent replica state.

## Validation checkpoints after recovery

After any recovery work, operators should verify more than "the process is running again". At minimum, confirm:

- the node can reach PostgreSQL locally
- the node now identifies a believable leader to follow
- `/ha/state` no longer shows a recovery-phase decision that keeps retrying or falling back
- logs and debug history show a coherent move from recovery into ordinary following or waiting-for-trust behavior

If those checks fail, the recovery is not complete even if PostgreSQL is technically alive.

## When to trust a recovered member again

Trust a recovered member only after both the coordination and local PostgreSQL views agree on its role. A node that has just exited rewind or bootstrap is not automatically safe to treat as a future primary candidate. It first needs to demonstrate that it can rejoin cleanly, follow the current leader, and publish fresh member state that matches the cluster's current reality.

This is why the recovery chapter exists separately from failover. Promotion and recovery are related but different problems. Getting writes available somewhere is only part of the job. Rebuilding the rest of the topology without preserving hidden divergence is the rest.
