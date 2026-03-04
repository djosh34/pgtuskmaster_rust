# How The System Solves It

The runtime follows a continuous observe-decide-act loop. It combines local PostgreSQL signals with distributed coordination data, then applies role decisions through controlled actions.

```mermaid
flowchart TD
  Observe[Observe PG + DCS state] --> Decide[Decide safest next phase]
  Decide --> Act[Execute process and coordination actions]
  Act --> Observe
```

At a high level, each node does three things repeatedly:

- It observes local PostgreSQL state and shared DCS state.
- It evaluates trust and role conditions.
- It executes bounded actions, then reevaluates.

This design keeps decisions current. Instead of assuming one static cluster view, every loop rechecks the evidence before progressing.

## Why this matters

Role changes are not single events. They are state transitions with preconditions. The loop model makes those preconditions explicit and continuously validated.

## Tradeoffs

A loop-based controller can look cautious, because it revalidates instead of rushing actions. The loop continuously re-runs decision from fresh state snapshots and uses explicit guards (trust, PostgreSQL reachability, leader availability, switchover intent), so transitions are controlled and repeatable. Safety behavior is bounded by transition logic and validated by fail-safe/fencing/switchover/idempotency tests.

## When this matters in operations

During incidents, operators can reason about current behavior by asking three questions: what is the node observing, what decision did it make, and what action is blocked or running. In practice, correlate `/ha/state` with debug payloads (`/debug/verbose` or `/debug/snapshot` when enabled), plus DCS record views and relevant logs.
