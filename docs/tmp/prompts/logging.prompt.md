Answer using only the information in this prompt.
Do not ask to inspect files, browse, run tools, or fetch more context.
If required information is missing, say exactly what is missing.

You are writing prose for an mdBook documentation page.
Return only markdown for the page body requested.
Use ASCII punctuation only.
Do not use em dashes.
Do not invent facts.

[Task]
- Revise an existing reference page so it stays strictly in Diataxis reference form.

[Page path]
- docs/src/reference/logging.md

[Page goal]
- Reference the logging record model, sinks, raw record builders, file tailing, and PostgreSQL ingest worker.

[Audience]
- Operators and contributors who need accurate repo-backed facts while working with pgtuskmaster.

[User need]
- Consult the machinery surface, data model, constraints, constants, and behavior without being taught procedures or background explanations.

[mdBook context]
- This is an mdBook page under docs/src/reference/.
- Keep headings and lists suitable for mdBook.
- Do not add verification notes, scratch notes, or commentary about how the page was produced.

[Diataxis guidance]
- This page must stay in the reference quadrant: cognition plus application.
- Describe and only describe.
- Structure the page to mirror the machinery, not a guessed workflow.
- Use neutral, technical language.
- Examples are allowed only when they illustrate the surface concisely.
- Do not include step-by-step operations, recommendations, rationale, or explanations of why the design exists.
- If action or explanation seems necessary, keep the page neutral and mention the boundary without turning the page into a how-to or explanation article.

[Required structure]
- Overview\n- Record model\n- Application event helpers\n- Builders and handles\n- Sink bootstrap and sink behaviors\n- File tailing\n- PostgreSQL ingest worker

[Output requirements]
- Preserve or improve the title so it matches the machinery.
- Prefer compact tables where they clarify enums, fields, constants, routes, or error variants.
- Include only facts supported by the supplied source excerpts.
- If the current page contains unsupported claims, remove or correct them rather than hedging.

[Existing page to revise]

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

[Repo facts and source excerpts]

--- BEGIN FILE: src/logging/mod.rs ---
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::LineWriter;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use serde_json::Value;
use thiserror::Error;
use tracing::{dispatcher, Dispatch};
use tracing_subscriber::layer::{Context, Layer};
use tracing_subscriber::prelude::*;
use tracing_subscriber::Registry;

mod event;
mod raw_record;

pub(crate) mod postgres_ingest;
pub(crate) mod tailer;

#[cfg(test)]
pub(crate) use event::decode_app_event;
pub(crate) use event::{AppEvent, AppEventHeader, StructuredFields};
pub(crate) use raw_record::{
    PostgresLineRecordBuilder, RawRecordBuilder, SubprocessLineRecord, SubprocessStream,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum SeverityText {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

impl SeverityText {
    pub(crate) fn number(self) -> u8 {
        // OpenTelemetry severity_number mapping.
        match self {
            Self::Trace => 1,
            Self::Debug => 5,
            Self::Info => 9,
            Self::Warn => 13,
            Self::Error => 17,
            Self::Fatal => 21,
        }
    }
}

impl From<crate::config::LogLevel> for SeverityText {
    fn from(value: crate::config::LogLevel) -> Self {
        match value {
            crate::config::LogLevel::Trace => Self::Trace,
            crate::config::LogLevel::Debug => Self::Debug,
            crate::config::LogLevel::Info => Self::Info,
            crate::config::LogLevel::Warn => Self::Warn,
            crate::config::LogLevel::Error => Self::Error,
            crate::config::LogLevel::Fatal => Self::Fatal,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LogProducer {
    App,
    Postgres,
    PgTool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LogTransport {
    Internal,
    FileTail,
    ChildStdout,
    ChildStderr,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LogParser {
    App,
    PostgresJson,
    PostgresPlain,
    Raw,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct LogSource {
    pub(crate) producer: LogProducer,
    pub(crate) transport: LogTransport,
    pub(crate) parser: LogParser,
    pub(crate) origin: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct LogRecord {
    pub(crate) timestamp_ms: u64,
    pub(crate) hostname: String,
    pub(crate) severity_text: SeverityText,
    pub(crate) severity_number: u8,
    pub(crate) message: String,
    pub(crate) source: LogSource,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub(crate) attributes: BTreeMap<String, Value>,
}

impl LogRecord {
    pub(crate) fn new(
        timestamp_ms: u64,
        hostname: String,
        severity_text: SeverityText,
        message: String,
        source: LogSource,
    ) -> Self {
        Self {
            timestamp_ms,
            hostname,
            severity_text,
            severity_number: severity_text.number(),
            message,
            source,
            attributes: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Error)]
pub(crate) enum LogError {
    #[error("json serialize failed: {0}")]
    Json(String),
    #[error("sink write failed: {0}")]
    SinkIo(String),
}

#[derive(Debug, Error)]
pub(crate) enum LogBootstrapError {
    #[error("logging misconfigured: {0}")]
    Misconfigured(String),
    #[error("sink init failed: {0}")]
    SinkInit(String),
}

pub(crate) trait LogSink: Send + Sync {
    fn emit(&self, record: &LogRecord) -> Result<(), LogError>;
}

pub(crate) struct JsonlStderrSink {
    stderr: Mutex<std::io::Stderr>,
}

impl JsonlStderrSink {
    pub(crate) fn new() -> Self {
        Self {
            stderr: Mutex::new(std::io::stderr()),
        }
    }
}

impl LogSink for JsonlStderrSink {
    fn emit(&self, record: &LogRecord) -> Result<(), LogError> {
        let line = serde_json::to_string(record).map_err(|err| LogError::Json(err.to_string()))?;
        let mut stderr = self
            .stderr
            .lock()
            .map_err(|_| LogError::SinkIo("stderr lock poisoned".to_string()))?;
        stderr
            .write_all(line.as_bytes())
            .and_then(|()| stderr.write_all(b"\n"))
            .map_err(|err| LogError::SinkIo(err.to_string()))?;
        Ok(())
    }
}

struct NullSink;

impl LogSink for NullSink {
    fn emit(&self, record: &LogRecord) -> Result<(), LogError> {
        let _ = record;
        Ok(())
    }
}

pub(crate) struct JsonlFileSink {
    path: PathBuf,
    writer: Mutex<LineWriter<File>>,
}

impl JsonlFileSink {
    pub(crate) fn new(path: PathBuf, mode: crate::config::FileSinkMode) -> Result<Self, LogError> {
        if path.as_os_str().is_empty() {
            return Err(LogError::SinkIo("file sink path is empty".to_string()));
        }

        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|err| {
                    LogError::SinkIo(format!(
                        "create log directory {} for {} failed: {err}",
                        parent.display(),
                        path.display()
                    ))
                })?;
            }
        }

        let mut options = OpenOptions::new();
        options.create(true).write(true);
        match mode {
            crate::config::FileSinkMode::Append => {
                options.append(true);
            }
            crate::config::FileSinkMode::Truncate => {
                options.truncate(true);
            }
        }

        let file = options.open(&path).map_err(|err| {
            LogError::SinkIo(format!("open log file {} failed: {err}", path.display()))
        })?;

        Ok(Self {
            path,
            writer: Mutex::new(LineWriter::new(file)),
        })
    }
}

impl LogSink for JsonlFileSink {
    fn emit(&self, record: &LogRecord) -> Result<(), LogError> {
        let line = serde_json::to_string(record).map_err(|err| LogError::Json(err.to_string()))?;
        let mut writer = self
            .writer
            .lock()
            .map_err(|_| LogError::SinkIo("file sink lock poisoned".to_string()))?;
        writer
            .write_all(line.as_bytes())
            .and_then(|()| writer.write_all(b"\n"))
            .map_err(|err| {
                LogError::SinkIo(format!(
                    "write to log file {} failed: {err}",
                    self.path.display()
                ))
            })?;
        Ok(())
    }
}

struct FanoutSink {
    sinks: Vec<(String, Arc<dyn LogSink>)>,
}

static FANOUT_DIAGNOSTIC_ACTIVE: AtomicBool = AtomicBool::new(false);

#[cfg(test)]
static FANOUT_DIAGNOSTIC_COUNT: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

struct AtomicResetGuard<'a> {
    value: &'a AtomicBool,
}

impl Drop for AtomicResetGuard<'_> {
    fn drop(&mut self) {
        self.value.store(false, Ordering::SeqCst);
    }
}

impl FanoutSink {
    fn new(sinks: Vec<(String, Arc<dyn LogSink>)>) -> Self {
        Self { sinks }
    }

    fn write_diagnostic(label: &str, err: &LogError) {
        let acquired = FANOUT_DIAGNOSTIC_ACTIVE
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok();
        if !acquired {
            return;
        }

        let _guard = AtomicResetGuard {
            value: &FANOUT_DIAGNOSTIC_ACTIVE,
        };

        #[cfg(test)]
        {
            FANOUT_DIAGNOSTIC_COUNT.fetch_add(1, Ordering::SeqCst);
        }

        let mut stderr = std::io::stderr();
        let _ = stderr.write_all(b"fanout sink failure: ");
        let _ = stderr.write_all(label.as_bytes());
        let _ = stderr.write_all(b": ");
        let _ = stderr.write_all(err.to_string().as_bytes());
        let _ = stderr.write_all(b"\n");
    }
}

impl LogSink for FanoutSink {
    fn emit(&self, record: &LogRecord) -> Result<(), LogError> {
        let mut ok_count: u64 = 0;
        let mut failures: Vec<(String, String)> = Vec::new();

        for (label, sink) in &self.sinks {
            match sink.emit(record) {
                Ok(()) => {
                    ok_count += 1;
                }
                Err(err) => {
                    Self::write_diagnostic(label.as_str(), &err);
                    failures.push((label.clone(), err.to_string()));
                }
            }
        }

        if ok_count > 0 {
            return Ok(());
        }

        let mut message = "all sinks failed".to_string();
        if !failures.is_empty() {
            message.push_str(": ");
            for (idx, (label, err)) in failures.iter().enumerate() {
                if idx > 0 {
                    message.push_str("; ");
                }
                message.push_str(label.as_str());
                message.push_str(" => ");
                message.push_str(err.as_str());
            }
        }
        Err(LogError::SinkIo(message))
    }
}

const TRACING_LOG_TARGET: &str = "pgtuskmaster::logging::record";

thread_local! {
    static CURRENT_TRACING_RECORD: RefCell<Option<LogRecord>> = const { RefCell::new(None) };
    static CURRENT_TRACING_RESULT: RefCell<Option<Result<(), LogError>>> = const { RefCell::new(None) };
}

struct ActiveTracingRecordGuard;

impl ActiveTracingRecordGuard {
    fn new(record: &LogRecord) -> Result<Self, LogError> {
        CURRENT_TRACING_RECORD.with(|slot| {
            let mut slot = slot.borrow_mut();
            if slot.is_some() {
                return Err(LogError::SinkIo(
                    "nested tracing-backed log emission is not supported".to_string(),
                ));
            }
            *slot = Some(record.clone());
            Ok(())
        })?;
        CURRENT_TRACING_RESULT.with(|slot| {
            *slot.borrow_mut() = None;
        });
        Ok(Self)
    }
}

impl Drop for ActiveTracingRecordGuard {
    fn drop(&mut self) {
        CURRENT_TRACING_RECORD.with(|slot| {
            let _ = slot.borrow_mut().take();
        });
        CURRENT_TRACING_RESULT.with(|slot| {
            let _ = slot.borrow_mut().take();
        });
    }
}

struct TracingRecordLayer {
    sink: Arc<dyn LogSink>,
}

impl<S> Layer<S> for TracingRecordLayer
where
    S: tracing::Subscriber,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        if event.metadata().target() != TRACING_LOG_TARGET {
            return;
        }

        let result = CURRENT_TRACING_RECORD.with(|slot| {
            let slot = slot.borrow();
            match slot.as_ref() {
                Some(record) => self.sink.emit(record),
                None => Err(LogError::SinkIo(
                    "tracing backend event emitted without an active record".to_string(),
                )),
            }
        });

        CURRENT_TRACING_RESULT.with(|slot| {
            *slot.borrow_mut() = Some(result);
        });
    }
}

#[derive(Clone)]
struct TracingBackend {
    dispatch: Dispatch,
}

impl TracingBackend {
    fn new(sink: Arc<dyn LogSink>) -> Self {
        let subscriber = Registry::default().with(TracingRecordLayer { sink });
        Self {
            dispatch: Dispatch::new(subscriber),
        }
    }

    fn emit(&self, record: &LogRecord) -> Result<(), LogError> {
        let _guard = ActiveTracingRecordGuard::new(record)?;
        dispatcher::with_default(&self.dispatch, || emit_tracing_record_event(record));
        CURRENT_TRACING_RESULT.with(|slot| {
            slot.borrow_mut().take().unwrap_or_else(|| {
                Err(LogError::SinkIo(
                    "tracing backend did not produce an emission result".to_string(),
                ))
            })
        })
    }
}

fn emit_tracing_record_event(record: &LogRecord) {
    match record.severity_text {
        SeverityText::Trace => tracing::event!(
            target: TRACING_LOG_TARGET,
            tracing::Level::TRACE,
            origin = record.source.origin.as_str(),
            producer = ?record.source.producer,
            transport = ?record.source.transport,
            parser = ?record.source.parser,
            severity_number = record.severity_number,
            message = record.message.as_str()
        ),
        SeverityText::Debug => tracing::event!(
            target: TRACING_LOG_TARGET,
            tracing::Level::DEBUG,
            origin = record.source.origin.as_str(),
            producer = ?record.source.producer,
            transport = ?record.source.transport,
            parser = ?record.source.parser,
            severity_number = record.severity_number,
            message = record.message.as_str()
        ),
        SeverityText::Info => tracing::event!(
            target: TRACING_LOG_TARGET,
            tracing::Level::INFO,
            origin = record.source.origin.as_str(),
            producer = ?record.source.producer,
            transport = ?record.source.transport,
            parser = ?record.source.parser,
            severity_number = record.severity_number,
            message = record.message.as_str()
        ),
        SeverityText::Warn => tracing::event!(
            target: TRACING_LOG_TARGET,
            tracing::Level::WARN,
            origin = record.source.origin.as_str(),
            producer = ?record.source.producer,
            transport = ?record.source.transport,
            parser = ?record.source.parser,
            severity_number = record.severity_number,
            message = record.message.as_str()
        ),
        SeverityText::Error | SeverityText::Fatal => tracing::event!(
            target: TRACING_LOG_TARGET,
            tracing::Level::ERROR,
            origin = record.source.origin.as_str(),
            producer = ?record.source.producer,
            transport = ?record.source.transport,
            parser = ?record.source.parser,
            severity_number = record.severity_number,
            message = record.message.as_str()
        ),
    }
}

#[derive(Clone)]
pub(crate) struct LogHandle {
    hostname: String,
    backend: Arc<TracingBackend>,
    min_app_severity_number: u8,
}

impl std::fmt::Debug for LogHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LogHandle")
            .field("hostname", &self.hostname)
            .field("min_app_severity_number", &self.min_app_severity_number)
            .finish()
    }
}

impl LogHandle {
    pub(crate) fn new(
        hostname: String,
        sink: Arc<dyn LogSink>,
        min_app_severity: SeverityText,
    ) -> Self {
        Self {
            hostname,
            backend: Arc::new(TracingBackend::new(sink)),
            min_app_severity_number: min_app_severity.number(),
        }
    }

    #[cfg(test)]
    pub(crate) fn null() -> Self {
        Self {
            hostname: "unknown".to_string(),
            backend: Arc::new(TracingBackend::new(Arc::new(NullSink))),
            min_app_severity_number: SeverityText::Trace.number(),
        }
    }

    pub(crate) fn disabled() -> Self {
        Self {
            hostname: "unknown".to_string(),
            backend: Arc::new(TracingBackend::new(Arc::new(NullSink))),
            min_app_severity_number: SeverityText::Trace.number(),
        }
    }

    #[cfg(test)]
    pub(crate) fn emit(
        &self,
        severity_text: SeverityText,
        message: impl Into<String>,
        source: LogSource,
    ) -> Result<(), LogError> {
        if severity_text.number() < self.min_app_severity_number {
            return Ok(());
        }
        let record = LogRecord::new(
            system_now_unix_millis(),
            self.hostname.clone(),
            severity_text,
            message.into(),
            source,
        );
        self.backend.emit(&record)
    }

    pub(crate) fn emit_app_event(
        &self,
        origin: impl Into<String>,
        event: AppEvent,
    ) -> Result<(), LogError> {
        if event.severity().number() < self.min_app_severity_number {
            return Ok(());
        }
        let record = event.into_record(
            system_now_unix_millis(),
            self.hostname.clone(),
            origin.into(),
        );
        self.backend.emit(&record)
    }

    pub(crate) fn emit_raw_record(&self, record: RawRecordBuilder) -> Result<(), LogError> {
        let final_record = record.into_record(system_now_unix_millis(), self.hostname.clone());
        self.emit_record(&final_record)
    }

    pub(crate) fn emit_record(&self, record: &LogRecord) -> Result<(), LogError> {
        self.backend.emit(record)
    }
}

pub(crate) fn system_now_unix_millis() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_millis() as u64,
        Err(_) => 0,
    }
}

pub(crate) fn detect_hostname() -> String {
    match std::env::var("HOSTNAME") {
        Ok(value) if !value.trim().is_empty() => value,
        _ => "unknown".to_string(),
    }
}

pub(crate) struct LoggingSystem {
    pub(crate) handle: LogHandle,
}

pub(crate) fn bootstrap(
    cfg: &crate::config::RuntimeConfig,
) -> Result<LoggingSystem, LogBootstrapError> {
    let hostname = detect_hostname();
    let mut sinks: Vec<(String, Arc<dyn LogSink>)> = Vec::new();

    if cfg.logging.sinks.stderr.enabled {
        sinks.push((
            "stderr".to_string(),
            Arc::new(JsonlStderrSink::new()) as Arc<dyn LogSink>,
        ));
    }

    if cfg.logging.sinks.file.enabled {
        let path = cfg.logging.sinks.file.path.clone().ok_or_else(|| {
            LogBootstrapError::Misconfigured(
                "logging.sinks.file.enabled=true but logging.sinks.file.path is not set"
                    .to_string(),
            )
        })?;

        let label = format!("file:{}", path.display());
        let sink = JsonlFileSink::new(path, cfg.logging.sinks.file.mode)
            .map_err(|err| LogBootstrapError::SinkInit(err.to_string()))?;
        sinks.push((label, Arc::new(sink) as Arc<dyn LogSink>));
    }

    let sink: Arc<dyn LogSink> = match sinks.len() {
        0 => Arc::new(NullSink),
        1 => sinks
            .pop()
            .map(|(_label, sink)| sink)
            .ok_or_else(|| LogBootstrapError::SinkInit("unexpected empty sink list".to_string()))?,
        _ => Arc::new(FanoutSink::new(sinks)),
    };

    Ok(LoggingSystem {
        handle: LogHandle::new(hostname, sink, SeverityText::from(cfg.logging.level)),
    })
}

#[cfg(test)]
#[derive(Clone, Default)]
pub(crate) struct TestSink {
    records: Arc<Mutex<Vec<LogRecord>>>,
}

#[cfg(test)]
impl TestSink {
    pub(crate) fn take(&self) -> Vec<LogRecord> {
        let mut locked = match self.records.lock() {
            Ok(locked) => locked,
            Err(poisoned) => poisoned.into_inner(),
        };
        std::mem::take(&mut *locked)
    }

    pub(crate) fn snapshot(&self) -> Result<Vec<LogRecord>, LogError> {
        let locked = self
            .records
            .lock()
            .map_err(|_| LogError::SinkIo("test sink lock poisoned".to_string()))?;
        Ok(locked.clone())
    }

    pub(crate) fn collect_matching(
        &self,
        matcher: impl Fn(&LogRecord) -> bool,
    ) -> Result<Vec<LogRecord>, LogError> {
        Ok(self.snapshot()?.into_iter().filter(matcher).collect())
    }
}

#[cfg(test)]
impl LogSink for TestSink {
    fn emit(&self, record: &LogRecord) -> Result<(), LogError> {
        let mut locked = self
            .records
            .lock()
            .map_err(|_| LogError::SinkIo("test sink lock poisoned".to_string()))?;
        locked.push(record.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::config::{
        DebugConfig, LogCleanupConfig, LogLevel, LoggingConfig, PostgresLoggingConfig,
        RuntimeConfig,
    };

    fn unique_temp_root(label: &str) -> PathBuf {
        let pid = std::process::id();
        static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let unique = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        std::env::temp_dir().join(format!("pgtuskmaster-{label}-{pid}-{unique}"))
    }

    fn remove_dir_all_if_exists(path: &std::path::Path) -> Result<(), std::io::Error> {
        match std::fs::remove_dir_all(path) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(err),
        }
    }

    fn remove_file_if_exists(path: &std::path::Path) -> Result<(), std::io::Error> {
        match std::fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(err),
        }
    }

    fn sample_record(message: &str) -> LogRecord {
        LogRecord::new(
            1,
            "host-a".to_string(),
            SeverityText::Info,
            message.to_string(),
            LogSource {
                producer: LogProducer::App,
                transport: LogTransport::Internal,
                parser: LogParser::App,
                origin: "test".to_string(),
            },
        )
    }

    fn read_lines(path: &std::path::Path) -> Result<Vec<String>, std::io::Error> {
        let bytes = std::fs::read(path)?;
        let text = String::from_utf8_lossy(&bytes);
        Ok(text
            .lines()
            .map(|line| line.to_string())
            .filter(|line| !line.trim().is_empty())
            .collect())
    }

    fn sample_runtime_config() -> RuntimeConfig {
        crate::test_harness::runtime_config::RuntimeConfigBuilder::new()
            .with_logging(LoggingConfig {
                level: LogLevel::Trace,
                postgres: PostgresLoggingConfig {
                    poll_interval_ms: 50,
                    cleanup: LogCleanupConfig {
                        enabled: false,
                        ..crate::test_harness::runtime_config::sample_postgres_logging_config()
                            .cleanup
                    },
                    ..crate::test_harness::runtime_config::sample_postgres_logging_config()
                },
                ..crate::test_harness::runtime_config::sample_logging_config()
            })
            .with_debug(DebugConfig { enabled: false })
            .build()
    }

    fn test_log_handle(min_app_severity: SeverityText) -> (LogHandle, TestSink) {
        let sink = TestSink::default();
        let sink_dyn: Arc<dyn LogSink> = Arc::new(sink.clone());
        (
            LogHandle::new("host-a".to_string(), sink_dyn, min_app_severity),
            sink,
        )
    }

    #[test]
    fn emit_app_event_encodes_typed_headers_and_fields() -> Result<(), Box<dyn std::error::Error>> {
        let (log, sink) = test_log_handle(SeverityText::Trace);
        let mut event = AppEvent::new(
            SeverityText::Info,
            "runtime starting",
            AppEventHeader::new("runtime.startup.entered", "runtime", "ok"),
        );
        event.fields_mut().insert("scope", "scope-a");
        event.fields_mut().insert("member_count", 3_u64);
        event
            .fields_mut()
            .insert_optional("optional_field", Option::<String>::None);

        log.emit_app_event("runtime::run_node_from_config", event)?;

        let records = sink.take();
        assert_eq!(records.len(), 1);
        let decoded = decode_app_event(&records[0])?;
        assert_eq!(
            decoded.header,
            AppEventHeader::new("runtime.startup.entered", "runtime", "ok")
        );
        assert_eq!(decoded.origin, "runtime::run_node_from_config");
        assert_eq!(decoded.message, "runtime starting");
        assert_eq!(
            decoded.fields.get("scope"),
            Some(&Value::String("scope-a".to_string()))
        );
        assert_eq!(
            decoded.fields.get("member_count"),
            Some(&Value::Number(3_u64.into()))
        );
        assert!(!decoded.fields.contains_key("optional_field"));
        Ok(())
    }

    #[test]
    fn structured_fields_encode_scalars_and_serialized_values(
    ) -> Result<(), Box<dyn std::error::Error>> {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
        #[serde(rename_all = "snake_case")]
        enum DemoState {
            NeedsRetry,
        }

        let mut fields = StructuredFields::new();
        fields.insert("bool_field", true);
        fields.insert("signed_field", -5_i64);
        fields.insert("unsigned_field", 7_u64);
        fields.insert("string_field", "value");
        fields.insert_optional("absent_field", Option::<u64>::None);
        fields.insert_serialized("state", &DemoState::NeedsRetry)?;

        let attributes = fields.into_attributes();
        assert_eq!(attributes.get("bool_field"), Some(&Value::Bool(true)));
        assert_eq!(
            attributes.get("signed_field"),
            Some(&Value::Number((-5_i64).into()))
        );
        assert_eq!(
            attributes.get("unsigned_field"),
            Some(&Value::Number(7_u64.into()))
        );
        assert_eq!(
            attributes.get("string_field"),
            Some(&Value::String("value".to_string()))
        );
        assert_eq!(
            attributes.get("state"),
            Some(&Value::String("needs_retry".to_string()))
        );
        assert!(!attributes.contains_key("absent_field"));
        Ok(())
    }

    #[test]
    fn emit_app_event_respects_min_severity() -> Result<(), Box<dyn std::error::Error>> {
        let (log, sink) = test_log_handle(SeverityText::Warn);
        log.emit_app_event(
            "runtime::run_node_from_config",
            AppEvent::new(
                SeverityText::Info,
                "filtered",
                AppEventHeader::new("runtime.filtered", "runtime", "ok"),
            ),
        )?;
        assert!(sink.take().is_empty());
        Ok(())
    }

    #[test]
    fn subprocess_line_record_builder_encodes_stream_metadata(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let record = SubprocessLineRecord::new(
            LogProducer::PgTool,
            "process_worker",
            "job-1",
            "start_postgres",
            "postgres",
            SubprocessStream::Stderr,
            vec![0xff_u8, 0x00, b'a', 0x80],
        )
        .into_raw_record()?
        .into_record(5, "host-a".to_string());

        assert_eq!(record.source.producer, LogProducer::PgTool);
        assert_eq!(record.source.transport, LogTransport::ChildStderr);
        assert_eq!(record.source.parser, LogParser::Raw);
        assert_eq!(record.source.origin, "process_worker");
        assert_eq!(record.severity_text, SeverityText::Warn);
        assert_eq!(record.message, "non_utf8_bytes_hex=ff006180");
        assert_eq!(
            record.attributes.get("job_id"),
            Some(&Value::String("job-1".to_string()))
        );
        assert_eq!(
            record.attributes.get("stream"),
            Some(&Value::String("stderr".to_string()))
        );
        assert_eq!(
            record.attributes.get("raw_bytes_hex"),
            Some(&Value::String("ff006180".to_string()))
        );
        Ok(())
    }

    #[test]
    fn jsonl_file_sink_creates_parent_dirs_and_writes_jsonl_line(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = unique_temp_root("file-sink-create");
        remove_dir_all_if_exists(&root)?;

        let path = root.join("a").join("b").join("log.jsonl");
        let sink = JsonlFileSink::new(path.clone(), crate::config::FileSinkMode::Append)?;
        sink.emit(&sample_record("hello"))?;
        drop(sink);

        let lines = read_lines(&path)?;
        assert_eq!(lines.len(), 1);
        let v: serde_json::Value = serde_json::from_str(lines[0].as_str())?;
        assert_eq!(v["message"], "hello");

        remove_dir_all_if_exists(&root)?;
        Ok(())
    }

    #[test]
    fn jsonl_file_sink_append_mode_preserves_existing_content(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = unique_temp_root("file-sink-append");
        remove_dir_all_if_exists(&root)?;
        std::fs::create_dir_all(&root)?;

        let path = root.join("log.jsonl");
        std::fs::write(&path, b"{\"pre\":1}\n")?;

        let sink = JsonlFileSink::new(path.clone(), crate::config::FileSinkMode::Append)?;
        sink.emit(&sample_record("post"))?;
        drop(sink);

        let lines = read_lines(&path)?;
        assert_eq!(lines.len(), 2);
        let pre: serde_json::Value = serde_json::from_str(lines[0].as_str())?;
        assert_eq!(pre["pre"], 1);
        let post: serde_json::Value = serde_json::from_str(lines[1].as_str())?;
        assert_eq!(post["message"], "post");

        remove_dir_all_if_exists(&root)?;
        Ok(())
    }

    #[test]
    fn jsonl_file_sink_truncate_mode_replaces_existing_content(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = unique_temp_root("file-sink-truncate");
        remove_dir_all_if_exists(&root)?;
        std::fs::create_dir_all(&root)?;

        let path = root.join("log.jsonl");
        std::fs::write(&path, b"{\"stale\":true}\n{\"stale\":true}\n")?;

        let sink = JsonlFileSink::new(path.clone(), crate::config::FileSinkMode::Truncate)?;
        sink.emit(&sample_record("fresh"))?;
        drop(sink);

        let lines = read_lines(&path)?;
        assert_eq!(lines.len(), 1);
        let v: serde_json::Value = serde_json::from_str(lines[0].as_str())?;
        assert_eq!(v["message"], "fresh");

        remove_dir_all_if_exists(&root)?;
        Ok(())
    }

    #[test]
    fn jsonl_file_sink_errors_when_parent_is_file() -> Result<(), Box<dyn std::error::Error>> {
        let root = unique_temp_root("file-sink-parent-file");
        remove_dir_all_if_exists(&root)?;
        std::fs::create_dir_all(&root)?;

        let not_a_dir = root.join("not_a_dir");
        remove_file_if_exists(&not_a_dir)?;
        let write_res = std::fs::write(&not_a_dir, b"im a file");
        assert!(write_res.is_ok());

        let path = not_a_dir.join("app.jsonl");
        let err = JsonlFileSink::new(path.clone(), crate::config::FileSinkMode::Append);
        assert!(matches!(err, Err(LogError::SinkIo(_))));

        if let Err(LogError::SinkIo(msg)) = err {
            assert!(msg.contains(path.display().to_string().as_str()));
        }

        remove_dir_all_if_exists(&root)?;
        Ok(())
    }

    #[derive(Clone)]
    struct FailSink;

    impl LogSink for FailSink {
        fn emit(&self, _record: &LogRecord) -> Result<(), LogError> {
            Err(LogError::SinkIo("fail sink".to_string()))
        }
    }

    #[test]
    fn fanout_sink_ok_when_any_sink_succeeds_and_emits_diagnostic() {
        FANOUT_DIAGNOSTIC_COUNT.store(0, Ordering::SeqCst);

        let ok = Arc::new(TestSink::default());
        let ok_dyn: Arc<dyn LogSink> = ok;
        let fail_dyn: Arc<dyn LogSink> = Arc::new(FailSink);

        let sink = FanoutSink::new(vec![
            ("ok".to_string(), ok_dyn),
            ("fail".to_string(), fail_dyn),
        ]);

        assert!(sink.emit(&sample_record("x")).is_ok());
        assert!(FANOUT_DIAGNOSTIC_COUNT.load(Ordering::SeqCst) >= 1);
    }

    #[test]
    fn fanout_sink_err_when_all_sinks_fail() {
        let fail_a: Arc<dyn LogSink> = Arc::new(FailSink);
        let fail_b: Arc<dyn LogSink> = Arc::new(FailSink);

        let sink = FanoutSink::new(vec![("a".to_string(), fail_a), ("b".to_string(), fail_b)]);

        let err = sink.emit(&sample_record("x"));
        assert!(matches!(err, Err(LogError::SinkIo(_))));
    }

    #[test]
    fn tracing_backend_preserves_emit_errors_when_sink_fails() {
        let sink: Arc<dyn LogSink> = Arc::new(FailSink);
        let log = LogHandle::new("host-a".to_string(), sink, SeverityText::Trace);

        let err = log.emit(
            SeverityText::Info,
            "should fail",
            LogSource {
                producer: LogProducer::App,
                transport: LogTransport::Internal,
                parser: LogParser::App,
                origin: "test".to_string(),
            },
        );

        assert!(matches!(err, Err(LogError::SinkIo(_))));
    }

    #[test]
    fn tracing_backend_preserves_partial_fanout_success() -> Result<(), Box<dyn std::error::Error>>
    {
        FANOUT_DIAGNOSTIC_COUNT.store(0, Ordering::SeqCst);

        let ok = TestSink::default();
        let ok_records = ok.clone();
        let sink: Arc<dyn LogSink> = Arc::new(FanoutSink::new(vec![
            ("ok".to_string(), Arc::new(ok) as Arc<dyn LogSink>),
            ("fail".to_string(), Arc::new(FailSink) as Arc<dyn LogSink>),
        ]));
        let log = LogHandle::new("host-a".to_string(), sink, SeverityText::Trace);

        log.emit(
            SeverityText::Info,
            "fanout-through-tracing",
            LogSource {
                producer: LogProducer::App,
                transport: LogTransport::Internal,
                parser: LogParser::App,
                origin: "test".to_string(),
            },
        )?;

        let records = ok_records.take();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].message, "fanout-through-tracing");
        assert!(FANOUT_DIAGNOSTIC_COUNT.load(Ordering::SeqCst) >= 1);
        Ok(())
    }

    #[test]
    fn bootstrap_file_enabled_without_path_returns_misconfigured() {
        let mut cfg = sample_runtime_config();
        cfg.logging.sinks.stderr.enabled = false;
        cfg.logging.sinks.file.enabled = true;
        cfg.logging.sinks.file.path = None;

        let res = bootstrap(&cfg);
        assert!(matches!(res, Err(LogBootstrapError::Misconfigured(_))));
    }

    #[test]
    fn bootstrap_file_enabled_with_path_writes_jsonl() -> Result<(), Box<dyn std::error::Error>> {
        let root = unique_temp_root("bootstrap-file-enabled");
        remove_dir_all_if_exists(&root)?;
        std::fs::create_dir_all(&root)?;

        let path = root.join("app.jsonl");

        let mut cfg = sample_runtime_config();
        cfg.logging.sinks.stderr.enabled = false;
        cfg.logging.sinks.file.enabled = true;
        cfg.logging.sinks.file.path = Some(path.clone());

        let system = bootstrap(&cfg)?;
        system.handle.emit(
            SeverityText::Info,
            "hello",
            LogSource {
                producer: LogProducer::App,
                transport: LogTransport::Internal,
                parser: LogParser::App,
                origin: "test".to_string(),
            },
        )?;
        drop(system);

        let lines = read_lines(&path)?;
        assert_eq!(lines.len(), 1);
        let v: serde_json::Value = serde_json::from_str(lines[0].as_str())?;
        assert_eq!(v["message"], "hello");
        assert_eq!(v["severity_text"], "info");

        remove_dir_all_if_exists(&root)?;
        Ok(())
    }

    #[test]
    fn bootstrap_with_stderr_and_file_still_writes_file() -> Result<(), Box<dyn std::error::Error>>
    {
        let root = unique_temp_root("bootstrap-stderr-and-file");
        remove_dir_all_if_exists(&root)?;
        std::fs::create_dir_all(&root)?;

        let path = root.join("app.jsonl");

        let mut cfg = sample_runtime_config();
        cfg.logging.sinks.stderr.enabled = true;
        cfg.logging.sinks.file.enabled = true;
        cfg.logging.sinks.file.path = Some(path.clone());

        let system = bootstrap(&cfg)?;
        system.handle.emit(
            SeverityText::Info,
            "fanout",
            LogSource {
                producer: LogProducer::App,
                transport: LogTransport::Internal,
                parser: LogParser::App,
                origin: "test".to_string(),
            },
        )?;
        drop(system);

        let lines = read_lines(&path)?;
        assert_eq!(lines.len(), 1);
        let v: serde_json::Value = serde_json::from_str(lines[0].as_str())?;
        assert_eq!(v["message"], "fanout");

        remove_dir_all_if_exists(&root)?;
        Ok(())
    }

    #[test]
    fn bootstrap_with_all_sinks_disabled_is_non_fatal() -> Result<(), LogBootstrapError> {
        let mut cfg = sample_runtime_config();
        cfg.logging.sinks.stderr.enabled = false;
        cfg.logging.sinks.file.enabled = false;

        let system = bootstrap(&cfg)?;
        let record = sample_record("dropped");
        let res = system.handle.emit_record(&record);
        assert!(res.is_ok(), "expected null sink to accept record: {res:?}");
        Ok(())
    }
}

--- END FILE: src/logging/mod.rs ---

--- BEGIN FILE: src/logging/event.rs ---
use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::Value;

use super::{LogError, LogParser, LogProducer, LogRecord, LogSource, LogTransport, SeverityText};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AppEventHeader {
    pub(crate) name: String,
    pub(crate) domain: String,
    pub(crate) result: String,
}

impl AppEventHeader {
    pub(crate) fn new(
        name: impl Into<String>,
        domain: impl Into<String>,
        result: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            domain: domain.into(),
            result: result.into(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct StructuredFields {
    fields: Vec<(String, StructuredValue)>,
}

impl StructuredFields {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn append_json_map(&mut self, attributes: BTreeMap<String, Value>) {
        self.fields.extend(
            attributes
                .into_iter()
                .map(|(key, value)| (key, StructuredValue::Json(value))),
        );
    }

    pub(crate) fn insert<V>(&mut self, key: impl Into<String>, value: V)
    where
        V: Into<StructuredValue>,
    {
        self.fields.push((key.into(), value.into()));
    }

    pub(crate) fn insert_optional<V>(&mut self, key: impl Into<String>, value: Option<V>)
    where
        V: Into<StructuredValue>,
    {
        if let Some(value) = value {
            self.insert(key, value);
        }
    }

    pub(crate) fn insert_serialized<T: Serialize>(
        &mut self,
        key: impl Into<String>,
        value: &T,
    ) -> Result<(), LogError> {
        let json_value =
            serde_json::to_value(value).map_err(|err| LogError::Json(err.to_string()))?;
        self.fields
            .push((key.into(), StructuredValue::Json(json_value)));
        Ok(())
    }

    pub(crate) fn into_attributes(self) -> BTreeMap<String, Value> {
        let mut attributes = BTreeMap::new();
        for (key, value) in self.fields {
            attributes.insert(key, value.into_json());
        }
        attributes
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum StructuredValue {
    Bool(bool),
    I64(i64),
    U64(u64),
    String(String),
    Json(Value),
}

impl StructuredValue {
    fn into_json(self) -> Value {
        match self {
            Self::Bool(value) => Value::Bool(value),
            Self::I64(value) => Value::Number(value.into()),
            Self::U64(value) => Value::Number(value.into()),
            Self::String(value) => Value::String(value),
            Self::Json(value) => value,
        }
    }
}

impl From<bool> for StructuredValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<i16> for StructuredValue {
    fn from(value: i16) -> Self {
        Self::I64(i64::from(value))
    }
}

impl From<i32> for StructuredValue {
    fn from(value: i32) -> Self {
        Self::I64(i64::from(value))
    }
}

impl From<i64> for StructuredValue {
    fn from(value: i64) -> Self {
        Self::I64(value)
    }
}

impl From<u16> for StructuredValue {
    fn from(value: u16) -> Self {
        Self::U64(u64::from(value))
    }
}

impl From<u32> for StructuredValue {
    fn from(value: u32) -> Self {
        Self::U64(u64::from(value))
    }
}

impl From<u64> for StructuredValue {
    fn from(value: u64) -> Self {
        Self::U64(value)
    }
}

impl From<usize> for StructuredValue {
    fn from(value: usize) -> Self {
        Self::U64(value as u64)
    }
}

impl From<&str> for StructuredValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<String> for StructuredValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<Value> for StructuredValue {
    fn from(value: Value) -> Self {
        Self::Json(value)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AppEvent {
    header: AppEventHeader,
    severity: SeverityText,
    message: String,
    fields: StructuredFields,
}

impl AppEvent {
    pub(crate) fn new(
        severity: SeverityText,
        message: impl Into<String>,
        header: AppEventHeader,
    ) -> Self {
        Self {
            header,
            severity,
            message: message.into(),
            fields: StructuredFields::new(),
        }
    }

    pub(crate) fn severity(&self) -> SeverityText {
        self.severity
    }

    pub(crate) fn fields_mut(&mut self) -> &mut StructuredFields {
        &mut self.fields
    }

    pub(crate) fn into_record(
        self,
        timestamp_ms: u64,
        hostname: String,
        origin: impl Into<String>,
    ) -> LogRecord {
        let source = LogSource {
            producer: LogProducer::App,
            transport: LogTransport::Internal,
            parser: LogParser::App,
            origin: origin.into(),
        };
        let mut record =
            LogRecord::new(timestamp_ms, hostname, self.severity, self.message, source);
        let mut attributes = self.fields.into_attributes();
        attributes.insert("event.name".to_string(), Value::String(self.header.name));
        attributes.insert(
            "event.domain".to_string(),
            Value::String(self.header.domain),
        );
        attributes.insert(
            "event.result".to_string(),
            Value::String(self.header.result),
        );
        record.attributes = attributes;
        record
    }
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DecodedAppEvent {
    pub(crate) header: AppEventHeader,
    pub(crate) severity: SeverityText,
    pub(crate) message: String,
    pub(crate) origin: String,
    pub(crate) fields: BTreeMap<String, Value>,
}

#[cfg(test)]
pub(crate) fn decode_app_event(record: &LogRecord) -> Result<DecodedAppEvent, LogError> {
    let name = decode_required_string_attribute(record, "event.name")?;
    let domain = decode_required_string_attribute(record, "event.domain")?;
    let result = decode_required_string_attribute(record, "event.result")?;
    let mut fields = record.attributes.clone();
    fields.remove("event.name");
    fields.remove("event.domain");
    fields.remove("event.result");

    Ok(DecodedAppEvent {
        header: AppEventHeader::new(name, domain, result),
        severity: record.severity_text,
        message: record.message.clone(),
        origin: record.source.origin.clone(),
        fields,
    })
}

#[cfg(test)]
fn decode_required_string_attribute(record: &LogRecord, key: &str) -> Result<String, LogError> {
    match record.attributes.get(key) {
        Some(Value::String(value)) => Ok(value.clone()),
        Some(other) => Err(LogError::Json(format!(
            "attribute `{key}` should be a string, got {other}"
        ))),
        None => Err(LogError::Json(format!(
            "attribute `{key}` is missing from app event record"
        ))),
    }
}

--- END FILE: src/logging/event.rs ---

--- BEGIN FILE: src/logging/raw_record.rs ---
use serde::Serialize;

use super::{
    LogError, LogParser, LogProducer, LogRecord, LogSource, LogTransport, SeverityText,
    StructuredFields,
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RawRecordBuilder {
    severity: SeverityText,
    message: String,
    source: LogSource,
    fields: StructuredFields,
}

impl RawRecordBuilder {
    pub(crate) fn new(
        severity: SeverityText,
        message: impl Into<String>,
        source: LogSource,
    ) -> Self {
        Self {
            severity,
            message: message.into(),
            source,
            fields: StructuredFields::new(),
        }
    }

    pub(crate) fn with_fields(mut self, fields: StructuredFields) -> Self {
        self.fields = fields;
        self
    }

    pub(crate) fn into_record(self, timestamp_ms: u64, hostname: String) -> LogRecord {
        let mut record = LogRecord::new(
            timestamp_ms,
            hostname,
            self.severity,
            self.message,
            self.source,
        );
        record.attributes = self.fields.into_attributes();
        record
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SubprocessStream {
    Stdout,
    Stderr,
}

impl SubprocessStream {
    fn severity(self) -> SeverityText {
        match self {
            Self::Stdout => SeverityText::Info,
            Self::Stderr => SeverityText::Warn,
        }
    }

    fn transport(self) -> LogTransport {
        match self {
            Self::Stdout => LogTransport::ChildStdout,
            Self::Stderr => LogTransport::ChildStderr,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SubprocessLineRecord {
    producer: LogProducer,
    origin: String,
    job_id: String,
    job_kind: String,
    binary: String,
    stream: SubprocessStream,
    bytes: Vec<u8>,
}

impl SubprocessLineRecord {
    pub(crate) fn new(
        producer: LogProducer,
        origin: impl Into<String>,
        job_id: impl Into<String>,
        job_kind: impl Into<String>,
        binary: impl Into<String>,
        stream: SubprocessStream,
        bytes: Vec<u8>,
    ) -> Self {
        Self {
            producer,
            origin: origin.into(),
            job_id: job_id.into(),
            job_kind: job_kind.into(),
            binary: binary.into(),
            stream,
            bytes,
        }
    }

    pub(crate) fn into_raw_record(self) -> Result<RawRecordBuilder, LogError> {
        let (message, raw_bytes_hex) = decode_bytes(self.bytes);
        let source = LogSource {
            producer: self.producer,
            transport: self.stream.transport(),
            parser: LogParser::Raw,
            origin: self.origin,
        };
        let mut fields = StructuredFields::new();
        fields.insert("job_id", self.job_id);
        fields.insert("job_kind", self.job_kind);
        fields.insert("binary", self.binary);
        fields.insert_serialized("stream", &self.stream)?;
        fields.insert_optional("raw_bytes_hex", raw_bytes_hex);
        Ok(RawRecordBuilder::new(self.stream.severity(), message, source).with_fields(fields))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PostgresLineRecordBuilder {
    producer: LogProducer,
    transport: LogTransport,
    origin: String,
}

impl PostgresLineRecordBuilder {
    pub(crate) fn new(
        producer: LogProducer,
        transport: LogTransport,
        origin: impl Into<String>,
    ) -> Self {
        Self {
            producer,
            transport,
            origin: origin.into(),
        }
    }

    pub(crate) fn build(
        self,
        parser: LogParser,
        severity: SeverityText,
        message: impl Into<String>,
        fields: StructuredFields,
    ) -> RawRecordBuilder {
        RawRecordBuilder::new(
            severity,
            message,
            LogSource {
                producer: self.producer,
                transport: self.transport,
                parser,
                origin: self.origin,
            },
        )
        .with_fields(fields)
    }
}

fn decode_bytes(bytes: Vec<u8>) -> (String, Option<String>) {
    match String::from_utf8(bytes) {
        Ok(message) => (message, None),
        Err(err) => {
            let raw_bytes = err.into_bytes();
            let hex = hex_encode(raw_bytes.as_slice());
            (format!("non_utf8_bytes_hex={hex}"), Some(hex))
        }
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        out.push(TABLE[(byte >> 4) as usize] as char);
        out.push(TABLE[(byte & 0x0f) as usize] as char);
    }
    out
}

--- END FILE: src/logging/raw_record.rs ---

--- BEGIN FILE: src/logging/tailer.rs ---
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use tokio::io::{AsyncReadExt, AsyncSeekExt};

use crate::state::WorkerError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StartPosition {
    Beginning,
    End,
}

#[derive(Clone, Debug)]
pub(crate) struct FileTailer {
    path: PathBuf,
    start: StartPosition,
    offset: Option<u64>,
    pending: Vec<u8>,
    #[cfg(unix)]
    last_inode: Option<u64>,
}

impl FileTailer {
    pub(crate) fn new(path: PathBuf, start: StartPosition) -> Self {
        Self {
            path,
            start,
            offset: None,
            pending: Vec::new(),
            #[cfg(unix)]
            last_inode: None,
        }
    }

    pub(crate) fn path(&self) -> &Path {
        self.path.as_path()
    }

    pub(crate) async fn read_new_lines(
        &mut self,
        max_bytes: usize,
    ) -> Result<Vec<Vec<u8>>, WorkerError> {
        if max_bytes == 0 {
            return Ok(Vec::new());
        }

        let meta = match tokio::fs::metadata(&self.path).await {
            Ok(meta) => meta,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                self.offset = None;
                self.pending.clear();
                #[cfg(unix)]
                {
                    self.last_inode = None;
                }
                return Ok(Vec::new());
            }
            Err(err) => {
                return Err(WorkerError::Message(format!(
                    "tailer metadata failed for {}: {err}",
                    self.path.display()
                )));
            }
        };

        #[cfg(unix)]
        let inode = Some(std::os::unix::fs::MetadataExt::ino(&meta));

        #[cfg(not(unix))]
        let inode: Option<u64> = None;

        let len = meta.len();

        let should_reset_for_rotation = {
            #[cfg(unix)]
            {
                if let (Some(prev), Some(now)) = (self.last_inode, inode) {
                    prev != now
                } else {
                    false
                }
            }
            #[cfg(not(unix))]
            {
                false
            }
        };

        if should_reset_for_rotation {
            self.offset = None;
            self.pending.clear();
        }

        let offset = match self.offset {
            Some(offset) => {
                if len < offset {
                    // truncation
                    0
                } else {
                    offset
                }
            }
            None => match self.start {
                StartPosition::Beginning => 0,
                StartPosition::End => len,
            },
        };

        let mut file = tokio::fs::File::open(&self.path).await.map_err(|err| {
            WorkerError::Message(format!("open failed for {}: {err}", self.path.display()))
        })?;
        file.seek(std::io::SeekFrom::Start(offset))
            .await
            .map_err(|err| {
                WorkerError::Message(format!("seek failed for {}: {err}", self.path.display()))
            })?;

        let mut out = Vec::new();
        let mut read_total = 0usize;
        let mut buf = vec![0u8; 8192];

        while read_total < max_bytes {
            let budget = max_bytes.saturating_sub(read_total);
            let chunk_len = buf.len().min(budget);
            let n = file.read(&mut buf[..chunk_len]).await.map_err(|err| {
                WorkerError::Message(format!("read failed for {}: {err}", self.path.display()))
            })?;
            if n == 0 {
                break;
            }
            read_total = read_total.saturating_add(n);
            self.pending.extend_from_slice(&buf[..n]);

            while let Some(pos) = self.pending.iter().position(|b| *b == b'\n') {
                let mut line = self.pending.drain(..=pos).collect::<Vec<u8>>();
                if let Some(b'\n') = line.last() {
                    line.pop();
                }
                if let Some(b'\r') = line.last() {
                    line.pop();
                }
                out.push(line);
            }
        }

        let new_offset = offset.saturating_add(read_total as u64);
        self.offset = Some(new_offset);
        #[cfg(unix)]
        {
            self.last_inode = inode;
        }
        Ok(out)
    }
}

#[derive(Default)]
pub(crate) struct DirTailers {
    tailers: BTreeMap<PathBuf, FileTailer>,
}

impl DirTailers {
    pub(crate) fn ensure_file(&mut self, path: PathBuf, start: StartPosition) {
        self.tailers
            .entry(path.clone())
            .or_insert_with(|| FileTailer::new(path, start));
    }

    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = (&PathBuf, &mut FileTailer)> {
        self.tailers.iter_mut()
    }

    pub(crate) fn len(&self) -> usize {
        self.tailers.len()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::{FileTailer, StartPosition};

    fn tmp_dir(label: &str) -> PathBuf {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "pgtuskmaster-tailer-{label}-{}-{unique}",
            std::process::id(),
        ))
    }

    #[tokio::test(flavor = "current_thread")]
    async fn file_tailer_reads_appends_and_handles_rotation(
    ) -> Result<(), crate::state::WorkerError> {
        let dir = tmp_dir("rotation");
        match std::fs::remove_dir_all(&dir) {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => {
                return Err(crate::state::WorkerError::Message(format!(
                    "remove_dir_all failed: {err}"
                )));
            }
        }
        std::fs::create_dir_all(&dir).map_err(|err| {
            crate::state::WorkerError::Message(format!("create_dir_all failed: {err}"))
        })?;

        let path = dir.join("postgres.log");
        tokio::fs::write(&path, b"a\n")
            .await
            .map_err(|err| crate::state::WorkerError::Message(format!("write failed: {err}")))?;

        let mut tailer = FileTailer::new(path.clone(), StartPosition::Beginning);
        let first = tailer.read_new_lines(1024).await?;
        assert_eq!(first, vec![b"a".to_vec()]);

        tokio::fs::write(&path, b"a\nb\n")
            .await
            .map_err(|err| crate::state::WorkerError::Message(format!("append failed: {err}")))?;
        let second = tailer.read_new_lines(1024).await?;
        assert_eq!(second, vec![b"b".to_vec()]);

        let rotated = dir.join("postgres.log.1");
        tokio::fs::rename(&path, &rotated)
            .await
            .map_err(|err| crate::state::WorkerError::Message(format!("rename failed: {err}")))?;
        tokio::fs::write(&path, b"c\n").await.map_err(|err| {
            crate::state::WorkerError::Message(format!("new file write failed: {err}"))
        })?;

        let third = tailer.read_new_lines(1024).await?;
        assert_eq!(third, vec![b"c".to_vec()]);

        match std::fs::remove_dir_all(&dir) {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => {
                return Err(crate::state::WorkerError::Message(format!(
                    "remove_dir_all cleanup failed: {err}"
                )));
            }
        }
        Ok(())
    }
}

--- END FILE: src/logging/tailer.rs ---

--- BEGIN FILE: src/logging/postgres_ingest.rs ---
use std::collections::BTreeMap;
use std::path::Path;
use std::time::{Duration, SystemTime};

use serde_json::Value;

use crate::config::{LogCleanupConfig, RuntimeConfig};
use crate::logging::{
    AppEvent, AppEventHeader, LogHandle, LogParser, LogProducer, LogTransport,
    PostgresLineRecordBuilder, RawRecordBuilder, SeverityText, StructuredFields,
};
use crate::state::WorkerError;

use super::tailer::{DirTailers, FileTailer, StartPosition};

pub(crate) struct PostgresIngestWorkerCtx {
    pub(crate) cfg: RuntimeConfig,
    pub(crate) log: LogHandle,
}

const POSTGRES_INGEST_ERROR_RATE_LIMIT_WINDOW_MS: u64 = 30_000;
const POSTGRES_INGEST_MAX_BYTES_PER_FILE: usize = 256 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct IngestErrorKey {
    stage: String,
    kind: String,
    path: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RateLimitDecision {
    emit: bool,
    suppressed: u64,
}

#[derive(Clone, Debug)]
struct RateLimitState {
    last_emit_ms: u64,
    suppressed: u64,
}

#[derive(Clone, Debug)]
struct IngestErrorRateLimiter {
    window_ms: u64,
    by_key: BTreeMap<IngestErrorKey, RateLimitState>,
}

impl IngestErrorRateLimiter {
    fn new(window_ms: u64) -> Self {
        Self {
            window_ms,
            by_key: BTreeMap::new(),
        }
    }

    fn record(&mut self, key: IngestErrorKey, now_ms: u64) -> RateLimitDecision {
        match self.by_key.get_mut(&key) {
            None => {
                self.by_key.insert(
                    key,
                    RateLimitState {
                        last_emit_ms: now_ms,
                        suppressed: 0,
                    },
                );
                RateLimitDecision {
                    emit: true,
                    suppressed: 0,
                }
            }
            Some(entry) => {
                let elapsed_ms = now_ms.saturating_sub(entry.last_emit_ms);
                if elapsed_ms >= self.window_ms {
                    let suppressed = entry.suppressed;
                    entry.last_emit_ms = now_ms;
                    entry.suppressed = 0;
                    RateLimitDecision {
                        emit: true,
                        suppressed,
                    }
                } else {
                    entry.suppressed = entry.suppressed.saturating_add(1);
                    RateLimitDecision {
                        emit: false,
                        suppressed: 0,
                    }
                }
            }
        }
    }
}

pub(crate) async fn run(ctx: PostgresIngestWorkerCtx) -> Result<(), WorkerError> {
    let mut state = PostgresIngestWorkerState::new(&ctx.cfg);
    let mut limiter = IngestErrorRateLimiter::new(POSTGRES_INGEST_ERROR_RATE_LIMIT_WINDOW_MS);
    let mut consecutive_failures = 0u32;
    loop {
        if ctx.cfg.logging.postgres.enabled {
            match step_once(&ctx, &mut state).await {
                Ok(()) => {
                    if consecutive_failures > 0 {
                        emit_ingest_retry_recovered(&ctx.log, consecutive_failures)?;
                        consecutive_failures = 0;
                    }
                }
                Err(error) => {
                    consecutive_failures = consecutive_failures.saturating_add(1);
                    let now_ms = crate::logging::system_now_unix_millis();
                    let key = ingest_error_key_best_effort(&error);
                    let decision = limiter.record(key, now_ms);
                    if decision.emit {
                        emit_ingest_step_failure(
                            &ctx.log,
                            &error,
                            consecutive_failures,
                            decision.suppressed,
                        )?;
                    }
                }
            }
        }
        tokio::time::sleep(Duration::from_millis(
            ctx.cfg.logging.postgres.poll_interval_ms,
        ))
        .await;
    }
}

fn ingest_error_key_best_effort(error: &WorkerError) -> IngestErrorKey {
    let msg = error.to_string();

    let mut stage = "unknown".to_string();
    let mut kind = "unknown".to_string();
    let mut path = "unknown".to_string();

    for token in msg.split_whitespace() {
        if stage == "unknown" {
            if let Some(value) = token.strip_prefix("stage=") {
                stage = value.to_string();
                continue;
            }
        }
        if kind == "unknown" {
            if let Some(value) = token.strip_prefix("kind=") {
                kind = value.to_string();
                continue;
            }
        }
        if path == "unknown" {
            if let Some(value) = token.strip_prefix("path=") {
                path = value.to_string();
                continue;
            }
        }
        if stage != "unknown" && kind != "unknown" && path != "unknown" {
            break;
        }
    }

    IngestErrorKey { stage, kind, path }
}

fn ingest_event(severity: SeverityText, message: &str, name: &str, result: &str) -> AppEvent {
    AppEvent::new(
        severity,
        message,
        AppEventHeader::new(name, "postgres_ingest", result),
    )
}

fn emit_ingest_event(
    log: &LogHandle,
    origin: &str,
    event: AppEvent,
    error_prefix: &str,
) -> Result<(), WorkerError> {
    log.emit_app_event(origin, event)
        .map_err(|err| WorkerError::Message(format!("{error_prefix}: {err}")))
}

fn emit_ingest_step_failure(
    log: &LogHandle,
    error: &WorkerError,
    attempts: u32,
    suppressed: u64,
) -> Result<(), WorkerError> {
    let mut event = ingest_event(
        SeverityText::Error,
        "postgres ingest step_once failed",
        "postgres_ingest.step_once_failed",
        "failed",
    );
    let fields = event.fields_mut();
    fields.insert("attempts", attempts);
    fields.insert("suppressed", suppressed);
    fields.insert("error", error.to_string());
    emit_ingest_event(
        log,
        "postgres_ingest::run",
        event,
        "postgres ingest error log emit failed",
    )
}

fn emit_ingest_retry_recovered(log: &LogHandle, attempts: u32) -> Result<(), WorkerError> {
    let mut event = ingest_event(
        SeverityText::Info,
        "postgres ingest recovered",
        "postgres_ingest.recovered",
        "recovered",
    );
    event.fields_mut().insert("attempts", attempts);
    emit_ingest_event(
        log,
        "postgres_ingest::run",
        event,
        "postgres ingest recovered log emit failed",
    )
}

struct PostgresIngestWorkerState {
    pg_ctl_log: FileTailer,
    dir_tailers: DirTailers,
}

impl PostgresIngestWorkerState {
    fn new(cfg: &RuntimeConfig) -> Self {
        let pg_ctl_log_file = match cfg.logging.postgres.pg_ctl_log_file.clone() {
            Some(path) => path,
            None => cfg.postgres.log_file.clone(),
        };

        Self {
            pg_ctl_log: FileTailer::new(pg_ctl_log_file, StartPosition::Beginning),
            dir_tailers: DirTailers::default(),
        }
    }
}

async fn step_once(
    ctx: &PostgresIngestWorkerCtx,
    state: &mut PostgresIngestWorkerState,
) -> Result<(), WorkerError> {
    let max_bytes_per_file = POSTGRES_INGEST_MAX_BYTES_PER_FILE;
    let mut pg_ctl_lines_emitted: u64 = 0;
    let mut log_dir_lines_emitted: u64 = 0;
    let mut log_dir_files_tailed: u64 = 0;

    #[derive(Clone, Debug)]
    struct IterationIssue {
        stage: &'static str,
        kind: &'static str,
        path: String,
        error: String,
    }

    fn encode_path_token(path: &Path) -> String {
        path.display().to_string().replace(' ', "%20")
    }

    fn file_name_best_effort(path: &Path) -> String {
        match path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => "log".to_string(),
        }
    }

    fn push_issue(
        issues: &mut Vec<IterationIssue>,
        stage: &'static str,
        kind: &'static str,
        path: &Path,
        error: WorkerError,
    ) {
        issues.push(IterationIssue {
            stage,
            kind,
            path: encode_path_token(path),
            error: error.to_string(),
        });
    }

    let mut issues: Vec<IterationIssue> = Vec::new();

    match state.pg_ctl_log.read_new_lines(max_bytes_per_file).await {
        Ok(pg_lines) => {
            for line in pg_lines {
                if let Err(err) = emit_postgres_line(
                    &ctx.log,
                    LogProducer::Postgres,
                    LogTransport::FileTail,
                    "pg_ctl_log_file",
                    state.pg_ctl_log.path(),
                    line,
                ) {
                    push_issue(
                        &mut issues,
                        "pg_ctl_log_file.emit",
                        "log.emit_record",
                        state.pg_ctl_log.path(),
                        err,
                    );
                } else {
                    pg_ctl_lines_emitted = pg_ctl_lines_emitted.saturating_add(1);
                }
            }
        }
        Err(err) => {
            push_issue(
                &mut issues,
                "pg_ctl_log_file.read",
                "tailer.read_new_lines",
                state.pg_ctl_log.path(),
                err,
            );
        }
    }

    if let Some(dir) = ctx.cfg.logging.postgres.log_dir.as_ref() {
        if let Err(err) = discover_log_dir(&mut state.dir_tailers, dir).await {
            push_issue(&mut issues, "log_dir.discover", "read_dir", dir, err);
        }

        for (path, tailer) in state.dir_tailers.iter_mut() {
            log_dir_files_tailed = log_dir_files_tailed.saturating_add(1);
            let origin = format!("postgres_log_dir:{}", file_name_best_effort(path));
            match tailer.read_new_lines(max_bytes_per_file).await {
                Ok(lines) => {
                    for line in lines {
                        if let Err(err) = emit_postgres_line(
                            &ctx.log,
                            LogProducer::Postgres,
                            LogTransport::FileTail,
                            origin.as_str(),
                            tailer.path(),
                            line,
                        ) {
                            push_issue(
                                &mut issues,
                                "log_dir.emit",
                                "log.emit_record",
                                tailer.path(),
                                err,
                            );
                        } else {
                            log_dir_lines_emitted = log_dir_lines_emitted.saturating_add(1);
                        }
                    }
                }
                Err(err) => {
                    push_issue(
                        &mut issues,
                        "log_dir.read",
                        "tailer.read_new_lines",
                        tailer.path(),
                        err,
                    );
                }
            }
        }

        if ctx.cfg.logging.postgres.cleanup.enabled {
            let protected: Vec<&Path> = vec![state.pg_ctl_log.path()];

            match cleanup_log_dir(
                dir,
                &ctx.cfg.logging.postgres.cleanup,
                protected.as_slice(),
                SystemTime::now(),
            )
            .await
            {
                Ok(report) => {
                    if report.issue_count > 0 {
                        let stage = "log_dir.cleanup";
                        let kind = "cleanup.issues";
                        let error = WorkerError::Message(format!(
                            "cleanup had issues: issue_count={} first={}",
                            report.issue_count, report.first_issue
                        ));
                        push_issue(&mut issues, stage, kind, dir, error);
                    }
                }
                Err(err) => {
                    push_issue(&mut issues, "log_dir.cleanup", "cleanup.fatal", dir, err);
                }
            }
        }
    }

    if issues.is_empty() {
        let mut event = ingest_event(
            SeverityText::Debug,
            "postgres ingest iteration ok",
            "postgres_ingest.iteration",
            "ok",
        );
        let fields = event.fields_mut();
        fields.insert("pg_ctl_lines_emitted", pg_ctl_lines_emitted);
        fields.insert("log_dir_files_tailed", log_dir_files_tailed);
        fields.insert("log_dir_lines_emitted", log_dir_lines_emitted);
        fields.insert("dir_tailers", state.dir_tailers.len());
        emit_ingest_event(
            &ctx.log,
            "postgres_ingest::step_once",
            event,
            "postgres ingest debug log emit failed",
        )?;
        return Ok(());
    }

    let first = match issues.first() {
        Some(first) => format!(
            "stage={} kind={} path={} error={}",
            first.stage, first.kind, first.path, first.error
        ),
        None => "stage=unknown kind=unknown path=unknown error=unknown".to_string(),
    };

    let mut extra = Vec::new();
    for issue in issues.iter().skip(1).take(2) {
        extra.push(format!(
            "stage={} kind={} path={} error={}",
            issue.stage, issue.kind, issue.path, issue.error
        ));
    }
    let extra_suffix = if extra.is_empty() {
        String::new()
    } else {
        format!(" extra=[{}]", extra.join(" | "))
    };

    Err(WorkerError::Message(format!(
        "postgres_ingest iteration_errors count={} {}{}",
        issues.len(),
        first,
        extra_suffix
    )))
}

async fn discover_log_dir(tailers: &mut DirTailers, dir: &Path) -> Result<(), WorkerError> {
    let mut entries = match tokio::fs::read_dir(dir).await {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(err) => {
            return Err(WorkerError::Message(format!(
                "read_dir failed for {}: {err}",
                dir.display()
            )));
        }
    };

    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|err| WorkerError::Message(format!("read_dir entry failed: {err}")))?
    {
        let path = entry.path();
        let is_file = match entry.file_type().await {
            Ok(ft) => ft.is_file(),
            Err(err) => {
                return Err(WorkerError::Message(format!(
                    "stage=log_dir.discover kind=file_type path={} error={err}",
                    path.display()
                )));
            }
        };
        if !is_file {
            continue;
        }

        let matches = matches!(
            path.extension().and_then(|s| s.to_str()),
            Some("log") | Some("json")
        );
        if !matches {
            continue;
        }

        let start = match path.file_name().and_then(|s| s.to_str()) {
            Some("postgres.stderr.log") | Some("postgres.stdout.log") => StartPosition::Beginning,
            _ => StartPosition::End,
        };
        tailers.ensure_file(path, start);
    }
    Ok(())
}

async fn cleanup_log_dir(
    dir: &Path,
    cleanup: &LogCleanupConfig,
    protected_paths: &[&Path],
    now: SystemTime,
) -> Result<CleanupReport, WorkerError> {
    let mut entries = match tokio::fs::read_dir(dir).await {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(CleanupReport::empty()),
        Err(err) => {
            return Err(WorkerError::Message(format!(
                "cleanup read_dir failed for {}: {err}",
                dir.display()
            )));
        }
    };

    let protected_basenames: [&str; 3] = [
        "postgres.json",
        "postgres.stderr.log",
        "postgres.stdout.log",
    ];

    let mut issues: Vec<String> = Vec::new();
    let mut candidates = Vec::new();
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|err| WorkerError::Message(format!("cleanup readdir entry failed: {err}")))?
    {
        let path = entry.path();
        let is_file = match entry.file_type().await {
            Ok(ft) => ft.is_file(),
            Err(err) => {
                return Err(WorkerError::Message(format!(
                    "stage=cleanup.file_type kind=file_type path={} error={err}",
                    path.display()
                )));
            }
        };
        if !is_file {
            continue;
        }

        let matches = matches!(
            path.extension().and_then(|s| s.to_str()),
            Some("log") | Some("json")
        );
        if !matches {
            continue;
        }

        let mut protected = false;
        for p in protected_paths {
            if path.as_path() == *p {
                protected = true;
                break;
            }
        }

        let file_name = match path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => String::new(),
        };
        if protected_basenames.contains(&file_name.as_str()) {
            protected = true;
        }

        let meta = match entry.metadata().await {
            Ok(meta) => meta,
            Err(err) => {
                protected = true;
                issues.push(format!(
                    "stage=cleanup.metadata kind=metadata path={} error={err}",
                    path.display()
                ));
                candidates.push((path, None, protected));
                continue;
            }
        };
        let modified = match meta.modified() {
            Ok(modified) => Some(modified),
            Err(err) => {
                protected = true;
                issues.push(format!(
                    "stage=cleanup.modified kind=modified path={} error={err}",
                    path.display()
                ));
                candidates.push((path, None, protected));
                continue;
            }
        };

        if !protected {
            let is_recent = match modified {
                Some(modified) => match now.duration_since(modified) {
                    Ok(age) => age.as_secs() <= cleanup.protect_recent_seconds,
                    Err(err) => {
                        issues.push(format!(
                            "stage=cleanup.age kind=duration_since path={} error={err}",
                            path.display()
                        ));
                        true
                    }
                },
                None => true,
            };
            if is_recent {
                protected = true;
            }
        }

        candidates.push((path, modified, protected));
    }

    let mut eligible = candidates
        .iter()
        .filter_map(|(path, modified, protected)| {
            if *protected {
                return None;
            }
            modified.map(|modified| (path.clone(), modified))
        })
        .collect::<Vec<_>>();

    eligible.sort_by(|a, b| {
        let by_time = a.1.cmp(&b.1);
        if by_time != std::cmp::Ordering::Equal {
            return by_time;
        }
        a.0.cmp(&b.0)
    });

    let mut to_remove: Vec<std::path::PathBuf> = Vec::new();

    if cleanup.max_files > 0 && (eligible.len() as u64) > cleanup.max_files {
        let remove_count = eligible.len().saturating_sub(cleanup.max_files as usize);
        for (path, _) in eligible.iter().take(remove_count) {
            to_remove.push(path.clone());
        }
    }

    if cleanup.max_age_seconds > 0 {
        for (path, modified) in eligible {
            match now.duration_since(modified) {
                Ok(age) => {
                    if age.as_secs() > cleanup.max_age_seconds {
                        to_remove.push(path);
                    }
                }
                Err(err) => {
                    issues.push(format!(
                        "stage=cleanup.age kind=duration_since path={} error={err}",
                        path.display()
                    ));
                }
            }
        }
    }

    for path in to_remove {
        match tokio::fs::remove_file(&path).await {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => {
                issues.push(format!(
                    "stage=cleanup.remove_file kind=remove_file path={} error={err}",
                    path.display()
                ));
            }
        }
    }

    Ok(CleanupReport::from_issues(issues))
}

#[derive(Clone, Debug)]
struct CleanupReport {
    issue_count: usize,
    first_issue: String,
}

impl CleanupReport {
    fn empty() -> Self {
        Self {
            issue_count: 0,
            first_issue: "<none>".to_string(),
        }
    }

    fn from_issues(issues: Vec<String>) -> Self {
        let issue_count = issues.len();
        let first_issue = match issues.first() {
            Some(first) => first.to_string(),
            None => "<none>".to_string(),
        };
        Self {
            issue_count,
            first_issue,
        }
    }
}

fn emit_postgres_line(
    log: &LogHandle,
    producer: LogProducer,
    transport: LogTransport,
    origin: &str,
    path: &Path,
    line: Vec<u8>,
) -> Result<(), WorkerError> {
    let decoded = decode_line(&line);
    let record = normalize_postgres_line(
        decoded.as_str(),
        PostgresLineRecordBuilder::new(producer, transport, format!("{origin}:{}", path.display())),
    );
    log.emit_raw_record(record).map_err(|err| {
        WorkerError::Message(format!(
            "log sink error while ingesting postgres log: {err}"
        ))
    })?;
    Ok(())
}

fn decode_line(line: &[u8]) -> String {
    match String::from_utf8(line.to_vec()) {
        Ok(s) => s,
        Err(err) => {
            let bytes = err.into_bytes();
            format!("non_utf8_bytes_hex={}", hex_encode(bytes.as_slice()))
        }
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len().saturating_mul(2));
    for b in bytes {
        out.push(TABLE[(b >> 4) as usize] as char);
        out.push(TABLE[(b & 0x0f) as usize] as char);
    }
    out
}

fn normalize_postgres_line(line: &str, builder: PostgresLineRecordBuilder) -> RawRecordBuilder {
    if let Ok(value) = serde_json::from_str::<Value>(line) {
        if let Some(parsed) = normalize_postgres_json(value) {
            return builder.build(
                LogParser::PostgresJson,
                parsed.severity,
                parsed.message,
                parsed.fields,
            );
        }
    }

    if let Some(parsed) = normalize_postgres_plain(line) {
        return builder.build(
            LogParser::PostgresPlain,
            parsed.severity,
            parsed.message,
            parsed.fields,
        );
    }

    let mut fields = StructuredFields::new();
    fields.insert("parse_failed", true);
    fields.insert("raw_line", line.to_string());
    builder.build(LogParser::Raw, SeverityText::Info, line.to_string(), fields)
}

struct ParsedLine {
    severity: SeverityText,
    message: String,
    fields: StructuredFields,
}

fn normalize_postgres_json(value: Value) -> Option<ParsedLine> {
    let obj = value.as_object()?;
    let message = match obj.get("message").and_then(|v| v.as_str()) {
        Some(message) => message.to_string(),
        None => String::new(),
    };
    if message.trim().is_empty() {
        return None;
    }

    let severity_raw = obj
        .get("error_severity")
        .and_then(|v| v.as_str())
        .or_else(|| obj.get("severity").and_then(|v| v.as_str()));
    let severity_raw = severity_raw.map_or("INFO", |severity| severity);
    let severity = map_pg_severity(severity_raw);

    let mut fields = StructuredFields::new();
    fields.insert("postgres.json", value.clone());

    Some(ParsedLine {
        severity,
        message,
        fields,
    })
}

fn normalize_postgres_plain(line: &str) -> Option<ParsedLine> {
    // Example:
    // 2026-01-01 12:34:56.789 UTC [123] LOG:  message
    let bracket = line.find('[')?;
    let after_bracket = line[bracket..].find(']')?;
    let rest = line[bracket + after_bracket + 1..].trim_start();

    let (level, message) = rest.split_once(':')?;
    let level = level.trim();
    let message = message.trim_start().to_string();
    if level.is_empty() || message.is_empty() {
        return None;
    }
    let severity = map_pg_severity(level);
    let mut fields = StructuredFields::new();
    fields.insert("postgres.level_raw", level.to_string());

    Some(ParsedLine {
        severity,
        message,
        fields,
    })
}

fn map_pg_severity(raw: &str) -> SeverityText {
    match raw.trim().to_ascii_uppercase().as_str() {
        "DEBUG" | "DEBUG1" | "DEBUG2" | "DEBUG3" | "DEBUG4" | "DEBUG5" => SeverityText::Debug,
        "INFO" | "NOTICE" | "LOG" => SeverityText::Info,
        "WARNING" => SeverityText::Warn,
        "ERROR" => SeverityText::Error,
        "FATAL" | "PANIC" => SeverityText::Fatal,
        _ => SeverityText::Info,
    }
}

pub(crate) fn build_ctx(cfg: RuntimeConfig, log: LogHandle) -> PostgresIngestWorkerCtx {
    PostgresIngestWorkerCtx { cfg, log }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::{Duration, SystemTime};

    use serde_json::Value;

    use crate::config::{
        DebugConfig, InlineOrPath, LogCleanupConfig, LogLevel, LoggingConfig, PgHbaConfig,
        PostgresLoggingConfig, RuntimeConfig,
    };
    use crate::logging::{
        decode_app_event, LogHandle, LogParser, LogProducer, LogTransport,
        PostgresLineRecordBuilder, SeverityText, TestSink,
    };

    use crate::state::WorkerError;

    use super::{
        cleanup_log_dir, decode_line, emit_ingest_step_failure, emit_postgres_line,
        ingest_error_key_best_effort, map_pg_severity, normalize_postgres_line, IngestErrorKey,
        IngestErrorRateLimiter,
    };

    const REAL_INGEST_RETRY_SLEEP: Duration = Duration::from_millis(20);
    const REAL_PROCESS_WORKER_POLL_INTERVAL: Duration = Duration::from_millis(5);
    const REAL_PSQL_RETRY_SLEEP: Duration = Duration::from_millis(50);

    fn remove_dir_all_if_exists(path: &std::path::Path) -> Result<(), WorkerError> {
        match std::fs::remove_dir_all(path) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(WorkerError::Message(err.to_string())),
        }
    }

    fn sample_runtime_config() -> RuntimeConfig {
        let baseline_logging =
            crate::test_harness::runtime_config::sample_postgres_logging_config();
        crate::test_harness::runtime_config::RuntimeConfigBuilder::new()
            .with_pg_hba(PgHbaConfig {
                source: InlineOrPath::Inline {
                    content: concat!("local all all trust\n", "host all all 127.0.0.1/32 trust\n",)
                        .to_string(),
                },
            })
            .with_logging(LoggingConfig {
                level: LogLevel::Trace,
                postgres: PostgresLoggingConfig {
                    poll_interval_ms: 50,
                    cleanup: LogCleanupConfig {
                        enabled: false,
                        ..baseline_logging.cleanup
                    },
                    ..baseline_logging
                },
                ..crate::test_harness::runtime_config::sample_logging_config()
            })
            .with_debug(DebugConfig { enabled: false })
            .build()
    }

    fn test_log_handle() -> (LogHandle, Arc<TestSink>) {
        let sink = Arc::new(TestSink::default());
        let sink_dyn: Arc<dyn crate::logging::LogSink> = sink.clone();
        (
            LogHandle::new("host-a".to_string(), sink_dyn, SeverityText::Trace),
            sink,
        )
    }

    #[test]
    fn ingest_error_rate_limiter_suppresses_and_reemits_with_count() {
        let mut limiter = IngestErrorRateLimiter::new(30_000);
        let key = IngestErrorKey {
            stage: "a".to_string(),
            kind: "b".to_string(),
            path: "c".to_string(),
        };

        let first = limiter.record(key.clone(), 1_000);
        assert_eq!(
            first,
            super::RateLimitDecision {
                emit: true,
                suppressed: 0
            }
        );

        let suppressed = limiter.record(key.clone(), 2_000);
        assert_eq!(
            suppressed,
            super::RateLimitDecision {
                emit: false,
                suppressed: 0
            }
        );

        let reemit = limiter.record(key, 31_000);
        assert_eq!(
            reemit,
            super::RateLimitDecision {
                emit: true,
                suppressed: 1
            }
        );
    }

    #[test]
    fn ingest_error_key_parsing_uses_first_stage_kind_path_tokens() {
        let err = WorkerError::Message(
            "postgres_ingest iteration_errors count=2 stage=first kind=k1 path=/a error=x extra=[stage=second kind=k2 path=/b error=y]"
                .to_string(),
        );
        let key = ingest_error_key_best_effort(&err);
        assert_eq!(key.stage, "first");
        assert_eq!(key.kind, "k1");
        assert_eq!(key.path, "/a");
    }

    #[test]
    fn emit_ingest_step_failure_emits_internal_error_record() -> Result<(), WorkerError> {
        let (log, sink) = test_log_handle();
        let err = WorkerError::Message("stage=x kind=y path=/z error=boom".to_string());

        let emitted = emit_ingest_step_failure(&log, &err, 2, 7);
        assert_eq!(emitted, Ok(()));

        let records = sink.take();
        assert_eq!(records.len(), 1);
        let decoded = decode_app_event(&records[0]).map_err(|err| {
            WorkerError::Message(format!("decode postgres ingest event failed: {err}"))
        })?;
        assert_eq!(decoded.severity, SeverityText::Error);
        assert_eq!(decoded.origin, "postgres_ingest::run");
        assert_eq!(
            decoded.header,
            crate::logging::AppEventHeader::new(
                "postgres_ingest.step_once_failed",
                "postgres_ingest",
                "failed",
            )
        );
        assert_eq!(
            decoded.fields.get("attempts"),
            Some(&Value::Number(serde_json::Number::from(2_u64)))
        );
        assert_eq!(
            decoded.fields.get("suppressed"),
            Some(&Value::Number(serde_json::Number::from(7_u64)))
        );
        Ok(())
    }

    #[test]
    fn map_pg_severity_maps_known_levels() {
        assert_eq!(map_pg_severity("ERROR"), SeverityText::Error);
        assert_eq!(map_pg_severity("warning"), SeverityText::Warn);
        assert_eq!(map_pg_severity("log"), SeverityText::Info);
    }

    #[test]
    fn normalize_postgres_line_parses_jsonlog() {
        let raw = r#"{"error_severity":"LOG","message":"hello from json"}"#;
        let record = normalize_postgres_line(
            raw,
            PostgresLineRecordBuilder::new(LogProducer::Postgres, LogTransport::FileTail, "test"),
        )
        .into_record(1, "host-a".to_string());
        assert_eq!(record.source.parser, LogParser::PostgresJson);
        assert_eq!(record.message, "hello from json");
        assert_eq!(record.severity_text, SeverityText::Info);
        assert_eq!(record.severity_number, SeverityText::Info.number());
        assert_eq!(record.hostname, "host-a");
    }

    #[test]
    fn normalize_postgres_line_parses_plain() {
        let raw = "2026-03-04 01:02:03 UTC [123] ERROR:  something bad";
        let record = normalize_postgres_line(
            raw,
            PostgresLineRecordBuilder::new(LogProducer::Postgres, LogTransport::FileTail, "test"),
        )
        .into_record(1, "host-a".to_string());
        assert_eq!(record.source.parser, LogParser::PostgresPlain);
        assert_eq!(record.severity_text, SeverityText::Error);
        assert_eq!(record.message, "something bad");
    }

    #[test]
    fn normalize_postgres_line_preserves_raw_on_failure() {
        let raw = "not a postgres log line";
        let record = normalize_postgres_line(
            raw,
            PostgresLineRecordBuilder::new(LogProducer::Postgres, LogTransport::FileTail, "test"),
        )
        .into_record(1, "host-a".to_string());
        assert_eq!(record.source.parser, LogParser::Raw);
        assert_eq!(record.message, raw);
        assert_eq!(
            record.attributes.get("parse_failed"),
            Some(&serde_json::Value::Bool(true))
        );
        assert_eq!(
            record.attributes.get("raw_line"),
            Some(&serde_json::Value::String(raw.to_string()))
        );
    }

    #[test]
    fn decode_line_encodes_non_utf8_bytes_as_hex() {
        let bytes = [0xff_u8, 0x00, b'a', 0x80];
        assert_eq!(decode_line(bytes.as_slice()), "non_utf8_bytes_hex=ff006180");
    }

    #[test]
    fn normalize_postgres_line_preserves_raw_on_non_utf8_failure() {
        let bytes = [0xff_u8, 0x00, b'a', 0x80];
        let raw = decode_line(bytes.as_slice());
        let record = normalize_postgres_line(
            raw.as_str(),
            PostgresLineRecordBuilder::new(LogProducer::Postgres, LogTransport::FileTail, "test"),
        )
        .into_record(1, "host-a".to_string());
        assert_eq!(record.source.parser, LogParser::Raw);
        assert_eq!(record.message, raw);
        assert_eq!(
            record.attributes.get("parse_failed"),
            Some(&Value::Bool(true))
        );
        assert_eq!(
            record.attributes.get("raw_line"),
            Some(&Value::String("non_utf8_bytes_hex=ff006180".to_string()))
        );
    }

    #[test]
    fn emit_postgres_line_emits_parse_failed_record_for_non_utf8() -> Result<(), WorkerError> {
        let (log, sink) = test_log_handle();
        let path = PathBuf::from("/tmp/pg.log");
        let bytes = vec![0xff_u8, 0x00, b'a', 0x80];
        emit_postgres_line(
            &log,
            LogProducer::Postgres,
            LogTransport::FileTail,
            "pg_ctl_log_file",
            path.as_path(),
            bytes,
        )?;
        let records = sink.take();
        assert_eq!(records.len(), 1);
        assert_eq!(
            records[0].attributes.get("parse_failed"),
            Some(&Value::Bool(true))
        );
        assert_eq!(
            records[0].attributes.get("raw_line"),
            Some(&Value::String("non_utf8_bytes_hex=ff006180".to_string()))
        );
        Ok(())
    }

    fn temp_dir(label: &str) -> PathBuf {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "pgtuskmaster-logging-cleanup-{label}-{}-{unique}",
            std::process::id()
        ))
    }

    #[tokio::test(flavor = "current_thread")]
    async fn cleanup_log_dir_enforces_max_files_and_protects_active_file() -> Result<(), WorkerError>
    {
        let dir = temp_dir("max-files");
        remove_dir_all_if_exists(&dir)?;
        std::fs::create_dir_all(&dir).map_err(|err| WorkerError::Message(err.to_string()))?;

        let protected = dir.join("active.log");
        std::fs::write(&protected, b"active\n")
            .map_err(|err| WorkerError::Message(err.to_string()))?;

        for i in 0..5 {
            let path = dir.join(format!("rotated-{i}.log"));
            std::fs::write(&path, b"x\n").map_err(|err| WorkerError::Message(err.to_string()))?;
        }

        let report = cleanup_log_dir(
            dir.as_path(),
            &LogCleanupConfig {
                enabled: true,
                max_files: 2,
                max_age_seconds: 365 * 24 * 60 * 60,
                protect_recent_seconds: 1,
            },
            &[protected.as_path()],
            SystemTime::now() + Duration::from_secs(3600),
        )
        .await?;
        assert_eq!(report.issue_count, 0);

        assert!(protected.exists());
        let mut remaining = 0usize;
        for entry in std::fs::read_dir(&dir).map_err(|err| WorkerError::Message(err.to_string()))? {
            let entry = entry.map_err(|err| WorkerError::Message(err.to_string()))?;
            if entry.path().extension().and_then(|s| s.to_str()) == Some("log") {
                remaining = remaining.saturating_add(1);
            }
        }
        // protected + max_files
        assert!(remaining <= 3);

        remove_dir_all_if_exists(&dir)?;
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn cleanup_log_dir_never_deletes_known_active_signals() -> Result<(), WorkerError> {
        let dir = temp_dir("protected-basenames");
        remove_dir_all_if_exists(&dir)?;
        std::fs::create_dir_all(&dir).map_err(|err| WorkerError::Message(err.to_string()))?;

        let json = dir.join("postgres.json");
        let stderr = dir.join("postgres.stderr.log");
        let stdout = dir.join("postgres.stdout.log");
        std::fs::write(&json, b"{}\n").map_err(|err| WorkerError::Message(err.to_string()))?;
        std::fs::write(&stderr, b"x\n").map_err(|err| WorkerError::Message(err.to_string()))?;
        std::fs::write(&stdout, b"x\n").map_err(|err| WorkerError::Message(err.to_string()))?;

        for i in 0..10 {
            let path = dir.join(format!("rotated-{i}.log"));
            std::fs::write(&path, b"x\n").map_err(|err| WorkerError::Message(err.to_string()))?;
        }

        let report = cleanup_log_dir(
            dir.as_path(),
            &LogCleanupConfig {
                enabled: true,
                max_files: 1,
                max_age_seconds: 365 * 24 * 60 * 60,
                protect_recent_seconds: 1,
            },
            &[],
            SystemTime::now() + Duration::from_secs(3600),
        )
        .await?;
        assert_eq!(report.issue_count, 0);

        assert!(json.exists());
        assert!(stderr.exists());
        assert!(stdout.exists());

        remove_dir_all_if_exists(&dir)?;
        Ok(())
    }

    #[cfg(unix)]
    #[tokio::test(flavor = "current_thread")]
    async fn cleanup_log_dir_surfaces_remove_failures() -> Result<(), WorkerError> {
        use std::os::unix::fs::PermissionsExt;

        let dir = temp_dir("remove-failure");
        remove_dir_all_if_exists(&dir)?;
        std::fs::create_dir_all(&dir).map_err(|err| WorkerError::Message(err.to_string()))?;

        let old = dir.join("old.log");
        std::fs::write(&old, b"x\n").map_err(|err| WorkerError::Message(err.to_string()))?;

        let mut perms = std::fs::metadata(&dir)
            .map_err(|err| WorkerError::Message(err.to_string()))?
            .permissions();
        perms.set_mode(0o555);
        std::fs::set_permissions(&dir, perms)
            .map_err(|err| WorkerError::Message(err.to_string()))?;

        let report = cleanup_log_dir(
            dir.as_path(),
            &LogCleanupConfig {
                enabled: true,
                max_files: 1,
                max_age_seconds: 1,
                protect_recent_seconds: 1,
            },
            &[],
            SystemTime::now() + Duration::from_secs(3600),
        )
        .await?;
        assert!(report.issue_count > 0);
        assert!(old.exists());

        let mut perms = std::fs::metadata(&dir)
            .map_err(|err| WorkerError::Message(err.to_string()))?
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&dir, perms)
            .map_err(|err| WorkerError::Message(err.to_string()))?;

        remove_dir_all_if_exists(&dir)?;
        Ok(())
    }

    mod real_binary {
        use std::path::PathBuf;
        use std::time::Duration;

        use tokio::process::Command;
        use tokio::sync::mpsc;
        use tokio::time::Instant;

        use crate::config::{InlineOrPath, RoleAuthConfig, SecretSource};
        use crate::logging::LogRecord;
        use crate::process::jobs::{
            BaseBackupSpec, BootstrapSpec, DemoteSpec, ShutdownMode, StartPostgresSpec,
        };
        use crate::process::state::{
            ProcessJobKind, ProcessJobRequest, ProcessState, ProcessWorkerCtx,
        };
        use crate::process::worker::{step_once as process_step_once, TokioCommandRunner};
        use crate::state::{new_state_channel, JobId, UnixMillis, WorkerError, WorkerStatus};
        use crate::test_harness::binaries::{
            require_pg16_bin_for_real_tests, require_pg16_process_binaries_for_real_tests,
        };
        use crate::test_harness::namespace::NamespaceGuard;
        use crate::test_harness::pg16::{
            prepare_pgdata_dir, spawn_pg16_for_vanilla_postgres, PgInstanceSpec,
        };
        use crate::test_harness::ports::allocate_ports;

        use super::super::{
            step_once as ingest_step_once, PostgresIngestWorkerCtx, PostgresIngestWorkerState,
        };
        use super::{
            sample_runtime_config, test_log_handle, REAL_INGEST_RETRY_SLEEP,
            REAL_PROCESS_WORKER_POLL_INTERVAL, REAL_PSQL_RETRY_SLEEP,
        };

        async fn wait_for_process_idle_success(
            ctx: &mut ProcessWorkerCtx,
            job_id: &JobId,
            timeout: Duration,
        ) -> Result<(), WorkerError> {
            wait_for_process_idle_success_with_debug(ctx, job_id, timeout, None).await
        }

        async fn wait_for_process_idle_success_with_debug(
            ctx: &mut ProcessWorkerCtx,
            job_id: &JobId,
            timeout: Duration,
            debug_log_path: Option<&PathBuf>,
        ) -> Result<(), WorkerError> {
            let started = Instant::now();
            while started.elapsed() < timeout {
                process_step_once(ctx).await?;
                if let ProcessState::Idle {
                    last_outcome: Some(outcome),
                    ..
                } = &ctx.state
                {
                    match outcome {
                        crate::process::state::JobOutcome::Success { id, .. } if id == job_id => {
                            return Ok(());
                        }
                        crate::process::state::JobOutcome::Failure { id, error, .. }
                            if id == job_id =>
                        {
                            let debug_tail = match debug_log_path {
                                Some(path) => tail_file_best_effort(path, 60),
                                None => String::new(),
                            };
                            return Err(WorkerError::Message(format!(
                                "process job {} failed unexpectedly: {error}{}",
                                job_id.0,
                                if debug_tail.is_empty() {
                                    "".to_string()
                                } else {
                                    format!(
                                        "\n--- debug tail {} ---\n{debug_tail}",
                                        path_display(debug_log_path)
                                    )
                                }
                            )));
                        }
                        _ => {}
                    }
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            Err(WorkerError::Message(format!(
                "timed out waiting for job {} success",
                job_id.0
            )))
        }

        fn path_display(path: Option<&PathBuf>) -> String {
            match path {
                Some(path) => path.display().to_string(),
                None => "<none>".to_string(),
            }
        }

        fn tail_file_best_effort(path: &PathBuf, max_lines: usize) -> String {
            let contents = match std::fs::read_to_string(path) {
                Ok(contents) => contents,
                Err(err) => return format!("(failed to read {}: {err})", path.display()),
            };
            let mut lines = contents.lines().collect::<Vec<_>>();
            if lines.len() > max_lines {
                let start = lines.len().saturating_sub(max_lines);
                lines.drain(0..start);
            }
            lines.join("\n")
        }

        fn is_transient_psql_failure(stderr: &str) -> bool {
            let normalized = stderr.to_ascii_lowercase();
            normalized.contains("the database system is starting up")
                || normalized.contains("the database system is shutting down")
                || normalized.contains("not yet accepting connections")
                || normalized.contains("could not connect to server")
                || normalized.contains("connection refused")
        }

        async fn run_psql_query_with_retry(
            psql_bin: &PathBuf,
            port: u16,
            query: &str,
            timeout: Duration,
        ) -> Result<(), WorkerError> {
            let deadline = Instant::now() + timeout;
            let mut last_stderr = String::new();
            let mut last_stdout = String::new();

            while Instant::now() < deadline {
                let mut cmd = Command::new(psql_bin);
                cmd.arg("-h")
                    .arg("127.0.0.1")
                    .arg("-p")
                    .arg(port.to_string())
                    .arg("-U")
                    .arg("postgres")
                    .arg("-d")
                    .arg("postgres")
                    .arg("-c")
                    .arg(query);

                let output = cmd
                    .output()
                    .await
                    .map_err(|err| WorkerError::Message(format!("psql spawn failed: {err}")))?;

                if output.status.success() {
                    return Ok(());
                }

                last_stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                last_stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

                if !is_transient_psql_failure(&last_stderr) {
                    return Err(WorkerError::Message(format!(
                        "psql exited unsuccessfully: {} (non-transient)\n--- stdout ---\n{}\n--- stderr ---\n{}",
                        output.status,
                        last_stdout,
                        last_stderr
                    )));
                }

                tokio::time::sleep(REAL_PSQL_RETRY_SLEEP).await;
            }

            Err(WorkerError::Message(format!(
                "timed out waiting for psql readiness after {:?}\n--- last stdout ---\n{}\n--- last stderr ---\n{}",
                timeout, last_stdout, last_stderr
            )))
        }

        #[tokio::test(flavor = "current_thread")]
        async fn ingests_jsonlog_and_stderr_files_from_real_postgres() -> Result<(), WorkerError> {
            let postgres_bin = require_pg16_bin_for_real_tests("postgres")?;
            let initdb_bin = require_pg16_bin_for_real_tests("initdb")?;
            let psql_bin = require_pg16_bin_for_real_tests("psql")?;

            let guard = NamespaceGuard::new("log-jsonlog-stderr")?;
            let ns = guard.namespace()?;

            let data_dir = prepare_pgdata_dir(ns, "node-a")?;
            let mut reservation = allocate_ports(1)?;
            let port = reservation.as_slice()[0];
            let socket_dir = ns.child_dir("pg16/node-a/socket");
            let log_dir = ns.child_dir("logs/pg16-node-a");

            let jsonlog_path = log_dir.join("postgres.json");
            std::fs::create_dir_all(&log_dir).map_err(|err| {
                WorkerError::Message(format!(
                    "create postgres ingest log dir {} failed: {err}",
                    log_dir.display()
                ))
            })?;
            std::fs::write(&jsonlog_path, b"").map_err(|err| {
                WorkerError::Message(format!(
                    "seed postgres ingest jsonlog file {} failed: {err}",
                    jsonlog_path.display()
                ))
            })?;

            let conf_lines = vec![
                "logging_collector = on".to_string(),
                "log_destination = 'jsonlog,stderr'".to_string(),
                format!("log_directory = '{}'", log_dir.display()),
                "log_filename = 'postgres.json'".to_string(),
                "log_statement = 'all'".to_string(),
            ];

            let spec = PgInstanceSpec {
                postgres_bin,
                initdb_bin,
                data_dir,
                socket_dir,
                log_dir: log_dir.clone(),
                port,
                startup_timeout: Duration::from_secs(10),
            };
            reservation.release_port(port).map_err(|err| {
                WorkerError::Message(format!("release reserved port failed: {err}"))
            })?;
            // This test validates raw PostgreSQL log emission and ingest parsing, not
            // pgtuskmaster-managed startup ownership, so it uses the explicit
            // vanilla-Postgres config exception path.
            let mut pg = spawn_pg16_for_vanilla_postgres(spec, &conf_lines).await?;

            let mut cfg = sample_runtime_config();
            cfg.logging.postgres.log_dir = Some(log_dir);
            cfg.logging.postgres.cleanup.enabled = false;
            cfg.postgres.log_file = ns.child_dir("runtime/pg_ctl.log");

            let (log_handle, sink) = test_log_handle();
            let ctx = PostgresIngestWorkerCtx {
                cfg,
                log: log_handle,
            };
            let mut state = PostgresIngestWorkerState::new(&ctx.cfg);

            // Prime ingestion offsets and then generate logs.
            ingest_step_once(&ctx, &mut state).await?;

            run_psql_query_with_retry(&psql_bin, port, "SELECT 1;", Duration::from_secs(10))
                .await?;

            let deadline = Instant::now() + Duration::from_secs(3);
            let mut collected = Vec::new();
            while Instant::now() < deadline {
                ingest_step_once(&ctx, &mut state).await?;
                collected.extend(sink.take());
                let saw_json = collected
                    .iter()
                    .any(|r| r.source.parser == crate::logging::LogParser::PostgresJson);
                let saw_stderr = collected
                    .iter()
                    .any(|r| r.source.origin.contains("postgres.stderr.log"));
                if saw_json && saw_stderr {
                    pg.shutdown().await?;
                    return Ok(());
                }
                tokio::time::sleep(REAL_INGEST_RETRY_SLEEP).await;
            }

            pg.shutdown().await?;
            drop(reservation);
            Err(WorkerError::Message(
                "timed out waiting for jsonlog+stderr ingestion".to_string(),
            ))
        }

        #[tokio::test(flavor = "current_thread")]
        async fn ingests_pg_ctl_log_file_and_captures_pg_tool_output() -> Result<(), WorkerError> {
            let binaries = require_pg16_process_binaries_for_real_tests()?;

            let guard = NamespaceGuard::new("log-pgctl")?;
            let ns = guard.namespace()?;

            let mut reservation = allocate_ports(1)?;
            let port = reservation.as_slice()[0];

            let data_dir = prepare_pgdata_dir(ns, "node-a")?;
            let socket_dir = ns.child_dir("sock");
            let log_file = ns.child_dir("runtime/pg_ctl.log");
            let log_dir = ns.child_dir("logs/pg16-node-a");
            std::fs::create_dir_all(&socket_dir)
                .map_err(|err| WorkerError::Message(format!("create socket_dir failed: {err}")))?;
            if let Some(parent) = log_file.parent() {
                std::fs::create_dir_all(parent).map_err(|err| {
                    WorkerError::Message(format!("create log file parent failed: {err}"))
                })?;
            }
            let _ = std::fs::create_dir_all(&log_dir);
            let jsonlog_path = log_dir.join("postgres.json");
            let _ = std::fs::write(&jsonlog_path, b"");

            let mut cfg = sample_runtime_config();
            cfg.process.binaries = binaries.clone();
            cfg.postgres.data_dir = data_dir.clone();
            cfg.postgres.socket_dir = socket_dir.clone();
            cfg.postgres.listen_port = port;
            cfg.postgres.log_file = log_file.clone();
            cfg.postgres
                .extra_gucs
                .insert("log_destination".to_string(), "jsonlog,stderr".to_string());
            cfg.postgres
                .extra_gucs
                .insert("log_filename".to_string(), "postgres.json".to_string());
            cfg.postgres
                .extra_gucs
                .insert("log_directory".to_string(), log_dir.display().to_string());
            cfg.postgres
                .extra_gucs
                .insert("log_statement".to_string(), "all".to_string());
            cfg.postgres
                .extra_gucs
                .insert("logging_collector".to_string(), "on".to_string());
            cfg.logging.postgres.log_dir = Some(log_dir.clone());
            cfg.logging.postgres.cleanup.enabled = false;

            let (log_handle, sink) = test_log_handle();

            let (publisher, _subscriber) = new_state_channel(
                ProcessState::Idle {
                    worker: WorkerStatus::Starting,
                    last_outcome: None,
                },
                UnixMillis(0),
            );
            let (tx, rx) = mpsc::unbounded_channel();
            let mut process_ctx = ProcessWorkerCtx {
                poll_interval: REAL_PROCESS_WORKER_POLL_INTERVAL,
                config: cfg.process.clone(),
                log: log_handle.clone(),
                capture_subprocess_output: true,
                state: ProcessState::Idle {
                    worker: WorkerStatus::Starting,
                    last_outcome: None,
                },
                publisher,
                inbox: rx,
                inbox_disconnected_logged: false,
                command_runner: Box::new(TokioCommandRunner),
                active_runtime: None,
                last_rejection: None,
                now: Box::new(crate::process::worker::system_now_unix_millis),
            };

            let ingest_ctx = PostgresIngestWorkerCtx {
                cfg,
                log: log_handle,
            };
            let mut ingest_state = PostgresIngestWorkerState::new(&ingest_ctx.cfg);

            let bootstrap_id = JobId("bootstrap".to_string());
            tx.send(ProcessJobRequest {
                id: bootstrap_id.clone(),
                kind: ProcessJobKind::Bootstrap(BootstrapSpec {
                    data_dir: data_dir.clone(),
                    superuser_username: ingest_ctx.cfg.postgres.roles.superuser.username.clone(),
                    timeout_ms: Some(30_000),
                }),
            })
            .map_err(|_| WorkerError::Message("send bootstrap job failed".to_string()))?;

            wait_for_process_idle_success(&mut process_ctx, &bootstrap_id, Duration::from_secs(30))
                .await?;
            let managed = crate::postgres_managed::materialize_managed_postgres_config(
                &ingest_ctx.cfg,
                &crate::postgres_managed_conf::ManagedPostgresStartIntent::primary(),
            )
            .map_err(|err| {
                WorkerError::Message(format!("materialize managed postgres config failed: {err}"))
            })?;

            reservation.release_port(port).map_err(|err| {
                WorkerError::Message(format!("release reserved port failed: {err}"))
            })?;
            let start_id = JobId("start".to_string());
            tx.send(ProcessJobRequest {
                id: start_id.clone(),
                kind: ProcessJobKind::StartPostgres(StartPostgresSpec {
                    data_dir: data_dir.clone(),
                    config_file: managed.postgresql_conf_path,
                    log_file: log_file.clone(),
                    wait_seconds: Some(30),
                    timeout_ms: Some(60_000),
                }),
            })
            .map_err(|_| WorkerError::Message("send start job failed".to_string()))?;

            let started = Instant::now();
            let mut collected_for_debug: Vec<LogRecord> = Vec::new();
            while started.elapsed() < Duration::from_secs(60) {
                process_step_once(&mut process_ctx).await?;
                collected_for_debug.extend(sink.take());

                if let ProcessState::Idle {
                    last_outcome: Some(outcome),
                    ..
                } = &process_ctx.state
                {
                    match outcome {
                        crate::process::state::JobOutcome::Success { id, .. }
                            if id == &start_id =>
                        {
                            break;
                        }
                        crate::process::state::JobOutcome::Failure { id, error, .. }
                            if id == &start_id =>
                        {
                            let pg_ctl_tail = tail_file_best_effort(&log_file, 120);
                            let postgres_json_tail = tail_file_best_effort(&jsonlog_path, 120);
                            let postmaster_pid =
                                tail_file_best_effort(&data_dir.join("postmaster.pid"), 60);

                            let mut pg_tool_lines = Vec::new();
                            for record in &collected_for_debug {
                                if record.source.producer != crate::logging::LogProducer::PgTool {
                                    continue;
                                }
                                let job_kind = record
                                    .attributes
                                    .get("job_kind")
                                    .and_then(|v| v.as_str())
                                    .map_or("<none>", |value| value);
                                let job_id_attr = record
                                    .attributes
                                    .get("job_id")
                                    .and_then(|v| v.as_str())
                                    .map_or("<none>", |value| value);
                                if job_kind != "start_postgres"
                                    && job_id_attr != start_id.0.as_str()
                                {
                                    continue;
                                }
                                pg_tool_lines.push(format!(
                                    "{:?} {}: {}",
                                    record.source.transport, record.source.origin, record.message
                                ));
                            }
                            if pg_tool_lines.len() > 60 {
                                let start = pg_tool_lines.len().saturating_sub(60);
                                pg_tool_lines.drain(0..start);
                            }
                            let pg_tool_debug = if pg_tool_lines.is_empty() {
                                "(no captured pg_tool stdout/stderr lines for start_postgres)"
                                    .to_string()
                            } else {
                                pg_tool_lines.join("\n")
                            };

                            return Err(WorkerError::Message(format!(
                                "process job {} failed unexpectedly: {error}\n--- pg_ctl log tail {} ---\n{}\n--- postgres jsonlog tail {} ---\n{}\n--- postmaster.pid tail {} ---\n{}\n--- captured pg_tool output (start_postgres) ---\n{}",
                                start_id.0,
                                log_file.display(),
                                pg_ctl_tail,
                                jsonlog_path.display(),
                                postgres_json_tail,
                                data_dir.join("postmaster.pid").display(),
                                postmaster_pid,
                                pg_tool_debug
                            )));
                        }
                        _ => {}
                    }
                }

                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            if started.elapsed() >= Duration::from_secs(60) {
                return Err(WorkerError::Message(
                    "timed out waiting for start_postgres job success".to_string(),
                ));
            }

            // Pump ingestion a bit to collect pg_ctl log lines.
            let mut cmd = Command::new(binaries.psql.clone());
            cmd.arg("-h")
                .arg("127.0.0.1")
                .arg("-p")
                .arg(port.to_string())
                .arg("-U")
                .arg("postgres")
                .arg("-d")
                .arg("postgres")
                .arg("-c")
                .arg("SELECT 1;");
            let status = cmd
                .status()
                .await
                .map_err(|err| WorkerError::Message(format!("psql spawn failed: {err}")))?;
            if !status.success() {
                return Err(WorkerError::Message(format!(
                    "psql pg_switch_wal exited unsuccessfully: {status}"
                )));
            }

            let deadline = Instant::now() + Duration::from_secs(10);
            let mut collected = Vec::new();
            while Instant::now() < deadline {
                ingest_step_once(&ingest_ctx, &mut ingest_state).await?;
                process_step_once(&mut process_ctx).await?;
                collected.extend(sink.take());
                let saw_pg_ctl_log = collected.iter().any(|r| {
                    r.source.producer == crate::logging::LogProducer::Postgres
                        && r.source.origin.contains("pg_ctl_log_file")
                });
                let saw_pg_tool = collected.iter().any(|r| {
                    r.source.producer == crate::logging::LogProducer::PgTool
                        && (r.source.transport == crate::logging::LogTransport::ChildStdout
                            || r.source.transport == crate::logging::LogTransport::ChildStderr)
                });
                let saw_jsonlog = collected.iter().any(|r| {
                    r.source.producer == crate::logging::LogProducer::Postgres
                        && r.source.parser == crate::logging::LogParser::PostgresJson
                });
                if saw_pg_ctl_log && saw_pg_tool && saw_jsonlog {
                    break;
                }
                tokio::time::sleep(REAL_INGEST_RETRY_SLEEP).await;
            }

            let stop_id = JobId("stop".to_string());
            tx.send(ProcessJobRequest {
                id: stop_id.clone(),
                kind: ProcessJobKind::Demote(DemoteSpec {
                    data_dir,
                    mode: ShutdownMode::Fast,
                    timeout_ms: Some(20_000),
                }),
            })
            .map_err(|_| WorkerError::Message("send stop job failed".to_string()))?;
            wait_for_process_idle_success(&mut process_ctx, &stop_id, Duration::from_secs(30))
                .await?;

            // One more ingestion pass after shutdown to catch any final flushes.
            ingest_step_once(&ingest_ctx, &mut ingest_state).await?;

            let mut all_records = collected;
            all_records.extend(sink.take());

            let saw_pg_ctl_log = all_records.iter().any(|r| {
                r.source.producer == crate::logging::LogProducer::Postgres
                    && r.source.origin.contains("pg_ctl_log_file")
            });
            let saw_pg_tool = all_records.iter().any(|r| {
                r.source.producer == crate::logging::LogProducer::PgTool
                    && r.attributes
                        .get("job_kind")
                        .and_then(|v| v.as_str())
                        .is_some()
            });
            let saw_jsonlog = all_records.iter().any(|r| {
                r.source.producer == crate::logging::LogProducer::Postgres
                    && r.source.parser == crate::logging::LogParser::PostgresJson
            });
            if !saw_pg_ctl_log {
                return Err(WorkerError::Message(
                    "missing ingested pg_ctl log file records".to_string(),
                ));
            }
            if !saw_pg_tool {
                return Err(WorkerError::Message(
                    "missing captured pg tool stdout/stderr records".to_string(),
                ));
            }
            if !saw_jsonlog {
                return Err(WorkerError::Message(
                    "missing ingested postgres jsonlog records".to_string(),
                ));
            }

            drop(reservation);
            Ok(())
        }

        #[tokio::test(flavor = "current_thread")]
        async fn captures_helper_binary_stdout_stderr_on_failure() -> Result<(), WorkerError> {
            let binaries = require_pg16_process_binaries_for_real_tests()?;

            let guard = NamespaceGuard::new("log-pgtool")?;
            let ns = guard.namespace()?;

            let data_dir = ns.child_dir("pg_basebackup/out");
            let _ = std::fs::create_dir_all(&data_dir);

            let mut cfg = sample_runtime_config();
            cfg.process.binaries = binaries;

            let (log_handle, sink) = test_log_handle();

            let initial = ProcessState::Idle {
                worker: WorkerStatus::Starting,
                last_outcome: None,
            };
            let (publisher, _subscriber) = new_state_channel(initial.clone(), UnixMillis(0));
            let (tx, rx) = mpsc::unbounded_channel();
            let mut ctx = ProcessWorkerCtx {
                poll_interval: REAL_PROCESS_WORKER_POLL_INTERVAL,
                config: cfg.process,
                log: log_handle,
                capture_subprocess_output: true,
                state: initial,
                publisher,
                inbox: rx,
                inbox_disconnected_logged: false,
                command_runner: Box::new(TokioCommandRunner),
                active_runtime: None,
                last_rejection: None,
                now: Box::new(crate::process::worker::system_now_unix_millis),
            };

            let job_id = JobId("basebackup-fail".to_string());
            tx.send(ProcessJobRequest {
                id: job_id.clone(),
                kind: ProcessJobKind::BaseBackup(BaseBackupSpec {
                    data_dir,
                    source: crate::process::jobs::ReplicatorSourceConn {
                        conninfo: crate::pginfo::state::PgConnInfo {
                            host: "127.0.0.1".to_string(),
                            port: 9,
                            user: "replicator".to_string(),
                            dbname: "postgres".to_string(),
                            application_name: None,
                            connect_timeout_s: Some(1),
                            ssl_mode: crate::pginfo::state::PgSslMode::Prefer,
                            options: None,
                        },
                        auth: RoleAuthConfig::Password {
                            password: SecretSource(InlineOrPath::Inline {
                                content: "secret-password".to_string(),
                            }),
                        },
                    },
                    timeout_ms: Some(5_000),
                }),
            })
            .map_err(|_| WorkerError::Message("send basebackup job failed".to_string()))?;

            let deadline = Instant::now() + Duration::from_secs(10);
            let mut collected = Vec::new();
            while Instant::now() < deadline {
                process_step_once(&mut ctx).await?;
                collected.extend(sink.take());
                let saw_stderr = collected.iter().any(|r| {
                    r.source.producer == crate::logging::LogProducer::PgTool
                        && r.source.transport == crate::logging::LogTransport::ChildStderr
                        && r.attributes.get("job_kind").and_then(|v| v.as_str())
                            == Some("basebackup")
                });
                if saw_stderr {
                    return Ok(());
                }
                tokio::time::sleep(REAL_INGEST_RETRY_SLEEP).await;
            }

            Err(WorkerError::Message(
                "timed out waiting for captured pg_basebackup stderr".to_string(),
            ))
        }
    }
}

--- END FILE: src/logging/postgres_ingest.rs ---

