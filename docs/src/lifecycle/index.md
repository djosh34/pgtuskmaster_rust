# System Lifecycle

This section explains the runtime as a sequence of operational phases. Instead of treating HA as a black box, it describes what the node is expected to do in each phase and what evidence gates transitions.

Lifecycle order in this guide:

1. Bootstrap and startup planning
2. Steady state
3. Planned switchover
4. Unplanned failover
5. Fail-safe and fencing
6. Recovery and rejoin

Use this section when behavior changes over time and you need to understand transition logic, not only static configuration. Treat implementation tests as the final source of truth for edge-case behavior.
