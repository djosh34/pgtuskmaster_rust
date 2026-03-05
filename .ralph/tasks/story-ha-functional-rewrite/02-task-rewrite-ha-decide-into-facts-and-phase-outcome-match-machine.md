---
## Task: Rewrite HA decide into a facts-and-PhaseOutcome match machine <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Replace mutation-driven HA decision code with a pure, match-based state machine that gathers immutable facts once and returns a full `PhaseOutcome` directly from each phase handler.

**Scope:**
- Edit `src/ha/{decide,state,mod}.rs` and any new decision modules needed for the rewrite.
- Introduce an immutable facts struct gathered once per tick before phase selection.
- Replace top-level mutable `next` and mutable action accumulation with pure per-phase functions returning complete outcomes.

**Context from research:**
- PR #1 feedback on `src/ha/decide.rs` repeatedly calls out that the function should be pure, should use higher-level `match` structure, and should stop mutating shared state.
- Current code starts with `let mut next` / `let mut candidates` and threads `&mut HaState` / `&mut Vec<_>` into helpers even though the logic is deterministic.
- We agreed on a concrete target shape:
  - gather immutable `DecisionFacts`
  - `match` on `HaPhase`
  - return a full `PhaseOutcome`
  - no mutation as the language of decision making
- The goal is functional programming in structure, not just fewer `mut` tokens.

**Expected outcome:**
- `decide` becomes a thin pure coordinator.
- Each HA phase handler is a pure function returning a complete outcome for that phase.
- The control flow becomes readable enough that correctness is argued from type shape and branch structure, not from reading through mutable accumulation.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Modify `src/ha/decide.rs`, `src/ha/state.rs`, `src/ha/mod.rs`, and any new `src/ha/decision*.rs` modules introduced by the rewrite.
- [ ] Introduce an immutable facts struct that is computed once from `DecideInput` and then passed by shared reference into phase decision functions.
- [ ] Introduce a `PhaseOutcome` type returned directly by per-phase pure handlers.
- [ ] Remove the production pattern of mutable `next` state plus mutable `candidates` accumulation from `decide`.
- [ ] Remove all production decision signatures that accept `&mut HaState` or `&mut Vec<_>`.
- [ ] Use match-driven phase dispatch as the primary control-flow shape for the HA state machine.
- [ ] Keep `decide` side-effect free and deterministic from `DecideInput` alone.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
</acceptance_criteria>
