## Bug: Test harness binary checks panic instead of returning errors <status>done</status> <passes>true</passes>

<description>
The test harness binary lookup in [src/test_harness/binaries.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/test_harness/binaries.rs) uses `panic!` to report missing binaries. This conflicts with the project policy of no `panic`/`expect`/`unwrap` and makes tests fail via uncontrolled panics rather than structured errors. Refactor `require_binary` (and callers) to return a typed `HarnessError` instead of panicking, and update callers/tests to propagate or assert errors explicitly.
</description>

<acceptance_criteria>
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] BDD features pass (covered by `make test`).
</acceptance_criteria>

## Implementation Plan (Draft)

### 1) Reconfirm bug still exists before touching code
- Validate current `src/test_harness/binaries.rs` behavior and signature:
  - `require_binary(path: &Path) -> Result<PathBuf, HarnessError>`
  - ensure missing binaries map to `HarnessError` (not `panic!`)
- Add or verify an explicit contract test in `src/test_harness/binaries.rs` for a missing path:
  - assert `require_binary(missing)` returns `Err(HarnessError::InvalidInput(_))`
  - this prevents regressions where future refactors accidentally reintroduce panic behavior
- Sweep for panic/expect/unwrap use in harness lookup paths:
  - `rg -n "panic!|expect\\(|unwrap\\(" src/test_harness src/pginfo src/process src/dcs tests`
- Enumerate all binary-lookup call sites and verify they already propagate errors:
  - `rg -n "require_binary|require_etcd_bin|require_pg16_bin|require_pg16_process_binaries" src tests`

### 2) Branch on verification outcome (stale vs active)
- If panic behavior is still present anywhere in lookup flow:
  - refactor to return `HarnessError` and propagate via `?` or typed mapping in callers
  - add/adjust tests so error behavior is asserted explicitly (no panic fallthrough)
- If panic behavior is absent (likely stale task):
  - do **not** force speculative code changes
  - collect evidence proving typed-error path already implemented and used
  - mark task done as stale-fixed-with-evidence after full gates pass

### 3) If edits are needed, apply them with strict error handling
- Keep policy-compliant code: no `unwrap`, `expect`, `panic`.
- Prefer existing `HarnessError::InvalidInput` for missing binaries unless caller needs a more specific variant.
- Update any affected tests to `Result`-returning style and assert variants using `matches!`.
- Re-scan to enforce zero panic/expect/unwrap in touched areas.

### 4) Compile and focused validation before full gates
- Run quick confidence checks:
  - `cargo test --all-targets --no-run`
  - `cargo test --all-targets test_harness::binaries::tests`
  - targeted suites around call sites:
    - `cargo test --all-targets test_harness::pg16::tests`
    - `cargo test --all-targets test_harness::etcd3::tests`
    - `cargo test --all-targets pginfo::worker::tests`
    - `cargo test --all-targets process::worker::tests`
    - `cargo test --all-targets dcs::etcd_store::tests`

### 5) Required gate chain (sequential only)
- Run exactly and in order:
  - `make check`
  - `make test`
  - `make test-long`
  - `make lint`
- If any gate fails, fix root cause and rerun full chain from the top.

### 6) Task file and repo closure updates
- Update this task file:
  - set `<status>done</status>`
  - set `<passes>true</passes>`
  - set `<passes>true</passes>`
  - tick acceptance checkboxes and add command evidence
- Append any non-obvious learning/surprise to `AGENTS.md`.
- Run `/bin/bash .ralph/task_switch.sh`.
- Commit all changes (including `.ralph` files) with:
  - `task finished test-harness-binary-check-panics: <summary with gate evidence and key challenge>`

## Execution Notes

- Deep skeptical verification done; plan updated to add an explicit `require_binary` missing-path contract test before execution.
- Bug status: stale-fixed-with-evidence for panic behavior; `require_binary` already returned `Result<_, HarnessError>` and no panic path remained.
- Added regression in `src/test_harness/binaries.rs`:
  - `require_binary_missing_path_returns_invalid_input`
  - asserts `Err(HarnessError::InvalidInput(_))` for a guaranteed-missing temp path.
- Evidence from focused checks:
  - `cargo test --all-targets --no-run` passed.
  - `cargo test --all-targets test_harness::binaries::tests` passed (new test exercised).
  - targeted call-site suites passed (`test_harness::pg16::tests`, `test_harness::etcd3::tests`, `pginfo::worker::tests`, `process::worker::tests`, `dcs::etcd_store::tests`).
- Required gate chain passed in order:
  - `make check` passed.
  - `make test` passed (`94` unit/integration + BDD test binaries succeeded).
  - `make test` passed (`3` API BDD + `1` state-watch BDD).
  - `make lint` passed (`clippy --all-targets --all-features -D warnings` and runtime strict clippy pass).

DONE
