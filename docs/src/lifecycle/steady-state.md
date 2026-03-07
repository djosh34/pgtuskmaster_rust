# Steady State

After startup planning, the runtime enters continuous reconciliation. Each loop reevaluates local PostgreSQL state, DCS trust, and coordination records.

In stable operation:

- one member acts as primary
- replicas follow the current leader
- leader record is re-evaluated each HA loop based on current DCS trust and coordination records
- switchover intent is empty unless requested

Steady state is not inactivity. It is the phase where the node keeps proving that the current role still matches the evidence it sees.

## What to look for

During healthy steady state, you should be able to explain:

- why this node is primary or replica
- whether `dcs_trust` is full quorum, fail-safe, or not trusted
- whether any switchover intent is pending
- whether the process worker is mostly idle because nothing needs doing

## When steady state looks suspicious

A node that appears idle may still be healthy and actively reconciling. Differentiate these cases with `/ha/state` and recent logs:

- healthy idle: stable phase, stable trust, no error churn
- blocked action: repeated process or HA warnings
- degraded trust: conservative phase changes without local PostgreSQL failure

## What "healthy" means under real conditions

Healthy steady state does not mean that nothing changes. It means changes remain explainable and bounded. The DCS worker keeps refreshing member records. The HA worker keeps increasing its tick count as it reevaluates the world snapshot. The debug timeline can still record normal updates even when the node is not transitioning roles. A healthy cluster is therefore a cluster whose control loop remains boring for understandable reasons, not a cluster with zero observable movement.

For a primary, healthy usually means:

- local PostgreSQL is reachable and already primary
- the leader record still names this member
- no conflicting active leader evidence appears
- trust remains strong enough that no fail-safe downgrade is needed

For a replica, healthy usually means:

- local PostgreSQL is reachable and not trying to self-promote
- there is a believable active leader to follow
- rewind or recovery is not currently required
- process activity is quiet because the replica is already aligned

## Expected drift windows

Even healthy clusters have short periods where one surface updates before another. A DCS record can refresh before a new API snapshot is published. A process action can finish before the next HA tick reclassifies the phase. A member can remain visible in DCS briefly after its surrounding topology has changed. Those small windows are normal consequences of a polling worker topology.

What matters is whether the system converges back to a self-consistent picture. Healthy drift windows are brief and explainable. Suspicious drift is repeated or one-directional, such as a node that remains stuck with stale trust, never reacquires a believable leader, or keeps reporting process churn that never changes the phase outcome.

## Quiet versus suspiciously idle

Operators often misread a quiet node in one of two ways: either they assume it is healthy because nothing dramatic is happening, or they assume it is broken because no visible work is happening. The right interpretation comes from the surrounding evidence.

Quiet is healthy when the current phase, trust posture, and role evidence all still line up. Quiet is suspicious when the cluster clearly needs action but the node keeps publishing the same blocked state without a good reason, or when logs show repeated failed attempts that the summary surfaces are not explaining clearly enough.

A good steady-state triage loop is:

1. Read `/ha/state` for phase, decision, trust, and leader context.
2. Compare that against local PostgreSQL reachability and role.
3. Check whether the process worker is actually idle or merely between repeated failed jobs.
4. Use the debug timeline or logs to see whether the picture is converging, stalled, or getting more contradictory.

## Why steady state still matters during incidents

Most incidents are not clean jumps from health to failover. They pass through a period where the old steady-state assumptions are becoming less true but have not yet fully collapsed. That is where correct interpretation matters most. If you understand what a healthy steady state should keep proving, you can recognize exactly which proof failed: trust freshness, leader visibility, local PostgreSQL reachability, or recovery feasibility.

That is also why this page sits before the explicit failover and fail-safe chapters. You need to know what normal proof looks like before you can tell whether the node is prudently leaving normal operation or is stuck for some unrelated reason.
