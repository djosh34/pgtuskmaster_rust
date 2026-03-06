## Task: Rework logging backends, exporters, tests, and docs after typed event migration <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Revisit backend wiring, exporters, sink abstractions, and documentation only after the typed event contract is in place across the codebase. The higher order goal is to prevent backend work from distorting the event model, and to make any future `tracing` or OTEL integration consume the typed event contract instead of becoming a substitute for it.

**Scope:**
- `src/logging/mod.rs`
- `Cargo.toml` and any logging backend dependencies that may be introduced or removed
- runtime config and docs for logging backends or exporters
- tests and docs that describe logging behavior
- story/task docs in `.ralph/tasks/story-tracing-based-logging/`
- evaluate whether the current custom sink bootstrap remains justified once typed events exist
- adopt `tracing` as the backend or adapter layer under the typed event contract rather than as a replacement for event modeling at call sites
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
- The final logging architecture uses typed events as the semantic layer and `tracing` as the backend integration layer for stderr JSONL, file sinks, and OTEL export.

</description>

<acceptance_criteria>
- [ ] `src/logging/mod.rs`: rework the custom sink/bootstrap layer so typed events feed a `tracing` backend or adapter layer, with the typed event contract remaining the source of truth.
- [ ] `Cargo.toml`: add the justified `tracing` backend dependencies for the post-migration design, and do not make domain call sites depend on `tracing` field construction APIs.
- [ ] Runtime logging config and docs: describe stderr JSONL, file sinks, and any OTEL export as backend choices layered under the typed event contract.
- [ ] File sink support uses the post-migration `tracing` backend layer rather than a parallel bespoke path.
- [ ] OTEL export, if implemented here, exports the already-modeled typed events and raw external log records through the `tracing` backend layer; it does not force the product to invent trace or span semantics.
- [ ] Documentation explains exactly how typed events map into the `tracing` backend without reintroducing stringly call sites.
- [ ] Test helpers and assertions across logging-related modules are updated to the final post-migration backend shape without regressing typed-event assertions.
- [ ] `.ralph/tasks/story-tracing-based-logging/`: remove stale tracing-first wording and keep the final task set internally consistent with the implemented backend strategy.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] `make test-long` — passes cleanly
</acceptance_criteria>
