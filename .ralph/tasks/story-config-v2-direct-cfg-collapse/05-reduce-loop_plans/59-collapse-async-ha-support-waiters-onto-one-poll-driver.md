## Plan: Collapse Async HA Support Waiters Onto One Poll Driver

### Why this is the next reduction target

Plan 56 proved that a broad "shared polling driver for everything" direction is mechanically viable, but it missed the real ownership boundary. `steps` has caller-specific terminal-container policy tied to `HaWorld`, while the rest of HA support still carries a separate family of plain async wait loops with the same retry mechanics:

- `tests/ha/support/world/mod.rs:677` `wait_for_service_health(...)`
- `tests/ha/support/world/mod.rs:707` `wait_for_seed_primary(...)`
- `tests/ha/support/invariants/primary_count.rs:146` `PrimaryCountInvariantRunner::ensure_healthy(...)`
- `tests/ha/support/invariants/write_convergence.rs:636` `wait_for_convergence(...)`
- `tests/ha/support/invariants/write_convergence.rs:1610` `wait_for_row_count_at_least(...)`
- `tests/ha/support/invariants/write_convergence.rs:1631` `wait_for_postgres_ready(...)`

Those call sites all own the same scaffolding:

- `let deadline = Instant::now() + ...`
- `let mut last_error = None`
- retry until timeout
- `tokio::time::sleep(...)`
- timeout text based on the last observed failure

The boundary problem is that the async polling algorithm lives in multiple owners even though the domain-specific checks already fit into small caller closures. This is a better target than another `steps` refactor because it deletes duplicated mechanics from `world` and both invariant modules without reintroducing the bulky step adapter that sank plan 56's net-line result.

### Current overlap already verified

- `tests/ha/support/world/mod.rs:678` and `tests/ha/support/world/mod.rs:708` duplicate startup-deadline wait loops with identical deadline/last-error/sleep structure.
- `tests/ha/support/invariants/primary_count.rs:153` duplicates the same timeout/poll loop inside `block_on_support_future(...)`, differing only in the readiness check and timeout wording.
- `tests/ha/support/invariants/write_convergence.rs:644`, `tests/ha/support/invariants/write_convergence.rs:1615`, and `tests/ha/support/invariants/write_convergence.rs:1635` duplicate the same async retry loop shape for convergence, row-count, and postgres-readiness waits.
- `tests/ha/support/steps/mod.rs:676` should stay as-is for this slice because its terminal-container-failure policy is not generic retry machinery.

### Execution plan

1. Add one shared async polling helper under `tests/ha/support`.
   - Prefer `tests/ha/support/mod.rs` unless a tiny sibling module is smaller.
   - The helper should own only:
     - deadline calculation,
     - `last_error` string capture,
     - `tokio::time::sleep(...)`,
     - timeout fallback formatting.
   - Keep the callback boundary narrow:
     - caller supplies poll interval and deadline window,
     - caller supplies an async retry body,
     - caller supplies timeout-message formatting.
   - Do not introduce a new enum, DTO, or boxed trait object just to model polling state.

2. Collapse the two `world` bootstrap waiters onto the shared helper.
   - Rebuild `wait_for_service_health(...)` on the helper.
   - Rebuild `wait_for_seed_primary(...)` on the helper.
   - Keep status snapshot recording, service-name text, and current timeout messages intact.

3. Collapse `PrimaryCountInvariantRunner::ensure_healthy(...)` onto the same helper.
   - Keep `ensure_task_running_state(...)` where it is.
   - Let the closure synthesize the current "not healthy yet" observation text so the timeout still reports the last observed count.
   - Do not touch the background runner loop in `start_with_observe_all(...)`; that is separate ownership.

4. Collapse the write-convergence waiters onto the same helper.
   - Rebuild `wait_for_convergence(...)` on the helper.
   - Rebuild test-only helpers `wait_for_row_count_at_least(...)` and `wait_for_postgres_ready(...)` on the helper.
   - Keep the current timeout wording and observation logic at the call sites.
   - Do not touch the authoritative worker loops in this slice.

5. Leave `steps::poll_until(...)` alone unless the async collapse is already clearly net-negative and deleting it is obviously smaller.
   - No `HaWorld` adapter layer in this plan.
   - No terminal-container-failure hook in the shared helper.
   - If `steps` starts pressuring the helper signature, stop and switch back to `TO BE VERIFIED`.

6. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and require the diff to stay net-negative.
   - Run `make check`.
   - Run `make lint`.
   - Run `make test`.
   - Run `make test-long`.

### Guardrails

- The shared helper must remain an async retry primitive, not a new polling taxonomy.
- Keep caller-specific messages, status recording, and domain decisions at the call sites.
- Avoid boxed futures and state-courier types; if the helper needs them, the boundary is wrong.
- Reuse existing error types and string formatting instead of creating a new wrapper error.
- If the helper shrinks `world` and `write_convergence` but makes `primary_count` larger, leave `primary_count` out and keep the slice net-negative.

### Expected yield

- Delete two open-coded bootstrap waiters from `world`.
- Delete one blocking health wait loop from `primary_count` if it stays smaller.
- Delete three async waiters from `write_convergence`.
- Keep `steps` isolated so the shared helper stays small enough to actually reduce lines this time.

NOW EXECUTE
