---
## Bug: Real Binary Tests Become Optional Via Early Return <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
Several real-binary test paths silently return `Ok(())` when required binaries are not discovered (for example `None => return Ok(())`).
This makes critical runtime coverage optional and can mask regressions in HA/bootstrap/process behavior.

Explore and research the full codebase first, then implement a fix so real-binary tests are enforced instead of being skipped by default.
The solution should preserve clear error messages about missing prerequisites and keep CI/local workflows deterministic.
</description>

<acceptance_criteria>
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] BDD features pass (covered by `make test`).
</acceptance_criteria>

## Research Findings (2026-03-03)

- Root cause: real-binary prerequisite helpers return `Option<PathBuf>` and callers treat `None` as test skip (`return Ok(())`).
- Primary source of optional behavior:
  - `src/test_harness/binaries.rs`
  - `require_etcd_bin_for_real_tests() -> Result<Option<PathBuf>, HarnessError>`
  - `require_pg16_bin_for_real_tests() -> Result<Option<PathBuf>, HarnessError>`
  - `require_pg16_process_binaries_for_real_tests() -> Result<Option<BinaryPaths>, HarnessError>`
  - env-driven skip policy via `PGTUSKMASTER_REQUIRE_REAL_BINARIES`
- Confirmed real-test skip call sites:
  - `src/test_harness/pg16.rs`
  - `src/test_harness/etcd3.rs`
  - `src/pginfo/worker.rs`
  - `src/process/worker.rs`
  - `src/dcs/etcd_store.rs` (`RealEtcdFixture::spawn` returns `Option<Self>`)
  - `src/ha/e2e_multi_node.rs`
- Documentation currently states optional default behavior:
  - `RUST_SYSTEM_HARNESS_PLAN.md` section `Real-Binary Test Prerequisites`
- `Makefile` currently uses env-enforced mode only for `test-long`/`test`, meaning default flows can still pass without executing real binaries.

## Full Implementation Plan (for execution when promoted)

### 1. Replace optional real-binary resolution with fail-fast resolution
- [x] Refactor `src/test_harness/binaries.rs` to remove skip-by-default behavior from real-test helpers.
- [x] Change signatures to non-optional returns:
  - `require_etcd_bin_for_real_tests() -> Result<PathBuf, HarnessError>`
  - `require_pg16_bin_for_real_tests(name: &str) -> Result<PathBuf, HarnessError>`
  - `require_pg16_process_binaries_for_real_tests() -> Result<BinaryPaths, HarnessError>`
- [x] Remove `require_or_skip_binary` and all `Option` plumbing used only for skipping.
- [x] Keep clear prerequisite error messages that include missing binary path and install guidance.
- [x] Update unit tests in `src/test_harness/binaries.rs`:
  - remove/replace tests that validate skip behavior
  - add/retain tests that validate deterministic errors for missing binaries
  - keep no unwrap/panic/expect usage.

### 2. Remove all `None => return Ok(())` in real-binary tests
- [x] `src/test_harness/pg16.rs`: require postgres/initdb directly; fail test immediately on missing binaries.
- [x] `src/test_harness/etcd3.rs`: require etcd directly; fail test immediately on missing binary.
- [x] `src/pginfo/worker.rs`: require postgres/initdb/pg_basebackup directly; no early-return skip.
- [x] `src/process/worker.rs`:
  - update `pg16_binaries()` to return `Result<BinaryPaths, WorkerError>`
  - remove all real-test early returns based on `Option`
  - preserve existing runtime assertions and cleanup behavior.
- [x] `src/dcs/etcd_store.rs`:
  - change `RealEtcdFixture::spawn` to return `Result<Self, HarnessError>`
  - update real-etcd tests to always execute or fail fast.
- [x] `src/ha/e2e_multi_node.rs`:
  - change `resolve_pg_binaries_for_real_tests` to return `Result<BinaryPaths, WorkerError>`
  - change `resolve_etcd_bin_for_real_tests` to return `Result<PathBuf, WorkerError>`
  - remove test-level early skip branches in both e2e tests.

### 3. Align build/test entry points with non-optional real-test policy
- [x] Update `Makefile` so no target relies on skip-by-default semantics.
- [x] Keep `make test` as a focused real-suite entry point, but remove unnecessary env toggles if helpers are always fail-fast.
- [x] Expand `make test` to include all real-binary suites that were previously skippable, including:
  - `dcs::etcd_store::tests::` real-etcd tests
  - both HA e2e real scenarios (`e2e_multi_node_unassisted_failover_sql_consistency` and `e2e_multi_node_real_ha_scenario_matrix`)
- [x] Re-evaluate `make test` export of `PGTUSKMASTER_REQUIRE_REAL_BINARIES=1`; remove if obsolete after helper refactor.

### 4. Update docs to reflect enforced behavior
- [x] Update `RUST_SYSTEM_HARNESS_PLAN.md`:
  - remove wording that real tests are optional in default flow
  - document that missing `.tools/postgres16/bin/*` and `.tools/etcd/bin/etcd` is now a hard failure
  - keep prerequisite installation guidance explicit.
- [x] Remove stale mention of env-gated enforcement (`PGTUSKMASTER_REQUIRE_REAL_BINARIES`) after helper refactor so docs and code policy match.

### 5. Verification plan and evidence capture
- [x] Before full gates, run focused compile/tests for touched modules to catch signature fallout quickly:
  - `cargo test --all-targets test_harness::binaries`
  - `cargo test --all-targets test_harness::pg16::tests::spawn_pg16_requires_binaries_and_spawns`
  - `cargo test --all-targets test_harness::etcd3::tests::spawn_etcd3_requires_binary_and_spawns`
  - `cargo test --all-targets pginfo::worker::tests::step_once_transitions_unreachable_to_primary_and_tracks_wal_and_slots`
  - `cargo test --all-targets process::worker::tests::real_`
  - `cargo test --all-targets dcs::etcd_store::tests::`
  - `cargo test --all-targets ha::e2e_multi_node::e2e_multi_node_real_ha_scenario_matrix`
- [x] Run required gates and capture outputs under a new evidence directory:
  - `make check`
  - `make test`
  - `make test-long`
  - `make lint`
- [x] For acceptance criteria that mention grep pass/fail phrases, save command outputs and grep artifacts explicitly.
- [x] If linker/object flake appears on this mount, use known mitigation:
  - `cargo clean`
  - `CARGO_BUILD_JOBS=1`
  - `CARGO_INCREMENTAL=0`

### 6. Completion protocol (after execution succeeds)
- [x] Tick acceptance checkboxes.
- [x] Update task header tags to done/passing values only after all required gates pass.
- [x] Run `/bin/bash .ralph/task_switch.sh`.
- [x] Commit all changed files, including `.ralph` updates, with message:
  - `task finished bug-real-binary-tests-are-optional-via-early-return: <summary/evidence/challenges>`
- [x] Append durable new learnings to `AGENTS.md` if any emerge.

