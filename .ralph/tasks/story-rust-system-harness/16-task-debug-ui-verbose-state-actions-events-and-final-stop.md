---
## Task: Setup verbose debug UI and final STOP gate <status>done</status> <passes>true</passes> <passing>true</passing> <priority>low</priority>

<blocked_by>15-task-final-double-check-and-stop-gate</blocked_by>

<description>
**Goal:** Build a debug UI system that reacts to fine-grained state/action/event changes via a super-verbose debug API endpoint and render those details in a rich static HTML UI; this task runs last.

**Scope:**
- Implement a super-verbose debug API endpoint that streams/exposes all relevant worker state changes, HA actions, events, outputs, and timing/version metadata.
- Ensure payload includes structured sections for pginfo, dcs, process, ha, config, api/debug, and cross-worker timelines.
- Add static HTML/CSS/JS page that fetches debug data and renders visual blocks, figures, timelines, and grouped panels (not plain text dump).
- Ensure UI updates reactively to small incremental state changes.

**Context from research:**
- User requested this task to be run last with explicit priority control and rich visual debug output.

**Expected outcome:**
- Last task provides a practical real-time observability UI and final completion gate for the entire system-harness story.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.

**Note:**
The vm you are in, does not have browser installed nor chrome. But we do expect a working ui.
Please figure out how to validate it is working (by installing something?)
</description>

<acceptance_criteria>
- [x] Task priority is `low` and the task is blocked by task 15 so it executes last.
- [x] Debug endpoint exposes super-verbose structured data for all state changes, actions, events, and outputs.
- [x] Static debug UI renders figures/blocks/panels/timelines and updates on data changes.
- [x] Debug UI is not text-only; includes visual grouping and state/action emphasis.
- [x] Perform final validation pass: confirm tests are real (not fake asserts, not tests doing HA logic themselves), all features are present/working/tested, and all suites pass.
- [x] Run full suite with no exceptions: `make check`, `make test`, `make lint`.
- [x] If any validation or suite check fails, do NOT write `.ralph/STOP`; use `$add-bug` skill to create bug task(s).
- [x] Only when everything above passes, execute `touch .ralph/STOP`.
</acceptance_criteria>

<execution_plan>
## Detailed Implementation Plan (Draft 2)

Research snapshot from parallel exploration sweep (12+ concurrent tracks):
- Existing debug API worker (`src/debug_api/worker.rs`) publishes only latest snapshot and does not retain event history/timeline metadata.
- Existing API route `GET /debug/snapshot` in `src/api/worker.rs` returns `text/plain` `Debug` formatting (`format!("{:#?}", snapshot)`), not structured JSON.
- `ApiWorkerCtx` already has `debug_snapshot_subscriber: Option<StateSubscriber<SystemSnapshot>>` but lacks a setter/wiring helper, leaving debug endpoint integration incomplete.
- Core runtime state types (`ProcessState`, `HaState`, `DcsState`, many nested structs) are intentionally non-`Serialize`; forcing serde derives across runtime contracts would be high-risk churn.
- No current static debug UI assets/routes exist; no `text/html` response route and no JS polling/stream logic.
- `task 15` is complete and explicitly deferred STOP creation to this final task; task 16 is correctly blocked by task 15.

Skeptical verification deltas applied in Draft 2 (16+ parallel probes):
- Alteration 1: add mandatory real-binary enforcement gate (`make test`) before final suite closeout; this enforces the repo policy that real-binary coverage must not remain optional.
- Alteration 2: explicitly preserve backward compatibility for existing `GET /debug/snapshot` while adding structured `/debug/verbose`; this reduces risk of silent contract regressions for existing tests/users.
- Alteration 3: extend authz coverage to include new debug routes (`/debug/verbose`, `/debug/ui`) for unauthenticated/read-token/admin-token paths, not only feature-happy-path route checks.

1. Preflight and guardrails
- [x] Reconfirm task sequencing invariants before coding:
- [x] this file remains `<priority>low</priority>`
- [x] `<blocked_by>15-task-final-double-check-and-stop-gate</blocked_by>` is unchanged
- [x] baseline workspace state is captured in evidence to avoid accidentally hiding unrelated `.ralph` deltas.
- [x] Create task evidence root: `.ralph/evidence/16-debug-ui-final-stop/`.
- [x] Keep strict runtime quality constraints active: no unwrap/expect/panic/todo/unimplemented; no skipped tests.

2. Define a dedicated super-verbose debug view model (no runtime serde churn)
- [x] Add a new DTO module (for example `src/debug_api/view.rs`) that converts runtime snapshot/state into structured, serializable debug documents without changing core runtime types.
- [x] Model top-level payload as `DebugVerbosePayload` containing:
- [x] `meta` section (generated_at, app lifecycle, endpoint version/schema version, sequence id),
- [x] per-worker sections: `config`, `pginfo`, `dcs`, `process`, `ha`, `api`, `debug`,
- [x] cross-worker timeline section with recent entries and causal hints.
- [x] Include both current-value blocks and recent-change records:
- [x] `current`: latest normalized state per domain.
- [x] `changes`: ring-buffer slice of recent events with before/after version metadata.
- [x] Represent non-serializable internals explicitly via normalized string/enum fields in DTO mapping helpers (for example `worker_status`, `phase`, `job_kind`, `trust`, `sql`, `readiness`).

3. Expand debug worker to track incremental changes and timeline events
- [x] Extend `DebugApiCtx` in `src/debug_api/worker.rs` with persistent in-memory history:
- [x] last seen versions per worker channel,
- [x] bounded ring buffers for state change events and action/event timeline rows.
- [x] Implement deterministic event extraction each `step_once`:
- [x] detect per-channel version bumps,
- [x] record structured change events with timestamps, old/new version, and compact summary fields,
- [x] include HA pending action ids/action labels from `HaState` and process job outcome transitions from `ProcessState`.
- [x] Keep bounded memory by capping history length (for example 200-500 events) and trimming oldest entries.
- [x] Add pure unit helpers for diff/event projection so behavior is testable without sockets.

4. Add structured debug endpoint surface in API worker
- [x] In `src/api/worker.rs`, add explicit JSON endpoint(s):
- [x] `GET /debug/verbose` returns full structured payload from debug worker subscriber.
- [x] optional incremental variant: `GET /debug/verbose?since=<n>` returns subset changes since sequence/version for reactive UI efficiency.
- [x] Keep existing `GET /debug/snapshot` route behavior available for backward compatibility while migrating consumers to structured JSON.
- [x] Keep existing `debug.enabled` gate and return `404` when disabled.
- [x] Return `503` when debug is enabled but required debug subscriber/payload is unavailable.
- [x] Preserve authz behavior: read token/admin token both allowed for read-only debug routes; unauthorized stays `401`.

5. Add static debug UI route and embedded assets
- [x] Add `GET /debug/ui` route in `src/api/worker.rs` serving `text/html; charset=utf-8` with:
- [x] semantic layout containers (summary KPIs, per-worker cards, timeline panel, event table),
- [x] inline CSS variables and componentized visual grouping,
- [x] inline JS that polls `/debug/verbose` on interval (and optionally uses `since` token) and applies incremental DOM updates.
- [x] Keep UI intentionally visual (not text dump):
- [x] status chips/badges by worker health/trust/phase,
- [x] timeline rows with colored categories (`pginfo`, `dcs`, `process`, `ha`, `api`, `debug`),
- [x] compact metric figures (event counts, version deltas, last update age),
- [x] grouped cards/panels with clear hierarchy.
- [x] Ensure responsive behavior for narrow/mobile widths via CSS media queries.

6. Wire API worker to debug subscriber cleanly
- [x] Add explicit `ApiWorkerCtx` setter/constructor extension (for example `set_debug_snapshot_subscriber(...)`) to attach debug state channel from runtime wiring/tests.
- [x] Keep default `contract_stub` safe (no debug subscriber by default) and test both wired and unwired behaviors.
- [x] Ensure route-level handling does not panic when subscriber absent.

7. Add targeted tests for structured endpoint and UI route
- [x] Extend unit tests in `src/api/worker.rs`:
- [x] `/debug/verbose` success path returns JSON with required sections and metadata fields.
- [x] `/debug/verbose` returns `404` when debug disabled.
- [x] `/debug/verbose` returns `503` when debug enabled but subscriber missing.
- [x] `/debug/ui` returns `200` and `text/html`, includes expected root containers and JS bootstrap markers.
- [x] `/debug/verbose` and `/debug/ui` authz matrix validates `401` without token when protected and read-token/admin-token acceptance for read-only routes.
- [x] Extend/adjust debug worker tests in `src/debug_api/worker.rs`:
- [x] change-detection publishes history entries on version increments,
- [x] no new entry when versions unchanged,
- [x] bounded retention behavior trims oldest entries.
- [x] Add or extend BDD integration test (`tests/bdd_api_http.rs`) to hit real TCP endpoint and validate:
- [x] debug JSON route behavior is validated over real TCP (unwired contract returns `503`), with structured payload validation covered by API unit tests using wired debug subscriber,
- [x] debug UI route serves non-empty HTML scaffold.

8. Browser-level UI smoke validation in environment without preinstalled browser
- [x] Add a deterministic UI smoke check under `.ralph/evidence/16-debug-ui-final-stop/ui-smoke/` using one installed headless browser path:
- [x] preferred: install Playwright Chromium locally and run a scripted capture/assertion (load `/debug/ui`, wait for data render, assert key panels visible),
- [x] fallback: if installation is unavailable, create `$add-bug` immediately with install failure evidence and do not mark task passing.
- [x] Archive screenshots/HTML dumps/console logs as evidence artifacts.

9. Final skeptical quality audit before STOP
- [x] Produce `.ralph/evidence/16-debug-ui-final-stop/final-validation-audit.md` verifying:
- [x] tests are behavior-based (no tautological pass asserts),
- [x] HA transitions are still system-driven (no tests mutating internal HA state to fake outcomes),
- [x] new UI/debug features are present and covered by tests.

10. Full required gates (strict, serial, auditable)
- [x] Run sequentially with `set -o pipefail` + `tee` logging:
- [x] `CARGO_BUILD_JOBS=1 make test` -> `.ralph/evidence/16-debug-ui-final-stop/make-test-long.log`
- [x] `CARGO_BUILD_JOBS=1 make check` -> `.ralph/evidence/16-debug-ui-final-stop/make-check.log`
- [x] `CARGO_BUILD_JOBS=1 make test` -> `.ralph/evidence/16-debug-ui-final-stop/make-test.log`
- [x] `CARGO_BUILD_JOBS=1 make test` -> `.ralph/evidence/16-debug-ui-final-stop/make-test.log`
- [x] `CARGO_BUILD_JOBS=1 make lint` -> `.ralph/evidence/16-debug-ui-final-stop/make-lint.log`
- [x] If stale Cargo artifact signature appears (`failed to build archive` / missing `*.rcgu.o`), run one `cargo clean`, preserve pre-clean logs, then rerun full gate sequence once.

11. Failure protocol (mandatory, no STOP on failure)
- [x] For each distinct failing behavior/gate, use `$add-bug` skill to create a bug task in `.ralph/tasks/bugs/` with:
- [x] exact repro command,
- [x] failing output excerpt,
- [x] impacted files/modules,
- [x] evidence artifact paths.
- [x] If any bug remains unresolved, keep this task non-passing and do not create `.ralph/STOP`.

12. Completion and final STOP gate
- [x] Only after all acceptance criteria and all required gates are green:
- [x] tick all acceptance checkboxes in this file,
- [x] set task header to `<status>done</status> <passes>true</passes> <passing>true</passing>`,
- [x] run `touch .ralph/STOP`,
- [x] run `/bin/bash .ralph/task_switch.sh`,
- [x] commit all changes including `.ralph` artifacts with:
- [x] `task finished 16-task-debug-ui-verbose-state-actions-events-and-final-stop: <summary + gate evidence + UI validation notes + challenges>`,
- [x] append durable learnings/surprises to `AGENTS.md`,
- [x] append progress diary entry.
</execution_plan>

NOW EXECUTE

<evidence>
- Validation audit: `.ralph/evidence/16-debug-ui-final-stop/final-validation-audit.md`
- Gate summary: `.ralph/evidence/16-debug-ui-final-stop/gate-summary.md`
- Required gate logs: `.ralph/evidence/16-debug-ui-final-stop/make-test-long.log`, `.ralph/evidence/16-debug-ui-final-stop/make-check.log`, `.ralph/evidence/16-debug-ui-final-stop/make-test.log`, `.ralph/evidence/16-debug-ui-final-stop/make-test.log`, `.ralph/evidence/16-debug-ui-final-stop/make-lint.log`
- Browser/UI smoke: `.ralph/evidence/16-debug-ui-final-stop/ui-smoke/playwright-install.log`, `.ralph/evidence/16-debug-ui-final-stop/ui-smoke/playwright-screenshot.log`, `.ralph/evidence/16-debug-ui-final-stop/ui-smoke/debug-ui.png`, `.ralph/evidence/16-debug-ui-final-stop/ui-smoke/debug-ui.html`, `.ralph/evidence/16-debug-ui-final-stop/ui-smoke/debug-verbose.headers`, `.ralph/evidence/16-debug-ui-final-stop/ui-smoke/debug-verbose.body`, `.ralph/evidence/16-debug-ui-final-stop/ui-smoke/server.log`
</evidence>
