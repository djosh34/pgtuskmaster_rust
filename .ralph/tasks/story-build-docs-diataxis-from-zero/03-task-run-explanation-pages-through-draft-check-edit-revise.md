## Task: Run Explanation Pages Through Draft Check Edit Revise <status>not_started</status> <passes>false</passes> <priority>high</priority>

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
- [ ] No more than 5 pages are authored in this run
- [ ] Every created page is intended as explanation and passes the compass as `cognition + acquisition`
- [ ] Competing drafts, when used, live under `docs/drafts/`
- [ ] No page drifts into reference dump or step-by-step how-to guidance
- [ ] `docs/src/SUMMARY.md` contains only real existing pages
- [ ] The task is free to radically change navigation if stronger explanation grouping emerges
- [ ] `make docs-build` — passes cleanly
- [ ] `make docs-lint` — passes cleanly
- [ ] `make check` — passes cleanly
- [ ] Expected docs-creation case: `git` shows no intentional changes under `src/` or `tests/`, so `make test` and `make test-long` are not run
- [ ] Only if `git` shows intentional changes under `src/` or `tests/`, and common sense says behavior may have changed: `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] Only if `git` shows intentional changes under `src/` or `tests/`, and those changes impact ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
