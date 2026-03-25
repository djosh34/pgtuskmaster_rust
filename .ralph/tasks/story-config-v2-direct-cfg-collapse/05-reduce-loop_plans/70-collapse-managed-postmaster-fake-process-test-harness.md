## Plan: Collapse Managed Postmaster Fake Process Test Harness

### Why this is the next reduction target

The managed-postmaster test harness is split across three modules even though all three are exercising the same process boundary.

- `src/process/postmaster.rs` already owns the real runtime API for managed-postmaster lookup, signaling, and start preflight.
- `src/api/worker.rs` duplicates a private fake-postgres child wrapper, script writer, pid-file writer, signal-log waiter, and lookup readiness loop just to drive `reload_managed_postmaster`.
- `src/process/worker.rs` carries a third copy of the same child-wrapper and readiness behavior for the start-noop path.

That is the wrong ownership boundary: the fake managed-postmaster process exists only to exercise `process::postmaster`, so the process layer should own one test harness and its callers should reuse it instead of each writing their own shell/python fixture.

### Current overlap already verified

- `src/api/worker.rs:450-473` and `src/process/postmaster.rs:488-511` define the same `ChildGuard` drop wrapper with the same missing-child error strings.
- `src/api/worker.rs:554-665` and `src/process/postmaster.rs:513-673` both own fake-postgres spawn logic, ready-file polling, postmaster-pid writing, and signal-log polling.
- `src/process/worker.rs:833-918` defines a third local `ChildGuard`, another fake-postgres launcher, and another `lookup_managed_postmaster` readiness loop.
- `src/api/worker.rs:667-681`, `src/process/postmaster.rs:614-627`, and `src/process/worker.rs:904-918` all poll `lookup_managed_postmaster(...)` until the fake process becomes observable.
- The runtime behavior under test still routes through the same owner APIs in `src/process/postmaster.rs:145-205`.

### Execution plan

1. Move the fake managed-postmaster process harness to the owning process boundary.
   - Add one `#[cfg(test)]` shared support module under `src/process/` for the fake managed-postmaster process.
   - Keep it test-only; do not widen runtime/public API surface.
   - The shared harness should own child cleanup, pid access, pid-file writing, lookup readiness waiting, and optional signal-log waiting.

2. Collapse the API worker tests onto the shared harness.
   - Delete the local `ChildGuard`, fake-postgres spawn helper, pid writer, and waiter functions from `src/api/worker.rs`.
   - Reuse the process-owned harness for the HTTPS reload tests instead of keeping a second shell-script implementation there.

3. Collapse the process worker tests onto the same harness.
   - Delete the local fake child wrapper and readiness loop from `src/process/worker.rs`.
   - Reuse the same fake managed-postmaster harness for the start-noop passfile test rather than keeping a separate `/bin/sleep` launcher path.

4. Collapse the postmaster tests onto the shared harness too.
   - Remove the private `ChildGuard`, fake spawn helper, duplicated ready/signal waiters, and duplicated pid writer from `src/process/postmaster.rs`.
   - Keep only the helpers that are uniquely about postmaster preflight behavior there, such as socket-lock setup, unless execution proves they also belong in shared support.

5. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and require the diff to remain net-negative.
   - Run `make check`.
   - Run `make lint`.
   - Run `make test`.
   - Run `make test-long`.

### Guardrails

- Do not move this harness into `dev_support`; it exists specifically to exercise `process::postmaster` behavior, so the process package should own it.
- Do not keep multiple fake-process flavors unless a test proves the shared harness cannot preserve behavior; if that happens, switch back to `TO BE VERIFIED`.
- Do not add compatibility shims or new wrapper structs around existing postmaster runtime types beyond what is needed for the shared test harness.

### Expected yield

- Delete three hand-rolled fake-postgres launcher stacks and replace them with one process-owned test helper.
- Remove duplicate polling loops and duplicate pid-file helper code across the API and process tests.
- Reduce lines while tightening the ownership boundary around the module the tests are actually exercising.

NOW EXECUTE
