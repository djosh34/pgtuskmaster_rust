# Draft: About the Runtime Control Loop

Compass classification: `cognition + acquisition` because the goal is to explain the runtime shape and its tradeoffs, not to instruct the reader to perform an action.

One way to read `runtime::node` is as the point where the project chooses not to be a single HA state machine. The node runtime divides the problem into startup planning, steady-state observation, decision-making, and effect execution. That division is one of the main architectural choices in the codebase.

## A node is assembled, not entered

`run_node_from_config` does not jump directly into HA. It validates configuration, starts logging, derives process defaults, plans startup, executes startup actions, and only then launches the long-lived workers together with `tokio::try_join!`.

This means a node is assembled from cooperating parts:

- startup logic decides how the node joins or resumes cluster life
- state publishers make each subsystem's latest view available
- HA reads those views and chooses the next move
- process and DCS effects carry out the chosen move
- API and debug surfaces observe the same evolving state rather than constructing private models

## Why the runtime uses published state instead of direct orchestration

The runtime creates channels for configuration, PostgreSQL state, DCS state, process state, HA state, and the debug snapshot. The point is not just convenience. Published state keeps expensive or failure-prone work localized.

PostgreSQL probing stays in `pginfo`. DCS watch refresh and trust evaluation stay in `dcs`. Shelling out to PostgreSQL utilities stays in `process`. HA therefore does not need to own sockets, filesystem probes, or child-process management in order to reason about cluster state.

That arrangement gives the system a stable center of gravity: decisions are derived from snapshots, while side effects are delegated outward. The runtime is therefore closer to an evented control plane than to a procedural script.

## Startup and steady-state are different kinds of coordination

Startup has to explain the local node to itself before it can coordinate with peers. It inspects the data directory, probes DCS, and picks one of three modes:

- initialize a new primary
- clone as a replica from another member
- resume existing local state

Steady-state coordination asks a different question: given the latest published world, what should happen next? That is why startup actions are planned in a dedicated sequence, but steady-state behavior is expressed through worker loops and HA phases.

## Why this matters for the rest of the docs

If the runtime were described only as a list of modules, the design would look arbitrary. The explanation is that the code is organized around boundaries between facts, decisions, and side effects:

- startup code handles one-time joining or resuming concerns
- workers publish durable subsystem views
- HA converts those views into decisions and effect plans
- process execution and DCS mutation happen through explicit dispatch boundaries

That is also why the codebase can have both an API server and a debug API without either becoming the center of the system. They sit on top of the runtime's published state rather than replacing it.

See also:

- [Configuration Reference](../src/reference/config.md)
- [DCS Reference](../src/reference/dcs.md)
- [HA Reference](../src/reference/ha.md)
