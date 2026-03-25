## Plan: Collapse Write-Convergence Session And Query Helper Stack

### Why this is the next verified reduction target

The previous write-convergence slice removed duplicate conninfo and PEM parsing ownership, but `tests/ha/support/invariants/write_convergence.rs` still carries a second private helper pile that fragments one workflow into artificial layers instead of keeping connection and query behavior in one owner.

This slice should stay inside `tests/ha/support/invariants/write_convergence.rs` and delete helper layers directly. It should not reopen the conninfo/TLS boundary that plan 79 already flattened.

### Verified overlap in the current code

1. The connection lifecycle is duplicated across three call sites.
   - `probe_routing_target_connectivity(...)` opens a connection, ignores the client, and aborts the connection task.
   - `attempt_authoritative_write(...)` opens a connection, runs one authoritative write flow, and aborts the connection task.
   - `read_count_via_fresh_connection_target(...)` opens a connection, runs one read, and aborts the connection task.
   - All three depend on the same `connect_member(...)` owner, which already probes `SELECT 1` before returning.

2. `connect_and_probe_member(...)` is a private second layer with only one real owner.
   - It is only called by `connect_member(...)`.
   - It exists only to hide timeout/probe mechanics from the actual connection owner in the same file.

3. Two single-caller helpers still fragment one authoritative-write workflow.
   - `apply_fixture_row_setup(...)` is only called by `perform_authoritative_write(...)`.
   - `read_member_count_via_fresh_connection(...)` is only called by `wait_for_convergence(...)`.

4. The timed query error handling is repeated with only the label and query future changed.
   - `perform_authoritative_write(...)` wraps `apply_fixture_row_setup(...)` in `tokio::time::timeout(...)` and maps timeout / driver errors.
   - `increment_fixture_row(...)` repeats the same timeout / error mapping shape.
   - `read_count(...)` repeats the same timeout / error mapping shape before its row-missing and signed-to-unsigned validation.

5. `convergence_failure(...)` is only a thin wrapper around `convergence_failure_message(...)`.
   - It is used only by the test module.
   - If removing it is still net-negative after the main refactor, delete it and construct the test error inline.

### Execution plan

1. Keep the public and cross-file surface unchanged for this slice.
   - Stay inside `tests/ha/support/invariants/write_convergence.rs`.
   - Do not move more behavior into runtime owners.
   - Do not add new enums, structs, or traits for this slice.

2. Collapse the connected-session lifecycle into one owner.
   - Replace the current `connect_member(...)` plus callsite-specific `connection_task.abort()` patterns with one shared local helper that:
     - opens the connection with the existing TLS / non-TLS split
     - probes `SELECT 1`
     - runs one caller-supplied operation against the live client
     - always aborts the spawned connection task before returning
   - Delete `connect_and_probe_member(...)`.
   - Rewrite these call sites to use the shared lifecycle helper directly:
     - `probe_routing_target_connectivity(...)`
     - `attempt_authoritative_write(...)`
     - `read_count_via_fresh_connection_target(...)`

3. Inline the single-caller write and convergence wrappers.
   - Inline `apply_fixture_row_setup(...)` into `perform_authoritative_write(...)`.
   - Inline `read_member_count_via_fresh_connection(...)` into `wait_for_convergence(...)` unless the resulting direct closure is clearly larger than one remaining helper.
   - If one helper remains for the convergence observation path, it must serve at least two real callers after the inline.

4. Collapse the repeated timed-query error mapping.
   - Introduce at most one small local helper for the shared `tokio::time::timeout(...)` plus timeout/driver error translation shape.
   - Use it from the authoritative-write setup path, `increment_fixture_row(...)`, and `read_count(...)`.
   - Keep the operation labels concrete in the final error messages; do not hide which query failed.

5. Remove any leftover wrapper that is only test glue after the main collapse.
   - Re-check whether `convergence_failure(...)` still earns its existence after the production-side helper deletion.
   - If it is still only a thin test-only alias, delete it and update the affected test.

6. Validate the slice.
   - `cargo fmt`
   - `bash .ralph/git_current_lines.sh` and verify total drops below `34168`
   - `bash .ralph/git_diff_lines_since.sh` and verify the diff improves from the current baseline
   - `make check`
   - `make lint`
   - `make test`
   - `make test-long`

### Guardrails

- No new structs, enums, session guards, or callback-heavy abstraction layers.
- Keep TLS behavior and connection error wording materially the same.
- If the shared connection helper becomes more abstract than the duplicated call sites it replaces, fall back to inlining at the call sites instead of adding another layer.
- If line count is not lower after formatting, switch this plan back to `TO BE VERIFIED` and stop.

### Expected yield

- Delete `connect_and_probe_member(...)`.
- Delete `apply_fixture_row_setup(...)`.
- Delete `read_member_count_via_fresh_connection(...)`, or keep only one remaining helper if it has at least two real callers.
- Delete repeated connection-task abort scaffolding from the connection/read/write call sites.
- Delete one more layer of timed-query error boilerplate without widening any cross-file API.

NOW EXECUTE
