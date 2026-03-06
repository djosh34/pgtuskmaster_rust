---
## Task: Rewrite HA decide into a facts-and-PhaseOutcome match machine <status>not_started</status> <passes>false</passes>

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
- Skip `make test-long` and any direct long HA cargo-test invocations in this task.
- Known long-test failures are deferred until the final story task after the rewrite story lands.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Modify `src/ha/decide.rs`, `src/ha/state.rs`, `src/ha/mod.rs`, and any new `src/ha/decision*.rs` modules introduced by the rewrite.
- [ ] Introduce an immutable facts struct that is computed once from `DecideInput` and then passed by shared reference into phase decision functions.
- [ ] Introduce a `PhaseOutcome` type returned directly by per-phase pure handlers.
- [ ] Introduce a high-level `HaDecision` enum carried by `PhaseOutcome` so the decision layer returns domain outcomes rather than low-level executable effects.
- [ ] Implement or closely match the explicit first-pass decision signature shape:
  - `fn decide_phase(current: HaPhase, facts: &DecisionFacts) -> PhaseOutcome`
  - `struct PhaseOutcome { next_phase: HaPhase, decision: HaDecision }`
- [ ] Define a domain-level `HaDecision` family at least equivalent in strength to:
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
- [ ] Ensure `HaDecision` variants carry enough payload for later lowering without requiring `DecisionFacts` or `WorldSnapshot` to be passed again.
- [ ] Remove the production pattern of mutable `next` state plus mutable `candidates` accumulation from `decide`.
- [ ] Remove all production decision signatures that accept `&mut HaState` or `&mut Vec<_>`.
- [ ] Remove mutable `last_error`-style decision bookkeeping and replace it with typed return/error handling that fits the pure decision contract.
- [ ] Use match-driven phase dispatch as the primary control-flow shape for the HA state machine.
- [ ] Keep `decide` side-effect free and deterministic from `DecideInput` alone.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] Explicitly skip `make test-long` and direct long HA cargo-test invocations in this task; long-test validation is deferred to task `06-task-move-and-split-ha-e2e-tests-after-functional-rewrite.md`
</acceptance_criteria>
