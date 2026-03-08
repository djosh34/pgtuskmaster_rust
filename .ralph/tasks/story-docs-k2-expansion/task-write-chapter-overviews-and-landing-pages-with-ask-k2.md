## Task: Write Chapter Overviews, Introductions, README, And Landing Page With Ask-K2 <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Use the `ask-k2` skill, in the same raw-prompt style used by the `k2-docs-loop` workflow, to author the docs navigation and orientation pages that tie the generated content together. The higher-order goal is to turn a set of isolated pages into a coherent documentation experience with chapter entry points, a clear first landing page, and a repository-facing README.

**Scope:**
- Work only in docs-facing files such as `docs/src/`, `docs/src/SUMMARY.md`, and the repository `README.md`, plus temporary prompt/context artifacts under `docs/tmp/` if needed.
- Use `ask-k2` directly for authoring. Do not handwrite the prose unless a minimal factual cleanup is required after generation.
- Prompt K2 separately for each output page. One prompt must produce exactly one page so outputs do not bleed together.
- Provide K2 with the same kind of raw grounding the `k2-docs-loop` skill expects: the Diataxis markdown summary, the current docs tree, the current summary/table-of-contents context, and the already-created docs pages that the new page should link to.
- Create one overview/introduction page for each of the four docs chapters:
- tutorial
- explanation
- how-to
- reference
- Create the main landing overview page that is the first page users should land on when entering the documentation set.
- Create or rewrite the repository `README.md` so it links into the docs set, includes a quickstart section, and includes a license statement of `All rights reserved 'Joshua Azimullah'`.
- Ensure the overview and chapter pages link to the doc pages created by the parallel K2 docs tasks rather than degenerating into isolated prose.
- Update the docs summary/TOC so users can reach the landing page and each chapter overview cleanly.
- Do not run any tests or code-oriented validation commands. This task changes no `src/` or `tests/` files.

**Context from research:**
- The `ask-k2` skill is intentionally small: pipe the full prompt on stdin to `.agents/skills/ask-k2/ask-k2.sh`.
- The user explicitly wants `ask-k2` used “just like the k2-docs-loop skill,” which means raw context should be provided directly rather than paraphrased into a heavily curated prompt.
- The `k2-docs-loop` skill already uses the Diataxis source summary as core context. Reuse that pattern here so K2 writes orientation pages from the same framework as the content pages.
- Current published docs are organized under `docs/src/` and exposed through `docs/src/SUMMARY.md`; the current `SUMMARY.md` is still a flat, minimal list and needs proper navigation structure once more pages exist.
- This task should happen after or alongside content generation so the overview pages can link to the actual page set created by the docs loop.

**Expected outcome:**
- There is a clear docs landing page for first-time readers.
- There is one introduction/overview page for each Diataxis chapter that links to the pages in that chapter and explains what belongs there.
- `README.md` becomes a useful entry point with quickstart, links into the docs, and the required `All rights reserved 'Joshua Azimullah'` license text.
- Each page is written through a separate `ask-k2` prompt with raw context, not by combining multiple pages into a single model call.
- The resulting docs navigation is coherent and linked, not just a pile of generated files.
- No tests are run anywhere in this task.

</description>

<acceptance_criteria>
- [ ] The task uses the `ask-k2` skill directly, with one prompt per output page
- [ ] Each `ask-k2` prompt includes the Diataxis markdown summary and the relevant current docs context, matching the raw-context style of `k2-docs-loop`
- [ ] A landing overview page is created as the primary first-stop page for docs readers
- [ ] A chapter overview/introduction page is created for `tutorial`
- [ ] A chapter overview/introduction page is created for `explanation`
- [ ] A chapter overview/introduction page is created for `how-to`
- [ ] A chapter overview/introduction page is created for `reference`
- [ ] `README.md` is created or updated to link to the docs, include a quickstart, and include the exact license text `All rights reserved 'Joshua Azimullah'`
- [ ] The new overview pages link to the actual docs pages created elsewhere instead of remaining generic placeholders
- [ ] `docs/src/SUMMARY.md` is updated so the landing page and chapter overview pages are discoverable through a sensible TOC structure
- [ ] No tests are run: do not run `cargo test`, `make test`, `make check`, `make lint`, or any equivalent test suite for this docs-only task
</acceptance_criteria>
