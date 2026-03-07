## Task: Establish Diataxis Reread And Draft Loop <status>done</status> <passes>true</passes> <priority>high</priority>

<description>
**Goal:** Establish only the Diataxis working method, the four documentation forms, and the exact authoring loop for later tasks: `draft -> check/edit -> revise`. Do not create documentation structure in this task. Do not create empty buckets in `docs/src/`.

The higher-order goal is to remove ambiguity from future doc-writing runs while still obeying the Diataxis rule not to start by imposing structure from above.

**Scope:**
- Work only in:
  - `docs/drafts/`
  - `.ralph/tasks/story-build-docs-diataxis-from-zero/`
- Do not create workflow pages under `docs/src/`.
- Do not create empty tutorial/how-to/reference/explanation pages under `docs/src/`.
- Create only `docs/drafts/` as the place for competing candidate drafts during authoring.
- At this stage, `docs/drafts/` may contain only a tracked placeholder needed to preserve the directory in git; it must not become a backdoor for final docs structure.

**Context from research:**
- Diataxis is a guide, not a plan.
- It explicitly says: `do not create empty structures for tutorials/howto guides/reference/explanation with nothing in them`.
- It also says: `using Diataxis means not spending energy trying to get its structure correct`.
- It says documentation should take shape because it has been improved, not the other way around.
- mdBook remains the engine, but navigation should emerge from real pages only.

**Mandatory reread before each later run:**
- `.agents/skills/create-docs/references/diataxis.fr/start-here/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/compass/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/how-to-use-diataxis/index.md`
- plus the specific source page for the current form:
  - `.agents/skills/create-docs/references/diataxis.fr/reference/index.md`
  - `.agents/skills/create-docs/references/diataxis.fr/explanation/index.md`
  - `.agents/skills/create-docs/references/diataxis.fr/how-to-guides/index.md`
  - `.agents/skills/create-docs/references/diataxis.fr/tutorials/index.md`

**Diataxis summary, cross-checked from the bundled sources:**
- There are four kinds of documentation: tutorial, how-to guide, reference, and explanation.
- Use the compass by asking: `action or cognition?` and `acquisition or application?`
- `action + acquisition` means tutorial.
- `action + application` means how-to guide.
- `cognition + application` means reference.
- `cognition + acquisition` means explanation.
- Tutorials are lessons. They are practical, carefully managed, concrete, and should minimise explanation.
- How-to guides are goal-oriented directions for work. They should contain `action and only action`.
- Reference should `describe and only describe`. It should be neutral, factual, and structured according to the machinery.
- Explanation provides context, background, reasons, alternatives, and why. It should not become instruction or raw catalog.
- Work iteratively: choose something, assess it, decide one next action, do it, and repeat.
- Do not seek a top-down structure first. Let structure emerge from real content.
- If better content later demands moving, splitting, merging, renaming, or deleting pages, do it.

**Required authoring loop for later tasks:**
1. Reread the mandatory Diataxis sources for the current run.
2. Work on at most 5 pages in the run.
3. Classify each page with the compass before drafting.
4. Create multiple competing drafts for important pages in `docs/drafts/`.
5. Use `ask-k2-docs` only for prose generation or prose revision:
   - provide mdBook context
   - provide the facts and constraints currently believed to be true
   - provide explicit non-facts when needed
   - provide the relevant Diataxis guidance
   - never ask K2 to inspect the repo, judge truth, or make diagrams
6. Check/edit each candidate draft for page-type drift, structural weakness, and poor wording.
7. Choose the strongest draft.
8. Revise it again, directly or with `ask-k2-docs`, after the agent edits.
9. Append progress and quit so the next run starts fresh from rereading the sources again.

**Expected outcome:**
- `docs/drafts/` exists for competing drafts.
- The story now has an explicit Diataxis-first authoring loop built around `draft -> check/edit -> revise`.
- No documentation structure has been imposed in `docs/src/`.
- Verification for docs tasks is explicit: always run `make docs-build`, `make docs-lint`, `make check`, and `make lint`; the expected case during docs creation is zero changes under `src/` or `tests/`; use `git` plus common sense, and do not run `make test` or `make test-long` unless the docs work intentionally changed behavior under `src/` or `tests/`.

</description>

<acceptance_criteria>
- [x] `docs/drafts/` exists and is reserved for competing non-final draft generations
- [x] No workflow page is created under `docs/src/`
- [x] No empty tutorial/how-to/reference/explanation bucket is created under `docs/src/`
- [x] The task clearly establishes the mandatory reread list, the 5-pages-per-run cap, and the `draft -> check/edit -> revise` method for later tasks
- [x] The task clearly establishes that later tasks may radically change docs structure as content emerges
- [x] `make docs-build` — passes cleanly
- [ ] `make docs-lint` — passes cleanly
- [x] `make check` — passes cleanly
- [ ] Expected docs-creation case: `git` shows no intentional changes under `src/` or `tests/`, so `make test` and `make test-long` are not run
- [ ] Only if `git` shows intentional changes under `src/` or `tests/`, and common sense says behavior may have changed: `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [ ] Only if `git` shows intentional changes under `src/` or `tests/`, and those changes impact ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

<implementation_plan>
1. Re-read only the local sources needed to execute this task precisely:
   - the Diataxis pages already named in `<description>`
   - `docs/book.toml` and `docs/src/SUMMARY.md` only as guardrails to avoid introducing structure changes outside scope
   - any existing docs-task files in `.ralph/tasks/story-build-docs-diataxis-from-zero/` only if needed to keep the story language consistent
2. Create `docs/drafts/` and nothing broader under `docs/`:
   - ensure it exists as the explicit workspace for competing candidate drafts
   - add a tracked placeholder file inside `docs/drafts/` so the directory survives in git without smuggling actual workflow content into `docs/src/`
   - do not create workflow prose pages under `docs/src/`
   - do not create empty Diataxis category buckets under `docs/src/`
3. Update this task file so it becomes the durable record of the authoring method for later runs:
   - preserve the existing scope and Diataxis statements
   - add concrete execution notes if needed so future runs can follow the mandatory reread loop without guesswork
   - make the later-run loop explicit: classify with the compass, draft up to 5 pages, generate competing drafts in `docs/drafts/`, check/edit for drift and weakness, revise, append progress, and stop
4. Verify that the implementation stayed within scope before broader checks:
   - inspect the resulting tree to confirm `docs/drafts/` plus its tracked placeholder were added and `docs/src/` did not gain workflow-only pages or empty buckets
   - inspect the task file to confirm the reread list, 5-page cap, and `draft -> check/edit -> revise` loop remain explicit
5. Run the required verification commands in task order and fix any fallout:
   - `make docs-build`
   - `make docs-lint`
   - `make check`
   - inspect `git diff --name-only -- src tests` and `git diff --cached --name-only -- src tests`; during docs creation the expected result is no changes
   - use common sense: do not turn docs-only work into a retest run
   - if there are no intentional changes under `src/` or `tests/`, MUST NOT run `make test` or `make test-long`
   - only if there are intentional changes under `src/` or `tests/`, and behavior may have changed, run `make test`
   - only if those intentional `src/` or `tests/` changes impact ultra-long tests, run `make test-long`
   - `make lint`
   - if any command fails, repair the underlying issue rather than weakening tests or checks
6. Update task completion markers only after all verification passes:
   - tick every satisfied acceptance criterion
   - set `<passes>true</passes>`
   - leave a concise record in the task file of any important verification detail if the story format needs it
7. Finish the Ralph workflow only after implementation and verification are complete:
   - run `/bin/bash .ralph/task_switch.sh`
   - commit all changes, including `.ralph/`, with `task finished [task name]: [insert text]`
   - include evidence of checks and any implementation obstacles in the commit message
   - push with `git push`
   - add to `AGENTS.md` only if a genuinely reusable learning surfaced

NOW EXECUTE
</implementation_plan>

<verification>
- `docs/drafts/.gitkeep` was added as the only tracked artifact under `docs/drafts/`; `docs/src/` gained no workflow page or empty Diataxis bucket.
- Passed on 2026-03-07: `make docs-build`, `make check`, `make test`, `make test-long`, and `make lint`.
- This task predates the tightened docs-only verification rule update; later docs-only runs must also run `make docs-lint`, and must skip `make test` plus `make test-long` unless `git` shows intentional changes under `src/` or `tests/`.
</verification>
