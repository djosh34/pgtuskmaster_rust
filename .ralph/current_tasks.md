# Current Tasks Summary

Generated: Fri Mar  6 06:17:00 AM CET 2026

**Path:** `.ralph/tasks/bugs/bug-bdd-http-tests-false-pass-via-fragile-status-and-read-patterns.md`

## Bug: BDD HTTP tests false-pass via fragile status and read patterns <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
BDD HTTP contract tests use weak status matching and response-read behavior that can hide protocol/handler regressions or induce hangs.

Detected during review of plan section 2.4 against:
- tests/bdd_api_http.rs:247-250, 298-301, 317-320, 336-339, 355-358, 407-410, 434-437, 455-458, 498-501, 539-542, 577, 598-601
  (status assertions use `contains("...")` instead of exact status code checks)

---

**Path:** `.ralph/tasks/bugs/bug-ha-e2e-false-pass-via-best-effort-polling-and-timestamp-fallback.md`

## Bug: HA e2e false-pass via best-effort polling and timestamp fallback <status>done</status> <passes>true</passes>

<description>
HA e2e assertions can pass without reliable cluster-wide observations during unstable windows.

Detected during review of plan section 2.4 against:
- src/ha/e2e_multi_node.rs:1395-1410 (`assert_no_dual_primary_window` ignores polling errors and returns success if every poll fails)
- src/ha/e2e_multi_node.rs:1412-1458 (`wait_for_all_failsafe` is explicitly best-effort but is used with all-node language in scenario logs)

---

**Path:** `.ralph/tasks/bugs/bug-real-binary-provenance-enforcement-gaps.md`

## Bug: Real-binary provenance enforcement gaps in installers and harness <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
Real-binary tooling currently enforces existence/executability but not strong provenance at runtime.

Detected during skeptical audit of `tools/install-postgres16.sh`, `tools/install-etcd.sh`, and `src/test_harness/binaries.rs`.

Key gaps:

---

**Path:** `.ralph/tasks/bugs/bug-real-binary-tests-are-optional-via-early-return.md`

## Bug: Real Binary Tests Become Optional Via Early Return <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
Several real-binary test paths silently return `Ok(())` when required binaries are not discovered (for example `None => return Ok(())`).
This makes critical runtime coverage optional and can mask regressions in HA/bootstrap/process behavior.

Explore and research the full codebase first, then implement a fix so real-binary tests are enforced instead of being skipped by default.
The solution should preserve clear error messages about missing prerequisites and keep CI/local workflows deterministic.

---

**Path:** `.ralph/tasks/bugs/bug-remove-unwrap-panic-allow.md`

## Bug: Remove Clippy Allowances For Unwrap/Panic <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
src/test_harness/mod.rs explicitly allows clippy unwrap/expect/panic, which violates the repo rule against unwraps, panics, or expects anywhere. This hides violations in test harness code and makes it easy to slip new ones in. Investigate all test_harness code (and any other modules) for unwrap/expect/panic usage, replace with proper error handling, and remove the lint allow attributes.
</description>

<acceptance_criteria>
- [x] `make check` — passes cleanly

---

**Path:** `.ralph/tasks/bugs/bug-remove-writable-ha-leader-api-and-ha-loop-test-steering.md`

## Bug: Remove writable HA leader API control path and enforce HA-loop-only leadership transitions <status>completed</status> <passes>true</passes> <passing>true</passing>

<description>
Investigation found that writable `/ha/leader` was introduced by task `22-task-ha-admin-api-read-write-surface` as part of a "full HA admin API read and write surface". In runtime code, `src/api/worker.rs` routes `POST /ha/leader` and `DELETE /ha/leader` to controller handlers in `src/api/controller.rs` that call `DcsHaWriter::write_leader_lease` / `delete_leader`, so external callers can directly mutate the leader key outside autonomous HA-loop decision flow.

This conflicts with lease/autonomous leadership expectations and enables direct DCS steering through API, including in e2e scenario code.

Research first, then fix end-to-end:

---

**Path:** `.ralph/tasks/bugs/bug-test-bdd-full-suite-hangs.md`

## Bug: test full-suite command hangs in real HA e2e <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
After updating `make test` to run `PGTUSKMASTER_REQUIRE_REAL_BINARIES=1 cargo test --all-targets -- --include-ignored`, the verification run did not complete within an extended runtime window (over 15 minutes observed on 2026-03-03).

Detection details:
- `make test` started and progressed through unit + real-binary tests.
- Output stalled at `ha::e2e_multi_node::e2e_multi_node_real_ha_scenario_matrix has been running for over 60 seconds` and never completed during the observed window.

---

**Path:** `.ralph/tasks/bugs/bug-test-harness-runtime-path-dependent-kill-command.md`

## Bug: Test harness runtime kill command is PATH-dependent and bypasses provenance guarantees <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
Real-binary harness paths are intended to be explicit and provenance-controlled, but runtime teardown logic still invokes `kill` by bare name via `Command::new("kill")`.

This creates a PATH-dependent execution path in real-binary e2e runs:
- `src/test_harness/pg16.rs` uses `Command::new("kill")` during postgres child shutdown.
- `src/test_harness/etcd3.rs` uses `Command::new("kill")` during etcd member shutdown.

---

**Path:** `.ralph/tasks/bugs/dcs-watch-refresh-errors-ignored.md`

## Bug: DCS watch refresh errors are tracked but ignored <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
`refresh_from_etcd_watch` in [src/dcs/store.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dcs/store.rs) records `had_errors` (for unknown keys or decode failures) but no caller uses it. In [src/dcs/worker.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dcs/worker.rs), `step_once` only checks for `Err`, so unknown/malformed watch events can be silently ignored while the worker still reports healthy state. Decide on the correct behavior (e.g., mark store unhealthy, emit faulted state, or log/telemetry), and wire `had_errors` into worker health so errors do not pass silently.
</description>

<acceptance_criteria>
- [x] `make check` — passes cleanly

---

**Path:** `.ralph/tasks/bugs/docs-claims-drift-harness-readiness-and-chapter-shape.md`

## Bug: Contributor docs claims drift from implementation and contract <status>done</status> <passes>true</passes>

<description>
Two contributor-facing documentation defects were found during slice verification.

1) `docs/src/contributors/harness-internals.md` states readiness is checked by connecting to the client port. Current etcd harness startup does port-connect checks per member (`wait_for_port`) and then performs an etcd KV round-trip readiness probe (`Client::connect` + put/get/delete) before considering the cluster ready. The doc wording is stale/incomplete.

2) `docs/src/contributors/docs-style.md` claims every contributor deep-dive chapter must include a minimum shape, including failure behavior, tradeoffs/sharp edges, and evidence pointers. At least `docs/src/contributors/codebase-map.md` does not currently satisfy that minimum contract as written.

---

**Path:** `.ralph/tasks/bugs/etcd-watch-bootstrap-startup-timeout-and-resnapshot-stale-events.md`

## Bug: Etcd watch bootstrap can hang startup and resnapshot can replay stale events <status>done</status> <passes>true</passes>

<description>
The etcd DCS store watch worker has subtle correctness issues in bootstrap/reconnect handling.

Detected during code audit of `src/dcs/etcd_store.rs`:
- `EtcdDcsStore::connect` waits only `COMMAND_TIMEOUT` for worker startup and then `join()`s the worker thread on timeout. If bootstrap (`connect + get + watch`) takes longer than that timeout, the join can block indefinitely while the worker continues running.
- On watch reconnect/resnapshot, bootstrap snapshot events are appended to the existing queue without clearing/draining stale pre-disconnect events. This can replay stale PUT events that should have been superseded by deletes included in the snapshot state.

---

**Path:** `.ralph/tasks/bugs/fencing-cutoff-commit-timestamp-zero-fallback-undercounts.md`

## Bug: Fencing cutoff commit timestamp fallback undercounts post-cutoff commits <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
In `src/ha/e2e_multi_node.rs`, successful SQL commits record `committed_at_unix_ms` using `ha_e2e::util::unix_now()`, but on error the code falls back to `0` (`Err(_) => 0`).

The no-quorum fencing assertion computes post-cutoff commits using `timestamp > cutoff_ms`. Any commit with fallback timestamp `0` is silently excluded, which can undercount post-cutoff commits and weaken (or falsely pass) the safety assertion.

Please explore and research the codebase first, then implement a fail-closed fix that does not use unwrap/panic/expect:

---

**Path:** `.ralph/tasks/bugs/gate-audit-timeout-silent-pass-hardening.md`

## Bug: Harden make gates against hangs and silent passes <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
`make test`, `make test-long`, `make lint`, and `make check` currently have uneven timeout behavior and incomplete pass assertions.

Observed issues from audit:
- `make test-long` has no timeout wrapper around `cargo test` executions, so one stalled real-binary test can block forever.
- `make test` has a timeout only around the final `cargo test` run, but not around preflight `cargo test -- --list`.

---

**Path:** `.ralph/tasks/bugs/ha-action-deduping-suppresses-retry.md`

## Bug: HA action dedupe suppresses legitimate retries <status>blocked</status> <passes>false</passes>

<blocked_by>06-task-move-and-split-ha-e2e-tests-after-functional-rewrite</blocked_by>

<description>
This bug is intentionally deferred until the HA functional rewrite story is fully complete. The current investigation already suggests the original report may be partly or fully stale on current `main`, and the remaining useful work may change shape substantially once the facts/decision/effect-plan/worker refactors land.

Do not pull this bug ahead of the rewrite. Reassess it only after `story-ha-functional-rewrite` is complete through its final task, then decide whether to:

---

**Path:** `.ralph/tasks/bugs/ha-decide-mutation-heavy-control-flow-needs-pure-refactor.md`

## Bug: HA decide mutation-heavy control flow needs pure refactor <status>blocked</status> <passes>false</passes>

<blocked_by>06-task-move-and-split-ha-e2e-tests-after-functional-rewrite</blocked_by>

<description>
This bug is intentionally deferred until the HA functional rewrite story is fully complete. It overlaps directly with the planned refactor work, and it does not make sense to force the bug queue to preempt the story that is supposed to absorb most or all of this concern.

Reassess this bug only after `story-ha-functional-rewrite` reaches its final task. At that point, answer a narrower question: how much mutation-heavy control flow is still present in the rewritten design, and what residual bug or cleanup work remains?

---

**Path:** `.ralph/tasks/bugs/ha-matrix-scenario-flakes-under-real-ha.md`

## Bug: HA Matrix Scenario Flakes Under Real HA <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
The deleted `e2e_multi_node_real_ha_scenario_matrix` mega-scenario was non-deterministic under real binaries.
During repeated reproductions it oscillated between:
- planned switchover never settling away from the original primary, even after multiple successful `/switchover` submissions
- all surviving nodes getting stuck in `WaitingPostgresReachable` with `leader=none`
- API transport resets while PostgreSQL/process workers continuously retried startup

---

**Path:** `.ralph/tasks/bugs/kill-path-injection-ha-e2e-util.md`

## Bug: HA e2e util executes PATH-resolved kill for process control <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
`src/test_harness/ha_e2e/util.rs` uses `tokio::process::Command::new("kill")` both to send signals and to probe liveness (`kill -0`).

This allows a PATH-prepended fake `kill` binary/script to be executed by tests or harness code, causing incorrect behavior and creating command-injection surface in test environments.

Investigate and replace shell-command `kill` usage with direct syscall-based signaling/liveness checks (or another non-PATH-resolved mechanism), then add regression tests that prove PATH-prepended fake `kill` is never executed.

---

**Path:** `.ralph/tasks/bugs/logging-archive-ingest-silent-failure-and-unsafe-cleanup.md`

## Bug: Postgres ingest silently swallows failures and cleanup/path ownership can destroy active observability signals <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
Postgres observability ingest has several correctness failures in the current logging pipeline:

1) `postgres_ingest::run()` suppresses errors from ingest and cleanup steps (`step_once` and `cleanup_log_dir`), so failures are silent and operators lose telemetry without actionable diagnostics.
2) `cleanup_log_dir()` only protects `pg_ctl_log_file` and can delete currently active files in `logging.postgres.log_dir` (for example active `postgres.json` and `postgres.stderr.log`), causing dropped logs.
3) No path ownership validation prevents sink/source overlap (for example `logging.sinks.file.path` overlapping tailed Postgres log files), which can create recursive self-ingestion loops and log amplification.

---

**Path:** `.ralph/tasks/bugs/pginfo-standby-polling-test-configure-primary-db-error.md`

## Bug: Pginfo standby polling test fails during primary configure with db error <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
`make test` failed in `pginfo::worker::tests::step_once_maps_replica_when_polling_standby` with a runtime panic while preparing the primary postgres fixture.

Repro:
- `make test`
- Failing test:

---

**Path:** `.ralph/tasks/bugs/process-worker-real-job-tests-accept-failure-outcomes.md`

## Bug: Process worker real job tests accept failure outcomes <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
Real-binary process worker tests in [src/process/worker.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/process/worker.rs) accept failure outcomes, so they can pass even when the binary invocation or behavior is broken. Examples:
- `real_promote_job_executes_binary_path`
- `real_demote_job_executes_binary_path`
- `real_restart_job_executes_binary_path`
- `real_fencing_job_executes_binary_path`

---

**Path:** `.ralph/tasks/bugs/process-worker-real-job-tests-state-channel-closed.md`

## Bug: Process worker real job tests fail with state channel closed <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
`make test` failed while running real process worker job tests. Multiple tests panic because process state publish fails with `state channel is closed`.

Repro:
- `make test`
- Failing tests:

---

**Path:** `.ralph/tasks/bugs/provenance-missing-helper-functions-break-lib-test-compile.md`

## Bug: Provenance Helpers Missing Break Lib Test Compile <status>done</status> <passes>true</passes>

<description>
`src/test_harness/provenance.rs` calls helper functions that are not defined in scope:
- `verify_policy_optional_pins`
- `verify_attestation_metadata`
- `verify_attested_entry_metadata`

---

**Path:** `.ralph/tasks/bugs/real-binary-tests-fail-when-port-allocation-is-blocked.md`

## Bug: Real-binary tests fail when port allocation is blocked <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
`make test` is not passing in the current environment because multiple tests panic when `allocate_ports(...)` returns `io error: Operation not permitted (os error 1)`.

Detected on 2026-03-02 with:
- `make test` (failed/terminated after reporting multiple failures and a long-running test)
- `cargo test test_harness::ports::tests::allocate_ports_returns_unique_ports -- --nocapture`

---

**Path:** `.ralph/tasks/bugs/remove-panics-expects-unwraps.md`

## Bug: Remove panics/expects/unwraps in codebase <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
`rg -n "unwrap\(|expect\(|panic!" src tests` shows multiple occurrences (mostly in tests and some src modules like `src/process/worker.rs`, `src/pginfo/state.rs`, `src/pginfo/query.rs`, `src/dcs/worker.rs`, `src/dcs/store.rs`, `src/ha/worker.rs`, `tests/bdd_state_watch.rs`, `src/config/parser.rs`). Policy requires no unwraps/panics/expects anywhere; replace with proper error handling and remove any lint exemptions if present. Explore and confirm current behavior before changing.
</description>

<acceptance_criteria>
- [x] `make check` — passes cleanly

---

**Path:** `.ralph/tasks/bugs/restore-terminal-phases-keep-ha-fencing.md`

## Bug: Restore terminal phases keep HA in repeated fencing <status>blocked</status> <passes>false</passes>

<blocked_by>05-task-remove-backup-docs-and-obsolete-task-artifacts</blocked_by>
<blocked_by>06-task-move-and-split-ha-e2e-tests-after-functional-rewrite</blocked_by>

<description>
This bug is intentionally deferred until the backup-removal story and the HA functional rewrite story are both fully complete. The restore control plane is being deleted, and the remaining HA core is being restructured; fixing this in the old design first would likely be throwaway work.

---

**Path:** `.ralph/tasks/bugs/test-harness-binary-check-panics.md`

## Bug: Test harness binary checks panic instead of returning errors <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
The test harness binary lookup in [src/test_harness/binaries.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/test_harness/binaries.rs) uses `panic!` to report missing binaries. This conflicts with the project policy of no `panic`/`expect`/`unwrap` and makes tests fail via uncontrolled panics rather than structured errors. Refactor `require_binary` (and callers) to return a typed `HarnessError` instead of panicking, and update callers/tests to propagate or assert errors explicitly.
</description>

<acceptance_criteria>
- [x] `make check` — passes cleanly

---

**Path:** `.ralph/tasks/bugs/unused-backup-recovery-mode-doc-configuration.md`

## Bug: backup.bootstrap.recovery_mode is documented but unused <status>blocked</status> <passes>false</passes>

<blocked_by>05-task-remove-backup-docs-and-obsolete-task-artifacts</blocked_by>

<description>
This bug is intentionally deferred until the backup-removal story is fully complete. The entire backup/restore config surface is scheduled for deletion, so this specific dead knob should be reassessed only after the story finishes end-to-end.

Reassess this bug after the final task in `story-remove-backup-feature`:

---

**Path:** `.ralph/tasks/bugs/worker-contract-tests-assert-only-callability.md`

## Bug: Worker contract tests only assert callability <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
[worker_contract_tests.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/worker_contract_tests.rs) primarily asserts that `step_once` functions are callable and return `Ok(())`, without validating resulting state changes or side effects. This means tests can pass even if core worker logic regresses or stops mutating state. Strengthen these tests with minimal behavioral assertions (state version bump, expected phase transitions, or expected publish effects), or split compile-time contract checks into non-test compile gates and add real behavioral tests.
</description>

<acceptance_criteria>
- [x] `make check` — passes cleanly

---

**Path:** `.ralph/tasks/story-container-first-deployment/01-task-container-first-docker-deployment-and-compose.md`

## Task: Container-first deployment baseline with Docker images, Compose stacks, and secrets <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Make container deployment the default operational path by adding production/development images and turnkey Docker Compose stacks that run etcd3 + pgtuskmaster with config maps and Docker secrets.

**Scope:**
- Add a production container image for `pgtuskmaster` nodes with PostgreSQL server/client binaries included (`postgres`, `pg_ctl`, `initdb`, `pg_basebackup`, `pg_rewind`, `psql`) and `pgtuskmaster` as entrypoint.
- Add a development container image variant that includes development tooling while keeping production image minimal (no Node/mdBook/runtime-unneeded tooling in prod).

---

**Path:** `.ralph/tasks/story-docs-useful-guides/01-task-rewrite-operator-docs-as-useful-user-guides.md`

## Task: Rewrite operator docs as useful user guides and remove horror pages <status>not_started</status> <passes>false</passes>

<description>
Rewrite the non-contributor documentation so it reads like a strong operator/product guide instead of a thin or awkwardly templated book.

The agent must explore the current docs and implementation first, then rewrite the docs around what actually helps a user understand and operate the system.

This task must implement the following fixed product decisions:

---

**Path:** `.ralph/tasks/story-docs-useful-guides/02-task-rebuild-contributor-docs-as-codebase-navigation-and-contract-guide.md`

## Task: Rebuild contributor docs as a codebase navigation and design-contract guide <status>not_started</status> <passes>false</passes>

<description>
Rewrite the contributor documentation so it becomes a genuinely useful guide for understanding the codebase, subsystem boundaries, implementation approach, and design contracts.

The agent must explore the current codebase and docs first, then rebuild contributor docs around the exact things a new contributor needs to learn:
- how to navigate the codebase
- which modules own which responsibilities

---

**Path:** `.ralph/tasks/story-docs-useful-guides/03-task-align-doc-file-order-and-names-with-rendered-site-structure.md`

## Task: Align doc file order and names with the rendered site structure <status>not_started</status> <passes>false</passes>

<description>
Make the docs source tree easier to navigate by aligning file names and ordering conventions with the rendered website structure.

The agent must explore the current docs tree, `SUMMARY.md`, and rendered navigation intent first, then implement the following fixed product decisions:
- docs source file naming should help a contributor understand the rendered order without guessing
- file names and ordering conventions should match the website structure closely enough that the source tree is not fighting the book navigation

---

**Path:** `.ralph/tasks/story-docs-useful-guides/04-task-create-repo-readme.md`

## Task: Create repository README as the front-door quick-start and project overview <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Add a normal, useful root `README.md` that explains what this project is, how to get started quickly, where to go next for deeper docs, and what the license status is.

**Scope:**
- Create a new root `README.md` for the repository.
- Keep it concise and practical: brief product explanation, quick-start-oriented getting-started section, common repo-entry information, and links into the mdBook docs for deeper operator/contributor material.

---

**Path:** `.ralph/tasks/story-full-verification/01-task-verify-build-and-static-gates.md`

## Task: Verify build and static quality gates <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
**Goal:** Validate the codebase can build and pass core static gates before deeper test execution.

**Scope:**
- Run initial repo health checks and build validation.
- Capture and classify any failures with concrete reproduction commands.

---

**Path:** `.ralph/tasks/story-full-verification/02-task-run-targeted-unit-and-integration-tests.md`

## Task: Run targeted unit and integration test suites <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
**Goal:** Execute and validate non-e2e automated tests after static/build gates to identify functional regressions early.

**Scope:**
- Run project test commands for unit/integration coverage.
- Isolate failures to test, code, or environment causes.

---

**Path:** `.ralph/tasks/story-full-verification/03-task-run-full-suite-regression-pass.md`

## Task: Run full regression suite end-to-end <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
**Goal:** Execute the entire validation suite in one pass to confirm holistic repository health.

**Scope:**
- Run all required project-level verification commands sequentially.
- Produce a consolidated pass/fail report.

---

**Path:** `.ralph/tasks/story-full-verification/04-task-resolve-failures-and-revalidate-full-suite.md`

## Task: Resolve discovered failures and revalidate full suite <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
**Goal:** Drive failure resolution from created bug tasks and confirm full-suite green status after fixes.

**Scope:**
- Execute bug tasks generated from prior verification tasks.
- Re-run full validation after each meaningful fix batch.

---

**Path:** `.ralph/tasks/story-greenfield-secure-config/01-task-remove-config-versioning-and-restore-a-greenfield-config-contract.md`

## Task: Remove config versioning and restore a greenfield config contract <status>not_started</status> <passes>false</passes>

<description>
Remove user-facing config versioning from the product and restore a simple greenfield config contract with no fake `v2` framing.

The agent must explore the current schema, parser, runtime assumptions, tests, and docs first, then implement the following fixed product decisions:
- there is no user-facing `config_version` field
- the parser must stop requiring or recognizing fake schema generations as the main product contract

---

**Path:** `.ralph/tasks/story-greenfield-secure-config/02-task-simplify-config-semantics-and-make-secure-mtls-the-documented-default.md`

## Task: Simplify config semantics and make secure mTLS the documented default <status>not_started</status> <passes>false</passes>

<description>
Rework the config contract and documentation so the supported settings make operational sense and the recommended setup is secure by default.

The agent must explore the current config model, docs, and runtime usage first, then implement the following fixed product decisions:
- the documented recommended setup uses TLS/mTLS by default for PostgreSQL, etcd, and the API
- the recommended config path uses CA/cert/key files for PostgreSQL, etcd, and the API

---

**Path:** `.ralph/tasks/story-greenfield-secure-config/03-task-derive-rewind-source-from-current-primary-instead-of-static-config.md`

## Task: Derive rewind source from the current primary instead of static config <status>not_started</status> <passes>false</passes>

<description>
Remove static rewind source addressing from the product and derive rewind behavior from current cluster state.

The agent must explore the current config schema, HA/runtime flow, DCS/member-state usage, process dispatch, and rewind execution path first, then implement the following fixed product decisions:
- a node rewinds from the current primary/leader, not from a permanently configured host/port
- static config fields for rewind source host/port are removed from the user-facing contract

---

**Path:** `.ralph/tasks/story-ha-functional-rewrite/01-task-remove-restore-control-plane-before-ha-functional-rewrite.md`

## Task: Remove restore control plane before HA functional rewrite <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
**Goal:** Delete the restore takeover control plane from HA, DCS, API, CLI, debug, and tests before rewriting HA around a functional state-machine design.

**Scope:**
- Edit `src/api/controller.rs`, `src/api/worker.rs`, `src/cli/client.rs`, `src/dcs/{state,keys,store,worker,etcd_store}.rs`, `src/ha/{actions,decide,worker,e2e_multi_node}.rs`, and any debug/test files still exposing restore state.
- Remove restore request/status DCS records and restore-specific API/debug/CLI surfaces.

---

**Path:** `.ralph/tasks/story-ha-functional-rewrite/02-task-rewrite-ha-decide-into-facts-and-phase-outcome-match-machine.md`

## Task: Rewrite HA decide into a facts-and-PhaseOutcome match machine <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
**Goal:** Replace mutation-driven HA decision code with a pure, match-based state machine that gathers immutable facts once and returns a full `PhaseOutcome { next_phase, decision }` directly from each phase handler.

**Scope:**
- Edit `src/ha/{decide,state,mod}.rs` and any new decision modules needed for the rewrite.
- Introduce an immutable facts struct gathered once per tick before phase selection.

---

**Path:** `.ralph/tasks/story-ha-functional-rewrite/03-task-replace-action-vectors-and-pending-state-with-typed-domain-effect-plan.md`

## Task: Replace action vectors and pending state with HaDecision plus lowered effect plan <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Replace `Vec<HaAction>` planning with a high-level `HaDecision` enum plus an inherent `HaDecision::lower(&self) -> HaEffectPlan` step, and remove `pending` entirely from HA state.

**Scope:**
- Edit `src/ha/{actions,state,decide,worker,mod}.rs`, `src/api/{mod,controller}.rs`, `src/cli/{client,output}.rs`, and all affected tests.
- Replace `Vec<HaAction>`-style planning with:

---

**Path:** `.ralph/tasks/story-ha-functional-rewrite/04-task-untangle-ha-worker-into-facts-plan-and-apply-layers.md`

## Task: Untangle HA worker into facts, plan, and apply layers <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Restructure HA runtime code so the worker clearly separates fact collection, pure decision selection, effect lowering, and effect application without forcing the design into object-heavy “executor” patterns.

**Scope:**
- Edit `src/ha/worker.rs`, `src/ha/mod.rs`, and any new helper modules needed to separate plan application by concern.
- Keep the worker as a thin orchestrator while moving low-level DCS path handling, process request assembly, filesystem mutations, and event payload construction into clearer domain helpers.

---

**Path:** `.ralph/tasks/story-ha-functional-rewrite/05-task-rebuild-ha-tests-around-invariants-and-continuous-observers.md`

## Task: Rebuild HA tests around invariants and continuous observers <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Rework HA tests so they validate the new functional architecture directly, using immutable builders for pure decision tests and continuous invariant observers for integration/e2e scenarios.

**Scope:**
- Edit HA unit, integration, and e2e tests in `src/ha/`, along with any helper modules needed for invariant observation.
- Replace mutation-heavy test setup where possible with immutable facts/world builders for pure-decision coverage.

---

**Path:** `.ralph/tasks/story-ha-functional-rewrite/06-task-move-and-split-ha-e2e-tests-after-functional-rewrite.md`

## Task: Move and split HA e2e tests after the functional rewrite <status>not_started</status> <passes>false</passes>

<description>
After the HA functional rewrite lands, move and restructure the HA end-to-end tests so they are no longer oversized mixed files living under `src/ha/`.

The agent must explore the rewritten HA tests and current repo structure first, then implement the following fixed product decisions:
- this task happens after the other HA migration tasks in this story
- large HA e2e scenario files should be moved out of `src/ha/` into the appropriate `tests/` structure

---

**Path:** `.ralph/tasks/story-operator-architecture-docs/01-task-restructure-operator-docs-for-flow-depth-and-rationale.md`

## Task: Restructure Operator Docs for Better Flow, Depth, and Decision Rationale <status>done</status> <passes>true</passes>

<description>
**Goal:** Rebuild the mdBook documentation into an operator-first guide that explains not only what the system does, but why it behaves that way and which tradeoffs drive key HA decisions.

**Scope:**
- Replace fragmented short chapters with a clearer, deeper structure:
  - merge pages that are too thin to stand alone

---

**Path:** `.ralph/tasks/story-operator-architecture-docs/02-task-post-rewrite-skeptical-claim-verification-with-spark.md`

## Task: Post-Rewrite Skeptical Claim Verification with 15+ Parallel Spark Subagents <status>done</status> <passes>true</passes>

<description>
**Goal:** After the operator-doc transformation is complete, run a deep, adversarial verification of every claim in the docs using many independent `spark` subagents, and resolve all mismatches before finalizing docs.

**Scope:**
- This task MUST start only after Task 01 is complete and docs structure/content are stabilized.
- Build a comprehensive claim inventory from the rewritten docs across the new structure:

---

**Path:** `.ralph/tasks/story-operator-architecture-docs/03-task-expand-contributor-docs-into-full-implementation-deep-dive.md`

## Task: Expand Contributor Docs into a Full Implementation Deep Dive <status>done</status> <passes>true</passes>

<description>
**Goal:** Rewrite the Contributors section into an in-depth engineering deep dive that explains how the code actually works, how modules connect, and how behavior flows through runtime paths, while keeping prose natural, readable, and technically precise.

**Scope:**
- Expand contributor chapters one by one, turning thin summaries into full implementation narratives with explicit call paths, ownership boundaries, and state transitions.
- Require section-by-section depth for architecture internals:

---

**Path:** `.ralph/tasks/story-operator-architecture-docs/04-task-expand-non-contributor-docs-with-deep-subsubchapters.md`

## Task: Expand Non-Contributor Docs with Deep Subsubchapters While Keeping Strong Overviews <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Vastly deepen the non-contributor documentation by adding long-form, detail-rich subsubchapters and flowing explanations, while preserving the existing high-level overview quality at chapter entry points.

**Scope:**
- Keep overview pages and high-level framing concise and strong; do not flatten everything into dense walls of text.
- Add substantial depth below those overviews:

---

**Path:** `.ralph/tasks/story-project-wide-code-hygiene/01-task-audit-and-replace-magic-numbers-project-wide.md`

## Task: Audit and replace magic numbers project-wide <status>not_started</status> <passes>false</passes> <priority>low</priority>

<description>
Audit the project for unexplained magic numbers and replace them with explicit typed constants, configuration, or otherwise well-justified named values.

The agent must explore the whole codebase first, not only HA, then implement the following fixed product decisions:
- this is a project-wide cleanup, not only an `src/ha/state.rs` cleanup
- unexplained magic numbers should be checked everywhere in runtime code, tests, harness code, and supporting modules

---

**Path:** `.ralph/tasks/story-remove-backup-feature/01-task-remove-backup-config-and-process-surface.md`

## Task: Remove backup config and pgBackRest process vocabulary while keeping basebackup replica cloning <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Delete the backup feature's config and process-language surface completely, while preserving `pg_basebackup`-based replica creation as a non-backup bootstrap path.

**Scope:**
- Remove all runtime config schema/default/parser/default exports for `backup.*`, `process.backup_timeout_ms`, and `process.binaries.pgbackrest`.
- Delete the pgBackRest provider/rendering/job-builder modules and all process job/spec/state variants tied to pgBackRest operations.

---

**Path:** `.ralph/tasks/story-remove-backup-feature/02-task-remove-runtime-restore-bootstrap-and-archive-helper-wiring.md`

## Task: Remove runtime restore bootstrap and the archive_command helper/proxy wiring <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Delete the runtime-owned restore bootstrap path and the hacky archive/restore helper stack, including the local event-ingest API used only for archive_command/restore_command passthrough logging.

**Scope:**
- Remove startup-time restore bootstrap selection and execution from the runtime.
- Remove managed Postgres ownership of `archive_mode`, `archive_command`, `restore_command`, helper JSON files, recovery takeover files, and self-executable helper lookup.

---

**Path:** `.ralph/tasks/story-remove-backup-feature/04-task-remove-backup-harness-installers-and-gate-selection.md`

## Task: Remove backup-specific harness, installer, and gate-selection surfaces while preserving real tests for replica cloning <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Delete the backup feature's harness and packaging residue so real-binary verification no longer provisions or expects pgBackRest, while preserving real coverage for normal Postgres and replica-clone behavior.

**Scope:**
- Remove pgBackRest requirements from the harness and provenance policy.
- Remove backup-specific HA test harness config and restore repository setup helpers.

---

**Path:** `.ralph/tasks/story-remove-backup-feature/05-task-remove-backup-docs-and-obsolete-task-artifacts.md`

## Task: Remove backup feature docs and delete obsolete pgBackRest task artifacts <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Remove all operator/interface/contributor documentation for the backup feature and clean the Ralph task inventory so it no longer contains implementation tasks for a feature we are deliberately deleting.

**Scope:**
- Delete or rewrite docs that describe backup config, restore bootstrap, restore takeover, WAL passthrough observability, and pgBackRest installation.
- Remove the obsolete pgBackRest story task files after the new removal story exists.

---

**Path:** `.ralph/tasks/story-rust-system-harness/01-task-core-types-time-errors-watch-channel.md`

## Task: Implement core ids time errors and typed watch channels <status>done</status> <passes>true</passes> <passing>true</passing> <priority>ultra_high</priority>

<description>
**Goal:** Build the foundational shared types and state-channel primitives used by every worker.

**Scope:**
- Create `src/state/ids.rs`, `src/state/time.rs`, `src/state/errors.rs`, `src/state/watch_state.rs`, and `src/state/mod.rs`.
- Implement `MemberId`, `ClusterName`, `SwitchoverRequestId`, `JobId`, `WalLsn`, `TimelineId`, `UnixMillis`, `Version`, `WorkerStatus`, and `Versioned<T>`.

---

**Path:** `.ralph/tasks/story-rust-system-harness/02-task-runtime-config-schema-defaults-parse-validate.md`

## Task: Implement runtime config schema defaults parser and validation <status>done</status> <passes>true</passes> <passing>true</passing> <priority>ultra_high</priority>

<blocked_by>01-task-core-types-time-errors-watch-channel</blocked_by>
<superseded_by>story-pgbackrest-managed-backup-recovery/03-task-high-prio-remove-shell-archive-wrapper-and-current-wiring</superseded_by>
<superseded_by>story-pgbackrest-managed-backup-recovery/04-task-rust-generic-argv-passthrough-binary-for-postgres-archive-restore-logging</superseded_by>

<description>
**Superseded note:** Follow-up review identified the shell archive wrapper path as unacceptable; user feedback was explicit: "this is horrific". The wrapper-based approach introduced brittle shell/script behavior into critical backup paths. New work is split into task 03 in `story-pgbackrest-managed-backup-recovery` (remove it entirely) and task 04 in `story-pgbackrest-managed-backup-recovery` (reintroduce as Rust-native generic passthrough with strict argv-only execution and no PATH-based resolution).

---

**Path:** `.ralph/tasks/story-rust-system-harness/03-task-worker-state-models-and-context-contracts.md`

## Task: Define worker state models and run step_once contracts <status>done</status> <passes>true</passes> <passing>true</passing> <priority>ultra_high</priority>

<blocked_by>02-task-runtime-config-schema-defaults-parse-validate</blocked_by>

<description>
**Goal:** Create all worker state enums/context structs and expose only minimal cross-module contracts.

**Scope:**

---

**Path:** `.ralph/tasks/story-rust-system-harness/04-task-pginfo-worker-single-query-and-real-pg-tests.md`

## Task: Implement pginfo worker single-query polling and real PG tests <status>done</status> <passes>true</passes> <passing>true</passing> <priority>high</priority>

<blocked_by>03-task-worker-state-models-and-context-contracts</blocked_by>

<description>
**Goal:** Implement pginfo state derivation with one SQL poll query and verify behavior against real PG16.

**Scope:**

---

**Path:** `.ralph/tasks/story-rust-system-harness/05a-task-enforce-strict-rust-lints-no-unwrap-expect-panic.md`

## Task: Enforce strict Rust lint policy and forbid unwrap expect panic in runtime code <status>done</status> <passes>true</passes> <passing>true</passing> <priority>ultra_high</priority>

<description>
**Goal:** Install and enforce strict Rust linting with explicit denial of `unwrap`, `expect`, and panic-prone patterns in runtime code.

**Scope:**
- Add repository-level clippy lint configuration for strict Rust style and correctness (`clippy.toml` and/or crate-level `#![deny(...)]` as appropriate).
- Update lint entrypoints (`Makefile` and any lint scripts/config) so CI/local lint always enforces the same deny set.

---

**Path:** `.ralph/tasks/story-rust-system-harness/05b-task-deep-review-codebase-and-verify-done-work.md`

## Task: Deep review codebase quality and verify done tasks are truly complete <status>done</status> <passes>true</passes> <passing>true</passing> <priority>ultra_high</priority>

<description>
**Goal:** Perform a deep end-to-end review of current repository quality, test reality, and completion truthfulness of all tasks already marked done.

**Scope:**
- Enforce preflight model-profile gate through `.ralph/model.txt` before any review work.
- Deeply review runtime and test code for quality issues, untested behavior, and code smells chosen by reviewer judgment (no fixed smell checklist).

---

**Path:** `.ralph/tasks/story-rust-system-harness/05c-task-zero-panic-unwrap-expect-across-runtime-and-tests.md`

## Task: Enforce zero panic/unwrap/expect across runtime and tests with proper Result handling <status>done</status> <passes>true</passes> <passing>true</passing> <priority>high</priority>

<description>
**Goal:** Remove all manual panic/unwrap/expect usage from runtime and test code, replace with proper Rust error handling, and make lint enforcement fail on any regression.

**Scope:**
- Enforce strict clippy policy for both runtime and test targets (no test exceptions).
- Refactor every current `panic!`, `expect`, and `expect_err` case in `src/` and `tests/` to idiomatic alternatives.

---

**Path:** `.ralph/tasks/story-rust-system-harness/05-task-dcs-worker-trust-cache-watch-member-publish.md`

## Task: Implement DCS worker trust evaluation cache updates and member publishing <status>done</status> <passes>true</passes> <priority>high</priority>

<blocked_by>03-task-worker-state-models-and-context-contracts</blocked_by>
<passing>true</passing>

<description>
**Goal:** Implement DCS ownership rules: trust evaluation, typed key parsing, cache updates, and local member publishing.

---

**Path:** `.ralph/tasks/story-rust-system-harness/06-task-process-worker-single-active-job-real-job-exec.md`

## Task: Implement process worker single-active-job execution with real job tests <status>done</status> <passes>true</passes> <passing>true</passing> <priority>high</priority>

<blocked_by>03-task-worker-state-models-and-context-contracts</blocked_by>

<description>
**Goal:** Implement process worker to run exactly one long-running job at a time and publish deterministic outcomes.

**Scope:**

---

**Path:** `.ralph/tasks/story-rust-system-harness/07-task-ha-decide-pure-matrix-idempotency-tests.md`

## Task: Implement pure HA decide engine with exhaustive transition tests <status>done</status> <passes>true</passes> <passing>true</passing> <priority>ultra_high</priority>

<blocked_by>03-task-worker-state-models-and-context-contracts</blocked_by>

<description>
**Goal:** Implement deterministic HA decision logic as a pure function with exhaustive matrix coverage.

**Scope:**

---

**Path:** `.ralph/tasks/story-rust-system-harness/08-task-ha-worker-select-loop-and-action-dispatch.md`

## Task: Implement HA worker select loop and action dispatch wiring <status>done</status> <passes>true</passes> <passing>true</passing> <priority>high</priority>

<blocked_by>04-task-pginfo-worker-single-query-and-real-pg-tests,05-task-dcs-worker-trust-cache-watch-member-publish,06-task-process-worker-single-active-job-real-job-exec,07-task-ha-decide-pure-matrix-idempotency-tests</blocked_by>

<description>
**Goal:** Wire the HA runtime loop that reacts to typed watcher changes and periodic ticks, then dispatches actions.

**Scope:**

---

**Path:** `.ralph/tasks/story-rust-system-harness/09-task-api-debug-workers-and-snapshot-contracts.md`

## Task: Implement API and Debug API workers with typed contracts <status>done</status> <passes>true</passes> <priority>high</priority>

<blocked_by>05-task-dcs-worker-trust-cache-watch-member-publish,08-task-ha-worker-select-loop-and-action-dispatch</blocked_by>
<passing>true</passing>

<description>
**Goal:** Implement typed API endpoints and debug snapshot visibility without bypassing system ownership rules.

---

**Path:** `.ralph/tasks/story-rust-system-harness/10a-task-enforce-real-binary-tests-and-ci-prereqs.md`

## Task: Enforce real-binary test execution (PG16 + etcd3) via explicit gate + CI prerequisites <status>done</status> <passes>true</passes> <passing>true</passing> <priority>high</priority>

<description>
**Goal:** Ensure “real-system” tests actually exercise real PostgreSQL 16 and etcd3 binaries in at least one deterministic gate (CI and/or developer opt-in), instead of silently passing via early-return skips.

**Scope:**
- Add an explicit enforcement mode (env var and/or `make` target) that:
  - fails fast when required binaries are missing, and

---

**Path:** `.ralph/tasks/story-rust-system-harness/10b-task-dcs-real-etcd3-store-adapter-and-tests.md`

## Task: Implement real etcd3-backed DCS store adapter and integration tests <status>done</status> <passes>true</passes> <passing>true</passing> <priority>high</priority>

<description>
**Goal:** Add a production-grade `DcsStore` implementation backed by a real etcd3 instance, and prove it via integration tests using the existing test harness spawner.

**Scope:**
- Implement an etcd3-backed adapter that satisfies the existing `src/dcs/store.rs` `DcsStore` trait (or evolve the trait minimally if required).
- Add integration tests that spawn a real etcd3 process (via `src/test_harness/etcd3.rs`) and verify:

---

**Path:** `.ralph/tasks/story-rust-system-harness/10-task-test-harness-namespace-ports-pg-etcd-spawners.md`

## Task: Build parallel-safe real-system test harness for PG16 and etcd3 <status>done</status> <passes>true</passes> <passing>true</passing> <priority>ultra_high</priority>

<blocked_by>02-task-runtime-config-schema-defaults-parse-validate,03-task-worker-state-models-and-context-contracts</blocked_by>

<description>
**Goal:** Provide deterministic, parallel-safe infrastructure for real integration and e2e tests.

**Scope:**

---

**Path:** `.ralph/tasks/story-rust-system-harness/11-task-typed-pg-config-and-conninfo-roundtrip-tests.md`

## Task: Implement typed postgres config and conninfo parser renderer <status>done</status> <passes>true</passes> <passing>true</passing> <priority>high</priority>

<blocked_by>02-task-runtime-config-schema-defaults-parse-validate</blocked_by>

<description>
**Goal:** Replace raw decisive postgres strings with typed config and strict conninfo parsing.

**Scope:**

---

**Path:** `.ralph/tasks/story-rust-system-harness/12-task-ha-loop-integration-tests-real-watchers-and-step-once.md`

## Task: Build HA loop integration tests with real watchers and deterministic stepping <status>done</status> <passes>true</passes> <passing>true</passing> <priority>ultra_high</priority>

<blocked_by>08-task-ha-worker-select-loop-and-action-dispatch,10-task-test-harness-namespace-ports-pg-etcd-spawners</blocked_by>

<description>
**Goal:** Verify HA loop correctness when worker states interact together through typed channels.

**Scope:**

---

**Path:** `.ralph/tasks/story-rust-system-harness/13-task-e2e-multi-node-real-ha-loops-scenario-matrix.md`

## Task: Implement e2e multi-node real HA-loop scenario matrix <status>done</status> <passes>true</passes> <passing>true</passing> <priority>ultra_high</priority>

<blocked_by>09-task-api-debug-workers-and-snapshot-contracts,10-task-test-harness-namespace-ports-pg-etcd-spawners,12-task-ha-loop-integration-tests-real-watchers-and-step-once</blocked_by>

<description>
**Goal:** Validate real-system HA behavior with all nodes running their own HA loops concurrently.

**Scope:**

---

**Path:** `.ralph/tasks/story-rust-system-harness/14-task-security-auth-tls-real-cluster-tests.md`

## Task: Implement security auth TLS validation tests in real cluster runs <status>done</status> <passes>true</passes> <passing>true</passing> <priority>high</priority>

<blocked_by>10-task-test-harness-namespace-ports-pg-etcd-spawners,13-task-e2e-multi-node-real-ha-loops-scenario-matrix</blocked_by>

<description>
**Goal:** Verify auth and TLS behavior under real deployment conditions.

**Scope:**

---

**Path:** `.ralph/tasks/story-rust-system-harness/15-task-final-double-check-and-stop-gate.md`

## Task: Final double-check gate for real testing completeness <status>done</status> <passes>true</passes> <passing>true</passing> <priority>ultra_high</priority>

<blocked_by>13-task-e2e-multi-node-real-ha-loops-scenario-matrix,14-task-security-auth-tls-real-cluster-tests</blocked_by>

<description>
**Goal:** Perform final independent verification that all components are truly tested, all required features exist and work, and all suites pass with no exceptions before final completion tasks.

**Scope:**

---

**Path:** `.ralph/tasks/story-rust-system-harness/16-task-debug-ui-verbose-state-actions-events-and-final-stop.md`

## Task: Setup verbose debug UI and final STOP gate <status>done</status> <passes>true</passes> <passing>true</passing> <priority>low</priority>

<blocked_by>15-task-final-double-check-and-stop-gate</blocked_by>

<description>
**Goal:** Build a debug UI system that reacts to fine-grained state/action/event changes via a super-verbose debug API endpoint and render those details in a rich static HTML UI; this task runs last.

**Scope:**

---

**Path:** `.ralph/tasks/story-rust-system-harness/18-task-recurring-meta-deep-skeptical-codebase-review.md`

## Task: Recurring meta-task for deep skeptical codebase quality verification <status>not_started</status> <passes>meta-task</passes> <passing>true</passing> <priority>very_low</priority>
NEVER TICK OFF THIS TASK. ALWAYS KEEP <passes>meta-task</passes>. This is a recurring deep verification task.

<description>
This is a **RECURRING META-TASK**.

Every time this task is picked up, the engineer must run a **FRESH verification** from scratch:
- Before starting the verification body, delete prior fresh-run artifacts for this meta-task to eliminate carry-over bias.

---

**Path:** `.ralph/tasks/story-rust-system-harness/22-task-ha-admin-api-read-write-surface.md`

## Task: Expose full HA admin API read and write surface <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
**Goal:** Add a first-class HA admin API that exposes operational read endpoints and write actions needed to control cluster behavior without touching DCS directly.

**Scope:**
- Extend `src/api/controller.rs` with typed request/response handlers for HA admin actions beyond switchover.
- Extend `src/api/worker.rs` routing/auth to expose a complete admin/read API surface.

---

**Path:** `.ralph/tasks/story-rust-system-harness/23-task-ha-admin-cli-over-api.md`

## Task: Build a simple Rust HA admin CLI over the exposed API <status>done</status> <passes>true</passes> <passing>true</passing>

<blocked_by>22-task-ha-admin-api-read-write-surface</blocked_by>

<description>
**Goal:** Provide a simple, production-usable Rust CLI that invokes the HA admin API for both read and write operations.

**Scope:**

---

**Path:** `.ralph/tasks/story-rust-system-harness/24-task-real-e2e-harness-3nodes-3etcd.md`

## Task: Upgrade real e2e harness to 3 pgtuskmaster nodes and 3 etcd members <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
**Goal:** Make the real e2e environment represent a true 3-node HA control plane with a 3-member etcd cluster instead of a single etcd instance.

**Scope:**
- Extend harness support for multi-member etcd cluster bootstrap and lifecycle management.
- Update e2e fixture setup to always launch 3 pgtuskmaster nodes wired to 3 etcd members.

---

**Path:** `.ralph/tasks/story-rust-system-harness/25-task-enforce-e2e-api-only-control-no-direct-dcs.md`

## Task: Enforce API-only control in e2e and ban direct DCS mutations <status>done</status> <passes>true</passes> <passing>true</passing>

<blocked_by>22-task-ha-admin-api-read-write-surface</blocked_by>
<blocked_by>24-task-real-e2e-harness-3nodes-3etcd</blocked_by>

<description>
**Goal:** Ensure full e2e tests never write/delete DCS keys directly and only control/read HA behavior through exposed API endpoints.

---

**Path:** `.ralph/tasks/story-rust-system-harness/26-task-e2e-unassisted-failover-sql-consistency.md`

## Task: Add unassisted failover e2e with before/after SQL consistency proof <status>done</status> <passes>true</passes> <passing>true</passing>

<blocked_by>24-task-real-e2e-harness-3nodes-3etcd</blocked_by>
<blocked_by>25-task-enforce-e2e-api-only-control-no-direct-dcs</blocked_by>

<description>
**Goal:** Create a skeptical e2e test proving full failover completes after killing one postgres instance, with no further interventions beyond API reads and SQL validation.

---

**Path:** `.ralph/tasks/story-rust-system-harness/27-task-e2e-ha-stress-workloads-during-role-changes.md`

## Task: Add HA stress e2e suites with concurrent SQL workloads during role changes <status>done</status> <passes>true</passes> <passing>true</passing>

<blocked_by>24-task-real-e2e-harness-3nodes-3etcd</blocked_by>
<blocked_by>25-task-enforce-e2e-api-only-control-no-direct-dcs</blocked_by>

<description>
**Goal:** Build stress-oriented e2e tests that continuously read/write/query SQL while HA switchover and failover paths execute, and verify safe demotion/promotion/fencing behavior.

---

**Path:** `.ralph/tasks/story-rust-system-harness/28-task-e2e-network-partition-chaos-no-split-brain.md`

## Task: Add network partition e2e chaos tests with proxy fault injection <status>completed</status> <passes>true</passes> <passing>true</passing>

<blocked_by>24-task-real-e2e-harness-3nodes-3etcd</blocked_by>

<description>
**Goal:** Validate split-brain safety and recovery under true network partition conditions using a controllable proxy layer for etcd, postgres, and API traffic.

**Scope:**

---

**Path:** `.ralph/tasks/story-rust-system-harness/29-task-e2e-tls-adversarial-cert-validation.md`

## Task: Expand TLS adversarial e2e tests for certificate validation hardening <status>done</status> <passes>true</passes> <passing>true</passing>

<blocked_by>22-task-ha-admin-api-read-write-surface</blocked_by>

<description>
**Goal:** Add skeptical TLS tests that actively try to break API and cluster TLS trust, including wrong certs and expired certs, and prove they are rejected.

**Scope:**

---

**Path:** `.ralph/tasks/story-rust-system-harness/30-task-full-e2e-blackbox-api-cli-orchestration.md`

## Task: Migrate full e2e suites to black-box API and CLI orchestration <status>completed</status> <passes>true</passes> <passing>true</passing>

<blocked_by>22-task-ha-admin-api-read-write-surface</blocked_by>
<blocked_by>23-task-ha-admin-cli-over-api</blocked_by>
<blocked_by>25-task-enforce-e2e-api-only-control-no-direct-dcs</blocked_by>

<description>
**Goal:** Convert full-system e2e tests into black-box tests that interact through public API/CLI surfaces rather than internal worker channels or binary-specific control paths.

---

**Path:** `.ralph/tasks/story-rust-system-harness/31-task-docs-framework-selection-install-and-artifact-hygiene.md`

## Task: Install mdBook docs framework and enforce artifact git hygiene <status>completed</status> <passes>true</passes> <passing>true</passing>

<description>
**Goal:** Use mdBook for this Rust project, install it, prove it renders a static HTML site correctly, and lock down strict git artifact hygiene before any docs commits.

**Scope:**
- No framework research or comparison is required for this task.
- The framework choice is fixed: mdBook must be used.

---

**Path:** `.ralph/tasks/story-rust-system-harness/32-task-author-complete-architecture-docs-with-diagrams-and-no-code.md`

## Task: Author full architecture documentation with rich diagrams and zero code-level narration <status>completed</status> <passes>true</passes> <passing>true</passing>

<blocked_by>31-task-docs-framework-selection-install-and-artifact-hygiene</blocked_by>

<description>
**Goal:** Create complete, human-flowing architecture documentation for the full system using the chosen framework, with diagram-first explanations and no implementation-level code discussion.

**Scope:**

---

**Path:** `.ralph/tasks/story-rust-system-harness/33-task-deep-skeptical-verification-of-doc-facts-and-writing-quality.md`

## Task: Perform deep skeptical verification of all docs facts and writing quality <status>completed</status> <passes>true</passes> <passing>true</passing>

<blocked_by>32-task-author-complete-architecture-docs-with-diagrams-and-no-code</blocked_by>

<description>
**Goal:** Rigorously validate every documentation claim against the real codebase and enforce a hard editorial quality gate that rejects overloaded, vague, or misleading writing.

**Scope:**

---

**Path:** `.ralph/tasks/story-rust-system-harness/34-task-add-non-test-unified-node-entrypoint-autobootstrap-and-ha-loop.md`

## Task: Add non-test unified node entrypoint from start through autonomous HA loop <status>done</status> <passes>true</passes> <passing>true</passing> <priority>high</priority>

<description>
**Goal:** Provide one production (non-test) entry path that starts a `pgtuskmaster` node from config only and runs it through bootstrap and HA loop without manual orchestration.

**Scope:**
- Add/extend runtime entry code in non-test modules so node startup is performed through a single canonical entrypoint.
- Ensure startup path decides bootstrap mode from existing state:

---

**Path:** `.ralph/tasks/story-rust-system-harness/35-task-migrate-all-node-startup-tests-to-unified-entrypoint-config-only.md`

## Task: Migrate all node-starting tests to unified entrypoint (config-only) <status>done</status> <passes>true</passes> <priority>high</priority>
<passing>true</passing>

<blocked_by>34-task-add-non-test-unified-node-entrypoint-autobootstrap-and-ha-loop</blocked_by>

<description>
**Goal:** Ensure every test that starts a `pgtuskmaster` node uses the same new production entrypoint and only provides configuration.

---

**Path:** `.ralph/tasks/story-rust-system-harness/36-task-enforce-post-startup-hands-off-test-policy-no-direct-coordination.md`

## Task: Enforce post-startup hands-off test policy (no direct coordination) <status>completed</status> <passes>true</passes> <passing>true</passing> <priority>high</priority>

<blocked_by>35-task-migrate-all-node-startup-tests-to-unified-entrypoint-config-only</blocked_by>

<description>
**Goal:** After cluster/node startup, tests must not perform direct internal coordination or DCS steering; they may only observe/listen plus allowed external actions.

**Scope:**

---

**Path:** `.ralph/tasks/story-rust-system-harness/37-task-unified-e2e-harness-testconfig-interface.md`

## Task: Unify HA E2E Harness Behind Stable `TestConfig` Interface <status>completed</status> <passes>true</passes> <passing>true</passing>

<description>
**Goal:** Design and implement one stable, shared HA e2e harness interface driven by a single `TestConfig` input that initializes the requested cluster topology + pre-test setup, returns a full test handle, and removes duplicated setup/wait/process glue from scenario files.

**Scope:**
- Replace duplicated orchestration in:
  - `src/ha/e2e_multi_node.rs`

---

**Path:** `.ralph/tasks/story-rust-system-harness/38-task-unified-structured-logging-and-postgres-binary-ingestion.md`

## Task: Build Unified Structured Logging Pipeline With Postgres/Binary Ingestion <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
**Goal:** Implement one unified, config-driven logging system that emits structured JSONL to `stderr` by default, ingests/normalizes all postgres and helper-binary logs into the same stream, and guarantees no log loss on parse failures.

**Scope:**
- Add a single logging subsystem/config entrypoint for the entire runtime (no split setup points).
- Enforce baseline structured fields on every emitted record:

---

**Path:** `.ralph/tasks/story-rust-system-harness/39-task-file-sink-support-for-structured-logging.md`

## Task: Add File Sink Support For Unified Structured Logging <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
**Goal:** Extend the unified structured logging subsystem to support configurable JSONL file sinks (in addition to the current stderr JSONL sink).

**Prerequisite:** `.ralph/tasks/story-rust-system-harness/38-task-unified-structured-logging-and-postgres-binary-ingestion.md` (unified log schema + ingestion + `LogSink` trait already exist).

**Scope:**

---

**Path:** `.ralph/tasks/story-rust-system-harness/39-task-logging-file-sink-backlog.md`

## Task: Add Structured File Sink Support (Backlog) <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
**Goal:** Extend the unified logging subsystem with optional structured file sink support after the base structured-ingestion task is complete.

**Scope:**
- Implement additional sink modes in the single existing logging config/setup path:
  - structured file output sink(s)

---

**Path:** `.ralph/tasks/story-rust-system-harness/40-task-ultra-high-prio-test-target-split-and-reference-migration.md`

## Task: Ultra-high-priority migrate repo gates to `make test` + `make test-long` only <status>done</status> <passes>true</passes> <passing>true</passing> <priority>ultra-high</priority>

<description>
**Goal:** Complete and verify the global migration from legacy test targets to only two test groups: `make test` (regular) and `make test-long` (ultra-long only).

**Scope:**
- Enforce Makefile target surface to only `test` and `test-long` (remove all legacy extra test targets).
- Keep `make test` as the default frequently-run suite and ensure it excludes only tests with evidence-backed runtime >= 3 minutes.

---

**Path:** `.ralph/tasks/story-rust-system-harness/41-task-ultra-high-prio-split-ultra-long-e2e-into-short-parallel-tests.md`

## Task: Ultra-high-priority split ultra-long e2e tests into shorter parallel real-binary tests <status>completed</status> <passes>true</passes> <passing>true</passing> <priority>ultra-high</priority>

<description>
**Goal:** Replace the current ultra-long HA e2e stress scenario(s) with multiple shorter real-binary e2e tests that preserve full coverage and must run in parallel.

**Scope:**
- Decompose each current ultra-long scenario (runtime >= 3 minutes from evidence) into smaller independent real-binary e2e tests with narrow objectives.
- Preserve all existing behavioral coverage and assertions from the original long scenarios.

---

**Path:** `.ralph/tasks/story-rust-system-harness/42-task-operator-grade-action-logging-and-no-silent-errors.md`

## Task: Enforce Operator-Grade Action Logging And No Silent Error Swallowing <status>done</status> <passes>true</passes>

<description>
**Goal:** Make runtime/operator observability explicit and uniform: debug-log all actions and all meaningful runtime flow steps across the codebase so operators can reconstruct exactly what code path executed, in order; info-log important operator lifecycle/default events; warn-log ignorable errors; error-log hard errors; and eliminate silent error swallowing.

**Scope:**
- Keep using the existing unified logging infra (`src/logging/*`, `LogHandle`, `LogRecord`, existing producers/parsers/sinks).
- Do not introduce alternate logging frameworks, side channels, or ad-hoc `println!/eprintln!` for runtime internals.

---

**Path:** `.ralph/tasks/story-rust-system-harness/task-real-ha-dcs-process-integration-tests.md`

## Task: Add real HA+DCS+Process integration tests <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
**Goal:** Build integration tests that wire real PG16 binaries, a real etcd-backed DCS store, the process worker, pginfo worker, and HA worker so failures cannot pass silently.

**Scope:**
- Use the existing test harness spawners in `src/test_harness/pg16.rs`, `src/test_harness/etcd3.rs`, `src/test_harness/namespace.rs`, and `src/test_harness/ports.rs`.
- Add integration tests under `tests/` or a dedicated `src/ha/worker` test module that:

---

**Path:** `.ralph/tasks/story-rust-system-harness/task-typed-dcs-writes-and-encapsulation.md`

## Task: Replace Stringly DCS Writes With Typed Writer API <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
**Goal:** Eliminate raw path/string DCS writes from HA by introducing a typed DCS writer API and restricting access to low-level write/delete operations.

**Scope:**
- `src/dcs/store.rs`: introduce typed writer helpers (e.g., leader lease write/delete) and hide raw write/delete from non-DCS modules where possible.
- `src/dcs/state.rs`: update contexts to carry the new typed writer interface (or wrapper) as needed.

---

**Path:** `.ralph/tasks/story-secure-explicit-node-config/01-task-expand-runtime-config-schema-for-explicit-secure-node-startup.md`

## Task: Expand runtime config schema for explicit secure node startup <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
**Goal:** Redesign the runtime config model so every required secure startup setting is explicitly represented (TLS, HTTP, PostgreSQL hosting, roles/auth, pg_hba/pg_ident, and DCS init config).

**Scope:**
- Expand `src/config/mod.rs` and `src/config/schema.rs` with strongly typed fields for:
- PostgreSQL TLS server identity and client auth material.

---

**Path:** `.ralph/tasks/story-secure-explicit-node-config/02-task-migrate-parser-defaults-and-validation-to-explicit-enum-driven-config.md`

## Task: Migrate parser/defaults/validation to explicit enum-driven config semantics <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
**Goal:** Remove hidden config inference by moving defaulting/validation behavior to explicit enum-driven semantics while preserving safe startup requirements.

**Scope:**
- Refactor `src/config/parser.rs` and `src/config/defaults.rs` to stop injecting implicit runtime identities (for example `postgres` user fallback).
- Introduce explicit default policy only where permitted by typed enums and safe documented defaults.

---

**Path:** `.ralph/tasks/story-secure-explicit-node-config/03-task-enforce-role-specific-credential-usage-across-runtime.md`

## Task: Enforce role-specific credential usage across runtime operations <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
**Goal:** Ensure each runtime function uses only its designated role (`superuser`, `replicator`, `rewinder`) and corresponding auth mode from config.

**Scope:**
- Trace and update credential usage in HA/process/pginfo/rewind/postgres control paths.
- Replace shared or hardcoded connection identities with explicit role-based selection.

---

**Path:** `.ralph/tasks/story-secure-explicit-node-config/04-task-wire-http-pg-tls-pg_hba-pg_ident-and-dcs-init-into-startup.md`

## Task: Wire HTTP/PG TLS, pg_hba/pg_ident, and DCS init config into startup orchestration <status>done</status> <passes>true</passes>

<description>
**Goal:** Make startup consume the expanded config end-to-end so node boot requires explicit secure config and does not infer missing values.

**Scope:**
- Update runtime/process/startup orchestration to consume new HTTP and PostgreSQL TLS cert/key settings.
- Ensure pg_hba and pg_ident config fields are materialized correctly during bootstrap/start.

---

**Path:** `.ralph/tasks/story-secure-explicit-node-config/05-task-migrate-fixtures-examples-and-cli-config-surfaces-to-new-schema.md`

## Task: Migrate fixtures/examples/CLI config surfaces to the secure explicit schema <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
**Goal:** Align all config producers/consumers (tests, examples, CLI entrypoints) with the expanded schema and explicit secure requirements.

**Scope:**
- Update test fixtures under `src/` and `tests/` to provide full explicit config values.
- Update `examples/` and any contract fixture builders to compile with new config fields.

---

**Path:** `.ralph/tasks/story-secure-explicit-node-config/06-task-full-verification-for-secure-explicit-config-refactor.md`

## Task: Run full verification for secure explicit config refactor <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
**Goal:** Execute full validation gates after the config refactor and convert any failures into actionable bug tasks.

**Scope:**
- Run full required project gates after merging upstream tasks.
- Record evidence logs and failure signatures.

---

**Path:** `.ralph/tasks/story-tracing-based-logging/01-task-replace-bespoke-app-logging-with-tracing-based-structured-logging.md`

## Task: Replace bespoke app logging with tracing-based structured logging <status>not_started</status> <passes>false</passes>

<description>
Replace the custom app-side logging stack with a more standard Rust structured logging architecture based on the `tracing` ecosystem, while preserving structured output and operator usefulness.

The agent must explore the current logging implementation, runtime integration points, and tests first, then implement the following fixed product decisions:
- use the standard Rust `tracing` ecosystem for app-side structured logging
- remove the bespoke logging framework pieces that reimplement standard `tracing` / `tracing-subscriber` behavior, including the custom sink abstraction and custom sink fanout/file/stderr framework

---

**Path:** `.ralph/tasks/story-tracing-based-logging/02-task-add-otel-log-export-alongside-default-stderr-jsonl.md`

## Task: Add OTEL log export alongside default stderr JSONL output <status>not_started</status> <passes>false</passes>

<description>
Add OpenTelemetry log export support alongside the default stderr JSONL output, without forcing trace/span semantics onto the product.

The agent must explore the current logging pipeline, available Rust ecosystem support, and the runtime configuration surface first, then implement the following fixed product decisions:
- default local behavior remains structured JSONL to stderr
- optional structured file output remains available through the same tracing-based pipeline

