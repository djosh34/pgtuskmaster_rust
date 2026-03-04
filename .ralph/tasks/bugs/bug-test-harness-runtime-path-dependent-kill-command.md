---
## Bug: Test harness runtime kill command is PATH-dependent and bypasses provenance guarantees <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
Real-binary harness paths are intended to be explicit and provenance-controlled, but runtime teardown logic still invokes `kill` by bare name via `Command::new("kill")`.

This creates a PATH-dependent execution path in real-binary e2e runs:
- `src/test_harness/pg16.rs` uses `Command::new("kill")` during postgres child shutdown.
- `src/test_harness/etcd3.rs` uses `Command::new("kill")` during etcd member shutdown.
- `src/test_harness/ha_e2e/util.rs` uses `Command::new("kill")` for postmaster fallback signal and liveness probes.

If PATH is accidentally or maliciously modified in CI/dev shells, teardown and fallback signaling may execute an unexpected binary, violating provenance assumptions and potentially producing false diagnostics.

Please explore and research the codebase first, then implement a fail-closed fix:
- remove PATH-dependent `kill` invocations (prefer direct signal APIs or absolute trusted path resolution),
- add a deterministic regression test/negative control proving PATH-prepended fake `kill` is not executed.
</description>

<acceptance_criteria>
- [x] Teardown and fallback signaling paths in the test harness no longer execute PATH-resolved `kill` binaries.
- [x] Add a deterministic test/fixture that prepends a fake `kill` in PATH and proves harness runtime paths do not execute it.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
</acceptance_criteria>

## Plan

### 0) Confirm scope + invariants (research)
- [x] Confirm all current PATH-dependent `kill` invocations:
  - `src/test_harness/pg16.rs` (`PgHandle::shutdown`)
  - `src/test_harness/etcd3.rs` (`EtcdHandle::shutdown`)
  - `src/test_harness/ha_e2e/util.rs` (`kill_best_effort`, `pid_is_alive`)
- [x] Confirm there are no other `Command::new("kill")` call sites (ripgrep).
- [x] Confirm existing dependencies: `libc` already exists under `cfg(unix)` in `Cargo.toml` (prefer using it; avoid adding new deps).

### 1) Add a syscall-based signaling helper (no PATH)
- [x] Add a new module `src/test_harness/signals.rs` (exported from `src/test_harness/mod.rs`) with *syscall-only* helpers (no shelling out, no PATH):
  - [x] `send_signal(pid: u32, signal: libc::c_int) -> Result<(), std::io::Error>`
    - Convert `u32 -> libc::pid_t` with `try_from`; return `InvalidInput` if out of range.
    - Call `unsafe { libc::kill(pid_t, signal) }` and map errors:
      - `ESRCH` => success (“already exited”)
      - any other errno (including `EPERM`) => error (unexpected for our own children)
  - [x] `pid_exists(pid: u32) -> Result<bool, std::io::Error>`
    - Use `kill(pid, 0)` and interpret conservatively (fail-closed):
      - `Ok` => `true`
      - `ESRCH` => `false`
      - `EPERM` => `true` (process exists but we cannot signal; do *not* treat as dead)
      - other errno => error
  - [x] Provide `#[cfg(not(unix))]` stubs that return a clear error (crate must still compile on non-unix even if runtime isn’t supported).

### 2) Remove PATH-dependent `kill` from harness shutdown paths
- [x] `src/test_harness/pg16.rs`:
  - [x] Replace `Command::new("kill").arg("-TERM")...` with `signals::send_signal(pid, libc::SIGTERM)` (map to `HarnessError`).
  - [x] Keep staged shutdown semantics, but make the *force-kill wait bounded* (avoid unbounded `child.wait().await` after `start_kill()`).
- [x] `src/test_harness/etcd3.rs`:
  - [x] Same replacement: remove `Command::new("kill")`, use `signals::send_signal`.
  - [x] Ensure teardown is fully bounded (TERM wait; then SIGKILL/start_kill; then bounded wait; error if still stuck).
- [x] `src/test_harness/ha_e2e/util.rs`:
  - [x] Replace `kill_best_effort` and `pid_is_alive` implementations with syscall-based helpers (no async timeouts needed around syscalls).
  - [x] Remove `unwrap_or(false)` in correctness-critical checks by making probe errors explicit (syscalls shouldn’t time out; remaining errors should surface).
  - [x] Ensure probe semantics treat `EPERM` as “alive” (conservative) rather than “dead”.

### 3) Deterministic regression test: fake `kill` on PATH is never executed
- [x] Add a two-stage (parent/child) regression test to avoid mutating PATH in the main test runner process:
  - [x] Parent test:
    - [x] Create a temp namespace directory (reuse existing `test_harness::namespace` helpers; no new deps).
    - [x] Write an executable `kill` script into it that touches a marker file and exits `0` (unix `chmod 0o755`).
    - [x] Spawn the current test binary (`std::env::current_exe()`), configuring:
      - [x] `PATH=<fake_dir>:<original_path>`
      - [x] `FAKE_KILL_MARKER=<marker_path>`
      - [x] libtest args to run ONLY the inner test: `--exact <inner_test_name> --test-threads 1`
    - [x] Assert child exits successfully.
    - [x] Assert marker file was NOT created (proves no PATH-resolved `kill` was invoked).
  - [x] Inner test:
    - [x] Spawn external long-running processes via an *absolute* binary path (pick first existing of `/bin/sleep` or `/usr/bin/sleep`) so spawn is not PATH-dependent.
    - [x] Exercise all three formerly-PATH-dependent paths in one place:
      - [x] Add `#[cfg(test)]` constructors on `PgHandle` and `EtcdHandle` (e.g. `PgHandle::new_for_test(child: Child)`) so a single test can call `.shutdown()` without requiring real postgres/etcd binaries.
      - [x] Call `PgHandle::shutdown()` and `EtcdHandle::shutdown()` against `sleep` children.
      - [x] Call `ha_e2e::util::force_kill_postmaster_pid()` + `pid_is_alive()` against a `sleep` PID.
    - [x] Ensure all children terminate and return `Ok(())`.

### 4) Verification (must be 100% green)
- [x] Re-run ripgrep to confirm there is no longer any `Command::new("kill")` in `src/`.
- [x] Run required gates (no skipping):
  - [x] `make check`
  - [x] `make test`
  - [x] `make test-long`
  - [x] `make lint`

### 5) Task closure (only after all gates pass)
- [x] Tick off acceptance criteria checkboxes.
- [x] Update header tags to `<status>done</status> <passes>true</passes>` and add `<passing>true</passing>` in this task file.
- [x] Run `/bin/bash .ralph/task_switch.sh`
- [x] Commit with message `task finished bug-test-harness-runtime-path-dependent-kill-command: ...` (include evidence: the four `make` commands passed + any challenges).
- [x] `git push`

NOW EXECUTE
