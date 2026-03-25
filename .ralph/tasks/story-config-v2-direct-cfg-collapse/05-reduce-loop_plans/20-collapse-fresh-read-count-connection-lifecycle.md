## Plan: Collapse Fresh Read Count Connection Lifecycle

### Why this reduction target

The next wrong-place overlap is still local to `tests/ha/support/invariants/write_convergence.rs`:

- `read_member_count_via_fresh_connection` and `read_count_via_target` both do the same fresh connection lifecycle: call `connect_member`, run `read_count`, abort the spawned connection task, and then adapt the read failure into caller-specific error handling.
- The real variation is only the outer error mapping: one path returns `MemberCountObservation`, while the other wraps failures in `WriteConvergenceInvariantError`.
- That means the connect/read/abort sequence is duplicated domain mechanics living above two different reporting layers.

### Current overlap already verified

- Both paths open a fresh connection through `connect_member`.
- Both paths immediately run `read_count` with the caller-provided timeout.
- Both paths always abort the spawned `connection_task` after the read attempt.
- Neither path owns routing resolution or observation message composition; those concerns can stay where they are.

### Execution plan

1. Extract one local helper for the shared fresh-read lifecycle.
   - Keep the helper inside `tests/ha/support/invariants/write_convergence.rs`.
   - Have it perform `connect_member`, `read_count`, and `connection_task.abort()`, returning only the raw success or failure string needed by callers.

2. Rebuild the two callers on top of the shared helper.
   - Keep `read_member_count_via_fresh_connection` responsible for observer refresh and previous-error composition.
   - Keep `read_count_via_target` responsible for wrapping failures in `WriteConvergenceInvariantError`.

3. Preserve behavior exactly while reducing code.
   - Do not change the wording of existing connection or count-read failures.
   - Do not change routing-target resolution, observer fallback behavior, or convergence semantics.

4. Validate reduction and repo health.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the diff improves beyond the current `+4959 -7382 diff: -2423` baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.
   - Run the validation gates sequentially, not in parallel.

### Guardrails

- Keep the refactor local to `write_convergence.rs`.
- Keep the helper scoped to the fresh connection/read lifecycle only.
- If this starts pulling observer-specific messaging or routing policy into a generic abstraction, switch this plan back to `TO BE VERIFIED`.

NOW EXECUTE
