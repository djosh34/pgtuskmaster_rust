# Why the runtime is built from workers and versioned state

pgtuskmaster is assembled from narrow workers that publish snapshots rather than from one giant controller with shared mutable internals. The [node runtime](../reference/node-runtime.md) wires workers for pginfo, DCS, process, HA, API, and debug API, and the [shared state layer](../reference/shared-state.md) gives them a common way to exchange state.

This design is about boundaries, not style. Each worker owns one domain, and cross-domain coordination happens through versioned snapshots.

## Why versioned snapshots matter

Shared state channels carry `Versioned<T>` values through tokio watch channels. Publishers attach `updated_at` timestamps and advance versions with strict `+1` semantics. Subscribers can read `latest()` or wait on `changed()`.

That gives the runtime a simple contract:

- downstream workers see immutable snapshots rather than half-mutated shared structs
- every published change carries ordering information
- lag is measurable from the snapshot itself rather than from ad hoc side channels

The model is intentionally lightweight. Watch channels keep the latest state, not an ever-growing event log.

## Why the runtime prefers workers over direct coupling

The HA worker consumes pginfo, DCS, process, and config state through subscribers. The debug API worker also consumes multiple subscribers to assemble a system-wide view. Those components do not need to reach into each other's private machinery just to understand the latest state of the world.

That separation makes each worker easier to test and easier to replace. A worker can be driven from published state without pulling in the full runtime stack around it.

## Why the debug API depends on this model

The [debug API](../reference/debug-api.md) is the clearest demonstration of the architecture. It builds snapshots from config, pginfo, DCS, process, and HA state, then records bounded change and timeline history. That only works cleanly because each domain already publishes a versioned snapshot that can be consumed without special-case integration code.

Without that publishing model, the debug surface would have to reconstruct state indirectly from logs or from tighter cross-module coupling. Instead, observability rides on the same state boundaries that the control loop already uses.

## The tradeoff

The price is eventual consistency. A worker can make a decision from a snapshot that is slightly behind the latest real-world state, and a composite view can briefly contain domains that have not converged yet.

The project accepts that cost because versioned snapshots make the lag visible and keep the architecture understandable. The runtime favors explicit asynchronous boundaries over hidden shared-state complexity.
