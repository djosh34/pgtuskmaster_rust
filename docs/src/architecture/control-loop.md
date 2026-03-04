# Control Loop

The steady-state behavior is a reconciliation loop: the system keeps observing and converging until the safest stable role is reached.

One useful way to picture a single “tick” is a sequence diagram:

```mermaid
sequenceDiagram
  participant PG as PostgreSQL
  participant PgInfo as PgInfo worker
  participant DCS as DCS worker
  participant HA as HA worker
  participant Proc as Process worker
  participant API as API worker

  PgInfo->>PG: Observe local SQL/readiness/LSN
  PgInfo-->>HA: Publish PostgreSQL view

  DCS->>DCS: Refresh cache + trust
  DCS-->>HA: Publish DCS view (members/leader/intent/trust)

  API->>DCS: Write operator intent (optional)<br/>(e.g. switchover request)

  HA->>HA: Decide next safe role/actions
  HA-->>Proc: Action requests (start/stop/rewind/bootstrap)<br/>with safety constraints
  HA-->>DCS: Coordination writes (leader lease,<br/>clear switchover)

  Proc->>PG: Execute actions
```

Important properties:
- Decisions are **guarded** by safety checks.
- Actions are **re-tried conservatively** (the system converges by re-evaluating state on each tick).
- DCS trust can intentionally block certain actions.
