# System Context

At a high level, the system has three categories of participants:
- **Operators/automation** (human or software) initiating and observing HA actions
- **The node runtime** coordinating local PostgreSQL
- **External state systems** (PostgreSQL itself and etcd for coordination)

```mermaid
flowchart LR
  subgraph Outside[Outside the node]
    Operator[Operator / Automation]
    Clients[App clients]
    ETCD[(etcd / DCS)]
  end

  subgraph Node[pgtuskmaster node]
    API[Node API\n(includes debug routes)]
    Runtime[Node Runtime]
    Debug[Debug snapshot worker]
    PG[(PostgreSQL)]
  end

  Operator -->|HTTP control + read\n(including debug)| API
  Clients -->|SQL| PG

  Debug --> API
  API --- Runtime
  Runtime <-->|watches/writes| ETCD
  Runtime -->|starts/stops/rewires| PG
```

Things that are intentionally *not* in scope of the node:
- Application traffic routing, connection pooling, or VIP management.
- A centralized scheduler across clusters.

The node is designed to be deployable as a local supervisor that makes safe decisions given the signals it can observe.
