---
## Task: Migrate all node-starting tests to unified entrypoint (config-only) <status>done</status> <passes>true</passes> <priority>high</priority>
<passing>true</passing>

<blocked_by>34-task-add-non-test-unified-node-entrypoint-autobootstrap-and-ha-loop</blocked_by>

<description>
**Goal:** Ensure every test that starts a `pgtuskmaster` node uses the same new production entrypoint and only provides configuration.

**Scope:**
- Inventory all tests/harnesses that currently spin nodes directly or through ad-hoc startup wiring.
- Replace those startup paths to call the new non-test unified entry API only.
- Remove or stop using alternative startup wiring in tests that bypasses production entry semantics.
- If required config is missing for test scenarios, extend shared config structs and fixtures so tests remain config-driven.
- Ensure startup semantics do not diverge by test type: all node-starting tests must invoke the same entry path.

**Context from research:**
- Request explicitly calls out inconsistent test startup behavior for "main/entry stuff".
- Desired behavior is production parity: tests and runtime should enter through one canonical startup flow.
- Any missing config needed by this flow is itself a required follow-up change, not a reason to bypass entrypoint.

**Expected outcome:**
- Node startup behavior is unified across test suites and production, reducing scenario skew and startup drift.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] Full exhaustive checklist completed with concrete module requirements: all test harness modules and test files that start nodes are enumerated and migrated to the unified entrypoint
- [x] No test that starts a node uses bespoke direct startup orchestration outside the unified entry API
- [x] Shared config structs/fixtures are updated where required so startup remains config-only
- [x] Add/update regression guard(s) that fail if new node-starting tests bypass the unified entrypoint
- [x] `make check --all-targets` (or stricter equivalent) passes after config surface changes
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] BDD features pass (covered by `make test`).
</acceptance_criteria>

## Detailed Implementation Plan (Draft 2 - Verified)

Parallel research tracks completed before this draft:
1. Startup path inventory across all `tests/`, `src/ha`, `src/process`, and `src/test_harness`.
2. Direct worker orchestration scan (`*_worker::run`, `tokio::spawn`) in tests/e2e.
3. Runtime entrypoint capability scan (`src/runtime/node.rs`, `src/bin/pgtuskmaster.rs`).
4. Real-binary fixture scan for direct `initdb`/`pg_ctl`/`postgres` invocations.
5. Policy/regression guard scan in `tests/policy_e2e_api_only.rs`.
6. Contract fixture scan in `src/worker_contract_tests.rs`.
7. API BDD harness scan in `tests/bdd_api_http.rs`.
8. Shared binary/config dependency scan (`BinaryPaths`, test_harness binaries).
9. Existing scenario timeouts/artifacts and shutdown path scan in HA e2e.
10. Cross-check of current task dependencies and prior task 34 runtime additions.

### Scope Clarification for this task

This task targets **node-starting tests**, meaning tests that spin full pgtuskmaster node behavior (startup + worker loop) and currently bypass the unified runtime entrypoint.

Based on inventory:
- Primary migration target:
  - `src/ha/e2e_multi_node.rs` (`ClusterFixture::start`) currently builds and spawns worker contexts manually and performs direct `initdb`/`pg_ctl` orchestration.
- Secondary guard/fixture updates to keep config-only startup and prevent regressions:
  - `tests/policy_e2e_api_only.rs` (extend policy checks).
  - `src/test_harness/*` support modules used by HA e2e.
  - Potentially `tests/cli_binary.rs` if we add binary-level startup smoke.
- Non-target (for this task) unless coupling forces edits:
  - `src/process/worker.rs` real process-job tests validate process jobs, not unified node entry.
  - `tests/bdd_api_http.rs` and `src/worker_contract_tests.rs` are API/contract step tests and do not start full nodes.

### Migration Objectives

1. Replace bespoke node startup orchestration in HA e2e with unified runtime entrypoint usage.
2. Ensure HA e2e supplies only config (and fixture-level environment details), not manual worker graph wiring.
3. Preserve existing scenario assertions (switchover/failover/no-quorum/stress invariants).
4. Add regression guards so new node-starting tests cannot reintroduce direct startup orchestration.
5. Keep full real-binary validation intact and passing with required gates.

### Deep Skeptical Verification Adjustments

1. API listen address discovery must be deterministic after removing manual `TcpListener` ownership.
- Change plan to allocate fixed per-node API ports in the HA topology and set `runtime_cfg.api.listen_addr` explicitly instead of using `127.0.0.1:0`.
- Keep cluster fixture request helpers using those known addresses, avoiding any test-only runtime introspection hook.

2. Regression policy patterns need stronger startup-bypass coverage.
- Include additional forbidden markers for e2e sources:
  - `DebugApiCtx::contract_stub(`
  - `crate::api::worker::step_once(`
  - `crate::debug_api::worker::step_once(`
  - `crate::dcs::worker::run(`
  - `ha_worker::run(`
- Keep scope restricted to `src/ha/e2e_*.rs` to avoid legitimate non-e2e uses.

3. Execution order should reduce churn risk.
- Add a first implementation slice that converts startup path and node bookkeeping before removing old helper functions, then compile and fix references, then remove dead code in a cleanup pass.

### Planned Changes (File-by-file)

1. `src/ha/e2e_multi_node.rs` (main migration)
- Introduce per-node runtime launch using unified entry API:
  - Use `crate::runtime::run_node_from_config(...)` (or config-path variant if fixture writes TOML files).
- Set explicit per-node API ports in config so tests can target stable addresses without runtime-private hooks.
- Remove bespoke node boot wiring from `ClusterFixture::start`:
  - Manual channel creation for pg/dcs/process/ha/debug/api.
  - Manual construction of `*Ctx::contract_stub`.
  - Manual `tokio::spawn` calls for each worker loop.
- Remove direct test bootstrap operations that duplicate runtime startup decisions:
  - `initialize_pgdata(...)` in fixture startup path.
- Keep controlled failure-injection primitives that are scenario-specific and not startup orchestration:
  - `pg_ctl_stop_immediate` for explicit failover injection remains valid.
- Add runtime-task handle tracking in fixture:
  - Replace vector of per-worker task handles with one runtime-task handle per node.
  - Ensure shutdown abort/join logic handles runtime-task lifecycle cleanly.
- Rework API-driving logic:
  - Keep existing request helper semantics, but decouple from manual `debug_ctx`/`api_ctx` stepping.
  - Use real HTTP request/response with retry deadlines while runtime loop serves API continuously.
- Rework process-state observation:
  - If direct `process_subscriber` is no longer owned by fixture, use debug/API surfaced state for equivalent assertions, or expose a minimal runtime-owned state observer hook only if required.

2. `src/runtime/node.rs` / runtime surface (only if needed for testability without bypass)
- Prefer no new test-only hooks.
- If HA e2e requires runtime lifecycle control, add **production-safe** cancellable runtime task wrapper:
  - Example shape: helper returning `JoinHandle<Result<(), RuntimeError>>` while still invoking same unified startup path.
  - No alternate startup semantics allowed.
- If API bind address discovery is needed (`127.0.0.1:0`), add a runtime-observable API-local-addr signal that is generic and not test-specific (for example state publication or callback channel built from normal runtime flow).

3. `src/test_harness/*` support updates
- Add/update harness helpers to generate per-node runtime configs and optional temp config files.
- Keep helpers config-only; no helper may perform startup sequencing that runtime now owns.

4. `tests/policy_e2e_api_only.rs` (regression guard expansion)
- Extend forbidden-pattern policy for `src/ha/e2e_*.rs` to catch startup bypass indicators such as:
  - `ProcessWorkerCtx::contract_stub(`
  - `HaWorkerCtx::contract_stub(`
  - `ApiWorkerCtx::contract_stub(`
  - `DebugApiCtx::contract_stub(`
  - direct worker-run orchestration snippets (`crate::pginfo::worker::run(` etc.) in e2e sources.
  - `crate::api::worker::step_once(`
  - `crate::debug_api::worker::step_once(`
  - `crate::dcs::worker::run(`
  - `ha_worker::run(`
  - direct bootstrap startup helpers (`initialize_pgdata(`) in e2e sources.
- Keep policy narrowly scoped to e2e source files to avoid false positives in legitimate unit/contract tests.

5. `tests/cli_binary.rs` (optional but likely useful)
- If coverage gap remains for entrypoint parity, add a lightweight startup-surface smoke assertion for `pgtuskmaster` invocation shape that does not attempt full cluster bring-up.
- Keep this as supplemental, not replacement for HA real-binary e2e validation.

### Execution Phases

Phase 1: Refactor HA e2e fixture startup to runtime entrypoint
- Introduce runtime task abstraction per node.
- Replace manual worker graph construction with one unified runtime call.
- Keep etcd cluster + ports + per-node config generation intact.
- Allocate explicit per-node API ports and bind runtime config to those addresses.
- Compile after this slice before dead-code cleanup to constrain breakage.

Phase 2: Adapt HA e2e control/observation paths
- Replace step-driven API servicing with runtime-served API requests + retry/timeout helpers.
- Preserve scenario-level assertions and evidence artifact output.
- Adjust any process/HA observation checks that depended on removed direct subscribers.

Phase 3: Add regression policy checks
- Extend `tests/policy_e2e_api_only.rs` with startup-bypass patterns.
- Ensure policy still passes for non-e2e modules where contract stubs are legitimate.

Phase 4: Run required verification gates
- `make check --all-targets` (explicit per acceptance criteria)
- `make check`
- `make test`
- `make test-long`
- `make lint`
- Record and fix all failures until green.

Phase 5: Task bookkeeping after green
- Tick acceptance checkboxes.
- Set `<passing>true</passing>`.
- Run `/bin/bash .ralph/task_switch.sh`.
- Commit all files including `.ralph` updates.
- Push branch.
- Add AGENTS.md learnings if new.

### Skeptical Risks and Mitigations

1. Risk: Runtime cancellation/cleanup semantics differ from prior per-worker abort model.
- Mitigation: explicit runtime task abort+await in fixture shutdown; preserve postgres/etcd cleanup safety net.

2. Risk: Losing direct `step_once` API/debug control can increase request flake.
- Mitigation: strengthen HTTP retry wrappers with bounded per-request timeout and clearer last-observation errors.

3. Risk: Removal of direct process subscriber access weakens some assertions.
- Mitigation: move those assertions to externally observable API/debug state where possible; only add minimal runtime-observer surface if absolutely required.

4. Risk: Policy guard too broad and breaks non-target tests.
- Mitigation: keep scan restricted to `src/ha/e2e_*.rs` and precise startup-bypass tokens.

5. Risk: Real-binary e2e duration/flakiness increases after orchestration switch.
- Mitigation: retain current timeouts, one-thread test execution guidance where needed, and deterministic artifact logging on failure.

### Completion Checklist for `NOW EXECUTE`

- [x] `src/ha/e2e_multi_node.rs` no longer manually wires worker contexts for node startup.
- [x] HA e2e nodes start through unified runtime entrypoint only.
- [x] Any startup helper that duplicates runtime bootstrap logic is removed from HA e2e path.
- [x] Regression policy test catches startup-bypass patterns in e2e sources.
- [x] Shared config/harness updates compiled under `--all-targets`.
- [x] `make check --all-targets` passes.
- [x] `make check` passes.
- [x] `make test` passes.
- [x] `make test` passes.
- [x] `make lint` passes.

NOW EXECUTE
