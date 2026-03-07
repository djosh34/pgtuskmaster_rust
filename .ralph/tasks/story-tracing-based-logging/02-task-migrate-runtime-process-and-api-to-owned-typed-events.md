## Task: Migrate runtime, process, and api logging to owned typed events <status>completed</status> <passes>true</passes>

<description>
**Goal:** Convert the most orchestration-heavy logging paths from ad hoc attr maps into typed domain events, and move event ownership to the code that actually owns the semantics of the action or failure. The higher order goal is to stop outer orchestration functions from being the default place where every event is assembled, while still preserving true orchestration boundary events where they add operator value.

**Scope:**
- `src/runtime/node.rs`
- `src/process/worker.rs`
- `src/api/worker.rs`
- related tests in those same files
- any shared typed event modules introduced by Task 01 that these domains must consume
- convert all current app-event emission in these files away from hand-built attr maps
- preserve raw child-process line capture, but route it through typed raw-log builders rather than manual record construction at arbitrary call sites
- apply the story-wide ownership rule explicitly in these domains:
- outer orchestration functions keep only orchestration-boundary events.
- functions that perform side effects own execution result events.
- pure helpers and planners do not emit unless they are themselves the operational boundary.
- explicitly classify each current event in these domains as one of:
- orchestration boundary event that stays in the outer function
- operation-owned event that moves into the function that actually performs the work
- raw line record that stays outside the app-event path

**Context from research:**
- These files currently contain the densest concentration of outer-function event assembly.
- `src/runtime/node.rs` currently emits from `run_node_from_config`, `plan_startup_with_probe`, `execute_startup`, `emit_startup_phase`, `run_startup_job`, and `emit_startup_subprocess_line`.
- `src/process/worker.rs` currently emits from `run`, `step_once`, `start_job`, `tick_active_job`, `emit_process_output_emit_failed`, and `emit_subprocess_line`.
- `src/api/worker.rs` currently emits from `run`, `step_once`, `emit_api_auth_decision`, and `accept_connection`.
- The exact pattern the migration must evaluate is the current outer orchestration style of “emit intent in outer function, call lower function, emit result in outer function” versus moving execution-result events into `do_x` / `do_y` ownership boundaries.

**Expected outcome:**
- No app event in these domains requires local `BTreeMap<String, Value>` assembly.
- Runtime startup, process job handling, and API request lifecycle events have a deliberate ownership model rather than accidental placement.
- Tests for these areas assert typed events or typed decoders instead of stringly typed map lookups.
- These domains emit typed events through the shared contract and later feed the `tracing` backend layer without domain code using `tracing` APIs directly.

</description>

<acceptance_criteria>
- [x] `src/runtime/node.rs`: migrate `run_node_from_config` startup-entered logging to typed runtime events and keep only true runtime-entry boundary events there.
- [x] `src/runtime/node.rs`: migrate `plan_startup_with_probe` events for data-dir inspection, DCS probe, and startup-mode selection to typed runtime events and explicitly decide which of those are planner-owned versus lower-level operation-owned.
- [x] `src/runtime/node.rs`: migrate `execute_startup` events for action-planned, action-started, action-completed, and action-failed; document whether action result events stay in `execute_startup` or move into lower action executors.
- [x] `src/runtime/node.rs`: migrate `emit_startup_phase`, `run_startup_job`, and `emit_startup_subprocess_line` to typed runtime/raw-log builders so startup subprocess output is not hand-built as free-form maps or records.
- [x] `src/process/worker.rs`: migrate `run` and `step_once` worker lifecycle events to typed process events.
- [x] `src/process/worker.rs`: migrate `start_job` events for busy reject, fencing noop, preflight failure, build-command failure, spawn failure, and job-started to typed process events owned by the operation that determines those outcomes.
- [x] `src/process/worker.rs`: migrate `tick_active_job` events for output-drain failure, timeout, exit success, exit failure, and poll failure to typed process events, and route subprocess line output through typed raw-log builders.
- [x] `src/process/worker.rs`: migrate `emit_process_output_emit_failed` and `emit_subprocess_line` so even fallback/failure logging paths use typed helpers rather than ad hoc attributes.
- [x] `src/api/worker.rs`: migrate `run` fatal or non-fatal step failure events to typed api events with explicit ownership.
- [x] `src/api/worker.rs`: migrate `step_once` connection accepted, request parse failure, response sent, and related request lifecycle events to typed api events without local map assembly.
- [x] `src/api/worker.rs`: migrate `emit_api_auth_decision` and `accept_connection` so auth and TLS outcomes are typed api events and the final placement is explicit about outer request orchestration versus TLS/auth operation ownership.
- [x] `src/runtime/node.rs`, `src/process/worker.rs`, and `src/api/worker.rs` tests: replace direct `record.attributes.get("event.name")` style assertions with typed event assertions or typed decoding helpers.
- [x] Domain code in these files does not emit normal application events via direct `tracing` macros or direct `BTreeMap<String, Value>` assembly; it emits typed events through the shared contract.
- [x] If new emit sites are introduced while refactoring these files, the task updates the story inventory instead of leaving them implicit.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] `make test-long` — passes cleanly
</acceptance_criteria>

<implementation_plan>

## Detailed plan

1. Lock the shared migration shape before touching domain code.
- Re-read the Task 01 logging contract and keep this task strictly on top of that existing surface: `AppEvent`, `AppEventHeader`, `StructuredFields`, `decode_app_event`, `SubprocessLineRecord`, and `RawRecordBuilder`.
- Do not add any new map-building escape hatch. If a domain needs extra structured payload, add typed fields or small typed event structs/builders rather than reintroducing local `BTreeMap<String, Value>` assembly.
- Decide early whether each domain gets a small sibling event module or private event structs in the existing file. The working default for execution is:
- `src/runtime/node.rs` gets a small runtime-startup typed event helper section or sibling module.
- `src/process/worker.rs` gets a small process-worker typed event helper section or sibling module.
- `src/api/worker.rs` gets a small api-worker typed event helper section or sibling module.
- Keep the public logging backend unchanged for this task. The only contract change at domain call sites should be switching from `emit_event(..., EventMeta, attrs)` to `emit_app_event(origin, typed_event)` or an equivalent typed builder call.

2. Migrate `src/runtime/node.rs` by separating orchestration events from operation-owned results.
- Keep `run_node_from_config` responsible only for the runtime-entry boundary event currently named `runtime.startup.entered`.
- Replace the current `start_attrs` map with a typed runtime-startup-entered event carrying `scope`, `member_id`, `startup_run_id`, and `logging.level`.
- Split the current planner emission in `plan_startup_with_probe` into explicit ownership buckets:
- data-dir inspection result becomes operation-owned by a small wrapper around `inspect_data_dir`, because filesystem inspection is the side effect boundary.
- DCS cache probe result becomes operation-owned by a small wrapper around the injected `probe`, because the external DCS read is the side effect boundary.
- startup-mode selection remains planner-owned in `plan_startup_with_probe`, because only the planner can describe the synthesized startup mode decision.
- Replace the current direct attr-map events with typed runtime events for:
- `runtime.startup.data_dir.inspected` success/failure
- `runtime.startup.dcs_cache_probe` success/failure
- `runtime.startup.mode_selected`
- Rework `execute_startup` so the sequence-level startup pipeline events remain there, because `execute_startup` is the only place that owns startup ordering, action indexes, and the full mode-wide context:
- `runtime.startup.actions_planned` stays in `execute_startup`
- `runtime.startup.action` with result `started` stays in `execute_startup`
- `runtime.startup.action` with result `ok` stays in `execute_startup`
- `runtime.startup.action` with result `failed` stays in `execute_startup`
- Do not force action result ownership downward just for the sake of the story rule. In this file, the outer loop still owns the sequencing semantics, so the migration should focus on typed payloads and on moving only operation-specific side-effect logs downward.
- Keep lower executors responsible for their own operation-specific events only where they already represent a distinct boundary, such as startup subprocess raw-line forwarding failures in `run_startup_job`.
- Replace `emit_startup_phase` with a typed runtime event instead of the current plain `log.emit(...)` path so startup phases are still application events with stable headers.
- Keep `emit_startup_subprocess_line` on the raw-record path, but ensure every call uses `SubprocessLineRecord` directly and never hand-builds a fallback record.
- Replace the `runtime.startup.subprocess_log_emit_failed` attr assembly in `run_startup_job` with a typed runtime/process-output-fallback event builder.
- During execution, update the task inventory if any new runtime event sites are introduced beyond these ownership moves.

3. Migrate `src/process/worker.rs` so the worker owns admission/execution outcomes where they are decided.
- Convert the outer worker lifecycle events in `run` and `step_once` to typed process-worker events without changing their placement:
- `process.worker.run_started` stays in `run`
- `process.worker.request_received` stays in `step_once`
- `process.worker.inbox_disconnected` stays in `step_once`
- Replace the local attr maps in those functions with typed event constructors/builders.
- Refactor `start_job` so admission and startup outcome events are emitted where the worker actually decides the state transition:
- busy reject stays in `start_job`, because `start_job` owns admission control
- fencing preflight noop and fencing preflight failure stay in `start_job`; the helper should remain an inspection helper, while `start_job` still owns the short-circuit and idle transition semantics
- start-postgres noop and start-postgres preflight failure also stay in `start_job` for the same reason
- build-command failure remains with the command-build operation boundary inside `start_job`
- spawn failure remains with the spawn operation boundary inside `start_job`
- job-started remains in `start_job`, because the worker has the only full context after command spawn and state transition
- Convert each of those existing names to typed process events while preserving payload like `job_id`, `job_kind`, `binary`, `data_dir`, and `error`.
- Refactor `tick_active_job` around two repeated concerns before migrating events:
- extract a small helper for draining subprocess output and emitting raw lines
- extract a small helper for building the common process log identity fields
- Then convert the current events to typed process events at the point the outcome is known:
- `process.worker.output_drain_failed`
- `process.job.timeout`
- `process.job.exited` with result `ok`
- `process.job.exited` with result `failed`
- `process.job.poll_failed`
- Keep raw stdout/stderr lines as raw records via `SubprocessLineRecord`; they are not app events.
- Replace `emit_process_output_emit_failed` with a typed failure-event helper instead of attr-map assembly, preserving `stream`, `bytes_len`, and identity fields.
- Preserve all existing state transition semantics; event migration must not change job lifecycle behavior.

4. Migrate `src/api/worker.rs` by making request-boundary and TLS/auth ownership explicit.
- Keep `api.step_once_failed` in `run`, because only `run` knows whether a step failure is fatal or non-fatal and therefore owns the severity/fatality classification.
- Convert `api_base_attrs` from a map helper into typed field insertion used by api event builders; the helper may remain only if it returns typed fields instead of raw JSON values.
- Keep request-boundary events in `step_once` and convert them to typed api events:
- `api.connection_accepted`
- `api.request_parse_failed`
- `api.response_sent`
- Preserve request-scoped fields such as `scope`, `member_id`, `api.peer_addr`, `api.tls_mode`, `api.status_code`, and request id when present.
- Replace `emit_api_auth_decision` with a typed auth-decision event builder or small typed event enum that owns:
- peer address
- method
- route template
- auth header presence
- auth decision result
- required role
- optional request id
- Keep the auth decision at the auth boundary rather than in outer orchestration after the response is written.
- Keep TLS outcomes in `accept_connection`, because TLS handshake and client-cert checks are owned there:
- `api.tls_client_cert_missing`
- `api.tls_handshake_failed`
- If execution shows a value in logging successful TLS handshake completion, only add it if the task file inventory is updated in the same change; otherwise keep the current event set stable.
- Ensure typed auth/TLS events continue to avoid leaking bearer token material in logs.

5. Migrate tests away from string-key assertions and toward typed decoding.
- Update runtime tests in `src/runtime/node.rs` to use `decode_app_event` and assert `AppEventHeader` plus structured fields instead of `record.attributes.get("event.name")`.
- Update process tests in `src/process/worker.rs` similarly for request-received, job-started, and any outcome assertions touched by the refactor.
- Update api tests in `src/api/worker.rs` to assert decoded typed events rather than raw attribute lookups, while preserving the bearer-token redaction assertion over serialized output.
- Add a small shared test helper if needed for these three files only, but prefer reusing the existing `decode_app_event` helper from `src/logging/mod.rs`.
- For raw subprocess-line tests, assert against raw-record fields and source metadata rather than forcing them through app-event decoding.

6. Execute the implementation in the order that minimizes churn and compile breaks.
- First add any small domain event structs/builders and lightweight typed-field helpers needed by runtime/process/api.
- Next migrate runtime, because it mixes all three categories: orchestration events, operation-owned results, and raw subprocess output.
- Then migrate process worker, using the same subprocess raw-record path and fallback-event pattern runtime will establish.
- Then migrate api worker, reusing the typed app-event pattern for boundary/auth/TLS events.
- After code migration, update any story inventory text in this task if event placement or helper structure changed materially during execution.
- Update docs that mention the old attr-map style or obsolete ownership expectations if any such docs are touched by the implementation.

7. Verification requirements for the execution pass.
- Run and pass all required gates with no skips:
- `make check`
- `make test`
- `make test-long`
- `make lint`
- Confirm the touched domain files no longer assemble normal app-event payloads with local `BTreeMap<String, Value>`.
- Confirm raw subprocess output still uses typed raw-record builders and remains visible in logs.
- Only after the full suite is green should this task flip `<passes>true</passes>`, run `/bin/bash .ralph/task_switch.sh`, commit, and push.

## Key review points for the next engineer

- The most important structural choice to verify was whether `runtime.startup.action` result events should move into lower executors. After reviewing the code, this plan now keeps `started`/`ok`/`failed` in `execute_startup`, because only that loop owns action ordering and index metadata.
- The second review correction is that process preflight noop/failure events should stay in `start_job`, not in thin wrappers around `fencing_preflight_is_already_stopped` or `start_postgres_preflight_is_already_running`. Those helpers determine filesystem/process facts, but `start_job` owns the worker-level decision and resulting state transition.
- The remaining point to verify during execution is the shape of domain-specific typed events: sibling modules may be cleaner than private sections inside already-large files, but the final choice should optimize for low churn and readable ownership.

NOW EXECUTE

</implementation_plan>
