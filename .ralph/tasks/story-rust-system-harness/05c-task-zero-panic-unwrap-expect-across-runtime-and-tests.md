---
## Task: Enforce zero panic/unwrap/expect across runtime and tests with proper Result handling <status>not_started</status> <passes>false</passes> <priority>high</priority>

<description>
**Goal:** Remove all manual panic/unwrap/expect usage from runtime and test code, replace with proper Rust error handling, and make lint enforcement fail on any regression.

**Scope:**
- Enforce strict clippy policy for both runtime and test targets (no test exceptions).
- Refactor every current `panic!`, `expect`, and `expect_err` case in `src/` and `tests/` to idiomatic alternatives.
- Keep assertions (`assert!`, `assert_eq!`, `matches!`) for behavior checks, but remove manual panic control-flow.
- Ensure helper APIs used in tests return typed `Result` and are consumed via `?`/explicit error matching.

**Context from research:**
- Current inventory found:
- `panic!`: 139
- `expect/expect_err`: 30
- `unwrap()/unwrap_err()`: 0 direct hits
- Main concentration files:
- `src/pginfo/worker.rs` (39 panic)
- `src/process/worker.rs` (38 panic)
- `src/test_harness/pg16.rs` (14 panic)
- `src/test_harness/etcd3.rs` (14 panic)
- Lint gap root causes:
- `src/lib.rs` uses `cfg_attr(not(test), deny(...))` so deny is disabled under `cfg(test)`.
- `src/test_harness/mod.rs` explicitly allows `clippy::unwrap_used`, `clippy::expect_used`, `clippy::panic`.
- `Makefile` strict clippy deny pass is currently `--lib` only.

**Expected outcome:**
- Zero manual panic/unwrap/expect exceptions across checked Rust sources.
- Linter blocks reintroduction in runtime and test targets.
- Test behavior remains equivalent, but failures are expressed through `Result` propagation and explicit assertions/error matching.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Full exhaustive checklist of all files/modules to modify with specific requirements for each:
- [ ] `src/lib.rs` — remove test-only deny bypass; enforce clippy no-unwrap/no-expect/no-panic policy for test and runtime targets.
- [ ] `Makefile` — update `lint` target so strict deny policy is applied to test targets too (not only `--lib`), and remains deterministic in CI/local.
- [ ] `src/test_harness/mod.rs` — remove broad `#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]`; keep no exceptions.
- [ ] `src/test_harness/binaries.rs` — replace `panic!`-based binary requirements with typed `Result` APIs and propagate failures to callers.
- [ ] `src/test_harness/namespace.rs` — remove panic branches in tests; use `Result`-returning tests and explicit cleanup error propagation.
- [ ] `src/test_harness/ports.rs` — remove thread/mutex/join panic branches; convert to fallible flows with error propagation.
- [ ] `src/test_harness/etcd3.rs` — remove panic-based setup/cleanup/spawn/shutdown handling in tests; propagate typed errors.
- [ ] `src/test_harness/pg16.rs` — remove panic-based setup/cleanup/spawn/shutdown handling in tests; propagate typed errors.
- [ ] `src/process/worker.rs` (test module) — replace panic control-flow branches with assertions and/or `Result` propagation.
- [ ] `src/pginfo/worker.rs` (test module) — replace panic-heavy orchestration with `Result`-returning async tests and structured matching.
- [ ] `src/dcs/keys.rs` (test module) — replace panic branches for unexpected variants with explicit assertions.
- [ ] `src/dcs/store.rs` (test module) — replace panic branches with assertions/`Result` handling.
- [ ] `src/dcs/worker.rs` (test module) — replace panic branch with assertion/`Result` handling.
- [ ] `src/ha/worker.rs` (test module) — replace panic branches in outcome/request matching with assertions.
- [ ] `src/pginfo/state.rs` (test module) — replace panic branches in variant checks with assertions.
- [ ] `src/pginfo/query.rs` (test module) — replace panic branch on parse success path with assertion/`Result` handling.
- [ ] `src/config/parser.rs` (test module) — replace `expect/expect_err` and panic variant branches with `Result`-returning tests plus explicit match/assertions.
- [ ] `src/state/watch_state.rs` (test module) — replace `expect` calls with propagated results/assertions.
- [ ] `src/worker_contract_tests.rs` — replace `expect` calls on worker steps with `Result` handling.
- [ ] `src/ha/decide.rs` (test module) — replace `expect` calls with `Result` handling and explicit assertion flow.
- [ ] `tests/bdd_state_watch.rs` — replace `expect` calls with propagated results/assertions.
- [ ] Repo-wide verification command `rg -n "\\bpanic!\\s*\\(|\\bexpect(_err)?\\s*\\(|\\bunwrap(_err)?\\s*\\(" src tests -g '*.rs'` returns zero matches.
- [ ] Repo-wide verification command `rg -n "allow\\(clippy::(unwrap_used|expect_used|panic)\\)" src -g '*.rs'` returns zero matches.
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test-bdd` — all BDD features pass
</acceptance_criteria>
