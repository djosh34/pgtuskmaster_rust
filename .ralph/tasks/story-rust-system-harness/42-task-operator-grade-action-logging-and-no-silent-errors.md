## Task: Enforce Operator-Grade Action Logging And No Silent Error Swallowing <status>done</status> <passes>true</passes>

<description>
**Goal:** Make runtime/operator observability explicit and uniform: debug-log all actions and all meaningful runtime flow steps across the codebase so operators can reconstruct exactly what code path executed, in order; info-log important operator lifecycle/default events; warn-log ignorable errors; error-log hard errors; and eliminate silent error swallowing.

**Scope:**
- Keep using the existing unified logging infra (`src/logging/*`, `LogHandle`, `LogRecord`, existing producers/parsers/sinks).
- Do not introduce alternate logging frameworks, side channels, or ad-hoc `println!/eprintln!` for runtime internals.
- Add structured log events for action intent/result paths and worker lifecycle/error paths.
- Instrument runtime flow broadly with debug logs, not only actions:
  - worker loop iterations and `step_once` entry/exit where meaningful,
  - branch decisions and selected execution paths,
  - job dispatch/queue/transition points,
  - external IO boundaries (accept/read/write, DCS watch/write refresh, subprocess spawn/poll/exit, ingest tail scan/parse/emit).
- Ensure debug records include enough context to trace execution unambiguously (module/function origin plus correlation ids like job_id/action id/tick/member when applicable).
- Replace silent `let _ = ...` error drops in runtime loops with explicit structured events at appropriate severities.
- Preserve existing parse fallback invariant in postgres ingest: parse failures must still emit the original line (`parse_failed` + raw payload), never drop.

**Context from research:**
- Current logging pipeline is centralized and structured, but several loops swallow operational errors:
  - `src/api/worker.rs` run loop ignores `step_once` errors.
  - `src/logging/postgres_ingest.rs` ignores `step_once` and cleanup errors.
  - `src/process/worker.rs` and `src/runtime/node.rs` ignore subprocess line emit failures.
- HA action dispatch is state-driven but lacks explicit, per-action intent/result structured events for operators.
- Debug APIs expose snapshots/changes, but operators still need direct structured event logs to reconstruct timelines without inference.

**Expected outcome:**
- Runtime code is traceable at debug level end-to-end: with debug logging enabled, operators can follow what happened and where in code without guessing.
- Every important control-plane/runtime action emits explicit structured logs with clear severity policy:
  - action intent + completion at debug,
  - general runtime flow and branch/decision breadcrumbs at debug,
  - operator-important lifecycle/status at info,
  - ignorable/recoverable errors at warn,
  - hard failures at error.
- Silent swallowing of runtime/worker errors is removed.
- Existing logging interfaces and schema remain the single source of truth.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] Global policy: with debug enabled, logs provide near step-by-step runtime traceability across workers and orchestration paths (not only HA actions), including clear module/function origin and correlation attributes.
- [x] `src/runtime/node.rs`: add pervasive debug breadcrumbs for startup planning, startup mode selection, startup job boundaries, worker wiring/startup, and key branch outcomes.
- [x] `src/ha/worker.rs`: add structured debug events for each dispatched HA action attempt and result (success/failure), with action id/type, tick, and correlation attributes.
- [x] `src/ha/worker.rs`: emit info events for major HA phase transitions and operator-important role/lease transitions.
- [x] `src/runtime/node.rs`: emit info lifecycle events for startup mode selection and startup stage boundaries; emit error events for startup failures/timeouts; do not silently ignore startup subprocess log-emit failures.
- [x] `src/process/worker.rs`: keep per-line subprocess ingestion, and replace ignored emit failures with structured warn/error events using existing logging interface.
- [x] `src/process/worker.rs`: add broad debug instrumentation for queue handling, state transitions (`Idle`/`Running`), poll/timeout/cancel paths, and job outcome publication with job_id/job_kind/binary attributes.
- [x] `src/logging/postgres_ingest.rs`: preserve parse-failure fallback semantics (no parse-loss regression), and surface ingest-step/cleanup failures as structured warn/error events instead of silent ignore.
- [x] `src/logging/postgres_ingest.rs`: add debug breadcrumbs for file discovery/tailing path decisions and parser path selection (json/plain/raw) at meaningful boundaries.
- [x] `src/api/worker.rs`: remove silent ignore of `step_once` errors in run loop; emit structured debug breadcrumbs for request lifecycle (accept/auth/route/respond) and warn/error events with connection/request context where available.
- [x] `src/dcs/worker.rs`: add debug instrumentation for watch/drain/apply/write phases, plus info for trust/health transitions and warn/error on degraded DCS health transitions where actionable.
- [x] `src/pginfo/worker.rs`: add debug instrumentation for poll/publish flow and warn/info visibility for connectivity transitions (healthy <-> unreachable) while keeping publish semantics.
- [x] `src/debug_api/worker.rs` and `src/debug_api/view.rs`: ensure debug timeline/events remain coherent with newly introduced logs (no contradictory severity/state narratives).
- [x] `src/logging/mod.rs`: keep existing logging interfaces as the only runtime logging path; do not add alternate logger frameworks.
- [x] `src/logging/postgres_ingest.rs` tests: explicitly verify parse-failed lines still emit with original payload retained.
- [x] `src/api/worker.rs`, `src/process/worker.rs`, `src/runtime/node.rs`, `src/ha/worker.rs`, `src/dcs/worker.rs`, `src/pginfo/worker.rs` tests: add/adjust coverage proving key debug breadcrumbs exist and previously swallowed errors are now surfaced as structured events.
- [x] Verification task: static/code-level check demonstrates no runtime-path swallowed errors remain (no silent `let _ = ...` for operational errors in worker/runtime loops; remaining intentional best-effort paths must emit warn/info with rationale in code).
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

---

## Plan

### Principles / Non-Negotiables (from task)
- [x] Use existing unified logging infra only (`src/logging/*`, `LogHandle`, `LogRecord`, existing sinks/producers/parsers).
- [x] Do not add alternate logging frameworks (`tracing`, `log`, `println!/eprintln!` for runtime internals).
- [x] Debug-level should allow near step-by-step reconstruction of runtime paths (within each worker), with enough correlation context to follow causality.
- [x] Info-level should include operator-relevant lifecycle/phase/role transitions.
- [x] Warn-level for recoverable/ignorable but actionable failures; Error-level for hard failures (including any previously swallowed operational errors).
- [x] Eliminate silent error swallowing in runtime loops (no `let _ = ...` / `.ok()` / ignored `Err(_)` in worker/runtime loops where the error is operationally meaningful).
- [x] Preserve postgres ingest invariant: parse failures still emit the original line (`parse_failed` + raw payload), never drop.

### Phase 0 — Repo Recon + Event Taxonomy (small additive schema; no sink changes)
- [x] Define a consistent set of *additive* structured attributes for “events” (keep `LogRecord` schema unchanged; use `attributes` only):
  - [x] `event.name` (string, low-cardinality, dot-separated; e.g. `process.job.started`)
  - [x] `event.domain` (string; e.g. `runtime|ha|dcs|process|api|pginfo|postgres_ingest`)
  - [x] `event.result` (string; `ok|failed|skipped|suppressed|timeout|recovered`)
  - [x] Common correlation keys (only when available): `scope`, `member_id`, `ha_tick`, `action_id`, `ha_dispatch_seq`, `job_id`, `job_kind`, `binary`.
  - [x] Preserve existing “stable” keys already emitted today (do not rename): `job_id`, `job_kind`, `binary`, `raw_bytes_hex`, `parse_failed`, `raw_line`, `postgres.level_raw`, `postgres.json`.
  - [x] Common request keys (API; privacy-filtered + namespaced to avoid collisions):
    - [x] `api.peer_addr`, `api.method`, `api.route_template`, `api.status_code`, `api.request_id` (from `X-Request-Id` if present; otherwise omitted).
    - [x] `api.auth.header_present` (bool), `api.auth.result` (`allowed|unauthorized|forbidden`), `api.auth.required_role` (string).
    - [x] CRITICAL: never log raw `Authorization` header value, bearer tokens, or request body fields. Only log safe metadata (bytes, parse_ok, route template).
- [x] Add a small helper API in logging core to make emitting structured “event records” consistent and low-boilerplate (no new framework):
  - [x] Option A (preferred): `LogHandle::emit_event(...) -> Result<(), LogError>` which:
    - [x] fills `LogRecord` with message + `LogSource { producer: App, transport: Internal, parser: App, origin: "<module>::<function>" }`
    - [x] injects `event.*` attributes plus caller-provided attributes
  - [x] Ensure *call sites never ignore* the returned `Result` (propagate or handle).
- [x] Decide origin naming convention for this task (apply uniformly):
  - [x] `origin="api_worker::step_once"`, `origin="process_worker::tick_active_job"`, `origin="ha_worker::dispatch_actions"`, `origin="runtime::plan_startup"`, `origin="postgres_ingest::step_once"`, etc.

### Phase 1 — Remove Concrete Silent Swallowing (targeted; then broaden)

#### 1A) `src/runtime/node.rs` (startup planning + startup job subprocess loop)
- [x] Add debug breadcrumbs for:
  - [x] `run_node_from_config` entry/exit (include `scope`, `member_id`, `startup_run_id`, config versions where available).
  - [x] logging bootstrap completed (explicit event after `crate::logging::bootstrap` succeeds).
  - [x] Data dir inspection result (`data_dir_state`, reason on ambiguity error).
  - [x] DCS cache probe: replace `.ok()` with explicit match; emit `Info` on start/success and `Warn` on failure (“continuing without DCS cache”), include error detail and fallback selected.
  - [x] Startup mode selection: `Info` (“mode selected”), include `mode`, `data_dir_state`, leader evidence source, init-lock presence, restore-bootstrap enabled flag.
  - [x] Startup action planning: `Debug` per planned action (index, action kind, expected job_kind/binary/timeout).
- [x] Add info lifecycle events for:
  - [x] startup planning began/completed,
  - [x] startup mode selection,
  - [x] startup stage boundaries (e.g., claim lock, bootstrap/restore, takeover, start postgres).
  - [x] worker bootstrap boundaries: API listener planned/bound, DCS store connected, all worker tasks spawned, “steady state reached”.
- [x] Add error events for:
  - [x] startup command spawn failures,
  - [x] startup command non-zero exit,
  - [x] startup command timeout/cancel failures.
- [x] Noise guard: do not emit per-iteration logs in tight polling loops (e.g., 20ms startup job polling). Emit only boundary transitions (job started, first output, exited, timed out, cancel failed, poll error).
- [x] Replace `let _ =` log-emit ignores in startup phases and startup subprocess output emission:
  - [x] If emitting a subprocess output line fails, emit a dedicated `Warn`/`Error` event (`runtime.startup.subprocess_log_emit_failed`) including `job_id/job_kind/binary/stream/bytes_len`.
  - [x] If emitting the failure event also fails, *propagate the error* (do not continue silently).
- [x] Add debug breadcrumbs for worker wiring:
  - [x] per worker context created, per worker task started, API listener bound (address), postgres ingest worker started.

#### 1B) `src/process/worker.rs` (queue/state machine/subprocess output)
- [x] Add broad debug instrumentation at meaningful boundaries (debug-only, default level remains Info):
  - [x] worker loop start/stop,
  - [x] inbox receive (`request_received`) and rejection (`busy`) with job context,
  - [x] state transitions (`Idle -> Running`, `Running -> Idle`) with outcome,
  - [x] poll boundaries (`poll_exit`) and timeout/cancel path decisions,
  - [x] job outcome publication (include `job_id/job_kind/binary`, exit_code when available).
- [x] Replace all `let _ = emit_subprocess_line(...)` in runtime paths:
  - [x] On failure, emit `process.worker.output_emit_failed` with correlation keys (job_id/job_kind/binary/stream).
  - [x] If failure-event emission fails, propagate (no silent swallow).
- [x] Add structured events for queue handling edge cases:
  - [x] `TryRecvError::Disconnected` should produce a `Warn` or `Info` event once (rate limited) so operators know the inbox is gone.
- [x] Capture and surface subprocess output reader failures (do not silently drop `drain_output` errors). Emit `process.worker.output_drain_failed` with `job_id/job_kind/binary/stream`.
- [x] Capture and surface cancel/wait failures (do not silently drop `child.wait()` errors after kill). Emit `process.worker.cancel_wait_failed` with `job_id/job_kind/binary`.

#### 1C) `src/api/worker.rs` (no silent `step_once` suppression + request lifecycle)
- [x] Thread a `LogHandle` into `ApiWorkerCtx` and initialize it from `src/runtime/node.rs`.
- [x] In `run`, replace silent swallow with explicit handling:
  - [x] classify `step_once` failures into `fatal` (listener/core invariants broken) vs `non_fatal` (per-connection / per-request I/O / TLS handshake / malformed HTTP).
  - [x] `non_fatal`: emit `Warn` `api.step_once_failed` with error detail + context, then continue serving.
  - [x] `fatal`: emit `Error` and return `Err` so runtime terminates (do not keep serving with a broken listener).
- [x] Add debug breadcrumbs for request lifecycle (only on actual connections/requests; do not spam on 1ms accept timeouts):
  - [x] connection accepted (peer addr, tls mode),
  - [x] request read started/completed,
  - [x] auth decision (allowed/unauthorized/forbidden) with method + *route template* (do not log tokens; do not log request bodies),
  - [x] route selected and response written (status code).
- [x] Replace silent TLS accept drops (`Err(_) => Ok(None)`) with explicit `Warn`/`Error` events:
  - [x] handshake failure (tls required/optional), missing client cert when required.
- [x] Replace `let _ = match status { ... }` (content-length parse) with explicit handling (even if outcome is intentionally ignored, it must be explicit and/or logged when anomalous).
- [x] Add/extend tests to assert key breadcrumbs and previously swallowed errors are now surfaced.
- [x] Add tests that assert privacy redaction invariants: bearer token strings and auth header values never appear in emitted records.

#### 1D) `src/logging/postgres_ingest.rs` (ingest worker; preserve parse invariant)
- [x] Add debug breadcrumbs for:
  - [x] file discovery decisions (summary only: which directory, eligible file counts, skipped-with-reason counts, iteration errors),
  - [x] tailing decisions (summary: which files tailed this cycle, byte limits, total lines emitted, suppressed lines),
  - [x] parser path selection (summary counts per parser per cycle; do not log per-line parser decisions).
- [x] Replace internal `let _ = log.emit(...)` ignores:
  - [x] change helpers like `emit_ingest_step_failure` / `emit_ingest_retry_recovered` to return `Result<(), WorkerError>` and handle failures explicitly (never `let _ = ...`).
- [x] Surface currently-silent filesystem metadata issues in discovery/cleanup:
  - [x] `entry.file_type()` failures should become an issue entry and/or warn event so operators can diagnose permissions/FS glitches.
  - [x] `read_dir` iteration `.filter_map(entry.ok())` should not silently drop errors; log/count them.
- [x] Maintain parse-failure fallback invariant and expand tests to guard it.
- [x] Add explicit non-UTF8 coverage: ensure a non-UTF8 line still produces a record (`parse_failed=true`) with the synthetic `non_utf8_bytes_hex=...` raw payload preserved (no drop).

#### 1E) `src/ha/worker.rs` (action intent/attempt/result + phase/role/lease transitions)
- [x] Thread a `LogHandle` into HA worker context.
- [x] Emit structured debug events for each action:
  - [x] `ha.action.intent` per action from `decide` output (includes `ha_tick`, `action_kind`, `action_id`).
  - [x] `ha.action.dispatch` immediately before dispatch (includes correlation + target details).
  - [x] `ha.action.result` after dispatch (success/failure + error detail + elapsed if available).
  - [x] Replace misleading `action_attempt` with a deterministic `ha_dispatch_seq` (monotonic counter in HA worker/state incremented per dispatch batch) to correlate intent/dispatch/result even though `recent_action_ids` suppresses true retries today.
- [x] Emit info events for major transitions:
  - [x] HA phase transitions (compare `prev.phase` vs `next.phase`),
  - [x] role transitions (Primary/Replica derived from phase + leader identity),
  - [x] lease transitions (acquired/released) when dispatch actions succeed.
- [x] Replace non-deterministic process `job_id` generation for HA-dispatched jobs (currently includes wall-clock):
  - [x] make job ids deterministic from `(scope, member_id, ha_tick, action_id, ha_dispatch_seq)` so logs/tests are stable and correlation is durable.
- [x] Add/adjust tests proving:
  - [x] action intent/dispatch/result logs exist with correct correlation keys,
  - [x] phase/role transitions emit info events,
  - [x] no action failures are silently swallowed (dispatch errors produce logs + worker faulted state).

#### 1F) `src/dcs/worker.rs` (+ small supporting store types if needed)
- [x] Thread a `LogHandle` into DCS worker context.
- [x] Replace `Err(_)` swallowing with `Err(err)` capture and emit structured events with severity decided per error class:
  - [x] `write-local-member` failure (member publish to DCS):
    - `DcsStoreError::Io` => `Warn` (degraded visibility/coordination); keep worker `Faulted`, but do not drop processing.
    - `DcsStoreError::Decode` => `Error` (local state serialization issue; unexpected and operator-visible).
  - [x] `drain-watch-events` failure:
    - `DcsStoreError::Io` => `Warn` (store watch path degraded; will attempt recovery via subsequent loops).
    - other errors should map to `Error` with raw cause (store invariant broken).
  - [x] `refresh/apply` failure:
    - `refresh_from_etcd_watch` should distinguish `Decode`/`MissingValue`/`InvalidKey` and emit `Warn` when a subset of events is malformed but recoverable.
    - hard parse failures that prevent refresh state convergence should emit `Error`.
  - [x] explicitly log and count `DcsKeyParseError::UnknownKey` / unexpected scope keys as `Warn` when discovered via refresh, even though processing continues.
- [x] Emit debug breadcrumbs for watch/drain/apply/write boundaries (including event counts).
  - [x] Emit trust/health transition events:
  - [x] `Info` for recoveries/upgrades (`FullQuorum` and previously unhealthy recovery),
  - [x] `Warn` when healthy->unhealthy transitions on first sight of degraded signals,
  - [x] `Error` if unhealthy persists across a bounded interval without successful refresh (to avoid false alarm spam, include `dcs.poll_interval_ticks` and cause).
- [x] Add/adjust tests that simulate store failures and assert:
  - [x] state transitions (`Running` -> `Faulted` -> `Running`) with one synthetic IO error and one malformed event payload,
  - [x] trust transitions,
  - [x] per-error-type severity mapping in emitted events (`io` vs `decode`/`invalid-key` paths).

#### 1G) `src/pginfo/worker.rs` (connectivity transitions)
- [x] Thread a `LogHandle` into PgInfo worker context.
- [x] Change `Err(_) => Unreachable` mapping to capture `Err(err)` locally:
  - [x] continue publishing Unreachable semantics as today,
  - [x] emit `Warn` on Healthy -> Unreachable transition (include error),
  - [x] emit `Info` on Unreachable -> Healthy transition.
- [x] Add debug breadcrumbs for poll/publish boundaries.
- [x] Add tests asserting these transition logs are emitted without changing publish semantics.

#### 1H) `src/debug_api/worker.rs` + `src/debug_api/view.rs` (coherence check)
- [x] Verify that newly-added runtime logs do not contradict debug narrative:
  - [x] if we add new HA phase/role transition logs, ensure debug summaries still reflect the same state transitions.
- [x] Tighten debug timeline emission now (to keep `/debug/verbose` coherent under increased runtime log volume):
  - [x] gate timeline entries on semantic diffs (not `Version` changes alone): compare summary/signature, not raw `tick`/version fields.
  - [x] specifically for HA: keep `tick` display if useful, but do not use `tick` as a change detector (otherwise timeline churns every publish).
  - [x] add a regression test proving no new timeline entry is emitted on “publish churn without semantic change”.

### Phase 2 — Tests (must be explicit; no optional test skipping)
- [x] Add a shared test helper for asserting structured log “breadcrumbs” from `TestSink` without brittle ordering:
  - [x] add a non-destructive `TestSink::snapshot() -> Result<Vec<LogRecord>, LogError>` (avoid `unwrap`/`expect` even in tests; handle poisoned locks explicitly).
  - [x] add helpers for matching on `event.name` + required attributes (assert presence, not index order).
- [x] For each target module, add focused tests that:
  - [x] validate key debug breadcrumbs exist,
  - [x] validate previously swallowed errors now produce warn/error events,
  - [x] validate correlations (`job_id`, `ha_tick`, `action_id`, `peer_addr`, etc.) are present where applicable.
- [x] `src/logging/postgres_ingest.rs` tests: add explicit coverage that parse-failed lines still emit with original payload retained (expand existing test coverage; include non-UTF8 path).

### Phase 3 — Static Verification: “No Silent Swallow” (code-level proof)
- [x] Add a repo-local verification script/target that is intentionally *narrow* and wired into the real gate (`make lint`):
  - [x] script scopes only to the runtime hot paths in this task (not the entire repo) to avoid false positives pushing “lint ignores”.
  - [x] check patterns: `let _ =`, `.ok()`, `filter_map(Result::ok)`, `Err(_) =>`, `Ok(None)` in TLS/request paths.
  - [x] require explicit handling or an allowlist entry (rare; documented).
  - [x] integrate as `make lint.no_silent_errors` and call it from `make lint`.
- [x] Run targeted searches as part of final verification:
  - [x] `rg -n \"^\\s*let\\s+_\\s*=\" src/api/worker.rs src/process/worker.rs src/runtime/node.rs src/ha/worker.rs src/dcs/worker.rs src/pginfo/worker.rs src/logging/postgres_ingest.rs`
  - [x] `rg -n \"\\.ok\\(\\)\"` in the same set and confirm each is justified + logged.

### Phase 4 — Docs
- [x] Update operator-facing docs to describe:
  - [x] new event taxonomy (`event.name`, correlation fields),
  - [x] how to correlate HA decisions ↔ process jobs ↔ subprocess output (`ha_tick`, `action_id`, `job_id`, `job_kind`, `binary`),
  - [x] examples for troubleshooting using these logs.
- [x] Update API docs (`/debug/verbose?since=...`) if any debug behavior changes.
- [x] Remove or update any stale doc references discovered during edits (project is greenfield; do not keep legacy/dead paths).

### Phase 5 — Full Gate Verification (must be 100% green)
- [x] `make check`
- [x] `make test`
- [x] `make test-long`
- [x] `make lint`
- [x] `make docs-lint` (already runs as part of `make lint`, but run explicitly after docs edits)
- [x] `make docs-hygiene` (if docs were modified)

NOW EXECUTE
