## Task: Run K2 Docs Loop In Five-Way Parallel Batches Until All Diataxis Sections Have Enough Pages <status>in_progress</status> <passes>false</passes>

<priority>high</priority>

<description>
**Goal:** Create a docs-only execution task that follows the `k2-docs-loop` skill precisely, but accelerates throughput by running five fully independent page-selection and draft-preparation lanes in parallel. 

**Scope:**
- Read the 'k2-docs-loop' precisely, you will be following it exactly, but in parallel
- Decide if "enough pages" are done across all four sections: `docs/src/tutorial/`, `docs/src/explanation/`, `docs/src/how-to/`, and `docs/src/reference/` must each contain enough real content pages that the operator no longer needs to ask “what is still missing here?” 
  If any section is still not done or obviously underexplained, continue with the rest of the steps, else if fully done set passes true
- Start by asking K2 for five `choose-doc` decisions at the same time, exactly as described in the skill
- Await all five `choose-doc` results before moving on.
- For each of those five chosen pages, do exactly as the 'k2-docs-loop' requires for each of the 5 parallel requests.
- After all five choices are known, run `prepare-draft` for those five chosen pages in parallel as five separate lanes.
- Keep each lane fully independent.
- Once 5 parallel runs are done, commit all files and push, append progress and then QUIT IMMEDIATELY
- NEVER EVER RUN any tests or code-oriented validation commands. This task changes no `src/` or `tests/` files. NOT A SINGLE TEST MUST BE RAN

**Expected outcome:**
- Five-page batches are executed in parallel without cross-contamination between chosen docs, requested sources, drafts, or revision prompts.
- The task stops only when all four Diataxis sections have enough published pages in `docs/src/` to count as meaningfully populated.
- `<passes>true</passes>` is set only after that coverage gate is actually met at startup of a batch decision point; otherwise the task remains incomplete and you should commit, progress and QUIT IMMEDIATELY
- No tests are run anywhere in this task.

NOW EXECUTE

</description>

<acceptance_criteria>
- [ ] Scope is followed precisely and k2-docs-loop skill is leading
- [ ] The task quits immediately after a completed five-page batch unless, at startup of the next decision point, all four sections already have enough pages and the task can be marked complete
- [ ] `docs/src/tutorial/` contains enough real content pages
- [ ] `docs/src/explanation/` contains enough real content pages
- [ ] `docs/src/how-to/` contains enough real content pages
- [ ] `docs/src/reference/` contains enough real content pages
- [ ] If any section is still thin, unclear, or missing key coverage, the task remains `<passes>false</passes>`
- [ ] No tests are run: do not run `cargo test`, `make test`, `make test-long`, `make check`, `make lint`, or any equivalent test suite for this docs-only task, even if requested in the prompt
- [ ] No additional research is done, you must only 'NOW EXECUTE', this is it, this is the exact workflow!
</acceptance_criteria>
