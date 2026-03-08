## Task: Run K2 Docs Loop In Five-Way Parallel Batches Until All Diataxis Sections Have Enough Pages <status>not_started</status> <passes>false</passes>

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
- Before setting `<passes>true</passes>`, run one final blind review through `ask-k2` over the full published `docs/src/` corpus only.
- Do not provide source code, tests, repo structure, task history, or suggested missing topics to that final review; provide only the docs corpus.
- `<passes>true</passes>` is allowed only if that blind docs-only review explicitly concludes that the docs feel complete enough that no Diataxis quadrant is thin, underdeveloped, mixed-mode in a blocking way, or obviously missing key operator-facing coverage.
- If that blind docs-only review identifies any concrete missing area, weak quadrant, mixed-mode page, or obvious “what is still missing here?” gap, the task remains `<passes>false</passes>` and another five-page batch must run.
- NEVER EVER RUN any tests or code-oriented validation commands. This task changes no `src/` or `tests/` files. NOT A SINGLE TEST MUST BE RAN

**Expected outcome:**
- Five-page batches are executed in parallel without cross-contamination between chosen docs, requested sources, drafts, or revision prompts.
- The task stops only when all four Diataxis sections have enough published pages in `docs/src/` to count as meaningfully populated.
- `<passes>true</passes>` is set only after that coverage gate is actually met at startup of a batch decision point and the final blind docs-only `ask-k2` review passes; otherwise the task remains incomplete and you should commit, progress and QUIT IMMEDIATELY
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
- [ ] Before `<passes>true</passes>`, a final blind docs-only `ask-k2` review is run over the full published `docs/src/` corpus with no code, test, repo-structure, or suggested-topic context
- [ ] That blind docs-only review does not identify any quadrant as thin or underdeveloped and does not identify any obvious missing operator-significant coverage
- [ ] No tests are run: do not run `cargo test`, `make test`, `make test-long`, `make check`, `make lint`, or any equivalent test suite for this docs-only task, even if requested in the prompt
- [ ] No additional research is done, you must only 'NOW EXECUTE', this is it, this is the exact workflow!
</acceptance_criteria>
