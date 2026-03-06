# Done Tasks Summary

Generated: Fri Mar  6 15:33:32 CET 2026

# Task `.ralph/tasks/bugs/bug-bdd-http-tests-false-pass-via-fragile-status-and-read-patterns.md`

```
## Bug: BDD HTTP tests false-pass via fragile status and read patterns <status>done</status> <passes>true</passes>

<description>
BDD HTTP contract tests use weak status matching and response-read behavior that can hide protocol/handler regressions or induce hangs.
```

==============

# Task `.ralph/tasks/bugs/bug-ha-e2e-false-pass-via-best-effort-polling-and-timestamp-fallback.md`

```
## Bug: HA e2e false-pass via best-effort polling and timestamp fallback <status>done</status> <passes>true</passes>

<description>
HA e2e assertions can pass without reliable cluster-wide observations during unstable windows.
```

==============

# Task `.ralph/tasks/bugs/bug-real-binary-provenance-enforcement-gaps.md`

```
## Bug: Real-binary provenance enforcement gaps in installers and harness <status>done</status> <passes>true</passes>

<description>
Real-binary tooling currently enforces existence/executability but not strong provenance at runtime.
```

==============

# Task `.ralph/tasks/bugs/bug-real-binary-tests-are-optional-via-early-return.md`

```
## Bug: Real Binary Tests Become Optional Via Early Return <status>done</status> <passes>true</passes>

<description>
Several real-binary test paths silently return `Ok(())` when required binaries are not discovered (for example `None => return Ok(())`).
This makes critical runtime coverage optional and can mask regressions in HA/bootstrap/process behavior.
```

==============

# Task `.ralph/tasks/bugs/bug-remove-unwrap-panic-allow.md`

```
## Bug: Remove Clippy Allowances For Unwrap/Panic <status>done</status> <passes>true</passes>

<description>
src/test_harness/mod.rs explicitly allows clippy unwrap/expect/panic, which violates the repo rule against unwraps, panics, or expects anywhere. This hides violations in test harness code and makes it easy to slip new ones in. Investigate all test_harness code (and any other modules) for unwrap/expect/panic usage, replace with proper error handling, and remove the lint allow attributes.
</description>
```

==============

# Task `.ralph/tasks/bugs/bug-remove-writable-ha-leader-api-and-ha-loop-test-steering.md`

```
## Bug: Remove writable HA leader API control path and enforce HA-loop-only leadership transitions <status>completed</status> <passes>true</passes>

<description>
Investigation found that writable `/ha/leader` was introduced by task `22-task-ha-admin-api-read-write-surface` as part of a "full HA admin API read and write surface". In runtime code, `src/api/worker.rs` routes `POST /ha/leader` and `DELETE /ha/leader` to controller handlers in `src/api/controller.rs` that call `DcsHaWriter::write_leader_lease` / `delete_leader`, so external callers can directly mutate the leader key outside autonomous HA-loop decision flow.
```

==============

# Task `.ralph/tasks/bugs/bug-test-bdd-full-suite-hangs.md`

```
## Bug: test full-suite command hangs in real HA e2e <status>done</status> <passes>true</passes>

<description>
After updating `make test` to run `PGTUSKMASTER_REQUIRE_REAL_BINARIES=1 cargo test --all-targets -- --include-ignored`, the verification run did not complete within an extended runtime window (over 15 minutes observed on 2026-03-03).
```

==============

# Task `.ralph/tasks/bugs/bug-test-harness-runtime-path-dependent-kill-command.md`

```
## Bug: Test harness runtime kill command is PATH-dependent and bypasses provenance guarantees <status>done</status> <passes>true</passes>

<description>
Real-binary harness paths are intended to be explicit and provenance-controlled, but runtime teardown logic still invokes `kill` by bare name via `Command::new("kill")`.
```

==============

# Task `.ralph/tasks/bugs/dcs-watch-refresh-errors-ignored.md`

```
## Bug: DCS watch refresh errors are tracked but ignored <status>done</status> <passes>true</passes>

<description>
`refresh_from_etcd_watch` in [src/dcs/store.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dcs/store.rs) records `had_errors` (for unknown keys or decode failures) but no caller uses it. In [src/dcs/worker.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dcs/worker.rs), `step_once` only checks for `Err`, so unknown/malformed watch events can be silently ignored while the worker still reports healthy state. Decide on the correct behavior (e.g., mark store unhealthy, emit faulted state, or log/telemetry), and wire `had_errors` into worker health so errors do not pass silently.
</description>
```

==============

# Task `.ralph/tasks/bugs/docs-claims-drift-harness-readiness-and-chapter-shape.md`

```
## Bug: Contributor docs claims drift from implementation and contract <status>done</status> <passes>true</passes>

<description>
Two contributor-facing documentation defects were found during slice verification.
```

==============

# Task `.ralph/tasks/bugs/etcd-watch-bootstrap-startup-timeout-and-resnapshot-stale-events.md`

```
## Bug: Etcd watch bootstrap can hang startup and resnapshot can replay stale events <status>done</status> <passes>true</passes>

<description>
The etcd DCS store watch worker has subtle correctness issues in bootstrap/reconnect handling.
```

==============

# Task `.ralph/tasks/bugs/fencing-cutoff-commit-timestamp-zero-fallback-undercounts.md`

```
## Bug: Fencing cutoff commit timestamp fallback undercounts post-cutoff commits <status>done</status> <passes>true</passes>

<description>
In `src/ha/e2e_multi_node.rs`, successful SQL commits record `committed_at_unix_ms` using `ha_e2e::util::unix_now()`, but on error the code falls back to `0` (`Err(_) => 0`).
```

==============

# Task `.ralph/tasks/bugs/gate-audit-timeout-silent-pass-hardening.md`

```
## Bug: Harden make gates against hangs and silent passes <status>done</status> <passes>true</passes>

<description>
`make test`, `make test-long`, `make lint`, and `make check` currently have uneven timeout behavior and incomplete pass assertions.
```

==============

# Task `.ralph/tasks/bugs/ha-matrix-scenario-flakes-under-real-ha.md`

```
## Bug: HA Matrix Scenario Flakes Under Real HA <status>done</status> <passes>true</passes>

<description>
The deleted `e2e_multi_node_real_ha_scenario_matrix` mega-scenario was non-deterministic under real binaries.
During repeated reproductions it oscillated between:
```

==============

# Task `.ralph/tasks/bugs/kill-path-injection-ha-e2e-util.md`

```
## Bug: HA e2e util executes PATH-resolved kill for process control <status>done</status> <passes>true</passes>

<description>
`src/test_harness/ha_e2e/util.rs` uses `tokio::process::Command::new("kill")` both to send signals and to probe liveness (`kill -0`).
```

==============

# Task `.ralph/tasks/bugs/pginfo-standby-polling-test-configure-primary-db-error.md`

```
## Bug: Pginfo standby polling test fails during primary configure with db error <status>done</status> <passes>true</passes>

<description>
`make test` failed in `pginfo::worker::tests::step_once_maps_replica_when_polling_standby` with a runtime panic while preparing the primary postgres fixture.
```

==============

# Task `.ralph/tasks/bugs/process-worker-real-job-tests-accept-failure-outcomes.md`

```
## Bug: Process worker real job tests accept failure outcomes <status>done</status> <passes>true</passes>

<description>
Real-binary process worker tests in [src/process/worker.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/process/worker.rs) accept failure outcomes, so they can pass even when the binary invocation or behavior is broken. Examples:
- `real_promote_job_executes_binary_path`
```

==============

# Task `.ralph/tasks/bugs/process-worker-real-job-tests-state-channel-closed.md`

```
## Bug: Process worker real job tests fail with state channel closed <status>done</status> <passes>true</passes>

<description>
`make test` failed while running real process worker job tests. Multiple tests panic because process state publish fails with `state channel is closed`.
```

==============

# Task `.ralph/tasks/bugs/provenance-missing-helper-functions-break-lib-test-compile.md`

```
## Bug: Provenance Helpers Missing Break Lib Test Compile <status>done</status> <passes>true</passes>

<description>
`src/test_harness/provenance.rs` calls helper functions that are not defined in scope:
- `verify_policy_optional_pins`
```

==============

# Task `.ralph/tasks/bugs/real-binary-tests-fail-when-port-allocation-is-blocked.md`

```
## Bug: Real-binary tests fail when port allocation is blocked <status>done</status> <passes>true</passes>

<description>
`make test` fails in the current environment because multiple tests panic when `allocate_ports(...)` returns `io error: Operation not permitted (os error 1)`.
```

==============

# Task `.ralph/tasks/bugs/remove-panics-expects-unwraps.md`

```
## Bug: Remove panics/expects/unwraps in codebase <status>done</status> <passes>true</passes>

<description>
`rg -n "unwrap\(|expect\(|panic!" src tests` shows multiple occurrences (mostly in tests and some src modules like `src/process/worker.rs`, `src/pginfo/state.rs`, `src/pginfo/query.rs`, `src/dcs/worker.rs`, `src/dcs/store.rs`, `src/ha/worker.rs`, `tests/bdd_state_watch.rs`, `src/config/parser.rs`). Policy requires no unwraps/panics/expects anywhere; replace with proper error handling and remove any lint exemptions if present. Explore and confirm current behavior before changing.
</description>
```

==============

# Task `.ralph/tasks/bugs/test-harness-binary-check-panics.md`

```
## Bug: Test harness binary checks panic instead of returning errors <status>done</status> <passes>true</passes>

<description>
The test harness binary lookup in [src/test_harness/binaries.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/test_harness/binaries.rs) uses `panic!` to report missing binaries. This conflicts with the project policy of no `panic`/`expect`/`unwrap` and makes tests fail via uncontrolled panics rather than structured errors. Refactor `require_binary` (and callers) to return a typed `HarnessError` instead of panicking, and update callers/tests to propagate or assert errors explicitly.
</description>
```

==============

# Task `.ralph/tasks/bugs/worker-contract-tests-assert-only-callability.md`

```
## Bug: Worker contract tests only assert callability <status>done</status> <passes>true</passes>

<description>
[worker_contract_tests.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/worker_contract_tests.rs) primarily asserts that `step_once` functions are callable and return `Ok(())`, without validating resulting state changes or side effects. This means tests can pass even if core worker logic regresses or stops mutating state. Strengthen these tests with minimal behavioral assertions (state version bump, expected phase transitions, or expected publish effects), or split compile-time contract checks into non-test compile gates and add real behavioral tests.
</description>
```

==============

# Task `.ralph/tasks/story-full-verification/01-task-verify-build-and-static-gates.md`

```
## Task: Verify build and static quality gates <status>done</status> <passes>true</passes>

<description>
**Goal:** Validate the codebase can build and pass core static gates before deeper test execution.
```

==============

# Task `.ralph/tasks/story-full-verification/02-task-run-targeted-unit-and-integration-tests.md`

```
## Task: Run targeted unit and integration test suites <status>done</status> <passes>true</passes>

<description>
**Goal:** Execute and validate non-e2e automated tests after static/build gates to identify functional regressions early.
```

==============

# Task `.ralph/tasks/story-full-verification/03-task-run-full-suite-regression-pass.md`

```
## Task: Run full regression suite end-to-end <status>done</status> <passes>true</passes>

<description>
**Goal:** Execute the entire validation suite in one pass to confirm holistic repository health.
```

==============

# Task `.ralph/tasks/story-full-verification/04-task-resolve-failures-and-revalidate-full-suite.md`

```
## Task: Resolve discovered failures and revalidate full suite <status>done</status> <passes>true</passes>

<description>
**Goal:** Drive failure resolution from created bug tasks and confirm full-suite green status after fixes.
```

==============

# Task `.ralph/tasks/story-ha-functional-rewrite/01-task-remove-restore-control-plane-before-ha-functional-rewrite.md`

```
## Task: Remove restore control plane before HA functional rewrite <status>done</status> <passes>true</passes>

<description>
**Goal:** Delete the restore takeover control plane from HA, DCS, API, CLI, debug, and tests before rewriting HA around a functional state-machine design.
```

==============

# Task `.ralph/tasks/story-ha-functional-rewrite/02-task-rewrite-ha-decide-into-facts-and-phase-outcome-match-machine.md`

```
## Task: Rewrite HA decide into a facts-and-PhaseOutcome match machine <status>done</status> <passes>true</passes>

<description>
**Goal:** Replace mutation-driven HA decision code with a pure, match-based state machine that gathers immutable facts once and returns a full `PhaseOutcome { next_phase, decision }` directly from each phase handler.
```

==============

# Task `.ralph/tasks/story-ha-functional-rewrite/03-task-replace-action-vectors-and-pending-state-with-typed-domain-effect-plan.md`

```
## Task: Replace action vectors and pending state with HaDecision plus lowered effect plan <status>done</status> <passes>true</passes>

<description>
**Goal:** Replace `Vec<HaAction>` planning with a high-level `HaDecision` enum plus an inherent `HaDecision::lower(&self) -> HaEffectPlan` step, and remove `pending` entirely from HA state.
```

==============

# Task `.ralph/tasks/story-ha-functional-rewrite/04-task-untangle-ha-worker-into-facts-plan-and-apply-layers.md`

```
## Task: Untangle HA worker into facts, plan, and apply layers <status>done</status> <passes>true</passes>

<description>
**Goal:** Restructure HA runtime code so the worker clearly separates fact collection, pure decision selection, effect lowering, and effect application without forcing the design into object-heavy “executor” patterns.
```

==============

# Task `.ralph/tasks/story-ha-functional-rewrite/05-task-rebuild-ha-tests-around-invariants-and-continuous-observers.md`

```
## Task: Rebuild HA tests around invariants and continuous observers <status>completed</status> <passes>true</passes>

<description>
**Goal:** Rework HA tests so they validate the new functional architecture directly, using immutable builders for pure decision tests and continuous invariant observers for integration/e2e scenarios.
```

==============

# Task `.ralph/tasks/story-ha-functional-rewrite/06-task-move-and-split-ha-e2e-tests-after-functional-rewrite.md`

```
## Task: Move and split HA e2e tests after the functional rewrite <status>done</status> <passes>true</passes>

<description>
After the HA functional rewrite lands, move and restructure the HA end-to-end tests so they are no longer oversized mixed files living under `src/ha/`.
```

==============

# Task `.ralph/tasks/story-operator-architecture-docs/01-task-restructure-operator-docs-for-flow-depth-and-rationale.md`

```
## Task: Restructure Operator Docs for Better Flow, Depth, and Decision Rationale <status>done</status> <passes>true</passes>

<description>
**Goal:** Rebuild the mdBook documentation into an operator-first guide that explains not only what the system does, but why it behaves that way and which tradeoffs drive key HA decisions.
```

==============

# Task `.ralph/tasks/story-operator-architecture-docs/02-task-post-rewrite-skeptical-claim-verification-with-spark.md`

```
## Task: Post-Rewrite Skeptical Claim Verification with 15+ Parallel Spark Subagents <status>done</status> <passes>true</passes>

<description>
**Goal:** After the operator-doc transformation is complete, run a deep, adversarial verification of every claim in the docs using many independent `spark` subagents, and resolve all mismatches before finalizing docs.
```

==============

# Task `.ralph/tasks/story-operator-architecture-docs/03-task-expand-contributor-docs-into-full-implementation-deep-dive.md`

```
## Task: Expand Contributor Docs into a Full Implementation Deep Dive <status>done</status> <passes>true</passes>

<description>
**Goal:** Rewrite the Contributors section into an in-depth engineering deep dive that explains how the code actually works, how modules connect, and how behavior flows through runtime paths, while keeping prose natural, readable, and technically precise.
```

==============

# Task `.ralph/tasks/story-remove-backup-feature/01-task-remove-backup-config-and-process-surface.md`

```
## Task: Remove backup config and pgBackRest process vocabulary while keeping basebackup replica cloning <status>completed</status> <passes>true</passes> <priority>high</priority>

<description>
**Goal:** Delete the backup feature's config and process-language surface completely, while preserving `pg_basebackup`-based replica creation as a non-backup bootstrap path.
This story is an immediate blocker: the backup feature must be removed before continuing broader rewrite work, because the leftover pgBackRest/archive/restore surface keeps reintroducing complexity and false dependencies across the runtime.
```

==============

# Task `.ralph/tasks/story-remove-backup-feature/02-task-remove-runtime-restore-bootstrap-and-archive-helper-wiring.md`

```
## Task: Remove runtime restore bootstrap and the archive_command helper/proxy wiring <status>completed</status> <passes>true</passes> <priority>high</priority>

<description>
**Goal:** Delete the runtime-owned restore bootstrap path and the hacky archive/restore helper stack, including the local event-ingest API used only for archive_command/restore_command passthrough logging.
This is now a top-priority blocker inside backup removal, because the surviving `archive_command`, `restore_command`, helper JSON sidecar, and WAL passthrough path are the most disruptive remaining pieces for debugging and further refactoring.
```

==============

# Task `.ralph/tasks/story-remove-backup-feature/04-task-remove-backup-harness-installers-and-gate-selection.md`

```
## Task: Remove backup-specific harness, installer, and gate-selection surfaces while preserving real tests for replica cloning <status>done</status> <passes>true</passes> <priority>high</priority>

<description>
**Goal:** Delete the backup feature's harness and packaging residue so real-binary verification no longer provisions or expects pgBackRest, while preserving real coverage for normal Postgres and replica-clone behavior.
This cleanup is part of the same immediate removal story and should follow the code-path deletion without being deferred to a later general cleanup pass.
```

==============

# Task `.ralph/tasks/story-rust-system-harness/01-task-core-types-time-errors-watch-channel.md`

```
## Task: Implement core ids time errors and typed watch channels <status>done</status> <passes>true</passes> <priority>ultra_high</priority>

<description>
**Goal:** Build the foundational shared types and state-channel primitives used by every worker.
```

==============

# Task `.ralph/tasks/story-rust-system-harness/02-task-runtime-config-schema-defaults-parse-validate.md`

```
## Task: Implement runtime config schema defaults parser and validation <status>done</status> <passes>true</passes> <priority>ultra_high</priority>

<blocked_by>01-task-core-types-time-errors-watch-channel</blocked_by>
<superseded_by>story-pgbackrest-managed-backup-recovery/03-task-high-prio-remove-shell-archive-wrapper-and-current-wiring</superseded_by>
<superseded_by>story-pgbackrest-managed-backup-recovery/04-task-rust-generic-argv-passthrough-binary-for-postgres-archive-restore-logging</superseded_by>
```

==============

# Task `.ralph/tasks/story-rust-system-harness/03-task-worker-state-models-and-context-contracts.md`

```
## Task: Define worker state models and run step_once contracts <status>done</status> <passes>true</passes> <priority>ultra_high</priority>

<blocked_by>02-task-runtime-config-schema-defaults-parse-validate</blocked_by>

<description>
```

==============

# Task `.ralph/tasks/story-rust-system-harness/04-task-pginfo-worker-single-query-and-real-pg-tests.md`

```
## Task: Implement pginfo worker single-query polling and real PG tests <status>done</status> <passes>true</passes> <priority>high</priority>

<blocked_by>03-task-worker-state-models-and-context-contracts</blocked_by>

<description>
```

==============

# Task `.ralph/tasks/story-rust-system-harness/05-task-dcs-worker-trust-cache-watch-member-publish.md`

```
## Task: Implement DCS worker trust evaluation cache updates and member publishing <status>done</status> <passes>true</passes> <priority>high</priority>

<blocked_by>03-task-worker-state-models-and-context-contracts</blocked_by>

<description>
```

==============

# Task `.ralph/tasks/story-rust-system-harness/05a-task-enforce-strict-rust-lints-no-unwrap-expect-panic.md`

```
## Task: Enforce strict Rust lint policy and forbid unwrap expect panic in runtime code <status>done</status> <passes>true</passes> <priority>ultra_high</priority>

<description>
**Goal:** Install and enforce strict Rust linting with explicit denial of `unwrap`, `expect`, and panic-prone patterns in runtime code.
```

==============

# Task `.ralph/tasks/story-rust-system-harness/05b-task-deep-review-codebase-and-verify-done-work.md`

```
## Task: Deep review codebase quality and verify done tasks are truly complete <status>done</status> <passes>true</passes> <priority>ultra_high</priority>

<description>
**Goal:** Perform a deep end-to-end review of current repository quality, test reality, and completion truthfulness of all tasks already marked as done.
```

==============

# Task `.ralph/tasks/story-rust-system-harness/05c-task-zero-panic-unwrap-expect-across-runtime-and-tests.md`

```
## Task: Enforce zero panic/unwrap/expect across runtime and tests with proper Result handling <status>done</status> <passes>true</passes> <priority>high</priority>

<description>
**Goal:** Remove all manual panic/unwrap/expect usage from runtime and test code, replace with proper Rust error handling, and make lint enforcement fail on any regression.
```

==============

# Task `.ralph/tasks/story-rust-system-harness/06-task-process-worker-single-active-job-real-job-exec.md`

```
## Task: Implement process worker single-active-job execution with real job tests <status>done</status> <passes>true</passes> <priority>high</priority>

<blocked_by>03-task-worker-state-models-and-context-contracts</blocked_by>

<description>
```

==============

# Task `.ralph/tasks/story-rust-system-harness/07-task-ha-decide-pure-matrix-idempotency-tests.md`

```
## Task: Implement pure HA decide engine with exhaustive transition tests <status>done</status> <passes>true</passes> <priority>ultra_high</priority>

<blocked_by>03-task-worker-state-models-and-context-contracts</blocked_by>

<description>
```

==============

# Task `.ralph/tasks/story-rust-system-harness/08-task-ha-worker-select-loop-and-action-dispatch.md`

```
## Task: Implement HA worker select loop and action dispatch wiring <status>done</status> <passes>true</passes> <priority>high</priority>

<blocked_by>04-task-pginfo-worker-single-query-and-real-pg-tests,05-task-dcs-worker-trust-cache-watch-member-publish,06-task-process-worker-single-active-job-real-job-exec,07-task-ha-decide-pure-matrix-idempotency-tests</blocked_by>

<description>
```

==============

# Task `.ralph/tasks/story-rust-system-harness/09-task-api-debug-workers-and-snapshot-contracts.md`

```
## Task: Implement API and Debug API workers with typed contracts <status>done</status> <passes>true</passes> <priority>high</priority>

<blocked_by>05-task-dcs-worker-trust-cache-watch-member-publish,08-task-ha-worker-select-loop-and-action-dispatch</blocked_by>

<description>
```

==============

# Task `.ralph/tasks/story-rust-system-harness/10-task-test-harness-namespace-ports-pg-etcd-spawners.md`

```
## Task: Build parallel-safe real-system test harness for PG16 and etcd3 <status>done</status> <passes>true</passes> <priority>ultra_high</priority>

<blocked_by>02-task-runtime-config-schema-defaults-parse-validate,03-task-worker-state-models-and-context-contracts</blocked_by>

<description>
```

==============

# Task `.ralph/tasks/story-rust-system-harness/10a-task-enforce-real-binary-tests-and-ci-prereqs.md`

```
## Task: Enforce real-binary test execution (PG16 + etcd3) via explicit gate + CI prerequisites <status>done</status> <passes>true</passes> <priority>high</priority>

<description>
**Goal:** Ensure “real-system” tests actually exercise real PostgreSQL 16 and etcd3 binaries in at least one deterministic gate (CI and/or developer opt-in), instead of silently reporting a pass via early-return skips.
```

==============

# Task `.ralph/tasks/story-rust-system-harness/10b-task-dcs-real-etcd3-store-adapter-and-tests.md`

```
## Task: Implement real etcd3-backed DCS store adapter and integration tests <status>done</status> <passes>true</passes> <priority>high</priority>

<description>
**Goal:** Add a production-grade `DcsStore` implementation backed by a real etcd3 instance, and prove it via integration tests using the existing test harness spawner.
```

==============

# Task `.ralph/tasks/story-rust-system-harness/11-task-typed-pg-config-and-conninfo-roundtrip-tests.md`

```
## Task: Implement typed postgres config and conninfo parser renderer <status>done</status> <passes>true</passes> <priority>high</priority>

<blocked_by>02-task-runtime-config-schema-defaults-parse-validate</blocked_by>

<description>
```

==============

# Task `.ralph/tasks/story-rust-system-harness/12-task-ha-loop-integration-tests-real-watchers-and-step-once.md`

```
## Task: Build HA loop integration tests with real watchers and deterministic stepping <status>done</status> <passes>true</passes> <priority>ultra_high</priority>

<blocked_by>08-task-ha-worker-select-loop-and-action-dispatch,10-task-test-harness-namespace-ports-pg-etcd-spawners</blocked_by>

<description>
```

==============

# Task `.ralph/tasks/story-rust-system-harness/13-task-e2e-multi-node-real-ha-loops-scenario-matrix.md`

```
## Task: Implement e2e multi-node real HA-loop scenario matrix <status>done</status> <passes>true</passes> <priority>ultra_high</priority>

<blocked_by>09-task-api-debug-workers-and-snapshot-contracts,10-task-test-harness-namespace-ports-pg-etcd-spawners,12-task-ha-loop-integration-tests-real-watchers-and-step-once</blocked_by>

<description>
```

==============

# Task `.ralph/tasks/story-rust-system-harness/14-task-security-auth-tls-real-cluster-tests.md`

```
## Task: Implement security auth TLS validation tests in real cluster runs <status>done</status> <passes>true</passes> <priority>high</priority>

<blocked_by>10-task-test-harness-namespace-ports-pg-etcd-spawners,13-task-e2e-multi-node-real-ha-loops-scenario-matrix</blocked_by>

<description>
```

==============

# Task `.ralph/tasks/story-rust-system-harness/15-task-final-double-check-and-stop-gate.md`

```
## Task: Final double-check gate for real testing completeness <status>done</status> <passes>true</passes> <priority>ultra_high</priority>

<blocked_by>13-task-e2e-multi-node-real-ha-loops-scenario-matrix,14-task-security-auth-tls-real-cluster-tests</blocked_by>

<description>
```

==============

# Task `.ralph/tasks/story-rust-system-harness/16-task-debug-ui-verbose-state-actions-events-and-final-stop.md`

```
## Task: Setup verbose debug UI and final STOP gate <status>done</status> <passes>true</passes> <priority>low</priority>

<blocked_by>15-task-final-double-check-and-stop-gate</blocked_by>

<description>
```

==============

# Task `.ralph/tasks/story-rust-system-harness/22-task-ha-admin-api-read-write-surface.md`

```
## Task: Expose full HA admin API read and write surface <status>done</status> <passes>true</passes>

<description>
**Goal:** Add a first-class HA admin API that exposes operational read endpoints and write actions needed to control cluster behavior without touching DCS directly.
```

==============

# Task `.ralph/tasks/story-rust-system-harness/23-task-ha-admin-cli-over-api.md`

```
## Task: Build a simple Rust HA admin CLI over the exposed API <status>done</status> <passes>true</passes>

<blocked_by>22-task-ha-admin-api-read-write-surface</blocked_by>

<description>
```

==============

# Task `.ralph/tasks/story-rust-system-harness/24-task-real-e2e-harness-3nodes-3etcd.md`

```
## Task: Upgrade real e2e harness to 3 pgtuskmaster nodes and 3 etcd members <status>done</status> <passes>true</passes>

<description>
**Goal:** Make the real e2e environment represent a true 3-node HA control plane with a 3-member etcd cluster instead of a single etcd instance.
```

==============

# Task `.ralph/tasks/story-rust-system-harness/25-task-enforce-e2e-api-only-control-no-direct-dcs.md`

```
## Task: Enforce API-only control in e2e and ban direct DCS mutations <status>done</status> <passes>true</passes>

<blocked_by>22-task-ha-admin-api-read-write-surface</blocked_by>
<blocked_by>24-task-real-e2e-harness-3nodes-3etcd</blocked_by>
```

==============

# Task `.ralph/tasks/story-rust-system-harness/26-task-e2e-unassisted-failover-sql-consistency.md`

```
## Task: Add unassisted failover e2e with before/after SQL consistency proof <status>done</status> <passes>true</passes>

<blocked_by>24-task-real-e2e-harness-3nodes-3etcd</blocked_by>
<blocked_by>25-task-enforce-e2e-api-only-control-no-direct-dcs</blocked_by>
```

==============

# Task `.ralph/tasks/story-rust-system-harness/27-task-e2e-ha-stress-workloads-during-role-changes.md`

```
## Task: Add HA stress e2e suites with concurrent SQL workloads during role changes <status>done</status> <passes>true</passes>

<blocked_by>24-task-real-e2e-harness-3nodes-3etcd</blocked_by>
<blocked_by>25-task-enforce-e2e-api-only-control-no-direct-dcs</blocked_by>
```

==============

# Task `.ralph/tasks/story-rust-system-harness/28-task-e2e-network-partition-chaos-no-split-brain.md`

```
## Task: Add network partition e2e chaos tests with proxy fault injection <status>completed</status> <passes>true</passes>

<blocked_by>24-task-real-e2e-harness-3nodes-3etcd</blocked_by>

<description>
```

==============

# Task `.ralph/tasks/story-rust-system-harness/29-task-e2e-tls-adversarial-cert-validation.md`

```
## Task: Expand TLS adversarial e2e tests for certificate validation hardening <status>done</status> <passes>true</passes>

<blocked_by>22-task-ha-admin-api-read-write-surface</blocked_by>

<description>
```

==============

# Task `.ralph/tasks/story-rust-system-harness/30-task-full-e2e-blackbox-api-cli-orchestration.md`

```
## Task: Migrate full e2e suites to black-box API and CLI orchestration <status>completed</status> <passes>true</passes>

<blocked_by>22-task-ha-admin-api-read-write-surface</blocked_by>
<blocked_by>23-task-ha-admin-cli-over-api</blocked_by>
<blocked_by>25-task-enforce-e2e-api-only-control-no-direct-dcs</blocked_by>
```

==============

# Task `.ralph/tasks/story-rust-system-harness/31-task-docs-framework-selection-install-and-artifact-hygiene.md`

```
## Task: Install mdBook docs framework and enforce artifact git hygiene <status>completed</status> <passes>true</passes>

<description>
**Goal:** Use mdBook for this Rust project, install it, prove it renders a static HTML site correctly, and lock down strict git artifact hygiene before any docs commits.
```

==============

# Task `.ralph/tasks/story-rust-system-harness/32-task-author-complete-architecture-docs-with-diagrams-and-no-code.md`

```
## Task: Author full architecture documentation with rich diagrams and zero code-level narration <status>completed</status> <passes>true</passes>

<blocked_by>31-task-docs-framework-selection-install-and-artifact-hygiene</blocked_by>

<description>
```

==============

# Task `.ralph/tasks/story-rust-system-harness/33-task-deep-skeptical-verification-of-doc-facts-and-writing-quality.md`

```
## Task: Perform deep skeptical verification of all docs facts and writing quality <status>completed</status> <passes>true</passes>

<blocked_by>32-task-author-complete-architecture-docs-with-diagrams-and-no-code</blocked_by>

<description>
```

==============

# Task `.ralph/tasks/story-rust-system-harness/34-task-add-non-test-unified-node-entrypoint-autobootstrap-and-ha-loop.md`

```
## Task: Add non-test unified node entrypoint from start through autonomous HA loop <status>done</status> <passes>true</passes> <priority>high</priority>

<description>
**Goal:** Provide one production (non-test) entry path that starts a `pgtuskmaster` node from config only and runs it through bootstrap and HA loop without manual orchestration.
```

==============

# Task `.ralph/tasks/story-rust-system-harness/35-task-migrate-all-node-startup-tests-to-unified-entrypoint-config-only.md`

```
## Task: Migrate all node-starting tests to unified entrypoint (config-only) <status>done</status> <passes>true</passes> <priority>high</priority>

<blocked_by>34-task-add-non-test-unified-node-entrypoint-autobootstrap-and-ha-loop</blocked_by>

<description>
```

==============

# Task `.ralph/tasks/story-rust-system-harness/36-task-enforce-post-startup-hands-off-test-policy-no-direct-coordination.md`

```
## Task: Enforce post-startup hands-off test policy (no direct coordination) <status>completed</status> <passes>true</passes> <priority>high</priority>

<blocked_by>35-task-migrate-all-node-startup-tests-to-unified-entrypoint-config-only</blocked_by>

<description>
```

==============

# Task `.ralph/tasks/story-rust-system-harness/37-task-unified-e2e-harness-testconfig-interface.md`

```
## Task: Unify HA E2E Harness Behind Stable `TestConfig` Interface <status>completed</status> <passes>true</passes>

<description>
**Goal:** Design and implement one stable, shared HA e2e harness interface driven by a single `TestConfig` input that initializes the requested cluster topology + pre-test setup, returns a full test handle, and removes duplicated setup/wait/process glue from scenario files.
```

==============

# Task `.ralph/tasks/story-rust-system-harness/38-task-unified-structured-logging-and-postgres-binary-ingestion.md`

```
## Task: Build Unified Structured Logging Pipeline With Postgres/Binary Ingestion <status>done</status> <passes>true</passes>

<description>
**Goal:** Implement one unified, config-driven logging system that emits structured JSONL to `stderr` by default, ingests/normalizes all postgres and helper-binary logs into the same stream, and guarantees no log loss on parse failures.
```

==============

# Task `.ralph/tasks/story-rust-system-harness/39-task-file-sink-support-for-structured-logging.md`

```
## Task: Add File Sink Support For Unified Structured Logging <status>done</status> <passes>true</passes>

<description>
**Goal:** Extend the unified structured logging subsystem to support configurable JSONL file sinks (in addition to the current stderr JSONL sink).
```

==============

# Task `.ralph/tasks/story-rust-system-harness/39-task-logging-file-sink-backlog.md`

```
## Task: Add Structured File Sink Support (Backlog) <status>done</status> <passes>true</passes>

<description>
**Goal:** Extend the unified logging subsystem with optional structured file sink support after the base structured-ingestion task is complete.
```

==============

# Task `.ralph/tasks/story-rust-system-harness/40-task-ultra-high-prio-test-target-split-and-reference-migration.md`

```
## Task: Ultra-high-priority migrate repo gates to `make test` + `make test-long` only <status>done</status> <passes>true</passes> <priority>ultra-high</priority>

<description>
**Goal:** Complete and verify the global migration from legacy test targets to only two test groups: `make test` (regular) and `make test-long` (ultra-long only).
```

==============

# Task `.ralph/tasks/story-rust-system-harness/41-task-ultra-high-prio-split-ultra-long-e2e-into-short-parallel-tests.md`

```
## Task: Ultra-high-priority split ultra-long e2e tests into shorter parallel real-binary tests <status>completed</status> <passes>true</passes> <priority>ultra-high</priority>

<description>
**Goal:** Replace the current ultra-long HA e2e stress scenario(s) with multiple shorter real-binary e2e tests that preserve full coverage and must run in parallel.
```

==============

# Task `.ralph/tasks/story-rust-system-harness/42-task-operator-grade-action-logging-and-no-silent-errors.md`

```
## Task: Enforce Operator-Grade Action Logging And No Silent Error Swallowing <status>done</status> <passes>true</passes>

<description>
**Goal:** Make runtime/operator observability explicit and uniform: debug-log all actions and all meaningful runtime flow steps across the codebase so operators can reconstruct exactly what code path executed, in order; info-log important operator lifecycle/default events; warn-log ignorable errors; error-log hard errors; and eliminate silent error swallowing.
```

==============

# Task `.ralph/tasks/story-rust-system-harness/task-real-ha-dcs-process-integration-tests.md`

```
## Task: Add real HA+DCS+Process integration tests <status>done</status> <passes>true</passes>

<description>
**Goal:** Build integration tests that wire real PG16 binaries, a real etcd-backed DCS store, the process worker, pginfo worker, and HA worker so failures cannot pass silently.
```

==============

# Task `.ralph/tasks/story-rust-system-harness/task-typed-dcs-writes-and-encapsulation.md`

```
## Task: Replace Stringly DCS Writes With Typed Writer API <status>done</status> <passes>true</passes>

<description>
**Goal:** Eliminate raw path/string DCS writes from HA by introducing a typed DCS writer API and restricting access to low-level write/delete operations.
```

==============

# Task `.ralph/tasks/story-secure-explicit-node-config/01-task-expand-runtime-config-schema-for-explicit-secure-node-startup.md`

```
## Task: Expand runtime config schema for explicit secure node startup <status>done</status> <passes>true</passes>

<description>
**Goal:** Redesign the runtime config model so every required secure startup setting is explicitly represented (TLS, HTTP, PostgreSQL hosting, roles/auth, pg_hba/pg_ident, and DCS init config).
```

==============

# Task `.ralph/tasks/story-secure-explicit-node-config/02-task-migrate-parser-defaults-and-validation-to-explicit-enum-driven-config.md`

```
## Task: Migrate parser/defaults/validation to explicit enum-driven config semantics <status>done</status> <passes>true</passes>

<description>
**Goal:** Remove hidden config inference by moving defaulting/validation behavior to explicit enum-driven semantics while preserving safe startup requirements.
```

==============

# Task `.ralph/tasks/story-secure-explicit-node-config/03-task-enforce-role-specific-credential-usage-across-runtime.md`

```
## Task: Enforce role-specific credential usage across runtime operations <status>done</status> <passes>true</passes>

<description>
**Goal:** Ensure each runtime function uses only its designated role (`superuser`, `replicator`, `rewinder`) and corresponding auth mode from config.
```

==============

# Task `.ralph/tasks/story-secure-explicit-node-config/04-task-wire-http-pg-tls-pg_hba-pg_ident-and-dcs-init-into-startup.md`

```
## Task: Wire HTTP/PG TLS, pg_hba/pg_ident, and DCS init config into startup orchestration <status>done</status> <passes>true</passes>

<description>
**Goal:** Make startup consume the expanded config end-to-end so node boot requires explicit secure config and does not infer missing values.
```

==============

# Task `.ralph/tasks/story-secure-explicit-node-config/05-task-migrate-fixtures-examples-and-cli-config-surfaces-to-new-schema.md`

```
## Task: Migrate fixtures/examples/CLI config surfaces to the secure explicit schema <status>done</status> <passes>true</passes>

<description>
**Goal:** Align all config producers/consumers (tests, examples, CLI entrypoints) with the expanded schema and explicit secure requirements.
```

==============

# Task `.ralph/tasks/story-secure-explicit-node-config/06-task-full-verification-for-secure-explicit-config-refactor.md`

```
## Task: Run full verification for secure explicit config refactor <status>done</status> <passes>true</passes>

<description>
**Goal:** Execute full validation gates after the config refactor and convert any failures into actionable bug tasks.
```

