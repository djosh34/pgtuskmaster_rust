## Plan: Collapse Process-Worker Real-Test Waiters With Sync Predicates

### Why this is the next reduction target

Plan 57 proved that a single generic async polling helper is the wrong boundary for `src/logging/postgres_ingest.rs`. The repeated loops do overlap, but the overlap is not "any async step plus any async predicate." Rust rejected that shape because the closures had to hold mutable borrows of `ProcessWorkerCtx`, `PostgresIngestWorkerState`, collected log buffers, and config-backed contexts across `.await`.

The tighter reusable boundary is the process-worker wait path only:

- `wait_for_process_idle_success_with_debug(...)` owns a `process_step_once(...)` poll loop and inspects `ProcessState`.
- `ingests_pg_ctl_log_file_and_captures_pg_tool_output(...)` has a second process-only wait loop for the `start_postgres` job that also accumulates `RunningTestLog` records for richer failure text.
- `captures_helper_binary_stdout_stderr_on_failure(...)` has a third process-only wait loop that steps the process worker and accumulates the same `RunningTestLog` records until a captured `PgTool` stderr record appears.

Those three loops can share one helper if the helper owns every `.await` and the caller supplies only a synchronous predicate over the already-updated process context and accumulated records. That keeps the borrowed state local to the helper body instead of inside an async closure.

### Current overlap already verified

- `src/logging/postgres_ingest.rs:1243` `wait_for_process_idle_success_with_debug(...)` duplicates:
  - timeout bookkeeping,
  - `process_step_once(ctx).await?`,
  - polling sleep cadence,
  - process-state success/continue/fail branching.
- `src/logging/postgres_ingest.rs:1629` the `start_postgres` wait loop duplicates the same timeout bookkeeping and `process_step_once(...)` cadence while also extending a local `Vec<LogRecord>` from `test_log.take().await`.
- `src/logging/postgres_ingest.rs:1882` the `pg_basebackup` stderr wait loop duplicates the same timeout bookkeeping, `process_step_once(...)`, record collection, and sleep cadence.

The ingest-state loops do still repeat polling structure, but they need different stepped owners (`ingest_step_once(...)` alone vs `ingest_step_once(...)` plus `process_step_once(...)`). That should stay out of this slice unless the process-only collapse lands small and obviously leaves room for more deletion.

### Execution plan

1. Discard the failed boxed-future experiment in `src/logging/postgres_ingest.rs`.
   - Remove the current generic `poll_until(...)` helper and the added `Future`/`Pin` imports.
   - Restore the still-valid baseline loops before applying the narrower reduction so the next diff is easy to reason about.

2. Extract one concrete process wait helper inside the `real_binary` test module.
   - Keep it local to `src/logging/postgres_ingest.rs`.
   - Let it own:
     - timeout window,
     - poll sleep cadence,
     - `process_step_once(ctx).await?`,
     - optional `test_log.take().await` accumulation into a local `Vec<LogRecord>`.
   - Give callers only a synchronous predicate closure such as:
     - `FnMut(&ProcessWorkerCtx<'_>, &[LogRecord]) -> Result<bool, WorkerError>`
   - Return `Result<bool, WorkerError>` where `true` means the predicate finished and `false` means timeout.
   - Do not introduce a new enum, DTO, or boxed future just to represent poll state.

3. Rebuild `wait_for_process_idle_success_with_debug(...)` on the concrete helper.
   - Pass no log collector and inspect only `ctx.state_channel.current`.
   - Keep the existing failure/timeout messages and debug-tail behavior intact.
   - If `debug_tail_suffix(...)` still reduces code after this rewrite, keep it; otherwise inline it and delete it.

4. Collapse the two other process-only loops onto the same helper.
   - Rewrite the `start_postgres` wait in `ingests_pg_ctl_log_file_and_captures_pg_tool_output(...)` to reuse the helper and keep its richer failure report logic inside the synchronous predicate.
   - Rewrite `captures_helper_binary_stdout_stderr_on_failure(...)` to reuse the helper and keep its current timeout wording.
   - Reuse the helper-owned accumulated `Vec<LogRecord>` instead of rebuilding timeout/deadline/sleep logic in each test.

5. Leave the ingest-state polling loops alone unless the process-only slice is already clearly net-negative.
   - Do not touch the `jsonlog+stderr` ingest loop yet.
   - Do not touch the mixed ingest+process collection loop yet.
   - Do not touch `run_psql_query_with_retry(...)` in this plan.
   - If, after the process-only collapse, one of those loops can be folded with an obviously smaller synchronous-helper boundary, write a follow-up plan instead of stretching this one mid-execution.

6. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the slice stays net-negative.
   - Run `make check`.
   - Run `make lint`.
   - Run `make test`.
   - Run `make test-long`.

### Guardrails

- The helper must own every `.await`; the callback must stay synchronous.
- Do not use boxed futures, higher-ranked async closure plumbing, or new courier types.
- Keep the helper in the test module; do not create a cross-module polling utility.
- Keep custom failure strings, debug-tail formatting, and job-specific log inspection at the call sites.
- If the helper needs more than one callback or starts needing an `Option<Option<...>>` style state courier, switch back to `TO BE VERIFIED`.

NOW EXECUTE
