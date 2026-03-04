---
## Task: Post-Rewrite Skeptical Claim Verification with 15+ Parallel Spark Subagents <status>not_started</status> <passes>false</passes>

<description>
**Goal:** After the operator-doc transformation is complete, run a deep, adversarial verification of every claim in the docs using many independent `spark` subagents, and resolve all mismatches before finalizing docs.

**Scope:**
- This task MUST start only after Task 01 is complete and docs structure/content are stabilized.
- Build a comprehensive claim inventory from the rewritten docs across the new structure:
  - Start Here
  - Quick Start
  - Operator Guide
  - System Lifecycle
  - Architecture Assurance
  - Interfaces
  - (Contributor sections only for implementation/process claims that are still normative)
  - extract all non-trivial claims (behavior, safety guarantees, endpoint semantics, config effects, DCS write ownership, failure behavior, startup/HA transitions, safety-case assumptions)
  - assign each claim a unique claim ID and exact location (`path:line`)
  - classify each claim type (descriptive, behavioral, invariant, absence/negative, operational expectation).
- Create a post-rewrite verification matrix (generated after rewrite, not before):
  - one row per claim with expected evidence type and verification method
  - include strict pass/fail criteria and required evidence anchors.
- Execute verification using 15+ parallel `spark` subagents with independent ownership slices.
- Keep all verification process details out of operator docs:
  - verification artifacts belong in internal task evidence and/or contributor-only verification records
  - operator-facing docs should contain final accurate content only.

**Context from research:**
- High-trust documentation requires evidence-backed claim validation, especially for architecture and operational behavior.
- Independent parallel verification reduces shared blind spots and confirmation bias.
- Negative claims and safety claims require stronger evidence standards than descriptive claims.

**Expected outcome:**
- Every operator-facing claim is either proven with code/test/runtime evidence, rewritten to bounded language, or removed.
- DCS key ownership and write-path claims are explicitly verified against implementation entry points.
- Verification is adversarial, reproducible, and explicitly skeptical.
- Final docs are accurate without exposing internal verification mechanics in the operator reading flow.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ]  Full exhaustive checklist of all files/modules to modify with specific requirements for each
- [ ] Task execution starts only after Task 01 completion and rewritten docs freeze point is recorded
- [ ] A full post-rewrite claim inventory exists with claim IDs and exact `path:line` anchors
- [ ] A verification matrix is generated after rewrite (not before), with per-claim evidence requirements and pass/fail rules
- [ ] At least 15 `spark` subagents run in parallel, each with a disjoint claim slice and explicit ownership
- [ ] Every subagent receives precise instructions to be maximally skeptical: assume docs/comments can be wrong, trust only code/tests/runtime evidence
- [ ] Every subagent instruction includes scoped verification bullets covering:
  - exact claims to verify
  - specific section coverage in the new IA (Start Here, Quick Start, Operator, Lifecycle, Assurance, Interfaces)
  - required code paths/symbols/tests/runtime checks
  - forbidden weak evidence (for example, unverified comments or second-hand doc statements)
  - handling for uncertain/ambiguous findings
  - required evidence output format
- [ ] Each claim outcome is one of: `verified`, `rewritten`, `removed`, `uncertain-with-followup`
- [ ] Absence/negative claims are accepted only with explicit guards/tests; otherwise rewritten to bounded wording or removed
- [ ] Conflicting subagent conclusions are adjudicated and resolved with final evidence-backed disposition
- [ ] Verification artifacts remain outside operator docs; operator docs show only corrected final content
- [ ] `make docs-lint` passes cleanly after all rewrites
- [ ] `make docs-build` passes cleanly after all rewrites
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
