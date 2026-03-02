---
## Task: Implement e2e multi-node real HA-loop scenario matrix <status>done</status> <passes>true</passes> <passing>true</passing> <priority>ultra_high</priority>

<blocked_by>09-task-api-debug-workers-and-snapshot-contracts,10-task-test-harness-namespace-ports-pg-etcd-spawners,12-task-ha-loop-integration-tests-real-watchers-and-step-once</blocked_by>

<description>
**Goal:** Validate real-system HA behavior with all nodes running their own HA loops concurrently.

**Scope:**
- Build e2e tests that boot full clusters using real PG16 + etcd3 + running node processes.
- Cover bootstrap, planned switchover, failover, no-quorum fail-safe, rewind rejoin, fencing-before-promotion, and split-brain prevention.
- Ensure test code only injects faults/inputs; HA actions must be executed by the system itself.

**Context from research:**
- This task encodes the user requirement for real scenarios with multiple pgtuskmaster nodes and autonomous loops.

**Expected outcome:**
- E2E suite proves cluster-level HA behavior under real multi-node conditions.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] E2E suite launches at least 3 nodes with all HA loops running concurrently.
- [x] Tests verify leader election, promotion, demotion, fencing, and recovery using observed system outputs only.
- [x] Tests do not call internal HA decision functions to enact transitions.
- [x] Scenario matrix includes all plan-listed HA paths.
- [x] Run e2e suite standalone and collect logs/artifacts.
- [x] Run full suite: `make check`, `make test`, `make lint`, `make test-bdd`.
- [x] For every failing scenario, use `$add-bug` and create bug task(s) with timeline/log evidence.
</acceptance_criteria>

<execution_plan>
## Detailed Implementation Plan

1. Preflight and baseline lock
- [x] Verify blockers `09`, `10`, and `12` are still `done` + `passing true` before edits.
- [x] Capture baseline with `cargo check --all-targets` so later regressions can be attributed to task 13 changes.
- [x] Keep hard constraints active: no `unwrap`/`expect`/`panic` in non-test runtime code; no optional/skipped real-binary tests.

2. Close the real-etcd gap with a sync-trait-compatible adapter
- [x] Implement a real etcd-backed `DcsStore` adapter under `src/dcs/` (new module, e.g. `etcd_store.rs`) instead of relying on in-memory test stores.
- [x] Add dependencies (`etcd-client`, plus minimal stream/helper crates) in `Cargo.toml`.
- [x] Keep the current synchronous `DcsStore` trait contract to avoid broad API churn; bridge async etcd I/O internally via a watch-pump task + thread-safe event buffer.
- [x] Add an explicit constructor that validates endpoint connectivity and starts prefix watch for `/{scope}/`.
- [x] Preserve existing unit-test behavior with in-memory stores implementing the same trait contract.

3. Wire real etcd adapter into worker paths and health semantics
- [x] Keep `dcs::worker::step_once` and `ha::worker::dispatch_actions` call shapes stable while swapping in the real adapter where contexts are built.
- [x] Keep typed error mapping (`DcsStoreError` -> `WorkerError` / dispatch error strings) stable and explicit.
- [x] Use `refresh_from_etcd_watch(...).had_errors` to degrade trust/worker health instead of silently ignoring malformed/unknown updates.
- [x] Add focused tests proving read/write/delete/watch flows against the real adapter plus failure-mode mapping (unreachable endpoint, decode failure, watch decode noise).

4. Add missing planned-switchover semantics before matrix assertions
- [x] Implement/adjust HA handling so observed DCS switchover intent can drive leadership transition through normal action dispatch (no direct state mutation from tests).
- [x] Add focused tests for switchover intent lifecycle (set/read/clear) and resulting phase/action sequence.
- [x] Keep rewind behavior explicit: if source conninfo remains static, document and test that limitation in e2e assertions instead of assuming full Patroni-like donor selection.

5. Add a reusable multi-node real cluster fixture
- [x] Create a dedicated real e2e fixture module in crate tests (prefer `src/ha/` test module to keep crate-private worker context access).
- [x] Fixture responsibilities:
- [x] allocate a unique `NamespaceGuard`,
- [x] spawn one real etcd instance via `src/test_harness/etcd3.rs`,
- [x] allocate per-node ports/directories,
- [x] prepare per-node PG data dirs,
- [x] construct per-node runtime configs with real binary paths from `require_pg16_process_binaries()`,
- [x] instantiate per-node pginfo + dcs + process + ha worker contexts and launch all `run(...)` loops concurrently.
- [x] Keep all state subscribers alive for fixture lifetime to avoid false channel-close failures.
- [x] Run each node as independent worker loops (`pginfo`, `dcs`, `process`, `ha`) concurrently; no direct `decide` or direct process command calls inside scenarios.

6. Add an explicit observability surface for e2e assertions
- [x] For each node, keep subscribers for `PgInfoState`, `DcsState`, `ProcessState`, and `HaState`.
- [x] Add helper probes with bounded polling timeouts:
- [x] wait for leader key ownership in etcd,
- [x] wait for specific HA phase(s),
- [x] wait for process outcomes/job kinds,
- [x] wait for postgres SQL reachability on target node.
- [x] Persist a concise per-scenario timeline artifact (phase transitions, DCS leader changes, process outcomes) under namespace logs for post-failure debugging.

7. Scenario matrix implementation (real loops, no direct decide calls)
- [x] Implement a table-driven e2e suite with at least these scenarios:
- [x] bootstrap/election from cold start,
- [x] planned switchover (API-triggered DCS request, system executes transition),
- [x] failover after primary stop/crash injection,
- [x] no-quorum enters fail-safe and blocks promotion,
- [x] rewind rejoin of former primary,
- [x] fencing-before-promotion when split-brain signals exist,
- [x] split-brain prevention (do not allow two primaries simultaneously).
- [x] Fault injection is limited to external stimuli only (stopping postgres, pausing etcd, writing request keys), never direct HA state mutation.
- [x] Define scenario assertions around control-plane invariants that are implemented today (leadership key ownership, HA phase, dispatched process jobs) and explicitly avoid assuming replication-manager features that do not exist yet.

8. Concurrency and determinism controls
- [x] Run all node loops concurrently inside one Tokio runtime and avoid manual `step_once` orchestration for e2e scenarios.
- [x] Use bounded waits/retries with clear timeout messages to avoid flaky hangs.
- [x] Keep make targets sequential (not parallel) to avoid Cargo artifact lock/contention issues observed previously.

9. Standalone e2e entrypoint and artifacts
- [x] Add a dedicated test target/file name for multi-node e2e so it can be run standalone quickly.
- [x] Ensure every scenario writes cluster logs/trace summaries into namespace-local artifact files and prints the namespace root on failure.
- [x] Document the exact standalone run command in the task file after implementation.

10. Verification gates (must all pass)
- [x] Run targeted new e2e tests first until stable.
- [x] Run `make check`.
- [x] Run `make test`.
- [x] Run `make test-bdd`.
- [x] Run `make lint`.
- [x] If any scenario/gate fails, create bug task(s) with `$add-bug` including timeline/log artifact paths and exact repro commands.

11. Task completion bookkeeping (post-green only)
- [x] Tick acceptance criteria with evidence.
- [x] Update task header tags to done/passes true and set `<passing>true</passing>` only after all required gates are green.
- [x] Run `/bin/bash .ralph/task_switch.sh`.
- [x] Commit all changes (including `.ralph` updates) with:
- [x] `task finished 13-task-e2e-multi-node-real-ha-loops-scenario-matrix: <summary + gate evidence + challenges>`
- [x] Append new learnings/surprises to `AGENTS.md`.
</execution_plan>

<evidence>
- Standalone matrix command: `cargo test ha::e2e_multi_node::e2e_multi_node_real_ha_scenario_matrix -- --nocapture`
- Timeline artifacts: `.ralph/evidence/13-e2e-multi-node/ha-e2e-scenario-matrix-*.timeline.log`
- Verification gates (all passing): `make check`, `make test`, `make test-bdd`, `make lint`
</evidence>

NOW EXECUTE
