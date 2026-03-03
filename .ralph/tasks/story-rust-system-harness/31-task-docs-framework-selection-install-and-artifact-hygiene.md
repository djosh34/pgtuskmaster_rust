---
## Task: Select docs framework, install it, and enforce artifact git hygiene <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Choose the best documentation framework for this Rust project (including VitePress as a candidate), install it, prove it renders a static HTML site correctly, and lock down strict git artifact hygiene before any docs commits.

**Scope:**
- Evaluate at least these candidates with concrete tradeoffs: VitePress, mdBook, and Docusaurus (or another Rust-friendly static docs option with justification).
- Treat VitePress as the preferred baseline because the user strongly prefers its visual polish and “beautiful docs” feel.
- Do not choose an uglier framework. Any non-VitePress choice must include hard evidence that visual quality and docs UX are at least on par with VitePress (theme quality, layout polish, navigation/search experience, readability, diagram presentation).
- Choose one framework based on maintainability, visual quality (highest priority), diagram support, search/navigation quality, and contributor workflow fit for this repository.
- Install and bootstrap the chosen framework in-repo with a clear docs project structure.
- Prove the docs site renders and builds static output end-to-end (local dev preview plus production/static build output).
- Identify actual generated artifacts/folders by running the framework (do not guess), then update `.gitignore` accordingly.
- Enforce “no generated artifacts in git” policy:
  - `node_modules` must be ignored if Node tooling is used.
  - Built outputs (`dist`, `out`, `.vitepress/dist`, `book/`, or whatever is truly produced) must be ignored based on observed output.
  - Verify ignored behavior with `git add` checks before commit.
  - If generated artifacts were staged/tracked accidentally, remove from index/history in this branch before final commit.

**Context from research:**
- User strongly prefers VitePress specifically due to its looks and wants VitePress-level beauty at minimum.
- Framework choice must preserve that aesthetic bar; selecting an “uglier” option is explicitly disallowed.
- This task must establish the technical docs platform and repository hygiene guardrails so following docs tasks can focus purely on content quality.

**Expected outcome:**
- One selected framework is installed and working, static HTML output is verified, and artifact ignore rules are proven effective with clean git status/add behavior.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Full exhaustive checklist completed with concrete module requirements: framework comparison notes committed under docs planning notes, framework decision recorded with rationale, docs scaffold added, dev server and static build commands documented and validated, rendered static output directory confirmed from real build logs
- [ ] Aesthetic gate satisfied: VitePress used by default unless a non-VitePress option is proven not uglier; if non-VitePress is chosen, decision record includes side-by-side evidence (theme/layout/navigation/diagram readability) demonstrating equal-or-better visual quality and UX
- [ ] Full exhaustive checklist completed for git hygiene: `.gitignore` updated only after observing produced artifacts, includes all generated dependency/build output (for chosen framework), `git add -n`/staging checks demonstrate artifacts are ignored, if artifacts were previously staged/tracked then they are removed from index and cleaned from branch history before final commit
- [ ] No generated docs artifacts committed (`node_modules`, build output folders, caches) and verification evidence captured in task notes
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test-bdd` — all BDD features pass
</acceptance_criteria>
