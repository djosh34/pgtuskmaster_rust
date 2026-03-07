## Task: Establish typed event contract and emit ownership rules <status>completed</status> <passes>true</passes>

<description>
**Goal:** Replace the current partially typed logging contract with a fully typed application event contract that owns event identity, severity, result, and structured fields without requiring call sites to assemble `BTreeMap<String, serde_json::Value>`. The higher order goal is to separate event semantics from backend choice so later decisions about `tracing`, OTEL export, file sinks, or keeping the current sink stack are downstream implementation choices rather than the source of application event truth.

**Scope:**
- `src/logging/mod.rs` and any new `src/logging/*` modules needed for typed event traits, field encoding, typed raw-log record builders, and test helpers.
- Story/task files in `.ralph/tasks/story-tracing-based-logging/` must be updated so the tracing-first plan is replaced by this typed-event-first plan.
- Define and pin one explicit ownership policy for where events are emitted. This policy is a non-optional rule for the whole story:
- boundary or orchestration events stay in orchestration functions only when they describe orchestration decisions, phase changes, or composition boundaries that only the outer function can see.
- operation result events must be emitted by the function or type that actually performs and owns the side effect, success, failure, timeout, rejection, or recovery.
- pure helpers, planners, serializers, builders, and mappers should not emit events unless they are themselves the operational boundary.
- raw external log line ingestion stays as a separate typed record path rather than pretending every child or postgres line is an app event.
- Pin the backend direction as part of the story:
- yes, the intended backend direction is still `tracing`, but only as a backend or adapter under the typed event contract.
- no domain call site should directly depend on `tracing` field-construction APIs, `event!`, or ad hoc macro field sets for normal application events.
- Capture the full current emit inventory before migration and keep it in this story so no emit site is left as an implicit follow-up.

**Context from research:**
- Current app logging is still backed by `LogRecord.attributes: BTreeMap<String, Value>` in `src/logging/mod.rs`, and `LogHandle::emit_event` only types `event.name`, `event.domain`, and `event.result`.
- The audited current emit inventory in domain code is 54 direct `.emit_event(...)` call sites, 3 direct `.emit_record(...)` raw-record call sites, 1 direct `.emit(...)` plain-record call site, plus the internal helper wrapper in `src/ha/events.rs`.
- The exact current emit inventory that this story must govern is:
- `src/logging/mod.rs`: `LogHandle::emit_event`, `LogHandle::emit`, `LogHandle::emit_record`.
- `src/runtime/node.rs`: `run_node_from_config`, `plan_startup_with_probe`, `execute_startup`, `emit_startup_phase`, `run_startup_job`, `emit_startup_subprocess_line`.
- `src/process/worker.rs`: `run`, `step_once`, `start_job`, `tick_active_job`, `emit_process_output_emit_failed`, `emit_subprocess_line`.
- `src/api/worker.rs`: `run`, `step_once`, `emit_api_auth_decision`, `accept_connection`.
- `src/ha/worker.rs`: `step_once`.
- `src/ha/events.rs`: `emit_ha_action_intent`, `emit_ha_decision_selected`, `emit_ha_effect_plan_selected`, `emit_ha_action_dispatch`, `emit_ha_action_result_ok`, `emit_ha_action_result_skipped`, `emit_ha_action_result_failed`, `emit_ha_lease_transition`, internal helper `emit_event`.
- `src/dcs/worker.rs`: `step_once`.
- `src/pginfo/worker.rs`: `step_once`.
- `src/logging/postgres_ingest.rs`: `run`, `emit_ingest_step_failure`, `emit_ingest_retry_recovered`, `step_once`, `emit_postgres_line`.
- Current tests in `src/process/worker.rs`, `src/api/worker.rs`, `src/ha/events.rs`, `src/dcs/worker.rs`, `src/pginfo/worker.rs`, `src/runtime/node.rs`, and `src/logging/postgres_ingest.rs` assert through stringly typed `record.attributes.get("event.name")` patterns and must be migrated with the contract.
- `Cargo.toml` does not currently declare direct `tracing` usage for the app logging path, so the previous tracing-first story is stale and must be superseded rather than extended.

**Expected outcome:**
- The codebase has one typed application event contract and one explicit placement policy for event emission.
- The story becomes the authoritative migration plan for every current emit site.
- Later tasks can migrate domains independently without reopening architecture questions or inventing inconsistent placement rules.
- There is no ambiguity about the backend direction: typed events first, `tracing` second as the backend integration layer.

</description>

<acceptance_criteria>
- [x] `src/logging/mod.rs`: introduce the typed event contract used by application logs, with no requirement for call sites to hand-build `BTreeMap<String, Value>` for app events.
- [x] `src/logging/mod.rs` or a new sibling module: add typed field encoding utilities and typed raw-log-record builders for subprocess/postgres line ingestion paths that still need record-level emission.
- [x] `src/logging/mod.rs` tests: add coverage for typed event encoding, stable event header fields, and raw-record encoding without `unwrap`, `expect`, or panic paths.
- [x] `.ralph/tasks/story-tracing-based-logging/`: the tracing-first and OTEL-first task framing is fully replaced by this typed-event-first migration plan.
- [x] Emit ownership policy is written into this story and is explicit about all three categories: orchestration boundary events, operation-owned events, and raw external log records.
- [x] Emit ownership policy is explicit that pure helpers and planners do not emit unless they are themselves the operational boundary.
- [x] The story text contains the full current emit inventory for `src/runtime/node.rs`, `src/process/worker.rs`, `src/api/worker.rs`, `src/ha/worker.rs`, `src/ha/events.rs`, `src/dcs/worker.rs`, `src/pginfo/worker.rs`, and `src/logging/postgres_ingest.rs`.
- [x] Migration guidance explicitly states that `tracing` remains the planned backend direction, but follows the typed event contract rather than defining it.
- [x] Migration guidance explicitly states that file sinks and OTEL export are implemented through the post-migration `tracing` backend layer, not by bypassing typed events.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] `make test-long` — passes cleanly
</acceptance_criteria>

<implementation_plan>

## Detailed plan

1. Correct the story to match the current repository before touching code.
- Update this task plus Tasks 02-04 to match the already-audited repository state instead of doing another discovery pass during execution.
- Remove the stale `src/api/events.rs` / `ingest_wal_event` references from Tasks 01-02 because that file is absent in the current repository.
- Replace the inaccurate “65 emit_event call sites” language with the audited inventory: 54 direct domain `.emit_event(...)` sites, 3 direct `.emit_record(...)` sites, 1 direct `.emit(...)` site, and the `src/ha/events.rs` wrapper helper.
- Keep the story typed-event-first throughout: domain code owns semantic events, raw external lines stay on a separate typed raw-record path, and `tracing` remains a backend follow-up rather than the source of event truth.
- Expand the ownership section so it explicitly states which events stay in orchestration functions, which must move to side-effect owners, and which are never app events at all because they are external raw log records.

2. Introduce a typed application-event contract inside `src/logging/` without forcing domain code to build maps.
- Split `src/logging/mod.rs` into focused sibling modules if that reduces complexity. The expected minimum slices are:
- typed app-event contract and event header types
- field encoding / typed value utilities
- raw record builders for subprocess and postgres lines
- test helpers / typed record decoders
- Replace the current `EventMeta` + `BTreeMap<String, Value>` call-site contract with a typed event API that owns:
- event identity (`name`, `domain`)
- event result / outcome
- severity
- human message
- structured fields
- Keep the public logging entry point domain-friendly. A target end state is a call shape like `log.emit_app_event(origin, event)` or equivalent, where the event type itself defines its header fields and knows how to encode its structured payload.
- Centralize all `serde_json::Value` construction in logging internals. Domain code may pass typed scalars / enums / identifiers into event structs, but it must not assemble `BTreeMap<String, Value>` for normal app events.
- Do not introduce any new compatibility shim or legacy parallel API. If `EventMeta` / `emit_event` must remain temporarily so Tasks 02-03 can land in smaller steps, make them a thin adapter over the typed event encoder only, forbid any new call sites from using them, and treat their deletion as mandatory story completion work.

3. Define the typed field model and event encoding boundary.
- Add a small typed field representation in logging, for example a dedicated field-builder / encoder that accepts booleans, signed and unsigned integers, strings, optional values, and already-typed enums rendered via `Serialize`.
- Ensure the encoder is infallible at call sites and that any serialization failure is surfaced from logging internals as `LogError`, never through `unwrap`, `expect`, or silent drops.
- Make the stable event header encoding explicit and uniform:
- `event.name`
- `event.domain`
- `event.result`
- any new header keys required by the typed contract
- Keep `LogRecord` as the sink-facing representation unless execution shows a cleaner replacement is needed, but make app-event-to-record conversion a single internal code path instead of repeated attr-map assembly.

4. Add a separate typed raw-record builder path for external log lines.
- Introduce builders / helper types for child stdout / stderr lines and postgres-ingest lines so record-level emission remains structured without pretending those records are app events.
- Move the shared raw-record assembly rules into logging internals:
- source identity (`producer`, `transport`, `parser`, `origin`)
- message body
- optional parsed metadata for postgres JSON / plain lines
- optional fallback parse-failure data
- Preserve `emit_record` only for true raw-record use cases and tests; normal application events should stop using it as the main abstraction.

5. Add logging-level tests and typed test helpers before migrating domain modules.
- Add focused unit tests in `src/logging/mod.rs` or new logging test modules for:
- stable typed event header encoding
- event field encoding for representative scalar and optional values
- raw-record builder encoding for subprocess and postgres paths
- severity filtering on typed events
- sink emission behavior for typed app events and raw records
- Add typed decode helpers for tests so later domain tasks can assert on typed event headers / fields instead of open-coded `record.attributes.get("event.name")`.
- Keep the helpers reusable by Tasks 02 and 03 so migration tests do not reinvent per-file decoding logic.

6. Rewrite the downstream story tasks so they are internally consistent with the contract introduced here.
- Task 02 should only mention actual runtime / process / API files that exist, remove `src/api/events.rs`, and classify every current emit site as orchestration boundary, operation-owned result, or raw-record emission.
- Task 03 should explicitly consume the shared typed event contract and raw-record builders from Task 01 instead of referring to ad hoc map construction.
- Task 04 should be rewritten as backend follow-up work only: adopt `tracing` under the typed event contract, wire file sinks / exporters there, and keep domain call sites free of direct `tracing` field construction.
- Where the current story text still sounds tracing-first or OTEL-first, replace that wording entirely rather than layering new guidance on top of stale guidance.

7. Define the execution order for the actual implementation pass.
- First update the story markdown files exactly as already audited here so the task inventory and migration ownership rules are accurate before code work starts drifting.
- Then implement the core logging contract, field encoder, raw-record builders, and test helpers in `src/logging/`.
- Then update logging-specific tests to validate the contract in isolation.
- Only after the shared contract is stable should later tasks migrate domain emit sites; Task 01 should avoid half-migrating domains unless a tiny targeted migration is required to keep the tree compiling.

8. Verification expectations for the execution pass.
- Run the full required suite with no skips:
- `make check`
- `make test`
- `make test-long`
- `make lint`
- If any existing docs outside the story task files describe the old tracing-first architecture or the old attr-map event contract, update or remove them in the same execution pass.
- Only after all checks pass should the task set `<passes>true</passes>`, run `.ralph/task_switch.sh`, commit, and push.

## Repository corrections discovered during planning

- The current story text is stale in at least one concrete place: `src/api/events.rs` / `ingest_wal_event` are referenced in Tasks 01 and 02, but that file is not present in the repository.
- The current story text is also stale in its numeric inventory summary: the audited codebase has 54 direct domain `.emit_event(...)` call sites, 3 direct `.emit_record(...)` call sites, 1 direct `.emit(...)` call site, and the `src/ha/events.rs` helper wrapper rather than the earlier “65 emit_event call sites” claim.
- The current logging contract in `src/logging/mod.rs` types only `event.name`, `event.domain`, and `event.result`; severity, message, origin, and structured payload ownership are still spread between `emit_event`, `emit`, and direct `emit_record` call sites.
- The current domain tests still rely on string-key assertions against `record.attributes`, so shared typed decode helpers are part of the contract work here, not an optional follow-up.

NOW EXECUTE

</implementation_plan>
