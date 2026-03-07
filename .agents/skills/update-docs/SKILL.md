---
name: update-docs
description: Update existing mdBook documentation after code changes, bug fixes, or newly discovered doc gaps. Use when docs must be revised in-place using the Diataxis method rather than rewritten from scratch.
---

# Update Docs

Use this skill when documentation already exists and must be updated because code changed, behavior changed, a bug exposed stale docs, or a doc inaccuracy was reported.

This skill is for maintenance and correction, not for designing a grand top-down docs structure. Apply Diataxis to the pages in front of you and improve them iteratively.

Use this skill together with:

- `create-docs` for the bundled Diataxis source material and repo-wide docs constraints
- `ask-k2-docs` only when prose drafting or prose revision is useful

Before editing docs, reread the Diataxis sources again. Do not rely on memory from an earlier run.

Minimum reread set before each docs-update run:

- `../create-docs/references/diataxis.fr/start-here/index.md`
- `../create-docs/references/diataxis.fr/compass/index.md`
- `../create-docs/references/diataxis.fr/how-to-use-diataxis/index.md`
- the form-specific source for the page you are updating:
  - `../create-docs/references/diataxis.fr/tutorials/index.md`
  - `../create-docs/references/diataxis.fr/how-to-guides/index.md`
  - `../create-docs/references/diataxis.fr/reference/index.md`
  - `../create-docs/references/diataxis.fr/explanation/index.md`

Use the original Diataxis rules, not a fuzzy summary:

- ask the compass questions: `action or cognition?` and `acquisition or application?`
- keep the four forms separate instead of mixing them on one page
- tutorials are lessons and a carefully-managed path for study
- how-to guides are goal-oriented directions for work and should contain `action and only action`
- reference should `describe and only describe` and mirror the structure of the machinery
- explanation should provide context, background, reasons, alternatives, and why
- `work one step at a time`: choose something, assess it, decide one next action, do it

Important maintenance rule:

- do not preserve existing docs structure just because it already exists
- if the current page is the wrong Diataxis form, split it, move material, rename it, merge it elsewhere, or delete it
- if code changes make the old docs structure weaker, overthrow it
- do not create empty buckets while restructuring

Repo-specific rules:

- keep mdBook as the engine
- Mermaid diagrams are allowed when they clarify something and the agent can check them
- during docs-update work, change at most 5 pages per run, append progress, then quit so the next run starts fresh from the Diataxis sources again
- only the final accuracy-verification workflow should write under `docs/verifications/`
- draft alternatives may live under `docs/drafts/`

Use this update method:

1. Identify the trigger.
   Examples: changed code path, fixed bug, renamed config, changed command behavior, contradictory docs, missing docs for new feature.
2. Locate the affected existing docs pages.
3. Classify each affected page with the compass before changing it.
4. Decide whether the update is:
   - a local correction within the same page form
   - a page split because one page mixes forms
   - a move or rename because the current location or title is wrong
   - a new page because the old docs cannot absorb the change cleanly
5. Run the authoring loop on the affected pages:
   - draft
   - check/edit
   - revise
6. Keep the scope to at most 5 pages in the run.
7. Append progress and quit.

Use `ask-k2-docs` only when prose help is needed. The agent still owns:

- rereading Diataxis
- deciding the page form
- understanding the code change or bug
- deciding whether pages should be split, moved, merged, renamed, or removed
- diagrams and Mermaid
- file edits

Good prompt inputs for `ask-k2-docs` in update mode:

- what changed in the code or behavior
- which page is being updated
- what form the page must be
- what text is now wrong
- what facts must stay true
- what new wording or section structure is needed
- mdBook context

Do not ask K2 to:

- inspect the repo
- judge truth from unstated context
- decide the Diataxis form
- design the docs structure on its own
- produce verification artifacts

When updating docs because of a bug or stale behavior:

- prefer correcting the affected pages immediately if the right fix is clear
- if the docs problem reveals a deeper product or docs defect that should be tracked separately, use `add-bug`
- if the docs change needs a later truth pass, leave that to the final verification workflow rather than faking certainty now

The goal is not to keep old pages alive at all costs. The goal is to keep the docs set healthy under Diataxis as the product changes.
