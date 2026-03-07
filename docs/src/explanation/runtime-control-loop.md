# About the Runtime Control Loop

`pgtuskmaster` starts with a short planning phase and then settles into a worker-based control loop. The runtime is arranged this way so that cluster management decisions come from shared snapshots of state rather than from tightly coupled direct calls between subsystems.

## Two distinct stages

The entrypoint in `runtime::node` first validates configuration, boots logging, derives process defaults, and chooses a startup mode. That startup mode is selected from two pieces of evidence: the local PostgreSQL data directory state and a one-time DCS cache probe. This keeps bootstrap, replica cloning, and existing-node resume logic outside the steady-state HA loop.

After startup actions succeed, the runtime changes shape. Instead of continuing as one monolithic procedure, it creates long-lived workers for PostgreSQL inspection, DCS synchronization, process execution, PostgreSQL log ingestion, HA decision-making, the debug API, and the external API server.

## Why the runtime is worker-based

The worker split creates a boundary between observation, decision, and side effects.

- `pginfo` observes the local PostgreSQL instance.
- `dcs` turns the backing store into a typed local cache plus a trust signal.
- `process` owns disruptive operating-system work such as starting PostgreSQL, rewind, base backup, bootstrap, demotion, and fencing.
- `ha` consumes the latest snapshots and chooses a decision plus an effect plan.
- `debug_api` republishes the current world as a snapshot.
- `api` exposes the supported external control surface.

That structure matters because the HA worker is not expected to discover facts for itself or run shell operations directly. It reasons over published state and sends work out through explicit channels.

## Versioned state as the runtime's shared language

The runtime creates a versioned state channel for each major subsystem. Each worker publishes its latest state, and other workers subscribe to the newest version instead of waiting on synchronous calls. This gives the system a common language for sharing observations without forcing all workers to run in lockstep.

The result is a runtime that behaves more like a set of observers around a shared world model than like a call stack. That is why HA can build a `WorldSnapshot` from the latest configuration, PostgreSQL, DCS, and process states at one moment in time.

## Why startup is outside the steady-state loop

Startup planning answers a narrow question: is this node creating a primary, cloning from another primary, or resuming existing data? That question depends on local disk state and whatever DCS evidence is immediately available. Once that initial shape is chosen and the required actions complete, the runtime can move into the more repetitive steady-state loop where HA decisions are revisited every poll interval.

Keeping startup separate avoids overloading the HA state machine with one-off filesystem inspection and bootstrap sequencing concerns. The HA loop can then focus on ongoing coordination and recovery, while the runtime entrypoint handles the transition from "a process with a config file" to "a node participating in a cluster."

## The control loop boundary

The runtime is not a single loop in one file. It is a composed loop:

- each worker polls or reacts on its own cadence
- each worker republishes its view of the world
- HA periodically turns those views into decisions
- effect dispatch pushes state-changing work back into DCS writes or process jobs

This arrangement is the reason the reference pages for configuration, DCS, and HA describe separate machinery. The explanation is that those pieces are loosely coupled in code, but intentionally coupled through state snapshots and explicit action dispatch.

See also:

- [Configuration Reference](../reference/config.md)
- [DCS Reference](../reference/dcs.md)
- [HA Reference](../reference/ha.md)
