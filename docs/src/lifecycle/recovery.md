# Recovery and Rejoin

After failover or fencing events, nodes may need recovery work before they can safely follow or become eligible again.

Common recovery paths:

- rewind when divergence is recoverable
- bootstrap when rewind is unsafe or unavailable
- rejoin as replica after data and coordination state are coherent

For managed starts after rewind, bootstrap, or rejoin work, pgtuskmaster rebuilds the authoritative PostgreSQL startup surface itself. That means:

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
