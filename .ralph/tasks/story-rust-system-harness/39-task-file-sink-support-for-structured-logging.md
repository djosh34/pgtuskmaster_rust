---
## Task: Add File Sink Support For Unified Structured Logging <status>done</status> <passes>true</passes>

<description>
**Goal:** Extend the unified structured logging subsystem to support configurable JSONL file sinks (in addition to the current stderr JSONL sink).

**Prerequisite:** `.ralph/tasks/story-rust-system-harness/38-task-unified-structured-logging-and-postgres-binary-ingestion.md` (unified log schema + ingestion + `LogSink` trait already exist).

**Scope:**
- Add one or more file-backed sink implementations, e.g. `JsonlFileSink`, that write one JSON object per line.
- Add config options under `logging` to enable/disable file sink(s) and set output paths.
- Ensure file sink writes are reliable and do not panic/unwrap under IO errors (errors must be handled and surfaced via `WorkerError` or structured app logs).
- Ensure the sink design remains compatible with future OTEL exporter work (schema already OTEL-ready).

**Context from research / existing code:**
- Logging core types and sink trait live in `src/logging/mod.rs`:
  - `LogRecord`, `LogSource`, `LogSink`, `LogHandle`, `JsonlStderrSink`.
- Runtime bootstrap currently happens in one place: `src/runtime/node.rs` calls `crate::logging::bootstrap(&cfg)`.
- Postgres ingestion and process-worker stdout/stderr capture already emit records through `LogHandle`.

**Expected outcome:**
- When configured, the runtime writes the same JSONL records to a configured file (optionally in addition to stderr).
- File sink behavior is deterministic and test-covered:
  - creates parent directories as needed
  - appends safely or truncates per config
  - supports a basic rotation/retention policy if configured (or explicitly defers rotation to a follow-up task)

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] Add file sink config to `src/config/schema.rs` (including `Partial*` variants where appropriate)
- [x] Add defaults in `src/config/defaults.rs` and validation in `src/config/parser.rs`
- [x] Implement `JsonlFileSink` (and optional multi-sink fanout) in `src/logging/`
- [x] Update `crate::logging::bootstrap(&RuntimeConfig)` to wire file sinks from config without adding a second bootstrap path
- [x] Add unit tests for file sink behavior (create/append/error paths) without skipping on platforms
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

---

## Plan

### 0) Ground rules / invariants
- Do **not** introduce any `unwrap()`, `expect()`, or `panic!()` in Rust code (repo lint rule).
- File sink must be deterministic and cross-platform in tests (no chmod/ACL-based permission assumptions).
- Keep bootstrap centralized: only `src/runtime/node.rs` calls `crate::logging::bootstrap(&cfg)`; do not add an alternate bootstrap entrypoint.
- Optional sinks must not become an availability dependency: if one sink fails to write but another succeeds, logging should still return `Ok(())` (and emit an out-of-band diagnostic).

### 1) Config schema additions (`src/config/schema.rs`)
- [x] Extend `LoggingConfig` with an explicit sink config section (future OTEL/exporters friendly):
  - `logging.sinks.stderr.enabled: bool` (default: `true`)
  - `logging.sinks.file.enabled: bool` (default: `false`)
  - `logging.sinks.file.path: Option<PathBuf>` (required when `enabled=true`)
  - `logging.sinks.file.mode: FileSinkMode` (append/truncate)
- [x] Add new structs/enums (all `#[serde(deny_unknown_fields)]` where applicable):
  - `LoggingSinksConfig`, `StderrSinkConfig`, `FileSinkConfig`, `FileSinkMode`
  - `FileSinkMode` should deserialize from TOML with `#[serde(rename_all = "lowercase")]` (`append`, `truncate`)
- [x] Add matching `Partial*` variants:
  - `PartialLoggingConfig` gains `sinks: Option<PartialLoggingSinksConfig>`
  - `PartialLoggingSinksConfig` contains optional `stderr`/`file`
  - `PartialFileSinkConfig` mirrors `enabled/path/mode`

### 2) Defaults merge (`src/config/defaults.rs`)
- [x] Add defaults constants:
  - `DEFAULT_LOGGING_SINK_STDERR_ENABLED: true`
  - `DEFAULT_LOGGING_SINK_FILE_ENABLED: false`
  - `DEFAULT_LOGGING_SINK_FILE_MODE: Append`
- [x] Update `apply_defaults(...)` to populate `LoggingConfig.sinks` using the established `*_raw.and_then(...)` pattern.
- [x] Update `src/config/defaults.rs` tests to assert sink defaults and that partial overrides work.

### 3) Validation (`src/config/parser.rs`)
- [x] Add validation rules:
  - If `logging.sinks.file.enabled == true` then `logging.sinks.file.path` must be `Some(...)` and non-empty.
  - If `logging.sinks.file.path` is `Some(path)` validate it is non-empty (`validate_non_empty_path`).
  - Keep validation filesystem-agnostic (do not attempt to open/create the file here).
- [x] Update `src/config/parser.rs` tests:
  - baseline config remains valid with defaults
  - invalid config when file sink enabled but `path` missing/empty
  - valid config when file sink enabled and `path` provided

### 3.5) Early compile gate
- [x] Run `make check` right after schema/default/parser changes to catch struct-literal breakages early (before implementing sinks).

### 4) Logging sinks implementation (`src/logging/`)
- [x] Implement `JsonlFileSink`:
  - Construction opens file once and holds `Mutex<LineWriter<File>>` (flush-on-newline for deterministic writes).
  - Create parent directories via `std::fs::create_dir_all(parent)` when `parent` is non-empty.
  - Open with `OpenOptions` according to mode:
    - `Append`: `create(true) + write(true) + append(true)`
    - `Truncate`: `create(true) + write(true) + truncate(true)`
  - `emit(...)` serializes record to JSON, writes `line + "\n"`, and maps all failures to `LogError` (no panics/unwraps).
  - Include path in IO error messages for debugging.
- [x] Implement `FanoutSink` (multi-sink):
  - Holds `Vec<(String, Arc<dyn LogSink>)>` (label used in aggregated errors; e.g. `"stderr"`, `"file:/path"`).
  - `emit` attempts all sinks; returns `Ok(())` if **any** sink succeeded, otherwise returns a single aggregated `LogError::SinkIo(...)`.
  - Additionally: on any sink failure, write a best-effort diagnostic to raw stderr (direct `std::io::stderr()` write) so failures aren’t silently swallowed when upstream uses `let _ = log.emit(...)`.
    - Keep this non-recursive (do not call back into the sink chain).
    - Add a simple reentrancy guard (e.g. atomic bool) so a failing stderr write path cannot loop.
    - Prefer JSONL-formatted diagnostic if feasible, but plain text is acceptable as long as it’s reliable and cannot recurse.

### 5) Bootstrap wiring (`src/logging/mod.rs`, `src/runtime/node.rs`)
- [x] Update `crate::logging::bootstrap(&RuntimeConfig)` to build sinks based on config:
  - If `stderr.enabled`: include `JsonlStderrSink`.
  - If `file.enabled`: require `path` and include `JsonlFileSink`.
  - If both disabled: use `NullSink` (intentional silence).
- [x] Make bootstrap **fail-fast** when file sink is enabled but cannot be constructed (dir create/open failure):
  - Change signature to `pub(crate) fn bootstrap(cfg: &RuntimeConfig) -> Result<LoggingSystem, LogBootstrapError>`.
  - Update `src/runtime/node.rs` to map this to `RuntimeError` with a clear message (do not `unwrap`), without pulling runtime/worker error types into `src/logging/`.
  - Rationale: this is the only place we can reliably “surface” misconfiguration/IO issues to the operator.

### 6) Unit tests for file sink (`src/logging/...`)
- [x] Add tests covering:
  - creates parent dirs and writes one JSONL line
  - append mode preserves existing content
  - truncate mode replaces existing content
  - deterministic IO failure:
    - create a file at `<tmp>/not_a_dir` then set sink path to `<tmp>/not_a_dir/app.jsonl` and assert `Err(LogError::SinkIo(_))`
- [x] Add unit tests for `FanoutSink` semantics:
  - if one sink fails but another succeeds: returns `Ok(())` and emits an out-of-band diagnostic
  - if all sinks fail: returns `Err(LogError::SinkIo(_))`
- [x] Avoid chmod/permission-based tests; do not skip tests per platform.

### 7) Repo-wide fixture updates (compile fixes)
- [x] Update every `LoggingConfig { ... }` literal to include the new `sinks: ...` field (search hits include):
  - `src/runtime/node.rs`, `src/config/parser.rs`, `src/logging/postgres_ingest.rs`,
  - `src/worker_contract_tests.rs`, `src/ha/*`, `src/dcs/*`, `src/api/*`, `src/debug_api/*`,
  - `src/test_harness/**` (real e2e fixtures compile under `--all-targets`).
- [x] Update any `PartialLoggingConfig { ... }` literals to include optional `sinks` as needed.

### 8) Verification gates (must be 100% green)
- [x] Run `make check`
- [x] Run `make test`
- [x] Run `make test-long`
- [x] Run `make lint`

### 9) Follow-ups / explicit deferrals
- Rotation/retention for the *new* unified JSONL file sink is deferred (outside this task) unless it is trivial to add without risking regressions. This task will provide append/truncate only.

NOW EXECUTE
