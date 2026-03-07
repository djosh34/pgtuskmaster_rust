## Task: Run Reference Pages Through Draft Check Edit Revise <status>not_started</status> <passes>false</passes>

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
11. Append progress and quit.

**Expected outcome:**
- The docs now contain a first reference batch written through the agreed authoring loop.
- The layout reflects only real pages, not speculative future sections.
- These pages are ready for the later accuracy pass, but are not yet treated as finally checked for truth.

</description>

<acceptance_criteria>
- [ ] No more than 5 pages are authored in this run
- [ ] Every created page is intended as reference and passes the compass as `cognition + application`
- [ ] Competing drafts, when used, live under `docs/drafts/`
- [ ] No page drifts into how-to or explanation content
- [ ] `docs/src/SUMMARY.md` contains only real existing pages
- [ ] The task is free to radically change navigation if stronger reference groupings emerge
- [ ] `make docs-build` — passes cleanly
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
