## Task: Rewrite Logging Around Private Typed Events And JSON Postgres Defaults <status>not_started</status> <passes>false</passes>

<priority>low</priority>

<description>
**Goal:** Refactor the logging subsystem so the rest of the codebase can no longer construct ad-hoc log records by mutating free-form field maps. Logging must become a narrow typed boundary: non-logging modules may only emit fully typed log events defined as enums or typed sub-enums, and the logging package itself must own all translation from those typed events into serialized log records. The higher-order goal is to make logging compiler-driven and boring: impossible to bypass with raw `add_field`-style mutation, impossible to couple the rest of the codebase to logging internals, and simple enough that PostgreSQL log ingestion and internal application logging are both understandable and maintainable.

**Decisions already made from user discussion:**
- This task must live in a non-operator-ergonomics story. Do not move it under `.ralph/tasks/story-ctl-operator-experience/`.
- The task is intentionally low priority.
- Logging is currently considered too messy because there are many `emit_*event`, `AppEvent::new(...)`, `fields_mut()`, `StructuredFields::new()`, `append_json_map(...)`, and raw-record builder call sites spread across the codebase.
- The final boundary must be extremely reduced: other modules may send typed events to logging, but they must not assemble raw field sets or generic records themselves.
- “Fully typed” is literal here: events must be represented as enums or typed sub-enums with typed payload structs where needed, not as arbitrary stringly headers plus mutable key/value bags.
- Raw record internals must be private to the logging implementation. If something still needs a raw record shape internally for sink emission or parsing external input, that shape must stay entirely inside `src/logging` and must not leak into the rest of the crate.
- Direct `add_field` / `fields_mut()` / `StructuredFields` style mutation from non-logging code is not acceptable in the final design.
- PostgreSQL log ingestion must be simplified into clearer responsibilities, with separate code for:
  - reading tailed PostgreSQL log files
  - cleaning / normalizing / parsing PostgreSQL log lines into typed events
  - emitting those typed events through the logger
- Managed PostgreSQL configuration must enable JSON logging by default. This is not meant to remain a user-tunable behavior toggle.
- Even with JSON logging enabled, ingestion must still pick up both JSON log files and plain `.log` files because PostgreSQL / helper output such as archive-command-related output can still land in `.log` files. The logging pipeline must handle both.
- The task is an implementation/refactor task, not a design note. It must carry the cleanup through module privacy, call-site rewrites, PostgreSQL defaults, tests, and validation.

**Concrete repo context from research:**
- `src/logging/mod.rs` currently re-exports and exposes logging building blocks crate-wide:
  - `AppEvent`
  - `AppEventHeader`
  - `StructuredFields`
  - `PostgresLineRecordBuilder`
  - `RawRecordBuilder`
  - `SubprocessLineRecord`
- `src/logging/event.rs` currently models application logs as:
  - `AppEventHeader { name, domain, result }`
  - `AppEvent { header, severity, message, fields }`
  - mutable `StructuredFields` with `insert`, `insert_optional`, `insert_serialized`, and `append_json_map`
- That shape is not strictly typed enough because arbitrary callers can create an event and then mutate a loose field bag.
- `src/logging/raw_record.rs` currently exposes raw-record construction helpers used to build free-form records for subprocess and PostgreSQL log lines.
- `src/logging/postgres_ingest.rs` currently mixes multiple responsibilities in one large file:
  - file tailing / polling state
  - rate limiting
  - line normalization
  - JSON/plain parsing
  - record building
  - ingest-worker event emission
  - real-binary integration tests
- `src/postgres_managed_conf.rs` renders the authoritative managed `postgresql.conf`, but the current managed config writer does not force PostgreSQL JSON logging defaults.
- `src/postgres_managed.rs` materializes that config and already has real PostgreSQL config tests.
- The current codebase has many non-logging call sites that construct `AppEvent` directly and mutate free-form fields:
  - `src/api/worker.rs`
  - `src/dcs/worker.rs`
  - `src/pginfo/worker.rs`
  - `src/process/worker.rs`
  - `src/runtime/node.rs`
  - `src/logging/postgres_ingest.rs`
- The PostgreSQL ingest code already contains tests proving ingestion of both JSON and stderr/plain log files from a real PostgreSQL binary, including `ingests_jsonlog_and_stderr_files_from_real_postgres` in `src/logging/postgres_ingest.rs`. That means the refactor should preserve and strengthen real-binary coverage rather than downgrading it.
- Research also found explicit JSON-log-related expectations already present in tests:
  - `logging_collector = on`
  - `log_destination = 'jsonlog,stderr'`
  Those expectations should become default managed behavior, not just test-only setup.

**Required architectural target:**
- `src/logging` must become the only place that knows how log records are serialized, how sink attributes are assembled, and how external PostgreSQL/subprocess lines are turned into output records.
- Non-logging modules must not import or mutate generic logging field bags.
- Non-logging modules must not construct raw log records, raw PostgreSQL record builders, or ad-hoc app-event headers.
- The externally usable boundary should become one intentionally small API such as:
  - a typed event enum or a small set of typed domain event enums
  - one logger handle / emit method that accepts only those typed events
  - optionally one small trait implemented by typed event enums inside `src/logging`
- The rest of the crate may describe facts only by constructing typed event values. It may not describe facts by pushing loose string keys into a map.

**Required type-system direction:**
- Replace free-form `AppEventHeader { name, domain, result }` plus mutable `StructuredFields` for application logs with typed enums and typed payloads.
- It is acceptable and preferred to introduce a top-level logging enum with nested domain enums, for example an application event enum with sub-enums for API, process, runtime, DCS, PG info, and ingest events.
- Use normal Rust types for event payloads. If an event needs additional data, that data should be expressed as named fields on a payload struct or enum variant, not via `insert("some_key", value)`.
- If some output attributes still need to become key/value JSON at sink time, the mapping from typed event payload to attributes must happen inside `src/logging` only.
- If raw external PostgreSQL lines cannot be fully parsed, they must still surface as typed logging events such as “postgres plain line” / “postgres json line” / “postgres parse failure” rather than as generic raw record construction from outside the logging boundary.

**Required privacy outcome:**
- `src/logging/mod.rs` must stop re-exporting generic event-construction and raw-record-construction types to the rest of the crate.
- `AppEvent`, `AppEventHeader`, `StructuredFields`, `RawRecordBuilder`, and `PostgresLineRecordBuilder` should either be deleted or reduced to private internal implementation details.
- The rest of the crate should not be able to call methods like `fields_mut()`, `append_json_map(...)`, `insert_optional(...)`, or `insert_serialized(...)`.
- Logging internals should be private by Rust module privacy, not by convention alone.

**Required PostgreSQL logging outcome:**
- Managed PostgreSQL configuration must default to JSON logging with collector enabled.
- The expected managed settings are:
  - `logging_collector = on`
  - `log_destination = 'jsonlog,stderr'`
- This behavior should be authored by the managed config path in `src/postgres_managed_conf.rs` / `src/postgres_managed.rs`, not left to example files or manual operator tuning.
- The ingest worker must continue to pick up both JSON log files and plain `.log` / stderr log files after that change.
- Do not make JSON logging a new optional compatibility mode. This repo is greenfield and should converge on one default behavior.

**Required PostgreSQL ingest simplification target:**
- Split `src/logging/postgres_ingest.rs` into smaller files or modules with clear ownership boundaries.
- At minimum, separate:
  - file polling / tailing state
  - PostgreSQL line normalization / cleanup
  - parsing plain vs JSON PostgreSQL lines
  - conversion into typed logging events
  - ingest worker orchestration / rate limiting
- Preserve the current useful behavior, but reduce file size, reduce mixed concerns, and make the pipeline legible.
- If shared helpers are needed for PostgreSQL and subprocess ingestion, they should remain inside `src/logging`, not become public crate utilities.

**Exact things that must become impossible outside `src/logging`:**
- constructing `AppEventHeader` directly
- calling `AppEvent::new(...)` directly from non-logging modules
- calling `event.fields_mut()`
- creating `StructuredFields` in non-logging code
- calling `append_json_map`, `insert`, `insert_optional`, or `insert_serialized` from non-logging code
- constructing `RawRecordBuilder`
- constructing `PostgresLineRecordBuilder`
- depending on logging attribute key strings as part of normal business logic

**Suggested implementation direction for the application-event side:**
- Introduce typed domain event enums close to the domains that emit them, or in one dedicated typed-events module under `src/logging`.
- Examples of the current direct-emitter domains that need migration:
  - API worker events from `src/api/worker.rs`
  - DCS worker events from `src/dcs/worker.rs`
  - PG info worker events from `src/pginfo/worker.rs`
  - process worker events from `src/process/worker.rs`
  - runtime startup events from `src/runtime/node.rs`
  - ingest-worker operational events from `src/logging/postgres_ingest.rs`
- A reasonable end state is that each of those modules emits typed variants and the logger handle performs the serialization internally.
- The implementation may choose whether typed event enums live in logging or per-domain modules, but the boundary rule is fixed: only typed events cross into the logger.

**Scope:**
- Refactor `src/logging/mod.rs` to expose a narrow typed logging API and hide generic record-building internals.
- Refactor `src/logging/event.rs` and/or replace it with typed event definitions and internal serialization logic.
- Refactor or privatize `src/logging/raw_record.rs`.
- Split and simplify `src/logging/postgres_ingest.rs`.
- Update managed PostgreSQL config rendering/materialization in `src/postgres_managed_conf.rs` and `src/postgres_managed.rs` so JSON logging is the default managed behavior.
- Rewrite logging call sites in `src/api/worker.rs`, `src/dcs/worker.rs`, `src/pginfo/worker.rs`, `src/process/worker.rs`, `src/runtime/node.rs`, and any other remaining emitter modules discovered during the refactor.
- Update logging tests, PostgreSQL ingest tests, and managed PostgreSQL config tests to reflect the new typed/event-private architecture and new PostgreSQL logging defaults.
- Remove dead logging code and compatibility shims instead of preserving them.

**Out of scope:**
- Do not redesign product behavior unrelated to logging just because those modules are touched.
- Do not weaken or skip real-binary PostgreSQL logging tests.
- Do not add a broad end-user compatibility layer for old logging internals. This repo is greenfield and should delete the old surface.

**Context from research:**
- `src/process/worker.rs` is the heaviest current example of the problem: it directly builds app events, mutates structured fields repeatedly, and also appends runtime JSON maps. That file is a good canary for whether the new typed boundary is actually strict.
- `src/api/worker.rs`, `src/dcs/worker.rs`, `src/pginfo/worker.rs`, and `src/runtime/node.rs` all currently construct `AppEvent` directly and mutate fields. Those modules should become simpler after the refactor if the logger boundary is correct.
- `src/logging/postgres_ingest.rs` currently contains both ingest operational events and PostgreSQL log-line record construction. That is a strong sign that the file should be decomposed.
- The current tests in `src/logging/mod.rs` verify app-event encoding behavior around headers and fields. Those tests will likely need to be rewritten around typed event serialization rather than generic mutable fields.
- The managed PostgreSQL config tests in `src/postgres_managed.rs` currently assert many generated settings and are the right place to lock in the new default logging settings.
- Keep the “both JSON and plain `.log` files are ingested” behavior explicit in tests after the refactor, because archive-command and helper output still make the plain log path relevant.

**Expected outcome:**
- The codebase has one small typed interface for emitting logs.
- Other modules no longer know how logging attributes are encoded.
- Free-form field mutation is gone from non-logging code.
- Logging internals are private and substantially smaller in surface area.
- PostgreSQL managed config always enables JSON logging by default.
- PostgreSQL ingest remains capable of consuming both JSON log files and plain `.log` files, but the implementation is split into understandable responsibilities.
- Real-binary tests still prove the logging pipeline works end to end.

</description>

<acceptance_criteria>
- [ ] Create or rename logging modules so the public/crate-visible logging boundary is narrow and typed, with `src/logging/mod.rs` no longer exposing generic mutable event/field builders to the rest of the crate.
- [ ] Remove or privatize direct use of `AppEvent`, `AppEventHeader`, `StructuredFields`, `RawRecordBuilder`, and `PostgresLineRecordBuilder` from non-logging modules; verify the rest of the crate cannot call `fields_mut()` or equivalent ad-hoc field mutation APIs.
- [ ] Replace application log emission in `src/api/worker.rs`, `src/dcs/worker.rs`, `src/pginfo/worker.rs`, `src/process/worker.rs`, `src/runtime/node.rs`, and any remaining discovered emitters with fully typed enum-based events or typed sub-enums handled by the logger.
- [ ] Refactor `src/logging/event.rs` and related serialization code so conversion from typed event values into output attributes happens only inside `src/logging`.
- [ ] Refactor or replace `src/logging/raw_record.rs` so raw-record details stay private to logging internals and are not a general-purpose crate interface.
- [ ] Split `src/logging/postgres_ingest.rs` into clearer responsibility-specific files or modules covering file polling, normalization/cleanup, parsing, typed-event conversion, and worker orchestration/rate limiting.
- [ ] Preserve and clarify PostgreSQL ingest behavior for both JSON log files and plain `.log` / stderr files, with tests explicitly covering both paths after the refactor.
- [ ] Update `src/postgres_managed_conf.rs` and `src/postgres_managed.rs` so managed PostgreSQL config defaults to `logging_collector = on` and `log_destination = 'jsonlog,stderr'`, with tests locking this in.
- [ ] Rewrite logging-focused tests in `src/logging/mod.rs`, `src/logging/postgres_ingest.rs`, `src/postgres_managed.rs`, and any other affected files so they validate the new typed/private architecture instead of the old mutable field-bag API.
- [ ] Remove dead compatibility code, obsolete helper APIs, and old tests that only exist for the deleted logging surface.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
