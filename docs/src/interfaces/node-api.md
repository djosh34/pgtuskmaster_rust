# Node API

The node API is the primary operator interface for HA control and state inspection.

The API is intentionally small: it is meant to express **intent**, not to expose internal mechanisms.

## High-level endpoints
- `GET /ha/state`: observe current HA-relevant state
- `POST /switchover`: request a planned primary transition
- `DELETE /ha/switchover`: cancel/clear a pending switchover request

```mermaid
sequenceDiagram
  participant Op as Operator
  participant API as Node API
  participant HA as HA worker
  participant ETCD as DCS (etcd)

  Op->>API: request intent\n(switchover)
  API->>ETCD: write intent record
  HA->>ETCD: observe intent
  HA-->>API: state reflects intent\nand progress over time
```

## Authentication / authorization model
At a high level, the API distinguishes:
- read-only status access
- admin actions that mutate intent

Exact token fields and deployment policy are documented under Operations.

