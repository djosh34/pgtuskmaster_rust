# Safety Invariants

Safety invariants are the conditions the system tries to preserve across all lifecycle transitions.

Core invariants:

- role changes are conditioned on current evidence, not stale assumptions
- conflicting leader evidence triggers conservative behavior
- degraded trust constrains promotion behavior
- recovery actions are explicit before rejoin eligibility

## How to use these invariants

When behavior surprises you, these invariants help separate expected strictness from a real defect. If the system is being conservative because trust is degraded or because a conflicting leader exists, that is policy. If the system violates one of these expectations without evidence, that is where to dig.
