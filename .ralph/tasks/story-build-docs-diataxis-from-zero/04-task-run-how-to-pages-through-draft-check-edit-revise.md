## Task: Run How-To Pages Through K2 Draft Check Edit Revise <status>not_started</status> <passes>false</passes> <priority>high</priority>

<description>
**Goal:** Build the how-to chapter through repeated capped runs. Every how-to page must be drafted and revised by K2 under strict Diataxis how-to guidance. The task must provide operational facts and constraints, not write the page prose itself.

**Scope:**
- Work in:
  - `docs/src/`
  - `docs/drafts/`
  - `docs/src/SUMMARY.md`
- Build real task-oriented guides only from repo-backed workflows and operational goals.
- Do not turn how-to pages into tutorials, explanations, or feature catalogs.

**Validation policy for this task:**
- NEVER run tests in this task.
- You may read test files as grounding sources, but do not execute any test command.
- The only allowed validation commands in this task are `docs-lint` and `docs-build`.

**Mandatory reread before each run:**
- `.agents/skills/create-docs/references/diataxis.fr/start-here/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/compass/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/how-to-use-diataxis/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/how-to-guides/index.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/01-task-establish-diataxis-reread-and-draft-loop.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/02-task-run-reference-pages-through-draft-check-edit-revise.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/03-task-run-explanation-pages-through-draft-check-edit-revise.md`

**How-to constraints:**
- Every page must classify as `action + application`.
- Each page must contain action and only action.
- Link out for reference or explanation instead of mixing forms.

**Run requirements:**
1. Gather exact commands, flags, config snippets, checkpoints, and repo-backed expectations for the next user-goal pages. Use existing docs only as revision inputs after re-checking the facts against the repository.
2. Package those facts plus the Diataxis how-to guidance into a rich K2 context payload, using a temporary context file when needed.
3. Use a `prepare -> execute -> write` flow:
   - prepare the K2 input files first, with one prepared input per target page or materially different prompt variant
   - execute the prepared K2 generations after preparation is complete, running multiple independent K2 doc generations in parallel whenever the prepared inputs do not depend on one another
   - write the returned docs only after checking each K2 result against the prepared facts and Diataxis constraints
4. Use `ask-k2-docs` for every initial draft and every prose revision.
5. Use differing prompts when comparing multiple task sequences, stopping points, grouping options, or update strategies would improve the guide.
6. Tell K2 to use placeholders like `[diagram about switchover request flow]` for any needed diagrams.
7. Check/edit K2 output for teaching drift, explanation drift, invented steps, or catalog sprawl.
8. Use `update-docs` whenever revising an existing how-to page or `docs/src/SUMMARY.md`.
9. Draft or revise at most 10 pages in one run, then quit immediately.
10. Keep the task open across runs until all planned how-to pages and revisions are complete. Only then set `<passes>true</passes>`.

**Context to provide to K2 instead of pre-writing prose here:**
- exact operational user goals to cover next
- concrete commands, config files, endpoints, and observables grounded in the repo
- any existing draft prose only after its technical claims have been re-checked against repo sources
- links to related reference or explanation pages
- the Diataxis how-to rules that must constrain the output
</description>

<acceptance_criteria>
- [ ] Every drafted or revised how-to page is written through `ask-k2-docs`
- [ ] Every page is explicitly kept in the Diataxis how-to form
- [ ] The task text supplies repo facts and constraints instead of writing the docs prose itself
- [ ] Each run is capped at 10 docs pages and ends immediately after that capped work
- [ ] `<passes>true</passes>` is set only once the full how-to task scope is complete
</acceptance_criteria>
