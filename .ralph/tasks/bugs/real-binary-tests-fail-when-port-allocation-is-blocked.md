## Bug: Real-binary tests fail when port allocation is blocked <status>done</status> <passes>true</passes>

<description>
`make test` fails in the current environment because multiple tests panic when `allocate_ports(...)` returns `io error: Operation not permitted (os error 1)`.

Detected on 2026-03-02 with:
- `make test` (failed/terminated after reporting multiple failures and a long-running test)
- `cargo test test_harness::ports::tests::allocate_ports_returns_unique_ports -- --nocapture`
- `cargo test test_harness::etcd3::tests::spawn_etcd3_requires_binary_and_spawns -- --nocapture`
- `cargo test pginfo::worker::tests::step_once_maps_replica_when_polling_standby -- --nocapture`
- `cargo test test_harness::ports::tests::concurrent_allocations_do_not_collide_while_reserved -- --nocapture`

Representative failures:
- `src/test_harness/ports.rs:76` and `src/test_harness/ports.rs:103` panic on `Operation not permitted (os error 1)`.
- `src/test_harness/etcd3.rs:214` fails with `allocate ports failed`.
- `src/pginfo/worker.rs:239` fails with `port allocation failed`.
- Process worker real-binary tests (`real_demote/promote/restart/start-stop/fencing`) fail in the same gate because they depend on port allocation.

Please explore and research the test harness and real-binary test code paths first, then implement a fix so tests fail deterministically for product regressions rather than environment socket-policy artifacts.
</description>

## Current observed behavior (2026-03-02)
- Initial repro on this run did **not** show `allocate_ports` `EPERM`; all listed targeted repro commands passed individually.
- Isolated full-suite run (`make test`) failed once with `pginfo::worker::tests::step_once_maps_replica_when_polling_standby` at primary connectivity (`connect to primary failed: db error`), which maps to failure mode `B` (startup/connect timing race) rather than the original `A` mode.
- Post-fix targeted and repeated runs were stable (5/5 passes on each flaky-prone target), and full required gates passed.
- Evidence logs are under `.ralph/evidence/real-binary-tests-fail-when-port-allocation-is-blocked-20260302/` and `.../post-fix/`.

## Fix summary
- Converted panic-oriented test control flow to `Result`-based propagation in:
  - `src/test_harness/ports.rs`
  - `src/test_harness/etcd3.rs`
  - `src/pginfo/worker.rs`
- Added bounded readiness probing before first SQL operations in pginfo real-PG tests to remove full-suite startup race exposure while keeping strict failure semantics.
- Preserved mandatory real-binary coverage and reservation timing semantics (`drop(reservation)` immediately before child bind/start).

<acceptance_criteria>
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] BDD features pass (covered by `make test`).
</acceptance_criteria>

## Implementation Plan (Verified Draft)

### 0) Scope reconciliation and evidence refresh
- [x] Re-run the exact listed repro commands and capture logs under a new evidence folder for this bug.
- [x] Re-run full `make test` once in isolation (no parallel Cargo gates) and record failing test names.
- [x] Classify failure mode into one of:
  - `A`: `allocate_ports` fails with `Operation not permitted`.
  - `B`: port allocation succeeds but real-PG/real-etcd tests fail later (connectivity/startup race).
  - `C`: mixed (`A` and `B`) across repeated runs.
- [x] Add a short “Current observed behavior” section if current failures differ from the original bug narrative.

### 1) Deep root-cause investigation (skeptical)
- [x] Audit all `allocate_ports` call sites (`test_harness`, `pginfo`, `process`) and document exact reservation drop timing per caller.
- [x] Confirm failure boundary for each affected test: allocation failure vs child-spawn bind failure vs readiness timeout vs SQL/bootstrap handshake failure.
- [x] Run deterministic repro loops (minimum 5 repetitions per flaky-prone target) to separate deterministic failures from transient noise.
- [x] Remove all temporary instrumentation before final verification gates.

### 2) Fix design constraints (before coding)
- [x] Enforce typed, explicit error propagation for harness setup/teardown paths; no unwrap/expect/panic in touched runtime code.
- [x] Enforce the same no-panic policy in touched tests by converting panic-oriented branches to `Result`-returning test functions (`-> Result<(), ...>`) with `?` and structured assertions.
- [x] Keep real-binary coverage mandatory (no skips/optionality), and make environment-policy failures explicit and diagnosable.
- [x] Decide bounded retry usage only for startup/readiness races (never to hide real bind/config errors).

### 3) Implement harness and test hardening
- [x] Update `src/test_harness/ports.rs` for clearer allocation diagnostics and panic-free test paths.
- [x] Update `src/test_harness/etcd3.rs` real-binary tests to use `Result`-based flow with reliable cleanup helpers.
- [x] Update `src/pginfo/worker.rs` real-PG tests to eliminate panic chains and preserve deterministic primary/replica assertions.
- [x] Update `src/process/worker.rs` only if needed for consistent reservation/subscriber lifecycle semantics.
- [x] Keep reservation semantics precise: hold while selecting; release immediately before the child bind/start call.

### 4) Validate determinism and regressions
- [x] Run targeted tests first:
  - `cargo test test_harness::ports::tests::allocate_ports_returns_unique_ports -- --nocapture`
  - `cargo test test_harness::ports::tests::concurrent_allocations_do_not_collide_while_reserved -- --nocapture`
  - `cargo test test_harness::etcd3::tests::spawn_etcd3_requires_binary_and_spawns -- --nocapture`
  - `cargo test pginfo::worker::tests::step_once_transitions_unreachable_to_primary_and_tracks_wal_and_slots -- --nocapture`
  - `cargo test pginfo::worker::tests::step_once_maps_replica_when_polling_standby -- --nocapture`
  - `cargo test process::worker::tests::real_restart_job_executes_binary_path -- --nocapture`
- [x] Repeat flaky-prone targeted tests (>=5 runs each) to ensure stability.
- [x] Run required gates sequentially once all targeted checks are stable:
  - [x] `make check`
  - [x] `make test`
  - [x] `make test`
  - [x] `make lint`

### 5) Task bookkeeping and closeout
- [x] Update this task file with final findings, exact root cause, and fix summary.
- [x] Tick all acceptance checkboxes only after evidence is confirmed.
- [x] Set status/passes tags to done/true when gates are green.
- [x] Append new learnings to `AGENTS.md` if any novel pitfalls were found.
- [x] Run `/bin/bash .ralph/task_switch.sh`.
- [x] Commit all changes (including `.ralph` artifacts) with:
  - `task finished real-binary-tests-fail-when-port-allocation-is-blocked: <summary with gate evidence and implementation notes>`
