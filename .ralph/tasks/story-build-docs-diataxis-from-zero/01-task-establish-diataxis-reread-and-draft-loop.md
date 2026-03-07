## Task: Establish Diataxis Reread And K2 Draft Loop <status>in_progress</status> <passes>false</passes> <priority>high</priority>

<description>
**Goal:** Establish the documentation-production method for this story. This task defines how later docs tasks must gather repo facts, ground themselves in Diataxis, and use K2 for all prose drafting and prose revision. It must not author final docs pages itself.

**Scope:**
- Work only in:
  - `docs/drafts/`
  - `.ralph/tasks/story-build-docs-diataxis-from-zero/`
- Do not write final docs pages under `docs/src/` in this task.
- Do not create speculative mdBook structure.
- A minimal workflow helper under `docs/drafts/` is allowed only if it is factual process scaffolding and not page prose.

**Validation policy for this task:**
- NEVER run tests in this task.
- You may read test files as repo facts, but do not execute any test command.
- The only allowed validation commands in this task are `docs-lint` and `docs-build`.

**Mandatory source reread before every later docs run:**
- `.agents/skills/create-docs/references/diataxis.fr/start-here/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/compass/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/how-to-use-diataxis/index.md`
- plus the form-specific Diataxis page for the current task

**Required method for later docs tasks:**
1. Re-read the relevant Diataxis sources at the start of every run. Use the correct form language: tutorial, how-to, reference, explanation.
2. Gather facts directly from code, config, tests, runnable assets, and the Diataxis references. The task text must provide context sources and constraints, but must not try to write the docs prose itself.
3. Do not treat prior drafts, existing docs pages, or task text as authoritative fact sources. They may be used only as inputs to revise structure or wording after the underlying facts are re-checked against the repository.
4. Use the `ask-k2-docs` skill for every docs draft and every prose revision. The agent must not hand-write final docs prose except tiny factual repairs during the final verification task.
5. Use the `update-docs` skill whenever an existing docs page or mdBook navigation page is being revised or promoted.
6. Give K2 a large, explicit context payload instead of a thin summary when needed:
   - create a temporary context file if that is the clearest way to package repo facts and Diataxis excerpts
   - pipe that context into the K2/opencode workflow used by `ask-k2-docs`
   - include long relevant Diataxis excerpts or summaries when they help keep the form strict
7. Generate multiple materially different K2 prompts when comparing structure, tone, or update strategy would improve the page. Do not ask the same prompt repeatedly with tiny wording changes.
8. When revising or promoting docs, ask K2 not only for better prose but also for how the page or docs structure should be updated continuously as the docs set grows, while still staying inside Diataxis boundaries.
9. Tell K2 to write only the page prose. For diagrams, instruct it to leave placeholders such as `[diagram about failover state transitions]`.
10. Each execution run may draft or revise at most 3 docs pages. After the capped work for that run is complete, quit immediately.
11. A task is not complete just because one run finished. Keep `<passes>false</passes>` until all pages, revisions, and related navigation work required by that specific task are fully done.
12. Tick task checkboxes only after the underlying work actually happened in that run. Do not mark planning expectations as complete ahead of execution.
13. Only set `<passes>true</passes>` once the entire task scope is complete and the required verification for that task has passed.

**Expected outcome:**
- The story uses a K2-authored, Diataxis-grounded docs workflow.
- Later task files give the agent enough repo context and source references to drive K2 well, without pre-writing the documentation themselves.
- Later runs stop after at most 3 docs pages per run and resume in subsequent runs until the task is actually complete.
</description>

<acceptance_criteria>
- [ ] The task clearly requires `ask-k2-docs` for all docs prose drafting and prose revision
- [ ] The task clearly requires Diataxis rereads before each docs run
- [ ] The task clearly limits each run to at most 3 docs pages before quitting immediately
- [ ] The task clearly states that `<passes>true</passes>` is allowed only after the full task scope is complete
- [ ] The task clearly directs agents to provide K2 with rich repo and Diataxis context rather than writing docs prose in the task file
</acceptance_criteria>

<implementation_plan>
1. Confirm the current baseline and identify the exact gaps this task must close.
   - Re-read this task file, the downstream story tasks `02` through `08`, and the skill instructions for `create-docs`, `ask-k2-docs`, and `update-docs`.
   - Re-check the current `docs/drafts/` contents so the plan does not add unnecessary helper artifacts or assume missing scaffolding.
   - Compare their wording against the goal of this task: later docs work must be grounded in repeated Diataxis rereads, repo-fact gathering, K2-only prose drafting/revision, capped page counts, and delayed `<passes>true</passes>`.
   - Record the specific mismatches, ambiguities, or missing operational details before editing anything.

2. Tighten this task so it becomes the authoritative workflow contract for the story.
   - Revise the description and run requirements in this file so they clearly define the loop later engineers must follow on every docs run.
   - Make the contract operational rather than aspirational:
     - the mandatory reread set must be explicit
     - the agent must gather facts from repo sources rather than from prior draft prose
     - prior prose must never be treated as a truth source without repo re-checking
     - `ask-k2-docs` must be required for every initial draft and prose revision
     - `update-docs` must be required for revisions/promotions of existing pages and navigation pages
     - large, explicit K2 context payloads and temporary context files must be encouraged when needed
     - materially different prompts must be used when comparing structure, framing, or update strategy
     - K2 must be told to produce only page prose and diagram placeholders
     - each execution run must stop after at most 3 pages
     - checkbox ticking must happen only after execution
     - `<passes>true</passes>` must remain forbidden until the full task scope is complete

3. Decide whether a reusable workflow artifact is needed under `docs/drafts/`.
   - Inspect the current `docs/drafts/` state and avoid creating ceremony if the story tasks alone already give enough instruction.
   - If a reusable artifact is needed, add a minimal, factual helper file under `docs/drafts/` that supports later runs without pre-writing docs prose.
   - Acceptable artifact types:
     - a K2 context-packing template
     - a run checklist for reread plus fact gathering plus draft-check-edit-revise
   - Reject artifacts that would become speculative mdBook structure, fake content, or prose templates that encourage invention.

4. Align downstream story tasks with the workflow contract from this task.
   - Review tasks `02` through `08` after tightening task `01`.
   - Update only the downstream tasks whose wording is weaker, inconsistent, or missing an operational instruction now mandated by task `01`.
   - Focus on consistency for:
     - reread prerequisites
     - mandatory use of `ask-k2-docs`
     - use of `update-docs` when revising existing pages
     - rich repo-context packaging for K2
     - run caps and immediate quit behavior
     - delayed `<passes>true</passes>` semantics
   - Keep each downstream task specific to its document form or verification role instead of duplicating large blocks of generic text unnecessarily.

5. Verify the task-story changes skeptically before declaring the plan executable.
   - Re-read the edited task files as if they will drive future engineers with no extra context.
   - Check for contradictions, hidden assumptions, duplicated instructions that can drift, or any wording that still allows hand-written final prose in task files.
   - Confirm that this task still stays within scope:
     - only `docs/drafts/`
     - only `.ralph/tasks/story-build-docs-diataxis-from-zero/`
     - no final docs pages under `docs/src/`
     - no speculative mdBook structure
   - Confirm the acceptance criteria in this file are directly satisfied by the edited wording rather than merely implied.

6. Run only the allowed docs validation before completion.
   - Execute only:
     - `docs-lint`
     - `docs-build`
   - Do not run `make check`, `make test`, `make test-long`, or any other test command in this task.
   - If an allowed docs validation fails, fix the relevant task/workflow files or other in-scope artifacts and re-run until the allowed docs validations pass.
   - Use the `update-docs` skill if docs changes outside the story-task wording become necessary to keep the repo documentation truthful and current.

7. Finish the task only after all validations and doc obligations are complete.
   - Tick off the acceptance criteria and any execution checkboxes in the edited task files only after the corresponding work is actually complete.
   - Set `<passes>true</passes>` in this task only when the workflow contract is fully established and all required checks pass.
   - Run `/bin/bash .ralph/task_switch.sh`.
   - Commit all changed files, including `.ralph` state, with a message of the required form and with evidence about completed checks.
   - Push the branch.
   - Add a short AGENTS.md note only if a genuinely reusable lesson emerged that future engineers would otherwise miss.

NOW EXECUTE
</implementation_plan>
