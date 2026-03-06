## Task: Migrate runtime, process, and api logging to owned typed events <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Convert the most orchestration-heavy logging paths from ad hoc attr maps into typed domain events, and move event ownership to the code that actually owns the semantics of the action or failure. The higher order goal is to stop outer orchestration functions from being the default place where every event is assembled, while still preserving true orchestration boundary events where they add operator value.

**Scope:**
- `src/runtime/node.rs`
- `src/process/worker.rs`
- `src/api/worker.rs`
- `src/api/events.rs`
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
- `src/api/events.rs` currently emits from `ingest_wal_event`.
- The exact pattern the migration must evaluate is the current outer orchestration style of “emit intent in outer function, call lower function, emit result in outer function” versus moving execution-result events into `do_x` / `do_y` ownership boundaries.

**Expected outcome:**
- No app event in these domains requires local `BTreeMap<String, Value>` assembly.
- Runtime startup, process job handling, and API request lifecycle events have a deliberate ownership model rather than accidental placement.
- Tests for these areas assert typed events or typed decoders instead of stringly typed map lookups.
- These domains emit typed events through the shared contract and later feed the `tracing` backend layer without domain code using `tracing` APIs directly.

</description>

<acceptance_criteria>
- [ ] `src/runtime/node.rs`: migrate `run_node_from_config` startup-entered logging to typed runtime events and keep only true runtime-entry boundary events there.
- [ ] `src/runtime/node.rs`: migrate `plan_startup_with_probe` events for data-dir inspection, DCS probe, and startup-mode selection to typed runtime events and explicitly decide which of those are planner-owned versus lower-level operation-owned.
- [ ] `src/runtime/node.rs`: migrate `execute_startup` events for action-planned, action-started, action-completed, and action-failed; document whether action result events stay in `execute_startup` or move into lower action executors.
- [ ] `src/runtime/node.rs`: migrate `emit_startup_phase`, `run_startup_job`, and `emit_startup_subprocess_line` to typed runtime/raw-log builders so startup subprocess output is not hand-built as free-form maps or records.
- [ ] `src/process/worker.rs`: migrate `run` and `step_once` worker lifecycle events to typed process events.
- [ ] `src/process/worker.rs`: migrate `start_job` events for busy reject, fencing noop, preflight failure, build-command failure, spawn failure, and job-started to typed process events owned by the operation that determines those outcomes.
- [ ] `src/process/worker.rs`: migrate `tick_active_job` events for output-drain failure, timeout, exit success, exit failure, and poll failure to typed process events, and route subprocess line output through typed raw-log builders.
- [ ] `src/process/worker.rs`: migrate `emit_process_output_emit_failed` and `emit_subprocess_line` so even fallback/failure logging paths use typed helpers rather than ad hoc attributes.
- [ ] `src/api/worker.rs`: migrate `run` fatal or non-fatal step failure events to typed api events with explicit ownership.
- [ ] `src/api/worker.rs`: migrate `step_once` connection accepted, request parse failure, response sent, and related request lifecycle events to typed api events without local map assembly.
- [ ] `src/api/worker.rs`: migrate `emit_api_auth_decision` and `accept_connection` so auth and TLS outcomes are typed api events and the final placement is explicit about outer request orchestration versus TLS/auth operation ownership.
- [ ] `src/api/events.rs`: migrate `ingest_wal_event` to a typed backup or api event path rather than local JSON-value map construction.
- [ ] `src/runtime/node.rs`, `src/process/worker.rs`, `src/api/worker.rs`, and `src/api/events.rs` tests: replace direct `record.attributes.get("event.name")` style assertions with typed event assertions or typed decoding helpers.
- [ ] Domain code in these files does not emit normal application events via direct `tracing` macros or direct `BTreeMap<String, Value>` assembly; it emits typed events through the shared contract.
- [ ] If new emit sites are introduced while refactoring these files, the task updates the story inventory instead of leaving them implicit.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] `make test-long` — passes cleanly
</acceptance_criteria>
