---
## Task: Replace action vectors and pending state with a typed domain effect plan <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Replace `Vec<HaAction>` planning with a typed domain-level effect plan so the HA state machine describes intent structurally instead of appending low-level commands into a vector.

**Scope:**
- Edit `src/ha/{actions,state,decide,worker,mod}.rs`, `src/api/{mod,controller}.rs`, `src/cli/{client,output}.rs`, and all affected tests.
- Replace `Vec<HaAction>`-style planning with a typed plan structure such as `HaEffectPlan` / `HaEffects` composed of orthogonal domain slots.
- Remove `pending` from `HaState` and eliminate `pending_actions` from state/API surfaces unless retained only as derived debug telemetry outside core HA state.
- Replace stringly typed HA phase/trust API exposure with typed serialized enums wherever HA state is surfaced externally.

**Context from research:**
- We discussed that “no more vec” is part of the real design goal: domain-level effects should be first-class, not just a deduped bag of low-level commands.
- A plain vector makes contradictory effects and duplicate cleanups too easy to represent, which is why the current code needs post-hoc dedupe.
- `pending` in `HaState` is not true machine state; it is the previous plan leaking execution details into persisted HA state.
- API/CLI currently stringify `ha_phase` and `dcs_trust`, which weakens type safety in the exact surfaces we use for tests and operations.

**Expected outcome:**
- The decision layer returns a typed effect plan whose shape prevents contradictions by construction as much as practical.
- `pending` is removed from core HA state.
- HA API/CLI surfaces use typed enums and stop normalizing the machine into string comparisons.
- Dedupe becomes unnecessary or dramatically smaller because the plan model is structured, not append-only.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Modify `src/ha/actions.rs`, `src/ha/state.rs`, `src/ha/decide.rs`, `src/ha/worker.rs`, `src/ha/mod.rs`, `src/api/mod.rs`, `src/api/controller.rs`, `src/cli/client.rs`, `src/cli/output.rs`, and every touched test file that depends on the old action-vector or stringified response shape.
- [ ] Replace `Vec<HaAction>` in production HA planning/output with a typed domain-level effect plan structure.
- [ ] Remove `pending: Vec<HaAction>` from `HaState`.
- [ ] Remove `pending_actions` from the HA state API/CLI surface unless reintroduced only as a derived debug-only summary outside core machine state.
- [ ] Replace string-based HA phase/trust exposure in API/CLI with typed serialized enums suitable for direct assertions in tests.
- [ ] Ensure the typed plan shape makes duplicate/conflicting effects impossible or explicit enough that post-hoc dedupe is no longer the primary correctness mechanism.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
</acceptance_criteria>
