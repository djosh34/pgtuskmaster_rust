Answer using only the information in this prompt.
Do not ask to inspect files, browse, run tools, or fetch more context.
If required information is missing, say exactly what is missing.

You are writing prose for an mdBook documentation page.
Return only markdown for the requested page body.
Do not include analysis, prefaces, scratch notes, or code fences.
Use ASCII punctuation only.
Do not use em dashes.
Do not invent facts.

[Task]
- Draft fresh prose for `docs/src/reference/logging.md`.

[Page goal]
- Describe the logging machinery implemented in `src/logging`.
- Keep the page in Diataxis reference form: cognition plus application.
- Describe and only describe.

[Audience]
- Engineers consulting the logging module while working on the runtime.

[mdBook context]
- This page lives at `docs/src/reference/logging.md`.
- Use short factual sections and tables.
- Mirror the structure of the machinery.

[Diataxis guidance]
- Reference should describe and only describe.
- Keep the page neutral, factual, and structured by the code surface.
- Do not turn the page into procedures, rationale, advice, or troubleshooting.

[Required structure]
- Title: `# Logging Reference`
- Short opening paragraph
- `## Module surface`
- `## Record model`
- `## Application events`
- `## Raw record builders`
- `## Sink bootstrap and sinks`
- `## Log handle and tracing backend`
- `## File tailing`
- `## PostgreSQL ingest worker`
- `## Cleanup`

[Verified facts]
- `src/logging/mod.rs` declares private modules `event` and `raw_record`, and crate-visible modules `postgres_ingest` and `tailer`.
- `mod.rs` re-exports `AppEvent`, `AppEventHeader`, `StructuredFields`, `PostgresLineRecordBuilder`, `RawRecordBuilder`, `SubprocessLineRecord`, and `SubprocessStream`.
- `SeverityText` variants are `Trace`, `Debug`, `Info`, `Warn`, `Error`, and `Fatal`.
- `SeverityText::number()` maps those variants to `1`, `5`, `9`, `13`, `17`, and `21`.
- `From<crate::config::LogLevel>` maps matching log-level variants one-to-one.
- `LogProducer` serializes as snake_case variants `app`, `postgres`, and `pg_tool`.
- `LogTransport` serializes as snake_case variants `internal`, `file_tail`, `child_stdout`, and `child_stderr`.
- `LogParser` serializes as snake_case variants `app`, `postgres_json`, `postgres_plain`, and `raw`.
- `LogSource` fields are `producer`, `transport`, `parser`, and `origin`.
- `LogRecord` fields are `timestamp_ms`, `hostname`, `severity_text`, `severity_number`, `message`, `source`, and `attributes`.
- `LogRecord::new` sets `severity_number` from `severity_text.number()` and initializes `attributes` as an empty `BTreeMap`.
- `attributes` is skipped during serialization when empty.
- `AppEventHeader` stores `name`, `domain`, and `result`.
- `StructuredFields` stores ordered `(String, StructuredValue)` pairs in a `Vec`.
- `StructuredFields::append_json_map` appends values from a `BTreeMap<String, serde_json::Value>` as `StructuredValue::Json`.
- `StructuredFields::insert` appends one key/value pair when the value implements `Into<StructuredValue>`.
- `StructuredFields::insert_optional` appends only when the option is `Some`.
- `StructuredFields::insert_serialized` serializes a value with `serde_json::to_value` and appends it as `StructuredValue::Json`.
- `StructuredFields::into_attributes` consumes the ordered fields and writes them into a `BTreeMap<String, Value>`.
- `StructuredValue` variants are `Bool`, `I64`, `U64`, `String`, and `Json`.
- `AppEvent` stores `header`, `severity`, `message`, and `fields`.
- `AppEvent::into_record` creates a `LogRecord` with producer `App`, transport `Internal`, parser `App`, and the supplied origin. It also inserts `event.name`, `event.domain`, and `event.result` into the attributes map.
- `RawRecordBuilder` stores `severity`, `message`, `source`, and `StructuredFields`.
- `RawRecordBuilder::with_fields` replaces the stored fields.
- `RawRecordBuilder::into_record` builds a `LogRecord` and converts stored fields into attributes.
- `SubprocessStream` variants are `Stdout` and `Stderr`.
- `SubprocessStream::severity()` maps `Stdout` to `Info` and `Stderr` to `Warn`.
- `SubprocessStream::transport()` maps `Stdout` to `ChildStdout` and `Stderr` to `ChildStderr`.
- `SubprocessLineRecord` stores producer, origin, job_id, job_kind, binary, stream, and raw bytes.
- `SubprocessLineRecord::into_raw_record` decodes the bytes, uses parser `Raw`, inserts `job_id`, `job_kind`, `binary`, serialized `stream`, and optional `raw_bytes_hex`, and returns a `RawRecordBuilder`.
- `decode_bytes` returns UTF-8 text when decoding succeeds. On invalid UTF-8 it returns message text `non_utf8_bytes_hex={hex}` and sets `raw_bytes_hex` to the same hex string.
- `PostgresLineRecordBuilder` stores producer, transport, and origin.
- `PostgresLineRecordBuilder::build` returns a `RawRecordBuilder` using the supplied parser, severity, message, and fields.
- `LogError` variants are `Json(String)` and `SinkIo(String)`.
- `LogBootstrapError` variants are `Misconfigured(String)` and `SinkInit(String)`.
- `JsonlStderrSink` writes one JSON line per record to stderr behind a mutex.
- `JsonlFileSink::new` rejects an empty path, creates parent directories when needed, opens the file in append or truncate mode according to `crate::config::FileSinkMode`, and stores a `LineWriter<File>` behind a mutex.
- `JsonlFileSink::emit` writes one JSON line per record to the configured path.
- `NullSink` accepts a record and does nothing.
- `FanoutSink` stores labeled sinks. On emit it writes to every sink, prints `fanout sink failure: {label}: {error}` to stderr for each sink error, succeeds if at least one sink succeeded, and returns `LogError::SinkIo` only when every sink failed.
- `LogHandle` fields are `hostname`, `backend`, and `min_app_severity_number`.
- `LogHandle::new` stores the hostname, creates a `TracingBackend`, and stores the minimum app severity number from `SeverityText::number()`.
- `LogHandle::null()` uses hostname `unknown`, a `NullSink`, and minimum severity `Fatal`.
- `LogHandle::emit_app_event` drops events whose severity number is below `min_app_severity_number`. Otherwise it stamps `system_now_unix_millis()` and the stored hostname, converts the event into a record, and emits it.
- `LogHandle::emit_raw_record` stamps `system_now_unix_millis()` and the stored hostname, then emits the built record.
- `LogHandle::emit_record` forwards a record to the tracing backend.
- `system_now_unix_millis()` returns milliseconds since `UNIX_EPOCH`, or `0` if the system clock is before `UNIX_EPOCH`.
- `detect_hostname()` returns the non-empty trimmed `HOSTNAME` environment variable, otherwise `unknown`.
- `TracingBackend::emit` uses a thread-local active-record guard, dispatches a tracing event with target `pgtuskmaster::logging::record`, and returns the sink result recorded by `TracingRecordLayer`.
- Nested tracing-backed emission returns `LogError::SinkIo("nested tracing-backed log emission is not supported")`.
- A tracing record event without an active record returns `LogError::SinkIo("tracing backend event emitted without an active record")`.
- Missing emission result returns `LogError::SinkIo("tracing backend did not produce an emission result")`.
- `bootstrap(cfg)` builds a sink list from `cfg.logging.sinks.stderr.enabled` and `cfg.logging.sinks.file.enabled`.
- When file sink is enabled without `cfg.logging.sinks.file.path`, `bootstrap` returns `LogBootstrapError::Misconfigured("logging.sinks.file.enabled=true but logging.sinks.file.path is not set")`.
- `bootstrap` uses `NullSink` for zero configured sinks, the only sink for one configured sink, and `FanoutSink` for multiple sinks.
- `bootstrap` returns `LoggingSystem { handle: LogHandle::new(detect_hostname(), sink, SeverityText::from(cfg.logging.level)) }`.
- `StartPosition` variants are `Beginning` and `End`.
- `FileTailer` stores `path`, `start`, `offset`, `pending`, and on Unix `last_inode`.
- `FileTailer::read_new_lines(max_bytes)` returns an empty vector when `max_bytes == 0`.
- When the tailed file is missing, `read_new_lines` clears `offset`, clears `pending`, clears `last_inode` on Unix, and returns an empty vector.
- On Unix, a changed inode resets the offset and pending buffer.
- If the file length is shorter than the stored offset, `read_new_lines` restarts from offset `0`.
- When no offset is stored, `StartPosition::Beginning` starts at byte `0` and `StartPosition::End` starts at the current file length.
- `read_new_lines` opens the file, seeks to the chosen offset, reads up to `max_bytes`, appends bytes to `pending`, splits on newline, strips trailing `\\n` and `\\r`, updates the stored offset, and returns the completed lines as `Vec<Vec<u8>>`.
- `DirTailers` stores tailers in a `BTreeMap<PathBuf, FileTailer>`.
- `DirTailers::ensure_file` inserts a tailer only when the path is not already present.
- `DirTailers` exposes `iter_mut()` and `len()`.
- `PostgresIngestWorkerCtx` fields are `cfg` and `log`.
- `run(ctx)` loops forever. When `cfg.logging.postgres.enabled` is true it calls `step_once`, tracks consecutive failures, rate-limits repeated error reports, and sleeps for `cfg.logging.postgres.poll_interval_ms` between iterations.
- `POSTGRES_INGEST_ERROR_RATE_LIMIT_WINDOW_MS` is `30_000`.
- `POSTGRES_INGEST_MAX_BYTES_PER_FILE` is `256 * 1024`.
- Rate limiting keys use `stage`, `kind`, and `path`.
- When an iteration succeeds after prior failures, the worker emits app event `postgres_ingest.recovered` with message `postgres ingest recovered`, result `recovered`, and field `attempts`.
- When `step_once` fails and the rate limiter allows emission, the worker emits app event `postgres_ingest.step_once_failed` with message `postgres ingest step_once failed`, result `failed`, and fields `attempts`, `suppressed`, and `error`.
- `PostgresIngestWorkerState::new(cfg)` chooses `cfg.logging.postgres.pg_ctl_log_file` when set, otherwise `cfg.postgres.log_file`, and creates that tailer with `StartPosition::Beginning`.
- `step_once` reads the pg_ctl log tailer first, then optionally processes `cfg.logging.postgres.log_dir`.
- Pg ctl log records use producer `Postgres`, transport `FileTail`, and origin `pg_ctl_log_file`.
- Log-dir file records use producer `Postgres`, transport `FileTail`, and origin `postgres_log_dir:{file_name}` where `file_name` is the basename or `log`.
- `discover_log_dir` ignores missing directories, scans only regular files, keeps only `.log` and `.json` files, and starts `postgres.stderr.log` and `postgres.stdout.log` at `Beginning` while other files start at `End`.
- On a clean iteration, `step_once` emits debug app event `postgres_ingest.iteration` with message `postgres ingest iteration ok`, result `ok`, and fields `pg_ctl_lines_emitted`, `log_dir_files_tailed`, `log_dir_lines_emitted`, and `dir_tailers`.
- On iteration failure, `step_once` returns `WorkerError::Message` beginning with `postgres_ingest iteration_errors count=...`.
- Cleanup runs only when `cfg.logging.postgres.cleanup.enabled` is true.
- `cleanup_log_dir` ignores missing directories, scans only regular `.log` and `.json` files, always protects explicit protected paths plus basenames `postgres.json`, `postgres.stderr.log`, and `postgres.stdout.log`, and also protects files newer than `cleanup.protect_recent_seconds`.
- Cleanup sorts eligible files by modified time and then path.
- When `cleanup.max_files > 0`, cleanup removes the oldest eligible files until the count is within the limit.
- When `cleanup.max_age_seconds > 0`, cleanup also removes eligible files older than that age.
- Cleanup returns `CleanupReport { issue_count, first_issue }`.

[Facts that must not be invented]
- Do not add configuration fields, commands, examples, or environment variables beyond the facts above.
- Do not describe parsers or helpers whose behavior is not stated above.
- Do not add operational advice, setup steps, or rationale.

[Style constraints]
- Keep wording compact and reference-like.
- Prefer tables for enum variants, fields, and constants.
- It is acceptable to omit methods or private helper types that are not needed to describe the module surface.
