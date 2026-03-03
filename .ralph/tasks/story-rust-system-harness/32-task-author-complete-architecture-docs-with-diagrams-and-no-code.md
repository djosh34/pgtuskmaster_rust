---
## Task: Author full architecture documentation with rich diagrams and zero code-level narration <status>not_started</status> <passes>false</passes>

<blocked_by>31-task-docs-framework-selection-install-and-artifact-hygiene</blocked_by>

<description>
**Goal:** Create complete, human-flowing architecture documentation for the full system using the chosen framework, with diagram-first explanations and no implementation-level code discussion.

**Scope:**
- Produce documentation for first-time readers who have never seen the repository.
- Cover top-level architecture, subsystem responsibilities, runtime/control/data flow, failure/recovery model, deployment/testing mental model, and operational interaction surfaces.
- Explain behavior in component-interaction terms (for example: “component X reacts when component Y emits/changes Z”), not function signatures or code argument details.
- Include substantial diagrams throughout (for example Mermaid/PlantUML/embedded SVG) to clarify:
  - system context
  - container/deployment topology
  - control loops/state transitions
  - request/response and event flow
  - failover and recovery paths
- Keep writing natural, editorial, and coherent (HashiCorp-docs-level readability and structure), while staying technically grounded in this codebase.
- Explicitly forbid code dumps and low-level API/argument walkthrough prose in the architecture docs.

**Context from research:**
- User requested “VitePress-level beauty” and highly readable docs with heavy diagram support.
- User requested architecture-oriented docs only: no nitty-gritty code details, no signature-driven narrative, and strong newcomer orientation.

**Expected outcome:**
- A polished, navigable docs set that lets a new engineer understand how the system works end-to-end at architecture level, with consistent diagrams and high-quality prose.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Full exhaustive checklist completed with concrete file/module requirements: docs IA/navigation pages added, architecture overview page added, subsystem pages added, runtime and failover behavior pages added, operational/testing mental model pages added, glossary/concepts page added
- [ ] Each major section includes at least one meaningful diagram and diagrams are consistent with actual component boundaries and behavior
- [ ] Writing quality bar met: natural flow, minimal jargon overload, reader-first explanations, and explicitly architecture-focused content with no function-signature or argument-level narrative
- [ ] “No code in architecture docs” rule enforced: no code blocks except optional tiny config/CLI examples if absolutely required for orientation, and those must not dominate content
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test-bdd` — all BDD features pass
</acceptance_criteria>
