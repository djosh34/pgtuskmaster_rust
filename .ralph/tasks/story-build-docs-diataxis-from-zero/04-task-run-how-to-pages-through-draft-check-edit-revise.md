## Task: Run How-To Pages Through Draft Check Edit Revise <status>not_started</status> <passes>false</passes> <priority>high</priority>

<description>
**Goal:** Create the first how-to guides by running them through the authoring loop `draft -> check/edit -> revise`. This task is for authoring, not for the final truth-check pass.

The higher-order goal is to produce useful work-oriented guides while preserving the difference between how-to pages and tutorials.

**Scope:**
- Work in:
  - `docs/src/`
  - `docs/drafts/`
  - `docs/src/SUMMARY.md`
- Create at most 5 how-to guides in this run.
- Choose real user goals or operational tasks, not vague product areas.
- Link out rather than mixing every kind of content into one page.

**Mandatory reread before this run:**
- `.agents/skills/create-docs/references/diataxis.fr/start-here/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/compass/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/how-to-use-diataxis/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/how-to-guides/index.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/01-task-establish-diataxis-reread-and-draft-loop.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/02-task-run-reference-pages-through-draft-check-edit-revise.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/03-task-run-explanation-pages-through-draft-check-edit-revise.md`

**How-to summary, cross-checked from the source:**
- A how-to guide addresses a real-world goal or problem.
- It serves work, not study.
- It should be written from the perspective of the user, not the machinery.
- It should contain `action and only action`.
- Practical usability matters more than completeness.
- If reference or explanation is needed, link to it instead of polluting the guide.

**Required execution loop:**
1. Reread the mandatory sources.
2. Select at most 5 real user-goal pages.
3. For each page, classify it with the compass as `action + application`.
4. Create multiple candidate drafts in `docs/drafts/` when comparison is useful.
5. Use `ask-k2-docs` when useful, with the user goal, mdBook context, and the reminder to keep `action and only action`.
6. Check/edit each candidate for tutorial teaching, explanation drift, or feature-catalog writing.
7. Choose the strongest draft and revise it again after agent edits.
8. Write the current best version under `docs/src/`, linking out instead of duplicating other page types.
9. Update `docs/src/SUMMARY.md` only with real pages that now exist.
10. If stronger grouping emerges, change the layout.
11. After the capped work for this run is done, write to `progress_append`.
12. QUIT IMMEDIATELY after the progress append. Do not continue into a sixth page, extra cleanup, or git workflow.
13. No git commit is required for this stop point.

**Expected outcome:**
- The docs now include work-oriented guides created through the agreed authoring loop.
- The distinction between how-to and tutorial remains visible.
- Verification for this docs task must always run `make docs-build`, `make docs-lint`, `make check`, and `make lint`; the expected docs-creation case is zero changes under `src/` or `tests/`; use `git` plus common sense, and do not run `make test` or `make test-long` unless the work intentionally changed behavior under `src/` or `tests/`.
- This run stops immediately after the capped docs work and progress append, to keep focus on new docs, refresh the Diataxis method in the next run, and reduce context bloat.

</description>

<acceptance_criteria>
- [ ] No more than 5 pages are authored in this run
- [ ] Every created page is intended as how-to and passes the compass as `action + application`
- [ ] Competing drafts, when used, live under `docs/drafts/`
- [ ] No page drifts into tutorial-style teaching or explanation-heavy discussion
- [ ] `docs/src/SUMMARY.md` contains only real existing pages
- [ ] The task is free to radically change navigation if stronger task-oriented grouping emerges
- [ ] `make docs-build` — passes cleanly
- [ ] `make docs-lint` — passes cleanly
- [ ] `make check` — passes cleanly
- [ ] Expected docs-creation case: `git` shows no intentional changes under `src/` or `tests/`, so `make test` and `make test-long` are not run
- [ ] Only if `git` shows intentional changes under `src/` or `tests/`, and common sense says behavior may have changed: `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] Only if `git` shows intentional changes under `src/` or `tests/`, and those changes impact ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
