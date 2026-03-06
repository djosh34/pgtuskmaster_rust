## Task: Enforce zero panic/unwrap/expect across runtime and tests with proper Result handling <status>done</status> <passes>true</passes> <priority>high</priority>

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
- [x] Full exhaustive checklist of all files/modules to modify with specific requirements for each:
- [x] `src/lib.rs` — remove test-only deny bypass; enforce clippy no-unwrap/no-expect/no-panic policy for test and runtime targets.
- [x] `Makefile` — update `lint` target so strict deny policy is applied to test targets too (not only `--lib`), and remains deterministic in CI/local.
- [x] `src/test_harness/mod.rs` — remove broad `#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]`; keep no exceptions.
- [x] `src/test_harness/binaries.rs` — replace `panic!`-based binary requirements with typed `Result` APIs and propagate failures to callers.
- [x] `src/test_harness/namespace.rs` — remove panic branches in tests; use `Result`-returning tests and explicit cleanup error propagation.
- [x] `src/test_harness/ports.rs` — remove thread/mutex/join panic branches; convert to fallible flows with error propagation.
- [x] `src/test_harness/etcd3.rs` — remove panic-based setup/cleanup/spawn/shutdown handling in tests; propagate typed errors.
- [x] `src/test_harness/pg16.rs` — remove panic-based setup/cleanup/spawn/shutdown handling in tests; propagate typed errors.
- [x] `src/process/worker.rs` (test module) — replace panic control-flow branches with assertions and/or `Result` propagation.
- [x] `src/pginfo/worker.rs` (test module) — replace panic-heavy orchestration with `Result`-returning async tests and structured matching.
- [x] `src/dcs/keys.rs` (test module) — replace panic branches for unexpected variants with explicit assertions.
- [x] `src/dcs/store.rs` (test module) — replace panic branches with assertions/`Result` handling.
- [x] `src/dcs/worker.rs` (test module) — replace panic branch with assertion/`Result` handling.
- [x] `src/ha/worker.rs` (test module) — replace panic branches in outcome/request matching with assertions.
- [x] `src/pginfo/state.rs` (test module) — replace panic branches in variant checks with assertions.
- [x] `src/pginfo/query.rs` (test module) — replace panic branch on parse success path with assertion/`Result` handling.
- [x] `src/config/parser.rs` (test module) — replace `expect/expect_err` and panic variant branches with `Result`-returning tests plus explicit match/assertions.
- [x] `src/state/watch_state.rs` (test module) — replace `expect` calls with propagated results/assertions.
- [x] `src/worker_contract_tests.rs` — replace `expect` calls on worker steps with `Result` handling.
- [x] `src/ha/decide.rs` (test module) — replace `expect` calls with `Result` handling and explicit assertion flow.
- [x] `tests/bdd_state_watch.rs` — replace `expect` calls with propagated results/assertions.
- [x] Repo-wide verification command `rg -n "\\bpanic!\\s*\\(|\\bexpect(_err)?\\s*\\(|\\bunwrap(_err)?\\s*\\(" src tests -g '*.rs'` returns zero matches.
- [x] Repo-wide verification command `rg -n "allow\\(clippy::(unwrap_used|expect_used|panic)\\)" src -g '*.rs'` returns zero matches.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] BDD features pass (covered by `make test`).
</acceptance_criteria>

## Execution Plan (Research Baseline: 2026-03-03)

### Parallel research evidence summary
- `rg -n "panic!|expect\\(|expect_err\\(|unwrap\\(|unwrap_err\\(" src tests -g '*.rs'` currently returns zero matches (stale inventory in task description).
- All listed hotspot files in acceptance criteria currently report `0` matches for panic/expect/unwrap patterns.
- Remaining enforcement gaps confirmed:
- `src/lib.rs` still gates strict clippy policy behind `cfg_attr(not(test), deny(...))`.
- `Makefile` strict deny invocation is currently runtime-only (`cargo clippy --lib ...`) and does not add explicit strict test-target pass.
- `src/test_harness/mod.rs` no longer contains broad clippy allow exceptions for panic/expect/unwrap.

### Step-by-step execution plan
1. Baseline and traceability
- Capture current `git status --short` and save task-local evidence under `.ralph/evidence/05c-zero-panic/` so pre-change and post-change state are auditable.
- Re-run repo-wide panic/expect/unwrap scan and clippy-allow scan; archive outputs for this task as "stale-inventory disproval" evidence.
- Record current `src/lib.rs` and `Makefile` snippets into evidence before editing so enforcement delta is reviewable line-by-line.

2. Remove runtime-vs-test deny split in crate root
- Edit `src/lib.rs` to enforce strict clippy deny policy without `cfg_attr(not(test), ...)` test bypass.
- Keep the same deny set (`unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`) but make it active uniformly for checked targets.
- Verify formatting and compile-level behavior by running `cargo check --all-targets`.

3. Strengthen lint pipeline for test targets
- Edit `Makefile` `lint` target to keep global warning pass and add strict clippy deny enforcement for tests explicitly (not only `--lib`).
- Ensure deterministic command sequencing (no parallel cargo invocations) and preserve existing developer ergonomics.
- Proposed strict passes:
- `cargo clippy --lib --all-features -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::todo -D clippy::unimplemented`
- `cargo clippy --tests --all-features -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::todo -D clippy::unimplemented`
- Add a strict all-targets guard pass for skeptical coverage:
- `cargo clippy --all-targets --all-features -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::todo -D clippy::unimplemented`

4. Regression sweep for hidden panic-style patterns
- Run exhaustive grep:
- `rg -n "\\bpanic!\\s*\\(|\\bexpect(_err)?\\s*\\(|\\bunwrap(_err)?\\s*\\(" src tests -g '*.rs'`
- `rg -n "allow\\(clippy::(unwrap_used|expect_used|panic)\\)" src -g '*.rs'`
- If any hits appear, convert each to `Result` propagation and explicit assertions without introducing `expect`, `unwrap`, or manual panic control-flow.

5. Targeted file-by-file validation against acceptance checklist
- For each required module listed in acceptance criteria, validate no panic/expect/unwrap control flow remains and no lint-allow escapes are introduced.
- If a module regressed during lint hardening, patch it with typed errors and `?` propagation, then rerun local module tests before full gates.

6. Mandatory quality gates (sequential, strict status capture)
- Run exactly and sequentially:
- `make check`
- `make test`
- `make test-long`
- `make lint`
- Use `set -o pipefail` if logging through `tee`, and archive logs under `.ralph/evidence/05c-zero-panic/gates/`.
- For `make test` and `make lint`, also grep archived logs for acceptance phrases (`congratulations`/`evaluation failed`) and save grep outputs as explicit acceptance evidence.
- Do not mark task passed unless all four commands exit cleanly.
- If any gate fails with stale/corrupt artifact symptoms (`failed to build archive` / missing `*.rcgu.o`), run one `cargo clean` recovery and rerun the full gate sequence once; archive both failing and recovery logs.

7. Task completion bookkeeping
- Update this task file checkboxes with concrete evidence references.
- Set `<status>done</status>` and `<passes>true</passes>` only after all gates pass.
- Run `/bin/bash .ralph/task_switch.sh`.
- Commit all relevant files (including `.ralph` updates) with message:
- `task finished 05c-task-zero-panic-unwrap-expect-across-runtime-and-tests: <summary with gate evidence and implementation notes>`
- Append final learnings/surprises to `AGENTS.md` if new cross-task insights were discovered.

NOW EXECUTE
