# Logging Reference

Records, sinks, tailing, and PostgreSQL ingest in `src/logging/mod.rs`, `src/logging/event.rs`, `src/logging/raw_record.rs`, `src/logging/tailer.rs`, and `src/logging/postgres_ingest.rs`.

## Record Model

### Core enums

| Enum | Serialized values | Purpose |
|---|---|---|
| `SeverityText` | `trace`, `debug`, `info`, `warn`, `error`, `fatal` | log severity levels |
| `LogProducer` | `app`, `postgres`, `pg_tool` | origin component |
| `LogTransport` | `internal`, `file_tail`, `child_stdout`, `child_stderr` | delivery mechanism |
| `LogParser` | `app`, `postgres_json`, `postgres_plain`, `raw` | line parsing strategy |

### `SeverityText` mapping

| Variant | `severity_number` |
|---|---|
| `Trace` | `1` |
| `Debug` | `5` |
| `Info` | `9` |
| `Warn` | `13` |
| `Error` | `17` |
| `Fatal` | `21` |

`From<crate::config::LogLevel>` maps matching log levels one-to-one.

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

`attributes` is omitted from serialized output when empty.

## Application Event Helpers

### `AppEventHeader`

| Field | Meaning |
|---|---|
| `name` | event identifier |
| `domain` | event namespace |
| `result` | event outcome |

### `StructuredValue` variants

| Variant | Stored value type |
|---|---|
| `Bool` | `bool` |
| `I64` | `i64` |
| `U64` | `u64` |
| `String` | `String` |
| `Json` | `serde_json::Value` |

### `StructuredFields`

Stores ordered key/value pairs. Methods: `append_json_map`, `insert`, `insert_optional`, `insert_serialized`, `into_attributes`.

### `AppEvent::into_record`

Emits a `LogRecord` with:

- `producer`: `App`
- `transport`: `Internal`
- `parser`: `App`
- `attributes`: `event.name`, `event.domain`, `event.result`

## Builders And Handles

### `RawRecordBuilder`

Stores `severity`, `message`, `source`, and `StructuredFields`. Converts them into a `LogRecord`.

### `SubprocessStream`

| Variant | Severity | Transport |
|---|---|---|
| `Stdout` | `Info` | `ChildStdout` |
| `Stderr` | `Warn` | `ChildStderr` |

### `SubprocessLineRecord`

Stores `producer`, `origin`, `job_id`, `job_kind`, `binary`, `stream`, and `bytes`.

`SubprocessLineRecord::into_raw_record` decodes UTF-8 when possible. Non-UTF-8 produces message `non_utf8_bytes_hex=<hex>` and stores the same hex string in `raw_bytes_hex`.

### `PostgresLineRecordBuilder`

Stores `producer`, `transport`, and `origin`. `build(parser, severity, message, fields)` returns a `RawRecordBuilder`.

### `LogHandle`

Stores `hostname`, a tracing-backed backend, and `min_app_severity_number`.

| Method | Behavior |
|---|---|
| `emit_app_event` | drops events below `min_app_severity_number` |
| `emit_raw_record` | stamps `system_now_unix_millis()` and `hostname`, then emits |
| `emit_record` | emits a supplied `LogRecord` through the backend |

`system_now_unix_millis()` returns milliseconds since `UNIX_EPOCH` or `0` when the system clock is before `UNIX_EPOCH`.

`detect_hostname()` uses a non-blank `HOSTNAME` environment value when present and `unknown` otherwise.

## Sink Bootstrap

### `bootstrap(cfg)` outcomes

| Configuration | Result |
|---|---|
| `cfg.logging.sinks.stderr.enabled = true` | add `JsonlStderrSink` |
| `cfg.logging.sinks.file.enabled = true` with path | add `JsonlFileSink` |
| file sink enabled without path | return `LogBootstrapError::Misconfigured` |
| zero sinks configured | use `NullSink` |
| exactly one sink configured | use that sink directly |
| multiple sinks configured | use `FanoutSink` |

Returns `LoggingSystem { handle }` where `handle` is `LogHandle::new(hostname, sink, SeverityText::from(cfg.logging.level))`.

### Sink behavior

| Sink | Behavior |
|---|---|
| `JsonlStderrSink` | writes one JSON line per `LogRecord` to stderr |
| `JsonlFileSink` | rejects empty path, creates parent directories, opens in append or truncate mode per `crate::config::FileSinkMode`, writes one JSON line per `LogRecord` |
| `NullSink` | accepts records and performs no write |
| `FanoutSink` | emits to every configured sink, writes stderr diagnostic per sink failure, succeeds when at least one sink succeeds, errors only when every sink fails |

### Error types

| Type | Variants |
|---|---|
| `LogError` | `Json(String)`, `SinkIo(String)` |
| `LogBootstrapError` | `Misconfigured(String)`, `SinkInit(String)` |

### Tracing-backed backend errors

- nested tracing-backed emission: `LogError::SinkIo("nested tracing-backed log emission is not supported")`
- `TracingRecordLayer` without active record: `LogError::SinkIo("tracing backend event emitted without an active record")`
- `TracingBackend` without emission result: `LogError::SinkIo("tracing backend did not produce an emission result")`

## File Tailing

### `StartPosition`

Values: `Beginning`, `End`.

### `FileTailer` fields

| Field | Purpose |
|---|---|
| `path` | tracked file path |
| `start` | initial read position |
| `offset` | current read offset when present |
| `pending` | buffered bytes without complete newline |
| `last_inode` | unix inode tracker for rotation detection |

### `FileTailer::read_new_lines(max_bytes)`

- returns no lines when `max_bytes` is `0`
- resets `offset`, `pending`, and `last_inode` and returns an empty vector when the target file is missing
- resets on truncation or, on unix, inode rotation
- strips trailing newline and trailing carriage return
- reads at most `max_bytes` bytes per call

### `DirTailers`

Stores path-to-`FileTailer` entries. Methods: `ensure_file`, `iter_mut`, `len`.

## PostgreSQL Ingest

### Worker context and state

| Type | Fields |
|---|---|
| `PostgresIngestWorkerCtx` | `cfg`, `log` |
| `PostgresIngestWorkerState` | `pg_ctl_log`, `dir_tailers` |

Constants:

- `POSTGRES_INGEST_ERROR_RATE_LIMIT_WINDOW_MS = 30_000`
- `POSTGRES_INGEST_MAX_BYTES_PER_FILE = 256 * 1024`

### Worker lifecycle

Runs forever. Performs work only when `cfg.logging.postgres.enabled` is true. Sleeps `cfg.logging.postgres.poll_interval_ms` between iterations. Tracks consecutive failures, rate-limits repeated error reports, emits `postgres_ingest.recovered` after a failure streak clears, and resets the streak on success.

### `PostgresIngestWorkerState::new`

Uses `cfg.logging.postgres.pg_ctl_log_file` when present, otherwise `cfg.postgres.log_file`. Starts `pg_ctl_log` from `Beginning`.

### Log discovery

`discover_log_dir` registers only files ending in `.log` or `.json`.

| File name | Start position |
|---|---|
| `postgres.stderr.log` | `Beginning` |
| `postgres.stdout.log` | `Beginning` |
| other `.log` or `.json` files | `End` |

### Line normalization

`decode_line` returns UTF-8 text when possible and `non_utf8_bytes_hex=<hex>` otherwise.

`normalize_postgres_line` applies parsers in order:

1. JSON with non-empty `message` field: `PostgresJson`
2. Plain format matching `timestamp [pid] LEVEL: message`: `PostgresPlain`
3. Raw fallback: `Raw`

#### JSON normalization

Stores full object under `postgres.json`. Derives severity from `error_severity`, then `severity`, defaulting to `Info`.

#### Plain normalization

Stores original level text under `postgres.level_raw`.

#### Raw fallback

Uses severity `Info` and fields `parse_failed=true` and `raw_line`.

### PostgreSQL severity mapping

| PostgreSQL level | `SeverityText` |
|---|---|
| `DEBUG`, `DEBUG1` through `DEBUG5` | `Debug` |
| `INFO`, `NOTICE`, `LOG` | `Info` |
| `WARNING` | `Warn` |
| `ERROR` | `Error` |
| `FATAL`, `PANIC` | `Fatal` |
| unknown values | `Info` |

### Iteration events and errors

Success: emits debug app event `postgres_ingest.iteration` with fields `pg_ctl_lines_emitted`, `log_dir_files_tailed`, `log_dir_lines_emitted`, and `dir_tailers`.

Failure: returns `WorkerError::Message` beginning with `postgres_ingest iteration_errors count=` and includes `stage`, `kind`, `path`, and error details.

The run loop may emit:

- `postgres_ingest.step_once_failed` with fields `attempts`, `suppressed`, and `error`
- `postgres_ingest.recovered` with field `attempts`

Repeated failure reports are rate-limited by `stage`, `kind`, and `path`.

### Cleanup

Cleanup is attempted only when `cfg.logging.postgres.cleanup.enabled` is true.

`cleanup_log_dir` operates only on `.log` and `.json` files.

Protected basenames:

- `postgres.json`
- `postgres.stderr.log`
- `postgres.stdout.log`

Additional protections:

- explicit protected paths passed by the caller
- files newer than `protect_recent_seconds`

Cleanup behavior:

- eligible files are sorted by modified time and then by path
- when `max_files > 0`, the oldest eligible files are removed first until the count no longer exceeds `max_files`
- when `max_age_seconds > 0`, eligible files older than that age are also removed

Cleanup returns `CleanupReport { issue_count, first_issue }`.

When cleanup reports issues, `step_once` records them as `stage=log_dir.cleanup kind=cleanup.issues`.

## Verified Behaviors

- `src/logging/mod.rs`: app event encoding, severity filtering, sink bootstrap variants, file output, fanout behavior, and the all-sinks-disabled bootstrap path
- `src/logging/tailer.rs`: file tailing append and rotation handling
- `src/logging/postgres_ingest.rs`: parser selection for JSON, plain, and raw PostgreSQL lines; non-UTF-8 fallback encoding and emission; ingest error rate limiting; cleanup protections and removal rules; recovery events after failure streaks
