---
## Task: Build Unified Structured Logging Pipeline With Postgres/Binary Ingestion <status>not_started</status> <passes>false</passes>

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
- [ ] A single logging setup point exists in runtime wiring/config (no dual bootstrap paths)
- [ ] Logging config is part of full app config and centrally governs behavior
- [ ] Default output sink is JSONL to `stderr`
- [ ] Every emitted record includes: severity/level, hostname, unix timestamp ms
- [ ] Every emitted record includes source attribution keys (producer + origin/parser channel)
- [ ] Structured field naming and level semantics are OTEL-ready (schema/key compatibility), without implementing OTEL exporters
- [ ] Unified stream includes application logs plus postgres server logs plus helper-binary logs
- [ ] Postgres JSON logger lines are ingested and normalized into unified structured records
- [ ] Postgres plain `.log` startup lines are ingested and normalized (or safely degraded) into unified structured records
- [ ] Postgres `stderr` lines are ingested and normalized (or safely degraded) into unified structured records
- [ ] `archive_command` output is captured; full message content is preserved in structured logs
- [ ] Outputs from `pg_rewind`, `pg_ctl`, `pg_basebackup`, and other invoked postgres tools are captured from stdout/stderr and structured
- [ ] Parse failures never drop logs; original raw line is preserved and parse failure metadata is emitted
- [ ] Postgres log auto-collection lifecycle includes auto cleanup/rotation handling
- [ ] Real postgres integration/e2e tests verify ingestion of JSON logger + `.log` + `stderr` behavior
- [ ] Tests verify archive_command output capture and full payload retention
- [ ] Tests verify helper-binary output capture into unified stream
- [ ] Large unit/fixture parser tests cover valid, mixed, and malformed inputs with lossless fallback behavior
- [ ] Follow-up/backlog task is created for file sink support (explicitly out of scope here)
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] BDD features pass (covered by `make test`).
</acceptance_criteria>
