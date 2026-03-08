## Task: Derive Navigation From Authored Pages With K2 Overviews <status>not_started</status> <passes>false</passes> <priority>high</priority>

<description>
**Goal:** Derive mdBook navigation and any needed landing or overview pages from real authored docs. Any new overview prose must be drafted and revised by K2 under Diataxis guidance. The task must provide navigation facts and content relationships, not hand-write the overview prose itself.

**Scope:**
- Work in:
  - `docs/src/`
  - `docs/drafts/`
  - `docs/src/SUMMARY.md`
- Rework navigation only from real existing pages.
- If landing pages are needed, they must be real overviews, not placeholders.

**Validation policy for this task:**
- NEVER run tests in this task.
- You may read test files as grounding sources when needed, but do not execute any test command.
- The only allowed validation commands in this task are `docs-lint` and `docs-build`.

**Mandatory reread before each run:**
- `.agents/skills/create-docs/references/diataxis.fr/start-here/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/compass/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/how-to-use-diataxis/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/complex-hierarchies/index.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/01-task-establish-diataxis-reread-and-draft-loop.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/02-task-run-reference-pages-through-draft-check-edit-revise.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/03-task-run-explanation-pages-through-draft-check-edit-revise.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/04-task-run-how-to-pages-through-draft-check-edit-revise.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/05-task-run-tutorial-pages-through-draft-check-edit-revise.md`

**Navigation constraints:**
- Navigation must emerge from real pages, not speculative structure.
- If overview pages are added, they must stay within an appropriate Diataxis form.
- Do not add empty buckets.

**Run requirements:**
1. Review the current authored pages and the content relationships they imply. Treat any existing overview prose as non-authoritative revision input only, and re-check every technical claim that must appear in overview prose against the repository instead of trusting prior prose.
2. Build rich K2 context from those real pages, the navigation problem, and the relevant Diataxis guidance. Use a temporary context file whenever that helps.
3. Use a `prepare -> execute -> write` flow:
   - prepare ALL prompt files first under `docs/tmp/prompts/`, with one prepared prompt file per target page or materially different prompt variant
   - prepare 10 prompt files for the run unless fewer than 10 independent navigation-page, overview-page, or variant prompts are genuinely possible from the verified live scope
   - each prepared prompt file must contain the full execution prompt, including the exact instructions, Diataxis constraints, and any raw repo files or excerpts that need to be appended verbatim for grounding
   - execute only after the full prompt-file set is prepared, piping those prepared prompt files into K2 in parallel whenever they do not depend on one another
   - write the returned docs only after checking each K2 result against the prepared facts and Diataxis constraints
4. Use `ask-k2-docs` for any landing-page or overview prose and for prose revisions to those pages.
5. Use differing prompts when comparing alternative navigation models, overview structures, or continuous update strategies would improve the result.
6. Tell K2 to leave placeholders like `[diagram about docs map]` for any diagram needs.
7. Use `update-docs` for every revision to existing overview pages and for `docs/src/SUMMARY.md`.
8. Draft or revise at most 10 docs pages in one run, then quit immediately.
9. Keep the task open across runs until navigation and any needed overviews are actually complete. Only then set `<passes>true</passes>`.

**Context to provide to K2 instead of pre-writing prose here:**
- the real pages that currently exist
- the user-facing grouping or entry problems those pages create
- the intended overview role for any landing page
- any existing overview prose only as non-authoritative revision input after its technical claims have been re-checked against repo sources
- the Diataxis constraints that must shape the output
</description>

<acceptance_criteria>
- [ ] Every new or revised overview page is written through `ask-k2-docs`
- [ ] Navigation and overview work is derived only from real authored pages
- [ ] The task text supplies structure/context inputs instead of writing the docs prose itself
- [ ] Each run is capped at 10 docs pages and ends immediately after that capped work
- [ ] `<passes>true</passes>` is set only once the full navigation task scope is complete
</acceptance_criteria>
