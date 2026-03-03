# Interfaces

This section describes the ways humans and automation interact with the node.

There are two primary interaction styles:
- “Control”: request an operation (for example, switchover)
- “Observe”: read current state (for example, HA state)

```mermaid
flowchart LR
  Op[Operator / Automation] --> API[Node API]
  Op --> CLI[pgtuskmasterctl]
  CLI --> API

  API --> Runtime[Node Runtime]
  Runtime --> ETCD[(DCS)]
  Runtime --> PG[(PostgreSQL)]
```

