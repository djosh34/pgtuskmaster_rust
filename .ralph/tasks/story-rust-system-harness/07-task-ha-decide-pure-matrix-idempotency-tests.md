---
## Task: Implement pure HA decide engine with exhaustive transition tests <status>not_started</status> <passes>false</passes> <priority>ultra_high</priority>

<blocked_by>03-task-worker-state-models-and-context-contracts</blocked_by>

<description>
**Goal:** Implement deterministic HA decision logic as a pure function with exhaustive matrix coverage.

**Scope:**
- Implement `src/ha/state.rs`, `src/ha/decide.rs`, `src/ha/actions.rs`, `src/ha/mod.rs`.
- Implement `DecideInput`, `DecideOutput`, `decide`, and action-id dedupe semantics.
- Cover transitions across `HaPhase` including `FailSafe`, `Rewinding`, `Bootstrapping`, and `Fencing`.

**Context from research:**
- Plan marks `ha::decide` exhaustiveness as critical and requires deterministic behavior.

**Expected outcome:**
- `decide` is side-effect free, reproducible, and fully unit-tested with matrix fixtures.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] `decide` uses only `DecideInput` and returns `DecideOutput` (no hidden IO).
- [ ] Tests cover normal paths plus split-brain prevention and no-quorum fail-safe transitions.
- [ ] Action idempotency tests verify duplicate-action suppression via `recent_action_ids`.
- [ ] Run targeted `ha::decide` matrix tests.
- [ ] Run `make check`.
- [ ] Run `make test`.
- [ ] Run `make lint`.
- [ ] Run `make test-bdd`.
- [ ] On failures, use `$add-bug` and include failing state snapshot fixture.
</acceptance_criteria>
