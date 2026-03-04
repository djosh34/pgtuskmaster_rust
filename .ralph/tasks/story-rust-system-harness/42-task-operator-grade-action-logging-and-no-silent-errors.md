---
## Task: Enforce Operator-Grade Action Logging And No Silent Error Swallowing <status>not_started</status> <passes>false</passes>

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
- [ ] Global policy: with debug enabled, logs provide near step-by-step runtime traceability across workers and orchestration paths (not only HA actions), including clear module/function origin and correlation attributes.
- [ ] `src/runtime/node.rs`: add pervasive debug breadcrumbs for startup planning, startup mode selection, startup job boundaries, worker wiring/startup, and key branch outcomes.
- [ ] `src/ha/worker.rs`: add structured debug events for each dispatched HA action attempt and result (success/failure), with action id/type, tick, and correlation attributes.
- [ ] `src/ha/worker.rs`: emit info events for major HA phase transitions and operator-important role/lease transitions.
- [ ] `src/runtime/node.rs`: emit info lifecycle events for startup mode selection and startup stage boundaries; emit error events for startup failures/timeouts; do not silently ignore startup subprocess log-emit failures.
- [ ] `src/process/worker.rs`: keep per-line subprocess ingestion, and replace ignored emit failures with structured warn/error events using existing logging interface.
- [ ] `src/process/worker.rs`: add broad debug instrumentation for queue handling, state transitions (`Idle`/`Running`), poll/timeout/cancel paths, and job outcome publication with job_id/job_kind/binary attributes.
- [ ] `src/logging/postgres_ingest.rs`: preserve parse-failure fallback semantics (no parse-loss regression), and surface ingest-step/cleanup failures as structured warn/error events instead of silent ignore.
- [ ] `src/logging/postgres_ingest.rs`: add debug breadcrumbs for file discovery/tailing path decisions and parser path selection (json/plain/raw) at meaningful boundaries.
- [ ] `src/api/worker.rs`: remove silent ignore of `step_once` errors in run loop; emit structured debug breadcrumbs for request lifecycle (accept/auth/route/respond) and warn/error events with connection/request context where available.
- [ ] `src/dcs/worker.rs`: add debug instrumentation for watch/drain/apply/write phases, plus info for trust/health transitions and warn/error on degraded DCS health transitions where actionable.
- [ ] `src/pginfo/worker.rs`: add debug instrumentation for poll/publish flow and warn/info visibility for connectivity transitions (healthy <-> unreachable) while keeping publish semantics.
- [ ] `src/debug_api/worker.rs` and `src/debug_api/view.rs`: ensure debug timeline/events remain coherent with newly introduced logs (no contradictory severity/state narratives).
- [ ] `src/logging/mod.rs`: keep existing logging interfaces as the only runtime logging path; do not add alternate logger frameworks.
- [ ] `src/logging/postgres_ingest.rs` tests: explicitly verify parse-failed lines still emit with original payload retained.
- [ ] `src/api/worker.rs`, `src/process/worker.rs`, `src/runtime/node.rs`, `src/ha/worker.rs`, `src/dcs/worker.rs`, `src/pginfo/worker.rs` tests: add/adjust coverage proving key debug breadcrumbs exist and previously swallowed errors are now surfaced as structured events.
- [ ] Verification task: static/code-level check demonstrates no runtime-path swallowed errors remain (no silent `let _ = ...` for operational errors in worker/runtime loops; remaining intentional best-effort paths must emit warn/info with rationale in code).
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
