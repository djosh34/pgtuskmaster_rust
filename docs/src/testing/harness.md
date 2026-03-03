# Harness Architecture

Many tests run the real node binaries and external dependencies (etcd, PostgreSQL) under a harness.

```mermaid
flowchart LR
  Test[Test code] --> Harness[Test harness]
  Harness --> Node[pgtuskmaster process]
  Harness --> ETCD[(etcd process)]
  Harness --> PG[(PostgreSQL process)]
  Test -->|assertions| Harness
```

The harness exists to make stateful scenarios reproducible:
- multi-node topologies
- failover transitions
- fencing and safety invariants

