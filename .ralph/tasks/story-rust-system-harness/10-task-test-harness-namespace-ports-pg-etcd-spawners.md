---
## Task: Build parallel-safe real-system test harness for PG16 and etcd3 <status>done</status> <passes>true</passes> <passing>true</passing> <priority>ultra_high</priority>

<blocked_by>02-task-runtime-config-schema-defaults-parse-validate,03-task-worker-state-models-and-context-contracts</blocked_by>

<description>
**Goal:** Provide deterministic, parallel-safe infrastructure for real integration and e2e tests.

**Scope:**
- Implement `src/test_harness/namespace.rs`, `ports.rs`, `pg16.rs`, `etcd3.rs`, `tls.rs`, `auth.rs`, and `mod.rs`.
- Implement `create_namespace`, `cleanup_namespace`, `allocate_ports`, `prepare_pgdata_dir`, `prepare_etcd_data_dir`, `spawn_pg16`, and `spawn_etcd3`.
- Ensure no shared dirs/sockets/ports across tests.

**Context from research:**
- Plan requires real-system heavy testing and parallel-safe isolation.

**Expected outcome:**
- Tests can run multiple nodes and infra instances concurrently without cross-test interference.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] Every test instance uses unique namespace + unique dirs + unique ports.
- [x] Harness cleanup executes on success and failure paths.
- [x] Harness tests verify concurrent test runs do not conflict.
- [x] Run targeted harness tests.
- [x] Run `make check`.
- [x] Run `make test`.
- [x] Run `make lint`.
- [x] Run `make test`.
- [x] If failures occur, add `$add-bug` tasks with namespace/port collision logs.
</acceptance_criteria>
<execution_plan>
## Detailed Implementation Plan

1. Baseline, prerequisites, and contract lock-in
- [x] Confirm blockers `02` and `03` remain done/passing before introducing new harness modules.
- [x] Run a baseline compile (`cargo check --all-targets`) and capture current green state before changes.
- [x] Confirm no `src/test_harness` module exists yet and wire it in `src/lib.rs` under `#[cfg(any(test, feature = "test-harness"))]` for test-only default visibility.

2. Create harness module skeleton and error model
- [x] Add `src/test_harness/mod.rs` with explicit submodules: `namespace`, `ports`, `pg16`, `etcd3`, `tls`, `auth`.
- [x] Define a crate-local `HarnessError` in `mod.rs` using `thiserror`, covering IO errors, process spawn failures, timeout, and invalid command outcomes.
- [x] Keep module access via direct paths (no broad root re-export fanout) to preserve minimal visibility and avoid unused import drift.

3. Namespace lifecycle (`namespace.rs`)
- [x] Implement `create_namespace(test_name)` returning a deterministic unique identifier containing sanitized test name + process id + atomic counter to avoid collisions under parallel tests.
- [x] Implement namespace root directory creation under `std::env::temp_dir()` + project-specific prefix.
- [x] Implement `cleanup_namespace(...)` that recursively removes namespace resources and is idempotent when paths are already gone.
- [x] Add RAII guard type (`NamespaceGuard`) that calls cleanup in `Drop` best-effort path so both success/failure paths attempt cleanup.

4. Port allocation (`ports.rs`)
- [x] Implement `allocate_ports(count: usize)` that binds ephemeral listeners on `127.0.0.1:0`, records assigned ports, and keeps listeners alive until caller explicitly drops reservation.
- [x] Validate no duplicate ports within a reservation and return structured error on `count == 0` or OS bind failure.
- [x] Provide helper methods (`as_slice`, `into_vec`) and keep reservations non-`Clone` so ownership clearly controls release timing.
- [x] Add concurrent tests using many tasks to prove no intra-run collisions.

5. Data directory preparation (`pg16.rs` and `etcd3.rs`)
- [x] Implement `prepare_pgdata_dir(namespace, node_id)` and `prepare_etcd_data_dir(namespace)` using namespace-scoped unique paths.
- [x] Ensure directory creation is recursive and rejects accidental reuse when stale directory exists without explicit cleanup.
- [x] Keep path generation deterministic from inputs so failures are diagnosable.

6. PG16 process spawner (`pg16.rs`)
- [x] Implement `spawn_pg16(...)` with `tokio::process::Command` (enable `tokio` `process` feature in `Cargo.toml`).
- [x] Use unique unix socket dir / port args per namespace, and write stdout/stderr to namespace-local log files.
- [x] Add startup readiness probing with bounded timeout using TCP connect checks on assigned port and clear error reporting if postgres fails to become ready.
- [x] Return a handle that supports graceful shutdown (`SIGTERM`) and forced kill fallback without `unwrap`.

7. etcd3 process spawner (`etcd3.rs`)
- [x] Implement `spawn_etcd3(...)` similarly with namespace-local data dir, listen/client/peer ports from reservation, and log file capture.
- [x] Add readiness wait loop (client endpoint TCP connect health) with timeout and structured failure context.
- [x] Return shutdown-capable handle with graceful-first termination semantics.

8. TLS/auth placeholders (`tls.rs`, `auth.rs`)
- [x] Add explicit typed stubs/config helpers needed by harness callers, but keep scope minimal for this task.
- [x] Ensure these modules compile cleanly and document `TODO` boundaries for future security tasks without dead code warnings.

9. Harness tests and parallel safety proof
- [x] Add focused tests under `src/test_harness/*` for namespace uniqueness, cleanup idempotency, and port allocation concurrency.
- [x] Add integration-style unit tests that spin multiple namespace instances concurrently and assert unique dirs/ports plus no cross-interference.
- [x] Gate real binary-dependent tests behind environment checks (skip with clear message when `postgres`/`etcd` binaries are unavailable) so CI remains deterministic.

10. Full verification and task bookkeeping
- [x] Run targeted harness tests first for rapid iteration.
- [x] Run `make check`.
- [x] Run `make test`.
- [x] Run `make test`.
- [x] Run `make lint`.
- [x] If any failure indicates namespace/port conflict, create `$add-bug` task(s) with reproduction command, logs, and failing namespace identifiers.
- [x] After successful implementation phase, tick acceptance criteria and update task tags to done/passing per workflow.

11. Skeptical verification amendments (added during TO BE VERIFIED)
- [x] Change module wiring from unconditional exposure to `#[cfg(any(test, feature = "test-harness"))]` to prevent production surface creep while keeping opt-in integration test access.
- [x] Remove explicit `release()` API from port reservations and use ownership/drop semantics to reduce misuse paths.
- [x] Tighten readiness probing to pure TCP-connect loops instead of assuming CLI helpers (`pg_isready`, `etcdctl`) exist.

NOW EXECUTE
</execution_plan>
