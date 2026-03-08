# Logging Reference

The logging layer provides structured, multi-sink log emission, file tailing, and PostgreSQL log ingestion with tracing integration.

## Module surface

`src/logging/mod.rs` declares private modules `event` and `raw_record`, and crate-visible modules `postgres_ingest` and `tailer`. It re-exports:

| Symbol | Description |
|--------|-------------|
| `AppEvent` | Application event carrier |
| `AppEventHeader` | Event classification header |
| `StructuredFields` | Ordered key-value builder |
| `PostgresLineRecordBuilder` | PostgreSQL-specific builder |
| `RawRecordBuilder` | General raw record builder |
| `SubprocessLineRecord` | Child process output container |
| `SubprocessStream` | Stdout/Stderr discriminator |

## Record model

### SeverityText

Maps textual severity to numeric levels.

| Variant | Number |
|---------|--------|
| Trace | 1 |
| Debug | 5 |
| Info | 9 |
| Warn | 13 |
| Error | 17 |
| Fatal | 21 |

`From<crate::config::LogLevel>` maps matching variants one-to-one.

### LogSource

| Field | Type |
|-------|------|
| producer | LogProducer |
| transport | LogTransport |
| parser | LogParser |
| origin | String |

### LogRecord

| Field | Type |
|-------|------|
| timestamp_ms | u64 |
| hostname | String |
| severity_text | SeverityText |
| severity_number | i8 |
| message | String |
| source | LogSource |
| attributes | BTreeMap<String, Value> |

`LogRecord::new` sets `severity_number` from `severity_text.number()` and initializes `attributes` as an empty `BTreeMap`. Empty `attributes` are skipped during serialization.

### Enums

| Enum | Variants (serialized as snake_case) |
|------|-------------------------------------|
| LogProducer | app, postgres, pg_tool |
| LogTransport | internal, file_tail, child_stdout, child_stderr |
| LogParser | app, postgres_json, postgres_plain, raw |

## Application events

### AppEventHeader

| Field | Type |
|-------|------|
| name | String |
| domain | String |
| result | String |

### StructuredValue

| Variant | Payload |
|---------|----------|
| Bool | bool |
| I64 | i64 |
| U64 | u64 |
| String | String |
| Json | Value |

### StructuredFields

Stores ordered `(String, StructuredValue)` pairs in a `Vec`.

| Method | Behavior |
|--------|----------|
| `append_json_map` | Appends values from `BTreeMap<String, serde_json::Value>` as `StructuredValue::Json` |
| `insert` | Appends one key-value pair when the value implements `Into<StructuredValue>` |
| `insert_optional` | Appends only when the option is `Some` |
| `insert_serialized` | Serializes a value with `serde_json::to_value` and appends it as `StructuredValue::Json` |
| `into_attributes` | Consumes ordered fields and writes them into a `BTreeMap<String, Value>` |

### AppEvent

| Field | Type |
|-------|------|
| header | AppEventHeader |
| severity | SeverityText |
| message | String |
| fields | StructuredFields |

`AppEvent::into_record` creates a `LogRecord` with producer `App`, transport `Internal`, parser `App`, and the supplied origin. It inserts `event.name`, `event.domain`, and `event.result` into the attributes map.

## Raw record builders

### RawRecordBuilder

| Field | Type |
|-------|------|
| severity | SeverityText |
| message | String |
| source | LogSource |
| fields | StructuredFields |

`RawRecordBuilder::with_fields` replaces the stored fields. `RawRecordBuilder::into_record` builds a `LogRecord` and converts stored fields into attributes.

### SubprocessStream

| Variant | severity() | transport() |
|---------|------------|-------------|
| Stdout | Info | child_stdout |
| Stderr | Warn | child_stderr |

### SubprocessLineRecord

| Field | Type |
|-------|------|
| producer | LogProducer |
| origin | String |
| job_id | JobId |
| job_kind | JobKind |
| binary | String |
| stream | SubprocessStream |
| raw | Vec<u8> |

`SubprocessLineRecord::into_raw_record` decodes the bytes, uses parser `Raw`, inserts `job_id`, `job_kind`, `binary`, serialized `stream`, and optional `raw_bytes_hex`, and returns a `RawRecordBuilder`. `decode_bytes` returns UTF-8 text when decoding succeeds; on invalid UTF-8 it returns message text `non_utf8_bytes_hex={hex}` and sets `raw_bytes_hex` to the same hex string.

### PostgresLineRecordBuilder

| Field | Type |
|-------|------|
| producer | LogProducer |
| transport | LogTransport |
| origin | String |

`PostgresLineRecordBuilder::build` returns a `RawRecordBuilder` using the supplied parser, severity, message, and fields.

## Sink bootstrap and sinks

### LogError

| Variant | Payload |
|---------|----------|
| Json | String |
| SinkIo | String |

### LogBootstrapError

| Variant | Payload |
|---------|----------|
| Misconfigured | String |
| SinkInit | String |

### JsonlStderrSink

Writes one JSON line per record to stderr behind a mutex.

### JsonlFileSink

| Constructor behavior |
|----------------------|
| Rejects an empty path |
| Creates parent directories when needed |
| Opens the file in append or truncate mode according to `crate::config::FileSinkMode` |
| Stores a `LineWriter<File>` behind a mutex |

`JsonlFileSink::emit` writes one JSON line per record to the configured path.

### NullSink

Accepts a record and does nothing.

### FanoutSink

Stores labeled sinks. On emit it writes to every sink, prints `fanout sink failure: {label}: {error}` to stderr for each sink error, succeeds if at least one sink succeeded, and returns `LogError::SinkIo` only when every sink failed.

### bootstrap

Builds a sink list from `cfg.logging.sinks.stderr.enabled` and `cfg.logging.sinks.file.enabled`.

| Condition | Behavior |
|-----------|----------|
| File sink enabled without path | Returns `LogBootstrapError::Misconfigured("logging.sinks.file.enabled=true but logging.sinks.file.path is not set")` |
| Zero configured sinks | Uses `NullSink` |
| One configured sink | Uses that sink directly |
| Multiple configured sinks | Uses `FanoutSink` |

Returns `LoggingSystem { handle: LogHandle::new(detect_hostname(), sink, SeverityText::from(cfg.logging.level)) }`.

## Log handle and tracing backend

### LogHandle

| Field | Type |
|-------|------|
| hostname | String |
| backend | TracingBackend |
| min_app_severity_number | i8 |

`LogHandle::new` stores the hostname, creates a `TracingBackend`, and stores the minimum app severity number from `SeverityText::number()`. `LogHandle::null()` uses hostname `unknown`, a `NullSink`, and minimum severity `Fatal`.

| Method | Behavior |
|--------|----------|
| `emit_app_event` | Drops events whose severity number is below `min_app_severity_number`. Otherwise stamps `system_now_unix_millis()` and the stored hostname, converts the event into a record, and emits it |
| `emit_raw_record` | Stamps `system_now_unix_millis()` and the stored hostname, then emits the built record |
| `emit_record` | Forwards a record to the tracing backend |

`system_now_unix_millis()` returns milliseconds since `UNIX_EPOCH`, or `0` if the system clock is before `UNIX_EPOCH`. `detect_hostname()` returns the non-empty trimmed `HOSTNAME` environment variable, otherwise `unknown`.

### TracingBackend

`TracingBackend::emit` uses a thread-local active-record guard, dispatches a tracing event with target `pgtuskmaster::logging::record`, and returns the sink result recorded by `TracingRecordLayer`.

| Failure condition | Result |
|-------------------|--------|
| Nested tracing-backed emission | `LogError::SinkIo("nested tracing-backed log emission is not supported")` |
| Event without active record | `LogError::SinkIo("tracing backend event emitted without an active record")` |
| Missing emission result | `LogError::SinkIo("tracing backend did not produce an emission result")` |

## File tailing

### StartPosition

| Variant | Behavior |
|---------|----------|
| Beginning | Starts at byte 0 |
| End | Starts at current file length |

### FileTailer

| Field | Type |
|-------|------|
| path | PathBuf |
| start | StartPosition |
| offset | u64 |
| pending | Vec<u8> |
| last_inode | Option<u64> (Unix only) |

`FileTailer::read_new_lines(max_bytes)` returns an empty vector when `max_bytes == 0`.

| Condition | Behavior |
|-----------|----------|
| Tailed file missing | Clears `offset`, clears `pending`, clears `last_inode` on Unix, returns empty vector |
| Changed inode (Unix) | Resets offset and pending buffer |
| File length shorter than offset | Restarts from offset 0 |

When no offset is stored, `StartPosition::Beginning` starts at byte `0` and `StartPosition::End` starts at the current file length. `read_new_lines` opens the file, seeks to the chosen offset, reads up to `max_bytes`, appends bytes to `pending`, splits on newline, strips trailing `\n` and `\r`, updates the stored offset, and returns the completed lines as `Vec<Vec<u8>>`.

### DirTailers

| Field | Type |
|-------|------|
| tailers | BTreeMap<PathBuf, FileTailer> |

`DirTailers::ensure_file` inserts a tailer only when the path is not already present. Exposes `iter_mut()` and `len()`.

## PostgreSQL ingest worker

### PostgresIngestWorkerCtx

| Field | Type |
|-------|------|
| cfg | Config |
| log | LogHandle |

### Constants

| Name | Value |
|------|-------|
| POSTGRES_INGEST_ERROR_RATE_LIMIT_WINDOW_MS | 30_000 |
| POSTGRES_INGEST_MAX_BYTES_PER_FILE | 256 * 1024 |

Rate limiting keys use `stage`, `kind`, and `path`.

### run

Loops forever. When `cfg.logging.postgres.enabled` is true it calls `step_once`, tracks consecutive failures, rate-limits repeated error reports, and sleeps for `cfg.logging.postgres.poll_interval_ms` between iterations.

When an iteration succeeds after prior failures, the worker emits app event `postgres_ingest.recovered` with message `postgres ingest recovered`, result `recovered`, and field `attempts`.

When `step_once` fails and the rate limiter allows emission, the worker emits app event `postgres_ingest.step_once_failed` with message `postgres ingest step_once failed`, result `failed`, and fields `attempts`, `suppressed`, and `error`.

### PostgresIngestWorkerState::new

Chooses `cfg.logging.postgres.pg_ctl_log_file` when set, otherwise `cfg.postgres.log_file`, and creates that tailer with `StartPosition::Beginning`.

### step_once

Reads the pg_ctl log tailer first, then optionally processes `cfg.logging.postgres.log_dir`.

| Record source | Producer | Transport | Origin |
|---------------|----------|-----------|--------|
| Pg ctl log | Postgres | FileTail | pg_ctl_log_file |
| Log-dir file | Postgres | FileTail | postgres_log_dir:{file_name} where file_name is basename or 'log' |

`discover_log_dir` ignores missing directories, scans only regular files, keeps only `.log` and `.json` files, and starts `postgres.stderr.log` and `postgres.stdout.log` at `Beginning` while other files start at `End`.

On a clean iteration, `step_once` emits debug app event `postgres_ingest.iteration` with message `postgres ingest iteration ok`, result `ok`, and fields `pg_ctl_lines_emitted`, `log_dir_files_tailed`, `log_dir_lines_emitted`, and `dir_tailers`.

On iteration failure, `step_once` returns `WorkerError::Message` beginning with `postgres_ingest iteration_errors count=...`.

## Cleanup

Cleanup runs only when `cfg.logging.postgres.cleanup.enabled` is true.

`cleanup_log_dir` ignores missing directories, scans only regular `.log` and `.json` files, always protects explicit protected paths plus basenames `postgres.json`, `postgres.stderr.log`, and `postgres.stdout.log`, and also protects files newer than `cleanup.protect_recent_seconds`.

Cleanup sorts eligible files by modified time and then path.

When `cleanup.max_files > 0`, cleanup removes the oldest eligible files until the count is within the limit.

When `cleanup.max_age_seconds > 0`, cleanup also removes eligible files older than that age.

Cleanup returns `CleanupReport { issue_count, first_issue }`.
