## Task: Run Reference Pages Through K2 Draft Check Edit Revise <status>in_progress</status> <passes>true</passes> <priority>high</priority>

<description>
**Goal:** Build the reference chapter through repeated capped runs. Every reference page must be drafted and revised by K2 under strict Diataxis reference guidance. The agent must gather facts and shape prompts, not pre-write the reference prose in this task file.

**Scope:**
- Work in:
  - `docs/src/`
  - `docs/drafts/`
  - `docs/src/SUMMARY.md`
- Build real reference pages only from actual machinery boundaries.
- Do not add empty sections or speculative placeholders.

**Validation policy for this task:**
- NEVER run tests in this task.
- You may read test files as grounding sources, but do not execute any test command.
- The only allowed validation commands in this task are `docs-lint` and `docs-build`.

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
1. Gather the concrete facts for the next candidate pages directly from code, config, tests, CLI/API surfaces, and runnable behavior. Use existing docs only as revision inputs after re-checking the facts against the repository.
2. Build rich K2 prompt context from those facts and from the Diataxis reference guidance. Use a temporary context file whenever that is the clearest way to pass enough material.
3. Use a `prepare -> execute -> write` flow:
   - prepare ALL prompt files first under `docs/tmp/prompts/`, with one prepared prompt file per target page or materially different prompt variant
   - prepare 10 prompt files for the run unless fewer than 10 independent reference-page or variant prompts are genuinely possible from the verified live scope
   - each prepared prompt file must contain the full execution prompt, including the exact instructions, Diataxis constraints, and any raw repo files or excerpts that need to be appended verbatim for grounding
   - execute only after the full prompt-file set is prepared, piping those prepared prompt files into K2 in parallel whenever they do not depend on one another
   - write the returned docs only after checking each K2 result against the prepared facts and Diataxis constraints
4. Use `ask-k2-docs` for every initial reference draft and every prose revision.
5. Use differing prompts when comparing alternative page structures, coverage splits, or update strategies would be useful.
6. Ask K2 for prose and structure only. If a diagram would help, instruct it to emit a placeholder such as `[diagram about runtime config loading]`.
7. Check/edit K2 output for invented facts, explanation drift, procedural drift, or duplicated pages.
8. Use `update-docs` when revising an existing reference page or `docs/src/SUMMARY.md`.
9. Draft or revise at most 10 pages in a single run, then quit immediately.
10. Keep the task open across runs until the reference chapter for this story is complete. Only then set `<passes>true</passes>`.

**Context to provide to K2 instead of pre-writing prose here:**
- current reference-page targets and any already-authored replacements
- exact source modules and tests that ground each page
- any existing draft prose only after its technical claims have been re-checked against repo sources
- required boundaries, terms, and non-goals
- the Diataxis reference rules that must constrain the output
</description>

<acceptance_criteria>
 - [x] Every drafted or revised reference page is written through `ask-k2-docs`
 - [x] Every page is explicitly kept in the Diataxis reference form
 - [x] The task text supplies context sources and constraints instead of writing the docs prose itself
 - [x] Each run is capped at 10 docs pages and ends immediately after that capped work
- [x] `<passes>true</passes>` is set only once the full reference task scope is complete
</acceptance_criteria>

NOW EXECUTE: PO requires no planning for this task. Re-read the mandatory sources, prepare all prompt files under `docs/tmp/prompts/`, target 10 prepared prompts unless fewer than 10 are genuinely possible from the verified live scope, then pipe the prepared prompt files into K2 in parallel and write only the checked results into the proper markdown destinations.
