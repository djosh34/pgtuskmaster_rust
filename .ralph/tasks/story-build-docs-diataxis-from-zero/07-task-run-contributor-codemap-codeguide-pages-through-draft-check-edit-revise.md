## Task: Run Contributor Codemap Codeguide Pages Through Draft Check Edit Revise <status>not_started</status> <passes>false</passes> <priority>high</priority>

<description>
**Goal:** Create a separate contributor chapter for codemap and codeguide material by running the pages through the authoring loop `draft -> check/edit -> revise`. This task is for authoring, not for the final truth-check pass.

The higher-order goal is to give contributors a very detailed, in-depth explanation of how the codebase works without muddling up the different forms of documentation. The chapter may be contributor-focused because the bundled Diataxis source explicitly recognizes documentation for `the contributors who help maintain it`, but each page must still be classified with the compass and kept true to its form.

**Scope:**
- Work in:
  - `docs/src/`
  - `docs/drafts/`
  - `docs/src/SUMMARY.md`
- Create at most 5 real contributor-facing pages in this run.
- Create a separate contributor/codemap/codeguide chapter only from real pages that exist now.
- Do not create an empty contributor bucket or empty Diataxis buckets.
- Prefer pages that explain actual code structure, module boundaries, runtime flows, ownership boundaries, and test surfaces from the current repo.

**Mandatory reread before this run:**
- `.agents/skills/create-docs/references/diataxis.fr/start-here/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/compass/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/how-to-use-diataxis/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/complex-hierarchies/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/explanation/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/reference/index.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/01-task-establish-diataxis-reread-and-draft-loop.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/02-task-run-reference-pages-through-draft-check-edit-revise.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/03-task-run-explanation-pages-through-draft-check-edit-revise.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/04-task-run-how-to-pages-through-draft-check-edit-revise.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/05-task-run-tutorial-pages-through-draft-check-edit-revise.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/06-task-derive-navigation-from-authored-pages.md`

**Contributor chapter summary, cross-checked from the source:**
- Diataxis is not a rigid four-box scheme.
- The docs may address different concerns, including `the contributors who help maintain it`.
- What matters is the experience of the reader.
- Documentation should be as complex as it needs to be.
- The contributor chapter may be a separate audience-facing hierarchy if that fits user needs, but it must still keep the forms separate instead of mixing them on one page.
- Use the compass by asking `action or cognition?` and `acquisition or application?`
- Reference should `describe and only describe` and mirror the structure of the machinery.
- Explanation should provide context, background, reasons, alternatives, and why.

**Required execution loop:**
1. Reread the mandatory sources.
2. Review the current repo and existing docs to identify the highest-value contributor pages.
3. Decide the separate contributor/codemap/codeguide chapter layout from real content only.
4. Select at most 5 pages.
5. For each page, classify it with the compass before drafting. If a page drifts between explanation and reference, split it instead of mixing forms.
6. Create multiple candidate drafts in `docs/drafts/` when comparison is useful.
7. Use `ask-k2-docs` when useful, with mdBook context, contributor audience, verified facts, explicit non-facts, relevant Diataxis reminders, and this additional writing rule: do not use `gate` for anything that means `test`.
8. Check/edit each candidate for form drift, invented facts, vague architecture hand-waving, shallow codemap coverage, and forbidden wording that uses `gate` where `test` is meant.
9. Choose the strongest draft and revise it again after agent edits.
10. Write the current best version under `docs/src/` and update `docs/src/SUMMARY.md` only with real pages that now exist.
11. If a stronger contributor hierarchy emerges, change the layout. Do not preserve a weaker structure.
12. Append progress and quit.

**Expected outcome:**
- The docs now contain a separate contributor/codemap/codeguide chapter justified by real contributor needs.
- The new pages give a detailed, in-depth explanation of how the codebase works.
- The chapter keeps Diataxis forms separate page by page while still serving contributors as a distinct audience.
- Verification for this docs task must always run `make docs-build`, `make docs-lint`, `make check`, and `make lint`; the expected docs-creation case is zero changes under `src/` or `tests/`; use `git` plus common sense, and do not run `make test` or `make test-long` unless the work intentionally changed behavior under `src/` or `tests/`.

</description>

<acceptance_criteria>
- [ ] No more than 5 pages are authored in this run
- [ ] A separate contributor/codemap/codeguide chapter is created only from real existing pages
- [ ] Every created page is classified with the compass and kept within a single Diataxis form
- [ ] Reference pages in the chapter `describe and only describe` and mirror the structure of the machinery
- [ ] Explanation pages in the chapter provide context, background, reasons, alternatives, and why without turning into procedure
- [ ] Competing drafts, when used, live under `docs/drafts/`
- [ ] `ask-k2-docs` is used only for prose work and its prompt includes the rule not to use `gate` where `test` is meant
- [ ] `docs/src/SUMMARY.md` contains only real existing pages
- [ ] The task is free to radically change contributor-facing navigation if a stronger hierarchy emerges
- [ ] `make docs-build` — passes cleanly
- [ ] `make docs-lint` — passes cleanly
- [ ] `make check` — passes cleanly
- [ ] Expected docs-creation case: `git` shows no intentional changes under `src/` or `tests/`, so `make test` and `make test-long` are not run
- [ ] Only if `git` shows intentional changes under `src/` or `tests/`, and common sense says behavior may have changed: `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] Only if `git` shows intentional changes under `src/` or `tests/`, and those changes impact ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
