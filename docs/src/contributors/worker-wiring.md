# Worker Wiring and State Flow

Worker composition follows a clear direction: observation feeds decision, decision feeds action, and actions feed next observations.

## High-level wiring

```text
Simplified shape: the runtime wires worker *contexts* with shared state channels.

- pginfo: PostgreSQL observations
- dcs: DCS cache + trust (started via `crate::dcs::worker::run(...)` using `DcsWorkerCtx`)
- ha: HA decision loop (phase + actions)
- process: bounded action execution
- api: operator-facing HTTP surface
```

The runtime starts workers with shared state receivers/senders and coordination store handles. Each worker owns one primary output state and consumes specific upstream inputs.

## Reaction model

- PgInfo worker publishes PostgreSQL observation state.
- DCS worker publishes trust plus cache state and writes local membership.
- HA worker consumes both views and emits lifecycle phase plus action requests.
- Process worker executes bounded actions and publishes outcomes.
- API worker projects state and writes operator intents.

## Why this matters

Most behavioral bugs come from misunderstood state ownership. When state ownership is explicit, regressions are easier to localize.
