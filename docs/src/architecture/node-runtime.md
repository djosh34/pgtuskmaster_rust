# Node Runtime

The node runtime is best understood as “a set of specialized workers connected by a shared state bus”.

```mermaid
flowchart LR
  subgraph Node[pgtuskmaster node]
    Bus[(State bus)]
    PgInfo[PgInfo worker] --> Bus
    Dcs[DCS worker] --> Bus
    Ha[HA worker] --> Bus
    Proc[Process worker] --> Bus
    Debug[Debug snapshot worker] --> Bus
    Bus --> Api[Node API]

    Ha -->|requests actions| Proc
  end

  PG[(PostgreSQL)] <-->|probe/control| PgInfo
  PG <-->|start/stop/rewind/bootstrap| Proc
  ETCD[(etcd)] <-->|watch/write| Dcs
  Api -->|intent| ETCD
  Ha -->|leader lease + coordination writes| ETCD
```

You should think of this as a “closed loop”:
- workers publish observations
- HA decides
- process executes actions
- DCS writes coordination state
- the next loop sees the consequences
