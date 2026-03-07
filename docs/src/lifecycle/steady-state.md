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
