## Task: Restructure Operator Docs for Better Flow, Depth, and Decision Rationale <status>done</status> <passes>true</passes>

<description>
**Goal:** Rebuild the mdBook documentation into an operator-first guide that explains not only what the system does, but why it behaves that way and which tradeoffs drive key HA decisions.

**Scope:**
- Replace fragmented short chapters with a clearer, deeper structure:
  - merge pages that are too thin to stand alone
  - expand pages that need decision context, failure reasoning, or operational implications
  - keep chapters concise, but substantial enough to stand on their own.
- Rebuild `docs/src/SUMMARY.md` around a clear progression:
  1. Start Here (what this is, what problem it solves, how it solves it, and how to navigate the rest of the docs)
  2. Quick Start (practical first setup/run/verification path before full operator depth)
  3. Operator Guide (configuration, deployment, observability, troubleshooting)
  4. System Lifecycle (bootstrap, steady state, switchover, failover, fencing/fail-safe, recovery)
  5. Architecture Assurance (invariants, decision model, tradeoffs, safety case)
  6. Contributors (code structure, worker interactions, test harness and test strategy, internal workflows)
- Add an operator-focused section under `docs/src/getting-started/`:
  - `index.md` (what this system controls, key boundaries, and document map)
  - `quickstart.md` (first successful run with expected checkpoints)
  - `initial-validation.md` (what to verify after first run before deeper operations)
- Ensure every major architecture/operations page includes:
  - a short "Why this exists" paragraph
  - a "Tradeoffs" section with practical implications
  - a "When this matters in operations" section.
- Rewrite prose for smoother narrative quality:
  - remove slash-heavy phrasing and compressed shorthand
  - use connected paragraphs with transitions and context-setting openings
  - keep terminology consistent and defined on first use.
- Enforce a no-code-docs rule for operator-facing chapters:
  - do not include programming-language code examples in docs
  - allow configuration examples and snippets where operator setup requires them
  - permit a full example configuration near the beginning of config docs, followed by deeper section-by-section explanations.
- Expand configuration documentation depth so operators can understand behavior, not only syntax:
  - present one recommended production profile before alternatives
  - for each important config field/group, document purpose, default/required behavior, and operational impact
  - explain how relevant fields map to PostgreSQL runtime behavior (for example, identity/auth/TLS/rewind/bootstrap implications)
  - include common misconfiguration patterns and their observed symptoms in terms of operator-facing errors/log signatures.
- Organize troubleshooting primarily by symptom/error case/log signature, with subsystem cross-references as secondary navigation.
- Add an explicit "Safety Case" chapter in Architecture Assurance that explains why split-brain risk is constrained and what assumptions this depends on.
- Expand the Contributors section in depth with subchapters and sub-subchapters:
  - runtime code structure and module boundaries
  - worker ownership and state propagation paths
  - decision/action flow between workers
  - test layers (unit, integration, real-binary e2e, BDD), harness internals, and failure triage workflow
  - how to safely evolve behavior with verification expectations.
- Allow intentional conceptual re-explanations across sections for readability, while keeping strict facts in canonical reference pages.
- Keep Mermaid diagrams only when they clarify runtime behavior, state transitions, or operational decision points.
- Remove Mermaid diagrams that are only visual table-of-contents or reading-order decoration.
- Move non-operator authoring/audit pages out of the default operator navigation:
  - relocate docs-authoring policy pages to contributor-only area
  - relocate verification/audit logs to contributor-only area or appendix outside default path.
- Remove obsolete redirect-stub pages from navigation once links are updated:
  - `docs/src/components.md`
  - `docs/src/glossary.md`
  - `docs/src/architecture.md`
  - `docs/src/operations.md`

**Context from research:**
- Users learn technical systems faster when docs follow intent-based content types and clear task progression (Diataxis).
- Effective operator docs combine procedural guidance with conceptual rationale, so actions are understandable under incident pressure.
- Plain-language guidance improves reliability in operations: shorter sentences, explicit verbs, and predictable structure reduce ambiguity.
- Current docs already have strong architecture correctness and useful diagrams; the main gap is depth calibration and narrative continuity for operators.

**Expected outcome:**
- An operator can read straight through the default docs path and understand:
  - what to do
  - what to look at
  - why the system chooses conservative HA actions
  - how tradeoffs affect incident decisions.
- Configuration docs make field intent and runtime effects explicit, including PostgreSQL-facing consequences for key settings.
- Quick Start provides a practical first success path before full operator depth.
- Lifecycle documentation explains bootstrap early within lifecycle context without overloading the opening orientation pages.
- Chapter length and depth feel deliberate: neither fragmented nor bloated.
- Default navigation is operator-focused, with a deep contributor section in the same book.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] `docs/src/SUMMARY.md` is restructured into an operator-first order: Start Here -> Quick Start -> Operator Guide -> System Lifecycle -> Architecture Assurance -> Interfaces -> Contributors
- [x] Start Here + Quick Start provide an orientation + first-run path (`docs/src/start-here/*`, `docs/src/quick-start/*`)
- [x] Operator Guide covers configuration, deployment/topology, observability, and troubleshooting (`docs/src/operator/*`)
- [x] Lifecycle documentation covers bootstrap, steady state, switchover, failover, fail-safe/fencing, and recovery (`docs/src/lifecycle/*`)
- [x] Architecture Assurance includes a dedicated Safety Case chapter (`docs/src/assurance/safety-case.md`)
- [x] Operator docs avoid programming-language code snippets; configuration docs may include configuration snippets where required
- [x] Contributors section contains implementation deep dives and harness/testing internals (`docs/src/contributors/*`)
- [x] Verification/audit material is not part of the default operator flow; it is discoverable under contributor/verification sections (`docs/src/verification/*`)
- [x] Docs hygiene and repo gates pass (`make check`, `make test`, `make lint`, `make test-long`)
</acceptance_criteria>
