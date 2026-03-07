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

Each page answers a different class of question:

- **Bootstrap and Startup Planning** explains how the node chooses initialize, clone, or resume before it can reconcile normally.
- **Steady State** explains what "healthy and quiet" really means and how the loop keeps re-proving the current role.
- **Planned Switchover** explains why accepted operator intent can still wait for safer conditions.
- **Unplanned Failover** explains how promotion is gated when the old leader disappears or becomes doubtful.
- **Fail-Safe and Fencing** explains the two main conservative safety brakes and why they should not be interpreted the same way.
- **Recovery and Rejoin** explains how a former writer or divergent replica becomes eligible to follow again.

Use this chapter family when behavior explanation matters more than step-by-step operation. If you already know which button or route to use, the Operator and Interfaces chapters will be faster. If you need to judge whether the conservative behavior is intentional or excessive, pair these lifecycle pages with [Architecture Assurance](../assurance/index.md).
