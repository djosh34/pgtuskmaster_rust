## Task: Run Reference Pages Through Draft Check Edit Revise <status>done</status> <passes>true</passes> <priority>high</priority>

<description>
**Goal:** Create the first real reference pages by running them through the authoring loop `draft -> check/edit -> revise`. This task is for authoring, not for the final truth-check pass.

The higher-order goal is to produce strong reference pages while keeping structure open and keeping the final truth-check as a distinct later task.

**Scope:**
- Work in:
  - `docs/src/`
  - `docs/drafts/`
  - `docs/src/SUMMARY.md`
- Create at most 5 real reference pages in this run.
- Choose pages from actual machinery boundaries in the repo.
- Do not add empty landing pages or empty categories.

**Mandatory reread before this run:**
- `.agents/skills/create-docs/references/diataxis.fr/start-here/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/compass/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/how-to-use-diataxis/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/reference/index.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/01-task-establish-diataxis-reread-and-draft-loop.md`

**Reference summary, cross-checked from the source:**
- Reference is information-oriented and serves work.
- The purpose of a reference page is to describe as succinctly as possible, and in an orderly way.
- `Neutral description` is the key imperative.
- Reference should be led by the product it describes, not by a guessed user journey.
- The structure of reference should mirror the structure of the machinery.
- Examples may illustrate, but must not turn into instruction.

**Required execution loop:**
1. Reread the mandatory sources.
2. Identify candidate reference pages from real machinery boundaries.
3. For each page, classify it with the compass as `cognition + application`.
4. Create multiple candidate drafts in `docs/drafts/` when the page is important enough to compare.
5. Use `ask-k2-docs` when useful, with mdBook context and the reminder to `describe and only describe`.
6. Check/edit each candidate for drift into explanation, instruction, speculation, or feature-tour writing.
7. Choose the strongest draft and revise it again after agent edits.
8. Write the current best version under `docs/src/`.
9. Update `docs/src/SUMMARY.md` only with real pages that now exist.
10. If the new pages suggest better grouping, change the layout. Do not preserve a weaker structure.
11. After the capped work for this run is done, write to `progress_append`.
12. QUIT IMMEDIATELY after the progress append. Do not continue into a sixth page, extra cleanup, or git workflow.
13. No git commit is required for this stop point.

**Expected outcome:**
- The docs now contain a first reference batch written through the agreed authoring loop.
- The layout reflects only real pages, not speculative future sections.
- These pages are ready for the later accuracy pass, but are not yet treated as finally checked for truth.
- Verification for this docs task must always run `make docs-build`, `make docs-lint`, `make check`, and `make lint`; the expected docs-creation case is zero changes under `src/` or `tests/`; use `git` plus common sense, and do not run `make test` or `make test-long` unless the work intentionally changed behavior under `src/` or `tests/`.
- This run stops immediately after the capped docs work and progress append, to keep focus on new docs, refresh the Diataxis method in the next run, and reduce context bloat.

</description>

<acceptance_criteria>
- [x] No more than 5 pages are authored in this run
- [x] Every created page is intended as reference and passes the compass as `cognition + application`
- [x] Competing drafts, when used, live under `docs/drafts/`
- [x] No page drifts into how-to or explanation content
- [x] `docs/src/SUMMARY.md` contains only real existing pages
- [x] The task is free to radically change navigation if stronger reference groupings emerge
- [x] `make docs-build` — passes cleanly
- [x] `make docs-lint` — passes cleanly
- [x] `make check` — passes cleanly
- [x] Expected docs-creation case: `git` shows no intentional changes under `src/` or `tests/`
- [x] `make test` — passes cleanly
- [x] `make lint` — passes cleanly
- [x] `make test-long` — passes cleanly
</acceptance_criteria>

<implementation_plan>
1. Re-read the mandatory Diataxis sources at the start of execution, not from memory:
   - `.agents/skills/create-docs/references/diataxis.fr/start-here/index.md`
   - `.agents/skills/create-docs/references/diataxis.fr/compass/index.md`
   - `.agents/skills/create-docs/references/diataxis.fr/how-to-use-diataxis/index.md`
   - `.agents/skills/create-docs/references/diataxis.fr/reference/index.md`
   - `./.ralph/tasks/story-build-docs-diataxis-from-zero/01-task-establish-diataxis-reread-and-draft-loop.md`
   - Use this reread to police form drift during authoring: every selected page must remain `cognition + application`, must `describe and only describe`, and must mirror machinery instead of a user journey.
2. Inspect the repo only enough to pick real machinery boundaries and facts before drafting:
   - review `src/` module boundaries and the current `docs/src/` tree, including `docs/src/SUMMARY.md`
   - prefer subsystems that are already stable top-level modules and have obvious reference value
   - avoid speculative pages, cross-cutting tour pages, and anything that would become explanation or how-to material
3. Check for overlap before selecting pages:
   - verify whether any existing page in `docs/src/` already covers part of a candidate subsystem
   - if overlap exists, prefer revising, splitting, moving, or deleting the weaker page instead of creating parallel pages with duplicated facts
   - keep the output tree lean; this run should reduce ambiguity, not add competing documentation for the same machinery
4. Select at most 5 first-batch reference pages from concrete subsystem boundaries:
   - start with the strongest 3 candidates and expand to a fourth or fifth page only if the earlier pages remain crisp, factual, and non-overlapping after editing
   - first-choice candidates for this run:
     - `docs/src/reference/config.md` for `src/config/`
     - `docs/src/reference/dcs.md` for `src/dcs/`
     - `docs/src/reference/ha.md` for `src/ha/`
     - `docs/src/reference/api.md` for `src/api/` plus `src/debug_api/` only if the relationship is factual and concise
     - `docs/src/reference/cli.md` for `src/cli/` and the two binaries under `src/bin/`
   - if one candidate proves too fuzzy or too explanation-heavy, replace it with another machinery-shaped page such as `process`, `logging`, `pginfo`, or `state`
   - do not create empty landing pages or category placeholders beyond pages that actually exist by the end of the run
5. Classify each chosen page explicitly with the compass before drafting:
   - ask `action or cognition?` and `acquisition or application?`
   - keep a short working note per page in draft material that confirms `cognition + application`
   - if a page wants to teach, justify decisions, narrate flows, or recommend operations, split that material out mentally and exclude it from this reference pass
6. Gather only the source facts needed for each chosen page:
   - read the relevant Rust modules, types, functions, and any existing tests that reveal stable surface area
   - extract neutral facts such as responsibilities, boundaries, key components, inputs/outputs, configuration surfaces, invariants, error surfaces, and relationships between modules
   - do not infer undocumented guarantees; when uncertain, either verify in code or omit
7. Draft in `docs/drafts/` before promoting anything into `docs/src/`:
   - create one draft per page at minimum
   - for the most important or structurally ambiguous pages, create multiple competing drafts under `docs/drafts/` so structure can be compared before choosing one
   - keep drafts easy to diff and compare; draft filenames should map clearly to the intended final page
8. Use `ask-k2-docs` only as a prose assistant where it adds value:
   - provide the exact facts collected from code, the intended mdBook destination, and the explicit instruction that this is reference prose that must `describe and only describe`
   - ask it to improve wording, compression, or structural ordering, not to discover facts or inspect the repo
   - reject any output that introduces explanation, instruction, guesses, or marketing tone
9. Check and edit each draft skeptically before promotion:
   - remove any sentence that tells the reader what to do, why the design exists, or how to achieve an operational goal
   - tighten headings so they mirror machinery, for example: purpose, module layout, key types/components, inputs and outputs, state/decision surfaces, API/CLI surface, errors, and limits
   - prefer concise tables or bullet lists only when they improve consultation speed
   - keep examples only if they illustrate a surface without turning into step-by-step instruction
10. Promote only the strongest current version of each page into `docs/src/`:
   - create real final pages only for drafts that survived the form check
   - place them under a real reference section such as `docs/src/reference/*.md` if that grouping emerges naturally from the authored pages
   - if the best structure is different after drafting, change the layout instead of preserving a weaker guess made earlier
11. Update `docs/src/SUMMARY.md` strictly from real pages that now exist:
   - add only links for authored pages
   - keep navigation minimal and factual
   - do not add empty reference indexes or future placeholders just to make the tree look complete
12. Verify the authored docs and only the needed code-impact suites:
   - run `make docs-build`
   - run `make docs-lint`
   - run `make check`
   - run `make lint`
   - inspect `git diff --name-only -- src tests` and the staged equivalent to confirm the expected docs-only case
   - do not run `make test` or `make test-long` unless the run intentionally changed behavior under `src/` or `tests/`
   - if code or test changes did occur and common sense says behavior changed, run `make test`
   - only if those changes affect the ultra-long suite or its selection, run `make test-long`
13. Finish the run without stretching scope:
   - tick only the acceptance boxes that are actually satisfied
   - append a concise progress note describing which reference pages were authored and any structural decisions
   - end this task section with a record of what still needs truth-checking later if relevant
   - QUIT IMMEDIATELY after the progress append; do not continue into a sixth page, bonus cleanup, or git workflow in this execution run

NOW EXECUTE
</implementation_plan>

<verification>
- Authored three reference pages under `docs/src/reference/`: `config.md`, `dcs.md`, and `ha.md`.
- Added draft material under `docs/drafts/`, including two competing HA drafts before promoting the final page.
- Updated `docs/src/SUMMARY.md` to include only the real authored reference pages.
- Confirmed the docs-only condition with `git diff --name-only -- src tests`, which returned no intentional `src/` or `tests/` changes for this run.
- Passed on 2026-03-07: `make docs-build`, `make docs-lint`, `make check`, `make test`, `make test-long`, and `make lint`.
- `make test` and `make test-long` were still run even though this was a docs-only change, because the repo-level completion rule for this run was stricter than the docs-task default skip condition.
</verification>
