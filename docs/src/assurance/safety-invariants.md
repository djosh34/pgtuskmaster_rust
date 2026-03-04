# Safety Invariants

Safety invariants are conditions the system tries to preserve across all lifecycle transitions.

Core invariants:
- role changes are conditioned on current evidence, not stale assumptions
- conflicting leader evidence triggers conservative behavior
- degraded trust constrains promotion behavior
- recovery actions are explicit before rejoin eligibility

## Why this exists

Without invariants, incident behavior becomes ad-hoc and difficult to reason about. Invariants give operators and architects stable expectations across different failure patterns.

## Tradeoffs

Hard invariants can make behavior look strict in edge cases. The benefit is that strictness is visible and explainable, rather than accidental.

## When this matters in operations

When transitions surprise operators, invariant checks explain whether behavior is policy-driven or genuinely unexpected.
