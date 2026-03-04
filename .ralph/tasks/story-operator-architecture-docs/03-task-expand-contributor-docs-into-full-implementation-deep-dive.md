---
## Task: Expand Contributor Docs into a Full Implementation Deep Dive <status>not_started</status> <passes>false</passes>

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
- [ ]  Full exhaustive checklist of all files/modules to modify with specific requirements for each
- [ ] `docs/src/contributors/index.md` is expanded into a clear deep-dive entrypoint that maps how to read the contributor docs and which chapter answers which engineering question.
- [ ] `docs/src/contributors/codebase-map.md` is rewritten to include concrete module boundaries, crate-level ownership, and explicit runtime call/data paths (startup, steady-state, decision/action loops).
- [ ] `docs/src/contributors/worker-wiring.md` explains state and message flow across workers in sequence, including channel ownership, update cadence, and failure-path behavior.
- [ ] `docs/src/contributors/ha-pipeline.md` documents HA decision internals end-to-end (inputs, decision logic, action dispatch, process feedback) with nitty-gritty transition reasoning.
- [ ] `docs/src/contributors/testing-system.md` is expanded into a full testing architecture guide covering unit/integration/BDD/real-binary layers, confidence boundaries, and when to add each test type.
- [ ] `docs/src/contributors/harness-internals.md` details harness internals deeply (resource setup, process lifecycle, networking/proxy behavior, determinism controls, and common flake sources).
- [ ] `docs/src/contributors/docs-style.md` is updated to require flowing, paragraph-first, technically dense writing for contributor chapters and to permit targeted code snippets when they improve clarity.
- [ ] `docs/src/contributors/verification.md` explains the verification workflow in concrete operational terms (claim extraction, evidence standards, and correction loop) without leaking audit noise into operator-facing docs.
- [ ] `docs/src/SUMMARY.md` contributor subsection ordering/cross-links are updated so chapter progression is coherent for deep technical reading.
- [ ] Every contributors chapter adds at least one explicit “how this connects to adjacent subsystem(s)” section that links the behavior across modules rather than describing files in isolation.
- [ ] Contributors prose is written in natural, flowing paragraphs with context-setting openings and clear transitions; no chapter remains a terse bullet-only outline.
- [ ] Code snippets are allowed only in contributor docs and are used where they clarify concrete behavior; each snippet has surrounding explanation that states why the snippet matters.
- [ ] All claims about behavior in contributor docs are aligned with current code paths and tests; over-claims are removed or rewritten to bounded language.
- [ ] `make docs-lint` passes cleanly
- [ ] `make docs-build` passes cleanly
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
