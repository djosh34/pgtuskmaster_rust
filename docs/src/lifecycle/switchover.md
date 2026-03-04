# Planned Switchover

Switchover is an operator-driven transition. It starts from explicit intent and progresses through demotion and promotion under safety checks.

```mermaid
sequenceDiagram
  participant Op as Operator
  participant API as Node API
  participant DCS as etcd / DCS
  participant HA as HA worker
  participant Old as Current primary
  participant New as Target node

  Op->>API: request switchover
  API->>DCS: write switchover intent
  HA->>DCS: observe intent and trust
  HA->>Old: demote when safe
  HA->>New: promote when lease and readiness allow
  HA->>DCS: clear switchover intent
```

## Why this exists

Planned role changes should be explicit and observable. Intent records provide durable coordination and allow operators to track progress through standard state surfaces.

## Tradeoffs

A strict sequence can be slower than forceful manual promotion. The benefit is lower risk of overlapping primaries and clearer auditability.

## When this matters in operations

If a switchover stalls, treat it as a precondition wait while trust, lease ownership/state, and PostgreSQL readiness constraints are enforced. Check trust posture, node reachability/readiness, and leader lease state first.
