# Decision Model

The HA decision model combines three evidence classes:

- local PostgreSQL state
- DCS trust and coordination records
- current lifecycle phase and safety constraints

Decisions are then projected into process actions and coordination writes.

```mermaid
flowchart LR
  PG[PostgreSQL view] --> Decide[HA decide]
  DCS[DCS trust + cache] --> Decide
  Phase[Current phase] --> Decide
  Decide --> Proc[Process actions]
  Decide --> Writes[DCS writes]
```

## Why this exists

The HA worker uses a single decision function to map explicit inputs (PostgreSQL view, DCS cache/trust, current phase) into a next phase and action set. Operator control inputs (like switchover requests) enter the same pipeline via DCS rather than bypassing it.

## Tradeoffs

The model is intentionally conservative under low-confidence coordination. This can delay actions that appear feasible from one signal alone.

## When this matters in operations

During incident triage, ask which input class is blocking progress: local readiness, trust level, or phase safety guard.
