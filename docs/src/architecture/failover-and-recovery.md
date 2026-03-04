# Failover and Recovery

Failover is “unplanned”: the cluster needs a new primary because the old one disappeared or became unsafe.

Recovery is what makes failover safe:
- the system avoids promoting when it might create split brain (for example, when coordination trust is degraded or when a conflicting leader record exists)
- diverged timelines need rewinding or bootstrapping before following again

## Leader loss → new leader
```mermaid
sequenceDiagram
  participant ETCD as DCS (etcd)
  participant Old as Old leader node
  participant New as Candidate node
  participant PG as PostgreSQL (candidate)

  Old--xETCD: stops refreshing / becomes unreachable
  New->>ETCD: observes leader missing<br/>(and trust level)
  New->>New: evaluates safety invariants
  New->>PG: checks local Postgres is reachable
  New->>ETCD: attempts to acquire leader record
  ETCD-->>New: leader acquired
  New->>PG: promote to primary
```

## Divergence recovery (rewind/bootstrap)
```mermaid
flowchart TD
  Diverged[Local data cannot safely follow primary] --> CanRewind{Can rewind safely?}
  CanRewind -->|yes| Rewind[Rewinding]
  CanRewind -->|no| Bootstrap[Bootstrapping]
  Rewind --> Follow[Resume following]
  Bootstrap --> Follow
```

The key architectural point is not the mechanics of `pg_rewind` or `pg_basebackup`, but that the node routes recovery into explicit phases (rewind when possible; otherwise bootstrap) instead of continuing with “best effort” replication under uncertainty.
