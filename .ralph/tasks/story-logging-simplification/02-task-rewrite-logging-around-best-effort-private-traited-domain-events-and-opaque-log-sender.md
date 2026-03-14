## Task: Rewrite Logging Around Best-Effort Private Traited Domain Events And An Opaque LogSender <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Refactor the logging subsystem so that the only outward-facing logging API is an opaque `LogSender` with a single `send(event)` style method, where `event` is a typed enum owned by the emitting domain and accepted through a trait bound. The higher-order goal is to make logging impossible to misuse and impossible to couple to business logic: non-logging code must not know about field maps, records, severities, sinks, tracing APIs, or serialization details, and logging failures must no longer alter worker or runtime control flow except when the send operation itself cannot enqueue because the channel is broken.

This task is a follow-up to the earlier logging simplification work. The repo already moved toward typed events, but the current implementation still has several architectural problems that must now be removed completely:
- logging still returns `Result` from sink emission and backend emission
- worker and runtime code still treat log emission failures as real workflow failures
- a central logging-owned sum enum still knows about multiple domains
- non-logging code still imports logging event wrapper types rather than owning its own domain log enums
- the current logging implementation still uses `tracing` as a backend bridge and preserves sink errors back to callers

The final design required by this task is strict:
- do not expose `tracing` anywhere outside `src/logging`
- do not expose any generic log record builder, field bag, manual severity constructor, or manual attribute assembly API outside `src/logging`
- all application logs must go through typed enums that implement a logging trait
- each domain owns its own log enum and the meaning of its variants
- severity must be implied by the enum variant or its typed payload, not passed around as a separate free-form argument from call sites
- the logging package must not require knowledge of every domain enum as a central public sum type
- `LogSender` must accept one argument, the typed event, and preferably render it immediately into one private queued-record shape before queueing
- send failure is only allowed when the queue itself is broken or closed; sink write failures, serialization failures, partial sink failures, or backend failures must become best-effort concerns handled inside logging rather than control-flow errors seen by domain code
- zero `emit_xxx` helper functions are allowed anywhere in the final design; domains must construct typed enum variants directly and call `log.send(event)` directly
- zero field-map construction or field mutation is allowed outside the final logging backend adapter that actually renders the private queued record into the backend output shape
- even inside `src/logging`, intermediate producers such as the postgres ingest pipeline must not manipulate generic field bags directly; they must also go through typed events plus the same trait/sender path, and only the final backend-rendering code may assemble output fields

The user explicitly does not want spans and does not want public `tracing` exposure. This task must not propose or preserve public span APIs. If `tracing` remains anywhere at all, it must remain a private implementation detail of `src/logging` only, and the preferred outcome is to remove the current tracing-backed bridge entirely if it is no longer needed for the final architecture.

The user also explicitly rejected keeping "some logs" outside this system. The required outcome is total coverage for application logging in this crate:
- all logs emitted by runtime, DCS, pginfo, process, postgres ingest, and any other crate-local component must flow through typed domain enums plus the single opaque `LogSender`
- no alternative direct app logging surface may remain available to normal code
- no raw `tracing::event!`, `tracing::info!`, or similar macros may be available outside `src/logging`

This task must be self-contained and should be implemented as a full architecture cleanup, not as a thin wrapper over the current design.

**Decisions already made from user discussion:**
- The user does not want spans.
- The user does not want `tracing` exposed anywhere except possibly as a private implementation detail inside logging.
- The user wants every application log to go through typed enums that implement a logging trait.
- The user wants each component/domain to own its own logging enum rather than centralizing domain event meaning inside logging.
- The user wants `LogSender` as the only public/crate-visible emission surface.
- `LogSender` should have one method that accepts one argument: the event.
- `LogSender` remains part of the design and is the only outward-facing logging handle seen by non-logging code.
- The `LogSender` internals must remain private. Non-logging code should only be able to hold/clone the opaque sender value and call its single exposed `send(event)` method.
- The logger must privately render the typed event into one queued-record shape before queueing so the queue can stay heterogeneous while domain enums remain separate.
- No non-logging code may manually construct logging fields, severities, or records.
- Logging should become best effort after enqueue. Only queue/send failure may be observed by callers; sink write failure must not fail runtime startup, worker steps, or state transitions.
- "Best effort" in this task means the logger keeps trying to log internally, buffering in memory where appropriate and degrading internally if needed, rather than surfacing backend write failures into domain logic.
- The current fail-fast logging checks are considered unnecessary complexity and must be removed from control flow.

**Scope:**
- Refactor `src/logging/mod.rs` into a narrow opaque sender-based boundary.
- Refactor `src/logging/event.rs` or replace it entirely so logging no longer owns one central public/crate-visible app-event sum enum for all domains.
- Remove any logging API that requires non-logging code to pass manual `SeverityText`, manual `origin`, generic field maps, or raw record builders.
- Refactor all non-logging emit sites in:
  - `src/runtime/node.rs`
  - `src/pginfo/worker.rs`
  - `src/dcs/worker.rs`
  - `src/process/worker.rs`
  - `src/logging/postgres_ingest.rs`
  - any additional app-log emission sites discovered with `rg -n "\.emit\(|emit_[a-z_]+\(" src`
- Ensure `src/logging` is the only place allowed to depend on `tracing` if `tracing` remains at all.
- Add a compile-time boundary that prevents other modules from manually assembling log fields or manual severity-bearing records.
- Replace logging-failure-as-control-flow with best-effort semantics across runtime and worker code.
- Keep PostgreSQL log ingestion inside the same typed-event architecture, but the postgres-specific typed enums may remain private to logging injectors if that is the cleanest design.

**Out of scope:**
- Do not add public spans or tracing APIs.
- Do not preserve backwards compatibility for the old logging surface. This repo is greenfield and should delete obsolete surfaces.
- Do not leave transitional helper APIs around "just in case". The final boundary should be singular and enforced.

**Context from research:**
- The current logging backend intentionally propagates sink failures:
  - `LogSink::emit` returns `Result<(), LogError>` in `src/logging/mod.rs`
  - `TracingBackend::emit` returns `Result<(), LogError>`
  - `LogHandle::emit` returns `Result<(), LogError>`
  - there is a test named `tracing_backend_preserves_emit_errors_when_sink_fails` in `src/logging/mod.rs`
- The current sink/backend error types are real operational failures, not placeholders:
  - JSON serialization failure
  - stderr lock poisoning / write failure
  - file sink lock poisoning / write failure
  - fanout total failure when all sinks fail
  - internal tracing bridge errors such as nested tracing-backed emission or missing result
- The current codebase currently has real control-flow coupling to logging failures:
  - startup fatal on runtime startup log failure in `src/runtime/node.rs`
  - pginfo worker step fails before publishing state if log emission fails in `src/pginfo/worker.rs`
  - DCS worker step fails on multiple DCS event log failures in `src/dcs/worker.rs`
  - several DCS error branches log first and only then set `store_healthy = false`, meaning log failure can prevent the unhealthy-state update
  - process worker startup, request handling, job transitions, timeout handling, exit handling, output drain handling, and subprocess output handling all propagate logging failure into `WorkerError` in `src/process/worker.rs`
  - several process branches log first and only then call `transition_to_idle(...)`, meaning log failure can prevent intended job-outcome state transitions
  - postgres ingest run loop propagates logging failures from summary/recovery/error events and also turns individual line-emission failures into iteration errors in `src/logging/postgres_ingest.rs`
- The current DCS and process helpers that reveal the problem are:
  - `emit_dcs_event(...)` in `src/dcs/worker.rs`
  - `emit_process_event(...)` in `src/process/worker.rs`
  - `emit_pginfo_event(...)` in `src/pginfo/worker.rs`
  - `emit_ingest_event(...)` and `emit_postgres_line(...)` in `src/logging/postgres_ingest.rs`
- The current logging-owned cross-domain event shape is in `src/logging/event.rs`:
  - `LogEvent` is a central sum enum with variants for runtime, DCS, pginfo, process, postgres ingest, postgres line, and subprocess line
  - `InternalEvent<T>` carries a separate `SeverityText`
- The current outward logging call shape is in `LogHandle::emit(origin, event)` in `src/logging/mod.rs`
- Current helper wrappers in non-logging code exist primarily to adapt domain enums into the logging-owned `LogEvent` plus separate severity:
  - `src/process/worker.rs`
  - `src/pginfo/worker.rs`
  - `src/dcs/worker.rs`
- Current `tracing` usage appears confined to `src/logging/mod.rs`. Research did not find repo-local evidence that normal domains need direct raw `tracing` access today.
- In `src/dcs/worker.rs`, several DCS error branches currently log first and only then set `store_healthy = false`; because those log helpers return `WorkerError` on sink failure, a logging failure can prevent the unhealthy-state update from happening in that step. The clearest examples are the branches around local member delete/write failure, watch drain failure, watch apply/refresh failure, snapshot apply/refresh/read failure, and local leader release failure.

**Required architectural target:**
- Introduce or preserve per-domain typed event enums owned by the domain modules or by domain-owned typed submodules. Logging must not own a central public/crate-visible application event sum type that forces all domains through one logging-owned enum.
- Introduce an opaque `LogSender` type in `src/logging` as the only emission surface seen by non-logging code.
- `LogSender` must expose one send method that accepts a single event argument through a trait bound.
- The logging trait must be the only way a domain event can become loggable.
- The trait must describe semantic output, not transport mechanics. It should provide enough information for logging internals to derive severity, message, event name/domain/result, and structured fields without letting callers mutate a field bag.
- The send path should preferably render the typed event immediately into one private queued-record shape so one shared queue can carry heterogeneous events without needing a central private per-producer enum. A central private enum is not the preferred target.
- The channel message/queued-record type must stay private to `src/logging`.
- Logging must own sink writing, buffering, retry/degrade behavior, serialization, attribute construction, and field dropping if needed.
- Normal code must not be able to manually choose arbitrary keys or severities for logs.

**Proposed API shape to implement:**
The exact names may differ if a better naming choice appears during implementation, but the architecture must be equivalent to this shape and remain intentionally small. The preferred design is to render a typed event into one private queued-record shape at `send(...)` time rather than queueing a central private enum with one variant per producer.

```rust
pub(crate) struct LogSender {
    // private sender field
}

impl Clone for LogSender { /* normal clone semantics */ }

impl LogSender {
    pub(crate) fn send<E>(&self, event: E) -> Result<(), LogSendError>
    where
        E: DomainLogEvent + Send + 'static,
    {
        // eagerly render event into a private queued record and enqueue it
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum LogSendError {
    #[error("log channel closed: {0}")]
    ChannelClosed(String),
}

pub(crate) trait DomainLogEvent: Send + 'static {
    fn severity(&self) -> LogSeverity;
    fn occurred_at(&self) -> Option<LogTimestamp>;
    fn domain(&self) -> &'static str;
    fn event_name(&self) -> &'static str;
    fn message(&self) -> String;
    fn write_fields(&self, out: &mut LogFieldWriter);
}
```

If naming permits without creating ambiguity with the private queued-record type, the implementer should prefer naming this trait `LogEvent` instead of `DomainLogEvent`, since the user explicitly prefers the simpler name. A longer trait name is acceptable only if needed to avoid a genuinely confusing core-type collision.

The task implementer may use a sealed trait if desired:

```rust
mod seal {
    pub trait Sealed {}
}

pub(crate) trait DomainLogEvent: seal::Sealed + Send + 'static {
    fn severity(&self) -> LogSeverity;
    fn occurred_at(&self) -> Option<LogTimestamp>;
    fn domain(&self) -> &'static str;
    fn event_name(&self) -> &'static str;
    fn message(&self) -> String;
    fn write_fields(&self, out: &mut LogFieldWriter);
}
```

This is not meant to create two real event traits in the final architecture. It is only showing an optional sealing pattern:
- either use one trait alone
- or use one real trait plus one tiny private `Sealed` trait to prevent arbitrary implementations

There must not be two separate user-meaningful event traits in the final design.

The concrete string-returning methods can be replaced by more static forms if the implementer finds a cleaner design, but the core idea is fixed:
- domains expose typed semantics
- `LogSender::send(...)` privately renders them
- the queue stores one private queued-record shape
- no per-producer queue enum is required

The private queued-record and final backend rendering internals should stay private to `src/logging`, with a shape along these lines:

```rust
struct QueuedLogRecord {
    severity: LogSeverity,
    occurred_at: Option<LogTimestamp>,
    observed_at: LogTimestamp,
    domain: &'static str,
    event_name: &'static str,
    message: String,
    fields: PrivateFieldList,
}
```

The task should explicitly prefer this queued-record design over a central private enum with one variant per producer. A central private enum is allowed only if the implementer can justify it as materially simpler after trying the generic queued-record approach first.

`LogFieldWriter` should have a private constructor and be creatable only by logging. If the implementer finds a stricter design that avoids even exposing field-writing methods in ordinary domain impls, that is preferred.

The important constraints are:
- no per-producer queue enum is required
- no private queue item type leaks out of logging
- non-logging code can only submit typed events
- field materialization happens only through the trait/rendering path, never through free-form field maps at call sites

The task should not assume a derive-macro solution. A derive macro is not required and should not drive the architecture. The main problem to solve is the runtime boundary and the type ownership, not reducing impl boilerplate.

**Proposed supporting private types:**

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LogSeverity {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LogTimestamp(pub(crate) i64);

pub(crate) struct LogFieldWriter {
    // private fields, private constructor
}
```

`LogTimestamp` should prefer signed Unix timestamp nanoseconds internally rather than `SystemTime`. The user explicitly asked for a Unix-nanoseconds representation, and signed `i64` is preferred over `u64` here so the type can also represent pre-epoch timestamps and align better with common storage/interoperability expectations.
`LogFieldWriter` should stay almost entirely opaque. The preferred surface is one small `write_field(...)` entry point over a closed set of logging field values, rather than exposing a mutable field bag API.

**Proposed DCS example enum shape:**
The exact variant list can differ if the codebase evolves. This is only an example to show the pattern, owned by the DCS domain rather than by a central logging-owned sum enum. Prefer a composed shape with shared DCS identity/domain data separated from the per-event kind:

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DcsLogDomain {
    pub(crate) scope: String,
    pub(crate) member_id: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DcsCommandName {
    AcquireLeadership,
    ReleaseLeadership,
    PublishSwitchover,
    ClearSwitchover,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DcsLogEvent {
    pub(crate) domain: DcsLogDomain,
    pub(crate) kind: DcsLogKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DcsLogKind {
    LocalMemberDeleteFailed {
        error: String,
    },
    StoreHealthTransition {
        store_healthy: bool,
    },
    TrustTransition {
        previous: Option<crate::dcs::DcsTrust>,
        next: crate::dcs::DcsTrust,
    },
    ...
}
```

This example is intentionally only an example for the DCS domain. The important part is the pattern:
- the domain owns the composed logging type
- shared DCS identity/domain data is not duplicated across every variant
- the event kind still implies severity and schema
- the call site only constructs the typed value and sends it
- no helper like `emit_dcs_event(...)` is allowed to survive
- no call site manually passes severity or field maps

The trait implementation may delegate field writing across the composed pieces. For example, `DcsLogEvent` may first ask `domain` to write shared fields such as `scope` and `member_id`, then ask `kind` to write event-specific fields. Message derivation may remain on the kind-specific side. This staggered internal composition is desirable if it keeps the code cleaner.

**Required boundary/privacy outcome:**
- `src/logging` is the only module allowed to depend on `tracing`. No other file under `src/` should import `tracing` or use `tracing::...` macros after this task.
- Non-logging modules must not construct:
  - `LogRecord`
  - raw attribute maps
  - manual severities for application events
  - any private queued-record or backend type
  - any raw sender type from the underlying queue implementation
- Non-logging modules must not call anything like:
  - `emit(origin, ...)`
  - `emit_direct(...)`
  - field insertion helpers
  - manual record builders
- The only allowed action outside logging should be "send this typed event enum value".
- `LogSender` must remain opaque. Non-logging code may hold it, clone it, and call `send(event)`, but may not access any inner queue, sink, or backend state.
- `emit_xxx` wrappers are explicitly forbidden everywhere in the final design, including inside non-logging domains and inside logging-internal postgres/subprocess injector code. There must be zero surviving helper functions whose purpose is "wrap event plus severity plus origin and call logger". Call sites must construct the typed enum value directly and send it directly.
- Generic field maps or key/value bags are explicitly forbidden outside the final backend renderer inside `src/logging`. The postgres ingest pipeline, subprocess ingest path, and any other logging-internal producer must also use typed events rather than building field bags by hand.

**Required control-flow outcome:**
- Logging backend failures must no longer be able to:
  - abort node startup after config validation
  - prevent pginfo state publication
  - prevent DCS health/trust transitions from being computed and published
  - prevent `store_healthy` updates in DCS
  - prevent process job transitions, outcome publication, or idle transitions
  - fail the postgres ingest loop because sink writes or event serialization failed
- The only logging-related failure visible to callers should be inability to enqueue because the sender/receiver channel is broken or closed.
- Best-effort logging must stay internal after enqueue.

**Required implementation direction for domains:**
- `runtime`, `pginfo`, `dcs`, `process`, and any other emitting domain should own their own typed log enums.
- Severity must be encoded by the event meaning itself, not by passing `SeverityText` from call sites.
- Existing helper functions like `emit_process_event`, `emit_pginfo_event`, and `emit_dcs_event` should disappear or collapse into plain domain event constructors plus `log.send(event)`.
- The postgres ingest pipeline should also emit typed events only. Postgres-json lines, plain postgres lines, subprocess lines, ingest iteration summaries, and ingest failures may remain private logging-internal enums if they are produced entirely inside `src/logging`, but they must still follow the same trait-based single-send-path architecture.

**Concrete repo areas to audit and modify:**
- `src/logging/mod.rs`
- `src/logging/event.rs`
- `src/logging/raw_record.rs`
- `src/logging/postgres_ingest.rs`
- `src/runtime/node.rs`
- `src/pginfo/worker.rs`
- `src/dcs/worker.rs`
- `src/process/worker.rs`
- any logging tests in `src/logging/mod.rs`
- any additional emitter sites found with ripgrep during implementation

**Specific current control-flow sites that must be eliminated:**
- Runtime startup log failure path in `src/runtime/node.rs`
- Pginfo `PollFailed` and `SqlTransition` log failure paths in `src/pginfo/worker.rs`
- DCS logging failure paths around:
  - `LocalMemberDeleteFailed`
  - `LocalMemberWriteFailed`
  - `WatchDrainFailed`
  - `WatchApplyHadErrors`
  - `WatchRefreshFailed`
  - `SnapshotApplyHadErrors`
  - `SnapshotRefreshFailed`
  - `SnapshotReadFailed`
  - `StoreHealthTransition`
  - `TrustTransition`
  - `LocalLeaderReleaseFailed`
  - `CommandResponseDropped`
- In particular, the current DCS branches that call `emit_dcs_event(...)` and only then set `store_healthy = false` must be rewritten so DCS correctness never depends on whether logging succeeds. The logging refactor is not complete unless those correctness updates happen independently of logger health.
- Process logging failure paths around:
  - worker startup
  - request received
  - inbox disconnected
  - busy rejected
  - start-postgres noop / preflight failure
  - intent materialization failure
  - build command failure
  - spawn failure
  - job started
  - timeout
  - exited successfully / unsuccessfully
  - poll failure
  - output drain failure
  - output emit failure
- Postgres ingest failure paths around:
  - recovered
  - step failure
  - iteration summary
  - line emission that currently becomes iteration failure

**Patterns to follow:**
- Prefer compile-time privacy over convention.
- Prefer a sealed trait or an otherwise closed trait surface if that helps prevent arbitrary external implementations.
- Prefer domain-owned ADTs that make invalid logging states impossible.
- Prefer eager rendering into one private queued-record shape before queueing rather than exposing a public queue item type.
- Prefer deleting old APIs over carrying compatibility layers.
- Prefer code reduction. The current thin `emit_*` wrappers are a smell and should disappear once the sender boundary is correct.
- Prefer direct call-site construction of typed enum variants over helper wrapper functions.
- Prefer one final backend-rendering adapter that turns a private queued record into backend fields or records; no other code should manipulate output field maps.

**Expected outcome:**
- Every application log in the crate flows through one opaque `LogSender`.
- Every emitted event is a typed enum value that implements the logging trait.
- No code outside `src/logging` can manually assemble log fields, records, or severities.
- No code outside `src/logging` can use `tracing`.
- Logging no longer affects runtime or worker correctness after an event has been accepted by the sender.
- Logging internals may still degrade internally when sinks misbehave, but those failures are contained within logging.
- The codebase has a much smaller and more defensible logging boundary than it has now.

</description>

<acceptance_criteria>
- [ ] Replace the current logging emission boundary in `src/logging/mod.rs` with an opaque `LogSender` that exposes only a single-event send API and hides the underlying queue type and private queued-record type.
- [ ] Remove the current `LogHandle::emit(origin, event)` style outward API for normal domain code, or reduce it to a private/internal implementation detail that non-logging modules cannot call.
- [ ] Refactor `src/logging/event.rs` so logging no longer owns one central public/crate-visible application-event sum enum spanning runtime, DCS, pginfo, process, and ingest domains.
- [ ] Introduce or preserve per-domain typed logging enums for runtime, DCS, pginfo, process, and any other emitting non-logging modules; each domain enum must define its own variants and implied severity semantics.
- [ ] Introduce a logging trait that typed events implement; the trait must be the only route by which normal code can make an event loggable, and it must not expose mutable field bags or manual severity assembly to callers.
- [ ] Implement the architecture using a concrete `LogSender` plus a trait/method shape materially equivalent to the proposed signatures in this task, including one private queued-record shape and a caller-visible send error that only represents broken channel/enqueue failure.
- [ ] Ensure the send path eagerly renders the typed event into one private queued-record shape before queueing so one shared queue can carry heterogeneous events without exposing any queue item type publicly.
- [ ] Ensure the logging package keeps `LogRecord`, attribute maps, severity constructors, sinks, and queue message types private to `src/logging`.
- [ ] Remove all non-logging imports or usages of `tracing`; after the refactor, only `src/logging` may depend on `tracing`, and if `tracing` is no longer needed internally, remove it entirely.
- [ ] Remove all non-logging ability to construct manual log fields, manual records, manual severities for application events, or direct private queued-record values.
- [ ] Remove every `emit_xxx` helper function from the codebase. Zero such wrapper functions are allowed to remain after the refactor, including in `src/process/worker.rs`, `src/pginfo/worker.rs`, `src/dcs/worker.rs`, and `src/logging/postgres_ingest.rs`.
- [ ] Remove every field-bag or manual field-construction path outside the final backend renderer in `src/logging`; postgres ingest and any other logging-internal producer code must also emit typed events rather than assembling generic fields directly.
- [ ] Rewrite `src/runtime/node.rs` so startup logging failure cannot abort node startup except for a broken sender/channel during bootstrap if that is still part of startup invariants.
- [ ] Rewrite `src/pginfo/worker.rs` so `PollFailed` and `SqlTransition` logging cannot prevent state calculation or publication after the event has been accepted by the sender.
- [ ] Rewrite `src/dcs/worker.rs` so logging cannot prevent `store_healthy` updates, trust computation, command handling, or state publication after send succeeds.
- [ ] Rewrite `src/process/worker.rs` so logging cannot prevent request handling, job start/failure/success transitions, timeout handling, output handling, or `transition_to_idle(...)` after send succeeds.
- [ ] Rewrite `src/logging/postgres_ingest.rs` so sink/serialization/backend failures no longer fail the ingest loop after an event has been accepted by the sender, and line-emission issues do not become workflow failures solely because logging backends misbehaved.
- [ ] Remove obsolete helper wrappers such as the current `emit_process_event`, `emit_pginfo_event`, `emit_dcs_event`, and similar adapter functions if they no longer serve a meaningful purpose beyond wrapping the old API.
- [ ] Update logging tests so they validate the new best-effort sender/worker architecture rather than asserting propagation of sink failures back to domain callers.
- [ ] Remove or rewrite the current sink-failure-preservation behavior and tests in `src/logging/mod.rs`, including the current expectation that tracing-backed emission returns sink write errors to callers.
- [ ] Add or update tests proving that normal business/workflow code continues correctly even when logging sinks fail internally after enqueue.
- [ ] Add or update tests proving that the only caller-visible send failure is a broken/closed queue and that backend sink failure remains internal to logging.
- [ ] Verify with ripgrep that no non-logging module under `src/` imports `tracing` or manually constructs logging records/field maps after the refactor.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
