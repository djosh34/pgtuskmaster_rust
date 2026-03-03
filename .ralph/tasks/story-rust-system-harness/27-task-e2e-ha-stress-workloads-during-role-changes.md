---
## Task: Add HA stress e2e suites with concurrent SQL workloads during role changes <status>done</status> <passes>true</passes> <passing>true</passing>

<blocked_by>24-task-real-e2e-harness-3nodes-3etcd</blocked_by>
<blocked_by>25-task-enforce-e2e-api-only-control-no-direct-dcs</blocked_by>

<description>
**Goal:** Build stress-oriented e2e tests that continuously read/write/query SQL while HA switchover and failover paths execute, and verify safe demotion/promotion/fencing behavior.

**Scope:**
- Add workload generators that run concurrent SQL read/write traffic through the active cluster endpoint during HA transitions.
- Cover planned switchover, unplanned failover, and fencing-sensitive windows with measurable assertions.
- Assert no split-brain writes, no dual-primary windows, and expected role convergence via API-visible state.
- Add timeline artifacts/metrics to aid deterministic debugging when stress tests fail.

**Context from research:**
- Existing e2e scenario is primarily a reaction matrix and does not sustain high SQL activity during transitions.
- Requirement calls for many more skeptical stress tests while HA operations are in flight.
- Current worker state channels and timeline artifacts can be extended for richer observability.

**Expected outcome:**
- The suite includes multiple high-load HA transition scenarios proving safety and data correctness under stress.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] Full exhaustive checklist completed with concrete module requirements: new/updated stress e2e files (`src/ha/e2e_*` and/or `tests/e2e_*`), SQL workload helper module(s), API-state assertion utilities, artifact logging paths under `.ralph/evidence` for stress timelines and summary stats
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] BDD features pass (covered by `make test`).
</acceptance_criteria>

<execution_plan>
## Detailed Implementation Plan (Draft 2)

### Exploration tracks completed in parallel (12)
- Track 1: mapped HA/e2e files and confirmed real e2e currently lives in `src/ha/e2e_multi_node.rs` with two scenarios only.
- Track 2: confirmed existing scenarios validate switchover/failover outcomes but do not run sustained concurrent SQL workload during transitions.
- Track 3: confirmed current fixture already has SQL execution helpers (`run_psql_statement`, row parsing, retries) and API polling helpers (`wait_for_stable_primary`, `assert_no_dual_primary_window`).
- Track 4: confirmed API reads are pumped through `step_once` and expose `HaStateResponse` fields needed for role/topology assertions.
- Track 5: confirmed current artifact writer only emits timeline logs in `.ralph/evidence/13-e2e-multi-node`; no structured stress metrics artifact exists.
- Track 6: verified current tests already use deterministic scenario/global timeouts; stress suites should reuse and extend this timeout model.
- Track 7: validated no-quorum path currently checks fail-safe convergence but does not validate write-fencing behavior under concurrent load.
- Track 8: inspected harness/binary requirements and verified real-binary tests are fail-fast (no optional skip path).
- Track 9: confirmed `make` gate order to satisfy completion remains `make check` -> `make test` -> `make test-long` -> `make lint`.
- Track 10: confirmed `src/ha/mod.rs` currently wires one e2e module (`e2e_multi_node`), so stress scenarios can be additive in same file or a new `e2e_*.rs` module.
- Track 11: validated current SQL helpers are node-targeted; no reusable concurrent workload runner abstraction exists yet.
- Track 12: reviewed prior completed e2e tasks and lifecycle markers to mirror task-file protocol (`TO BE VERIFIED` -> skeptical delta -> `NOW EXECUTE`).

### Skeptical verification deltas applied (17 parallel tracks)
- Delta 1: replaced vague "active cluster endpoint" assumption with node-aware execution design because current fixture only exposes per-node SQL ports and API addresses.
- Delta 2: added explicit split between mutable API polling fixture and immutable SQL workload context to avoid borrow/contention issues while workload loops run concurrently.
- Delta 3: added required SQL error classification contract (`transient`, `fencing/read-only`, `hard`) so assertions do not mislabel expected failover turbulence as regressions.
- Delta 4: tightened no-quorum expectation from "writes stop immediately" to "writes stop after bounded fencing grace window", with explicit cutoff assertion.
- Delta 5: required one table namespace per scenario (`ha_stress_<scenario>`) to prevent cross-test contamination and simplify forensic queries.
- Delta 6: required deterministic workload IDs (`worker_id`, `seq`) and uniqueness checks over committed rows to strengthen split-brain evidence.
- Delta 7: added requirement to cap API sample retention (ring buffer) while still persisting summary counters to avoid unbounded in-memory growth.
- Delta 8: added explicit artifact schema version field in summary JSON for forward-compatible debugging.
- Delta 9: added fallback validation queries on each replica after transition to prove convergence and continuity beyond primary-only reads.
- Delta 10: required explicit scenario-local timeout budgets (bootstrap, transition, stabilization, finalization) in addition to global timeout.
- Delta 11: added post-workload drain interval before final assertions so in-flight statements settle deterministically.
- Delta 12: required final timeline entries to include outcome class and artifact paths regardless of pass/fail branch.
- Delta 13: tightened module layout decision: keep first implementation in `src/ha/e2e_multi_node.rs` unless file growth becomes non-reviewable; only then split.
- Delta 14: added gate-run stability environment for this workspace mount: `CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1`.
- Delta 15: added required marker-grep evidence capture after `make test` and `make lint` into task-specific evidence folder.
- Delta 16: added rollback safety check that existing two e2e tests remain intact and still listed by test name.
- Delta 17: confirmed no optional/skip behavior may be introduced for real-binary prerequisites or stress scenarios.

### Design targets for this task
1. Add stress suites that generate continuous SQL read/write pressure while HA role transitions are in flight.
2. Verify safety invariants under load: no split-brain write acceptance, no dual-primary window, deterministic role convergence.
3. Produce deterministic debugging artifacts: per-scenario timeline and machine-readable workload summary metrics under `.ralph/evidence`.
4. Keep all error paths fully handled with `Result` propagation, no unwrap/expect/panic.

### Concrete implementation plan
1. Introduce workload runner utilities in HA e2e test surface.
- Location: `src/ha/e2e_multi_node.rs` (or split to new `src/ha/e2e_stress_workload.rs` and include from `src/ha/mod.rs` under `#[cfg(test)]`).
- Add `SqlWorkloadSpec` (worker_count, run_interval_ms, operation_mix, deadline, scenario_name).
- Add `SqlWorkloadStats` and per-worker stats structures (attempted writes, committed writes, read successes, transient failures, hard failures, max/avg latency buckets).
- Build a `SqlWorkloadCtx` cloned from fixture data (`psql_bin`, node id->port map, table name, scenario stamp) so workload tasks can run without borrowing mutable fixture state.
- Add cancellable async workload loop(s) that:
- repeatedly issue inserts/upserts/selects against cluster node ports,
- classify failures into transient vs fencing/read-only vs hard counters,
- preserve successful write IDs as `(worker_id, seq)` tuples for post-run uniqueness/continuity assertions.

2. Add API-state assertion utilities for stress windows.
- Extend fixture helpers to sample `/ha/state` repeatedly during stress windows and store observation snapshots.
- Add helpers:
- `assert_no_dual_primary_in_samples(...)`
- `assert_role_converged_to_single_primary(...)`
- `assert_former_primary_demoted_after_transition(...)`
- `assert_no_split_brain_write_evidence(...)` based on write-id ownership and state samples.
- Add bounded ring-buffer sampling to cap memory and keep aggregate counters (max primary count, leader changes, failsafe observations).
- Ensure assertions emit actionable context (last N states, leader ids, phase history, workload counters).

3. Add stress artifact writers under dedicated task path.
- Add per-scenario timeline path: `.ralph/evidence/27-e2e-ha-stress/<scenario>-<stamp>.timeline.log`.
- Add structured summary path: `.ralph/evidence/27-e2e-ha-stress/<scenario>-<stamp>.summary.json`.
- Summary payload includes:
- schema version + timestamp metadata,
- scenario metadata (seed, duration, worker_count, operation cadence),
- HA observations (phase transitions per node, leader changes, max concurrent primaries observed),
- workload metrics (attempts/success/fail categories, distinct committed ids, continuity proof details),
- failure diagnostics when assertions fail.

4. Add stress scenario: planned switchover under concurrent workload.
- Bootstrap stable primary.
- Start concurrent workload and let it warm up.
- Trigger `POST /switchover` while workload keeps running.
- Assert during and after transition:
- no dual-primary observations,
- exactly one final stable primary different from bootstrap,
- writes continue eventually and committed rows remain unique/consistent.
- Stop workload, allow short drain interval, flush stats, run final continuity queries on promoted primary and replicas.

5. Add stress scenario: unassisted failover under concurrent workload.
- Bootstrap stable primary and start workload.
- Inject failure by stopping current primary postgres only.
- Do not issue control mutations after injection.
- Assert:
- eventual new stable primary,
- former primary leaves `Primary`,
- no dual-primary window,
- post-failover writes continue and pre-failure committed data remains queryable from all surviving nodes.

6. Add stress scenario: fencing-sensitive no-quorum window under workload.
- Bootstrap stable primary and start workload.
- Stop etcd majority (2/3) while workload is active.
- Assert cluster converges to `FailSafe` and new write commits stop being accepted after bounded fencing grace window.
- Validate safety posture:
- no evidence of concurrent accepted writes from multiple primaries,
- expected increase in rejected/transient write outcomes while in fail-safe, with explicit cutoff timestamp recorded in summary.

7. Keep compile and runtime scope controlled.
- Prefer additive helper methods and tests; avoid regressions in existing non-stress scenarios.
- If stress helpers become large, move them to a dedicated `src/ha/e2e_*` module and keep compatibility with existing fixture APIs.
- Ensure no new optional-test behavior is introduced.

### Planned execution phases (for `NOW EXECUTE`)
1. Scaffolding phase.
- Implement workload spec/stats/runner types and cancellation plumbing.
- Add summary/timeline artifact writing utilities for stress scenarios.

2. Assertion phase.
- Implement reusable API-sampling + safety assertion helpers.
- Add phase-history and leader-change summarizers for diagnostic output.

3. Scenario phase.
- Implement three stress tests (switchover, unassisted failover, no-quorum fencing) using shared helpers.
- Hook scenario-specific artifact names and deterministic timeout guards.

4. Validation phase.
- Run required gates in strict order with stability env:
- `CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1 make check`
- `CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1 make test`
- `CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1 make test`
- `CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1 make lint`
- Capture logs under `.ralph/evidence/27-e2e-ha-stress/gates/`.
- Capture required marker grep evidence from make logs:
- `congratulations`
- `evaluation failed`

5. Finalization phase.
- Tick acceptance checklist, set status/passing tags only after all gates pass.
- Run `/bin/bash .ralph/task_switch.sh`.
- Commit all files with required message format including gate evidence and implementation summary.

### Parallel implementation tracks to run during execution (15)
- Track 1: workload spec/state model definitions.
- Track 2: workload runner loop and cancellation semantics.
- Track 3: SQL statement templates for write/read/mutation mix.
- Track 4: transient vs hard SQL error classification.
- Track 5: workload metrics aggregation and invariant counters.
- Track 6: API state sampling ring buffer.
- Track 7: dual-primary and convergence assertion helpers.
- Track 8: split-brain write-evidence assertion logic.
- Track 9: artifact directory/path writer + JSON summary serializer.
- Track 10: switchover stress scenario implementation.
- Track 11: unassisted failover stress scenario implementation.
- Track 12: no-quorum fencing stress scenario implementation.
- Track 13: existing e2e compatibility and compile fallout cleanup.
- Track 14: make target runs and evidence capture.
- Track 15: task file completion tags + task switch + commit.
</execution_plan>

NOW EXECUTE
