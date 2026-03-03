# Switchover

Switchover is “planned”: an operator intentionally requests a controlled primary change.

Unlike failover, switchover is driven by a **human/automation intent** record and should be observable end-to-end.

```mermaid
sequenceDiagram
  participant Op as Operator
  participant API as Node API
  participant ETCD as DCS (etcd)
  participant HA as HA worker
  participant Old as Old primary (Postgres)
  participant New as New primary (Postgres)

  Op->>API: POST /switchover
  API->>ETCD: write switchover intent
  HA->>ETCD: observe intent + trust
  HA->>Old: request demotion
  HA->>ETCD: establish new leader record\nwhen safe
  HA->>New: request promotion
  HA->>ETCD: clear switchover intent
```

Architectural goals:
- make the intent explicit and durable (DCS record)
- ensure demotion happens before promotion when safety requires it
- make the switchover observable via node state reporting

