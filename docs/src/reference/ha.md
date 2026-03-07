# HA Reference

The `ha` subsystem derives high-availability decisions from the current world state, lowers those decisions into effect plans, and dispatches the resulting actions to DCS and process workers.

## Module layout

| Module | Surface |
| --- | --- |
| `ha::state` | HA worker context, state types, world snapshot, and process dispatch defaults |
| `ha::decision` | Derived decision facts and the decision/result enums |
| `ha::decide` | Phase-by-phase state machine |
| `ha::apply` | Effect-plan dispatch and dispatch error aggregation |
| `ha::actions` | Action identifiers and concrete action variants |
| `ha::lower` | Lowering from `HaDecision` to effect plans |
| `ha::process_dispatch` | Translation from HA actions to process jobs |
| `ha::events` | Structured log emission for decisions, transitions, and dispatch |

## Inputs and published state

`HaWorkerCtx` contains subscribers for four versioned inputs:

| Input | Source |
| --- | --- |
| `config_subscriber` | Runtime configuration |
| `pg_subscriber` | PostgreSQL state |
| `dcs_subscriber` | DCS state |
| `process_subscriber` | Process worker state |

The context also owns a DCS store handle, a process inbox, scope and member identifiers, dispatch defaults, a clock function, and a log handle.

`HaState` publishes four values:

| Field | Meaning |
| --- | --- |
| `worker` | Worker status |
| `phase` | Current HA phase |
| `tick` | Monotonic decision counter |
| `decision` | Last selected `HaDecision` |

## Phases

`HaPhase` contains these states:

| Phase | Meaning |
| --- | --- |
| `init` | Initial state before PostgreSQL reachability checks |
| `waiting_postgres_reachable` | Waiting for PostgreSQL to become reachable or for a start request to complete |
| `waiting_dcs_trusted` | Waiting for trusted DCS state before choosing a role |
| `waiting_switchover_successor` | Waiting for a different leader after a switchover-triggered step-down |
| `replica` | Following another leader or preparing replica recovery |
| `candidate_leader` | Competing for leadership when no usable leader is present |
| `primary` | Acting as primary |
| `rewinding` | Recovering a replica through `pg_rewind` |
| `bootstrapping` | Recovering a replica through base backup or bootstrap |
| `fencing` | Fencing after a conflicting leader condition or failed recovery |
| `fail_safe` | Loss of trusted DCS state while the node must avoid unsafe transitions |

## Decision surface

`DecisionFacts::from_world` derives the decision input from the world snapshot. The derived facts include DCS trust, PostgreSQL reachability and role, leader records, available primary candidates, pending switchover requests, rewind necessity, and current process activity.

`HaDecision` contains these variants:

| Decision | Meaning |
| --- | --- |
| `NoChange` | No action for this tick |
| `WaitForPostgres` | Wait for PostgreSQL, optionally after requesting a start |
| `WaitForDcsTrust` | Hold until DCS is trusted |
| `AttemptLeadership` | Try to acquire leadership |
| `FollowLeader` | Follow the named leader member |
| `BecomePrimary` | Promote or confirm primary state |
| `StepDown` | Step down with explicit lease and switchover handling |
| `RecoverReplica` | Recover through rewind, base backup, or bootstrap |
| `FenceNode` | Fence the local node |
| `ReleaseLeaderLease` | Release the leader lease for a specific reason |
| `EnterFailSafe` | Enter fail-safe mode, optionally releasing leader lease |

`StepDownPlan` records the reason, whether to release leader lease, whether to clear switchover state, and whether fencing is required. `RecoveryStrategy` distinguishes `rewind`, `base_backup`, and `bootstrap`.

## Worker loop and dispatch

`ha::worker::step_once` performs one HA iteration:

1. Build a `WorldSnapshot` from the latest versioned config, PostgreSQL, DCS, and process state.
2. Call `decide` to derive the next `HaState`.
3. Lower the selected decision into an effect plan.
4. Publish the next HA state with `WorkerStatus::Running`.
5. Emit decision, effect-plan, phase-transition, and role-transition events.
6. Dispatch the effect plan unless redundant-process-dispatch suppression applies.
7. Republish a faulted HA state when dispatch returns errors.

`apply_effect_plan` dispatches up to five effect groups: PostgreSQL, lease, switchover, replication, and safety. Replication recovery can prepend `WipeDataDir` before `StartBaseBackup` or `RunBootstrap`.

`should_skip_redundant_process_dispatch` suppresses repeated dispatch when both phase and decision are unchanged and the decision is one of the process-triggering wait or recovery cases tracked in `worker.rs`.

## Error surface

`ActionDispatchError` covers:

| Variant | Meaning |
| --- | --- |
| `ProcessSend` | Sending a process request failed |
| `ManagedConfig` | Managed PostgreSQL configuration materialization failed |
| `Filesystem` | Filesystem work for an action failed |
| `DcsWrite` | A DCS write failed |
| `DcsDelete` | A DCS delete failed |
