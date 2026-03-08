# Verbose context for docs/src/explanation/ha-decision-engine.md

Core framing:
- The HA decision engine is phase-driven. `src/ha/decide.rs` converts a world snapshot into `DecisionFacts`, then selects the next phase and a `HaDecision`.
- The first major safety gate is trust: when trust is not `FullQuorum`, the engine routes into `FailSafe`.
- If trust is degraded and PostgreSQL is primary, the engine emits `EnterFailSafe { release_leader_lease: false }`.
- If trust is degraded and PostgreSQL is not primary, it still enters the `FailSafe` phase but can emit `NoChange`.

Phases handled by the decision engine:
- `Init`
- `WaitingPostgresReachable`
- `WaitingDcsTrusted`
- `WaitingSwitchoverSuccessor`
- `Replica`
- `CandidateLeader`
- `Primary`
- `Rewinding`
- `Bootstrapping`
- `Fencing`
- `FailSafe`

Decision taxonomy from `src/ha/decision.rs`:
- `NoChange`
- `WaitForPostgres { start_requested, leader_member_id }`
- `WaitForDcsTrust`
- `AttemptLeadership`
- `FollowLeader { leader_member_id }`
- `BecomePrimary { promote }`
- `StepDown(StepDownPlan { reason, release_leader_lease, clear_switchover, fence })`
- `RecoverReplica { strategy }`
- `FenceNode`
- `ReleaseLeaderLease { reason }`
- `EnterFailSafe { release_leader_lease }`

How decisions lower into effects:
- `src/ha/lower.rs` lowers decisions into a bucketed `HaEffectPlan` made of:
  - lease effect
  - switchover effect
  - replication effect
  - postgres effect
  - safety effect
- This shows the intended architecture clearly: the decision engine chooses intent, then the lowerer turns it into concrete effect categories.
- Examples:
  - `AttemptLeadership` -> acquire leader lease
  - `FollowLeader` -> follow replication source
  - `BecomePrimary { promote: true }` -> promote PostgreSQL
  - `StepDown(...)` -> demote PostgreSQL, maybe release lease, maybe clear switchover, maybe fence
  - `RecoverReplica::Rewind` -> one replication recovery step
  - `RecoverReplica::BaseBackup` or `Bootstrap` -> multi-step recovery
  - `EnterFailSafe { release_leader_lease: ... }` -> safety fencing and maybe lease release

What appears idempotent versus more disruptive:
- Most read-and-wait style decisions are effectively idempotent:
  - `NoChange`
  - `WaitForDcsTrust`
  - `AttemptLeadership` can repeat until lease acquisition succeeds or a leader appears
  - `FollowLeader` is a steady-state direction rather than a destructive action
- The more disruptive decisions are the ones that lower into process or safety actions:
  - `BecomePrimary { promote: true }`
  - `StepDown(...)`
  - `RecoverReplica { strategy = Rewind | BaseBackup | Bootstrap }`
  - `FenceNode`
  - `ReleaseLeaderLease`
  - `EnterFailSafe`
- The cleanest precise wording is "some decisions are observational or steady-state, while others dispatch irreversible or externally visible process actions."

When FailSafe is preferred over recovery:
- Trust is checked before the normal phase handlers run.
- If trust is not `FullQuorum`, the engine does not continue with ordinary leadership or recovery logic first.
- That means degraded trust takes precedence over opportunistic recovery.
- In `decide_fencing`, a fencing failure also transitions to `FailSafe`.
- In `decide_fail_safe`, the engine stays conservative until either:
  - fencing is still running
  - a primary path can be re-evaluated safely
  - a stale self-leader lease needs release
  - or the node can go back to `WaitingDcsTrusted`

How recovery choices are selected:
- Rewind is preferred when there is an available primary member and the facts indicate rewind is required.
- Base backup is used as the fallback after rewind failure when a recovery leader can still be identified.
- Bootstrap appears when the node must initialize fresh state rather than catch up to an existing leader.

What process dispatch turns these effects into:
- `src/ha/process_dispatch.rs` maps lowered actions to concrete process jobs.
- Examples:
  - `StartPostgres` materializes managed config and requests a `StartPostgres` job.
  - `PromoteToPrimary` requests a `Promote` job.
  - `DemoteToReplica` requests a `Demote` job.
  - `StartRewind` requests a `PgRewind` job after validating the source member from DCS.
  - `StartBaseBackup` requests a `BaseBackup` job.
  - `RunBootstrap` requests a `Bootstrap` job.
  - `FenceNode` requests a fencing job with immediate shutdown mode.
- `FollowLeader` and `SignalFailSafe` do not dispatch a process job directly; they are effectively no-op at this dispatch layer.

Good explanatory angle:
- This page should explain the engine as a trust-gated state machine that values safety before availability.
- It should show that DCS trust, active leader evidence, PostgreSQL reachability, and process-job outcomes all feed phase transitions.
- It should avoid becoming a pure reference table, because the repo already has `ha-decisions` reference material for that level of detail.
