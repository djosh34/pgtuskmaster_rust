# How the HA Decision Engine Works

The HA decision engine is the centralized resolver that turns cluster observations into concrete actions. Instead of scattering if-then logic across multiple modules, the engine gathers facts about PostgreSQL, the distributed consensus store (DCS), and running processes, then selects a single next state and decision. This design keeps safety checks in one place and makes the failover logic explicit and testable.

## Core Flow: World to Action

The engine runs every tick. It takes a `WorldSnapshot`—a consistent view of configuration, PostgreSQL state, DCS state, and process activity—and produces a `HaState` that names the next phase and a `HaDecision` that names the intent.

[diagram about ha decision flow, showing world snapshot feeding into decide function which outputs phase and decision, then lower function which converts decision to effects, then dispatch which turns effects into process jobs, **diagram shows trust gate at start of decide function**]

The `decide` function performs three steps:
1. Build `DecisionFacts` from the world snapshot
2. Run the phase-specific logic to pick the next phase and decision
3. Return the next state

After deciding, the system "lowers" the abstract decision into a concrete `HaEffectPlan`. Lowering splits the decision into five buckets: lease management, switchover cleanup, replication recovery, PostgreSQL state changes, and safety actions. The process dispatcher then turns effects into actual jobs like `pg_rewind` or `promote`.

## Trust-First Safety

The engine's first question is not "who should be leader?" but "do I trust the DCS?" Every decision path starts with a trust check.

[diagram about trust states, showing DcsTrust enum with FullQuorum, FailSafe, NotTrusted, and how each routes decisions, **diagram shows FullQuorum allowing normal phase flow, FailSafe entering conservative mode, NotTrusted blocking all leadership actions**]

`DcsTrust` has three states:
- **FullQuorum**: The DCS is healthy and enough members are fresh. Normal operation proceeds.
- **FailSafe**: The DCS is healthy but member data is stale or incomplete. The engine avoids disruptive actions.
- **NotTrusted**: The DCS store is unreachable or broken. The engine cannot safely read or write leader leases.

If trust is not `FullQuorum`, the engine immediately routes to `FailSafe` phase. If the local PostgreSQL is primary in this state, it emits `EnterFailSafe { release_leader_lease: false }` to fence the node. If PostgreSQL is not primary, it emits `NoChange` and waits. This split-brain prevention is the engine's top priority: safety before availability.

## Phase-Driven Decision Making

Each of the eleven phases represents a stable "mode" the engine can be in. The engine never jumps arbitrarily between phases; it follows a directed graph where each phase's decision logic determines the next.

[diagram about phase transitions, showing Init -> WaitingPostgresReachable -> WaitingDcsTrusted -> Replica/CandidateLeader/Primary branches, with FailSafe as a central hub that many phases can enter, **diagram shows trust failures always leading to FailSafe, and recovery after fencing leading back into WaitingDcsTrusted**]

Key phase behaviors:
- **Init**: Always moves to waiting for PostgreSQL
- **WaitingPostgresReachable**: Stays until PostgreSQL is reachable or a start job finishes
- **WaitingDcsTrusted**: Stays until trust is full and a leader is clear
- **CandidateLeader**: Attempts to acquire lease and become primary
- **Primary**: Holds the lease and monitors for foreign leaders or switchover requests
- **Replica**: Follows a known leader and watches for rewind needs
- **Rewinding/Bootstrapping**: Executes recovery jobs, then transitions based on job outcome
- **Fencing**: Terminates PostgreSQL to break split-brain, then releases lease
- **FailSafe**: Conservative hold phase when trust is degraded

## Decision Taxonomy and Intent

Decisions are named by intent, not by mechanism. This separation lets the engine express what it wants to happen without coupling to implementation details.

| Decision | Intent | Idempotent |
|----------|--------|------------|
| `NoChange` | Wait for next tick | Yes |
| `WaitForPostgres` | Start PostgreSQL if needed | Yes (start is guarded) |
| `WaitForDcsTrust` | Do nothing until DCS is reliable | Yes |
| `AttemptLeadership` | Try to become leader | Yes (acquire is atomic) |
| `FollowLeader` | Replicate from a specific member | Yes (steady state) |
| `BecomePrimary` | Promote PostgreSQL or confirm primary state | No (state-changing when `promote=true`) |
| `StepDown` | Demote and maybe fence | No (destructive) |
| `RecoverReplica` | Run rewind or basebackup | No (destructive) |
| `FenceNode` | Shutdown PostgreSQL immediately | No (destructive) |
| `ReleaseLeaderLease` | Surrender leadership claim | No (state change) |
| `EnterFailSafe` | Enter safety mode | No (disables normal operation) |

Idempotent decisions can be emitted repeatedly without harm. The engine reissues them while conditions persist. More invasive decisions lower into process jobs that modify data directories, promote instances, or terminate processes; these are dispatched once and guarded by the phase machine. The idempotency characterization here derives from observable behaviors in the lowering and dispatch mechanisms.

## Lowering: From Intent to Effect Plan

Lowering is the bridge between high-level decisions and low-level actions. It partitions a decision into five independent effect categories, allowing the dispatcher to handle each safely.

[diagram about lowering flow, showing HaDecision on left, HaEffectPlan in middle with five buckets (lease, switchover, replication, postgres, safety), and concrete actions like AcquireLeader, Promote, FenceNode on right, **diagram shows one-to-many relationship between decision and effects**]

Examples:
- `AttemptLeadership` → lease: `AcquireLeader`. All other buckets empty.
- `FollowLeader { leader_member_id }` → replication: `FollowLeader { ... }`
- `BecomePrimary { promote: true }` → postgres: `Promote`
- `StepDown(Plan { release_leader_lease: true, fence: false })` → lease: `ReleaseLeader`, postgres: `Demote`
- `EnterFailSafe { release_leader_lease: true }` → lease: `ReleaseLeader`, safety: `FenceNode`

This bucketing prevents contradictory actions. The lowering logic ensures you cannot simultaneously `Promote` and `FollowLeader`, or `AcquireLeader` while `Demote` runs.

## Process Dispatch: Effects Become Jobs

The `process_dispatch` module takes effects and creates concrete job requests. Not every effect generates a job; some are handled by other workers.

[diagram about dispatch mapping, showing PostgreSQL effect mapping to Start/Promote/Demote jobs, replication effect mapping to PgRewind/BaseBackup jobs, safety effect mapping to Fencing job, and lease/switchover effects being skipped, **diagram shows FollowLeader and SignalFailSafe as no-ops at this layer**]

Dispatch highlights:
- `StartPostgres` materializes a managed configuration based on intended role (primary or replica with conninfo)
- `PromoteToPrimary` and `DemoteToReplica` send simple promote/demote jobs
- `StartRewind` and `StartBaseBackup` resolve the target member from DCS and construct connection specs
- `FenceNode` uses immediate shutdown to break split-brain
- `FollowLeader` and `SignalFailSafe` do nothing at this layer; they are signals for the HA loop, not process actions

## When FailSafe Precedes Recovery

Normal recovery logic only runs under `FullQuorum` trust. If trust is degraded, the engine chooses `FailSafe` even when a recovery leader is visible. This is deliberate: the engine will not initiate a basebackup or rewind when it cannot reliably read the leader lease or member health.

After fencing completes, trust may return to `FullQuorum`. At that point the engine exits `FailSafe` and evaluates recovery afresh. This ordering ensures recovery actions never run with a stale or contested view of cluster membership.

## Recovery Strategy Selection

The engine chooses recovery strategies based on available state and process outcomes. When a primary member exists in DCS and local timeline comparison indicates divergence, rewind is typically preferred. When rewind is unavailable or fails, basebackup provides a fallback.

The key decision factors are:
- `available_primary_member_id`: a healthy member recorded in DCS
- `rewind_required`: true when rewind predicate evaluates based on timeline comparison
- Process activity: a rewind job failure triggers fallback to basebackup

This approach keeps recovery efficient while guaranteeing progress even after multiple failures.
