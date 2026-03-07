## Task: Rework logging backends, exporters, tests, and docs after typed event migration <status>done</status> <passes>true</passes>

<description>
**Goal:** Revisit backend wiring, exporters, sink abstractions, and documentation only after the typed event contract is in place across the codebase. The higher order goal is to prevent backend work from distorting the event model, and to make any future `tracing` or OTEL integration consume the typed event contract instead of becoming a substitute for it.

**Scope:**
- `src/logging/mod.rs`
- `Cargo.toml` and any logging backend dependencies that may be introduced or removed
- runtime config and docs for logging backends or exporters
- tests and docs that describe logging behavior
- story/task docs in `.ralph/tasks/story-tracing-based-logging/`
- evaluate whether the current custom sink bootstrap remains justified once typed events exist
- treat Tasks 01-03 typed events and typed raw-record builders as fixed semantic inputs, then adopt `tracing` as the backend or adapter layer under that contract rather than as a replacement for event modeling at call sites
- wire file sinks and OTEL export through that post-migration `tracing` backend layer
- keep domain call sites on typed events only; no domain module should bypass the typed event layer by constructing `tracing` fields directly for normal application events

**Context from research:**
- The previous story incorrectly assumed a tracing-first migration should come before event modeling.
- The repo currently uses a custom `LogHandle` / `LogSink` pipeline, and `Cargo.toml` does not currently declare direct app-side `tracing` usage.
- File sink support and OTEL export are real follow-up concerns, but they are secondary to eliminating attr-map event construction from domain code.
- This task is where old “replace bespoke logging with tracing” and “add OTEL export” work is merged into the new strategy.
- The backend answer for this story is yes: use `tracing`, but only after typed event migration has made the event model explicit.

**Expected outcome:**
- Backend and exporter choices are made against a stable typed event contract.
- Docs describe the typed event model first, backend/export options second.
- The story no longer contains stale tracing-first or OTEL-first assumptions.
- The final logging architecture uses typed events as the semantic layer and `tracing` as the backend integration layer for stderr JSONL and file sinks, while leaving OTEL explicitly deferred instead of half-configured.
- Execution decision for this task: OTEL remains deferred until a future task can add a real operator-facing config, docs, and testable export path without speculation.

</description>

<acceptance_criteria>
- [x] `src/logging/mod.rs`: rework the custom sink/bootstrap layer so typed events feed a `tracing` backend or adapter layer, with the typed event contract remaining the source of truth.
- [x] `Cargo.toml`: add the justified `tracing` backend dependencies for the post-migration design, and do not make domain call sites depend on `tracing` field construction APIs.
- [x] Runtime logging config and docs: describe stderr JSONL, file sinks, and any OTEL export as backend choices layered under the typed event contract.
- [x] File sink support uses the post-migration `tracing` backend layer rather than a parallel bespoke path.
- [x] OTEL export, if implemented here, exports the already-modeled typed events and raw external log records through the `tracing` backend layer; it does not force the product to invent trace or span semantics.
- [x] Documentation explains exactly how typed events map into the `tracing` backend without reintroducing stringly call sites.
- [x] Test helpers and assertions across logging-related modules are updated to the final post-migration backend shape without regressing typed-event assertions.
- [x] `.ralph/tasks/story-tracing-based-logging/`: remove stale tracing-first wording and keep the final task set internally consistent with the implemented backend strategy.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] `make test-long` — passes cleanly
</acceptance_criteria>

<implementation_plan>

1. Lock the contract boundary before touching the backend.
- Keep `AppEvent`, `AppEventHeader`, `StructuredFields`, `RawRecordBuilder`, and `LogRecord` as the semantic/source-of-truth layer that domain code talks to.
- Do not introduce `tracing` macros, spans, field sets, or subscriber-specific types into domain modules. `runtime`, `ha`, `dcs`, `pginfo`, `api`, `process`, and `postgres_ingest` should continue to call `LogHandle` with typed events or typed raw-record builders only.
- Preserve the existing operator-facing JSON record shape unless execution reveals a deliberate schema correction is necessary and the docs/tests are updated in the same patch.

2. Replace the bespoke backend shape carefully: preserve the fallible contract first, then migrate internals under it.
- Do not start by deleting the current `LogSink` / file-writer / fanout pieces. The current code has an explicit `Result<(), LogError>` emission contract and "some sinks failed" diagnostics that generic tracing subscriber plumbing would erase if we are careless.
- Introduce a dedicated tracing bootstrap layer inside `src/logging/` that can route `LogRecord`-backed emissions synchronously while still surfacing per-emission failures back to `LogHandle`.
- Keep `LogHandle` as the only app-facing handle, but change its internals so `emit_app_event`, `emit_raw_record`, and `emit_record` hand the already-modeled `LogRecord` into a tracing-backed router/adapter rather than directly owning all routing logic themselves.
- Remove old sink abstractions only after the tracing-backed path fully preserves the current stderr/file behavior, JSONL shape, and failure reporting semantics. If a narrow writer utility remains useful under the tracing layer, keep it instead of deleting it for purity.

3. Choose the minimum dependency set that enables the backend shift cleanly.
- Add `tracing` and `tracing-subscriber` as direct dependencies.
- Do not assume `tracing-appender` belongs here. Prefer the current explicit writer/bootstrap path unless execution proves an appender can preserve the same error visibility and deterministic JSONL behavior.
- Treat OTEL as deferred by default for this task unless a concrete, fully tested export path naturally fits the new backend without speculative runtime config. The absence of any current OTEL config/docs surface is a real constraint, so stderr/file migration must not be blocked on OTEL.
- Remove old custom-backend code or stale comments only when the tracing-backed replacement is verified and fully covers the operator-facing behavior.

4. Design the tracing bridge around `LogRecord` emission and synchronous backend diagnostics, not ad hoc `tracing` fields.
- Introduce a focused internal adapter that emits one tracing-backed backend action per `LogRecord`, but do not force the design through stock `tracing::event!` + fmt subscriber paths if that would swallow sink I/O failures.
- Preserve deterministic JSONL serialization in one owned formatting path. `tracing` should provide backend routing/composition, not take ownership of the log schema.
- Keep severity mapping driven from the existing `SeverityText` and `severity_number`, with a single translation point into tracing levels.
- Ensure raw external records and typed app events both follow the same backend path after they have become `LogRecord`; the semantic split stays above the backend, not inside it.
- Avoid stringly reconstructing app-event headers from tracing metadata or parsing debug strings back into records. The tracing side must carry the modeled payload in a deliberate adapter structure.

5. Preserve the current logging config shape where possible, and only extend it if the backend requires a real operator-facing knob.
- Keep `[logging.sinks.stderr]` and `[logging.sinks.file]` as the operator-facing runtime controls for this task unless execution proves they are fundamentally incompatible with tracing-backed bootstrap.
- Keep existing validation invariants in `src/config/parser.rs` for file-path ownership and absolute-path requirements.
- Update `src/config/schema.rs`, `src/config/defaults.rs`, and parser tests only if the tracing backend needs additional explicit config. Do not add speculative OTEL config without a working implementation and full docs/tests.
- Ensure runtime bootstrap in `src/runtime/node.rs` continues to receive a `LogHandle` and does not become tracing-aware.

6. Rebuild logging tests around the backend shape without weakening typed-event verification.
- Update `src/logging/mod.rs` tests so they verify:
- typed app events still encode the same headers and structured fields
- raw-record emission still preserves parsed/fallback external log content
- stderr/file backend bootstrap still honors enablement and file-mode behavior
- backend failure reporting remains explicit rather than swallowed
- Keep or adapt the current in-memory capture helpers where they remain the clearest way to assert on `LogRecord` semantics; add tracing-aware capture only at the backend boundary that actually changes.
- Update logging-related tests in touched modules only as needed to accommodate the backend refactor. Domain tests should remain focused on typed-event semantics, not on backend internals.

7. Update docs so they describe semantics first and backend choice second.
- Rewrite `docs/src/operator/observability.md` so it explains that typed events/raw records are modeled first and then routed through the tracing backend to stderr JSONL and optional file sinks.
- Update `docs/src/operator/configuration.md` to describe file sinks as tracing-backed output destinations under the typed event contract, not bespoke sink plumbing.
- Search `.ralph/tasks/story-tracing-based-logging/` for stale tracing-first or OTEL-first language and bring Task 04 plus any still-stale siblings into alignment with the implemented plan.
- Remove stale wording that implies operators or domain code should reason in terms of ad hoc attribute maps.

8. Make the OTEL decision explicit up front and bias toward deferral unless execution proves otherwise.
- Default execution decision: defer OTEL in this task unless a clean export path with real config, tests, and docs clearly falls out of the tracing backend migration without speculative design.
- Do not half-add OTEL dependencies or placeholder config. If OTEL stays deferred, update the docs/task text to say so clearly and leave the tracing-backed stderr/file path in a shape that a later OTEL layer can consume without changing domain code.
- If execution unexpectedly uncovers a clean, fully testable OTEL layer, route it from the same tracing backend layer and export the already-modeled `LogRecord` payloads without inventing span semantics.

9. Execute in a low-risk sequence.
- First refactor `src/logging/mod.rs` so the tracing backend exists behind `LogHandle` while preserving the external typed-event API.
- Then update config/default/parser/doc surfaces that describe bootstrap behavior.
- Then update logging tests and any cross-module helpers affected by the backend swap.
- Then update story/task docs to reflect the final implemented state, not the pre-migration assumption set.
- Finish with the required full verification gates in the exact required set:
- `make check`
- `make test`
- `make test-long`
- `make lint`

10. Completion rules for the eventual execution pass.
- Tick off acceptance boxes only when the code, tests, and docs are actually updated.
- Set `<passes>true</passes>` only after all required make targets pass.
- Include `.ralph` bookkeeping changes in the completion commit.
- Run `/bin/bash .ralph/task_switch.sh`, commit with the required `task finished ...` prefix, push, and stop immediately after task completion.

NOW EXECUTE

</implementation_plan>
