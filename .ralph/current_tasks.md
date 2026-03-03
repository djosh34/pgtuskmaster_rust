# Current Tasks Summary

Generated: Tue Mar  3 02:59:42 AM CET 2026

**Path:** `.ralph/tasks/bugs/bug-remove-unwrap-panic-allow.md`

## Bug: Remove Clippy Allowances For Unwrap/Panic <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
src/test_harness/mod.rs explicitly allows clippy unwrap/expect/panic, which violates the repo rule against unwraps, panics, or expects anywhere. This hides violations in test harness code and makes it easy to slip new ones in. Investigate all test_harness code (and any other modules) for unwrap/expect/panic usage, replace with proper error handling, and remove the lint allow attributes.
</description>

---

**Path:** `.ralph/tasks/bugs/dcs-watch-refresh-errors-ignored.md`

## Bug: DCS watch refresh errors are tracked but ignored <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
`refresh_from_etcd_watch` in [src/dcs/store.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dcs/store.rs) records `had_errors` (for unknown keys or decode failures) but no caller uses it. In [src/dcs/worker.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dcs/worker.rs), `step_once` only checks for `Err`, so unknown/malformed watch events can be silently ignored while the worker still reports healthy state. Decide on the correct behavior (e.g., mark store unhealthy, emit faulted state, or log/telemetry), and wire `had_errors` into worker health so errors do not pass silently.
</description>

---

**Path:** `.ralph/tasks/bugs/pginfo-standby-polling-test-configure-primary-db-error.md`

## Bug: Pginfo standby polling test fails during primary configure with db error <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
`make test` failed in `pginfo::worker::tests::step_once_maps_replica_when_polling_standby` with a runtime panic while preparing the primary postgres fixture.

---

**Path:** `.ralph/tasks/bugs/process-worker-real-job-tests-accept-failure-outcomes.md`

## Bug: Process worker real job tests accept failure outcomes <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
Real-binary process worker tests in [src/process/worker.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/process/worker.rs) accept failure outcomes, so they can pass even when the binary invocation or behavior is broken. Examples:
- `real_promote_job_executes_binary_path`

---

**Path:** `.ralph/tasks/bugs/process-worker-real-job-tests-state-channel-closed.md`

## Bug: Process worker real job tests fail with state channel closed <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
`make test` failed while running real process worker job tests. Multiple tests panic because process state publish fails with `state channel is closed`.

---

**Path:** `.ralph/tasks/bugs/real-binary-tests-fail-when-port-allocation-is-blocked.md`

## Bug: Real-binary tests fail when port allocation is blocked <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
`make test` is not passing in the current environment because multiple tests panic when `allocate_ports(...)` returns `io error: Operation not permitted (os error 1)`.

---

**Path:** `.ralph/tasks/bugs/remove-panics-expects-unwraps.md`

## Bug: Remove panics/expects/unwraps in codebase <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
`rg -n "unwrap\(|expect\(|panic!" src tests` shows multiple occurrences (mostly in tests and some src modules like `src/process/worker.rs`, `src/pginfo/state.rs`, `src/pginfo/query.rs`, `src/dcs/worker.rs`, `src/dcs/store.rs`, `src/ha/worker.rs`, `tests/bdd_state_watch.rs`, `src/config/parser.rs`). Policy requires no unwraps/panics/expects anywhere; replace with proper error handling and remove any lint exemptions if present. Explore and confirm current behavior before changing.
</description>

---

**Path:** `.ralph/tasks/bugs/test-harness-binary-check-panics.md`

## Bug: Test harness binary checks panic instead of returning errors <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
The test harness binary lookup in [src/test_harness/binaries.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/test_harness/binaries.rs) uses `panic!` to report missing binaries. This conflicts with the project policy of no `panic`/`expect`/`unwrap` and makes tests fail via uncontrolled panics rather than structured errors. Refactor `require_binary` (and callers) to return a typed `HarnessError` instead of panicking, and update callers/tests to propagate or assert errors explicitly.
</description>

---

**Path:** `.ralph/tasks/bugs/worker-contract-tests-assert-only-callability.md`

## Bug: Worker contract tests only assert callability <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
[worker_contract_tests.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/worker_contract_tests.rs) primarily asserts that `step_once` functions are callable and return `Ok(())`, without validating resulting state changes or side effects. This means tests can pass even if core worker logic regresses or stops mutating state. Strengthen these tests with minimal behavioral assertions (state version bump, expected phase transitions, or expected publish effects), or split compile-time contract checks into non-test compile gates and add real behavioral tests.
</description>

---

**Path:** `.ralph/tasks/story-full-verification/01-task-verify-build-and-static-gates.md`

## Task: Verify build and static quality gates <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
**Goal:** Validate the codebase can build and pass core static gates before deeper test execution.

---

**Path:** `.ralph/tasks/story-full-verification/02-task-run-targeted-unit-and-integration-tests.md`

## Task: Run targeted unit and integration test suites <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
**Goal:** Execute and validate non-e2e automated tests after static/build gates to identify functional regressions early.

---

**Path:** `.ralph/tasks/story-full-verification/03-task-run-full-suite-regression-pass.md`

## Task: Run full regression suite end-to-end <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
**Goal:** Execute the entire validation suite in one pass to confirm holistic repository health.

---

**Path:** `.ralph/tasks/story-full-verification/04-task-resolve-failures-and-revalidate-full-suite.md`

## Task: Resolve discovered failures and revalidate full suite <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
**Goal:** Drive failure resolution from created bug tasks and confirm full-suite green status after fixes.

---

**Path:** `.ralph/tasks/story-rust-system-harness/01-task-core-types-time-errors-watch-channel.md`

## Task: Implement core ids time errors and typed watch channels <status>done</status> <passes>true</passes> <passing>true</passing> <priority>ultra_high</priority>

<description>
**Goal:** Build the foundational shared types and state-channel primitives used by every worker.

---

**Path:** `.ralph/tasks/story-rust-system-harness/02-task-runtime-config-schema-defaults-parse-validate.md`

## Task: Implement runtime config schema defaults parser and validation <status>done</status> <passes>true</passes> <passing>true</passing> <priority>ultra_high</priority>

<blocked_by>01-task-core-types-time-errors-watch-channel</blocked_by>

<description>

---

**Path:** `.ralph/tasks/story-rust-system-harness/03-task-worker-state-models-and-context-contracts.md`

## Task: Define worker state models and run step_once contracts <status>done</status> <passes>true</passes> <passing>true</passing> <priority>ultra_high</priority>

<blocked_by>02-task-runtime-config-schema-defaults-parse-validate</blocked_by>

<description>

---

**Path:** `.ralph/tasks/story-rust-system-harness/04-task-pginfo-worker-single-query-and-real-pg-tests.md`

## Task: Implement pginfo worker single-query polling and real PG tests <status>done</status> <passes>true</passes> <passing>true</passing> <priority>high</priority>

<blocked_by>03-task-worker-state-models-and-context-contracts</blocked_by>

<description>

---

**Path:** `.ralph/tasks/story-rust-system-harness/05a-task-enforce-strict-rust-lints-no-unwrap-expect-panic.md`

## Task: Enforce strict Rust lint policy and forbid unwrap expect panic in runtime code <status>done</status> <passes>true</passes> <passing>true</passing> <priority>ultra_high</priority>

<description>
**Goal:** Install and enforce strict Rust linting with explicit denial of `unwrap`, `expect`, and panic-prone patterns in runtime code.

---

**Path:** `.ralph/tasks/story-rust-system-harness/05b-task-deep-review-codebase-and-verify-done-work.md`

## Task: Deep review codebase quality and verify done tasks are truly complete <status>done</status> <passes>true</passes> <passing>true</passing> <priority>ultra_high</priority>

<description>
**Goal:** Perform a deep end-to-end review of current repository quality, test reality, and completion truthfulness of all tasks already marked done.

---

**Path:** `.ralph/tasks/story-rust-system-harness/05c-task-zero-panic-unwrap-expect-across-runtime-and-tests.md`

## Task: Enforce zero panic/unwrap/expect across runtime and tests with proper Result handling <status>done</status> <passes>true</passes> <passing>true</passing> <priority>high</priority>

<description>
**Goal:** Remove all manual panic/unwrap/expect usage from runtime and test code, replace with proper Rust error handling, and make lint enforcement fail on any regression.

---

**Path:** `.ralph/tasks/story-rust-system-harness/05-task-dcs-worker-trust-cache-watch-member-publish.md`

## Task: Implement DCS worker trust evaluation cache updates and member publishing <status>done</status> <passes>true</passes> <priority>high</priority>

<blocked_by>03-task-worker-state-models-and-context-contracts</blocked_by>
<passing>true</passing>

---

**Path:** `.ralph/tasks/story-rust-system-harness/06-task-process-worker-single-active-job-real-job-exec.md`

## Task: Implement process worker single-active-job execution with real job tests <status>done</status> <passes>true</passes> <passing>true</passing> <priority>high</priority>

<blocked_by>03-task-worker-state-models-and-context-contracts</blocked_by>

<description>

---

**Path:** `.ralph/tasks/story-rust-system-harness/07-task-ha-decide-pure-matrix-idempotency-tests.md`

## Task: Implement pure HA decide engine with exhaustive transition tests <status>done</status> <passes>true</passes> <passing>true</passing> <priority>ultra_high</priority>

<blocked_by>03-task-worker-state-models-and-context-contracts</blocked_by>

<description>

---

**Path:** `.ralph/tasks/story-rust-system-harness/08-task-ha-worker-select-loop-and-action-dispatch.md`

## Task: Implement HA worker select loop and action dispatch wiring <status>done</status> <passes>true</passes> <passing>true</passing> <priority>high</priority>

<blocked_by>04-task-pginfo-worker-single-query-and-real-pg-tests,05-task-dcs-worker-trust-cache-watch-member-publish,06-task-process-worker-single-active-job-real-job-exec,07-task-ha-decide-pure-matrix-idempotency-tests</blocked_by>

<description>

---

**Path:** `.ralph/tasks/story-rust-system-harness/09-task-api-debug-workers-and-snapshot-contracts.md`

## Task: Implement API and Debug API workers with typed contracts <status>done</status> <passes>true</passes> <priority>high</priority>

<blocked_by>05-task-dcs-worker-trust-cache-watch-member-publish,08-task-ha-worker-select-loop-and-action-dispatch</blocked_by>
<passing>true</passing>

---

**Path:** `.ralph/tasks/story-rust-system-harness/10a-task-enforce-real-binary-tests-and-ci-prereqs.md`

## Task: Enforce real-binary test execution (PG16 + etcd3) via explicit gate + CI prerequisites <status>done</status> <passes>true</passes> <passing>true</passing> <priority>high</priority>

<description>
**Goal:** Ensure “real-system” tests actually exercise real PostgreSQL 16 and etcd3 binaries in at least one deterministic gate (CI and/or developer opt-in), instead of silently passing via early-return skips.

---

**Path:** `.ralph/tasks/story-rust-system-harness/10b-task-dcs-real-etcd3-store-adapter-and-tests.md`

## Task: Implement real etcd3-backed DCS store adapter and integration tests <status>done</status> <passes>true</passes> <passing>true</passing> <priority>high</priority>

<description>
**Goal:** Add a production-grade `DcsStore` implementation backed by a real etcd3 instance, and prove it via integration tests using the existing test harness spawner.

---

**Path:** `.ralph/tasks/story-rust-system-harness/10-task-test-harness-namespace-ports-pg-etcd-spawners.md`

## Task: Build parallel-safe real-system test harness for PG16 and etcd3 <status>done</status> <passes>true</passes> <passing>true</passing> <priority>ultra_high</priority>

<blocked_by>02-task-runtime-config-schema-defaults-parse-validate,03-task-worker-state-models-and-context-contracts</blocked_by>

<description>

---

**Path:** `.ralph/tasks/story-rust-system-harness/11-task-typed-pg-config-and-conninfo-roundtrip-tests.md`

## Task: Implement typed postgres config and conninfo parser renderer <status>done</status> <passes>true</passes> <passing>true</passing> <priority>high</priority>

<blocked_by>02-task-runtime-config-schema-defaults-parse-validate</blocked_by>

<description>

---

**Path:** `.ralph/tasks/story-rust-system-harness/12-task-ha-loop-integration-tests-real-watchers-and-step-once.md`

## Task: Build HA loop integration tests with real watchers and deterministic stepping <status>done</status> <passes>true</passes> <passing>true</passing> <priority>ultra_high</priority>

<blocked_by>08-task-ha-worker-select-loop-and-action-dispatch,10-task-test-harness-namespace-ports-pg-etcd-spawners</blocked_by>

<description>

---

**Path:** `.ralph/tasks/story-rust-system-harness/13-task-e2e-multi-node-real-ha-loops-scenario-matrix.md`

## Task: Implement e2e multi-node real HA-loop scenario matrix <status>done</status> <passes>true</passes> <passing>true</passing> <priority>ultra_high</priority>

<blocked_by>09-task-api-debug-workers-and-snapshot-contracts,10-task-test-harness-namespace-ports-pg-etcd-spawners,12-task-ha-loop-integration-tests-real-watchers-and-step-once</blocked_by>

<description>

---

**Path:** `.ralph/tasks/story-rust-system-harness/14-task-security-auth-tls-real-cluster-tests.md`

## Task: Implement security auth TLS validation tests in real cluster runs <status>done</status> <passes>true</passes> <passing>true</passing> <priority>high</priority>

<blocked_by>10-task-test-harness-namespace-ports-pg-etcd-spawners,13-task-e2e-multi-node-real-ha-loops-scenario-matrix</blocked_by>

<description>

---

**Path:** `.ralph/tasks/story-rust-system-harness/15-task-final-double-check-and-stop-gate.md`

## Task: Final double-check gate for real testing completeness <status>done</status> <passes>true</passes> <passing>true</passing> <priority>ultra_high</priority>

<blocked_by>13-task-e2e-multi-node-real-ha-loops-scenario-matrix,14-task-security-auth-tls-real-cluster-tests</blocked_by>

<description>

---

**Path:** `.ralph/tasks/story-rust-system-harness/16-task-debug-ui-verbose-state-actions-events-and-final-stop.md`

## Task: Setup verbose debug UI and final STOP gate <status>not_started</status> <passes>false</passes> <priority>low</priority>

<blocked_by>15-task-final-double-check-and-stop-gate</blocked_by>

<description>

---

**Path:** `.ralph/tasks/story-rust-system-harness/18-task-recurring-meta-deep-skeptical-codebase-review.md`

## Task: Recurring meta-task for deep skeptical codebase quality verification <status>not_started</status> <passes>meta-task</passes>
NEVER TICK OFF THIS TASK. ALWAYS KEEP <passes>meta-task</passes>. This is a recurring deep verification task.

<description>
This is a **RECURRING META-TASK**.

---

**Path:** `.ralph/tasks/story-rust-system-harness/19-task-do-meta-deep-skeptical-review-pass-1.md`

## Task: Do meta-task 18 deep skeptical review pass 1 <status>not_started</status> <passes>false</passes>

<blocked_by>18-task-recurring-meta-deep-skeptical-codebase-review</blocked_by>

<description>

---

**Path:** `.ralph/tasks/story-rust-system-harness/20-task-do-meta-deep-skeptical-review-pass-2.md`

## Task: Do meta-task 18 deep skeptical review pass 2 <status>not_started</status> <passes>false</passes>

<blocked_by>19-task-do-meta-deep-skeptical-review-pass-1</blocked_by>

<description>

---

**Path:** `.ralph/tasks/story-rust-system-harness/21-task-do-meta-deep-skeptical-review-pass-3.md`

## Task: Do meta-task 18 deep skeptical review pass 3 <status>not_started</status> <passes>false</passes>

<blocked_by>20-task-do-meta-deep-skeptical-review-pass-2</blocked_by>

<description>

---

**Path:** `.ralph/tasks/story-rust-system-harness/task-real-ha-dcs-process-integration-tests.md`

## Task: Add real HA+DCS+Process integration tests <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
**Goal:** Build integration tests that wire real PG16 binaries, a real etcd-backed DCS store, the process worker, pginfo worker, and HA worker so failures cannot pass silently.

---

**Path:** `.ralph/tasks/story-rust-system-harness/task-typed-dcs-writes-and-encapsulation.md`

## Task: Replace Stringly DCS Writes With Typed Writer API <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
**Goal:** Eliminate raw path/string DCS writes from HA by introducing a typed DCS writer API and restricting access to low-level write/delete operations.

