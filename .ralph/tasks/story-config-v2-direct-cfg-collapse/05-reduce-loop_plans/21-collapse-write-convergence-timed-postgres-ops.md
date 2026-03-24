## Plan: Collapse Write Convergence Timed Postgres Ops

### Why this reduction target

The next wrong-place overlap is still local to `tests/ha/support/invariants/write_convergence.rs`:

- `perform_authoritative_write`, `increment_fixture_row`, and `read_count` each wrap a Postgres operation with the same timeout/error-adaptation mechanics.
- In all three places, the real variation is only the operation label and the inner future being run.
- That leaves repeated transport/error-wrapper mechanics living above the actual fixture-row domain operations.

### Current overlap already verified

- `perform_authoritative_write` wraps `apply_fixture_row_setup(client)` with `tokio::time::timeout(...)`, then maps timeout and execution failures into `WriteConvergenceInvariantError`.
- `increment_fixture_row` does the same timeout/error wrapping around `client.query_one(...)`.
- `read_count` does the same timeout/error wrapping around `client.query_opt(...)`.
- The negative-count and missing-row checks are distinct domain logic and should stay in their current functions.

### Execution plan

1. Extract one local helper for timed Postgres operations.
   - Keep the helper inside `tests/ha/support/invariants/write_convergence.rs`.
   - Have it own only the shared `tokio::time::timeout(...)` plus `WriteConvergenceInvariantError::Failed(...)` mapping.

2. Rebuild the three callers on top of the shared helper.
   - Keep `perform_authoritative_write` responsible for fixture setup and accepted-count bookkeeping.
   - Keep `increment_fixture_row` responsible for row decoding and negative-count validation.
   - Keep `read_count` responsible for missing-row detection and row decoding.

3. Preserve behavior exactly while reducing code.
   - Do not change the wording of timeout or execution failure messages.
   - Do not move fixture-row semantics into the helper.
   - Do not broaden the helper beyond this file.

4. Validate reduction and repo health.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the diff improves beyond the current `+4984 -7411 diff: -2427` baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.
   - Run the validation gates sequentially, not in parallel.

### Guardrails

- Keep the refactor local to `write_convergence.rs`.
- Keep the helper scoped to timeout/error adaptation only.
- If this starts pulling row decoding or accepted-count policy into a generic abstraction, switch this plan back to `TO BE VERIFIED`.

### Attempt result

- A direct `run_timed_postgres_operation(...)` helper was tried locally in `write_convergence.rs`.
- After `cargo fmt`, the file diff was exactly `32 insertions / 32 deletions`, so the slice produced no file-level line reduction.
- The repo reduction metric regressed from `+4984 -7411 diff: -2427` to `+5093 -7443 diff: -2350`, so this exact helper shape is not worth executing.

TO BE VERIFIED
