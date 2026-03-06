---
## Task: Rewrite HA decide into a facts-and-PhaseOutcome match machine <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
**Goal:** Replace mutation-driven HA decision code with a pure, match-based state machine that gathers immutable facts once and returns a full `PhaseOutcome { next_phase, decision }` directly from each phase handler.

**Scope:**
- Edit `src/ha/{decide,state,mod}.rs` and any new decision modules needed for the rewrite.
- Introduce an immutable facts struct gathered once per tick before phase selection.
- Replace top-level mutable `next` and mutable action accumulation with pure per-phase functions returning complete outcomes.
- Keep the decision layer at the domain level by returning a high-level `HaDecision` enum instead of low-level executable effects.

**Context from research:**
- PR #1 feedback on `src/ha/decide.rs` repeatedly calls out that the function should be pure, should use higher-level `match` structure, and should stop mutating shared state.
- Current code starts with `let mut next` / `let mut candidates` and threads `&mut HaState` / `&mut Vec<_>` into helpers even though the logic is deterministic.
- We agreed on a concrete target shape:
  - gather immutable `DecisionFacts`
  - `match` on `HaPhase`
  - return a full `PhaseOutcome { next_phase, decision }`
  - no mutation as the language of decision making
- We also agreed that the decision layer should describe the chosen HA outcome in domain language, not immediately collapse into tiny imperative effects.
- We agreed on the boundary rule:
  - `DecisionFacts` are only for deciding
  - `HaDecision` must carry enough payload that later lowering does not need the original world facts again
- We also agreed that `decide` should stop mutating stateful error placeholders such as `last_error`; decision failures or exceptional outcomes should be represented through typed results/contracts rather than hidden mutable state.
- The intended first-pass signature shape should be treated as the default target unless implementation uncovers a concrete reason to improve it:
  - `fn decide_phase(current: HaPhase, facts: &DecisionFacts) -> PhaseOutcome`
  - `struct PhaseOutcome { next_phase: HaPhase, decision: HaDecision }`
- The intended first-pass `HaDecision` family should be explicit enough to keep the decision layer at domain level. A close equivalent is expected, not a weaker bag-of-actions rename:
  - `NoChange`
  - `WaitForPostgres`
  - `WaitForDcsTrust`
  - `AttemptLeadership`
  - `FollowLeader { leader }`
  - `BecomePrimary`
  - `StepDown { reason }`
  - `RecoverReplica { strategy }`
  - `FenceNode`
  - `EnterFailSafe`
- The goal is functional programming in structure, not just fewer `mut` tokens.

**Expected outcome:**
- `decide` becomes a thin pure coordinator.
- Each HA phase handler is a pure function returning a complete outcome for that phase.
- The returned decision is a high-level `HaDecision` value that can later be lowered into smaller executable effects.
- The control flow becomes readable enough that correctness is argued from type shape and branch structure, not from reading through mutable accumulation.

**Story test policy:**
- The original story note deferred `make test-long`, but final task completion followed the outer Ralph contract and required a green `make test-long` before closing the task.
- Known long-test instability was therefore not treated as deferrable for this completion pass.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] Modify `src/ha/decide.rs`, `src/ha/state.rs`, `src/ha/mod.rs`, and any new `src/ha/decision*.rs` modules introduced by the rewrite.
- [x] Introduce an immutable facts struct that is computed once from `DecideInput` and then passed by shared reference into phase decision functions.
- [x] Introduce a `PhaseOutcome` type returned directly by per-phase pure handlers.
- [x] Introduce a high-level `HaDecision` enum carried by `PhaseOutcome` so the decision layer returns domain outcomes rather than low-level executable effects.
- [x] Implement or closely match the explicit first-pass decision signature shape:
  - `fn decide_phase(current: HaPhase, facts: &DecisionFacts) -> PhaseOutcome`
  - `struct PhaseOutcome { next_phase: HaPhase, decision: HaDecision }`
- [x] Define a domain-level `HaDecision` family at least equivalent in strength to:
  - `NoChange`
  - `WaitForPostgres`
  - `WaitForDcsTrust`
  - `AttemptLeadership`
  - `FollowLeader { leader }`
  - `BecomePrimary`
  - `StepDown { reason }`
  - `RecoverReplica { strategy }`
  - `FenceNode`
  - `EnterFailSafe`
- [x] Ensure `HaDecision` variants carry enough payload for later lowering without requiring `DecisionFacts` or `WorldSnapshot` to be passed again.
- [x] Remove the production pattern of mutable `next` state plus mutable `candidates` accumulation from `decide`.
- [x] Remove all production decision signatures that accept `&mut HaState` or `&mut Vec<_>`.
- [x] Remove mutable `last_error`-style decision bookkeeping and replace it with typed return/error handling that fits the pure decision contract.
- [x] Use match-driven phase dispatch as the primary control-flow shape for the HA state machine.
- [x] Keep `decide` side-effect free and deterministic from `DecideInput` alone.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] `make test-long` — passes cleanly; the outer task-runner completion contract overrode the earlier story note deferring long-test validation to task `06-task-move-and-split-ha-e2e-tests-after-functional-rewrite.md`
</acceptance_criteria>

<plan>
## Execution plan (draft 2026-03-06)

1. Lock the architectural boundary before changing code.
   - Treat `src/ha/decide.rs` as the pure domain-decision layer and keep `src/ha/worker.rs` as the lowering and dispatch layer.
   - Do not keep a hidden bag-of-actions under a new name. Each phase handler must return one complete domain outcome, and any place that currently needs multiple `HaAction`s must be modeled as one richer `HaDecision` carrying the payload needed for later lowering.
   - Treat the current `HaState.pending: Vec<HaAction>` contract as expected to change; continuing to publish only low-level pending actions would leave observability and docs describing the old architecture.
   - Preserve existing runtime behavior where possible, especially action ordering for composite cases such as switchover, fencing, and recovery, unless the refactor reveals a correctness bug that should be fixed as part of the rewrite.

2. Introduce explicit decision-domain types first so the refactor has a stable center.
   - Add a new HA decision module, preferably `src/ha/decision.rs`, and export it from `src/ha/mod.rs`.
   - Move the new pure decision model into that module:
     - `DecisionFacts`
     - `PhaseOutcome`
     - `HaDecision`
     - helper enums such as `RecoveryStrategy`, `StepDownReason`, or equivalent domain payload types
   - Add a separate lowering module, preferably `src/ha/lower.rs`, rather than burying the lowering table inside the already-large worker. The skeptical review conclusion is that keeping decision selection, decision lowering, and side-effect dispatch in three distinct modules will make the post-rewrite shape auditable and easier to test.
   - `HaDecision` must cover every current branch without falling back to `Vec<HaAction>`. In particular:
     - no-op / steady-state branches
     - waiting for Postgres
     - waiting for trusted DCS
     - leadership attempt
     - following a concrete leader member
     - becoming primary
     - stepping down with enough reason/payload to lower into demotion plus any required DCS cleanup
     - replica recovery path selection (`rewind`, `basebackup`, `bootstrap`) with any required target-leader payload
     - fencing
     - fail-safe entry with any required release-lease semantics

3. Gather immutable facts once per tick and stop recomputing branch predicates inside handlers.
   - Replace the current local-variable bundle at the top of `decide` with a dedicated `DecisionFacts::from_input(&DecideInput)` or equivalent pure constructor.
   - `DecisionFacts` should own or precompute every branch predicate currently spread across `decide.rs`, including at least:
     - self member id
     - current leader record and whether that leader is available
     - active leader id if any
     - DCS trust level
     - switchover presence
     - Postgres reachability
     - whether this node currently holds leadership
     - whether another leader record exists
     - whether another available leader exists
     - timeline/rewind facts needed to decide whether following can proceed directly or requires rewind
     - process-state facts needed for `Rewinding`, `Bootstrapping`, and `Fencing`
   - The rule for the rewrite is that phase handlers may inspect only `HaPhase` plus `DecisionFacts`; they should not reach back into `WorldSnapshot` directly.

4. Reshape the HA state contract around decisions instead of action vectors.
   - In `src/ha/state.rs`, replace the old `DecideOutput { next, actions }` shape with a pure output that keeps the domain decision explicit.
   - Update `HaState` to record the selected domain decision for the current tick instead of `pending: Vec<HaAction>`. A name like `decision` or `last_decision` is preferable to `pending`.
   - Keep `tick` and `phase`, but ensure the published state after each step is understandable in domain terms without reading low-level dispatch logs.
   - Do not add a second stored `pending_action_count` or stored lowered-actions cache to `HaState`. If a consumer still wants a count, derive it from `lower::lower_decision(&state.decision)` at the presentation boundary so the source of truth stays singular.
   - Update contract stubs, initial state constructors, and any helper fixtures that still seed `pending` actions.

5. Rewrite the decision machine as pure per-phase handlers with a match-first structure.
   - Make `src/ha/decide.rs` a thin coordinator:
     - build `DecisionFacts`
     - dispatch by `match current.phase`
     - return a `PhaseOutcome`
     - build the next `HaState` from the returned outcome without mutating shared accumulators
   - Split the current large phase arms into dedicated pure functions such as `decide_init`, `decide_waiting_postgres`, `decide_replica`, `decide_primary`, and so on.
   - Remove all uses of `let mut next`, `let mut candidates`, and `dedupe_actions_per_tick`; there should be no per-tick action accumulation in the decision layer after the rewrite.
   - Eliminate any decision signatures that still accept `&mut HaState` or `&mut Vec<_>`.
   - Ensure multi-effect branches are represented as a single domain decision:
     - primary switchover should become one step-down decision with enough payload to demote, release lease, and clear switchover during lowering
     - conflicting leader while primary should become one fencing-oriented decision with enough payload to demote, release lease, and fence
     - rewind/basebackup/bootstrap selection should become one recovery decision carrying the chosen recovery strategy and any needed leader identity
     - loss of quorum should become one fail-safe decision carrying whether the lease must be released

6. Keep executable lowering explicit and outside the pure decision code.
   - Add a pure lowering function in `src/ha/lower.rs` that converts `&HaDecision` into an ordered `Vec<HaAction>`.
   - Preserve the current executable sequencing when lowering composite decisions:
     - switchover: demote, release lease, clear switchover
     - conflicting leader fencing: demote, release lease, fence
     - recovery: wipe first when required, then run the selected recovery action
     - fail-safe while primary: release lease before signaling fail-safe
   - Keep `dispatch_actions` action-based for now; this task is about purifying decision selection, not rewriting the process/DCS dispatch layer.
   - Do not require `DecisionFacts` or `WorldSnapshot` during lowering. If lowering needs data, add that payload directly to `HaDecision`.

7. Update the worker to publish domain decisions while still dispatching executable actions.
   - In `src/ha/worker.rs`, change `step_once` to:
     - call the pure decision function
     - emit/log the chosen domain decision
     - lower that decision into actions
     - dispatch the lowered actions
     - publish the updated `HaState` containing the selected decision and worker status
   - Review existing HA logs and helper functions so the main observability message names the domain decision, not just action ids.
   - Keep dispatch error handling typed; do not reintroduce mutable “last error” placeholders in the decision layer.

8. Update downstream state consumers that currently assume `pending` means the HA contract.
   - Compile errors are expected in:
     - `src/api/{mod,controller,worker}.rs`
     - `src/cli/{client,output}.rs`
     - `src/debug_api/{view,worker}.rs`
     - `src/runtime/node.rs`
     - `src/worker_contract_tests.rs`
   - Replace action-count-only exposure with decision-centric exposure. Prefer surfacing the chosen `HaDecision` label or summary over keeping `pending_actions` as the primary API/debug contract.
   - Introduce a small presentation helper for decision summaries so API/CLI/debug output does not stringify raw Rust `Debug` output from `HaDecision`. The skeptical review conclusion is that state publication should expose stable decision labels, while any lowered-action count remains a derived debug convenience.
   - If a count of lowered actions remains useful in debug output, derive it from lowering at the consumer boundary rather than storing a low-level action vector in `HaState`.
   - Update any docs or snapshot assertions that still explain HA as “phase + tick + pending actions”.

9. Rewrite tests around the new contract instead of preserving the old assertions mechanically.
   - In `src/ha/decide.rs` tests, assert phase plus `HaDecision` outcomes directly for the existing table-driven cases.
   - Add focused lowering tests to prove each composite domain decision lowers into the correct ordered `HaAction` sequence.
   - Update `src/ha/worker.rs` tests to assert the published HA state carries the expected decision and that dispatch still sees the expected lowered actions.
   - Update contract/API/debug/CLI tests that currently check `pending_actions` counts or `pending` contents.
   - Keep tests free of `unwrap`, `expect`, or panic-shaped shortcuts; use typed `Result` returns and explicit assertions.

10. Execute in parallel only after the shared type skeleton lands.
   - First land the shared decision types and `HaState` contract locally because the rest of the tree depends on those names.
   - Then split follow-on work into parallel tracks:
     - Track A: pure `decide.rs` phase handlers plus decision tests
     - Track B: worker lowering and dispatch/logging updates
     - Track C: API/CLI/debug/docs/contract-test updates for the new state contract
   - Reconcile all tracks before final validation so the final tree presents one coherent decision vocabulary.

11. Update documentation for the new architecture before final closeout.
   - At minimum review and update:
     - `docs/src/contributors/ha-pipeline.md`
     - `docs/src/contributors/api-debug-contracts.md`
     - `docs/src/contributors/worker-wiring.md`
     - `docs/src/interfaces/node-api.md`
     - `docs/src/interfaces/cli.md`
   - Remove or rewrite any stale explanation that says HA decides by accumulating pending low-level actions in the decision layer.
   - If API/debug output names change from `pending_actions` to a decision-oriented field, document the new response shape in the interface docs the same turn.

12. Run the full validation set required for actual task completion, then do Ralph bookkeeping.
   - During implementation, use targeted tests first to stabilize the new types and the worker lowering path.
   - Before marking the task done, run the final gates required by the outer Ralph instructions:
     - `make check`
     - `make test`
     - `make test-long`
     - `make lint`
   - The story text currently says long-test validation is deferred, but the surrounding task-runner instructions for completion require the full gate set. Do not set `<passing>true</passing>` or close the task until that policy conflict is resolved in favor of an actually green final tree.
   - After all gates pass: tick the checklist, set `<passing>true</passing>`, run `/bin/bash .ralph/task_switch.sh`, commit all tracked changes including `.ralph`, and push.

## Specific branch mappings that must survive the refactor

- `Init` should only advance into `WaitingPostgresReachable`; it should not emit executable work directly.
- `WaitingPostgresReachable` should choose between “still waiting for Postgres” and “advance to DCS-trust waiting”.
- `WaitingDcsTrusted`, `Replica`, and `CandidateLeader` must all derive their leader/follow/promote behavior from the precomputed leader facts rather than recomputing raw world access in each arm.
- `Primary` must continue to distinguish:
  - switchover-requested step-down
  - local Postgres failure leading into recovery/rewind flow
  - conflicting leader record leading into fencing
- `Rewinding`, `Bootstrapping`, and `Fencing` must keep their current process-outcome-driven transitions, but expressed through typed recovery/fencing/fail-safe decisions rather than action accumulation.
- `FailSafe` should remain a real phase with an explicit exit back to `WaitingDcsTrusted` once quorum is restored.

NOW EXECUTE
</plan>
