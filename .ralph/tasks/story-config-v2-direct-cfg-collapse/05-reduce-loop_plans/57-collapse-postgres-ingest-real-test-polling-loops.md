## Plan: Collapse Postgres Ingest Real-Test Polling Loops

### Why this is the next reduction target

The current HA polling-helper slice is still net-positive in local lines because `tests/ha/support/steps/mod.rs` needs a policy-heavy adapter to preserve terminal-container failure handling. A better immediate reduction target is the real-test section of `src/logging/postgres_ingest.rs`, where the same deadline/sleep polling boundary is still hand-rolled several times inside one owner.

- `wait_for_process_idle_success_with_debug(...)` already owns one local "deadline, pump worker state, inspect condition, sleep, timeout" loop.
- `ingests_jsonlog_and_stderr_files_from_real_postgres(...)` repeats the same shape while pumping ingest state and collecting test logs until a predicate passes.
- `ingests_pg_ctl_log_file_and_captures_pg_tool_output(...)` repeats the same shape again while pumping both ingest and process state and accumulating records.
- `captures_helper_binary_stdout_stderr_on_failure(...)` repeats the same deadline/poll/collect loop a third time.

That is a cleaner boundary problem than plan 56: one test module already owns the real retry cadence, but three nearby tests still inline the same loop with only caller-specific pump work, success predicates, and timeout text changed.

### Current overlap already verified

- `src/logging/postgres_ingest.rs:1241` defines `wait_for_process_idle_success_with_debug(...)`, which already owns:
  - deadline calculation,
  - repeated async tick work,
  - timeout sleep cadence,
  - timeout rendering.
- `src/logging/postgres_ingest.rs:1495` hand-rolls the same structure to wait for jsonlog + stderr ingestion.
- `src/logging/postgres_ingest.rs:1718` hand-rolls the same structure to wait for pg_ctl log ingestion plus pg tool capture plus jsonlog capture.
- `src/logging/postgres_ingest.rs:1864` hand-rolls the same structure to wait for captured `pg_basebackup` stderr.
- `src/logging/postgres_ingest.rs:1367` also has a deadline/sleep retry loop in `run_psql_query_with_retry(...)`, but that one should only be merged if doing so is a clear net deletion after the first three loops are collapsed.

### Execution plan

1. Extract one local real-test polling helper in `src/logging/postgres_ingest.rs`.
   - Keep it inside the existing test module.
   - Let it own only:
     - timeout window,
     - poll sleep cadence,
     - one async tick closure,
     - caller-specific success/continue decision,
     - caller-specific timeout error text.
   - Do not introduce a new enum or DTO just to represent polling state.

2. Rebuild the existing process-idle waiter on the shared local helper.
   - Rewrite `wait_for_process_idle_success_with_debug(...)` to use the new helper instead of owning its own deadline loop.
   - Preserve the current failure/timeout debug-tail rendering.

3. Collapse the three duplicated real-log polling loops onto the same helper.
   - Rewrite the loop in `ingests_jsonlog_and_stderr_files_from_real_postgres(...)`.
   - Rewrite the loop in `ingests_pg_ctl_log_file_and_captures_pg_tool_output(...)`.
   - Rewrite the loop in `captures_helper_binary_stdout_stderr_on_failure(...)`.
   - Keep each test’s current collected-record behavior and timeout wording intact.

4. Only fold `run_psql_query_with_retry(...)` in if it is obviously smaller afterward.
   - If the helper can absorb it with a small closure and no new tri-state courier, include it.
   - If adapting the transient/non-transient branch adds more scaffolding than it removes, leave `run_psql_query_with_retry(...)` alone for this slice.

5. Keep the current dirty HA polling experiment out of this execution slice.
   - Do not stack further edits onto `tests/ha/support/mod.rs`, `tests/ha/support/steps/mod.rs`, `tests/ha/support/world/mod.rs`, or `tests/ha/support/invariants/write_convergence.rs`.
   - Revert or otherwise discard that unfinished local experiment before validating this new slice, because it is not the plan being executed.

6. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the reduction improves beyond the current baseline.
   - Run `make check`.
   - Run `make lint`.
   - Run `make test`.
   - Run `make test-long`.

### Guardrails

- Reuse the existing local test-module boundary; do not create a new cross-module polling utility for this slice.
- Do not replace one local helper pile with a generic state courier or result taxonomy.
- Keep record collection and timeout messages owned by each test; only the retry mechanics should move.
- If the new helper starts needing more than one small closure result type to distinguish "keep polling" from "hard fail", switch this plan back to `TO BE VERIFIED`.
- Do not commit any part of the abandoned HA polling-helper experiment as part of this slice.

### Blocker discovered during execution

The proposed helper shape is not valid as written. Rust rejects the async `FnMut` polling closures because each loop needs to borrow mutable local state such as `ProcessWorkerCtx`, `PostgresIngestWorkerState`, collected log vectors, and borrowed config-backed contexts across `.await` points. A generic `poll_until(...)` that takes one async closure therefore either:

- explodes into higher-ranked future/lifetime plumbing that is larger than the deleted code, or
- forces ownership moves/clones that break the current local test setup and still do not compose cleanly with the borrowed contexts.

Before this plan can be executed again, the polling reduction needs a different boundary, likely one of:

- a helper that owns a concrete borrowed context pair explicitly instead of a generic async closure, or
- a narrower reduction that only collapses the purely process-worker loops and leaves the ingest-state loops separate.

TO BE VERIFIED
