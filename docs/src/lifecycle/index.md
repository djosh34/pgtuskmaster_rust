# System Lifecycle

This section explains what the runtime is doing over time. Read it when you need to understand why a node chose a startup path, why it is following or promoting, or why it entered a safety phase instead of doing the thing you hoped it would do.

The usual reading order is:

1. Bootstrap and startup planning
2. Steady state
3. Planned switchover
4. Unplanned failover
5. Fail-safe and fencing
6. Recovery and rejoin

Treat this section as the operational explanation of the HA loop. For exact API shapes or commands, use [Interfaces](../interfaces/index.md).
