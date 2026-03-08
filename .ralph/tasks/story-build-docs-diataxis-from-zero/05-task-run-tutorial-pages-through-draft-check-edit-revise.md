## Task: Run Tutorial Pages Through K2 Draft Check Edit Revise <status>not_started</status> <passes>false</passes> <priority>high</priority>

<description>
**Goal:** Build the tutorial chapter through repeated capped runs. Every tutorial page must be drafted and revised by K2 under strict Diataxis tutorial guidance. The task must provide the learner path, guardrails, and source facts, not hand-write the tutorial prose.

**Scope:**
- Work in:
  - `docs/src/`
  - `docs/drafts/`
  - `docs/src/SUMMARY.md`
- Build real tutorials only where the repo supports a safe, concrete learning path.
- Do not let tutorial pages branch into how-to choice trees or explanation-heavy lectures.

**Validation policy for this task:**
- NEVER run tests in this task.
- You may read test files as grounding sources, but do not execute any test command.
- The only allowed validation commands in this task are `docs-lint` and `docs-build`.

**Mandatory reread before each run:**
- `.agents/skills/create-docs/references/diataxis.fr/start-here/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/compass/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/how-to-use-diataxis/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/tutorials/index.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/01-task-establish-diataxis-reread-and-draft-loop.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/02-task-run-reference-pages-through-draft-check-edit-revise.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/03-task-run-explanation-pages-through-draft-check-edit-revise.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/04-task-run-how-to-pages-through-draft-check-edit-revise.md`

**Tutorial constraints:**
- Every page must classify as `action + acquisition`.
- The learner path must be concrete, carefully managed, and reliable.
- Minimize explanation and avoid unnecessary branches.

**Run requirements:**
1. Gather the exact learner journey, prerequisites, commands, checkpoints, safe stopping points, and runnable behavior from the repo. Use existing docs only as revision inputs after re-checking the facts against the repository.
2. Package those facts plus the Diataxis tutorial guidance into rich K2 context, using a temporary context file whenever that makes the prompt clearer.
3. Use a `prepare -> execute -> write` flow:
   - prepare the K2 input files first, with one prepared input per target page or materially different prompt variant
   - execute the prepared K2 generations after preparation is complete, running multiple independent K2 doc generations in parallel whenever the prepared inputs do not depend on one another
   - write the returned docs only after checking each K2 result against the prepared facts and Diataxis constraints
4. Use `ask-k2-docs` for every tutorial draft and every prose revision.
5. Use differing prompts when comparing lesson structure, learner pacing, or update strategies would improve the tutorial.
6. Tell K2 to leave placeholders like `[diagram about tutorial environment layout]` instead of inventing diagrams.
7. Check/edit K2 output for branching how-to drift, lecture drift, invented steps, or unstable assumptions.
8. Use `update-docs` whenever revising an existing tutorial page or `docs/src/SUMMARY.md`.
9. Draft or revise at most 10 pages in one run, then quit immediately.
10. Keep the task open until the tutorial sequence for this story is actually complete. Only then set `<passes>true</passes>`.

**Context to provide to K2 instead of pre-writing prose here:**
- the concrete learner outcomes to achieve
- exact repo-backed commands, assets, configs, and checkpoints
- any existing draft prose only after its technical claims have been re-checked against repo sources
- links to supporting reference or explanation pages
- the Diataxis tutorial rules that must constrain the output
</description>

<acceptance_criteria>
- [ ] Every drafted or revised tutorial page is written through `ask-k2-docs`
- [ ] Every page is explicitly kept in the Diataxis tutorial form
- [ ] The task text supplies repo facts and learner constraints instead of writing the docs prose itself
- [ ] Each run is capped at 10 docs pages and ends immediately after that capped work
- [ ] `<passes>true</passes>` is set only once the full tutorial task scope is complete
</acceptance_criteria>
