## Plan: Collapse Postgres Ingest Parser Couriers And Step Glue

### Why this is the next verified reduction target

`src/logging/postgres_ingest.rs` is now the largest remaining file in the tree, and the next still-local seam is not another new abstraction. It is leftover local glue that keeps one ingest workflow fragmented across private wrappers and courier structs.

The file still has three verified reduction smells:

- one single-caller courier type only used to pass parser output back into the real event owner
- one single-caller discover wrapper that only bridges a file scan into `DirTailers::ensure_file(...)`
- repeated per-tailer bookkeeping in `step_once(...)` where the same `emit_tailer_lines(...)` success/error handling is rebuilt for the pg-ctl tailer and the log-dir tailers

That is a good next slice because it stays inside one owner file, reuses the existing `PostgresLineLogEvent`, `IngestIssue`, and `DirTailers` owners, and should delete code without introducing another DTO or trait layer.

### Verified overlap in the current code

1. `ParsedLine` is still a fake boundary.
   - `src/logging/postgres_ingest.rs`
     - `normalize_postgres_line(...)`
     - `normalize_postgres_json(...)`
     - `normalize_postgres_plain(...)`
   - `normalize_postgres_json(...)` and `normalize_postgres_plain(...)` are only called from `normalize_postgres_line(...)`.
   - Both helpers only exist to fill `ParsedLine { severity, message, payload, level_raw }`, then `normalize_postgres_line(...)` immediately turns that courier back into `PostgresLineLogEvent::{Json, Plain}`.
   - The `source.path.clone()` plumbing is repeated in all three event branches because the real owner is split across those helpers.

2. The log-dir discover flow still has a one-caller wrapper plus repeated start-position resolution.
   - `collect_ingestable_postgres_log_paths(...)` already filters paths by `ingestable_postgres_log_start(...)`.
   - `discover_log_dir(...)` is only called once, from `step_once(...)`.
   - Inside `discover_log_dir(...)`, each returned path is immediately passed back through `ingestable_postgres_log_start(...)` again just to recover the `StartPosition` needed by `DirTailers::ensure_file(...)`.
   - That means the file scan owner is still split across:
     - `collect_ingestable_postgres_log_paths(...)`
     - `discover_log_dir(...)`
     - `step_once(...)`

3. `step_once(...)` still rebuilds the same per-tailer error/bookkeeping path.
   - The pg-ctl tailer path:
     - calls `emit_tailer_lines(...)`
     - updates a line counter on success
     - pushes one `IngestIssue` on failure
   - The log-dir tailer loop repeats the same control flow:
     - calls `emit_tailer_lines(...)`
     - updates a line counter on success
     - pushes one `IngestIssue` on failure
   - The only real variation is which counter gets incremented and whether `log_dir_files_tailed` should advance.

4. The parser tests already assert behavior through the real event boundary.
   - Current tests exercise:
     - `normalize_postgres_line(...)`
     - `postgres_line_event(...)`
     - emitted `LogRecord` shape
   - That means the courier cleanup can mostly reuse existing tests instead of adding coverage for another helper layer.

### Execution plan

1. Collapse the parser courier layer onto the actual event owner.
   - Delete `ParsedLine`.
   - Rewrite `normalize_postgres_json(...)` and `normalize_postgres_plain(...)` so they no longer return a shared courier.
   - Keep at most one parser entrypoint:
     - either make `normalize_postgres_line(...)` build `PostgresLineLogEvent` directly
     - or let the JSON/plain helpers return `PostgresLineLogEvent` directly if that is smaller after formatting
   - Do not add a new parser result enum or wrapper struct.

2. Collapse the log-dir discover wrapper onto the scan owner.
   - Replace the current `collect_ingestable_postgres_log_paths(...)` return shape with the smallest shape the callers actually need.
   - Prefer returning the already-known `StartPosition` together with each path so `discover_log_dir(...)` does not have to call `ingestable_postgres_log_start(...)` a second time.
   - If the direct scan result makes `discover_log_dir(...)` a one-line wrapper, inline it into `step_once(...)` and delete the wrapper entirely.

3. Collapse the repeated per-tailer bookkeeping in `step_once(...)`.
   - Add at most one small local helper for:
     - calling `emit_tailer_lines(...)`
     - updating the appropriate emitted-line counter
     - pushing one `IngestIssue` on failure
   - Reuse that same path for:
     - `state.pg_ctl_log`
     - each tailer in `state.dir_tailers`
   - Do not add callback-heavy plumbing or another stats struct just to share this.

4. Keep cleanup local and only touch it if the scan shape naturally simplifies it.
   - If step 2 changes the scan helper return type, update `cleanup_log_dir(...)` to use the smaller shared scan owner.
   - Do not reopen the cleanup policy itself unless the line count clearly drops.

5. Reuse the existing tests at the real boundary.
   - Keep the parser behavior tests, but rewrite them against the surviving owner if any helper name disappears.
   - Delete test code that only exists to exercise removed local wrappers.
   - Do not add new test-only abstraction helpers unless they replace repeated setup and are net-negative after formatting.

6. Validate the slice.
   - `cargo fmt`
   - `bash .ralph/git_current_lines.sh` and verify total drops below `34102`
   - `bash .ralph/git_diff_lines_since.sh` and verify the diff improves beyond `+10241 -14214 diff: -3973`
   - `make check`
   - `make lint`
   - `make test`
   - `make test-long`

### Guardrails

- No new enums, structs, traits, or parser DTOs for this slice.
- Keep `PostgresLineLogEvent` as the logging boundary; do not replace it with a new intermediate shape.
- If the per-tailer helper becomes more abstract than the duplicated branches it removes, inline directly in `step_once(...)` instead of adding the helper.
- If the file-scan helper cannot carry `StartPosition` without widening the API in a larger way, keep the existing scan helper and only delete the clearly single-caller wrapper.

### Expected yield

- Delete the `ParsedLine` courier and one more layer of parser helper indirection.
- Delete the extra `discover_log_dir(...)` wrapper or at least its duplicate `ingestable_postgres_log_start(...)` pass.
- Remove one more chunk of repeated per-tailer bookkeeping from `step_once(...)`.
- Keep the whole slice inside `src/logging/postgres_ingest.rs`, which should make the reduction easy to validate and easy to replan if formatting does not go net-negative.

NOW EXECUTE
