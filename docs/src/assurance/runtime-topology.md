# Runtime Topology and Boundaries

A node contains multiple specialized workers with bounded responsibilities. The system boundary is local PostgreSQL management plus DCS coordination.

```mermaid
flowchart TB
  subgraph Node[pgtuskmaster node]
    PgInfo[PgInfo]
    Dcs[DCS worker]
    Ha[HA worker]
    Proc[Process worker]
    Api[API worker]
  end

  PG[(PostgreSQL)] --> PgInfo
  Proc --> PG
  DCS[(etcd)] --> Dcs
  Dcs --> Ha
  PgInfo --> Ha
  Ha --> Proc
  Api --> DCS
```

## Why this exists

Bounded worker responsibilities reduce coupling and make transition reasoning clearer.

## Tradeoffs

More explicit worker boundaries create more internal interfaces. The benefit is better observability and easier targeted testing of behavior paths.

## When this matters in operations

When a symptom appears, this topology helps identify whether the issue starts in observation, decision, action, or coordination.
