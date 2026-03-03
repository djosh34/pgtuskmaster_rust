# Node Runtime

The node runtime is best understood as “a set of specialized workers connected by a shared state bus”.

```mermaid
flowchart LR
  subgraph Node[pgtuskmaster node]
    Plan[Startup planner] --> Bus[(State bus)]
    PgInfo[PgInfo] --> Bus
    Dcs[DCS] --> Bus
    Ha[HA] --> Bus
    Proc[Process] --> Bus
    Api[API] --> Bus
    Debug[Debug API] --> Bus

    Ha -->|requests actions| Proc
    Ha -->|writes leader/intent| Dcs
    Api -->|operator intent| Ha
  end

  PG[(PostgreSQL)] <-->|probe/control| PgInfo
  PG <-->|start/stop/rewind/bootstrap| Proc
  ETCD[(etcd)] <-->|watch/write| Dcs
```

You should think of this as a “closed loop”:
- workers publish observations
- HA decides
- process executes actions
- DCS writes coordination state
- the next loop sees the consequences

