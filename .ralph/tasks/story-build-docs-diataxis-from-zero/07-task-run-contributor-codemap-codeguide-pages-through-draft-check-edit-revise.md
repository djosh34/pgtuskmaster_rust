## Task: Run Contributor Codemap Codeguide Pages Through K2 Draft Check Edit Revise <status>not_started</status> <passes>false</passes> <priority>high</priority>

<description>
**Goal:** Build contributor-focused codemap and codeguide docs through repeated capped runs. Every contributor page must be drafted and revised by K2 under the correct Diataxis form. The task must provide codebase context, audience needs, and constraints, not hand-write the docs prose itself.

**Scope:**
- Work in:
  - `docs/src/`
  - `docs/drafts/`
  - `docs/src/SUMMARY.md`
- Build a contributor chapter only from real pages that exist.
- Keep each page within a single Diataxis form even though the chapter serves contributors.

**Validation policy for this task:**
- NEVER run tests in this task.
- You may read test files as grounding sources, but do not execute any test command.
- The only allowed validation commands in this task are `docs-lint` and `docs-build`.

**Mandatory reread before each run:**
- `.agents/skills/create-docs/references/diataxis.fr/start-here/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/compass/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/how-to-use-diataxis/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/complex-hierarchies/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/explanation/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/reference/index.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/01-task-establish-diataxis-reread-and-draft-loop.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/02-task-run-reference-pages-through-draft-check-edit-revise.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/03-task-run-explanation-pages-through-draft-check-edit-revise.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/04-task-run-how-to-pages-through-draft-check-edit-revise.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/05-task-run-tutorial-pages-through-draft-check-edit-revise.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/06-task-derive-navigation-from-authored-pages.md`

**Contributor-doc constraints:**
- Every page must be classified with the Diataxis compass.
- Reference pages must describe and only describe.
- Explanation pages must provide context and reasons without turning procedural.
- Keep the wording rule: do not use `gate` where `test` is meant.

**Run requirements:**
1. Gather the next contributor topics directly from code structure, module boundaries, runtime flows, ownership boundaries, tests, and runnable behavior where relevant. Use existing docs only as revision inputs after re-checking their claims against the repository.
2. Build rich K2 context from those repo facts, the contributor audience, and the relevant Diataxis form guidance. Use a temporary context file whenever that is clearer.
3. Use a `prepare -> execute -> write` flow:
   - prepare ALL prompt files first under `docs/tmp/prompts/`, with one prepared prompt file per target page or materially different prompt variant
   - prepare 10 prompt files for the run unless fewer than 10 independent contributor-page or variant prompts are genuinely possible from the verified live scope
   - each prepared prompt file must contain the full execution prompt, including the exact instructions, Diataxis constraints, and any raw repo files or excerpts that need to be appended verbatim for grounding
   - execute only after the full prompt-file set is prepared, piping those prepared prompt files into K2 in parallel whenever they do not depend on one another
   - write the returned docs only after checking each K2 result against the prepared facts and Diataxis constraints
4. Use `ask-k2-docs` for every draft and every prose revision.
5. Use differing prompts when comparing alternative codemap structures, audience framing, page splits, or update strategies would improve the chapter.
6. Tell K2 to leave placeholders like `[diagram about module ownership map]` instead of inventing diagrams.
7. Check/edit K2 output for invented facts, vague architecture prose, mixed forms, or the forbidden `gate` wording.
8. Use `update-docs` whenever revising an existing contributor page or `docs/src/SUMMARY.md`.
9. Draft or revise at most 10 pages in one run, then quit immediately.
10. Keep `<passes>false</passes>` until the whole contributor-doc scope is complete across however many runs are needed.

**Context to provide to K2 instead of pre-writing prose here:**
- exact code paths, modules, and tests that ground each page
- intended contributor audience and page purpose
- any existing draft prose only after its technical claims have been re-checked against repo sources
- the required Diataxis form for each page
- wording and terminology constraints that must shape the output
</description>

<acceptance_criteria>
- [ ] Every drafted or revised contributor page is written through `ask-k2-docs`
- [ ] Every page is explicitly kept within a single Diataxis form
- [ ] The task text supplies codebase context and constraints instead of writing the docs prose itself
- [ ] Each run is capped at 10 docs pages and ends immediately after that capped work
- [ ] `<passes>true</passes>` is set only once the full contributor-doc task scope is complete
</acceptance_criteria>
