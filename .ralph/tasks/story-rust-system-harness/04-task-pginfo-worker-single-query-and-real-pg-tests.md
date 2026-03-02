---
## Task: Implement pginfo worker single-query polling and real PG tests <status>done</status> <passes>true</passes> <passing>true</passing> <priority>high</priority>

<blocked_by>03-task-worker-state-models-and-context-contracts</blocked_by>

<description>
**Goal:** Implement pginfo state derivation with one SQL poll query and verify behavior against real PG16.

**Scope:**
- Implement `src/pginfo/query.rs`, `src/pginfo/state.rs`, `src/pginfo/worker.rs`, `src/pginfo/mod.rs`.
- Implement `PGINFO_POLL_SQL`, `poll_once`, `derive_readiness`, `to_member_status`, `run`, `step_once`.
- Add tests using real postgres instances for primary/replica role transitions and WAL/slot fields.

**Context from research:**
- Plan requires exactly one query per poll cycle for all HA-needed fields.

**Expected outcome:**
- PgInfo publishes correct typed state snapshots from real database behavior.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] `PGINFO_POLL_SQL` returns all required fields for role/readiness/timeline/WAL/slots.
- [x] `PgInfoState` transitions are verified for unavailable -> primary/replica.
- [x] Real PG16 tests validate WAL movement and role switch behavior.
- [x] Run targeted pginfo tests.
- [x] Run `make check`.
- [x] Run `make test`.
- [x] Run `make lint`.
- [x] Run `make test-bdd`.
- [x] If failures occur, create `$add-bug` task(s) with logs and SQL/result evidence.
</acceptance_criteria>

<execution_plan>
## Detailed Implementation Plan

1. Prerequisites and baseline gates
- [x] Confirm blocker task `03-task-worker-state-models-and-context-contracts` is marked done and passing before modifying pginfo behavior.
- [x] Capture baseline by running `cargo check --all-targets` and note any pre-existing failures separately from pginfo work.
- [x] Confirm availability expectations for real PG tests (PG16 binaries at `/usr/lib/postgresql/16/bin/postgres` and `/usr/lib/postgresql/16/bin/initdb`) and keep tests self-skipping when binaries are not present.

2. Build a typed query-row contract for one-shot polling
- [x] Implement a typed poll row in `src/pginfo/query.rs` that captures all fields needed for both primary and replica projections in a single SQL result row.
- [x] Keep SQL as one statement (`PGINFO_POLL_SQL`) that returns role/recovery, readiness, timeline, WAL positions, and slot details from one snapshot.
- [x] Avoid any `unwrap` usage; propagate parse and conversion errors as `WorkerError::Message`.
- [x] Add postgres client dependency (`tokio-postgres`) and map database errors into `WorkerError::Message` boundaries to keep worker contracts stable.

3. Define one-query SQL payload shape
- [x] Replace `SELECT 1` with a single query returning:
- [x] role signal (`pg_is_in_recovery()`).
- [x] readiness signal (SQL expression mapping to ready/not-ready).
- [x] timeline (`pg_control_checkpoint()` timeline id).
- [x] primary WAL (`pg_current_wal_lsn()`).
- [x] replica replay WAL (`pg_last_wal_replay_lsn()`).
- [x] upstream/follow WAL (`pg_last_wal_receive_lsn()`).
- [x] replication slot names (aggregated deterministically with `COALESCE(array_agg(... ORDER BY ...), '{}')`).
- [x] Ensure nullability is explicit so primary and replica paths can coexist from one row.

4. Implement `poll_once` in `src/pginfo/query.rs`
- [x] Add async DB polling function that opens one SQL connection, executes `PGINFO_POLL_SQL` exactly once, maps the row to typed poll output, then closes cleanly.
- [x] Add helpers to parse WAL LSN text values into `WalLsn` (`u64`) with strict validation for hexadecimal `X/Y` formats.
- [x] Parse timeline into `TimelineId` and slot arrays into `Vec<ReplicationSlotInfo>` with stable ordering.
- [x] Return unreachable/SQL-failure states as recoverable `WorkerError` values (no panics).

5. Implement state derivation in `src/pginfo/state.rs`
- [x] Add `derive_readiness` that maps query indicators and SQL health into `Readiness` values.
- [x] Add `to_member_status` (or equivalent projection helper) to map typed poll output to `PgInfoState::{Unknown,Primary,Replica}`.
- [x] Ensure `PgInfoCommon` fields (`worker`, `sql`, `readiness`, `timeline`, `last_refresh_at`) are always populated consistently.
- [x] Preserve deterministic behavior for missing fields (e.g. absent replay LSN or upstream on primary).

6. Implement worker loop behavior in `src/pginfo/worker.rs`
- [x] Implement `step_once` to perform one poll cycle, derive next typed state, and return `Result<(), WorkerError>`.
- [x] Implement `run` loop to repeatedly call `step_once` with configured sleep interval and graceful error propagation.
- [x] Ensure `step_once` never panics and can tolerate temporary SQL outages.
- [x] Expand `PgInfoWorkerCtx` with poll inputs/outputs needed by real execution (`postgres_dsn`, `poll_interval`, and a state publisher handle) and update contract tests accordingly.

7. Wire module surfaces in `src/pginfo/mod.rs`
- [x] Export only crate-needed contracts from `query`, `state`, and `worker` modules.
- [x] Keep visibility narrow (`pub(crate)` only where required by current/follow-up tasks).

8. Unit tests for SQL/parse/derive logic
- [x] Add focused tests for WAL LSN parser valid/invalid inputs.
- [x] Add tests for role mapping and readiness derivation (`Unknown -> Primary/Replica` projections).
- [x] Add tests ensuring slot aggregation parsing and ordering are stable.
- [x] Add tests for SQL constant shape (single statement, expected selected fields present).

9. Real PG16 integration tests
- [x] Add pginfo integration tests that spawn real PG via `test_harness::pg16` and validate state transitions from unavailable to reachable primary.
- [x] Add replica-path test setup (two instances with replication config) to verify `Replica` mapping and replay/follow fields.
- [x] Add WAL movement assertions by creating writes and observing WAL/replay movement across polls.
- [x] Add slot field assertions by creating replication slots and verifying projected slot names.
- [x] Keep tests robust with cleanup guards and no `unwrap`.
- [x] Add helper SQL execution utility for tests (via `tokio-postgres`) to issue writes, slot creation, and polling assertions without shelling out to `psql`.

10. Targeted verification commands
- [x] Run targeted pginfo test selection first (unit + integration).
- [x] Run `make check`.
- [x] Run `make test`.
- [x] Run `make test-bdd`.
- [x] Run `make lint`.

11. Failure handling and bug-task protocol
- [x] If any required command fails, collect failing command output and SQL/result evidence.
- [x] Use `$add-bug` skill to create `.ralph/tasks/bugs/` entries for each distinct failure mode with reproduction steps.

12. Completion bookkeeping (execution phase)
- [x] Tick acceptance criteria and update header tags only after all required commands pass.
- [x] Set `<passing>true</passing>` only after full gate success.
- [x] Run `/bin/bash .ralph/task_switch.sh`.
- [x] Commit all files (including `.ralph` updates) with message format `task finished 04-task-pginfo-worker-single-query-and-real-pg-tests: <summary + evidence + challenges>`.
- [x] Append any learnings/surprises to `AGENTS.md`.
- [x] Append a progress diary entry before exit.

13. Skeptical verification amendments (added during TO BE VERIFIED)
- [x] Correct reconnaissance references to current repo layout (`src/state/{errors,ids,time,watch_state}.rs`; no `src/worker/*` tree), and remove assumptions tied to non-existent files.
- [x] Require deterministic timeline extraction using scalar cast from `pg_control_checkpoint()` (`(pg_control_checkpoint()).timeline_id::bigint`) to avoid composite-deserialization ambiguity.
- [x] Require single-row slot projection with `array_remove(array_agg(slot_name ORDER BY slot_name), NULL)` and explicit SQL-level `COALESCE` for empty arrays.
</execution_plan>

NOW EXECUTE
