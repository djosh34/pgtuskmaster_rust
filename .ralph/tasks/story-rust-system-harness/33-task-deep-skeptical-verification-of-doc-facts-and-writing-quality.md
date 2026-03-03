---
## Task: Perform deep skeptical verification of all docs facts and writing quality <status>not_started</status> <passes>false</passes>

<blocked_by>32-task-author-complete-architecture-docs-with-diagrams-and-no-code</blocked_by>

<description>
**Goal:** Rigorously validate every documentation claim against the real codebase and enforce a hard editorial quality gate that rejects overloaded, vague, or misleading writing.

**Scope:**
- Treat all docs as potentially wrong until proven correct from source of truth in this repository.
- Perform line-by-line claim validation:
  - map each architecture statement to concrete source files/modules/tests/configs
  - flag and correct every mismatch, overstatement, outdated claim, and ambiguous phrasing
- Apply an aggressive writing-quality review:
  - remove overloaded/excessively dense sections
  - replace hand-wavy statements with precise architecture truth
  - enforce consistent terminology across all docs
  - ensure newcomer readability and progressive explanation depth
- Validate diagram correctness against real system behavior and component boundaries.
- Produce a verification report/checklist documenting what was checked and what was fixed.
- If uncertain on any claim, resolve uncertainty through direct code inspection before keeping text.

**Context from research:**
- User requested a deeply skeptical pass that assumes docs are wrong by default and judges writing harshly when reader cognitive load is high.
- This is a standards-and-truth gate: content quality is not “done” until facts and writing are both proven strong.

**Expected outcome:**
- Documentation that is fact-checked against current code, internally consistent, readable, and aligned to architecture documentation standards.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Full exhaustive checklist completed with concrete verification requirements: every major factual claim mapped to source evidence, every page reviewed for terminology consistency, every diagram validated against real component behavior, all identified inaccuracies corrected
- [ ] Dedicated verification artifact added (checklist/report) showing skeptical review method, evidence references, and resolved issues
- [ ] Writing quality gate passed: no overloaded “wall of jargon” sections, no vague causal statements, no contradictory terminology, no architecture claims without repository evidence
- [ ] Any unresolved uncertainty is explicitly called out and tracked as follow-up work (no silent assumptions)
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test-long` — ultra-long suite passes; if any failure appears here, create a new shorter real-binary e2e regression that reproduces it
</acceptance_criteria>
