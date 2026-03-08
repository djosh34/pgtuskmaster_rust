# Logging Reference

The logging layer provides structured record emission, multi-sink output, file tailing, and PostgreSQL log ingestion through the `src/logging` module family.

## Module surface

`src/logging/mod.rs` declares private modules `event` and `raw_record`, and crate-visible modules `postgres_ingest` and `tailer`. It re-exports:

| Symbol | Description |
|---|---|
| `AppEvent` | Application event carrier |
| `AppEventHeader` | Event classification header |
| `StructuredFields` | Ordered key/value builder |
| `PostgresLineRecordBuilder` | PostgreSQL-oriented raw-record builder |
| `RawRecordBuilder` | General raw-record builder |
| `SubprocessLineRecord` | Child-process output container |
| `SubprocessStream` | Stdout or stderr discriminator |

## Record model

### `SeverityText`

| Variant | Number |
|---|---|
| `Trace` | `1` |
| `Debug` | `5` |
| `Info` | `9` |
| `Warn` | `13` |
| `Error` | `17` |
| `Fatal` | `21` |

`From<crate::config::LogLevel>` maps matching runtime log levels one-to-one.

### Enum surfaces

| Enum | Variants |
|---|---|
| `LogProducer` | `App`, `Postgres`, `PgTool` |
| `LogTransport` | `Internal`, `FileTail`, `ChildStdout`, `ChildStderr` |
| `LogParser` | `App`, `PostgresJson`, `PostgresPlain`, `Raw` |

All three enums serialize with `snake_case` names.

### `LogSource`

| Field | Type |
|---|---|
| `producer` | `LogProducer` |
| `transport` | `LogTransport` |
| `parser` | `LogParser` |
| `origin` | `String` |

### `LogRecord`

| Field | Type |
|---|---|
| `timestamp_ms` | `u64` |
| `hostname` | `String` |
| `severity_text` | `SeverityText` |
| `severity_number` | `u8` |
| `message` | `String` |
| `source` | `LogSource` |
| `attributes` | `BTreeMap<String, serde_json::Value>` |

`LogRecord::new` sets `severity_number` from `severity_text.number()` and initializes `attributes` as an empty `BTreeMap`. Serialization omits `attributes` when the map is empty.

## Application events

### `AppEventHeader`

| Field | Type |
|---|---|
| `name` | `String` |
| `domain` | `String` |
| `result` | `String` |

### `StructuredValue`

| Variant | Stored value |
|---|---|
| `Bool` | `bool` |
| `I64` | `i64` |
| `U64` | `u64` |
| `String` | `String` |
| `Json` | `serde_json::Value` |

### `StructuredFields`

`StructuredFields` stores ordered `(String, StructuredValue)` pairs in a `Vec`.

| Method | Behavior |
|---|---|
| `append_json_map` | Appends `BTreeMap<String, serde_json::Value>` entries as `StructuredValue::Json` |
| `insert` | Appends one key/value pair when the value implements `Into<StructuredValue>` |
| `insert_optional` | Appends only when the option is `Some` |
| `insert_serialized` | Serializes a value with `serde_json::to_value` and appends it as `StructuredValue::Json` |
| `into_attributes` | Consumes the ordered fields and writes them into a `BTreeMap<String, serde_json::Value>` |

### `AppEvent`

`AppEvent` stores `header`, `severity`, `message`, and `fields`.

`AppEvent::into_record` creates a `LogRecord` with:

| Source field | Value |
|---|---|
| `producer` | `LogProducer::App` |
| `transport` | `LogTransport::Internal` |
| `parser` | `LogParser::App` |
| `origin` | supplied argument |

It also inserts `event.name`, `event.domain`, and `event.result` into the emitted attribute map.

## Raw record builders

### `RawRecordBuilder`

| Field | Type |
|---|---|
| `severity` | `SeverityText` |
| `message` | `String` |
| `source` | `LogSource` |
| `fields` | `StructuredFields` |

`with_fields` replaces the stored fields. `into_record` builds a `LogRecord` and converts the stored fields into `attributes`.

### `SubprocessStream`

| Variant | `severity()` | `transport()` |
|---|---|---|
| `Stdout` | `SeverityText::Info` | `LogTransport::ChildStdout` |
| `Stderr` | `SeverityText::Warn` | `LogTransport::ChildStderr` |

### `SubprocessLineRecord`

| Field | Type |
|---|---|
| `producer` | `LogProducer` |
| `origin` | `String` |
| `job_id` | `String` |
| `job_kind` | `String` |
| `binary` | `String` |
| `stream` | `SubprocessStream` |
| `bytes` | `Vec<u8>` |

`into_raw_record` decodes the stored bytes, uses parser `LogParser::Raw`, inserts `job_id`, `job_kind`, `binary`, serialized `stream`, and optional `raw_bytes_hex`, then returns a `RawRecordBuilder`.

`decode_bytes` returns UTF-8 text when decoding succeeds. On invalid UTF-8 it returns message text `non_utf8_bytes_hex={hex}` and sets `raw_bytes_hex` to the same hex string.

### `PostgresLineRecordBuilder`

| Field | Type |
|---|---|
| `producer` | `LogProducer` |
| `transport` | `LogTransport` |
| `origin` | `String` |

`build` returns a `RawRecordBuilder` from the supplied parser, severity, message, and fields.

## Sink bootstrap and sinks

### Error enums

| Enum | Variants |
|---|---|
| `LogError` | `Json(String)`, `SinkIo(String)` |
| `LogBootstrapError` | `Misconfigured(String)`, `SinkInit(String)` |

### Sink implementations

| Sink | Behavior |
|---|---|
| `JsonlStderrSink` | Writes one JSON line per record to stderr behind a mutex |
| `JsonlFileSink` | Rejects an empty path, creates parent directories when needed, opens the file in append or truncate mode according to `crate::config::FileSinkMode`, and writes one JSON line per record |
| `NullSink` | Accepts a record and performs no write |
| `FanoutSink` | Emits to every configured sink, prints `fanout sink failure: {label}: {error}` to stderr for each sink error, succeeds when at least one sink succeeds, and returns `LogError::SinkIo` only when every sink fails |

### `bootstrap`

`bootstrap(cfg)` builds the active sink set from `cfg.logging.sinks.stderr.enabled` and `cfg.logging.sinks.file.enabled`.

| Condition | Result |
|---|---|
| `cfg.logging.sinks.file.enabled = true` and `cfg.logging.sinks.file.path` is unset | `LogBootstrapError::Misconfigured("logging.sinks.file.enabled=true but logging.sinks.file.path is not set")` |
| zero configured sinks | `NullSink` |
| one configured sink | that sink |
| multiple configured sinks | `FanoutSink` |

On success it returns `LoggingSystem { handle: LogHandle::new(detect_hostname(), sink, SeverityText::from(cfg.logging.level)) }`.

## Log handle and tracing backend

### `LogHandle`

| Field | Type |
|---|---|
| `hostname` | `String` |
| `backend` | `Arc<TracingBackend>` |
| `min_app_severity_number` | `u8` |

`LogHandle::new` stores the hostname, creates a `TracingBackend`, and stores the minimum app severity number from `SeverityText::number()`. `LogHandle::null()` uses hostname `unknown`, a `NullSink`, and minimum severity `SeverityText::Fatal`.

| Method | Behavior |
|---|---|
| `emit_app_event` | Drops events whose severity number is below `min_app_severity_number`; otherwise stamps `system_now_unix_millis()` and the stored hostname, converts the event into a record, and emits it |
| `emit_raw_record` | Stamps `system_now_unix_millis()` and the stored hostname, then emits the built record |
| `emit_record` | Forwards a completed `LogRecord` to the tracing backend |

`system_now_unix_millis()` returns milliseconds since `UNIX_EPOCH`, or `0` if the system clock is before `UNIX_EPOCH`. `detect_hostname()` returns the non-empty trimmed `HOSTNAME` environment variable, otherwise `unknown`.

### `TracingBackend`

`TracingBackend::emit` uses a thread-local active-record guard, dispatches a tracing event with target `pgtuskmaster::logging::record`, and returns the sink result recorded by `TracingRecordLayer`.

| Failure condition | Result |
|---|---|
| nested tracing-backed emission | `LogError::SinkIo("nested tracing-backed log emission is not supported")` |
| tracing record event without an active record | `LogError::SinkIo("tracing backend event emitted without an active record")` |
| missing emission result | `LogError::SinkIo("tracing backend did not produce an emission result")` |

## File tailing

### `StartPosition`

| Variant | Initial offset behavior |
|---|---|
| `Beginning` | starts at byte `0` |
| `End` | starts at the current file length |

### `FileTailer`

| Field | Type |
|---|---|
| `path` | `PathBuf` |
| `start` | `StartPosition` |
| `offset` | `Option<u64>` |
| `pending` | `Vec<u8>` |
| `last_inode` | `Option<u64>` on Unix builds |

`read_new_lines(max_bytes)` returns an empty vector when `max_bytes == 0`.

| Condition | Behavior |
|---|---|
| tailed file missing | clears `offset`, clears `pending`, clears `last_inode` on Unix, returns an empty vector |
| inode changed on Unix | resets `offset` and `pending` |
| file length shorter than stored offset | restarts from offset `0` |

`read_new_lines` opens the file, seeks to the selected offset, reads up to `max_bytes`, appends bytes to `pending`, splits on newline, strips trailing `\n` and `\r`, updates the stored offset, and returns completed lines as `Vec<Vec<u8>>`.

### `DirTailers`

`DirTailers` stores tailers in a `BTreeMap<PathBuf, FileTailer>`.

| Method | Behavior |
|---|---|
| `ensure_file` | inserts a tailer only when the path is not already present |
| `iter_mut` | yields mutable `(path, tailer)` pairs |
| `len` | returns the number of tracked tailers |

## PostgreSQL ingest worker

### `PostgresIngestWorkerCtx`

| Field | Type |
|---|---|
| `cfg` | `RuntimeConfig` |
| `log` | `LogHandle` |

### Constants

| Name | Value |
|---|---|
| `POSTGRES_INGEST_ERROR_RATE_LIMIT_WINDOW_MS` | `30_000` |
| `POSTGRES_INGEST_MAX_BYTES_PER_FILE` | `256 * 1024` |

Repeated error reports are rate-limited by keys built from `stage`, `kind`, and `path`.

### `run`

`run(ctx)` loops forever. When `cfg.logging.postgres.enabled` is true it calls `step_once`, tracks consecutive failures, rate-limits repeated error reports, and sleeps for `cfg.logging.postgres.poll_interval_ms` between iterations.

When an iteration succeeds after prior failures, the worker emits:

| Event name | Message | Result | Fields |
|---|---|---|---|
| `postgres_ingest.recovered` | `postgres ingest recovered` | `recovered` | `attempts` |

When `step_once` fails and the rate limiter allows emission, the worker emits:

| Event name | Message | Result | Fields |
|---|---|---|---|
| `postgres_ingest.step_once_failed` | `postgres ingest step_once failed` | `failed` | `attempts`, `suppressed`, `error` |

### `PostgresIngestWorkerState::new`

The worker chooses `cfg.logging.postgres.pg_ctl_log_file` when set, otherwise `cfg.postgres.log_file`, and creates that tailer with `StartPosition::Beginning`.

### `step_once`

`step_once` reads the pg ctl log tailer first, then optionally processes `cfg.logging.postgres.log_dir`.

| Record source | Producer | Transport | Origin |
|---|---|---|---|
| pg ctl log | `Postgres` | `FileTail` | `pg_ctl_log_file` |
| log-dir file | `Postgres` | `FileTail` | `postgres_log_dir:{file_name}` where `file_name` is the basename or `log` |

`discover_log_dir` ignores missing directories, scans only regular files, keeps only `.log` and `.json` files, and starts `postgres.stderr.log` and `postgres.stdout.log` at `Beginning` while other files start at `End`.

On a clean iteration, `step_once` emits:

| Event name | Message | Result | Fields |
|---|---|---|---|
| `postgres_ingest.iteration` | `postgres ingest iteration ok` | `ok` | `pg_ctl_lines_emitted`, `log_dir_files_tailed`, `log_dir_lines_emitted`, `dir_tailers` |

On failure, `step_once` returns `WorkerError::Message` beginning with `postgres_ingest iteration_errors count=...`.

## Cleanup

Cleanup runs only when `cfg.logging.postgres.cleanup.enabled` is true.

`cleanup_log_dir`:

| Rule | Behavior |
|---|---|
| directory handling | ignores missing directories |
| file filter | scans only regular `.log` and `.json` files |
| always protected basenames | `postgres.json`, `postgres.stderr.log`, `postgres.stdout.log` |
| additional protected paths | explicit protected paths passed by the caller |
| recent-file protection | protects files newer than `cleanup.protect_recent_seconds` |
| ordering | sorts eligible files by modified time and then path |
| count limit | when `cleanup.max_files > 0`, removes the oldest eligible files until the count is within the limit |
| age limit | when `cleanup.max_age_seconds > 0`, also removes eligible files older than that age |

The function returns `CleanupReport { issue_count, first_issue }`.
