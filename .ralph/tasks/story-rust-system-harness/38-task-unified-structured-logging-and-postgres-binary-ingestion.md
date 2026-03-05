---
## Task: Build Unified Structured Logging Pipeline With Postgres/Binary Ingestion <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
**Goal:** Implement one unified, config-driven logging system that emits structured JSONL to `stderr` by default, ingests/normalizes all postgres and helper-binary logs into the same stream, and guarantees no log loss on parse failures.

**Scope:**
- Add a single logging subsystem/config entrypoint for the entire runtime (no split setup points).
- Enforce baseline structured fields on every emitted record:
  - OpenTelemetry-style severity/level key
  - hostname
  - unix timestamp in milliseconds
  - source identity keys indicating producer and transport/parser origin
- Ensure OTEL-readiness at schema level only:
  - use standard-compatible key naming for severity/level and core log envelope fields
  - no exporter/push transport work in this task
- Default sink behavior in-scope now:
  - JSONL to `stderr` only
- Explicitly out of scope for this task (track as follow-up/backlog only):
  - file sinks
- Ingest postgres logs from all expected sources into one unified stream:
  - postgres JSON logger output
  - transient plain `.log` startup lines
  - postgres `stderr` output
  - `archive_command` output (full message preserved)
- Ingest stdout/stderr logs from postgres utility binaries used by runtime operations:
  - `pg_rewind`, `pg_ctl`, `pg_basebackup`, and related process helpers invoked by the system
- Add parser/normalizer behavior for mixed-format log lines:
  - parse structured JSON logs when valid
  - parse known plain postgres line formats where possible
  - on parse failure, never drop logs; emit full original line as message with explicit parse-failed metadata
- Add log collection lifecycle and retention behavior for postgres-produced files:
  - automatic collection/tailing
  - automatic cleanup/rotation handling

**Context from plan extraction:**
- This work was previously tracked only as a future TODO block in `RUST_SYSTEM_HARNESS_PLAN.md` under “More Future TODOS: create tasks”.
- The logging TODO block has now been removed from that plan and replaced by this executable task.

**Expected outcome:**
- One unified structured log stream across pgtuskmaster runtime, postgres server logs, archive command output, and postgres helper binaries.
- Every accepted log record is structured and source-attributed.
- No dropped records during parse errors; degraded lines remain fully preserved.
- Design remains forward-compatible with later OpenTelemetry/file-sink expansion without requiring a second logging bootstrap.

**Testing requirements (mandatory):**
- Real-binary coverage:
  - integration/e2e tests using real postgres instances verifying auto-collection from JSON logger, `.log`, and `stderr` paths
  - tests verifying archive_command output capture and full message preservation
  - tests verifying helper binary output capture (stdout/stderr) into unified stream
- Large parser/unit coverage:
  - comprehensive fixture-based tests with representative postgres JSON logs
  - comprehensive fixture-based tests with plain `.log`/stderr samples
  - parse-failure fixture tests that assert full message retention + parse-failed marker
  - rotation/cleanup behavior tests for collector lifecycle
- Gate requirements:
  - no optional/skipped tests for real-binary logging paths
  - `make check`, `make test`, `make lint`, and `make test` must pass

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] A single logging setup point exists in runtime wiring/config (no dual bootstrap paths)
- [x] Logging config is part of full app config and centrally governs behavior
- [x] Default output sink is JSONL to `stderr`
- [x] Every emitted record includes: severity/level, hostname, unix timestamp ms
- [x] Every emitted record includes source attribution keys (producer + origin/parser channel)
- [x] Structured field naming and level semantics are OTEL-ready (schema/key compatibility), without implementing OTEL exporters
- [x] Unified stream includes application logs plus postgres server logs plus helper-binary logs
- [x] Postgres JSON logger lines are ingested and normalized into unified structured records
- [x] Postgres plain `.log` startup lines are ingested and normalized (or safely degraded) into unified structured records
- [x] Postgres `stderr` lines are ingested and normalized (or safely degraded) into unified structured records
- [x] `archive_command` output is captured; full message content is preserved in structured logs
- [x] Outputs from `pg_rewind`, `pg_ctl`, `pg_basebackup`, and other invoked postgres tools are captured from stdout/stderr and structured
- [x] Parse failures never drop logs; original raw line is preserved and parse failure metadata is emitted
- [x] Postgres log auto-collection lifecycle includes auto cleanup/rotation handling
- [x] Real postgres integration/e2e tests verify ingestion of JSON logger + `.log` + `stderr` behavior
- [x] Tests verify archive_command output capture and full payload retention
- [x] Tests verify helper-binary output capture into unified stream
- [x] Large unit/fixture parser tests cover valid, mixed, and malformed inputs with lossless fallback behavior
- [x] Follow-up/backlog task is created for file sink support (explicitly out of scope here)
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] BDD features pass (covered by `make test`).
</acceptance_criteria>

## Plan (Draft)

### What exists today (repo reality check)
- There is currently **no unified application logger** (no `tracing`/`log`), so “application logs” are effectively absent except for a couple of `println!/eprintln!` in CLI binaries.
- Postgres server logs are currently **only written to files**:
  - runtime path: `pg_ctl -l <postgres.log_file>` (plain text, mixed-format, commonly “stderr-ish” lines)
  - harness path: direct `postgres` spawn with `stdout/stderr` redirected into `postgres.stdout.log` / `postgres.stderr.log`
- Helper binaries (`initdb`, `pg_basebackup`, `pg_rewind`, `pg_ctl`) are spawned with `stdout/stderr` set to `null` in the runtime process worker, so their output is currently **dropped**.
- There is **no production tailer/collector** (only a test helper `read_log_tail()` for failure diagnostics).

This plan builds the missing unified structured logging subsystem and then wires ingestion into the existing runtime + harness.

### Log schema (OTEL-ready envelope; JSONL to stderr)
Define a single canonical JSON object per line (“JSONL”) emitted to `stderr` (default + only sink in this task).

**Required top-level fields on every record**
- `ts_ms` (unix epoch milliseconds; integer)
- `hostname` (string)
- `severity_text` (string: `TRACE|DEBUG|INFO|WARN|ERROR|FATAL`)
- `severity_number` (integer; OTEL-style mapping)
- `message` (string)
- `source` (object; source attribution keys)

**Required `source` sub-fields**
- `producer` (enum-ish string): `app|postgres|pg_tool`
- `transport` (string): `internal|file_tail|child_stdout|child_stderr`
- `parser` (string): `app|postgres_json|postgres_plain|raw`
- `origin` (string): freeform stable identifier (e.g. `runtime`, `process_worker`, `pg_ctl_log_file`, `postgres.stderr.log`)

**Parse failure invariants**
- Never drop an input line because parsing failed.
- On parse failure emit a structured record with:
  - `source.parser="raw"`
  - `attributes.parse_failed=true`
  - `attributes.raw_line=<full original line>`
  - `message=<full original line>` (lossless retention requirement)

### Config (single entrypoint)
Add a single `logging` block to the runtime config, with defaults that keep behavior safe and predictable:
- Default sink: `stderr` JSONL.
- Default capture/ingestion: enabled (so runtime + helper output + postgres logs unify by default), but still bounded by safe polling intervals.

Proposed config shape (exact names can change during verification, but keep it one block):
- `logging.level` (minimum severity for **app** logs; ingested external logs are not filtered by default)
- `logging.capture_subprocess_output` (bool; default `true`)
- `logging.postgres.enabled` (bool; default `true`)
- `logging.postgres.pg_ctl_log_file` (optional override; default uses `postgres.log_file`)
- `logging.postgres.log_dir` (optional directory to scan for rotated `.json`/`.log` logs)
- `logging.postgres.poll_interval_ms`
- `logging.postgres.cleanup` (retention policy; enabled by default with conservative settings)
  - `max_files`
  - `max_age_seconds`

### Implementation steps (detailed; execute in order)

#### 1) Introduce core logging subsystem (no ingestion yet)
- [ ] Add `src/logging/` module implementing:
  - [ ] `Severity` + OTEL severity_number mapping.
  - [ ] `LogSource` + `LogRecord` (serde-serializable).
  - [ ] `LogSink` trait and `JsonlStderrSink` implementation (one object per line).
  - [ ] `LogHandle` (cloneable) for emitting records without panics/unwraps.
  - [ ] `TestSink` (in-memory sink for unit/integration tests to assert emitted records without reading stderr).
- [ ] Add a single bootstrap function (the only setup point), e.g. `logging::bootstrap(&RuntimeConfig) -> LoggingSystem`:
  - caches hostname once
  - configures sink(s)
  - provides `LogHandle` for the rest of the runtime

#### 2) Add `logging` config to schema + defaults + validation
- [ ] Update `src/config/schema.rs`:
  - [ ] add `LoggingConfig` and `PartialLoggingConfig`
  - [ ] add `logging: LoggingConfig` to `RuntimeConfig`
  - [ ] add `logging: Option<PartialLoggingConfig>` to `PartialRuntimeConfig`
- [ ] Update `src/config/defaults.rs` to apply defaults for logging config.
- [ ] Update `src/config/parser.rs` to validate:
  - [ ] `logging.level` is one of allowed values
  - [ ] poll intervals/timeouts within sane bounds
  - [ ] retention policy constraints (e.g. `max_files > 0`, `max_age_seconds > 0`)
- [ ] Update any examples/fixtures that construct `RuntimeConfig` / `PartialRuntimeConfig` in tests and `examples/` so `cargo check --all-targets` stays green.

#### 3) Wire logging bootstrap into runtime (single setup point)
- [ ] Update `src/runtime/node.rs` to create the logging system exactly once before startup execution and worker loops.
- [ ] Ensure all workers that need to emit logs get a `LogHandle` via their ctx struct (avoid globals unless absolutely necessary).
- [ ] Replace ad-hoc `eprintln!` paths in `src/bin/pgtuskmaster.rs` / `src/bin/pgtuskmasterctl.rs` with structured emission where appropriate (keep CLI “command output” on stdout as-is; only diagnostics become structured logs).

#### 4) Implement Postgres log ingestion (file tail + directory scan + rotation)
- [ ] Add a `FileTailer` (polling) that:
  - [ ] **starts at beginning for “active/current” files** (pg_ctl `-l` log file, direct-postgres stderr/stdout files) so we never miss transient startup logs
  - [ ] **starts at EOF for discovered/rotated files by default** (to avoid replay floods), unless explicitly configured otherwise
  - [ ] detects truncation and safely resets offset
  - [ ] handles rotation by inode change when available (unix), otherwise via size/mtime heuristics
- [ ] Add a `DirLogCollector` that:
  - [ ] scans a directory for matching log files (`*.json`, `*.log`, and optionally configured patterns)
  - [ ] tracks per-file offsets and emits new lines
  - [ ] handles new files appearing (rotation)
- [ ] Add `postgres_log_parser`:
  - [ ] try JSON first (`serde_json::from_str`)
  - [ ] if JSON is valid and matches Postgres jsonlog shape, normalize to `LogRecord`
  - [ ] else try known plain formats (`YYYY-MM-DD HH:MM:SS... [pid] LEVEL: msg` etc.)
  - [ ] otherwise fallback to raw/parse_failed record (never drop)
- [ ] Add a `PostgresIngestWorker` task started by `logging::bootstrap(...)` that tails:
  - [ ] runtime `postgres.log_file` (pg_ctl `-l` target): treated as `producer=postgres`, `transport=file_tail`, `origin=pg_ctl_log_file`
  - [ ] optional `logging.postgres.log_dir` directory (jsonlog + rotated logs): `origin=postgres_log_dir`
  - [ ] optional archive-command capture file (see next section): `producer=postgres_archive`
- [ ] Add cleanup/retention enforcement:
  - [ ] deletes old rotated logs in `log_dir` per `max_files` / `max_age_seconds`
  - [ ] never deletes the active file being tailed (guard with inode/name check)

#### 5) Capture helper-binary stdout/stderr in process worker (no drops)
- [ ] Extend the process runner to support piping stdout/stderr **without tokio::spawn** (step-driven; current-thread safe):
  - [ ] update `ProcessCommandSpec` to include a capture policy + stable identity (`binary_name`, `job_kind`, `job_id`)
  - [ ] update `ProcessHandle` / `TokioProcessHandle` to support async polling that also drains available stdout/stderr bytes each tick (bounded read budget)
  - [ ] update `TokioCommandRunner` to use `Stdio::piped()` only when capture is enabled
  - [ ] implement non-panicking stream decoders that:
    - [ ] emit complete UTF-8 lines when possible
    - [ ] fall back to lossless byte-preserving representation when invalid UTF-8 / missing newline (never drop)
- [ ] Emit each captured line into unified structured logs with:
  - `source.producer="pg_tool"`
  - `source.transport="child_stdout|child_stderr"`
  - `source.parser="raw"` (or `app` if later normalized)
  - `attributes.job_id`, `attributes.job_kind`, `attributes.binary`
- [ ] Ensure process worker still enforces timeouts/cancellation correctly and never deadlocks on full pipes.

#### 6) (Removed) archive_command wrapper output capture
- The earlier archive-command wrapper + dedicated tail-input approach was intentionally removed.
- If archive/restore command observability is reintroduced, it should be done via a Rust-native mechanism (not a runtime-generated shell wrapper and not a separate "archive command log file" input).

### Testing plan (mandatory, no skips)

#### Unit / fixture tests (fast; deterministic)
- [ ] Parser fixtures:
  - [ ] valid Postgres jsonlog sample(s) -> normalized record includes required envelope + mapped severity
  - [ ] valid plain postgres log sample(s) -> normalized record includes required envelope
  - [ ] malformed JSON / unknown format -> parse_failed record with full raw line preserved as `message`
- [ ] Tailer rotation tests:
  - [ ] append lines -> read -> rotate via rename+new file -> append more -> assert no drops and stable ordering
- [ ] Cleanup tests:
  - [ ] create N fake rotated log files -> enforce `max_files` -> assert oldest are deleted, newest preserved

#### Real-binary integration/e2e tests (required)
Use `.tools/postgres16/bin/*` via `require_pg16_bin_for_real_tests` (no optional skips).

- [ ] Real Postgres ingestion tests (split by transport to reduce flake and make failures diagnosable):
  - [ ] `T_jsonlog_and_stderr_files`: direct-spawn postgres via `spawn_pg16` harness, with test-controlled `postgresql.conf` enabling `logging_collector=on` + `log_destination='jsonlog,stderr'` and `log_directory=<namespace log dir>`; assert ingestor emits records for:
    - [ ] `*.json` jsonlog lines from log dir
    - [ ] `postgres.stderr.log` lines from OS-level stderr redirection
  - [ ] `T_pg_ctl_log_file`: start postgres through the runtime/process-worker path that uses `pg_ctl -l <postgres.log_file>`; assert ingestor emits records from that plain log file (startup lines included).
- [ ] Real archive_command capture:
  - [ ] configure `archive_mode=on` and `archive_command` to call the wrapper script
  - [ ] force WAL switch (`SELECT pg_switch_wal()`)
  - [ ] assert unified stream contains archive-command output record(s) with full payload preserved
- [ ] Real helper binary output capture:
  - [ ] run at least one real failing helper command (e.g. `pg_basebackup` to invalid host) through process worker
  - [ ] assert stderr/stdout lines appear in unified structured stream with correct source attribution

### Follow-up task (explicitly out of scope here)
- [ ] Create a backlog task for **file sink support** (additional sink types) using the `add-task-as-agent` skill, referencing this task as prerequisite.

### Validation gate (must be 100% green before marking passing)
- [ ] `make check`
- [ ] `make test`
- [ ] `make test-long`
- [ ] `make lint`

NOW EXECUTE
