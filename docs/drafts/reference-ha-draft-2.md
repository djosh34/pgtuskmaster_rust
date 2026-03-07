# HA Draft 2

Compass classification: cognition + application.

## Scope

This draft describes the HA subsystem in `src/ha/` by mirroring the module split instead of leading with the phase list.

## Candidate structure

- Purpose
- Module map
- State types
- Decision derivation
- Lowering and dispatch
- Runtime loop

## Notes

- `state.rs` defines worker context, state, world snapshots, and dispatch defaults.
- `decision.rs` defines `DecisionFacts`, `HaDecision`, `StepDownPlan`, `RecoveryStrategy`, and lease-release reasons.
- `decide.rs` maps `(phase, facts)` to the next phase and decision.
- `apply.rs` dispatches lowered effects and accumulates `ActionDispatchError` values.
- `worker.rs` publishes the new HA state before dispatch and can republish a faulted state if dispatch fails.
