# Steady State

After startup planning, the runtime enters continuous reconciliation. Each loop reevaluates local PostgreSQL state, DCS trust, and coordination records.

In stable operation:
- one member acts as primary
- replicas follow the current leader
- leader lease remains current
- switchover intent is empty unless requested

## Why this exists

Steady-state control is not inactivity. It is active validation that current role and coordination evidence still agree.

## Tradeoffs

Continuous checking adds control-plane activity. The benefit is faster detection of drift and more reliable response when assumptions break.

## When this matters in operations

A node that appears idle may still be healthy and actively reconciling. Use API state and logs to differentiate idle stability from blocked action.
