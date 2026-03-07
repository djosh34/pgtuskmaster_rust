## Task: Migrate HA, DCS, PgInfo, and Postgres ingest logging to owned typed events <status>done</status> <passes>true</passes>

<description>
**Goal:** Convert the remaining control-plane and ingest domains to the typed event contract, with special attention to keeping orchestration decisions separate from operation-owned results and keeping external postgres log lines on a typed raw-record path. The higher order goal is a uniform event model across control-plane state machines and ingest workers, without reintroducing free-form `serde_json` value assembly in domain code.

**Scope:**
- `src/ha/worker.rs`
- `src/ha/events.rs`
- `src/dcs/worker.rs`
- `src/pginfo/worker.rs`
- `src/logging/postgres_ingest.rs`
- related tests in those same files
- any shared typed event modules introduced by Task 01 that these domains must consume
- convert all current app-event emission in these files away from hand-built attr maps and onto the Task 01 typed event contract
- keep parsed postgres lines and raw fallback lines on the Task 01 typed raw-log-record builder path rather than forcing them through the app-event type
- apply the story-wide ownership rule explicitly in these domains:
- outer orchestration functions keep only orchestration-boundary events.
- functions that perform side effects own execution result events.
- pure helpers and planners do not emit unless they are themselves the operational boundary.

**Context from research:**
- `src/ha/worker.rs` currently emits phase and role transitions from `step_once`, while `src/ha/events.rs` separately emits action-intent, decision-selected, effect-plan-selected, dispatch, result, and lease events through helper functions that still build `BTreeMap<String, Value>`.
- `src/dcs/worker.rs` currently centralizes all DCS health, trust, write, drain, and refresh events in `step_once`.
- `src/pginfo/worker.rs` currently emits poll-failed and sql-transition events directly from `step_once`.
- `src/logging/postgres_ingest.rs` currently emits ingest-worker operational events from `run`, `emit_ingest_step_failure`, `emit_ingest_retry_recovered`, and `step_once`, while `emit_postgres_line` constructs raw records for external log lines.
- These are the remaining emit-heavy domains after runtime/process/api and they complete the inventory required by this story.

**Expected outcome:**
- HA, DCS, PgInfo, and ingest code no longer construct app-event payloads with ad hoc maps.
- HA action and transition events have a clear ownership model between `ha::worker` orchestration and lower helper execution.
- Postgres ingest keeps typed external-log records and typed ingest-worker events as two deliberate paths rather than one mixed map-based mechanism.
- These domains emit typed events through the shared contract and later feed the `tracing` backend layer without domain code using `tracing` APIs directly.

</description>

<acceptance_criteria>
- [x] `src/ha/events.rs`: replace the current map-building helper layer with typed HA events for decision-selected, effect-plan-selected, action-intent, action-dispatch, action-result, and lease-transition emission.
- [x] `src/ha/worker.rs`: migrate phase-transition and role-transition emission to typed HA events and explicitly decide which events belong in `step_once` orchestration versus lower HA helpers.
- [x] `src/dcs/worker.rs`: migrate all DCS write-failed, watch-drain-failed, watch-apply-had-errors, watch-refresh-failed, store-health-transition, and trust-transition events to typed DCS events.
- [x] `src/pginfo/worker.rs`: migrate poll-failed and sql-status-transition events to typed PgInfo events owned by the poll or transition logic that determines them.
- [x] `src/logging/postgres_ingest.rs`: migrate `run`, `emit_ingest_step_failure`, `emit_ingest_retry_recovered`, and `step_once` operational events to typed ingest events.
- [x] `src/logging/postgres_ingest.rs`: migrate `emit_postgres_line` and related parser output paths to typed raw-log-record builders so parsed postgres lines and parse-failed fallback lines are still structured without call-site `BTreeMap<String, Value>` assembly.
- [x] `src/ha/events.rs`, `src/ha/worker.rs`, `src/dcs/worker.rs`, `src/pginfo/worker.rs`, and `src/logging/postgres_ingest.rs` tests: replace direct string-key event assertions with typed event assertions or typed decoding helpers.
- [x] Domain code in these files does not emit normal application events via direct `tracing` macros or direct `BTreeMap<String, Value>` assembly; it emits typed events through the shared contract.
- [x] The migration preserves current operator-facing semantics for HA correlation fields, DCS trust or health transitions, PgInfo health transitions, and Postgres ingest parse-failure retention.
- [x] If new emit sites are introduced while refactoring these files, the task updates the story inventory instead of leaving them implicit.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] `make test-long` — passes cleanly
</acceptance_criteria>

<implementation_plan>

## Detailed plan

1. Reconfirm the migration contract before touching domain code.
- Re-read the Task 01 logging surface and keep this task strictly on top of the existing primitives: `AppEvent`, `AppEventHeader`, `StructuredFields`, `decode_app_event`, `RawRecordBuilder`, and `PostgresLineRecordBuilder`.
- Do not introduce any new compatibility layer that lets these domains keep hand-building app-event attr maps. If extra payload is needed, add typed field helpers or small domain-specific typed event constructors instead.
- Keep raw external-log ingestion on the raw-record path. Parsed postgres lines and parse-failed fallback lines are not app events and should not be forced through `AppEvent`.
- Prefer local helper sections or small sibling event helpers only where they reduce duplication; do not add broad cross-domain abstractions for one-off field sets.

2. Migrate `src/ha/events.rs` into typed HA event constructors while preserving event semantics.
- Replace `ha_base_attrs` with a typed HA field helper that fills the stable shared fields: `scope`, `member_id`, `ha_tick`, and `ha_dispatch_seq`.
- Replace the current `emit_ha_*` functions so each constructs an `AppEvent` plus `StructuredFields` instead of a `BTreeMap<String, Value>`.
- Keep the same operator-facing names/results unless code review during execution proves a name is already wrong and all affected tests/docs are updated in the same change:
- `ha.action.intent`
- `ha.decision.selected`
- `ha.effect_plan.selected`
- `ha.action.dispatch`
- `ha.action.result` with `ok`
- `ha.action.result` with `skipped`
- `ha.action.result` with `failed`
- `ha.lease.acquired`
- `ha.lease.released`
- Preserve the serialized `decision`, `effect_plan`, and `phase_*` payload shapes via `StructuredFields::insert_serialized` or an equivalent typed encoding path so downstream semantics stay stable.
- Delete the private `emit_event` wrapper once the file no longer needs to adapt `EventMeta` plus attr maps.

3. Migrate `src/ha/worker.rs` with explicit ownership between orchestration and operation helpers.
- Keep `step_once` responsible for orchestration-boundary events that only it can know:
- `ha.decision.selected`
- `ha.effect_plan.selected`
- `ha.phase.transition`
- `ha.role.transition`
- Add typed HA helpers for phase and role transitions instead of open-coding `ctx.log.emit_event(...)` with local maps.
- Keep dispatch and lease result events in the lower HA apply path via the existing `src/ha/events.rs` helpers, because those helpers sit closer to the side-effect boundary and already own action-specific context.
- During execution, inspect `src/ha/apply.rs` and any HA helper callers to ensure no remaining HA app events still build ad hoc maps outside the migrated helper layer. If any are found, either migrate them in this task or update the task inventory immediately.
- Update HA tests in both `src/ha/events.rs` and `src/ha/worker.rs` to decode typed app events with `decode_app_event` instead of asserting directly on `record.attributes.get("event.name")`.

4. Migrate `src/dcs/worker.rs` by introducing typed DCS worker events at the exact failure and transition boundaries already present.
- Add a small typed DCS event helper section or sibling module that owns the common `scope` and `member_id` fields and builds `AppEvent` instances for each current emission site.
- Replace the attr-map emission in `step_once` for:
- `dcs.local_member.write_failed`
- `dcs.watch.drain_failed`
- `dcs.watch.apply_had_errors`
- `dcs.watch.refresh_failed`
- `dcs.store.health_transition`
- `dcs.trust.transition`
- Preserve the current severity/result choices, especially the IO-vs-other failure classification and the recovered/failed result on store health transitions.
- Keep these events in `step_once`; that function still owns the publish loop, cache refresh boundary, and trust/health transition comparison.
- Update DCS tests to use typed decode helpers and assert the stable fields/results rather than raw string-key lookups.

5. Migrate `src/pginfo/worker.rs` so polling and SQL transition ownership stays local to the poll loop but uses typed events.
- Add a small pginfo typed event helper section or sibling module rather than open-coding app-event construction twice in `step_once`.
- Replace the current map-based emissions for:
- `pginfo.poll_failed`
- `pginfo.sql_transition`
- Preserve the current transition classification:
- `healthy -> unreachable` remains warn/failed
- `unreachable -> healthy` remains info/recovered
- all other status transitions remain debug/ok
- Keep these events in `step_once`; the poll loop owns both the poll result and the previous-vs-next SQL status comparison.
- Update pginfo tests so they decode app events and assert header plus structured fields, while keeping the real-postgres coverage intact.

6. Migrate `src/logging/postgres_ingest.rs` by separating typed ingest-worker app events from typed raw postgres records.
- Keep the raw postgres line path centered on `emit_postgres_line`, `normalize_postgres_line`, `PostgresLineRecordBuilder`, and `RawRecordBuilder`.
- Verify that `emit_postgres_line` never assembles `BTreeMap<String, Value>` directly for app events. Parsed JSON/plain lines should still become raw records with structured fields, and non-UTF8/fallback lines should remain structured raw records with `parse_failed` and `raw_line`.
- Tighten the raw-record side as well: replace `ParsedLine { attributes: BTreeMap<String, Value> }` plus `StructuredFields::from_json_map(...)` with `ParsedLine { fields: StructuredFields }` or equivalent typed raw-record helpers, so postgres ingest no longer hand-assembles raw-record attr maps either.
- Replace the app-event attr maps in:
- `emit_ingest_step_failure`
- `emit_ingest_retry_recovered`
- the success event emitted from `step_once` for `postgres_ingest.iteration`
- Add a small ingest typed-event helper for common fields such as `attempts`, `suppressed`, `error`, `pg_ctl_lines_emitted`, `log_dir_files_tailed`, `log_dir_lines_emitted`, and `dir_tailers`.
- Preserve the existing rate-limit behavior exactly. This task changes event construction, not retry or suppression semantics.
- Update ingest tests so worker app events use typed decoding and raw postgres line tests continue asserting raw-record content rather than app-event headers.

7. Sequence the implementation to minimize compile churn and review risk.
- First add or refactor any small typed event helper sections for HA, DCS, PgInfo, and postgres ingest.
- Then migrate HA first, because its split between orchestration events in `worker.rs` and operation-owned events in `events.rs` is the main ownership-sensitive part of the task.
- Next migrate DCS and PgInfo, which are both localized `step_once` conversions with straightforward typed-field shapes.
- Migrate postgres ingest last so the app-event changes can reuse patterns established earlier while keeping the raw-record path unchanged.
- After each domain migration, run the smallest relevant test target if a focused target exists; after the full code pass, run the required full gates in the order requested by the task instructions.

8. Update tests and docs as part of the same execution pass.
- Replace direct stringly assertions in the touched test modules with `decode_app_event` or a tiny local typed helper that wraps it.
- Keep raw-record assertions raw; do not decode raw postgres lines as app events.
- Search for docs or story text that still describe these domains as using attr-map app events and update or remove that stale text in the same pass.
- If execution reveals additional event sites in the scoped files, update this task inventory rather than leaving them implicit.

9. Verification and completion requirements for the execution pass.
- Confirm the touched domain files no longer assemble normal application events with local `BTreeMap<String, Value>` payloads.
- Confirm postgres raw-line ingestion still emits structured raw records for parsed and fallback lines.
- Run and pass all required gates with no skips:
- `make check`
- `make test`
- `make test-long`
- `make lint`
- Only after the full suite is green should this task flip `<passes>true</passes>`, run `/bin/bash .ralph/task_switch.sh`, commit all changes including `.ralph`, and push.

## Key review points for the next engineer

- The main ownership decision to verify is the HA split: `step_once` should keep orchestration-only events such as decision, effect-plan, phase-transition, and role-transition, while dispatch/result/lease events stay nearer the lower HA side-effect helpers.
- The second point to review is postgres ingest: only the worker operational events should move to typed `AppEvent`; `emit_postgres_line` must stay on the raw-record builder path even when parsing fails.
- The skeptical review changed one requirement: the raw-record helper internals also need to stop returning `BTreeMap<String, Value>` payloads. Keeping raw records instead of app events was correct, but the plan now also requires typed `StructuredFields` inside `normalize_postgres_json` and `normalize_postgres_plain`.
- The third point to review is whether any helper module split actually improves readability. If small local helper sections inside the existing files are clearer and lower-churn, prefer that over creating extra modules.

NOW EXECUTE

</implementation_plan>
