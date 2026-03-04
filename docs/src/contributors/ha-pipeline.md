# HA Decision and Action Pipeline

The HA pipeline is the architectural core.

## Decision path

1. Read latest PostgreSQL and DCS state snapshots.
2. Evaluate trust posture and leader evidence.
3. Compute next phase and required actions.
4. Publish HA state.
5. Dispatch process actions and coordination writes.

## Example decision shape

```rust
pub(crate) fn decide(world: &HaWorldView) -> HaDecision {
    // Evaluate trust, role eligibility, and safety invariants.
    // Return phase + actions + coordination updates.
}
```

## Action categories

- local process actions: start, stop, promote, demote, rewind, bootstrap
- coordination actions: leader lease acquire/release, switchover intent clear

## Why this structure exists

The separation between pure decision logic and side-effect execution keeps behavior testable and deterministic.

## Tradeoffs

This split can feel verbose in implementation, but it allows matrix-style decision testing without process side effects.
