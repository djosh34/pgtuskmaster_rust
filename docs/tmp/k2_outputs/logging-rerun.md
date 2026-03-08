# Logging

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
