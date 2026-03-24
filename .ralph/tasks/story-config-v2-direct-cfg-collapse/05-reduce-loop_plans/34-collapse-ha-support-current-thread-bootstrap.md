## Plan: Collapse HA Support Current-Thread Bootstrap

### Why this reduction target

The HA support layer still owns the same current-thread runtime bootstrap in too many places:

- `tests/ha/support/mod.rs::run_feature(...)`
- `tests/ha/support/mod.rs::block_on_support_future(...)` fallback branch
- `tests/ha/support/invariants/write_convergence.rs::spawn_authoritative_worker(...)`
- `tests/ha/support/invariants/write_convergence.rs::current_thread_cleanup_waits_for_detached_worker_shutdown(...)`
- `tests/ha/support/invariants/write_convergence.rs::build_authoritative_write_worker(...)`
- `tests/ha/support/invariants/write_convergence.rs::build_blocked_write_worker(...)`

Across those sites the same bootstrap mechanics are repeated:

- `Builder::new_current_thread()`
- `.enable_all()`
- `.build()`
- render a caller-specific runtime-build failure string
- `block_on(...)` the real async work

The actual differences are small and local:

- some call sites need a named spawned thread
- some call sites run inline
- each caller wants its own existing error wording

That makes runtime bootstrap itself the wrong boundary. It belongs in the HA support root, while the worker loops and test logic stay in `write_convergence.rs`.

### Current overlap already verified

- `tests/ha/support/mod.rs` already owns the shared HA harness runtime-entry helper (`block_on_support_future(...)`), so support root is the right owner for the remaining bootstrap mechanics too.
- `tests/ha/support/invariants/write_convergence.rs` still repeats four separate current-thread runtime builds after the last runtime-bridge reduction.
- `run_feature(...)` still builds the same current-thread runtime inline instead of reusing a support-root bootstrap helper.
- None of the remaining sites need a new domain type; they only differ in thread name, future body, and existing error strings.

### Execution plan

1. Add one small HA support bootstrap helper family in `tests/ha/support/mod.rs`.
   - Keep it in the existing support root; do not create a new module.
   - Reuse one helper for `Builder::new_current_thread().enable_all().build()`.
   - Add a thin spawned-thread helper only if it removes more lines than it adds.
   - Keep helpers string-based for runtime-build failures instead of introducing a new enum or wrapper type.

2. Rebuild the existing support-root entry points on the shared bootstrap.
   - Replace the inline runtime construction in `run_feature(...)`.
   - Rebuild the fallback branch of `block_on_support_future(...)` on the same bootstrap helper.
   - Preserve current error wording.

3. Collapse the remaining `write_convergence.rs` thread/runtime shells onto the same support-root bootstrap.
   - Replace the runtime bootstrap in `spawn_authoritative_worker(...)`.
   - Replace the runtime bootstrap in the detached cleanup test thread.
   - Replace the runtime bootstrap in `build_authoritative_write_worker(...)`.
   - Replace the runtime bootstrap in `build_blocked_write_worker(...)`.
   - Keep worker loop logic, thread names, and invariant semantics unchanged.

4. Validate reduction and repo health.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the diff improves beyond the current `+7025 -9701 diff: -2676` baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.
   - Run the validation gates sequentially, not in parallel.

### Guardrails

- Do not create another `runtime.rs` helper module for this slice.
- Do not introduce a new enum/struct just to label bootstrap failures.
- Keep the shared helper scoped to current-thread runtime bootstrap only. It must not absorb write-loop behavior or test-specific assertions.
- If a generic spawned-thread helper starts requiring more closure scaffolding than the deleted bootstrap blocks, switch this plan back to `TO BE VERIFIED`.
- Keep current error strings and thread names stable.

NOW EXECUTE
