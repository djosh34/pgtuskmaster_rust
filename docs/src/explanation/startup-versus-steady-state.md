# Why startup planning is separate from steady-state control

Steady-state HA assumes that the node already knows what kind of PostgreSQL state it has, what the cluster looks like, and which managed files belong on disk. Startup exists because those assumptions are not safe at process birth.

The [node runtime reference](../reference/node-runtime.md) lists the startup modes and actions. This page explains why startup is treated as its own phase before the normal worker topology begins.

## Startup is about discovering reality

`run_node_from_config` validates config, bootstraps logging, plans startup, executes startup, and only then launches the long-running workers.

Planning startup means inspecting the data directory, probing DCS cache state, and classifying the node into one of three internal modes:

- initialize a primary
- clone a replica
- resume an existing installation

Those are runtime classifications, not user-facing commands. The runtime is deciding what kind of physical reality it is standing in, not yet what HA policy should do about an already-running node.

## Why startup is not "just another HA tick"

The HA worker reasons over published runtime state. Startup exists before that state topology is fully trustworthy. At startup time, the node may need to claim an init lock, seed configuration, run process jobs, reconstruct resume intent from existing managed files, and only then start PostgreSQL with the right managed start intent.

Trying to fold all of that into the ordinary HA loop would blur two different jobs:

- discovering and normalizing local state
- coordinating an already-started node with the rest of the cluster

By keeping them separate, the runtime avoids asking HA to reason about a node whose own local foundation is still being established.

## Why execution is sequential first and concurrent later

Startup work is intentionally ordered. The runtime resolves the node's local state first and only then launches concurrent workers. That gives the rest of the system a cleaner starting point: the data directory has been classified, managed files have been written, and PostgreSQL has been started according to the chosen mode.

The benefit is determinism. Worker concurrency begins after the runtime has reduced boot ambiguity as far as it can.

## The tradeoff

This split adds a visible boundary between process launch and steady-state control, and it means startup carries special logic that HA does not own. The payoff is that runtime coordination becomes simpler after boot, because the long-running workers can assume they are operating on a node that has already been brought into a known local posture.
