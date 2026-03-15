## Task: Rewrite Logging Around One Owned LogDto, Logger-Owned Global Context, And An Exhaustive Event Set <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Replace the current logging trait-and-visitor design with a much simpler boundary: each domain owns typed log event enums, each event converts itself in one step into one owned logging DTO, and the logger itself injects global node context such as hostname, cluster name, scope, and member id. The higher-order goal is to make logging compiler-driven and minimal: no emitter should know about logger-global context, no emitter should know how fields are encoded, no generic field visitor should exist, no logging trait should be shaped by process-specific concepts such as `job_id`, and no log-event types should exist outside the exhaustive set defined in this task.

This task is intentionally a replacement task, not an incremental tweak. The repo already completed a logging refactor that introduced `DomainLogEvent`, `LogEventMetadata`, `LogFieldVisitor`, and several domain-owned event enums. That design is still too complicated and still leaks too much logging structure into emitters:
- the trait has two methods instead of one owned conversion step
- event implementations are split across `metadata()` and `write_fields()`
- many emitters still carry node-global data such as `scope` and `member_id`
- several log-specific wrapper types exist only to satisfy logging, not domain logic
- some logging field names are weak or misleading, especially `error` versus top-level `message`
- bootstrap failures are still stringly and not modeled as typed bootstrap errors

The required end state from this task is:
- one real logging trait with one real conversion method
- logger-owned global context added automatically to every record
- exhaustive, repo-wide log event enum set defined directly in this task
- no log event enum outside the set defined in this task
- no `origin` enums for function names
- no log-specific identity wrapper structs such as `DcsLogIdentity` or `ProcessExecutionIdentity`
- no generic field visitor trait
- no event-local repetition of `hostname`, `cluster_name`, `scope`, or `member_id`
- no use of field name `error`; use `cause` for event-local failure detail instead
- bootstrap/configuration failures remain typed startup/bootstrap errors, not runtime log events

Anything in the repo that is still a log event after this task, and is not listed explicitly below, must be removed or folded into one of the listed event enums. This task is the authoritative and exhaustive log-event inventory.

**Exact required target API:**

The old trait surface in `src/logging/event.rs` must be deleted and replaced with this shape:

```rust
mod sealed {
    pub trait Sealed {}
}

pub(crate) trait LoggableEvent: sealed::Sealed + Send + 'static {
    fn into_log_event(self) -> LogEventDto;
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub(crate) struct LogEventDto {
    pub(crate) severity: LogSeverity,
    pub(crate) event_name: String,
    pub(crate) result: LogEventResult,
    pub(crate) message: std::borrow::Cow<'static, str>,
    pub(crate) source: LogSource,
    pub(crate) fields: serde_json::Value,
}
```

This is a deliberate design decision:
- there is one trait method only: `into_log_event(self)`
- that method returns one fully owned DTO
- there is no separate `metadata()` method
- there is no `write_fields()` method
- there is no `LogFieldVisitor`
- there is no split between metadata and field materialization

Built-in Rust does not have generic arbitrary attributes that affect a normal trait automatically. A tag such as `#[event_name = "..."]` only does something if a derive macro or attribute macro is written to understand it. This task does not require a custom proc-macro. Instead, it requires using `serde`'s existing derive/tag machinery to auto-materialize variant fields into log fields.

The required serialization pattern for every log event enum is:

```rust
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "event_name", content = "fields")]
pub(crate) enum ProcessLogEvent {
    #[serde(rename = "process.request_received")]
    RequestReceived {
        job_kind: crate::process::jobs::ProcessJobKind,
    },
    #[serde(rename = "process.inbox_disconnected")]
    InboxDisconnected,
    #[serde(rename = "process.output_emit_failed")]
    OutputEmitFailed {
        job_kind: crate::process::jobs::ProcessJobKind,
        stream: CapturedStream,
        bytes_len: usize,
        cause: String,
    },
}
```

That serialization must produce these shapes automatically:

```json
{"event_name":"process.request_received","fields":{"job_kind":"start_primary"}}
```

```json
{"event_name":"process.inbox_disconnected"}
```

```json
{"event_name":"process.output_emit_failed","fields":{"job_kind":"start_primary","stream":"stderr","bytes_len":128,"cause":"channel closed"}}
```

The logger must then split that serialized value into `event_name` and `fields` with one internal helper:

```rust
fn split_serialized_event<T>(event: &T) -> Result<(String, serde_json::Value), LogInternalError>
where
    T: serde::Serialize,
{
    let value = serde_json::to_value(event)
        .map_err(|err| LogInternalError::SerializeEvent(err.to_string()))?;

    let object = value.as_object().ok_or_else(|| {
        LogInternalError::SerializeEvent(
            "serialized log event must be a JSON object".to_string(),
        )
    })?;

    let event_name = object
        .get("event_name")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| {
            LogInternalError::SerializeEvent(
                "serialized log event must contain string event_name".to_string(),
            )
        })?
        .to_string();

    let fields = object
        .get("fields")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));

    if !fields.is_object() {
        return Err(LogInternalError::SerializeEvent(
            "serialized log event fields must be a JSON object".to_string(),
        ));
    }

    Ok((event_name, fields))
}
```

Every `LoggableEvent::into_log_event(self)` implementation must call this helper instead of manually assembling field maps. That is the concrete mechanism by which enum fields become logging fields automatically.

`LogEventDto.fields` must be produced from typed `serde::Serialize` structs or enums owned by the emitting domain or by logging-internal postgres ingest code. Emitters must not hand-build arbitrary field maps. Using `serde_json::to_value(typed_fields)` is acceptable and preferred. `fields` must always serialize to a JSON object; if serialization would produce any non-object value, that is a bug and must be handled as an internal logging bug, not ignored.

The logger-owned global context must be defined as:

```rust
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub(crate) struct LogContext {
    pub(crate) hostname: String,
    pub(crate) cluster_name: String,
    pub(crate) scope: String,
    pub(crate) member_id: String,
}
```

This context is owned by logging and injected automatically by `LogSender` during record materialization. No application log event enum may contain these fields anymore.

The sink-facing record type must be reduced to this shape:

```rust
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub(crate) struct LogRecord {
    pub(crate) timestamp_ns: i64,
    pub(crate) context: LogContext,
    pub(crate) severity_text: LogSeverity,
    pub(crate) severity_number: u8,
    pub(crate) message: String,
    pub(crate) event_name: String,
    pub(crate) event_result: LogEventResult,
    pub(crate) source: LogSource,
    pub(crate) fields: serde_json::Value,
}
```

The required logger-owned supporting types are:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum LogSeverity {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LogEventResult {
    Ok,
    Failed,
    Recovered,
    Timeout,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LogProducer {
    App,
    Postgres,
    PgTool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LogTransport {
    Internal,
    FileTail,
    ChildStdout,
    ChildStderr,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LogParser {
    App,
    PostgresJson,
    PostgresPlain,
    Raw,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub(crate) struct LogSource {
    pub(crate) producer: LogProducer,
    pub(crate) transport: LogTransport,
    pub(crate) parser: LogParser,
}
```

`LogSource` must not contain an `origin` string anymore. Function-name origin enums such as `ProcessLogOrigin`, `DcsLogOrigin`, `PgInfoLogOrigin`, `RuntimeLogOrigin`, and `PostgresIngestOrigin` are logging noise and must be deleted.

`LogSender` must remain the only emission surface visible outside logging, and it must look like this:

```rust
#[derive(Clone)]
pub(crate) struct LogSender {
    // private fields only
}

impl LogSender {
    pub(crate) fn send<E>(&self, event: E) -> Result<(), LogSendError>
    where
        E: LoggableEvent,
    {
        // private implementation
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum LogSendError {
    #[error("log queue is closed")]
    QueueClosed,
}
```

The queue item type stays private to `src/logging`. It should be one private materialized shape derived from `LogEventDto` plus logger context and timestamp.

**Exact required bootstrap error model:**

Bootstrap/configuration failures are not log events because logging is not usable until bootstrap succeeds. The current stringly bootstrap errors must be replaced with typed variants. The minimum required enum is:

```rust
#[derive(Debug, thiserror::Error)]
pub(crate) enum LogBootstrapError {
    #[error("file sink enabled but no file path was configured")]
    FileSinkPathMissing,
    #[error("file sink init failed for `{path}`: {cause}")]
    FileSinkInit { path: std::path::PathBuf, cause: String },
}
```

If more typed bootstrap variants are needed during implementation, add them here in the same spirit. Do not regress back to `Misconfigured(String)` or `SinkInit(String)`. Do not invent bootstrap log events such as `LogBootstrapPathNotSetError`; that is the wrong lifecycle layer.

**Exhaustive event inventory:**

After this task, the only log event enums in the repo must be the seven listed below. Any other log-event enum, wrapper struct, or trait impl must be removed.

1. `RuntimeLogEvent`
2. `PgInfoLogEvent`
3. `DcsLogEvent`
4. `ProcessLogEvent`
5. `SubprocessLogEvent`
6. `PostgresIngestLogEvent`
7. `PostgresLineLogEvent`

All seven enums must implement `LoggableEvent`, and anything not in this list that currently implements logging must be deleted or merged into one of them.

The exact required event enums are:

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum RuntimeLogEvent {
    StartupEntered {
        startup_run_id: String,
        logging_level: crate::config::LogLevel,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PgInfoLogEvent {
    PollFailed {
        cause: String,
    },
    SqlTransition {
        previous: crate::pginfo::state::SqlStatus,
        next: crate::pginfo::state::SqlStatus,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DcsLogEvent {
    ConnectedStepStoreIoFailed {
        cause: String,
    },
    ConnectedStepDecodeFailed {
        cause: String,
    },
    ConnectedStepAlreadyExists {
        cause: String,
    },
    InitialConnectStoreIoFailed {
        cause: String,
    },
    InitialConnectDecodeFailed {
        cause: String,
    },
    InitialConnectAlreadyExists {
        cause: String,
    },
    CoordinationModeTransition {
        previous: Option<crate::dcs::DcsMode>,
        next: crate::dcs::DcsMode,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CapturedStream {
    Stdout,
    Stderr,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ProcessLogEvent {
    WorkerRunStarted {
        capture_subprocess_output: bool,
    },
    RequestReceived {
        job_kind: crate::process::jobs::ProcessJobKind,
    },
    InboxDisconnected,
    BusyRejected {
        job_kind: crate::process::jobs::ProcessJobKind,
    },
    StartPostgresAlreadyRunning {
        data_dir: String,
    },
    StartPostgresPreflightFailed {
        cause: String,
    },
    IntentMaterializationFailed {
        job_kind: crate::process::jobs::ProcessJobKind,
        cause: String,
    },
    BuildCommandFailed {
        job_kind: crate::process::jobs::ProcessJobKind,
        cause: String,
    },
    SpawnFailed {
        job_kind: crate::process::jobs::ProcessJobKind,
        cause: String,
    },
    Started {
        job_kind: crate::process::jobs::ProcessJobKind,
    },
    OutputDrainFailed {
        job_kind: crate::process::jobs::ProcessJobKind,
        cause: String,
    },
    Timeout {
        job_kind: crate::process::jobs::ProcessJobKind,
    },
    ExitedSuccessfully {
        job_kind: crate::process::jobs::ProcessJobKind,
    },
    ExitedUnsuccessfully {
        job_kind: crate::process::jobs::ProcessJobKind,
        cause: String,
    },
    PollFailed {
        job_kind: crate::process::jobs::ProcessJobKind,
        cause: String,
    },
    OutputEmitFailed {
        job_kind: crate::process::jobs::ProcessJobKind,
        stream: CapturedStream,
        bytes_len: usize,
        cause: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SubprocessLogEvent {
    Line {
        job_kind: crate::process::jobs::ProcessJobKind,
        stream: CapturedStream,
        line: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PostgresIngestLogEvent {
    StepOnceFailed {
        attempts: u32,
        suppressed: u64,
        cause: String,
    },
    Recovered {
        attempts: u32,
    },
    IterationSummary {
        pg_ctl_lines_emitted: u64,
        log_dir_files_tailed: u64,
        log_dir_lines_emitted: u64,
        dir_tailers: usize,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PostgresLineSource {
    pub(crate) producer: LogProducer,
    pub(crate) transport: LogTransport,
    pub(crate) path: String,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum PostgresLineLogEvent {
    Json {
        source: PostgresLineSource,
        severity: LogSeverity,
        message: String,
        payload: serde_json::Value,
    },
    Plain {
        source: PostgresLineSource,
        severity: LogSeverity,
        message: String,
        level_raw: String,
    },
    Unparsed {
        source: PostgresLineSource,
        raw_line: String,
    },
}
```

`ProcessJobKind` is already defined in `src/process/jobs.rs` and remains the authoritative exhaustive process-job enum used by logging. Its current exhaustive set must stay:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) enum ProcessJobKind {
    Bootstrap,
    BaseBackup,
    PgRewind,
    Promote,
    Demote,
    StartPostgres,
    StartPrimary,
    StartDetachedStandby,
    StartReplica,
}
```

The `LoggableEvent` implementation for each enum must produce exactly one `LogEventDto` and must convert its typed fields through typed `Serialize` DTOs. The required DTO structs are:

```rust
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub(crate) struct RuntimeStartupFields {
    pub(crate) startup_run_id: String,
    pub(crate) logging_level: crate::config::LogLevel,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub(crate) struct CauseFields {
    pub(crate) cause: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub(crate) struct SqlTransitionFields {
    pub(crate) previous: crate::pginfo::state::SqlStatus,
    pub(crate) next: crate::pginfo::state::SqlStatus,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub(crate) struct DcsModeTransitionFields {
    pub(crate) previous: Option<crate::dcs::DcsMode>,
    pub(crate) next: crate::dcs::DcsMode,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub(crate) struct CaptureSubprocessOutputFields {
    pub(crate) capture_subprocess_output: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub(crate) struct JobKindFields {
    pub(crate) job_kind: crate::process::jobs::ProcessJobKind,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub(crate) struct DataDirFields {
    pub(crate) data_dir: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub(crate) struct JobKindCauseFields {
    pub(crate) job_kind: crate::process::jobs::ProcessJobKind,
    pub(crate) cause: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub(crate) struct JobKindStreamBytesCauseFields {
    pub(crate) job_kind: crate::process::jobs::ProcessJobKind,
    pub(crate) stream: CapturedStream,
    pub(crate) bytes_len: usize,
    pub(crate) cause: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub(crate) struct JobKindStreamLineFields {
    pub(crate) job_kind: crate::process::jobs::ProcessJobKind,
    pub(crate) stream: CapturedStream,
    pub(crate) line: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub(crate) struct AttemptsFields {
    pub(crate) attempts: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub(crate) struct AttemptsSuppressedCauseFields {
    pub(crate) attempts: u32,
    pub(crate) suppressed: u64,
    pub(crate) cause: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub(crate) struct IterationSummaryFields {
    pub(crate) pg_ctl_lines_emitted: u64,
    pub(crate) log_dir_files_tailed: u64,
    pub(crate) log_dir_lines_emitted: u64,
    pub(crate) dir_tailers: usize,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub(crate) struct PostgresJsonLineFields {
    pub(crate) path: String,
    pub(crate) payload: serde_json::Value,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub(crate) struct PostgresPlainLineFields {
    pub(crate) path: String,
    pub(crate) level_raw: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub(crate) struct PostgresUnparsedLineFields {
    pub(crate) path: String,
    pub(crate) raw_line: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, Default)]
pub(crate) struct EmptyFields {}
```

These DTO structs are the only allowed reusable field DTO shapes. If implementation work finds a genuine missing shape, add it to this list in the task and use it deliberately. Do not fall back to `BTreeMap<String, Value>` or a field visitor.

However, because the required automatic field conversion mechanism is the serde-tagged enum representation shown above, event enums may also serialize their own inline variant fields directly. For example:

```rust
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "event_name", content = "fields")]
pub(crate) enum RuntimeLogEvent {
    #[serde(rename = "runtime.startup_entered")]
    StartupEntered {
        startup_run_id: String,
        logging_level: crate::config::LogLevel,
    },
}
```

That shape is valid and preferred over an extra wrapper struct when the variant fields are already clean. The reusable DTO structs listed above are for shared or repeated field groups, not a mandate to wrap every variant unnecessarily.

**Exact semantic rules for the exhaustive event set:**
- All app-domain events (`RuntimeLogEvent`, `PgInfoLogEvent`, `DcsLogEvent`, `ProcessLogEvent`, `PostgresIngestLogEvent`) use `LogSource { producer: App, transport: Internal, parser: App }`.
- `SubprocessLogEvent::Line` uses `LogSource { producer: PgTool, transport: ChildStdout | ChildStderr, parser: Raw }` based on `CapturedStream`.
- `PostgresLineLogEvent::Json` uses `LogParser::PostgresJson`.
- `PostgresLineLogEvent::Plain` uses `LogParser::PostgresPlain`.
- `PostgresLineLogEvent::Unparsed` uses `LogParser::Raw`.
- For all failure detail fields, the field name is `cause`, never `error`.
- `scope`, `member_id`, `cluster_name`, and `hostname` never appear in any event enum payload and never appear in any event field DTO. They are logger context only.
- `job_id` and `binary` are removed from logging entirely. They are not part of the generic logging contract and are not part of the final exhaustive event inventory.
- `message` remains the top-level human-readable summary line in `LogEventDto` / `LogRecord`.
- `event_name` is the stable machine-readable event identifier; `event_domain` is removed entirely because event names are already namespaced.
- Event names should be sourced from `#[serde(rename = "...")]` on enum variants, not from a separate manual string field whenever possible.

**Required event-name mapping:**

The event name strings must be fixed and exhaustive:

```text
runtime.startup_entered
pginfo.poll_failed
pginfo.sql_transition
dcs.connected_step_store_io_failed
dcs.connected_step_decode_failed
dcs.connected_step_already_exists
dcs.initial_connect_store_io_failed
dcs.initial_connect_decode_failed
dcs.initial_connect_already_exists
dcs.coordination_mode_transition
process.worker_run_started
process.request_received
process.inbox_disconnected
process.busy_rejected
process.start_postgres_already_running
process.start_postgres_preflight_failed
process.intent_materialization_failed
process.build_command_failed
process.spawn_failed
process.started
process.output_drain_failed
process.timeout
process.exited_successfully
process.exited_unsuccessfully
process.poll_failed
process.output_emit_failed
process.subprocess_line
postgres_ingest.step_once_failed
postgres_ingest.recovered
postgres_ingest.iteration_summary
postgres.line_json
postgres.line_plain
postgres.line_unparsed
```

If any implementation wants a different event name, update this task first. This list is authoritative.

**Required deletion list:**

The following log-specific types must be deleted and must not survive under other names unless explicitly replaced by a type defined in this task:
- `DomainLogEvent`
- `SealedLogEvent`
- `LogFieldVisitor`
- `LogEventMetadata`
- `LogEventSource` with `origin`
- `ProcessLogOrigin`
- `DcsLogOrigin`
- `PgInfoLogOrigin`
- `RuntimeLogOrigin`
- `PostgresIngestOrigin`
- `DcsFailure`
- `DcsLogIdentity`
- `RuntimeNodeIdentity`
- `PgInfoMemberIdentity`
- `PgInfoSqlTransition`
- `ProcessJobIdentity`
- `ProcessExecutionIdentity`

`event_domain` output and all function-name `origin` output must be removed from the final serialized record shape.

**Scope:**
- Rewrite `src/logging/event.rs` around the exact `LoggableEvent` and `LogEventDto` shape defined above.
- Rewrite `src/logging/mod.rs` so `LogSender` owns logger-global context, materializes `LogRecord`, and exposes only `send(event)`.
- Rewrite `src/logging/raw_record.rs` or remove it if the new DTO-to-record path makes it obsolete. The final queue/record materialization must remain private to `src/logging`.
- Replace stringly `LogBootstrapError` variants with the typed bootstrap error model defined above.
- Rewrite all event definitions in:
  - `src/runtime/log_event.rs`
  - `src/pginfo/log_event.rs`
  - `src/dcs/log_event.rs`
  - `src/process/log_event.rs`
  - `src/logging/postgres_ingest.rs`
- Rewrite all emitter call sites so they construct the new exhaustive event enums directly and do not pass logger-global context:
  - `src/runtime/node.rs`
  - `src/pginfo/worker.rs`
  - `src/dcs/worker.rs`
  - `src/process/worker.rs`
  - `src/logging/postgres_ingest.rs`
- Update any tests in:
  - `src/logging/mod.rs`
  - `src/logging/postgres_ingest.rs`
  - `src/runtime/node.rs`
  - `src/process/worker.rs`
  - any other file with log-shape assertions discovered during implementation

**Context from research:**
- Current logging trait split lives in `src/logging/event.rs` and is implemented in `src/runtime/log_event.rs`, `src/pginfo/log_event.rs`, `src/dcs/log_event.rs`, `src/process/log_event.rs`, and `src/logging/postgres_ingest.rs`.
- Current `LogSender::send` already gates by severity before queueing in `src/logging/mod.rs`, but it still calls `metadata()` separately and then `QueuedRecord::from_event(...)` calls `metadata()` again. The new one-step owned DTO design removes that duplication.
- Current process logs carry `job_id` and `binary` through `ProcessJobIdentity` and `ProcessExecutionIdentity`; those are logging-only wrappers and should be removed.
- Current DCS/runtime/pginfo logs carry `scope` and `member_id` in event payloads even though those values are constant for the node and come from config/bootstrap. Those belong in logger-owned context, not event payloads.
- Current bootstrap path in `src/logging/mod.rs` fails before any usable logger exists, so bootstrap/config problems must stay typed errors rather than emitted log events.
- Current postgres ingest line logs already have dynamic severity and dynamic parsed/unparsed payloads; those remain as typed internal enums in `src/logging/postgres_ingest.rs`, but they must still implement the one-step DTO conversion path.

**Expected outcome:**
- The repo has exactly one logging trait, and it has exactly one method that returns one owned DTO.
- The logger owns and injects `hostname`, `cluster_name`, `scope`, and `member_id`.
- The repo has exactly seven log event enums, no more and no less.
- The event enums derive `serde::Serialize` with `#[serde(tag = "event_name", content = "fields")]`, and logger internals extract `event_name` plus `fields` automatically from that serialized representation.
- No emitter passes node-global identity data into log events.
- No emitter uses `job_id` or `binary` in logs.
- No log-specific identity/origin wrapper types remain.
- No `error` field remains; event-local failure detail is `cause`.
- Bootstrap/configuration failures are typed bootstrap errors, not runtime log events.
- Record serialization is simpler and easier to inspect, and the event inventory is fixed by this task rather than inferred from the codebase.

</description>

<acceptance_criteria>
- [ ] `src/logging/event.rs` defines exactly the `LoggableEvent`, `LogEventDto`, and support types described in this task, with no `metadata()`/`write_fields()` split and no field visitor.
- [ ] Every log event enum in the exhaustive inventory derives `serde::Serialize` with `#[serde(tag = "event_name", content = "fields")]`, uses `#[serde(rename = "...")]` per variant, and logger internals extract `event_name` and `fields` via one shared helper instead of manual field-map assembly.
- [ ] `src/logging/mod.rs` injects logger-owned `LogContext` automatically and exposes only `LogSender::send(event)`.
- [ ] `LogRecord` matches the shape defined in this task, including `timestamp_ns`, `context`, `severity_text`, `severity_number`, `message`, `event_name`, `event_result`, `source`, and `fields`.
- [ ] `LogBootstrapError` is rewritten as typed variants and no longer uses stringly `Misconfigured(String)` / `SinkInit(String)` variants.
- [ ] `src/runtime/log_event.rs` contains exactly the `RuntimeLogEvent` enum defined in this task and implements `LoggableEvent`.
- [ ] `src/pginfo/log_event.rs` contains exactly the `PgInfoLogEvent` enum defined in this task and implements `LoggableEvent`.
- [ ] `src/dcs/log_event.rs` contains exactly the `DcsLogEvent` enum defined in this task and implements `LoggableEvent`.
- [ ] `src/process/log_event.rs` contains exactly `CapturedStream`, `ProcessLogEvent`, and `SubprocessLogEvent` as defined in this task and implements `LoggableEvent` for both events.
- [ ] `src/logging/postgres_ingest.rs` contains exactly `PostgresIngestLogEvent`, `PostgresLineSource`, and `PostgresLineLogEvent` as defined in this task and implements `LoggableEvent`.
- [ ] No other file under `src/` defines or implements any additional log event enum or any additional log-event trait.
- [ ] All function-origin enums and all log-only identity wrapper structs listed in the deletion list are removed.
- [ ] All emitters in `src/runtime/node.rs`, `src/pginfo/worker.rs`, `src/dcs/worker.rs`, `src/process/worker.rs`, and `src/logging/postgres_ingest.rs` stop passing `scope`, `member_id`, `hostname`, `cluster_name`, `job_id`, and `binary` as log-event payload data.
- [ ] All failure detail fields emitted by the new event set use `cause`, never `error`.
- [ ] All event names match the authoritative mapping in this task exactly.
- [ ] Logging tests are rewritten so they assert the new record shape and the new exhaustive event inventory instead of the old metadata/visitor design.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
