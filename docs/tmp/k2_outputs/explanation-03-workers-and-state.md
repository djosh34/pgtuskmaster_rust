# Why the runtime is built from workers and versioned state

## State‑publishing topology

pgtuskmaster composes its control plane from narrow workers that publish immutable snapshots rather than calling across layers. Each worker owns one concern: `pginfo` probes PostgreSQL, the `DCS` worker reads the distributed consensus store, the `process` worker manages the local postmaster, `HA` makes high‑availability decisions, and two API workers expose the state to operators. These components never invoke each other directly; they emit versioned updates through tokio watch channels and consume the published streams of other workers when they need context.

The pattern is visible in `src/runtime/node.rs`, where the wiring creates initial states and connects the subscribers. `src/state/watch_state.rs` provides the primitives: `new_state_channel<T>()` returns a `(publisher, subscriber)` pair, `publish(next)` increments the version strictly by one, `latest()` returns the current `Versioned<T>`, and `changed().await` lets a task wait for the next snapshot. Because watches only retain the latest value, the runtime relies on versioning to detect stale reads without locking.

## Benefits of versioned snapshots

Versioned snapshots serve three purposes. First, they make data races impossible: a worker reads an immutable `Versioned<T>` and can trust that its fields will not change underneath it. Second, they give every worker a consistent, point‑in‑time view of the whole system even while other workers continue publishing. Third, they enable cheap change detection: comparing versions costs a single integer comparison.

Timestamps (`updated_at`) attached by publishers let downstream workers measure lag directly from the snapshot. No extra metadata channel is required.

## Why the debug API depends on this model

`src/debug_api/worker.rs` assembles a coherent system portrait from six independent state streams (config, pginfo, DCS, process, HA, and its own internal state). Because each source is a `watch::Receiver<Versioned<T>>`, the debug worker can subscribe without coupling to the implementation of any other worker. It reads the latest snapshot from each stream, records a bounded history in memory, and exposes the composite view over HTTP. Without the publisher‑subscriber contract, the debug API would need to poll internals or hold locks, breaking isolation and adding jitter to the critical control loop.

The same pattern appears in the `HA` worker: it consumes the pginfo, DCS, and process subscribers, merges their snapshots, and emits its own derived decisions. No component reaches into another's structs directly.

## Tradeoffs: eventual consistency and lag

The architecture optimizes for observability and testability at the cost of strict coherency. Workers operate on slightly stale data: a decision in `HA` may lag the true database state by the duration it takes the `pginfo` publisher to probe and the watch to propagate. The version field makes that lag measurable but does not eliminate it.

Eventual consistency also means that a status endpoint can return a snapshot where two sub‑systems have not yet converged. Operators should interpret composite responses as “as‑of‑timestamp” views rather than atomic truths. The bounded history in the debug API helps diagnose such races after the fact, but it does not provide transactional snapshot isolation across workers.

Testing benefits directly: unit tests can inject arbitrary `Versioned<T>` values into a worker’s subscriber without starting the real dependencies. Integration tests can record the timeline by intercepting published snapshots, replay them, and assert version monotonicity. The isolation makes failures local and reproducible.
