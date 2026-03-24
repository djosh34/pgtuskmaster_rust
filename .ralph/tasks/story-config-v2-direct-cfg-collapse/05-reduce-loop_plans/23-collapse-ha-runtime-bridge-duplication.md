## Plan: Collapse HA Runtime Bridge Duplication

### Why this reduction target

The next confirmed overlap is not another write-loop helper inside one file. It is the repeated sync-to-async runtime bridge spread across the HA harness:

- `tests/ha/support/invariants/write_convergence.rs` repeats the same `Handle::try_current` / `block_in_place` / `thread::spawn` / `Builder::new_current_thread()` bridge in both `ensure_healthy(...)` and `probe_routing_target_connectivity(...)`.
- `tests/ha/support/invariants/primary_count.rs` repeats the same bridge in both `ensure_healthy(...)` and `ensure_running(...)`.
- `tests/ha/support/world/mod.rs` already has the same bridge again inside `block_on_harness_future(...)`.

That is one harness boundary living in three places with mostly string differences around error text.

### Current overlap already verified

- `tests/ha/support/world/mod.rs` owns a working `block_on_harness_future(...)` helper for `HarnessError`.
- `tests/ha/support/invariants/primary_count.rs` duplicates that logic inline twice instead of calling a shared helper.
- `tests/ha/support/invariants/write_convergence.rs` duplicates the same logic inline twice with `WriteConvergenceInvariantError::Failed(...)`.
- The duplicated code is larger than the call-site-specific futures, so the wrong boundary is the runtime bridge itself, not each invariant method.

### Execution plan

1. Extract one shared HA harness runtime bridge helper.
   - Put it under `tests/ha/support/` so both invariants and `world` can use it.
   - Keep the helper responsible only for:
     - reusing the current multithread runtime via `block_in_place`, or
     - spawning a current-thread runtime on a plain thread when no suitable runtime is active, and
     - adapting runtime-build / thread-panic failures into the caller's error type.
   - Use existing error types; do not create a new enum or struct.

2. Rebuild the `HarnessError` call sites on the shared helper.
   - Replace `tests/ha/support/world/mod.rs:block_on_harness_future(...)` with either:
     - a direct call to the shared helper, or
     - a very thin wrapper if keeping the local function produces fewer lines.
   - Replace both runtime bridge sites in `tests/ha/support/invariants/primary_count.rs` with the shared helper.

3. Rebuild the write-convergence call sites on the same helper.
   - Replace the runtime bridge in `WriteConvergenceInvariantRunner::ensure_healthy(...)`.
   - Replace the runtime bridge in `probe_routing_target_connectivity(...)`.
   - Keep all current error wording stable by continuing to feed the existing context strings through `WriteConvergenceInvariantError::Failed(...)`.

4. Validate reduction and repo health.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the diff improves beyond the current `+4984 -7411 diff: -2427` baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.
   - Run the validation gates sequentially, not in parallel.

### Guardrails

- Reuse the existing `block_on_harness_future` shape instead of inventing a second abstraction family.
- Do not introduce a helper that only serves one file; this helper must replace the already-verified cross-file overlap.
- Do not create a generic helper so abstract that the adapter closures become larger than the duplicated runtime bridge.
- If `write_convergence.rs` needs more than a small error-adapter closure or function pointer to use the shared helper, switch this plan back to `TO BE VERIFIED`.
- Do not change the invariant semantics, only the runtime-entry boundary.

### Failed attempt

- Tried a shared `tests/ha/support/runtime.rs` bridge for the planned call sites.
- After `cargo fmt`, `bash .ralph/git_diff_lines_since.sh` worsened from `-2427` to `-2408`, so the code change was reverted.
- Any follow-up needs a different shape that removes more runtime-entry copies than the new shared boundary adds.

TO BE VERIFIED
