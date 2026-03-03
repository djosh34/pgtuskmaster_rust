---
## Task: Install mdBook docs framework and enforce artifact git hygiene <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Use mdBook for this Rust project, install it, prove it renders a static HTML site correctly, and lock down strict git artifact hygiene before any docs commits.

**Scope:**
- No framework research or comparison is required for this task.
- The framework choice is fixed: mdBook must be used.
- Install and bootstrap mdBook in-repo with a clear docs project structure.
- Prove the docs site renders and builds static output end-to-end (local dev preview plus production/static build output).
- Identify actual generated artifacts/folders by running the framework (do not guess), then update `.gitignore` accordingly.
- Enforce “no generated artifacts in git” policy:
  - `node_modules` must be ignored if Node tooling is used.
  - Built outputs (`book/`, `.mdbook/`, or whatever is truly produced) must be ignored based on observed output.
  - Verify ignored behavior with `git add` checks before commit.
  - If generated artifacts were staged/tracked accidentally, remove from index/history in this branch before final commit.

**Context from research:**
- User decision is final: mdBook is required for this task.
- Do not spend time on framework selection research in this task.
- This task should establish the mdBook platform and repository hygiene guardrails so following docs tasks can focus purely on content quality.

**Expected outcome:**
- mdBook is installed and working, static HTML output is verified, and artifact ignore rules are proven effective with clean git status/add behavior.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible, but skip framework research/selection.
</description>

<acceptance_criteria>
- [ ] Full exhaustive checklist completed with concrete module requirements: mdBook scaffold added, dev server and static build commands documented and validated, rendered static output directory confirmed from real build logs, and mdBook choice recorded as a fixed requirement (no comparison research)
- [ ] Full exhaustive checklist completed for git hygiene: `.gitignore` updated only after observing produced artifacts, includes all generated dependency/build output (for chosen framework), `git add -n`/staging checks demonstrate artifacts are ignored, if artifacts were previously staged/tracked then they are removed from index and cleaned from branch history before final commit
- [ ] No generated docs artifacts committed (`node_modules`, build output folders, caches) and verification evidence captured in task notes
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test-bdd` — all BDD features pass
</acceptance_criteria>
