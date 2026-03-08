# Why pginfo observes PostgreSQL instead of controlling it

pginfo is a sensor, not a controller. Its job is to report what it sees, not to act on it.

The worker polls PostgreSQL with `poll_once` using `postgres_conninfo`. When the poll succeeds, it publishes a state with `WorkerStatus::Running` and `SqlStatus::Healthy`. When the poll fails, it emits a warning event and publishes `WorkerStatus::Running` with `SqlStatus::Unreachable`. Any change in SQL status triggers a transition event. Tests in `src/pginfo/worker.rs` verify these transitions, including recovery from unreachable to primary and tracking of WAL and replication slots.

This observational stance is deliberate. pginfo does not restart PostgreSQL, nor does it promote a standby. It does not fence nodes or trigger failover. Instead, it feeds its observations to the HA worker, which subscribes to pginfo state as one input among many when deciding what to do next.

## Degraded observations let the system reason

Publishing `SqlStatus::Unreachable` rather than halting with a fatal error keeps the system responsive. A failed poll might mean the network dropped a packet, PostgreSQL is restarting, or the node is genuinely down. By reporting the degradation and continuing to poll, pginfo gives the HA layer the data it needs to distinguish transient noise from real failure. The rest of the system can weigh pginfo's report against other signals—heartbeats, disk health, replica lag—before committing to action.

## Separation improves reasoning

Keeping observation separate from process control makes each component easier to reason about. pginfo's logic is simple: connect, query, publish. The HA state machine (`src/ha/worker.rs`) consumes those facts and applies policy. The process worker handles starting and stopping PostgreSQL. When control logic is tangled with observation, a bug in one can mask the other. By staying in its lane, pginfo provides a trustworthy stream of facts that any consumer—HA, dashboards, alerts—can interpret without side effects.

## Tradeoffs of polling and asynchronous observation

Polling introduces lag. A brief outage might be missed if it falls between poll intervals, and a single failed poll does not guarantee PostgreSQL is down. Asynchronous observation also means consumers see a snapshot, not a live system state. The benefit is resilience: pginfo continues to publish what it can, and the HA layer can smooth over spurious failures with its own heuristics. This tradeoff favors robustness over immediacy, giving the system room to breathe rather than reacting to every transient blip.
