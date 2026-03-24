## Plan: Collapse HA Runtime Bridge Onto Support Root

### Why this reduction target

The HA harness still carries the same sync-to-async runtime-entry boundary in three places:

- `tests/ha/support/world/mod.rs::block_on_harness_future(...)`
- `tests/ha/support/invariants/primary_count.rs::ensure_healthy(...)`
- `tests/ha/support/invariants/primary_count.rs::ensure_running(...)`
- `tests/ha/support/invariants/write_convergence.rs::ensure_healthy(...)`
- `tests/ha/support/invariants/write_convergence.rs::probe_routing_target_connectivity(...)`

All five sites repeat the same `Handle::try_current()` / `RuntimeFlavor::MultiThread` / `block_in_place(...)` / fallback `thread::spawn(...)` / `Builder::new_current_thread()` sequence. The only real variation is the error string each caller wants after runtime-build or thread-panic failures.

The earlier `runtime.rs` attempt proved the wrong shape: adding a brand-new module plus wider adapters cost more lines than it removed. The remaining opportunity is to collapse the duplicated bridge onto the existing `tests/ha/support/mod.rs` root instead of creating another helper layer.

### Current overlap already verified

- `tests/ha/support/world/mod.rs` already owns a working `block_on_harness_future(...)` bridge for `HarnessError`.
- `tests/ha/support/invariants/primary_count.rs` duplicates that bridge twice inline with only primary-count-specific error messages changed.
- `tests/ha/support/invariants/write_convergence.rs` duplicates the same bridge twice inline, then maps the returned `String` into `WriteConvergenceInvariantError::Failed(...)`.
- `tests/ha/support/mod.rs` is already the shared HA harness root, so it can own one small bridge helper without adding another support submodule.

### Execution plan

1. Move the shared runtime-entry mechanics into the existing HA support root.
   - Add one small helper in `tests/ha/support/mod.rs`.
   - Have it return `Result<T, String>` and own only:
     - `Handle::try_current()`
     - the multithread `block_in_place(...)` fast path
     - the fallback spawned current-thread runtime path
     - runtime-build failure rendering
     - thread-panic failure rendering
   - Pass the two caller-specific message stems as `&'static str` inputs instead of introducing a new enum or error adapter type.

2. Rebuild the existing `HarnessError` bridge on top of that helper.
   - Replace `world/mod.rs::block_on_harness_future(...)` with either:
     - a thin wrapper around the new root helper, or
     - direct call sites if that is smaller after formatting.
   - Keep `wait_for_write_convergence_attachment(...)` behavior unchanged.

3. Collapse the invariant call sites onto the same shared helper.
   - Replace both inline bridges in `primary_count.rs`.
   - Replace the inline bridges in `write_convergence.rs::ensure_healthy(...)` and `probe_routing_target_connectivity(...)`.
   - Keep existing error wording stable by preserving the current per-call runtime-build and panic strings.

4. Validate reduction and repo health.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the diff improves beyond the current `+6955 -9578 diff: -2623` baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.
   - Run the validation gates sequentially, not in parallel.

### Guardrails

- Do not create a new `runtime.rs` helper module for this slice.
- Do not introduce a new shared enum/struct for runtime-entry failures; string-returning reuse is enough here.
- Keep the helper scoped to runtime-entry mechanics only. Worker-thread loops elsewhere in `write_convergence.rs` are out of scope.
- If the shared root helper forces larger adapter closures or wrapper functions than the deleted bridge blocks, switch this plan back to `TO BE VERIFIED`.
- Keep all current error strings and invariant semantics unchanged.

NOW EXECUTE
