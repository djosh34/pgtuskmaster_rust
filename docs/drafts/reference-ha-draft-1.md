# HA Draft 1

Compass classification: cognition + application.

## Scope

This draft describes the HA subsystem in `src/ha/` as phases, decisions, and effect dispatch.

## Candidate structure

- Purpose
- Module layout
- Inputs
- State machine phases
- Decision surface
- Action/effect surface
- Worker loop

## Notes

- `HaWorkerCtx` consumes config, pg, dcs, and process state plus a DCS writer and process inbox.
- `HaState` stores `worker`, `phase`, `tick`, and `decision`.
- The phase enum is `init`, `waiting_postgres_reachable`, `waiting_dcs_trusted`, `waiting_switchover_successor`, `replica`, `candidate_leader`, `primary`, `rewinding`, `bootstrapping`, `fencing`, and `fail_safe`.
- `decide` derives `DecisionFacts`, selects a `HaDecision`, and increments `tick`.
- `apply_effect_plan` dispatches lease, switchover, replication, postgres, and safety effects.
