## Plan: Collapse Postgres Ingest Dir Scan And Issue Couriers

### Why this reduction target

`src/logging/postgres_ingest.rs` is still one of the largest untouched files in `src/`, and a lot of that size comes from local courier layers around a single ingest workflow:

- `step_once(...)` has two near-identical loops that drain a tailer, emit normalized line events, and then rewrap failures into temporary issue state.
- `discover_log_dir(...)` and `cleanup_log_dir(...)` both rescan the same directory, re-check `file_type`, and reapply the same `.log` / `.json` filtering rules before diverging.
- `cleanup_log_dir(...)` returns `CleanupReport` only so `step_once(...)` can immediately turn it back into another issue string.

That is the same overabstraction pattern as earlier slices: one owner (`postgres_ingest.rs`) already has the real facts, but it still keeps projecting them into extra wrappers and duplicate passes instead of running one direct workflow.

### Current overlap already verified

- `step_once(...)` reads `state.pg_ctl_log` and each `state.dir_tailers` entry with the same `read_new_lines(...) -> ctx.log.send(postgres_line_event(...)) -> push_issue(...)` shape.
- `discover_log_dir(...)` and `cleanup_log_dir(...)` each call `tokio::fs::read_dir(...)`, iterate `next_entry()`, inspect `file_type()`, and reject non-`.log` / non-`.json` files.
- The `IterationIssue` struct and `push_issue(...)` helper exist only inside `step_once(...)`; they are not a reusable boundary.
- `CleanupReport` is produced only by `cleanup_log_dir(...)` and consumed only by `step_once(...)`.
- The rate limiter only needs the final error string to preserve `stage=... kind=... path=...` tokens; it does not need either courier type.

### Execution plan

1. Collapse repeated tailer emission onto one ingest path.
   - Extract or inline one shared path for `FileTailer` draining plus line emission.
   - Reuse it for both the pg_ctl tailer and discovered directory tailers.
   - Keep the emitted `postgres.line_*` records and summary counters unchanged.

2. Collapse duplicated directory scans.
   - Reuse one directory-entry filter for "ingestable postgres log file" selection, including the `StartPosition` choice for stdout/stderr logs.
   - Use the same filter when discovering tailers and when collecting cleanup candidates.
   - Preserve the current protected basenames and recent-file protection rules.

3. Delete the issue/report couriers.
   - Remove `IterationIssue`, `push_issue(...)`, and `CleanupReport`.
   - Record issue strings directly in the `Vec<String>` that already feeds the final worker error.
   - Keep the final `postgres_ingest iteration_errors ...` message format stable enough for `ingest_error_key_best_effort(...)`.

4. Rewrite tests around observable behavior.
   - Update cleanup tests to assert retained/deleted files and surfaced error text instead of `CleanupReport` fields.
   - Add or adjust coverage so one `step_once(...)` path exercises both the shared line-emission flow and the stable `stage=... kind=... path=...` error tokens.

5. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the reduction improves beyond the current `+7442 -10525 diff: -3083` baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.

### Guardrails

- Do not replace `CleanupReport` or `IterationIssue` with another renamed DTO.
- Reuse `FileTailer`, `DirTailers`, and the existing error-string contract instead of inventing a new issue model.
- If sharing the directory scan forces a new metadata courier that just mirrors `tokio::fs::DirEntry`, switch this plan back to `TO BE VERIFIED`.

NOW EXECUTE
