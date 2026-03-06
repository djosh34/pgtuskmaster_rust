---
## Task: Rebuild HA tests around invariants and continuous observers <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Rework HA tests so they validate the new functional architecture directly, using immutable builders for pure decision tests and continuous invariant observers for integration/e2e scenarios.

**Scope:**
- Edit HA unit, integration, and e2e tests in `src/ha/`, along with any helper modules needed for invariant observation.
- Replace mutation-heavy test setup where possible with immutable facts/world builders for pure-decision coverage.
- Add a continuous observer layer for e2e/integration HA scenarios that samples cluster state throughout a scenario and checks invariants continuously.

**Context from research:**
- We discussed two distinct testing needs:
  - pure decision tests should assert exact `DecisionFacts -> PhaseOutcome` mappings and invariants
  - e2e/integration tests should continuously observe invariants such as “never more than one primary,” not only assert a final converged state
- The current partition and multi-node tests already poll HA state, but the observer/invariant logic is not modeled as a clear continuous layer and remains too interleaved with scenario scripting.
- Existing runtime pollers for DCS and pginfo are not a substitute for a test-side continuous invariant observer.

**Expected outcome:**
- Pure decision tests become declarative, table-driven, and easy to extend without mutating nested world fixtures by hand.
- HA integration/e2e tests gain a reusable observer that checks invariants throughout a scenario window and fails closed when observation is insufficient.
- Scenario drivers, observers, and assertions become clearer and easier to maintain separately.

**Story test policy:**
- Skip `make test-long` and any direct long HA cargo-test invocations in this task.
- Known long-test failures are deferred until the final story task after the rewrite story lands.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Modify the exact HA test files touched by the rewrite, including `src/ha/decide.rs` tests, `src/ha/worker.rs` tests, `src/ha/e2e_partition_chaos.rs`, `src/ha/e2e_multi_node.rs`, and any helper modules introduced for invariant observation.
- [ ] Add immutable builders/helpers for pure decision test inputs instead of relying on widespread in-test mutation of nested world state where avoidable.
- [ ] Add table-driven tests that assert exact pure outputs for representative phase/facts combinations.
- [ ] Add invariant/property-style tests that cover determinism, fail-safe safety, and impossible contradictory plans.
- [ ] Introduce a continuous HA invariant observer for integration/e2e scenarios that samples during the whole scenario window and can assert invariants such as “never more than one primary.”
- [ ] Ensure observer-based tests fail closed when there are zero or insufficient successful samples.
- [ ] Split or refactor scenario code so drivers, observers, and assertions are clearer layers than they are today.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] Explicitly skip `make test-long` and direct long HA cargo-test invocations in this task; long-test validation is deferred to task `06-task-move-and-split-ha-e2e-tests-after-functional-rewrite.md`
</acceptance_criteria>
