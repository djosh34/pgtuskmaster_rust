## Plan: Collapse Postgres Ingest Stringly Issue Round-Trip

### Why this is the next reduction target

`src/logging/postgres_ingest.rs` is still carrying a wrong boundary where typed ingest failures are flattened into ad-hoc text early, then parsed back into typed data later just so the rate limiter can recover the same fields.

- `src/logging/postgres_ingest.rs:186-224` receives a `WorkerError`, immediately calls `ingest_error_key_best_effort(...)`, and extracts `stage`, `kind`, and `path` back out of the string only for `IngestErrorRateLimiter`.
- `src/logging/postgres_ingest.rs:236-267` is pure recovery code for that boundary mistake: it reparses the string form into `IngestErrorKey`.
- `src/logging/postgres_ingest.rs:303-377` and `src/logging/postgres_ingest.rs:409-489` build string issues in multiple helpers, aggregate those strings in `step_once`, and then wrap them into another `WorkerError::Message`.
- `src/logging/postgres_ingest.rs:310-345`, `src/logging/postgres_ingest.rs:492-507`, and `src/logging/postgres_ingest.rs:510-649` also pass raw stage/message strings around only to format one-off error text later.

That is the wrong ownership boundary. The ingest worker already knows the typed issue fields at the failure site, so it should keep those fields typed until the final log/error rendering boundary.

### Current overlap already verified

- `IngestErrorKey` in `src/logging/postgres_ingest.rs:116-121` and `ingest_error_key_best_effort(...)` in `src/logging/postgres_ingest.rs:236-267` encode the same `stage`/`kind`/`path` shape twice.
- `ingest_issue(...)` in `src/logging/postgres_ingest.rs:303-308` and the direct `format!(...)` calls in `src/logging/postgres_ingest.rs:337-340`, `src/logging/postgres_ingest.rs:548-550`, `src/logging/postgres_ingest.rs:560-562`, `src/logging/postgres_ingest.rs:574-576`, `src/logging/postgres_ingest.rs:627-629`, and `src/logging/postgres_ingest.rs:641-643` all serialize the same issue shape by hand.
- The tests at `src/logging/postgres_ingest.rs:918-983` spend real code covering the parser helper and the string-only failure boundary instead of the actual ingest behavior.

### Execution plan

1. Make postgres-ingest failures stay typed until the worker boundary.
   - Introduce one local typed issue owner in `src/logging/postgres_ingest.rs` that carries `stage`, `kind`, `path`, and `cause`.
   - Change `emit_tailer_lines`, `discover_log_dir`, `cleanup_log_dir`, and any helper that currently returns or pushes issue strings so they traffic in that typed issue instead.
   - Keep `WorkerError::Message` as the outer crate boundary, but only render the issue text once when the ingest worker needs the final human-readable message.

2. Delete the string reparse layer and collapse the rate-limit key onto the typed issue source.
   - Remove `ingest_error_key_best_effort(...)`.
   - Make `run(...)` rate-limit directly from the typed issue or typed iteration error returned by `step_once(...)`, instead of reparsing `WorkerError` text.
   - If a dedicated key type still helps the limiter, derive it directly from the typed issue rather than rebuilding it from strings.

3. Collapse the stringly helper parameters while touching the same area.
   - Remove the extra message/stage string plumbing from `collect_ingestable_postgres_log_paths(...)` and let the caller construct typed issues at the actual failure site.
   - Remove the dead `_origin` argument from `postgres_line_event(...)` and its callers, because it no longer contributes any behavior.
   - Update the ingest tests to assert the surviving typed failure behavior and delete parser-specific tests that only existed for the removed round-trip.

4. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and require the diff to remain net-negative.
   - Run `make check`.
   - Run `make lint`.
   - Run `make test`.
   - Run `make test-long`.

### Guardrails

- Keep the outer `WorkerError` crate boundary unchanged unless deleting it is clearly net-reductive across more than this one file.
- Do not introduce a second typed wrapper around the new issue owner; one local issue type is enough.
- Preserve the current ingest failure semantics: first issue drives the summary, extras remain capped, and rate limiting still keys on `stage`/`kind`/`path` rather than full message text.

### Expected yield

- Delete the string parser helper that exists only to undo the module’s own formatting decisions.
- Remove multiple repeated `format!("stage=... kind=... path=... error=...")` call sites and the helper strings they force through the call graph.
- Shorten both implementation and tests by keeping ingest failures typed until the single real rendering boundary.

NOW EXECUTE
