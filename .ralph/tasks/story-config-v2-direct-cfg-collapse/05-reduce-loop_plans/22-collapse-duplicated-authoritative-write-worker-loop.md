## Plan: Collapse Duplicated Authoritative Write Worker Loop

### Why this reduction target

The next wrong-place overlap in `tests/ha/support/invariants/write_convergence.rs` is larger than the failed timed-query helper:

- `run_authoritative_worker` and the test-only `build_authoritative_write_worker` both own the same write-gate loop.
- In both places, the loop does the same stop check, `try_start_write`, result recording, second stop check, and `tokio::time::sleep(...)`.
- The real variation is only the single write attempt that runs while the permit is held.

That means the gate lifecycle currently lives in two workflows instead of one shared local boundary.

### Current overlap already verified

- `run_authoritative_worker` contains the full loop around `attempt_authoritative_write(...)`.
- `build_authoritative_write_worker` repeats the same loop around `connect_session(...)` plus `perform_authoritative_write(...)`.
- Both branches clear `last_error` on success, record the error string on failure, and stop immediately if `stop_requested` becomes true.
- The blocked test worker is different and should stay separate because it models drain behavior instead of repeated writes.

### Execution plan

1. Extract one local async helper for the shared write-gate loop.
   - Keep it inside `tests/ha/support/invariants/write_convergence.rs`.
   - Let it own only the repeated permit lifecycle, stop checks, error recording, and sleep.
   - Pass the per-iteration write attempt in as a closure/fn argument that returns `Result<(), String>`.

2. Rebuild production worker execution on the shared loop.
   - Keep `spawn_authoritative_worker` responsible for spawning the named thread and current-thread runtime.
   - Rebuild `run_authoritative_worker` so it delegates the repeated loop mechanics to the new helper.
   - Keep `attempt_authoritative_write(...)` responsible for authoritative routing, connection setup, and member-specific error context.

3. Rebuild the test authoritative worker on the same shared loop.
   - Keep `build_authoritative_write_worker` responsible for the test thread/runtime shell.
   - Keep the test closure responsible for `connect_session(...)`, `perform_authoritative_write(...)`, and dropping the session.
   - Preserve the current test error strings and stop behavior.

4. Validate reduction and repo health.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the diff improves beyond the current `+4984 -7411 diff: -2427` baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.
   - Run the validation gates sequentially, not in parallel.

### Guardrails

- Do not introduce a new enum or struct for this slice.
- Keep the shared helper local to `write_convergence.rs`; do not create a cross-file abstraction.
- Do not fold the blocked test worker into this helper.
- If the closure/generic shape starts adding more scaffolding than it deletes, switch this plan back to `TO BE VERIFIED`.
- Keep error wording and write-gate semantics unchanged.

### Failed attempt note

- Extracting a local `run_write_gate_loop` helper and rebuilding both worker loops on top of it did compile cleanly enough to format, but it made the repo larger instead of smaller.
- `bash .ralph/git_diff_lines_since.sh` regressed from the prior `+4984 -7411 diff: -2427` baseline to `+5091 -7440 diff: -2349` with that helper present.
- The closure/generic scaffolding cost more than the duplicate loop it removed, so this exact shape should not be retried.

TO BE VERIFIED
