# Mental Model

At runtime, a node behaves like a small control plane: several specialized components continuously share state and converge on a safe role for PostgreSQL.

The important mental model is **ownership**: each component owns one slice of the world and publishes it for others to consume.

```mermaid
flowchart TB
  subgraph Node[One pgtuskmaster node]
    PgInfo[PgInfo worker\n\"What is Postgres doing?\"] --> Bus[(State bus)]
    Dcs[DCS worker\n\"What does etcd say?\"] --> Bus
    Ha[HA worker\n\"What should we do next?\"] --> Bus
    Proc[Process worker\n\"Perform actions safely\"] --> Bus
    Debug[Debug snapshot worker\n\"Explain what changed\"] --> Bus
    Bus --> Api[Node API worker\n\"Operator controls & status\"]
  end

  PG[(PostgreSQL)] --> PgInfo
  ETCD[(etcd)] --> Dcs
  Proc --> PG
  Ha --> ETCD
  Api --> ETCD
```

What to look for when debugging behavior:
- If PostgreSQL is down or misconfigured: start with `PgInfo`.
- If coordination looks wrong or stale: start with `DCS` trust and cache.
- If the node refuses promotion: check `HA` safety/fencing decisions.
- If the node is “doing nothing”: check whether `Process` is blocked on a safety precondition.
