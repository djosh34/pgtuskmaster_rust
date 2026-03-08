## Task: Write Chapter Overviews, Introductions, README, And Landing Page With Ask-K2 <status>completed</status> <passes>true</passes>

<priority>high</priority>

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
- [x] The task uses the `ask-k2` skill directly, with one prompt per output page
- [x] Each `ask-k2` prompt includes the Diataxis markdown summary and the relevant current docs context, matching the raw-context style of `k2-docs-loop`
- [x] A landing overview page is created as the primary first-stop page for docs readers
- [x] A chapter overview/introduction page is created for `tutorial`
- [x] A chapter overview/introduction page is created for `explanation`
- [x] A chapter overview/introduction page is created for `how-to`
- [x] A chapter overview/introduction page is created for `reference`
- [x] `README.md` is created or updated to link to the docs, include a quickstart, and include the exact license text `All rights reserved 'Joshua Azimullah'`
- [x] The new overview pages link to the actual docs pages created elsewhere instead of remaining generic placeholders
- [x] `docs/src/SUMMARY.md` is updated so the landing page and chapter overview pages are discoverable through a sensible TOC structure
- [x] No tests are run: do not run `cargo test`, `make test`, `make test-long`, `make check`, `make lint`, or any equivalent test suite for this docs-only task, even if requested in the prompt
</acceptance_criteria>

## Execution Plan

1. Establish the raw authoring inputs before generating any page.
   - Re-read `.agents/skills/ask-k2/SKILL.md` and `.agents/skills/k2-docs-loop/SKILL.md` immediately before execution so the run uses the same raw-prompt pattern as the existing docs workflow.
   - Gather the raw grounding files that every prompt must include:
     - `.agents/skills/k2-docs-loop/diataxis-summary.md`
     - `docs/src/SUMMARY.md`
     - a current tree/listing of `docs/src/`
     - the existing chapter pages under the target chapter directory
   - Create `docs/tmp/` prompt/context artifacts as needed so every generated page has a preserved per-page prompt/input trail.
   - Do not combine outputs: there must be one distinct `ask-k2` invocation per destination page.

2. Map the exact output files and their required link targets before prompting K2.
   - Landing page target: create `docs/src/overview.md` as the first docs stop and ensure it links into all four chapters.
   - Chapter overview targets: create one new overview page for each chapter directory at explicit paths so the execution pass does not need to invent filenames:
     - `docs/src/tutorial/overview.md`
     - `docs/src/explanation/overview.md`
     - `docs/src/how-to/overview.md`
     - `docs/src/reference/overview.md`
   - Repository entry point target: create a new root `README.md` because none exists yet.
   - For each target page, enumerate the concrete docs pages it should link to from the current tree so the prompts can instruct K2 to produce navigational pages rather than generic prose.

3. Build and run the landing-page prompt first.
   - Assemble a raw prompt file in `docs/tmp/` containing:
     - the task goal for the landing page only
     - the Diataxis summary
     - the current docs tree
     - the current `docs/src/SUMMARY.md`
     - the chapter file inventory that the landing page should surface
   - Pipe that prompt through `.agents/skills/ask-k2/ask-k2.sh`.
   - Save the first result to a draft file under `docs/tmp/`, inspect only for factual correctness and link accuracy, and make only minimal factual cleanup if required.
   - Move the final result into the chosen landing page path under `docs/src/`.

4. Generate the four chapter overview pages with separate raw prompts.
   - Run four independent `ask-k2` calls, one per chapter overview page.
   - Each chapter prompt must include:
     - the Diataxis summary
     - the current docs tree
     - the current `docs/src/SUMMARY.md`
     - the list and contents of the existing pages in that chapter that should be linked from the overview
     - the specific instruction that the page acts as a chapter introduction and navigation hub for that chapter only
   - Save each output to `docs/tmp/` first, review only for factual issues or broken links, and if the draft contains unsupported claims or wrong facts, mark those with inline `// todo: [...]` notes and send that draft back through `ask-k2` with the raw revise context instead of hand-rewriting it.
   - Move the minimally corrected result into the explicit destination paths named in step 2.

5. Create the repository `README.md` through its own `ask-k2` prompt.
   - Build a raw prompt dedicated only to `README.md`.
   - Include the Diataxis summary, the docs tree, the docs landing page path, the key tutorial/how-to/reference entry points, and the requirement for:
     - a repository overview
     - a quickstart section
     - links into the docs set
     - the exact license text `All rights reserved 'Joshua Azimullah'`
   - Because `README.md` is repo-facing rather than mdBook-facing, verify that links are correct from the repo root and adjust only facts or relative links if necessary.

6. Update `docs/src/SUMMARY.md` after all pages exist.
   - Insert `docs/src/overview.md` as the primary top-level entry immediately under `# Summary`.
   - Replace the current empty-link placeholders like `[Tutorials]()` with real chapter entries that point to:
     - `tutorial/overview.md`
     - `how-to/overview.md`
     - `explanation/overview.md`
     - `reference/overview.md`
   - Add each new chapter overview page as the first page inside its chapter section.
   - Keep the existing content pages discoverable beneath those overviews.

7. Perform docs-only verification without violating the task’s docs scope.
   - Re-open every newly created page plus `README.md` and `docs/src/SUMMARY.md`.
   - Check for:
     - correct relative links
     - no chapter overview left as generic placeholder text
     - the landing page linking to the actual chapter overviews and relevant docs
     - `README.md` containing the exact required license string
   - Run `make docs-build` to confirm the mdBook navigation compiles with the new pages.
   - Run `make docs-lint` to catch Mermaid or docs-architecture guard issues that apply to generated markdown.
   - Do not run `cargo test`, `make test`, `make test-long`, `make check`, or `make lint` during this task because the task explicitly forbids code/test validation for this docs-only change set.

8. Finish the task bookkeeping only after the docs work is complete.
   - Tick every acceptance-criteria checkbox that is satisfied.
   - Update task status metadata if the task format elsewhere in the repo expects that on completion.
   - Append progress-log notes summarizing which prompts were used, which files were created, and any factual cleanups made after K2 generation.

## Execution Notes

- Used `.agents/skills/ask-k2/ask-k2.sh` for six separate outputs: `docs/src/overview.md`, the four chapter overview pages, and `README.md`.
- Each prompt included the Diataxis summary plus raw docs context. Chapter prompts also included the full contents of the chapter pages they were summarizing.
- Promoted the K2 drafts into `docs/src/overview.md`, `docs/src/tutorial/overview.md`, `docs/src/how-to/overview.md`, `docs/src/explanation/overview.md`, `docs/src/reference/overview.md`, and `README.md`.
- Applied minimal factual cleanup after generation:
  - fixed the reference overview links to use paths relative to `docs/src/reference/overview.md`
  - linked the repository README to `docs/src/overview.md` and the chapter overviews
  - updated `docs/src/SUMMARY.md` so the landing page and chapter overview pages are first-class navigation entries
- Verification completed with `make docs-build` and `make docs-lint`.

NOW EXECUTE
