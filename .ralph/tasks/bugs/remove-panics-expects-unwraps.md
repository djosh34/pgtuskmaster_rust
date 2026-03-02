---
## Bug: Remove panics/expects/unwraps in codebase <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
`rg -n "unwrap\(|expect\(|panic!" src tests` shows multiple occurrences (mostly in tests and some src modules like `src/process/worker.rs`, `src/pginfo/state.rs`, `src/pginfo/query.rs`, `src/dcs/worker.rs`, `src/dcs/store.rs`, `src/ha/worker.rs`, `tests/bdd_state_watch.rs`, `src/config/parser.rs`). Policy requires no unwraps/panics/expects anywhere; replace with proper error handling and remove any lint exemptions if present. Explore and confirm current behavior before changing.
</description>

<acceptance_criteria>
- [x] `make check` — passes cleanly
- [x] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [x] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [x] `make test-bdd` — all BDD features pass
</acceptance_criteria>

## Implementation Plan (Draft)

### 1) Baseline and scope lock
- Capture an exact inventory before edits:
  - `rg -n "\\bpanic!\\s*\\(|\\bexpect(_err)?\\s*\\(|\\bunwrap(_err)?\\s*\\(" src tests -g '*.rs'`
- Confirm current task-scoped callsites (expected at plan time):
  - `tests/bdd_state_watch.rs`
  - `src/state/watch_state.rs`
  - `src/process/worker.rs` (test module)
  - `src/dcs/worker.rs` (test module)
  - `src/dcs/store.rs` (test module)
  - `src/dcs/keys.rs` (test module)
  - `src/ha/worker.rs` (test module)
  - `src/pginfo/state.rs` (test module)
  - `src/pginfo/query.rs` (test module)
  - `src/ha/decide.rs` (test module)
  - `src/config/parser.rs` (test module)
- Do not widen runtime behavior unless required; prioritize deterministic test refactors that preserve assertions.

### 2) Refactor pattern decisions (consistent policy)
- Replace `.expect(...)`/`.expect_err(...)` in tests with one of:
  - explicit `match` + `assert!(matches!(...))` + structured assertions (preferred where only one callsite would use `?`)
  - test functions returning `Result<(), WorkerError>` / `Result<(), ConfigError>` / `Result<(), Box<dyn Error>>` and using `?` (use this for multi-step setup/IO-heavy tests)
  - `assert!(result.is_ok())`/`assert!(result.is_err())` + explicit match on error variant
- Replace `panic!` in match fallthroughs with explicit `assert!(matches!(...))` then destructure, or `match` that returns `Err(...)` from `Result` test functions.
- For time/file setup in config tests (`SystemTime::duration_since`, `std::fs::write`), make tests fallible and propagate errors; avoid panicking setup code.
- Keep production code panic-free as-is; all observed violations are test-side and should be removed without changing runtime contracts.

### 3) File-by-file execution plan
- `tests/bdd_state_watch.rs`
  - Convert test to `-> Result<(), StateRecvError>` (or boxed error), replace publish/changed `.expect` with `?`, keep closed-channel assertion intact.
- `src/state/watch_state.rs` tests
  - Convert async tests using publish/changed expectations into `Result`-returning tests.
  - Preserve semantic checks on version increments and channel-close propagation.
- `src/process/worker.rs` tests
  - Replace `panic!` branches in state assertions with `assert!(matches!(...))` + follow-up destructuring.
  - For `last_rejection` presence, use `let Some(rejection) = ... else { return Err(...) };` in `Result` test, or equivalent non-panicking assertion flow.
- `src/dcs/worker.rs` tests
  - Replace manual `serde_json::to_string` panic branch with `?` by converting test signature to fallible.
- `src/dcs/store.rs` tests
  - Replace encoding and drain panic branches with `?`.
  - Replace decode-error fallback panic with `assert!(matches!(...))`.
- `src/dcs/keys.rs` tests
  - Replace wrong-scope fallback panic with `assert!(matches!(...))`.
- `src/ha/worker.rs` tests
  - Replace process channel `panic!` branches with explicit `assert!(matches!(...))` and deterministic extraction.
- `src/pginfo/state.rs` tests
  - Replace enum-shape panic fallthroughs with `assert!(matches!(...))` + structured extraction.
- `src/pginfo/query.rs` tests
  - Replace parse success panic branch with `?` style assertion.
- `src/ha/decide.rs` tests
  - Replace decision `.expect(...)` callsites with either direct `?` in fallible tests or a small non-panicking local helper that returns `Result<DecideOutput, WorkerError>`.
  - Keep scenario table assertions unchanged; only error-handling surface changes.
- `src/config/parser.rs` tests
  - Remove `.expect_err(...)` and panic fallthroughs by pattern matching on returned errors via `match` + `assert_eq!`.
  - Convert roundtrip/invalid-file tests to return `Result<(), Box<dyn std::error::Error>>` so file writes/time checks use `?`.
  - Keep cleanup (`remove_file`) best-effort and non-panicking.

### 4) Incremental verification (before full gates)
- Fast compile/sig check before running full tests:
  - `cargo test --all-targets --no-run`
- After each module group, run focused commands to catch breakage early:
  - `cargo test --all-targets state::watch_state`
  - `cargo test --all-targets process::worker::tests`
  - `cargo test --all-targets dcs::store::tests dcs::worker::tests dcs::keys::tests`
  - `cargo test --all-targets ha::decide::tests ha::worker::tests`
  - `cargo test --all-targets pginfo::state::tests pginfo::query::tests config::parser::tests`
- Re-run inventory command and require zero matches in `src/` + `tests/`.

### 5) Required gate execution (sequential only)
- Run sequentially (no parallel cargo invocations):
  - `make check`
  - `make test`
  - `make test-bdd`
  - `make lint`
- If any gate fails:
  - fix root cause
  - re-run failed gate
  - re-run full gate chain to confirm no hidden regressions.

### 6) Task closure steps (after implementation succeeds)
- Update this task file:
  - mark checklist boxes completed
  - set `<status>done</status>` and `<passes>true</passes>` (and `<passing>true</passing>` if required by task-runner contract)
  - record concise evidence (commands + outcome)
- Append new learnings/surprises to `AGENTS.md` if any non-obvious issue surfaced.
- Run `/bin/bash .ralph/task_switch.sh`.
- Commit all changes (including `.ralph` artifacts) with:
  - `task finished remove-panics-expects-unwraps: <summary with gate evidence and key challenge>`

## Execution Evidence

- `rg -n "\\bpanic!\\s*\\(|\\bexpect(_err)?\\s*\\(|\\bunwrap(_err)?\\s*\\(" src tests -g '*.rs'` returned no matches after refactor.
- `cargo test --all-targets --no-run` passed.
- Focused suites passed:
  - `cargo test --all-targets state::watch_state`
  - `cargo test --all-targets process::worker::tests`
  - `cargo test --all-targets dcs::store::tests`
  - `cargo test --all-targets dcs::worker::tests`
  - `cargo test --all-targets dcs::keys::tests`
  - `cargo test --all-targets ha::decide::tests`
  - `cargo test --all-targets ha::worker::tests`
  - `cargo test --all-targets pginfo::state::tests`
  - `cargo test --all-targets pginfo::query::tests`
  - `cargo test --all-targets config::parser::tests`
- Required gates passed sequentially:
  - `make check`
  - `make test`
  - `make test-bdd`
  - `make lint`
