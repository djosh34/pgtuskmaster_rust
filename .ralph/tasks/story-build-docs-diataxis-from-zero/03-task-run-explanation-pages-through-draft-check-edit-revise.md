## Task: Run Explanation Pages Through Draft Check Edit Revise <status>done</status> <passes>true</passes> <priority>high</priority>

<description>
**Goal:** Create the first explanation pages by running them through the authoring loop `draft -> check/edit -> revise`. This task is for authoring, not for the final truth-check pass.

The higher-order goal is to make the docs more understandable without polluting reference pages and without pretending the pages are already fact-verified to the final standard.

**Scope:**
- Work in:
  - `docs/src/`
  - `docs/drafts/`
  - `docs/src/SUMMARY.md`
- Create at most 5 explanation pages in this run.
- Use existing reference pages as anchors where useful.
- Do not create empty explanation landing pages.

**Mandatory reread before this run:**
- `.agents/skills/create-docs/references/diataxis.fr/start-here/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/compass/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/how-to-use-diataxis/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/explanation/index.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/01-task-establish-diataxis-reread-and-draft-loop.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/02-task-run-reference-pages-through-draft-check-edit-revise.md`

**Explanation summary, cross-checked from the source:**
- Explanation is understanding-oriented.
- It provides context, background, reasons, alternatives, history, and why.
- Explanation is an answer to `Can you tell me about ...?`
- It may contain perspective and judgement.
- It must not become a how-to guide or a reference catalog.

**Required execution loop:**
1. Reread the mandatory sources.
2. Select at most 5 explanation pages.
3. For each page, classify it with the compass as `cognition + acquisition`.
4. Create multiple candidate drafts in `docs/drafts/` when comparison is useful.
5. Use `ask-k2-docs` when useful, with the explanatory angle, mdBook context, and a warning not to drift into procedure.
6. Check/edit each candidate for reference dumping, step-by-step guidance, or vague empty abstraction.
7. Choose the strongest draft and revise it again after agent edits.
8. Write the current best version under `docs/src/` and link to reference where appropriate.
9. Update `docs/src/SUMMARY.md` only with real pages that now exist.
10. If better grouping emerges, change the layout.
11. After the capped work for this run is done, write to `progress_append`.
12. QUIT IMMEDIATELY after the progress append. Do not continue into a sixth page, extra cleanup, or git workflow.
13. No git commit is required for this stop point.

**Expected outcome:**
- The docs now include explanation pages created through the agreed authoring loop.
- Explanation remains distinct from reference and how-to material.
- Verification for this docs task must always run `make docs-build`, `make docs-lint`, `make check`, and `make lint`; the expected docs-creation case is zero changes under `src/` or `tests/`; use `git` plus common sense, and do not run `make test` or `make test-long` unless the work intentionally changed behavior under `src/` or `tests/`.
- This run stops immediately after the capped docs work and progress append, to keep focus on new docs, refresh the Diataxis method in the next run, and reduce context bloat.

</description>

<acceptance_criteria>
- [x] No more than 5 pages are authored in this run
- [x] Every created page is intended as explanation and passes the compass as `cognition + acquisition`
- [x] Competing drafts, when used, live under `docs/drafts/`
- [x] No page drifts into reference dump or step-by-step how-to guidance
- [x] `docs/src/SUMMARY.md` contains only real existing pages
- [x] The task is free to radically change navigation if stronger explanation grouping emerges
- [x] `make docs-build` — passes cleanly
- [x] `make docs-lint` — passes cleanly
- [x] `make check` — passes cleanly
- [x] Expected docs-creation case confirmed: `git` shows no intentional changes under `src/` or `tests/`; this run still executed `make test` and `make test-long` because the repo-level completion rule required the full suite
- [x] Repo-level override for this docs-only run: `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] Repo-level override for this docs-only run: `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

<implementation_plan>
1. Re-read the mandatory sources again at execution time, not from memory:
   - `.agents/skills/create-docs/references/diataxis.fr/start-here/index.md`
   - `.agents/skills/create-docs/references/diataxis.fr/compass/index.md`
   - `.agents/skills/create-docs/references/diataxis.fr/how-to-use-diataxis/index.md`
   - `.agents/skills/create-docs/references/diataxis.fr/explanation/index.md`
   - `./.ralph/tasks/story-build-docs-diataxis-from-zero/01-task-establish-diataxis-reread-and-draft-loop.md`
   - `./.ralph/tasks/story-build-docs-diataxis-from-zero/02-task-run-reference-pages-through-draft-check-edit-revise.md`
   - Use that reread to police form drift: every selected page must remain `cognition + acquisition`, answer an implicit `about ...` question, and avoid instructions plus reference catalogs.
2. Inspect the current authored docs and the codebase only enough to choose the first explanation topic that is grounded in real machinery:
   - review `docs/src/SUMMARY.md` and the existing reference pages under `docs/src/reference/`
   - review only the module boundaries under `src/` that are needed to justify the first chosen page, not the whole explanation batch up front
   - prefer cross-cutting topics that help readers understand why the system is shaped the way it is, rather than subsystem inventories that would duplicate reference material
   - treat later page selection as a repeated decision after each finished page, not as a fixed batch chosen in advance
3. Select explanation pages one at a time, with an explicit stop/reselect gate after each page, and author no more than 5 in the full run:
   - start from this priority order, but do not commit to all of them before the first page survives editing:
     - `docs/src/explanation/runtime-control-loop.md` about how the node composes workers, versioned state, and polling loops into one runtime
     - `docs/src/explanation/dcs-trust-and-ha-gating.md` about why HA decisions are gated by DCS trust and freshness
     - `docs/src/explanation/replica-recovery-paths.md` about why rewind, base backup, bootstrap, and fencing are separate recovery paths
     - `docs/src/explanation/config-normalization-and-validation.md` about why configuration is versioned, normalized, and heavily validated before runtime
     - `docs/src/explanation/control-surfaces.md` about the separation between runtime workers, API/debug surfaces, and CLI control paths
   - after each page is drafted and edited, decide whether the next-best improvement is another explanation page, a replacement topic, or stopping short of 5 because the remaining candidates would drift into reference or how-to territory
   - if a candidate collapses into reference listing or operational procedure, replace it rather than force it through
4. Classify each chosen page explicitly with the compass before drafting:
   - ask `action or cognition?` and `acquisition or application?`
   - keep short working notes in draft material showing why each page is `cognition + acquisition`
   - reject pages whose center of gravity becomes `how to operate`, `what knobs exist`, or `which types/functions exist`
5. Gather source facts and tensions for each chosen page from code and existing references:
   - read the relevant modules for runtime orchestration, HA decisions, DCS trust evaluation, process dispatch, configuration loading, and recovery paths
   - extract only the facts needed to explain relationships, reasons, alternatives, and consequences
   - use the existing reference pages as anchors for factual surfaces, linking out when detailed catalog material would otherwise bloat the explanation page
   - omit any claim that cannot be grounded in code or existing reference prose
6. Draft in `docs/drafts/` before promoting pages:
   - create at least one draft per page
   - create multiple competing drafts for the pages that are most structurally ambiguous, especially the runtime-control-loop and recovery-path pages
   - keep filenames explicit so draft lineage to final pages is obvious
   - finish the full `draft -> check/edit -> revise` loop for one page before investing in deep drafting for the next page, unless a direct comparison between two competing structures is needed for that same page
7. Use `ask-k2-docs` only if prose revision materially helps:
   - provide the exact facts, intended explanatory angle, and mdBook destination
   - explicitly warn against drifting into procedure, recommendations, or reference dumping
   - treat the output as prose assistance only; reject any invented facts, repo inspection claims, or Diataxis form confusion
8. Check and edit each draft skeptically before promotion:
   - cut any steps, commands, or imperative guidance that turn the page into a how-to
   - cut any module-by-module cataloging that belongs in reference
   - strengthen context, alternatives, history-of-the-design style reasoning, and cross-links to relevant reference pages
   - keep titles in an `About ...` style even if the literal heading text is more concise
9. Promote only the strongest current drafts into `docs/src/` and update navigation from real pages only:
   - create final pages only for drafts that survive the explanation form check
   - place them under a real `docs/src/explanation/` section only if at least one real explanation page exists by the end of the run
   - update `docs/src/SUMMARY.md` to include only authored explanation pages
   - if a stronger layout emerges while drafting, prefer moving or regrouping pages over preserving an earlier guess
10. Verify the docs set and only the code-impact suites that are actually required:
   - run `make docs-build`
   - run `make docs-lint`
   - run `make check`
   - inspect `git diff --name-only -- src tests` and the staged equivalent to confirm whether this stayed docs-only
   - if the run remains docs-only, do not run `make test` or `make test-long`
   - only if intentional `src/` or `tests/` changes exist and common sense says behavior changed, run `make test`
   - only if those intentional changes affect the ultra-long suite or its selection, run `make test-long`
   - run `make lint`
   - fix failures by correcting the underlying docs or code problem, not by weakening checks
11. Finish the run without scope creep:
   - tick only the acceptance boxes that are actually satisfied
   - set `<passes>true</passes>` only after all required checks pass
   - append progress describing which explanation pages were authored, which drafts competed, and what still needs later truth-checking
   - QUIT IMMEDIATELY after the progress append and do not continue into a sixth page, bonus cleanup, or unrelated git workflow

NOW EXECUTE
</implementation_plan>

<verification>
- Authored three explanation pages under `docs/src/explanation/`: `runtime-control-loop.md`, `dcs-trust-and-ha-gating.md`, and `replica-recovery-paths.md`.
- Added competing draft candidates under `docs/drafts/` for the runtime-control-loop and replica-recovery-path topics, plus a draft for the DCS trust page.
- Updated `docs/src/SUMMARY.md` to include only the real authored explanation pages alongside the existing reference section.
- Confirmed the docs-only condition with `git diff --name-only -- src tests`, which returned no intentional `src/` or `tests/` changes for this run.
- Passed on 2026-03-07: `make docs-build`, `make docs-lint`, `make check`, `make test` (`396 passed, 9 skipped`), `make test-long` (`9 passed, 396 skipped`, plus Compose validation and Docker smoke checks), and `make lint`.
- `make test` and `make test-long` were still run even though this stayed docs-only, because the repo-level completion rule for the turn required the full verification stack.
</verification>
