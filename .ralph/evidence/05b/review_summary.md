# 05b Deep Review Summary (2026-03-02)

## What was verified

- All required gates pass sequentially in this environment:
  - `make check`
  - `make test`
  - `make test-bdd`
  - `make lint`
- Lint enforcement for runtime `unwrap` is proven by canary:
  - Introducing an `unwrap()` in `src/lib.rs` causes `make lint` to fail.
  - Reverting the canary restores `make lint` pass.
- Tasks marked done in `.ralph/tasks/story-rust-system-harness/` were reviewed for “truthiness” against the current tree (code + tests).

## Material findings

### 1) Real-binary tests are present but do not execute in this environment

- PostgreSQL 16 binaries (`/usr/lib/postgresql/16/bin/*`) and etcd (`/usr/bin/etcd`) are not present here.
- The “real-system” tests are written to early-return when required binaries are missing, which makes them *pass* without exercising real binaries.
- This is acceptable as a local-dev convenience, but it means “real-binary coverage” is not currently enforced by default.

### 2) Port reservation lifetime bug in real-binary tests (fixed in this review)

Several tests were holding `PortReservation` listeners through the spawn/start step, which would prevent the child process from binding the requested port when the binaries are actually installed.

This review moved `drop(reservation)` to the correct point (immediately before spawn/start) and refactored the replica test to allocate ports closer to use.

Files patched:
- `src/pginfo/worker.rs`
- `src/process/worker.rs`
- `src/test_harness/pg16.rs`
- `src/test_harness/etcd3.rs`

## Follow-up work recommended

1) Add an enforcement mode for “real-binary” tests (CI/dev):
   - Provide an env var (or a dedicated make target) that fails tests if required binaries are missing.
   - Ensure CI has a job that installs/provides PG16 + etcd3 and runs that mode.

2) Consider reducing TOCTOU port-allocation risk for process spawns:
   - Current approach is “reserve port by binding, then drop right before spawn”.
   - This can still race on highly contended systems; a retry-on-bind-failure strategy would improve robustness.

