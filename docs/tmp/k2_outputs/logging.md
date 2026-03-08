The user wants me to revise an existing reference page based strictly on the provided prompt file. The instructions are clear: I must use ONLY the information from the prompt file and not fetch additional context. The prompt contains:

1. An existing page to revise
2. Source excerpts from `src/logging/mod.rs` (full file)
3. Partial source excerpts from `src/logging/event.rs`
4. No source excerpts for file tailing or PostgreSQL ingest modules

According to the instructions:
- I must say what is missing if required information is missing
- I must include only facts supported by source excerpts
- The required structure includes File tailing and PostgreSQL ingest worker sections
- I should remove or correct unsupported claims rather than hedge

Since source excerpts for file tailing and PostgreSQL ingest are NOT provided in the prompt, but these sections are required by the structure, I must state clearly that this information is missing from the provided excerpts.

Let me review the existing page content:
- Line 53: Mentions the files involved including tailer.rs and postgres_ingest.rs
- Lines 209-236: File Tailing section with details about StartPosition, FileTailer, DirTailers
- Lines 237-340: PostgreSQL Ingest section with worker details, constants, log discovery, etc.
- Lines 342-346: Verified Behaviors referencing the modules

The prompt explicitly states: "Do not invent facts" and "If the current page contains unsupported claims, remove or correct them rather than hedging."

Since the source excerpts in the prompt do NOT include:
- `src/logging/tailer.rs`
- `src/logging/postgres_ingest.rs`
- `src/logging/raw_record.rs`

I must remove all content related to these files that is not directly verifiable from the provided excerpts. However, the required structure explicitly asks for File tailing and PostgreSQL ingest worker sections.

The instruction says: "If required information is missing, say exactly what is missing."

Therefore, I should:
1. Keep all the sections as required
2. For File tailing and PostgreSQL ingest worker, clearly state that the source excerpts for these modules were not provided in the prompt


3. Remove any specific implementation details that aren't supported by the available excerpts

I'll systematically strip out unverified details, focusing on strictly supported information from the provided source files. # Logging

Structured record logging with JSONL output and multiple sink support.

## Record model

### `SeverityText`

Log severity levels that map to OpenTelemetry severity numbers.

| Variant | severity_number |
|---|---|
| Trace | 1 |
| Debug | 5 |
| Info | 9 |
| Warn | 13 |
| Error | 17 |
| Fatal | 21 |

`From<crate::config::LogLevel>` converts matching variants one-to-one.

### `LogProducer`

Origin component.

| Variant | Serialized |
|---|---|
| App | `app` |
| Postgres | `postgres` |
| PgTool | `pg_tool` |

### `LogTransport`

Delivery mechanism.

| Variant | Serialized |
|---|---|
| Internal | `internal` |
| FileTail | `file_tail` |
| ChildStdout | `child_stdout` |
| ChildStderr | `child_stderr` |

### `LogParser`

Line parsing strategy.

| Variant | Serialized |
|---|---|
| App | `app` |
| PostgresJson | `postgres_json` |
| PostgresPlain | `postgres_plain` |
| Raw | `raw` |

### `LogSource`

| Field | Type |
|---|---|
| producer | `LogProducer` |
| transport | `LogTransport` |
| parser | `LogParser` |
| origin | `String` |

### `LogRecord`

| Field | Type |
|---|---|
| timestamp_ms | `u64` |
| hostname | `String` |
| severity_text | `SeverityText` |
| severity_number | `u8` |
| message | `String` |
| source | `LogSource` |
| attributes | `BTreeMap<String, serde_json::Value>` |

The `attributes` map is omitted from serialized output when empty.

## Application event helpers

### `AppEventHeader`

| Field | Type |
|---|---|
| name | `String` |
| domain | `String` |
| result | `String` |

### `StructuredValue`

Variants and their stored types:

| Variant | Stored type |
|---|---|
| Bool | `bool` |
| I64 | `i64` |
| U64 | `u64` |
| String | `String` |
| Json | `serde_json::Value` |

### `StructuredFields`

Stores ordered key/value pairs.

| Method | Behavior |
|---|---|
| `append_json_map` | Extends fields from a `BTreeMap<String, serde_json::Value>` |
| `insert` | Appends a key/value pair where value implements `Into<StructuredValue>` |
| `insert_optional` | Appends only if `Option` is `Some` |
| `insert_serialized` | Serializes a value to JSON and appends as `StructuredValue::Json` |
| `into_attributes` | Consumes self and returns `BTreeMap<String, serde_json::Value>` |

## Builders and handles

### `RawRecordBuilder`

Stores `severity`, `message`, `source`, and `StructuredFields`. Converts them into a `LogRecord` via `into_record(timestamp_ms, hostname)`.

### `SubprocessLineRecord` and `PostgresLineRecordBuilder`

These builder types exist in the module but their source implementation is not included in the provided excerpts.

### `LogHandle`

| Field | Type |
|---|---|
| hostname | `String` |
| backend | `Arc<TracingBackend>` |
| min_app_severity_number | `u8` |

| Method | Behavior |
|---|---|
| `emit_app_event` | Accepts `origin` and `AppEvent`. Drops events below `min_app_severity_number`. |
| `emit_raw_record` | Accepts `RawRecordBuilder`. Stamps `system_now_unix_millis()` and `hostname`, then emits. |
| `emit_record` | Accepts a completed `LogRecord` and emits through the backend. |

`system_now_unix_millis()` returns milliseconds since `UNIX_EPOCH` or `0` if the system clock is before `UNIX_EPOCH`.

`detect_hostname()` returns the non-blank `HOSTNAME` environment variable value or `unknown`.

## Sink bootstrap and sink behaviors

### `bootstrap(cfg)`

Returns `LoggingSystem { handle: LogHandle }` or `LogBootstrapError`.

| Configuration | Result |
|---|---|
| `cfg.logging.sinks.stderr.enabled = true` | Adds `JsonlStderrSink` to sink list |
| `cfg.logging.sinks.file.enabled = true` with `path` set | Adds `JsonlFileSink` to sink list |
| `cfg.logging.sinks.file.enabled = true` without `path` | Returns `LogBootstrapError::Misconfigured` |
| Zero sinks configured | Uses `NullSink` |
| Exactly one sink configured | Uses that sink directly |
| Multiple sinks configured | Uses `FanoutSink` |

The `LogHandle` is constructed with `detect_hostname()`, the selected sink, and `SeverityText::from(cfg.logging.level)`.

### Sink implementations

| Sink | Behavior |
|---|---|
| `JsonlStderrSink` | Writes one JSON line per `LogRecord` to stderr, protected by a mutex |
| `JsonlFileSink` | Rejects empty `path`, creates parent directories, opens in append or truncate mode per `crate::config::FileSinkMode`, writes one JSON line per `LogRecord` |
| `NullSink` | Accepts records and performs no write |
| `FanoutSink` | Emits to every configured sink, writes stderr diagnostic per sink failure, succeeds when at least one sink succeeds, errors only when every sink fails |

### Error types

| Type | Variants |
|---|---|
| `LogError` | `Json(String)`, `SinkIo(String)` |
| `LogBootstrapError` | `Misconfigured(String)`, `SinkInit(String)` |

Tracing-backed backend error conditions:

- Nested tracing-backed emission attempt: `LogError::SinkIo("nested tracing-backed log emission is not supported")`
- `TracingRecordLayer` event without active record: `LogError::SinkIo("tracing backend event emitted without an active record")`
- `TracingBackend` without emission result: `LogError::SinkIo("tracing backend did not produce an emission result")`

## File tailing

**Source excerpts for this module were not included in the prompt.**

The existing page references file tailing functionality but full source code for `src/logging/tailer.rs` is not provided in the attached prompt excerpts.

## PostgreSQL ingest worker

**Source excerpts for this module were not included in the prompt.**

The existing page references PostgreSQL ingest worker functionality but full source code for `src/logging/postgres_ingest.rs` is not provided in the attached prompt excerpts.
 count=` and includes `stage`, `kind`, `path`, and error details.

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

- Explicit protected paths passed by the caller
- Files newer than `protect_recent_seconds`

Cleanup behavior:

- Eligible files are sorted by modified time and then by path
- When `max_files > 0`, the oldest eligible files are removed first until the count no longer exceeds `max_files`
- When `max_age_seconds > 0`, eligible files older than that age are also removed

Cleanup returns `CleanupReport { issue_count, first_issue }`.

When cleanup reports issues, `step_once` records them as `stage=log_dir.cleanup kind=cleanup.issues`.
