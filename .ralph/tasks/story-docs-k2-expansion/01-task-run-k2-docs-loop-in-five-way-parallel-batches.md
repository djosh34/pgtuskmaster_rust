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

**Batch 1 execution notes:**
- Ran five parallel `choose-doc` lanes; one duplicate target was replaced with a fresh lane so the batch stayed five-way and independent.
- Chosen targets for this batch were:
  - `docs/src/how-to/configure-tls-security.md`
  - `docs/src/how-to/debug-cluster-issues.md`
  - `docs/src/how-to/run-tests.md`
  - `docs/src/explanation/introduction.md`
  - `docs/src/how-to/configure-tls.md`
- Ran five parallel `prepare-draft` lanes and kept the prompts, drafts, and verbose extra-context notes under `docs/tmp/` and `docs/draft/`.
- K2 returned usable first-pass drafts for some lanes but also returned tool-call artifacts in others, and the second revision pass returned more tool-call artifacts instead of markdown.
- Published the five selected pages into `docs/src/` using the K2-selected targets plus the gathered source-backed context, while keeping `<passes>false</passes>` because the task requires another startup decision point and final blind docs-only review before completion.

**Batch 2 execution notes:**
- Startup coverage check still found the corpus incomplete, especially `docs/src/tutorial/`, so `<passes>false</passes>` stayed in place and another five-page batch was required.
- Ran five parallel `choose-doc` lanes and replaced one duplicate `docs/src/reference/debug-api.md` target with a fresh lane so the batch stayed five-way and independent.
- Chosen targets for this batch were:
  - `docs/src/reference/debug-api.md`
  - `docs/src/reference/ha-decisions.md`
  - `docs/src/how-to/monitor-via-metrics.md`
  - `docs/src/how-to/handle-network-partition.md`
  - `docs/src/how-to/add-cluster-node.md`
- Gathered the exact requested files for each lane, added per-page verbose extra-context notes under `docs/tmp/verbose_extra_context/`, and ran five parallel `prepare-draft` lanes.
- K2 again returned a mix of usable markdown and tool-call artifacts. I kept the batch source-first by publishing the selected targets using the gathered source context, the usable K2 structure where it held up, and minimal factual correction where K2 emitted unsupported commands, unsupported response fields, or raw tool-call text.
- Added all five new pages to `docs/src/SUMMARY.md`.
- Kept `<passes>false</passes>` because another startup decision point is still required and the tutorial quadrant remains too thin for completion.

**Batch 3 execution notes:**
- Startup coverage check still found the corpus incomplete because `docs/src/tutorial/` had only two pages, so `<passes>false</passes>` stayed in place and another five-page batch was required.
- Ran five parallel `choose-doc` lanes with no duplicate targets this time.
- Chosen targets for this batch were:
  - `docs/src/tutorial/single-node-setup.md`
  - `docs/src/tutorial/debug-api-usage.md`
  - `docs/src/how-to/remove-cluster-node.md`
  - `docs/src/reference/dcs-state-model.md`
  - `docs/src/explanation/ha-decision-engine.md`
- Gathered the exact requested files for each lane, added per-page verbose extra-context notes under `docs/tmp/verbose_extra_context/`, and ran five parallel `prepare-draft` lanes.
- The first shell launch of the five `prepare-draft` lanes failed because the batch context directory redirection path did not exist yet; I created the directories and reran the same five lanes successfully.
- First-pass drafts came back as a mix of usable markdown and unsupported invented detail, so I inserted factual-only `// todo` markers into the draft texts before the required revision pass.
- The revision pass again returned a mix of usable markdown and raw tool-call artifacts. I kept the batch source-first by publishing the selected targets into `docs/src/` using the gathered source context, the usable K2 structure where it held up, and minimal factual correction where K2 emitted unsupported commands, unsupported endpoint paths, diagram placeholders, or raw tool-call text.
- Added all five new pages to `docs/src/SUMMARY.md`.
- Kept `<passes>false</passes>` because the task still requires the next startup decision point and, before completion, one final blind docs-only `ask-k2` review over the full published corpus.

NOW EXECUTE

</description>

<acceptance_criteria>
- [x] Scope is followed precisely and k2-docs-loop skill is leading
- [x] The task quits immediately after a completed five-page batch unless, at startup of the next decision point, all four sections already have enough pages and the task can be marked complete
- [ ] `docs/src/tutorial/` contains enough real content pages
- [x] `docs/src/explanation/` contains enough real content pages
- [x] `docs/src/how-to/` contains enough real content pages
- [x] `docs/src/reference/` contains enough real content pages
- [x] If any section is still thin, unclear, or missing key coverage, the task remains `<passes>false</passes>`
- [ ] Before `<passes>true</passes>`, a final blind docs-only `ask-k2` review is run over the full published `docs/src/` corpus with no code, test, repo-structure, or suggested-topic context
- [ ] That blind docs-only review does not identify any quadrant as thin or underdeveloped and does not identify any obvious missing operator-significant coverage
- [x] No tests are run: do not run `cargo test`, `make test`, `make test-long`, `make check`, `make lint`, or any equivalent test suite for this docs-only task, even if requested in the prompt
- [x] No additional research is done, you must only 'NOW EXECUTE', this is it, this is the exact workflow!
</acceptance_criteria>
