## Plan: Collapse Write-Convergence Attachment State

### Why this reduction target

`tests/ha/support/world/mod.rs` still owns a large amount of lifecycle machinery that does not belong in the HA harness:

- `WriteConvergenceState`
- `with_write_convergence_runner(...)`
- `start_write_convergence_invariant(...)`
- `wait_for_write_convergence_attachment(...)`
- test-only delayed attachment helpers and state assertions

That state machine exists only to defer `WriteConvergenceInvariantRunner::start(...)` into a background task and then settle it later. The actual invariant behavior already lives inside `tests/ha/support/invariants/write_convergence.rs`, which is the right owner for worker startup, readiness, and shutdown semantics.

This means the boundary is backwards today:

- `HarnessShared` stores startup-task bookkeeping instead of a runner
- `world/mod.rs` knows about attachment timing, join errors, and pending states
- tests in `world/mod.rs` validate attachment-state transitions instead of harness-visible outcomes

If startup is made direct, `HarnessShared` can own the runner like it already owns `PrimaryCountInvariantRunner`, and the attachment state machine disappears entirely.

### Current overlap already verified

- `HarnessShared::initialize(...)` already awaits `PrimaryCountInvariantRunner::start(...)` directly.
- The only deferred lifecycle in `world/mod.rs` is write convergence.
- `WriteConvergenceInvariantRunner::start(...)` already returns the final runner type, so no new enum/struct is needed to represent attachment.
- Callers only ask for two harness-visible behaviors:
  - `ensure_accepted_writes_healthy(...)`
  - `ensure_accepted_writes_running(...)`

Neither caller needs access to a pending attachment state.

### Execution plan

1. Replace deferred attachment storage in `tests/ha/support/world/mod.rs` with direct runner ownership.
   - Remove `WriteConvergenceState`.
   - Store `WriteConvergenceInvariantRunner` directly in `HarnessShared`.
   - Drop the `Mutex` currently dedicated to attachment state.

2. Start write convergence directly during harness initialization.
   - After cluster bootstrap, await `WriteConvergenceInvariantRunner::start(...)`.
   - Keep cleanup-on-startup-failure behavior intact.
   - Preserve startup timeline notes, but record direct startup completion instead of background attachment wording.

3. Collapse the harness methods onto the direct runner.
   - Replace `with_write_convergence_runner(...)` with straight runner calls.
   - Remove `start_write_convergence_invariant(...)`.
   - Remove `wait_for_write_convergence_attachment(...)`.
   - Keep existing user-facing error strings where they still make sense.

4. Trim tests to match the simpler ownership model.
   - Remove tests that only exist to validate pending attachment-state transitions.
   - Keep or replace tests that still verify harness-visible error propagation and cleanup behavior under direct startup.

5. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the diff improves beyond the current `+7147 -9818 diff: -2671` baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long` sequentially.

### Guardrails

- Do not introduce a new wrapper enum/struct to replace `WriteConvergenceState`.
- Do not move write-convergence lifecycle logic out of `write_convergence.rs` and into another new helper module.
- If direct startup breaks an intended early-bootstrap behavior that callers actually rely on, switch this plan back to `TO BE VERIFIED`.
- Prefer deleting attachment-specific tests over rewriting them into equally indirect scaffolding.

NOW EXECUTE
