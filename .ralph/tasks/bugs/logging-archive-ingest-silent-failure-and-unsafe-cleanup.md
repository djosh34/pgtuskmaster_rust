---
## Bug: Postgres ingest silently swallows failures and cleanup/path ownership can destroy active observability signals <status>done</status> <passes>true</passes>

<description>
Postgres observability ingest has several correctness failures in the current logging pipeline:

1) `postgres_ingest::run()` suppresses errors from ingest and cleanup steps (`step_once` and `cleanup_log_dir`), so failures are silent and operators lose telemetry without actionable diagnostics.
2) `cleanup_log_dir()` only protects `pg_ctl_log_file` and can delete currently active files in `logging.postgres.log_dir` (for example active `postgres.json` and `postgres.stderr.log`), causing dropped logs.
3) No path ownership validation prevents sink/source overlap (for example `logging.sinks.file.path` overlapping tailed Postgres log files), which can create recursive self-ingestion loops and log amplification.

Please investigate the full flow first, then implement a fix that is explicit about ownership boundaries and failure surfacing.
</description>

<acceptance_criteria>
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make test-long` — passes cleanly (ultra-long-only)
- [x] `make lint` — passes cleanly
- [x] `make docs-build` — passes cleanly (docs render)
- [x] `make docs-hygiene` — passes cleanly (docs repo hygiene)
</acceptance_criteria>

---

## Implementation plan (draft)

Context from investigation:
- The ingest worker is wired into `tokio::try_join!` in `src/runtime/node.rs`, so returning `Err` from `postgres_ingest::run()` will currently abort the whole node. Because of that, we should surface errors loudly but keep the worker alive for transient failures.
- `src/logging/postgres_ingest.rs` currently swallows errors in two places:
  - `run()` ignores `step_once()` results (`let _ = ...`).
  - `step_once()` ignores `cleanup_log_dir()` results (`let _ = ...`).
- `cleanup_log_dir()` only protects a single path (passed as `pg_ctl_log_file`), and can delete actively-written `logging.postgres.log_dir` files such as `postgres.json` and `postgres.stderr.log`.
- `validate_logging_path_ownership_invariants()` exists in `src/config/parser.rs` and already checks for overlap, but comparisons are lexical (raw `Path` equality / `starts_with`) and can be bypassed by `./` / `../` segments and potentially symlink aliasing.

### Goals
- Make ingest/cleanup failures *observable* (no more silent failure).
- Make cleanup *safe by default* (must not delete active observability signals in `logging.postgres.log_dir`).
- Make path ownership invariants *robust* (prevent self-ingestion loop configs even with path aliasing).

### Non-goals (for this bug, unless we discover it’s required)
- Introduce a new “logging worker health” domain in the debug snapshot / state channel system.
- Re-architect worker shutdown / cancellation across the whole node runtime.

---

## Phase 0 — Ensure prerequisites for non-optional tests
- [x] Verify real-binary policy is satisfied (Postgres 16, etcd, pgbackrest) so real-binary tests cannot “skip”.
  - If failing, run the repo scripts referenced by the harness error messages:
    - [ ] `./tools/install-etcd.sh`
    - [ ] `./tools/install-postgres16.sh`
    - [ ] `./tools/install-pgbackrest.sh`
  - [x] Re-run `make test` after installs to ensure failures were due to missing binaries only.

---

## Phase 1 — Stop swallowing errors in `postgres_ingest`

### 1.1: Make `run()` surface `step_once()` failures (without killing the node)
- [x] In `src/logging/postgres_ingest.rs`, replace `let _ = step_once(&ctx, &mut state).await;` with explicit handling:
  - [x] On `Ok(())`: reset a local consecutive-failure counter.
  - [x] On `Err(err)`: emit a structured internal log record via `LogHandle` (severity `Error`), then continue the loop.
- [x] Add minimal anti-spam protection to avoid flooding sinks on repeated identical failures:
  - [x] Implement a small, deterministic rate-limiter keyed by `(stage, error_kind, path-ish)` that accepts an injected “tick” (u64) so tests do not need `sleep`.
  - [x] Emit the first failure immediately; suppress repeats within a window (e.g. 30s); when re-emitting include `suppressed=N` in the message/attributes.
- [x] Keep `run()` signature as `Result<(), WorkerError>` for compatibility with `try_join!`, but do not return `Err` for normal transient ingest failures.

### 1.2: Make `step_once()` resilient (don’t stop at first failure)
- [x] Refactor `step_once()` to attempt all phases even if one fails:
  - [x] `pg_ctl_log_file` tail/read
  - [x] `log_dir` discovery + tail/read (if configured)
  - [x] `log_dir` cleanup (if enabled)
- [x] Collect per-phase errors with a stable `stage=` tag (string-only is fine; `WorkerError` is currently `Message(String)`).
- [x] At the end of `step_once()`:
  - [x] If no errors: return `Ok(())`
  - [x] If any errors: return `Err(WorkerError::Message(...))` that includes a short summary + at least the first error with `stage=`.
    - `run()` will rate-limit and emit this error.

---

## Phase 2 — Make `cleanup_log_dir()` safe and observable

### 2.1: Change cleanup API to be explicit about protection + failures
- [x] Change `cleanup_log_dir()` to avoid silent IO failures:
  - [x] Stop using `let _ = tokio::fs::remove_file(...).await`.
  - [x] Record failures (path + io error) in a `CleanupReport` (or bubble them up as a `WorkerError` with clear `stage=cleanup.remove_file`).
- [x] Update the call site in `step_once()` to stop ignoring cleanup:
  - [x] Either treat cleanup failures as non-fatal but returned from `step_once()` (preferred: aggregated error), or emit an internal warning and continue.

### 2.2: Protect actively-written `log_dir` signals from deletion
Constraints:
- `DirTailers` keeps entries forever and can include old files; we must not “protect everything we ever discovered”.

Implementation approach inside cleanup (conservative + explicit, but less heuristic):
- [x] Add a new cleanup safety knob to config: `logging.postgres.cleanup.protect_recent_seconds` (default: 300s).
- [x] Build a protected set per cleanup pass:
  - [x] Always protect the `pg_ctl_log_file` path (existing behavior).
  - [x] Always protect these basenames if present in `log_dir` (default active-signal conventions):
    - [x] `postgres.json`
    - [x] `postgres.stderr.log`
    - [x] `postgres.stdout.log`
  - [x] Protect any `log_dir` file whose `modified()` time is within `protect_recent_seconds` (generic “likely active” guard; covers custom filenames).
- [x] Treat missing/unknown metadata conservatively but *observable*:
  - [x] Do not delete files whose metadata/mtime cannot be read; record this as a cleanup issue so operators understand why retention may not progress.
- [x] Apply retention policies only to eligible candidates:
  - [x] Filter to `*.log` and `*.json`.
  - [x] Exclude protected files.
  - [x] Enforce `max_files` on remaining files (delete oldest first).
  - [x] Enforce `max_age_seconds` on remaining files (delete only when clearly older).

### 2.3: Cleanup determinism + race handling
- [x] Make ordering deterministic and stable:
  - [x] Sort by `(modified_time, path)` so ties are deterministic.
- [x] Handle TOCTOU expected races:
  - [x] `remove_file` with `NotFound` is OK (already removed).
  - [x] Other `remove_file` failures are reported (not silent).

---

## Phase 3 — Strengthen path ownership validation (prevent self-ingest loops)

### 3.1: Require absolute paths + lexical normalization before comparisons
Rationale:
- Rust unit tests run in parallel, so validations that depend on global `current_dir()` are hard to test deterministically.
- For a greenfield, “absolute paths only” is an acceptable hardening that prevents relative/absolute mismatch bypasses entirely.

- [x] In `src/config/parser.rs`, enforce that the following paths are absolute when set / enabled:
  - [x] `postgres.log_file`
  - [x] `logging.postgres.pg_ctl_log_file` (if set; otherwise `postgres.log_file` applies)
  - [x] `logging.postgres.log_dir` (if set)
  - [x] `logging.sinks.file.path` (when `logging.sinks.file.enabled=true`)
- [x] Add a small helper to do *lexical* normalization (`.` and `..` removal) using `Path::components()` (no panics/unwraps).
- [x] For comparisons, use normalized paths for:
  - [x] equality checks (`sink_path` vs tailed input paths)
  - [x] containment checks (`sink_path` inside `logging.postgres.log_dir`)
  - [x] Defer symlink canonicalization to a follow-up (runtime-only hardening), unless we find a safe way to do it without making config validation brittle for paths that don’t exist yet.

### 3.2: Add config validation tests for bypass cases
- [x] Add tests in `src/config/parser.rs` covering at least:
  - [x] sink path equals tailed file via `./` segment
  - [x] sink path equals tailed file via `..` segment
  - [x] sink path is inside `log_dir` via `./` and `..` segments
  - [ ] (unix-only) symlink alias cases, if canonicalization is implemented

---

## Phase 4 — Tests: prevent regression of “silent failure” and “unsafe cleanup”

### 4.1: Unit tests for cleanup safety + error surfacing
- [x] Extend `src/logging/postgres_ingest.rs` tests:
  - [x] New test: cleanup never deletes `postgres.json` / `postgres.stderr.log` / `postgres.stdout.log` even under `max_files` pressure.
  - [x] New test: cleanup deletion failures are observable (unix-only: readonly dir or permission-denied removal).
  - [x] Update existing test `cleanup_log_dir_enforces_max_files_and_protects_active_file` to match new protection semantics (if signature changes).

### 4.2: Runtime-level tests for ingest error surfacing
- [x] Add a test sink (or reuse existing `test_log_handle()`) to assert:
  - [x] `run()` emits an internal error record when `step_once()` fails (rate-limit permitting).
  - [x] repeated failures are rate-limited (suppression counter increases, summary emitted later).

### 4.3: Real-binary test (if unit tests aren’t sufficient)
- [ ] Add/extend a `tests::real_binary` scenario in `src/logging/postgres_ingest.rs`:
  - [ ] Start real PG16 with `logging_collector` writing into `log_dir`.
  - [ ] Enable cleanup with aggressive limits.
  - [ ] Generate logs and run `ingest_step_once` repeatedly.
  - [ ] Assert the active `log_dir` file still exists after cleanup cycles and ingestion continues.

---

## Phase 5 — Documentation updates (remove stale docs; document new behavior)
- [x] Update `docs/src/operator/configuration.md`:
  - [x] Document `logging.postgres.log_dir` cleanup semantics, including which files are protected and that failures are surfaced as internal log records.
  - [x] Document path ownership invariants: sink path must not overlap tailed Postgres inputs and must not be inside `log_dir` (with normalized-path semantics).
- [x] Update `docs/src/operator/observability.md` (or the most relevant observability page):
  - [x] Mention that ingest/cleanup errors appear as `PgTool/Internal` log records with `origin=postgres_ingest` and `stage=...`.

---

## Phase 6 — Verification gates (must be green)
- [x] `make check`
- [x] `make test`
- [x] `make test-long`
- [x] `make lint`
- [x] `make docs-build`
- [x] `make docs-hygiene`
- [x] Confirm docs changes reflect actual behavior (no stale statements / examples).
