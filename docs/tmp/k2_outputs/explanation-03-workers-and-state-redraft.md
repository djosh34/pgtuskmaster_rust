# Why the runtime is built from workers and versioned state

> The pgtuskmaster runtime is a state-publishing topology, not a call graph. Each worker owns its domain and publishes immutable snapshots that others subscribe to.

## State as versioned snapshots

Shared state travels over tokio watch channels wrapped in `Versioned<T>` snapshots. Every update increments the version strictly by one. Publishers attach `updated_at` timestamps, but subscribers never see partial updates; they receive whole, atomic snapshots. You either read the latest value or await the next change.

This design removes locks and mutexes from the critical path. Workers are not interrupted by readers, and readers are not blocked by writers. The API is trivial: subscribe, check version, use the data.

## Why workers must stay narrow

The node runtime wires dedicated workers for `pginfo`, DCS, process, HA, API, and debug API. Each worker monitors one source of truth: the local PostgreSQL instance, the consensus store, child processes, or external events.

The HA worker, for example, does not reach into the `pginfo` worker to inspect its internals. It subscribes to `Versioned<InstanceInfo>`. The debug API worker, likewise, does not query HA state directly; it subscribes to `Versioned<ClusterState>` and any other snapshots it needs.

This decoupling lets you restart, replace, or refactor a worker without rewiring the rest of the system. A crash in the debug API worker cannot poison the HA worker, because the only coupling is a watch channel.

## How the debug API exploits the model

The debug API assembles a composite snapshot by reading the latest version from each state channel: config, pginfo, DCS, process, and HA. It records change and timeline history by storing each snapshot with version and timestamp.

Because every publisher increments its version by one, the debug API can detect which domains changed since the last poll and build a minimal diff. No ad-hoc polling loops, no shared pointers, no locks.

## Tradeoffs: stale views and lag

Versioned snapshots are always slightly stale. A subscriber reading `latest()` sees the value from the last scheduler tick, not the current wall-clock truth. The lag is usually microseconds but can stretch if a worker is blocked.

Waiting for `changed()` reduces staleness but adds latency: you block until the publisher commits a new version. You can have freshness or responsiveness, not both.

The strict +1 increment also means a noisy publisher can starve the channel if subscribers do not keep up. Tokio's watch channel drops intermediate snapshots, so a slow subscriber may miss versions and see larger gaps in time. The system favors availability over linearizability.
