# Verbose Context: Process Logging Boundary

This file is raw factual context for documentation drafting.

The logging subsystem is now centered on an opaque `LogSender` in `src/logging/mod.rs`.

Facts from the current code:

- `LogSender` is the only outward-facing application logging handle used by non-logging code.
- `LogSender` exposes `send(event)` where `event` implements the sealed `DomainLogEvent` trait.
- `LogSender::send(...)` only returns `LogSendError::QueueClosed`.
- `LogSender::send(...)` does not expose field bags, records, severities, tracing APIs, or queue internals.
- `LogSender` filters by minimum app severity before queueing, using event metadata severity.
- `LogSender` eagerly materializes the typed event into the private `raw_record::QueuedRecord` shape before queueing.
- The queue payload type is private to `src/logging`.
- `LogWorker` receives queued records, converts them into final `LogRecord` values, and forwards them to the backend.
- `LogWorker` discards backend sink failures internally after dequeue. The worker currently does `let _ = self.backend.emit(&materialized);`.
- This means logging is best effort after enqueue.

Facts about the logging trait and domain ownership:

- `src/logging/event.rs` defines the sealed logging contract with `DomainLogEvent`, `SealedLogEvent`, `LogEventMetadata`, `LogEventSource`, `LogEventResult`, and `LogFieldVisitor`.
- Each domain owns its own typed log ADTs instead of routing application meaning through one central logging-owned sum enum.
- Runtime-owned events live in `src/runtime/log_event.rs`.
- DCS-owned events live in `src/dcs/log_event.rs`.
- PgInfo-owned events live in `src/pginfo/log_event.rs`.
- Process-owned events live in `src/process/log_event.rs`.
- Logging-internal postgres ingest events live in `src/logging/postgres_ingest.rs` as private typed enums.

Facts about process-domain logging after the refactor:

- `src/process/worker.rs` no longer has `emit_process_event(...)` wrappers.
- `src/process/worker.rs` now constructs `ProcessLogEvent` or `SubprocessLogEvent` values directly and calls `ctx.runtime.log.send(...)` directly.
- `src/process/log_event.rs` owns the process event taxonomy.
- `ProcessLogEvent` covers worker startup, request receipt, inbox disconnect, busy rejection, preflight failures, command-build failures, spawn failures, job started, timeout, exit success, exit failure, poll failure, output drain failure, and output emit failure.
- `SubprocessLogEvent` represents stdout/stderr lines from child processes as a separate typed event.
- `SubprocessLogEvent` carries producer, origin, execution identity, stream, and bytes.
- Process execution identity is modeled with `ProcessExecutionIdentity`, which embeds `ProcessJobIdentity`.
- Process job kind is recorded through `ProcessJobKind`.
- Process stdout lines map to info severity and child-stdout transport.
- Process stderr lines map to warn severity and child-stderr transport.
- Process log fields include `job.id`, `job.kind`, `binary`, `stream`, `bytes_len`, and `error` where appropriate.

Facts about process worker behavior and control flow after the refactor:

- Process worker log sends still map queue-closed errors into `WorkerError::Message(...)` at the point of send.
- Backend sink failures no longer affect the process worker after the event has been accepted by the queue.
- Output-drain and output-emit logging still publish typed process events, but only queue failure is visible to the caller.
- The process worker still transitions jobs back to idle and publishes outcomes after successful sends.
- Subprocess output capture is still controlled by `logging.capture_subprocess_output` in runtime configuration.
- `ProcessRuntime` stores `log: LogSender`, `capture_subprocess_output`, and the process command runner.

Facts about runtime startup and the log worker:

- `src/runtime/node.rs` bootstraps logging first.
- `bootstrap(...)` returns `LoggingSystem { sender, worker }`.
- Runtime sends the startup event through `log.send(runtime_startup_event(...))`.
- Worker orchestration now joins the non-fallible log worker separately from fallible workers using `tokio::join!`.
- Runtime, pginfo, dcs, process, HA, API, and postgres ingest all share cloned `LogSender` values.

Facts about postgres ingest in the new architecture:

- `src/logging/postgres_ingest.rs` no longer uses `emit_ingest_event(...)`, `emit_ingest_step_failure(...)`, `emit_ingest_retry_recovered(...)`, or `emit_postgres_line(...)`.
- Postgres ingest now constructs `PostgresIngestLogEvent` or `PostgresLineLogEvent` values directly and sends them through `LogSender`.
- `PostgresIngestLogEvent` covers step failure, recovery, and iteration summary.
- `PostgresLineLogEvent` covers JSON, plain, and unparsed postgres lines.
- The helper `postgres_line_event(...)` is a pure builder that returns a typed event. The caller performs the send.
- Postgres ingest queue-send failures are still surfaced as worker errors because they mean the logging queue is broken.
- Sink and backend failures after enqueue remain internal to logging.

Facts about tracing visibility:

- `tracing` usage remains inside `src/logging/mod.rs`.
- Non-logging modules do not use `tracing`.
- The private backend bridge currently uses `TracingBackend` and a private `dispatch_tracing_record_event(...)` helper.

Facts about DCS logging after the refactor:

- DCS owns its own typed events in `src/dcs/log_event.rs`.
- The refactor also corrected a misleading event mapping by using generic failure events `ConnectedStepFailed` and `InitialConnectFailed`.
- Those names now match the real failure boundary instead of implying that every connected failure was specifically a watch-refresh failure or that every initial connect failure was specifically a snapshot-read failure.

Facts about documentation impact:

- The existing `docs/src/explanation/process-management.md` page already discusses subprocess output capture and typed subprocess events.
- The new code makes the logging boundary more explicit: process code only holds an opaque `LogSender`, owns typed process log ADTs locally, and does not know about record rendering or backend sinks.
- A documentation update should stay within process-management scope and explain how process execution and logging interact today.
