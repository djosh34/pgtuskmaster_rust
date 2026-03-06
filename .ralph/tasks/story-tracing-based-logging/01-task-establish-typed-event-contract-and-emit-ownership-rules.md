---
## Task: Establish typed event contract and emit ownership rules <status>not_started</status> <passes>false</passes>

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
- There are currently 65 `emit_event` call sites plus `emit` / `emit_record` paths across the runtime code.
- The exact current emit inventory that this story must govern is:
- `src/logging/mod.rs`: `LogHandle::emit_event`, `LogHandle::emit`, `LogHandle::emit_record`.
- `src/runtime/node.rs`: `run_node_from_config`, `plan_startup_with_probe`, `execute_startup`, `emit_startup_phase`, `run_startup_job`, `emit_startup_subprocess_line`.
- `src/process/worker.rs`: `run`, `step_once`, `start_job`, `tick_active_job`, `emit_process_output_emit_failed`, `emit_subprocess_line`.
- `src/api/worker.rs`: `run`, `step_once`, `emit_api_auth_decision`, `accept_connection`.
- `src/api/events.rs`: `ingest_wal_event`.
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
- [ ] `src/logging/mod.rs`: introduce the typed event contract used by application logs, with no requirement for call sites to hand-build `BTreeMap<String, Value>` for app events.
- [ ] `src/logging/mod.rs` or a new sibling module: add typed field encoding utilities and typed raw-log-record builders for subprocess/postgres line ingestion paths that still need record-level emission.
- [ ] `src/logging/mod.rs` tests: add coverage for typed event encoding, stable event header fields, and raw-record encoding without `unwrap`, `expect`, or panic paths.
- [ ] `.ralph/tasks/story-tracing-based-logging/`: the tracing-first and OTEL-first task framing is fully replaced by this typed-event-first migration plan.
- [ ] Emit ownership policy is written into this story and is explicit about all three categories: orchestration boundary events, operation-owned events, and raw external log records.
- [ ] Emit ownership policy is explicit that pure helpers and planners do not emit unless they are themselves the operational boundary.
- [ ] The story text contains the full current emit inventory for `src/runtime/node.rs`, `src/process/worker.rs`, `src/api/worker.rs`, `src/api/events.rs`, `src/ha/worker.rs`, `src/ha/events.rs`, `src/dcs/worker.rs`, `src/pginfo/worker.rs`, and `src/logging/postgres_ingest.rs`.
- [ ] Migration guidance explicitly states that `tracing` remains the planned backend direction, but follows the typed event contract rather than defining it.
- [ ] Migration guidance explicitly states that file sinks and OTEL export are implemented through the post-migration `tracing` backend layer, not by bypassing typed events.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] `make test-long` — passes cleanly
</acceptance_criteria>
