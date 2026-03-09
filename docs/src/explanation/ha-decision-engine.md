# HA Decision Engine

The HA decision engine turns a world snapshot into a single next HA phase and a single HA decision. Its structure is deliberately layered:

1. derive `DecisionFacts` from the current world
2. choose the next phase and semantic decision
3. lower that decision into effect buckets
4. dispatch only the effects that need concrete process work

## Trust Gates Everything

The first branch in `decide_phase(...)` is the trust gate.

If trust is not `FreshQuorum`:

- a local primary enters `FailSafe` with `EnterFailSafe { release_leader_lease: false }`
- a non-primary also enters the `FailSafe` phase, but with `NoChange`

That means the engine prefers safety over recovery whenever its view of cluster coordination is degraded.

`DcsTrust` has three variants:

- `FreshQuorum`
- `NoFreshQuorum`
- `NotTrusted`

`FreshQuorum` is the only trust state that allows the normal phase handlers to run.

## Phase-Driven Logic

The decision engine handles these phases:

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

Each phase handler answers a narrow question:

- should the node wait
- should it try to lead
- should it follow a leader
- should it recover
- should it demote or fence

For switchovers, the decisive fact is only whether a request is pending. The engine does not consume a caller-supplied successor identity; successor choice is derived from observed cluster state.

The engine does not jump directly to process jobs. It first emits a semantic `HaDecision`.

## Decision Taxonomy

The decision enum includes:

- `NoChange`
- `WaitForPostgres`
- `WaitForDcsTrust`
- `AttemptLeadership`
- `FollowLeader`
- `BecomePrimary`
- `StepDown`
- `RecoverReplica`
- `FenceNode`
- `ReleaseLeaderLease`
- `EnterFailSafe`

Some decisions are mostly steady-state or observational:

- `NoChange`
- `WaitForDcsTrust`
- `FollowLeader`

Some decisions can repeat while the phase machine waits for progress:

- `AttemptLeadership`
- `WaitForPostgres`

Some decisions are clearly invasive because they lower into process or safety work:

- `BecomePrimary` when promotion is needed
- `StepDown`
- `RecoverReplica`
- `FenceNode`
- `ReleaseLeaderLease`
- `EnterFailSafe`

That idempotent-versus-invasive split is an inference from the lowering and dispatch behavior, not a separate public contract.

## Lowering Into Effect Buckets

The lowerer turns a semantic decision into a `HaEffectPlan` with five independent buckets:

- lease
- switchover
- replication
- postgres
- safety

Examples:

- `AttemptLeadership` lowers to lease acquisition
- `FollowLeader` lowers to replication follow behavior
- `BecomePrimary { promote: true }` lowers to PostgreSQL promotion
- `StepDown(...)` lowers to some combination of demotion, lease release, switchover clearing, and fencing
- `EnterFailSafe { release_leader_lease: true }` lowers to lease release plus fencing

This layer is what prevents contradictory action mixes from being dispatched as one HA choice.

## When Recovery Is Chosen

Recovery paths only make sense once trust is good enough to rely on DCS-backed membership and leader information.

Under `FreshQuorum`, the engine can choose recovery behaviors such as:

- `RecoverReplica { strategy: Rewind { ... } }`
- `RecoverReplica { strategy: BaseBackup { ... } }`
- `RecoverReplica { strategy: Bootstrap }`

The requested source files show these broad patterns:

- rewind is attempted when a usable recovery leader exists and rewind is required
- base backup is used as a fallback after rewind failure when a usable leader still exists
- trust degradation interrupts ordinary recovery and routes back through `FailSafe`

## Process Dispatch Boundaries

The process dispatcher only handles actions that require concrete process jobs.

It turns lowered HA actions into jobs such as:

- `StartPostgres`
- `Promote`
- `Demote`
- `PgRewind`
- `BaseBackup`
- `Bootstrap`
- `Fencing`

At the same time, the dispatcher explicitly does not treat every HA effect as a process job:

- lease and switchover actions are not process actions there
- `SignalFailSafe` is skipped at the process-dispatch layer

`FollowLeader` has a steady-state fast path and an invasive correction path. If the local replica already reports the authoritative upstream, dispatch skips the action. If the replica is still pointed at an old leader, dispatch rewrites managed recovery config for the authoritative leader and queues a demote so the ordinary wait/start or recovery path can bring PostgreSQL back following the corrected primary.

That separation is important: the decision engine describes intent, the lowerer organizes effects, and only part of that plan becomes spawned process work.

## Why This Design Matters

This architecture gives the runtime three useful properties:

- trust-based safety decisions are centralized
- phase transitions are explicit and testable
- invasive work is separated from the semantic decision that requested it

For operators, that explains why the HA API exposes both a phase and a decision: one tells you where the node is in the state machine, and the other tells you what the node wants to do next.
