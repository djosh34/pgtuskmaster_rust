## Task: Run Explanation Pages Through K2 Draft Check Edit Revise <status>not_started</status> <passes>false</passes> <priority>high</priority>

<description>
**Goal:** Build the explanation chapter through repeated capped runs. Every explanation page must be drafted and revised by K2 under strict Diataxis explanation guidance. The task must provide source context, tensions, and grounding, not hand-write the explanation prose.

**Scope:**
- Work in:
  - `docs/src/`
  - `docs/drafts/`
  - `docs/src/SUMMARY.md`
- Build real explanation pages only where the codebase supports meaningful context and rationale.
- Do not turn explanation pages into reference dumps or how-to procedures.

**Validation policy for this task:**
- NEVER run tests in this task.
- You may read test files as grounding sources, but do not execute any test command.
- The only allowed validation commands in this task are `docs-lint` and `docs-build`.

**Mandatory reread before each run:**
- `.agents/skills/create-docs/references/diataxis.fr/start-here/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/compass/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/how-to-use-diataxis/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/explanation/index.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/01-task-establish-diataxis-reread-and-draft-loop.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/02-task-run-reference-pages-through-draft-check-edit-revise.md`

**Explanation constraints:**
- Every page must classify as `cognition + acquisition`.
- Explanation should provide context, reasons, tradeoffs, alternatives, and consequences.
- If the page turns procedural or catalog-like, split or replace it.

**Run requirements:**
1. Gather the relevant code paths, tests, design tensions, cross-links, and runnable behavior before drafting. Use existing docs only as revision inputs after re-checking their claims against the repository.
2. Package those facts plus the Diataxis explanation guidance into rich K2 context, using a temporary context file when helpful.
3. Use `ask-k2-docs` for all explanation-page prose drafts and prose revisions.
4. Use meaningfully different prompts when comparing alternative explanatory frames, structures, or update strategies.
5. Tell K2 to leave diagram placeholders such as `[diagram about DCS trust inputs]` instead of inventing diagrams.
6. Check/edit K2 output for factual invention, shallow hand-waving, procedural drift, or reference dumping.
7. Use `update-docs` for revisions to existing explanation pages or to `docs/src/SUMMARY.md`.
8. Draft or revise at most 3 pages in a single run, then quit immediately.
9. Keep `<passes>false</passes>` until the whole explanation task scope is complete across however many runs are needed.

**Context to provide to K2 instead of pre-writing prose here:**
- exact modules, configs, tests, and control flows relevant to each topic
- any existing draft prose only after its technical claims have been re-checked against repo sources
- tensions or design questions the page should illuminate
- links to related reference or how-to pages
- the Diataxis explanation rules that must constrain the output
</description>

<acceptance_criteria>
- [ ] Every drafted or revised explanation page is written through `ask-k2-docs`
- [ ] Every page is explicitly kept in the Diataxis explanation form
- [ ] The task text supplies grounding context and constraints instead of writing the docs prose itself
- [ ] Each run is capped at 3 docs pages and ends immediately after that capped work
- [ ] `<passes>true</passes>` is set only once the full explanation task scope is complete
</acceptance_criteria>
