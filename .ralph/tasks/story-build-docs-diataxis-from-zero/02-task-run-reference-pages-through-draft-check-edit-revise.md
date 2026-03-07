## Task: Run Reference Pages Through K2 Draft Check Edit Revise <status>not_started</status> <passes>false</passes> <priority>high</priority>

<description>
**Goal:** Build the reference chapter through repeated capped runs. Every reference page must be drafted and revised by K2 under strict Diataxis reference guidance. The agent must gather facts and shape prompts, not pre-write the reference prose in this task file.

**Scope:**
- Work in:
  - `docs/src/`
  - `docs/drafts/`
  - `docs/src/SUMMARY.md`
- Build real reference pages only from actual machinery boundaries.
- Do not add empty sections or speculative placeholders.

**Mandatory reread before each run:**
- `.agents/skills/create-docs/references/diataxis.fr/start-here/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/compass/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/how-to-use-diataxis/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/reference/index.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/01-task-establish-diataxis-reread-and-draft-loop.md`

**Reference constraints:**
- Every page must classify as `cognition + application`.
- Reference must `describe and only describe`.
- Structure should mirror the machinery, not a guessed user journey.
- If explanation or procedure appears necessary, link to the right page type instead of mixing forms.

**Run requirements:**
1. Gather the concrete facts for the next candidate pages directly from code, config, tests, CLI/API surfaces, and existing docs references.
2. Build rich K2 prompt context from those facts and from the Diataxis reference guidance. Use a temporary context file whenever that is the clearest way to pass enough material.
3. Use `ask-k2-docs` for every initial reference draft and every prose revision.
4. Use differing prompts when comparing alternative page structures, coverage splits, or update strategies would be useful.
5. Ask K2 for prose and structure only. If a diagram would help, instruct it to emit a placeholder such as `[diagram about runtime config loading]`.
6. Check/edit K2 output for invented facts, explanation drift, procedural drift, or duplicated pages.
7. Use `update-docs` when revising an existing reference page or `docs/src/SUMMARY.md`.
8. Draft or revise at most 3 pages in a single run, then quit immediately.
9. Keep the task open across runs until the reference chapter for this story is complete. Only then set `<passes>true</passes>`.

**Context to provide to K2 instead of pre-writing prose here:**
- current reference-page targets and any already-authored replacements
- exact source modules and tests that ground each page
- required boundaries, terms, and non-goals
- the Diataxis reference rules that must constrain the output
</description>

<acceptance_criteria>
- [ ] Every drafted or revised reference page is written through `ask-k2-docs`
- [ ] Every page is explicitly kept in the Diataxis reference form
- [ ] The task text supplies context sources and constraints instead of writing the docs prose itself
- [ ] Each run is capped at 3 docs pages and ends immediately after that capped work
- [ ] `<passes>true</passes>` is set only once the full reference task scope is complete
</acceptance_criteria>
