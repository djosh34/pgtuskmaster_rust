## Task: Run Tutorial Pages Through Draft Check Edit Revise <status>not_started</status> <passes>false</passes> <priority>high</priority>

<description>
**Goal:** Create the first tutorials by running them through the authoring loop `draft -> check/edit -> revise`. This task is for authoring, not for the final truth-check pass.

The higher-order goal is to produce real managed lessons for newcomers while preserving the difference between tutorials and work-oriented how-to pages.

**Scope:**
- Work in:
  - `docs/src/`
  - `docs/drafts/`
  - `docs/src/SUMMARY.md`
- Create at most 5 tutorial pages in this run.
- Prefer a concrete learner path that can actually be followed.
- Link out to reference or explanation instead of overloading the lesson.

**Mandatory reread before this run:**
- `.agents/skills/create-docs/references/diataxis.fr/start-here/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/compass/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/how-to-use-diataxis/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/tutorials/index.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/01-task-establish-diataxis-reread-and-draft-loop.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/02-task-run-reference-pages-through-draft-check-edit-revise.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/03-task-run-explanation-pages-through-draft-check-edit-revise.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/04-task-run-how-to-pages-through-draft-check-edit-revise.md`

**Tutorial summary, cross-checked from the source:**
- A tutorial is a lesson and a learning experience.
- The learner learns through what they do.
- The path should be carefully managed, concrete, and safe.
- Every step should produce a comprehensible result.
- The tutorial should maintain a narrative of the expected.
- It should minimise explanation and avoid unnecessary choices.
- It must aspire to reliability because the absent teacher cannot rescue the learner.

**Required execution loop:**
1. Reread the mandatory sources.
2. Select at most 5 tutorial pages that together form a concrete learning path.
3. For each page, classify it with the compass as `action + acquisition`.
4. Create multiple candidate drafts in `docs/drafts/` when comparison is useful.
5. Use `ask-k2-docs` when useful, with the learner goal, mdBook context, and the reminder to minimise explanation.
6. Check/edit each candidate for how-to branching, explanation-heavy lecture, or abstract teaching.
7. Choose the strongest draft and revise it again after agent edits.
8. Write the current best version under `docs/src/`, linking out where deeper material belongs.
9. Update `docs/src/SUMMARY.md` only with real pages that now exist.
10. If a stronger tutorial path emerges, change the layout.
11. After the capped work for this run is done, write to `progress_append`.
12. QUIT IMMEDIATELY after the progress append. Do not continue into a sixth page, extra cleanup, or git workflow.
13. No git commit is required for this stop point.

**Expected outcome:**
- The docs now include tutorials created through the agreed authoring loop.
- Tutorial pages remain distinct from how-to guides because they are managed lessons rather than work procedures.
- Verification for this docs task must always run `make docs-build`, `make docs-lint`, `make check`, and `make lint`; the expected docs-creation case is zero changes under `src/` or `tests/`; use `git` plus common sense, and do not run `make test` or `make test-long` unless the work intentionally changed behavior under `src/` or `tests/`.
- This run stops immediately after the capped docs work and progress append, to keep focus on new docs, refresh the Diataxis method in the next run, and reduce context bloat.

</description>

<acceptance_criteria>
- [ ] No more than 5 pages are authored in this run
- [ ] Every created page is intended as tutorial and passes the compass as `action + acquisition`
- [ ] Competing drafts, when used, live under `docs/drafts/`
- [ ] No page drifts into how-to branching or explanation-heavy lecture
- [ ] `docs/src/SUMMARY.md` contains only real existing pages
- [ ] The task is free to radically change navigation if a stronger learning path emerges
- [ ] `make docs-build` — passes cleanly
- [ ] `make docs-lint` — passes cleanly
- [ ] `make check` — passes cleanly
- [ ] Expected docs-creation case: `git` shows no intentional changes under `src/` or `tests/`, so `make test` and `make test-long` are not run
- [ ] Only if `git` shows intentional changes under `src/` or `tests/`, and common sense says behavior may have changed: `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] Only if `git` shows intentional changes under `src/` or `tests/`, and those changes impact ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
