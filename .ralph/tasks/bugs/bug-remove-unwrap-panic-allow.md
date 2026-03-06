---
## Bug: Remove Clippy Allowances For Unwrap/Panic <status>done</status> <passes>true</passes>

<description>
src/test_harness/mod.rs explicitly allows clippy unwrap/expect/panic, which violates the repo rule against unwraps, panics, or expects anywhere. This hides violations in test harness code and makes it easy to slip new ones in. Investigate all test_harness code (and any other modules) for unwrap/expect/panic usage, replace with proper error handling, and remove the lint allow attributes.
</description>

<acceptance_criteria>
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] BDD features pass (covered by `make test`).
</acceptance_criteria>

<implementation_plan>
## Plan (drafted 2026-03-02)

### Scope confirmed from deep skeptical scan
- `src/test_harness/mod.rs` has crate-level lint allowances: `#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]`.
- `src/test_harness` contains explicit `panic!` call sites in:
  - `mod.rs` test module.
  - `namespace.rs` test module.
  - `pg16.rs` test module.
  - `binaries.rs` runtime helper (`require_binary`) currently panics when binaries are absent.
- `src/test_harness/etcd3.rs` and `src/test_harness/ports.rs` already follow `Result`-based test patterns and can be used as style references.
- `require_*` binary helpers are consumed by tests outside `src/test_harness` (notably `src/pginfo/worker.rs`, `src/process/worker.rs`, and `src/dcs/etcd_store.rs` test modules), so any API change must include those call sites.

### Execution plan
1. Remove blanket lint exceptions from harness root.
- Edit `src/test_harness/mod.rs`:
  - delete `#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]`.
  - keep only the allowances that are still justified (`dead_code`) if needed.

2. Make binary lookup non-panicking and propagate errors through all callers.
- Refactor `src/test_harness/binaries.rs`:
  - change `require_binary` to return `Result<PathBuf, HarnessError>` instead of panicking.
  - propagate that change through `require_etcd_bin`, `require_pg16_bin`, and `require_pg16_process_binaries`.
- Update all harness call sites to use `?` and return rich errors.
- Update all non-harness test call sites (`pginfo`, `process`, `dcs`) that currently assume infallible binary resolution.

3. Convert remaining panic-based harness tests to explicit error flows.
- `src/test_harness/mod.rs` test:
  - convert to `fn ...() -> Result<(), HarnessError>`.
  - replace manual `match` + `panic!` paths with `?` and assertions.
  - use `NamespaceGuard` where possible to avoid cleanup panics.
- `src/test_harness/namespace.rs` tests:
  - convert each panic path to `Result` and `?` propagation.
- `src/test_harness/pg16.rs` tests:
  - remove all panic branches.
  - preserve cleanup guarantees by structuring test body as `Result` and performing shutdown/cleanup regardless of assertion outcomes (without unwrap/expect/panic).

4. Validate no prohibited patterns remain in harness sources and no stale non-panicking assumptions remain.
- Run targeted searches:
  - `rg -n "\\b(expect\\(|unwrap\\(|panic!\\()" src/test_harness`
  - `rg -n "allow\\(clippy::(unwrap_used|expect_used|panic)" src/test_harness`
- Run targeted API-impact search:
  - `rg -n "require_binary\\(|require_etcd_bin\\(|require_pg16_bin\\(|require_pg16_process_binaries\\(" src`
- If any matches remain, iterate until clean.

5. Run fast focused checks before full gates (to catch interface fallout early).
- `cargo test test_harness -- --nocapture` (or closest targeted subset if naming differs)
- targeted module tests for updated non-harness call sites:
  - `cargo test pginfo::worker::tests -- --nocapture`
  - `cargo test process::worker::tests -- --nocapture`
  - `cargo test dcs::etcd_store::tests -- --nocapture`

6. Run mandatory full gates sequentially.
- Execute and verify pass/fail exactly in this order:
  - `make check`
  - `make test`
  - `make test-long`
  - `make lint`
- If any command fails, fix root cause and rerun from the failing gate onward, then re-run full sequence for confidence.

7. Task bookkeeping after successful execution.
- Update this task file:
  - tick acceptance checkboxes.
  - set `<status>done</status>` and `<passes>true</passes>`.
  - add concise evidence (command outcomes and any notable fixes).
- Run `/bin/bash .ralph/task_switch.sh`.
- Commit all changed files (including `.ralph/*`) with message format:
  - `task finished bug-remove-unwrap-panic-allow: ...` with test evidence and implementation summary.
- Append cross-task learnings/surprises to `AGENTS.md`.
</implementation_plan>

## Execution Evidence (2026-03-03)
- Refactored `src/test_harness/binaries.rs` so binary lookup APIs return `Result<..., HarnessError>` instead of panicking.
- Removed `#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]` from `src/test_harness/mod.rs`.
- Converted panic-based harness tests to `Result` flows in:
  - `src/test_harness/mod.rs`
  - `src/test_harness/namespace.rs`
  - `src/test_harness/pg16.rs`
- Propagated fallible binary lookup to impacted non-harness test modules:
  - `src/pginfo/worker.rs`
  - `src/dcs/etcd_store.rs`
  - `src/process/worker.rs`
- Verification searches after changes:
  - `rg -n "\\b(expect\\(|unwrap\\(|panic!\\()" src/test_harness` (no matches)
  - `rg -n "allow\\(clippy::(unwrap_used|expect_used|panic)" src/test_harness` (no matches)
- Targeted tests passed:
  - `cargo test test_harness -- --nocapture`
  - `cargo test pginfo::worker::tests -- --nocapture`
  - `cargo test process::worker::tests -- --nocapture`
  - `cargo test dcs::etcd_store::tests -- --nocapture`
- Required gates passed sequentially:
  - `make check`
  - `make test`
  - `make test-long`
  - `make lint`
