## Task: Expand Contributor Docs into a Full Implementation Deep Dive <status>done</status> <passes>true</passes>

<description>
**Goal:** Rewrite the Contributors section into an in-depth engineering deep dive that explains how the code actually works, how modules connect, and how behavior flows through runtime paths, while keeping prose natural, readable, and technically precise.

**Scope:**
- Expand contributor chapters one by one, turning thin summaries into full implementation narratives with explicit call paths, ownership boundaries, and state transitions.
- Require section-by-section depth for architecture internals:
  - module responsibilities and why boundaries exist
  - startup/runtime data flow
  - worker-to-worker coordination
  - DCS, HA, process, and API interaction contracts
  - testing strategy and harness mechanics tied back to code.
- Keep writing quality high and human:
  - flowing paragraph-first explanations, not checklist-only fragments
  - clear transitions between concepts
  - precise terminology with definitions where needed.
- Allow code snippets in contributor docs where examples clarify behavior:
  - examples should explain by concrete code path, not generic pseudo-prose
  - snippets must be minimal, relevant, and tied to surrounding explanation.
- Update navigation and cross-links so contributor docs read as a coherent deep technical guide rather than disconnected pages.

**Context from research:**
- Current contributors pages exist under `docs/src/contributors/`, but several chapters are still too slim for onboarding engineers who need a code-level mental model.
- The existing docs restructure already improved top-level flow; this task focuses specifically on contributor depth and implementation-level clarity.
- The codebase has clear runtime worker boundaries (`runtime`, `api`, `dcs`, `ha`, `process`, `pginfo`, `test_harness`) that should be documented with explicit “who calls whom, when, and why.”

**Expected outcome:**
- A contributor can read the Contributors section and build an accurate mental model of runtime architecture, control flow, and failure handling without jumping constantly into source.
- Each contributor chapter includes dense, correct, practical detail with natural narrative flow, not just high-level summaries.
- Code snippets are present where they materially improve comprehension, and each snippet is contextualized in prose.
- Contributors docs become the definitive deep-dive reference for implementation understanding and extension work.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x]  Full exhaustive checklist of all files/modules to modify with specific requirements for each
- [x] `docs/src/contributors/index.md` is expanded into a clear deep-dive entrypoint that maps how to read the contributor docs and which chapter answers which engineering question.
- [x] `docs/src/contributors/codebase-map.md` is rewritten to include concrete module boundaries, crate-level ownership, and explicit runtime call/data paths (startup, steady-state, decision/action loops).
- [x] `docs/src/contributors/worker-wiring.md` explains state and message flow across workers in sequence, including channel ownership, update cadence, and failure-path behavior.
- [x] `docs/src/contributors/ha-pipeline.md` documents HA decision internals end-to-end (inputs, decision logic, action dispatch, process feedback) with nitty-gritty transition reasoning.
- [x] `docs/src/contributors/api-debug-contracts.md` documents API and debug contracts end-to-end (route/controller ownership, intent write path, debug snapshot projection), and explains how HA/process/DCS state surfaces to clients.
- [x] `docs/src/contributors/testing-system.md` is expanded into a full testing architecture guide covering unit/integration/BDD/real-binary layers, confidence boundaries, and when to add each test type.
- [x] `docs/src/contributors/harness-internals.md` details harness internals deeply (resource setup, process lifecycle, networking/proxy behavior, determinism controls, and common flake sources).
- [x] `docs/src/contributors/docs-style.md` is updated to require flowing, paragraph-first, technically dense writing for contributor chapters and to permit targeted code snippets when they improve clarity.
- [x] `docs/src/contributors/verification.md` explains the verification workflow in concrete operational terms (claim extraction, evidence standards, and correction loop) without leaking audit noise into operator-facing docs.
- [x] `docs/src/SUMMARY.md` contributor subsection ordering/cross-links are updated so chapter progression is coherent for deep technical reading.
- [x] The contributor deep-dive *technical chapters* each include at least one explicit “how this connects to adjacent subsystem(s)” section that links behavior across modules (not file-by-file isolation):
  - [x] `docs/src/contributors/codebase-map.md`
  - [x] `docs/src/contributors/worker-wiring.md`
  - [x] `docs/src/contributors/ha-pipeline.md`
  - [x] `docs/src/contributors/api-debug-contracts.md`
  - [x] `docs/src/contributors/testing-system.md`
  - [x] `docs/src/contributors/harness-internals.md`
  - [x] `docs/src/contributors/verification.md`
- [x] Contributors prose is written in natural, flowing paragraphs with context-setting openings and clear transitions; no chapter remains a terse bullet-only outline.
- [x] Code snippets are allowed only in contributor docs and are used where they clarify concrete behavior; each snippet has surrounding explanation that states why the snippet matters.
- [x] All claims about behavior in contributor docs are aligned with current code paths and tests; over-claims are removed or rewritten to bounded language.
- [x] The reading order described in `docs/src/contributors/index.md` matches the Contributors subsection order in `docs/src/SUMMARY.md` (no drift / no double sources of truth).
- [x] `make docs-lint` passes cleanly
- [x] `make docs-build` passes cleanly
- [x] `make docs-hygiene` passes cleanly
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

## Plan (implementation deep dive rewrite)

### Guardrails for this doc rewrite

- Keep Contributors pages implementation-focused and code-traceable.
- Prefer flowing paragraphs for the main narrative; use bullets/tables only to summarize or to present crisp matrices.
- Use bounded language for behavior claims unless there is a guard/test that makes the statement effectively absolute.
- Code snippets are allowed only in `docs/src/contributors/` and must be:
  - minimal (prefer short excerpts, type definitions, or key match arms)
  - real (no “toy” placeholder signatures that do not exist in-tree)
  - surrounded by prose that explains why the snippet matters

### Exhaustive checklist: files to modify (with requirements)

- [x] `docs/src/contributors/index.md`
  - Expand into the entrypoint for deep technical onboarding.
  - Add a “question → chapter” map (examples: “I’m changing HA transitions”, “I’m changing DCS watch semantics”, “I’m debugging e2e flakes”).
  - Add a “read paths” section: quick skim path vs full deep-dive path.
  - Add explicit cross-links to adjacent non-contributor chapters (lifecycle, assurance, interfaces).
  - Keep this page navigation-focused (avoid forcing “adjacent subsystem connections” boilerplate here; reserve that for technical deep dives).

- [x] `docs/src/contributors/codebase-map.md`
  - Replace the current short list with a concrete ownership map and runtime call/data paths.
  - Include: startup call path (from binary → runtime → startup planner/executor), steady-state worker loops, and decision/action loops.
  - Provide a boundary table: module owns X; allowed dependencies; output state it publishes; inputs it consumes.
  - Add an “adjacent subsystem connections” section.

- [x] `docs/src/contributors/worker-wiring.md`
  - Describe the actual state bus (watch/versioned channels): what each worker publishes, who owns the sender, and what cadence/triggers updates.
  - Provide an “ownership matrix” for worker inputs/outputs (table preferred here).
  - Provide a step-by-step “one tick through the system” narrative (pginfo → dcs → ha → process + api/debug projections).
  - Include failure-path behavior: stale inputs, trust loss, dispatch failures, and how that is surfaced (HA worker fault state vs continuing publication).
  - Add an “adjacent subsystem connections” section.

- [x] `docs/src/contributors/ha-pipeline.md`
  - Document HA decision internals end-to-end:
    - inputs (pginfo, dcs cache/trust, process outcomes, config, clock/tick model)
    - pure decision core (FSM/decide)
    - action categories and dispatch semantics (DCS writes/deletes vs process jobs)
    - feedback loop from process outcomes back into the next decision
  - Include a “phase transitions” narrative with at least:
    - bootstrap path
    - election/promotion path
    - switchover request path (API → DCS → HA → process/DCS)
    - fencing/split-brain path
    - rewind/recovery path
  - Add an “action-to-side-effect matrix” (action → DCS op / process job kind).
  - Add an “adjacent subsystem connections” section.

- [x] `docs/src/contributors/api-debug-contracts.md`
  - Add a contributor-facing deep dive for API and debug contracts (task scope explicitly includes API interaction contracts).
  - Document the intent path end-to-end (high-level):
    - HTTP request → `src/api/*` controller/handler
    - “intent” writes into DCS (what is written, how it is keyed, and why HA trusts/doesn’t trust it)
    - HA worker consumes the intent via DCS cache/snapshot and translates it into actions (tie back to HA pipeline chapter)
    - process outcomes update observable state that the API returns
  - Document the debug API projection path:
    - how `src/debug_api/*` builds a snapshot
    - how `src/debug_api/view.rs` renders “verbose” views and why it exists (projection vs core state)
  - Include a short “client contract” section: what external clients can safely rely on vs what is “debug-only / best-effort”.
  - Add an “adjacent subsystem connections” section.

- [x] `docs/src/contributors/testing-system.md`
  - Expand into a test architecture guide that maps *what each layer can prove*.
  - Explicitly document how `make test` vs `make test-long` differ and why the suite is split.
  - Include “when to add which test type” decision guidance (unit vs contract vs BDD vs real-binary).
  - Include a minimal but concrete inventory of key tests and what subsystem they protect.
  - Add a “flake triage” section (symptoms → likely root cause → next probe).
  - Add an “adjacent subsystem connections” section.

- [x] `docs/src/contributors/harness-internals.md`
  - Document harness primitives deeply:
    - namespace and path layout
    - port leasing/reservation and race avoidance
    - binaries discovery/validation and why tests are not optional
    - Postgres and etcd process lifecycle + teardown guarantees
    - proxy/fault injection behavior (pass-through / blocked / latency, listener threading model)
    - determinism controls (timeouts, stable-primary sampling, local task sets)
  - Include common flake sources + how the harness mitigates them.
  - Add an “adjacent subsystem connections” section.

- [x] `docs/src/contributors/docs-style.md`
  - Upgrade into a strict contributor-doc writing contract:
    - required chapter shape (context → internals → adjacent links → tradeoffs)
    - paragraph-first density rules (bullets only as summaries)
    - snippet policy and size limits
    - claim language rules (bounded vs absolute)
    - minimum cross-link requirements between contributor chapters
  - Add a short “review checklist” for PR reviewers.

- [x] `docs/src/contributors/verification.md`
  - Turn principles into a concrete operational workflow:
    - how to identify and rewrite “claims” in docs
    - evidence standards (code paths, tests, runtime proofs)
    - handling “absence” claims (never/does not/cannot) with extra skepticism
    - correction loop and how to record artifacts without polluting operator docs
  - Cross-link to the canonical verification section under `docs/src/verification/`.
  - Add an “adjacent subsystem connections” section.

- [x] `docs/src/contributors/task-33-docs-verification-report.md`
  - Keep as historical appendix; update only if the preface/pointers are stale.
  - Do not expand it into the “current workflow”; keep current workflow in `docs/src/contributors/verification.md` + `docs/src/verification/`.

- [x] `docs/src/SUMMARY.md`
  - Reorder Contributors subsection for coherent progression (deep technical reading order).
  - Ensure nested structure still makes sense (keep harness under testing, or justify a different structure).

### Code modules to inspect and cite (evidence sources; not necessarily modified)

The rewrite must cite concrete call paths and types from these modules (paths are stable; line numbers are not):

- Startup + wiring: `src/bin/pgtuskmaster.rs`, `src/runtime/node.rs`
- Startup + wiring (managed Postgres config): `src/postgres_managed.rs`
- Config parsing/validation: `src/config/parser.rs`, `src/config/schema.rs`
- Shared state channel model: `src/state/watch_state.rs`, `src/state/time.rs`
- Pg observation: `src/pginfo/worker.rs`, `src/pginfo/query.rs`, `src/pginfo/state.rs`
- Pg connection/config details: `src/pginfo/conninfo.rs`
- DCS integration + cache + trust: `src/dcs/worker.rs`, `src/dcs/store.rs`, `src/dcs/state.rs`, `src/dcs/etcd_store.rs`, `src/dcs/keys.rs`
- HA decision + actions + worker loop: `src/ha/decide.rs`, `src/ha/worker.rs`, `src/ha/state.rs`, `src/ha/actions.rs`
- Process execution: `src/process/worker.rs`, `src/process/state.rs`, `src/process/jobs.rs`
- API + controller: `src/api/worker.rs`, `src/api/controller.rs`, `src/api/fallback.rs`
- Debug snapshot: `src/debug_api/worker.rs`, `src/debug_api/snapshot.rs`
- Debug snapshot projection: `src/debug_api/view.rs`
- Harness: `src/test_harness/*` (especially `mod.rs`, `binaries.rs`, `namespace.rs`, `ports.rs`, `pg16.rs`, `etcd3.rs`, `net_proxy.rs`, `auth.rs`, `tls.rs`, `ha_e2e/*`)
- Representative black-box + contract tests: `tests/bdd_api_http.rs`, `tests/bdd_state_watch.rs`, `tests/cli_binary.rs`, `tests/policy_e2e_api_only.rs`, HA e2e in `src/ha/e2e_*.rs`, `src/worker_contract_tests.rs`

### Execution steps (what to do, in order)

- [x] 1) Define the chapter contracts first
  - [x] Update `docs/src/contributors/docs-style.md` early so the rest of the rewrite has a crisp target format.
  - [x] Decide the common reusable section names (so cross-links and “adjacent subsystem connections” are consistent).

- [x] 2) Rewrite the entrypoint and map the reader journey
  - [x] Expand `docs/src/contributors/index.md` with question-driven navigation and “if you’re changing X, read Y”.
  - [x] Add cross-links to lifecycle/assurance/interfaces pages where contributors commonly need context.

- [x] 3) Rewrite the codebase map with explicit call/data paths
  - [x] Rewrite `docs/src/contributors/codebase-map.md` with:
    - module ownership table
    - startup path narrative (binary → runtime → startup planner/executor)
    - steady-state loop narrative (workers + state bus)
    - decision/action path narrative (HA decide → dispatch → process → feedback)

- [x] 4) Rewrite worker wiring as a “state bus” deep dive
  - [x] Rewrite `docs/src/contributors/worker-wiring.md` with:
    - channel ownership matrix (who publishes what, who consumes what)
    - update triggers/cadence per worker
    - failure-path semantics and how they surface in HA/API/debug state

- [x] 5) Rewrite HA pipeline as an end-to-end decision/action story
  - [x] Rewrite `docs/src/contributors/ha-pipeline.md` with:
    - input contract section (world snapshot components)
    - transition reasoning and examples tied to `src/ha/decide.rs`
    - action dispatch table tied to `src/ha/worker.rs`
    - switchover request walkthrough (API → DCS → HA → process)
    - fencing walkthrough (conflicting leader record → fence job)

- [x] 6) Expand testing system and harness internals (map to code)
  - [x] Expand `docs/src/contributors/testing-system.md` with:
    - confidence boundaries (what each layer can/cannot prove)
    - “when to add which test” decision guide
    - `make test` vs `make test-long` explanation (exact-name preflight, fail-closed semantics)
  - [x] Expand `docs/src/contributors/harness-internals.md` with:
    - module inventory (`src/test_harness/*`)
    - startup sequence and why ordering matters
    - proxy primitives and determinism controls
    - flake taxonomy + mitigations (with concrete probes/commands)

- [x] 7) Add API/debug contracts deep dive
  - [x] Add and write `docs/src/contributors/api-debug-contracts.md` (and wire into navigation).

- [x] 8) Make verification operational and cross-linked
  - [x] Expand `docs/src/contributors/verification.md` to include a concrete workflow and link to `docs/src/verification/index.md`.
  - [x] Update `docs/src/contributors/task-33-docs-verification-report.md` only if its preface/pointers are stale (avoid churn).

- [x] 9) Update navigation and ensure deep-dive coherence
  - [x] Update `docs/src/SUMMARY.md` contributor ordering for a deliberate deep technical progression.
  - [x] Update `docs/src/contributors/index.md` reading order to match `docs/src/SUMMARY.md` exactly (single source of truth).
  - [x] Add cross-links between contributor chapters so none reads in isolation.

- [x] 10) Verification pass on the docs themselves (skeptical)
  - [x] Re-scan all contributors chapters for “over-claims” and rewrite to bounded language unless supported by code/tests.
  - [x] Ensure every chapter contains an explicit “how this connects to adjacent subsystem(s)” section with concrete links.

- [x] 11) Run all required gates and fix failures (no skipping)
  - [x] `make docs-lint` (note: `make lint` includes `docs-lint`, but keep this as a fast, docs-only early checkpoint)
  - [x] `make docs-build`
  - [x] `make docs-hygiene`
  - [x] `make check`
  - [x] `make test`
  - [x] `make lint`
  - [x] `make test-long`

NOW EXECUTE
