# Failover and Recovery

Failover is “unplanned”: the cluster needs a new primary because the old one disappeared or became unsafe.

Recovery is what makes failover safe:
- replicas must avoid promoting if they might create split brain
- diverged timelines need rewinding or bootstrapping before following again

## Leader loss → new leader
```mermaid
sequenceDiagram
  participant ETCD as DCS (etcd)
  participant Old as Old leader node
  participant New as Candidate node
  participant PG as PostgreSQL (candidate)

  Old--xETCD: stops refreshing / becomes unreachable
  New->>ETCD: observes leader missing\n(and trust level)
  New->>New: evaluates safety invariants
  New->>PG: ensures local Postgres is promotable
  New->>ETCD: attempts to acquire leader record
  ETCD-->>New: leader acquired
  New->>PG: promote to primary
```

## Divergence recovery (rewind/bootstrap)
```mermaid
flowchart TD
  Diverged[Replica diverged from leader timeline] --> CanRewind{Can rewind safely?}
  CanRewind -->|yes| Rewind[Rewinding]
  CanRewind -->|no| Bootstrap[Bootstrapping]
  Rewind --> Follow[Resume following]
  Bootstrap --> Follow
```

The key architectural point is not the mechanics of `pg_rewind` or `pg_basebackup`, but that the node treats “timeline mismatch” as a first-class safety trigger that routes to explicit recovery phases.

