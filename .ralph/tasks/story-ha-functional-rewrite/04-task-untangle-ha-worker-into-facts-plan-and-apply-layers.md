---
## Task: Untangle HA worker into facts, plan, and apply layers <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Restructure HA runtime code so the worker clearly separates fact collection, pure decision selection, effect lowering, and effect application without forcing the design into object-heavy “executor” patterns.

**Scope:**
- Edit `src/ha/worker.rs`, `src/ha/mod.rs`, and any new helper modules needed to separate plan application by concern.
- Keep the worker as a thin orchestrator while moving low-level DCS path handling, process request assembly, filesystem mutations, and event payload construction into clearer domain helpers.
- Make the worker responsible for logging the chosen `HaDecision`, lowering it into executable effects, and then applying the lowered plan.
- Preserve the existing DCS worker and pginfo worker responsibilities while moving stray logic back to the layer where it belongs.

**Context from research:**
- We agreed the main need is to untangle the architecture, not necessarily to introduce heavyweight executor objects.
- `src/ha/worker.rs` currently mixes:
  - state snapshot collection
  - pure decision invocation
  - DCS path formatting and writes
  - process job request construction
  - filesystem mutation helpers
  - event/log attribute map construction
- We also discussed that runtime pollers already exist for DCS and pg state ingestion; what is missing is a clean separation so HA applies a plan instead of carrying random low-level concerns.

**Expected outcome:**
- HA worker code reads like:
  - collect facts
  - choose decision
  - lower decision
  - apply plan
  - publish state
- Non-HA concerns are pushed behind small typed helpers grouped by domain instead of being interleaved in one large function/file.
- The codebase becomes easier to continue migrating toward a functional style because the imperative edge is isolated.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Modify `src/ha/worker.rs`, `src/ha/mod.rs`, and any new helper modules introduced for plan-application untangling.
- [ ] Keep `step_once` focused on orchestration rather than low-level implementation details.
- [ ] Make `step_once` consume the high-level `HaDecision`, log it, call `HaDecision::lower()`, and apply the lowered effect plan without re-embedding planning logic in the worker.
- [ ] Move DCS path knowledge and raw DCS write/delete mechanics out of the main HA worker control flow.
- [ ] Move process request construction out of the main HA worker control flow into typed helpers or modules.
- [ ] Move filesystem mutation helpers out of the main HA worker control flow into the correct domain layer.
- [ ] Keep existing DCS/pginfo polling responsibilities intact and move any half-misplaced logic to the correct module rather than duplicating polling concepts inside HA tests/runtime code.
- [ ] Update worker tests to reflect the new facts/plan/apply split and verify the new helper boundaries.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
</acceptance_criteria>
