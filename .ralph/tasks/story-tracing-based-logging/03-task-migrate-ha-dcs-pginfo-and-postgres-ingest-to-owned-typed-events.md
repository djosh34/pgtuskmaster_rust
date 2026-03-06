## Task: Migrate HA, DCS, PgInfo, and Postgres ingest logging to owned typed events <status>not_started</status> <passes>false</passes>

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
- convert all current app-event emission in these files away from hand-built attr maps
- keep parsed postgres lines and raw fallback lines in a typed raw-log path rather than forcing them through the app-event type
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
- [ ] `src/ha/events.rs`: replace the current map-building helper layer with typed HA events for decision-selected, effect-plan-selected, action-intent, action-dispatch, action-result, and lease-transition emission.
- [ ] `src/ha/worker.rs`: migrate phase-transition and role-transition emission to typed HA events and explicitly decide which events belong in `step_once` orchestration versus lower HA helpers.
- [ ] `src/dcs/worker.rs`: migrate all DCS write-failed, watch-drain-failed, watch-apply-had-errors, watch-refresh-failed, store-health-transition, and trust-transition events to typed DCS events.
- [ ] `src/pginfo/worker.rs`: migrate poll-failed and sql-status-transition events to typed PgInfo events owned by the poll or transition logic that determines them.
- [ ] `src/logging/postgres_ingest.rs`: migrate `run`, `emit_ingest_step_failure`, `emit_ingest_retry_recovered`, and `step_once` operational events to typed ingest events.
- [ ] `src/logging/postgres_ingest.rs`: migrate `emit_postgres_line` and related parser output paths to typed raw-log-record builders so parsed postgres lines and parse-failed fallback lines are still structured without call-site `BTreeMap<String, Value>` assembly.
- [ ] `src/ha/events.rs`, `src/ha/worker.rs`, `src/dcs/worker.rs`, `src/pginfo/worker.rs`, and `src/logging/postgres_ingest.rs` tests: replace direct string-key event assertions with typed event assertions or typed decoding helpers.
- [ ] Domain code in these files does not emit normal application events via direct `tracing` macros or direct `BTreeMap<String, Value>` assembly; it emits typed events through the shared contract.
- [ ] The migration preserves current operator-facing semantics for HA correlation fields, DCS trust or health transitions, PgInfo health transitions, and Postgres ingest parse-failure retention.
- [ ] If new emit sites are introduced while refactoring these files, the task updates the story inventory instead of leaving them implicit.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] `make test-long` — passes cleanly
</acceptance_criteria>
