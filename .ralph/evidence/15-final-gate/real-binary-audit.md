# Real Binary Audit (Task 15)

Date (UTC): 2026-03-03

## Checks performed
- Reviewed binary resolution helper: `src/test_harness/binaries.rs`.
- Reviewed harness spawn tests requiring real binaries:
  - `src/test_harness/etcd3.rs::spawn_etcd3_requires_binary_and_spawns`
  - `src/test_harness/pg16.rs::spawn_pg16_requires_binaries_and_spawns`
- Searched for explicit skip/ignore patterns across `src/` and `tests/`.

## Findings
- Binary paths are required through `require_binary(...)`; missing binaries return `HarnessError::InvalidInput` with explicit message.
- Real harness tests call `require_etcd_bin()` / `require_pg16_bin(...)` and then run spawn/shutdown paths.
- No `#[ignore]` attributes found in `src/` or `tests/` scan for this repo state.
- No ad-hoc skip paths detected in audited source/test files.

## Result
- Tests that claim real PG16/etcd behavior resolve binaries through harness helpers and fail clearly when binaries are unavailable.
- No newly introduced optional-skip behavior found in current scan.
