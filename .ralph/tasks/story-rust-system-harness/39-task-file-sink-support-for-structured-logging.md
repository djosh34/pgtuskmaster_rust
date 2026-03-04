---
## Task: Add File Sink Support For Unified Structured Logging <status>not_started</status> <passes>false</passes>

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
- [ ] Add file sink config to `src/config/schema.rs` (including `Partial*` variants where appropriate)
- [ ] Add defaults in `src/config/defaults.rs` and validation in `src/config/parser.rs`
- [ ] Implement `JsonlFileSink` (and optional multi-sink fanout) in `src/logging/`
- [ ] Update `crate::logging::bootstrap(&RuntimeConfig)` to wire file sinks from config without adding a second bootstrap path
- [ ] Add unit tests for file sink behavior (create/append/error paths) without skipping on platforms
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

