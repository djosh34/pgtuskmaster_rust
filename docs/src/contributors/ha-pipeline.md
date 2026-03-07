# HA Decision and Effect-Plan Pipeline

This chapter explains the current HA control loop in the code, not the idealized one you might expect from a generic leader-election system. The quickest way to build a correct mental model is to read one HA tick as four owned stages:

1. collect the latest world snapshot
2. choose the next `HaDecision`
3. lower that decision into a bucketed `HaEffectPlan`
4. publish `HaState`, then try to apply the plan

If you keep those stages separate while editing HA code, the rest of the module layout starts to make sense.

## Where to start in code

Open these files in this order:

- `src/ha/worker.rs`: the runtime orchestration around one HA tick
- `src/ha/decide.rs` and `src/ha/decision.rs`: pure decision inputs, facts, and domain decisions
- `src/ha/lower.rs`: conversion from decision to effect plan
- `src/ha/apply.rs`: deterministic bucket application and DCS coordination writes
- `src/ha/process_dispatch.rs`: process job construction plus managed-config/filesystem prep
- `src/ha/events.rs`: structured HA event payloads

## One HA tick, exactly

`src/ha/worker.rs::step_once(...)` is the canonical read of the current behavior.

The flow is:

1. `world_snapshot(ctx)` reads the latest config, pginfo, DCS, and process snapshots.
2. `decide(DecideInput { current, world })` returns the next phase plus a `HaDecision`.
3. `HaDecision::lower()` builds a `HaEffectPlan`.
4. The worker emits decision and plan events.
5. The worker publishes the next `HaState` with `WorkerStatus::Running`.
6. The worker applies the plan unless redundant process dispatch suppression says it should skip.
7. If any dispatches fail, the worker republishes the same state as `WorkerStatus::Faulted(...)`.

That publish-before-apply shape is important. The system records what HA selected even when the side effects fail, so downstream readers can see the intended phase and the fact that execution faulted.

## Inputs: what HA is allowed to believe

HA does not probe the world directly. It builds `DecisionFacts` from a `WorldSnapshot` in `src/ha/decision.rs`, using only:

- `PgInfoState` for local Postgres reachability and role evidence
- `DcsState` for leader, members, switchover request, and trust
- `ProcessState` for whether a relevant job is running, succeeded, or failed
- `RuntimeConfig` for member identity and behavior settings

That separation is the core design contract. If HA looks wrong, the bug usually lives in one of five places:

- observation is wrong (`pginfo`)
- the DCS cache or trust view is wrong (`dcs`)
- the decision logic is wrong (`decide.rs`)
- the effect application is wrong (`apply.rs` or `process_dispatch.rs`)
- the local process outcome is wrong (`process`)

Do not patch HA by sneaking in new direct probes from the worker loop. That breaks the shared truth model.

## Decisions and phases

The phase enum lives in `src/ha/state.rs`; the domain decisions live in `src/ha/decision.rs`; the actual transition rules live in `src/ha/decide.rs`.

The steady-state mental map is:

- `Init`, `WaitingPostgresReachable`, and `WaitingDcsTrusted` are convergence phases before the node is allowed to make stronger HA moves.
- `Replica`, `CandidateLeader`, and `Primary` are the normal role-control phases.
- `Rewinding` and `Bootstrapping` are recovery phases driven by process outcomes and leader availability.
- `Fencing` and `FailSafe` are protective phases when leadership evidence or trust is unsafe.

The most important type for understanding transition inputs is `DecisionFacts`. It already encodes the facts the decision logic is allowed to use, including `active_leader_member_id`, `available_primary_member_id`, `switchover_requested_by`, and `rewind_required`.

## From decision to plan

`src/ha/lower.rs` intentionally turns `HaDecision` into a fixed-shape plan:

- lease
- switchover
- replication
- postgres
- safety

That plan shape matters because it prevents the worker from building an ad hoc action vector on each branch. Contributors changing HA semantics should update the decision and the lowering layer together so the meaning stays explicit and testable.

## The real dispatch order

The code does not apply effect buckets in the same order they appear in the plan struct. `src/ha/apply.rs::apply_effect_plan(...)` currently dispatches them in this exact order:

1. Postgres
2. Lease
3. Switchover
4. Replication
5. Safety

That ordering is deliberate. It means a step-down can demote Postgres before releasing leadership, and switchover cleanup happens only after the lease work for the same tick.

If you change the order, you are changing HA semantics, not just refactoring.

## What each boundary owns

The side-effect split is strict:

- `src/ha/apply.rs` owns DCS coordination writes and the deterministic bucket walk.
- `src/ha/process_dispatch.rs` owns local process requests, managed Postgres config materialization, data-dir wiping, and leader-source resolution.
- `src/ha/events.rs` owns HA-specific structured event payloads so the orchestration code stays readable.

Some effects are intentionally no-ops at the process boundary:

- `FollowLeader` is represented in the decision and plan, but does not enqueue a local process job.
- `SignalFailSafe` is also represented explicitly even though the current process-dispatch layer skips it.

That explicitness is useful because the chosen plan still appears in logs and debug surfaces.

## Redundant dispatch suppression

The current implementation does have a narrow form of cross-tick suppression. `src/ha/worker.rs::should_skip_redundant_process_dispatch(...)` suppresses duplicate process dispatch only when the phase and decision are unchanged and the decision is one of:

- `WaitForPostgres { start_requested: true, .. }`
- `RecoverReplica { .. }`
- `FenceNode`

Everything else may be re-applied on later ticks if the same decision is still selected. That is why contributor docs should not claim broad "no cross-tick suppression" behavior.

## Contributor story paths

These are the fastest ways to reason about common HA changes.

### Election and promotion

Read `decide_candidate_leader(...)` and `decide_primary(...)` in `src/ha/decide.rs`.

The important contract is that leadership selection uses DCS trust plus leader/member evidence from the cache. Promotion is not a free-standing process action; it is a decision that becomes a `PostgresEffect::Promote` only after the worker already chose the new phase.

### Switchover

The end-to-end path is:

1. API writes `/{scope}/switchover`
2. DCS worker decodes and caches the request
3. HA sees `switchover_requested_by`
4. HA lowers the step-down decision into demote, lease release, and switchover clear

If you are changing switchover behavior, review `src/api/controller.rs`, `src/dcs/state.rs`, `src/ha/decide.rs`, `src/ha/lower.rs`, and `src/ha/apply.rs` together.

### Recovery

Read `recovery_after_rewind_failure(...)`, `follow_target(...)`, and the `RecoverReplica` lowering path. Recovery decisions depend on leader availability plus process outcomes; they are not only "leader exists or not". `process_dispatch.rs` is where those decisions become `pg_rewind`, `basebackup`, or bootstrap requests.

### Fencing and fail-safe

Read `decide_fencing(...)` and `decide_fail_safe(...)`. These phases are the protective edge of the state machine. Be especially careful with any change that weakens when lease release or fencing is requested.

## Failure behavior

There are two important failure modes:

- if HA cannot publish the chosen state, `step_once(...)` returns an error and the runtime can fail closed
- if effect application fails after publishing, the worker republishes the same state as `Faulted(...)`

That means a faulted HA snapshot still tells you which phase and decision were selected. It is not equivalent to "no decision".

## How to change HA safely

- Keep new cluster facts in `DecisionFacts` or upstream state, not hidden inside worker orchestration.
- Update decision tests and lowering tests together when you add or change a decision variant.
- Treat dispatch order changes as behavior changes that need explicit tests.
- Preserve the separation between DCS coordination writes and local process jobs.
- Verify the change in a real-binary HA scenario if it affects timing, fencing, promotion, or recovery.

## Adjacent subsystem connections

- Read [Worker Wiring and State Flow](./worker-wiring.md) for the upstream channels HA consumes and the downstream feedback loop from process outcomes.
- Read [API and Debug Contracts](./api-debug-contracts.md) for how operator intent enters the system and how HA state reaches clients.
- Read [Testing System Deep Dive](./testing-system.md) for which tests should prove a decision-layer change versus an apply-layer change.

## Evidence pointers

- `src/ha/worker.rs`
- `src/ha/decide.rs`
- `src/ha/decision.rs`
- `src/ha/lower.rs`
- `src/ha/apply.rs`
- `src/ha/process_dispatch.rs`
- `tests/ha_multi_node_failover.rs`
- `tests/ha_partition_recovery.rs`
