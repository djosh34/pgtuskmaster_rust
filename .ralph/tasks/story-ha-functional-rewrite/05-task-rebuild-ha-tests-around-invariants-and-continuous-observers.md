## Task: Rebuild HA tests around invariants and continuous observers <status>completed</status> <passes>true</passes>

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

**Story test policy for this execution:**
- Treat all repo gates as required for completion in this task: `make check`, `make test`, `make test-long`, and `make lint`.
- If the long HA scenarios expose missing prerequisites or flaky assumptions, fix those issues here instead of deferring them.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] Modify the exact HA test files touched by the rewrite, including `src/ha/decide.rs` tests, `src/ha/worker.rs` tests, `src/ha/e2e_partition_chaos.rs`, `src/ha/e2e_multi_node.rs`, and any helper modules introduced for invariant observation.
- [x] Add immutable builders/helpers for pure decision test inputs instead of relying on widespread in-test mutation of nested world state where avoidable.
- [x] Add table-driven tests that assert exact pure outputs for representative phase/facts combinations.
- [x] Add invariant/property-style tests that cover determinism, fail-safe safety, and impossible contradictory plans.
- [x] Introduce a continuous HA invariant observer for integration/e2e scenarios that samples during the whole scenario window and can assert invariants such as “never more than one primary.”
- [x] Ensure observer-based tests fail closed when there are zero or insufficient successful samples.
- [x] Split or refactor scenario code so drivers, observers, and assertions are clearer layers than they are today.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make test-long` — passes cleanly
- [x] `make lint` — passes cleanly
</acceptance_criteria>

<plan>
1. Establish the pure-decision test shape in `src/ha/decide.rs`.
   - Replace ad hoc `world(...)` usage plus post-construction mutation with immutable test builders/helpers that return fully-formed `WorldSnapshot` and `HaState` values in one expression.
   - Keep the builders local to HA tests unless a shared helper is clearly reused by both `decide.rs` and `worker.rs`; if shared, place them in a `#[cfg(test)]` HA helper module under `src/ha/` so production code does not absorb test-only fixture APIs.
   - Rewrite the existing transition-matrix coverage so each case reads as exact `input facts -> expected phase + expected decision + expected lowered effects`, with no fixture mutation hidden after construction.
   - Preserve coverage already present for switchover, rewind/basebackup/bootstrap, fencing, and fail-safe transitions, but express each case through the new immutable builders.

2. Add deeper invariant-style decision tests in `src/ha/decide.rs`.
   - Add a determinism test that runs `decide(...)` multiple times over identical immutable inputs and proves the same `PhaseOutcome` and next `HaState` are produced each time.
   - Add fail-safe safety coverage across representative phases showing every non-`FullQuorum` trust state routes to `FailSafe`, with `release_leader_lease=true` only when the current phase is `Primary`.
   - Add contradiction guards over lowered plans so impossible combinations are rejected by tests, for example:
     - no plan both acquires and releases the leader lease
     - no plan both promotes and demotes PostgreSQL
     - no plan both follows a leader and requests promotion in the same decision
     - no fail-safe decision carries conflicting replication/postgres side effects
   - Keep these tests table-driven rather than property-framework-based unless the repo already uses a property crate elsewhere.

3. Rebuild HA worker tests around immutable fixture construction in `src/ha/worker.rs`.
   - Introduce a fixture/builder layer for `HaWorkerCtx`, runtime config, DCS state, pg state, process state, and clock setup so tests stop mutating nested context fields directly after `build_context(...)`.
   - Preserve the existing contract and integration coverage, but rewrite the core tests to express setup as “given facts/world” and assertions as “decision/effect/state publication”.
   - Add or tighten tests that prove:
     - `step_once(...)` publishes the exact same state that `decide(...)` selected for the same snapshot
     - repeated ticks under unchanged conditions reissue the same effect plan where the design requires it
     - worker transitions do not emit contradictory dispatch requests when the decision is fail-safe, fencing, or recovery oriented
   - Keep all error handling explicit; do not add `unwrap`, `expect`, or panic-driven helpers.

4. Extract a real continuous invariant observer for HA scenario tests.
   - Introduce a shared `#[cfg(test)]` observer helper module in `src/ha/` dedicated to test-side HA observation so `src/ha/e2e_multi_node.rs` and `src/ha/e2e_partition_chaos.rs` stop embedding the same polling/finalization logic inline.
   - Wire the helper through `src/ha/mod.rs` as a test-only module so the location is explicit before implementation begins; do not hide the observer inside one of the existing e2e files if both files consume it.
   - The observer API should clearly separate:
     - scenario driver actions: partition, heal, stop, restart, switchover, SQL workload
     - sampling: API polls and any fallback SQL role checks used to avoid false negatives during transient API unavailability
     - invariant evaluation: no dual primary, minimum successful sample count, leader-change accounting, fail-safe observation, recent sample evidence ring
     - final assertion/reporting: fail closed when evidence is missing or insufficient, include last observations and error counts in failures
   - Prefer a model where scenarios open an observation window around the whole fault sequence instead of only sampling after convergence.

5. Refactor `src/ha/e2e_multi_node.rs` to use the shared observer.
   - Keep `wait_for_stable_primary*` helpers focused on convergence/waiting only; remove invariant-accounting responsibilities from them where possible.
   - Migrate `HaObservationStats`, `sample_ha_states_window(...)`, `assert_no_dual_primary_in_samples(...)`, and related finalize helpers into the shared observer layer if they are useful outside one file.
   - Update the stress/no-quorum/failover/switchover scenarios so they:
     - start the observer before the disruptive action
     - run the scenario driver steps
     - stop/finalize the observer after the healing/convergence window
     - assert both scenario-specific outcomes and observer invariants
   - Preserve the current “fail closed on zero samples” behavior and tighten it where sample counts can be insufficient but non-zero.

6. Refactor `src/ha/e2e_partition_chaos.rs` to use the same observer model.
   - Replace the local `finalize_no_dual_primary_window(...)` and similar observation bookkeeping with the shared observer/finalizer.
   - Keep resilient primary waiters for recovery checkpoints, but make the no-dual-primary guarantee come from the continuous observer window rather than from final-state-only checks.
   - Ensure partition scenarios continuously record enough evidence to detect:
     - transient dual-primary windows
     - missing observation evidence
     - API-only blind spots that require SQL-role fallback evidence
   - Preserve the timeline/artifact writing flow so failures remain diagnosable.

7. Add focused unit coverage for the new observer helper module.
   - Test zero-sample finalization failures.
   - Test insufficient-sample failures when a configured minimum threshold is not reached.
   - Test successful finalization when enough samples exist and no invariant is violated.
   - Test invariant failures when sampled data shows more than one primary.
   - Keep these tests fast and deterministic with synthetic samples rather than real cluster startup.

8. Update docs that describe the HA testing strategy.
   - Update `docs/src/contributors/testing-system.md` so it explains the new split between pure decision tests, worker/integration tests, and continuous-observer HA scenarios.
   - Update `docs/src/contributors/ha-pipeline.md` if needed so contributor guidance points to exact-output decision tests and continuous invariant observation for real-binary scenarios.
   - Remove stale wording that implies e2e assertions are primarily final-state convergence checks.

9. Execute in parallel where safe.
   - One implementation lane should handle pure decision/worker test builders and invariant tests.
   - Another implementation lane should handle the shared observer extraction plus e2e scenario rewiring.
   - Docs updates should be done after code/test structure settles so documentation reflects the landed design instead of an intermediate state.

10. Verification sequence for execution mode.
   - Run targeted HA tests first for fast feedback while editing:
     - focused `cargo test` for `ha::decide`
     - focused `cargo test` for `ha::worker`
     - focused `cargo test` for the touched e2e unit/helper tests, including the extracted observer module
   - Then run the repo gates required for actual completion:
     - `make check`
     - `make test`
     - `make test-long`
     - `make lint`
   - Do not mark the task complete or set `<passes>true</passes>` until all four commands pass, even though the original story note says long tests are deferred; the current turn-level instructions require full verification before completion.
</plan>

NOW EXECUTE
