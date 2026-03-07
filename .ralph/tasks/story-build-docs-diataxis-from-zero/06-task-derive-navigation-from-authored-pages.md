## Task: Derive Navigation From Authored Pages <status>not_started</status> <passes>false</passes> <priority>high</priority>

<description>
**Goal:** After real content exists across multiple Diataxis forms, derive mdBook navigation and any landing pages from that content. This task is for authored structure, not for the final truth-check pass.

The higher-order goal is to make visible structure emerge from authored pages rather than from speculation.

**Scope:**
- Work in:
  - `docs/src/`
  - `docs/drafts/`
  - `docs/src/SUMMARY.md`
- Rework navigation, grouping, and landing pages only from real existing pages.
- If explicit Diataxis categories now make sense in navigation, expose them. Do not add empty sections.

**Mandatory reread before this run:**
- `.agents/skills/create-docs/references/diataxis.fr/start-here/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/compass/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/how-to-use-diataxis/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/complex-hierarchies/index.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/01-task-establish-diataxis-reread-and-draft-loop.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/02-task-run-reference-pages-through-draft-check-edit-revise.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/03-task-run-explanation-pages-through-draft-check-edit-revise.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/04-task-run-how-to-pages-through-draft-check-edit-revise.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/05-task-run-tutorial-pages-through-draft-check-edit-revise.md`

**Navigation summary, cross-checked from the source:**
- Diataxis is not a rigid four-box scheme.
- Clear structural division into the four categories is a likely outcome of good practice, not the starting requirement.
- Landing pages should read like overviews, not empty lists.
- The user experience matters more than forcing the scheme.

**Required execution loop:**
1. Reread the mandatory sources.
2. Review the real pages that now exist.
3. Decide whether explicit Diataxis grouping in navigation is now justified by the content.
4. If yes, expose tutorial/how-to/reference/explanation groupings in `docs/src/SUMMARY.md`, but only where there are real pages to place.
5. If landing pages are needed, create them as real overviews that introduce the pages beneath them.
6. Create multiple candidate navigation or landing drafts in `docs/drafts/` when comparison is useful.
7. Use `ask-k2-docs` when useful for landing-page prose, with mdBook context and explicit instruction that the page must read like an overview.
8. Check/edit and revise the navigation and landing-page candidates.
9. Choose the strongest arrangement, update `docs/src/SUMMARY.md`, and remove weaker obsolete groupings.
10. Append progress and quit.

**Expected outcome:**
- The visible mdBook structure now reflects real authored content instead of speculative planning.
- If explicit Diataxis categories appear in navigation, they do so because the authored pages justify them.

</description>

<acceptance_criteria>
- [ ] `docs/src/SUMMARY.md` is derived from real content rather than speculative future sections
- [ ] Any explicit Diataxis groupings exposed in navigation contain real pages and no empty buckets
- [ ] Any landing page created is a real overview page, not a placeholder
- [ ] Competing drafts, when used, live under `docs/drafts/`
- [ ] The resulting navigation improves clarity without muddling tutorial, how-to, reference, and explanation
- [ ] `make docs-build` — passes cleanly
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
