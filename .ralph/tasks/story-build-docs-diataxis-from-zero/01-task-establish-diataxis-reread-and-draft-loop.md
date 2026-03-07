## Task: Establish Diataxis Reread And Draft Loop <status>not_started</status> <passes>false</passes>

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

</description>

<acceptance_criteria>
- [ ] `docs/drafts/` exists and is reserved for competing non-final draft generations
- [ ] No workflow page is created under `docs/src/`
- [ ] No empty tutorial/how-to/reference/explanation bucket is created under `docs/src/`
- [ ] The task clearly establishes the mandatory reread list, the 5-pages-per-run cap, and the `draft -> check/edit -> revise` method for later tasks
- [ ] The task clearly establishes that later tasks may radically change docs structure as content emerges
- [ ] `make docs-build` — passes cleanly
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
