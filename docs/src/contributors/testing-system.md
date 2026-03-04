# Testing System Deep Dive

This project verifies behavior at multiple layers so that:

- pure logic is correct and remains easy to refactor, and
- the “real world” system (real Postgres, real etcd, real timing) is exercised end-to-end.

HA systems are especially vulnerable to false confidence: a unit test can pass while the real system fails due to process startup behavior, coordination timing, or “unhappy path” state transitions.

This chapter describes what each test layer can prove, how the Makefile gates are structured, and how to choose the right test type when adding coverage.

## Test layers

### Unit tests (fast, pure, refactor-friendly)

Best for:

- pure decision logic (for example HA state machine transitions)
- small data model invariants
- parsing/validation behavior
- edge-case matrices.

Evidence style:

- pass typed snapshots into functions
- assert exact outputs and state transitions
- avoid spawning real processes or relying on timing.

Representative examples:

- HA transition matrix tests in `src/ha/decide.rs`
- state channel semantics tests in `src/state/watch_state.rs`.

### Contract/unit-integration tests (workers in isolation)

Best for:

- “this worker does the right thing given these inputs”
- dispatch behavior (which DCS keys are written, which process jobs are enqueued)
- error handling (faulting worker status, retry behavior).

These tests often use “contract stub” contexts (for example `HaWorkerCtx::contract_stub(...)`) with recording stores or fake inboxes.

Representative examples:

- dispatch tests in `src/ha/worker.rs` (recording DCS store, recording process inbox)
- worker contract tests in `src/worker_contract_tests.rs`.

### Black-box / BDD-style integration tests (external behavior)

Best for:

- “if I call the API endpoint, do I get the correct response shape and DCS writes?”
- CLI behavior and compatibility expectations
- policy/safety guards that should be enforced at the interface boundary.

Representative examples (in `tests/`):

- `bdd_api_http.rs`: exercises the HTTP surface as an external client
- `bdd_state_watch.rs`: asserts watch/state semantics from a client perspective
- `cli_binary.rs`: smoke tests around the built binary invocation
- `policy_e2e_api_only.rs`: policy-level checks that should hold without needing a full HA cluster.

### Real-binary e2e tests (slow, highest confidence)

Best for:

- “does this work with real Postgres and real etcd?”
- lifecycle sequencing under timing variation
- process-level failure behavior (crashes, restarts, fencing, rewind behavior)
- network fault injection (blocked links, latency).

These tests use the harness (`src/test_harness/*`) and are considered required gates: missing binaries are an environment problem to fix, not a reason to skip tests.

## Why this depth exists

Each layer answers a different question:

- **unit tests**: “is the logic right?”
- **worker/contract tests**: “is the boundary correct (writes, job dispatch, error handling)?”
- **BDD tests**: “does the external contract behave as intended?”
- **real-binary e2e**: “does the full system converge under real constraints?”

If you only add unit tests, you will miss regressions in side-effect boundaries. If you only add e2e, refactors become painful and slow.

## Tradeoffs

Deep coverage costs runtime and requires local prerequisites (real binaries under `.tools/`). The payoff is confidence in failure-path behavior that matters in production, especially for HA transitions and fencing.

This repo is intentionally strict about not silently skipping tests: a “green” run that skipped real-binary tests is worse than a failing run, because it hides risk.

## Makefile gates: `make test` vs `make test-long`

The Makefile splits the default test run from “ultra-long” scenarios:

- `make test`:
  - runs `cargo test --all-targets`
  - skips a curated list of ultra-long tests
  - validates that every skip token is an **exact match** (preflight `-- --list`), so the run fails closed if a test is renamed or missing.
- `make test-long`:
  - runs only the ultra-long tests, one by one, with `-- --exact`
  - is intended for scenarios that take minutes and are not appropriate for a tight edit/compile loop.

When you add a new slow scenario:

- first try to keep it in `make test`
- if it consistently exceeds “developer cycle” time, move it to `make test-long` by adding it to the Makefile’s `ULTRA_LONG_TESTS` list.

## When to add which test type (decision guide)

Use this as a rule of thumb:

- You changed **pure logic** (state transition rules, parsing, small helpers) → add/extend **unit tests**.
- You changed **which side effects happen** (DCS key writes, process job mapping, dispatch retries) → add/extend a **worker contract test** with a recording store/inbox.
- You changed **an external contract** (API route, response fields, auth behavior) → add/extend a **BDD test** in `tests/`.
- You changed **timing, startup, or real processes** (basebackup/rewind, fencing, leader lease stability) → add/extend a **real-binary e2e** scenario using the harness.

When in doubt, add two tests: one fast (unit/contract) and one realistic (BDD/e2e).

## Minimal inventory: what protects what

This is a deliberately small “map” of important tests and the subsystem boundary they protect:

- `src/ha/decide.rs` tests: HA phase transitions and action selection logic.
- `src/ha/worker.rs` tests: HA dispatch behavior (DCS writes/deletes, process job enqueueing) and error surfacing.
- `tests/bdd_api_http.rs`: API contract and intent writes (for example switchover endpoints).
- `tests/bdd_state_watch.rs`: state/watch semantics visible to clients.
- `tests/cli_binary.rs`: binary-level smoke coverage (packaging, CLI invocation).
- `src/ha/e2e_*.rs`: multi-node real-binary HA scenarios (leader election, switchover/failover/fencing sequences).

## Flake triage (symptoms → likely causes → next probe)

When a test fails, start by classifying it as:

- deterministic logic failure (repeatable), or
- environment/timing failure (intermittent).

Common symptoms and next steps in this repo:

- **“real-binary prerequisite missing”**:
  - likely cause: `.tools/postgres16/bin/*` or `.tools/etcd/bin/etcd` not installed
  - next probe: run the installer scripts under `tools/` and re-run the failing test.
- **Port already in use / connection refused**:
  - likely cause: incomplete teardown left a process running, or port reservation logic is being bypassed
  - next probe: inspect the test namespace under `/tmp/pgtuskmaster-rust/*` for leftover processes/logs.
- **Unix socket path length or permissions issues**:
  - likely cause: too-long socket dir path or incorrect permissions on a data directory
  - next probe: check harness namespace layout and postgres startup logs.
- **DCS connect timeouts in single-threaded tokio tests**:
  - likely cause: long synchronous work starving an in-runtime listener/proxy
  - next probe: check whether a proxy listener is running on a dedicated thread runtime.

Treat flakes as bugs. If a test is important enough to exist, it is important enough to be made reliable.

## Adjacent subsystem connections

Testing coverage is meaningful only if you understand what is being exercised:

- Read [Harness Internals](./harness-internals.md) for how real-binary tests construct namespaces, allocate ports, start etcd/postgres, and inject faults.
- Read [HA Decision and Action Pipeline](./ha-pipeline.md) to understand which transitions your tests should assert for each scenario.
- Read [API and Debug Contracts](./api-debug-contracts.md) to understand which interfaces need black-box coverage vs internal-only assertions.

## Failure triage workflow (simple loop)

Use this operational loop when debugging:

1. Reproduce in the narrowest failing layer (unit → contract → BDD → e2e).
2. Identify which boundary is failing (observation, cache/trust, decision, dispatch, process).
3. Capture the most useful artifact for the layer:
   - unit/contract: the input snapshot that triggered failure
   - BDD: request/response pairs and DCS writes
   - e2e: namespace logs + debug snapshot timeline.
4. Fix the root cause and add a regression test at the narrowest layer that can prove it.
