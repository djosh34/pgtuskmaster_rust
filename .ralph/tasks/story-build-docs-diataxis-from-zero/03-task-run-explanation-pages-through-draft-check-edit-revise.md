## Task: Run Explanation Pages Through K2 Draft Check Edit Revise <status>completed</status> <passes>true</passes> <priority>high</priority>

<description>
**Goal:** Build the explanation chapter through repeated capped runs. Every explanation page must be drafted and revised by K2 under strict Diataxis explanation guidance. The task must provide source context, tensions, and grounding, not hand-write the explanation prose.

**Scope:**
- Work in:
  - `docs/src/`
  - `docs/drafts/`
  - `docs/src/SUMMARY.md`
- Build real explanation pages only where the codebase supports meaningful context and rationale.
- Do not turn explanation pages into reference dumps or how-to procedures.

**Validation policy for this task:**
- NEVER run tests in this task.
- You may read test files as grounding sources, but do not execute any test command.
- The only allowed validation commands in this task are `docs-lint` and `docs-build`.

**Mandatory reread before each run:**
- `.agents/skills/create-docs/references/diataxis.fr/start-here/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/compass/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/how-to-use-diataxis/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/explanation/index.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/01-task-establish-diataxis-reread-and-draft-loop.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/02-task-run-reference-pages-through-draft-check-edit-revise.md`

**Explanation constraints:**
- Every page must classify as `cognition + acquisition`.
- Explanation should provide context, reasons, tradeoffs, alternatives, and consequences.
- If the page turns procedural or catalog-like, split or replace it.

**Run requirements:**
1. Gather the relevant code paths, tests, design tensions, cross-links, and runnable behavior before drafting. Use existing docs only as revision inputs after re-checking their claims against the repository.
2. Package those facts plus the Diataxis explanation guidance into rich K2 context, using a temporary context file when helpful.
3. Use a `prepare -> execute -> write` flow:
   - prepare ALL prompt files first under `docs/tmp/prompts/`, with one prepared prompt file per target page or materially different prompt variant
   - prepare 10 prompt files for the run unless fewer than 10 independent explanation-page or variant prompts are genuinely possible from the verified live scope
   - each prepared prompt file must contain the full execution prompt, including the exact instructions, Diataxis constraints, and any raw repo files or excerpts that need to be appended verbatim for grounding
   - execute only after the full prompt-file set is prepared, piping those prepared prompt files into K2 in parallel whenever they do not depend on one another
   - write the returned docs only after checking each K2 result against the prepared facts and Diataxis constraints
4. Use `ask-k2-docs` for all explanation-page prose drafts and prose revisions.
5. Use meaningfully different prompts when comparing alternative explanatory frames, structures, or update strategies.
6. Tell K2 to leave diagram placeholders such as `[diagram about DCS trust inputs]` instead of inventing diagrams.
7. Check/edit K2 output for factual invention, shallow hand-waving, procedural drift, or reference dumping.
8. Use `update-docs` for revisions to existing explanation pages or to `docs/src/SUMMARY.md`.
9. Draft or revise at most 10 pages in a single run, then quit immediately.
10. Keep `<passes>false</passes>` until the whole explanation task scope is complete across however many runs are needed.

**Context to provide to K2 instead of pre-writing prose here:**
- exact modules, configs, tests, and control flows relevant to each topic
- any existing draft prose only after its technical claims have been re-checked against repo sources
- tensions or design questions the page should illuminate
- links to related reference or how-to pages
- the Diataxis explanation rules that must constrain the output
</description>

<acceptance_criteria>
- [x] Every drafted or revised explanation page is written through `ask-k2-docs`
- [x] Every page is explicitly kept in the Diataxis explanation form
- [x] The task text supplies grounding context and constraints instead of writing the docs prose itself
- [x] Each run is capped at 10 docs pages and ends immediately after that capped work
- [x] `<passes>true</passes>` is set only once the full explanation task scope is complete
</acceptance_criteria>

<implementation_plan>
1. Re-read the mandatory sources for every execution run and re-check the local workflow artifacts before touching page content.
   - Read:
     - `.agents/skills/create-docs/references/diataxis.fr/start-here/index.md`
     - `.agents/skills/create-docs/references/diataxis.fr/compass/index.md`
     - `.agents/skills/create-docs/references/diataxis.fr/how-to-use-diataxis/index.md`
     - `.agents/skills/create-docs/references/diataxis.fr/explanation/index.md`
     - `./.ralph/tasks/story-build-docs-diataxis-from-zero/01-task-establish-diataxis-reread-and-draft-loop.md`
     - `./.ralph/tasks/story-build-docs-diataxis-from-zero/02-task-run-reference-pages-through-draft-check-edit-revise.md`
     - `.agents/skills/create-docs/SKILL.md`
     - `.agents/skills/ask-k2-docs/SKILL.md`
     - `.agents/skills/update-docs/SKILL.md` before any revision to existing pages or `docs/src/SUMMARY.md`
   - Re-check `docs/drafts/diataxis-run-checklist.md` before the run so any generic guidance that conflicts with this task is caught explicitly instead of leaking into execution. At planning time, there is already one mismatch to account for: the checklist says to pick at most 3 pages, while this explanation task requires preparing 10 prompt files unless fewer than 10 genuinely independent explanation-page or variant prompts are possible.
   - Treat this task file's explicit execution rules as the override when the reusable checklist or `create-docs` skill expresses a lower generic per-run page cap. The generic artifacts still matter for process shape, but the verified explanation-task cap is `prepare 10 prompt files unless fewer are genuinely possible`.

2. Audit the live docs baseline and identify the next explanation-scope slice from the real repository rather than from guesses.
   - Inspect `docs/src/SUMMARY.md` and `docs/src/reference/` to see which machinery areas already have reference coverage and which ones still need explanation pages to provide rationale, tradeoffs, or conceptual framing.
   - Build a concrete candidate list of explanation topics backed by code and tests. Favor topics that naturally answer "why" questions around the existing reference surfaces, such as DCS trust and coordination, HA state transitions, node-runtime responsibilities, process-worker boundaries, shared-state semantics, config layering, TLS design tradeoffs, debug-vs-HTTP API separation, managed PostgreSQL abstractions, and pginfo observation boundaries.
   - Reject candidates that would become procedures, command walkthroughs, or reference dumps. Split or discard any topic that does not clearly classify as `cognition + acquisition`.

3. Decide the exact run scope before drafting and keep it capped.
   - Target one execution run of at most 10 explanation pages or materially different prompt variants.
   - Prepare exactly 10 prompt files under `docs/tmp/prompts/` when the verified candidate set can support that many independent explanation pages or variants.
   - It is acceptable for the run to prepare 10 prompts but land fewer than 10 final written pages when some prompts are deliberate alternative explanatory frames for the same topic and only one checked result should survive.
   - If the verified scope supports fewer than 10 genuinely independent explanation prompts, record that limitation in the task file or progress log before execution so the reduced count is explained by repository reality rather than convenience.
   - Keep this task open across runs until the explanation chapter is complete; do not set `<passes>true</passes>` after only one capped run unless the whole explanation scope is actually finished.

4. Gather the repo facts for each chosen explanation page before any K2 execution.
   - For every target page, collect the exact code paths, config files, relevant tests, runtime entry points, and any existing reference pages that define the machinery the explanation will discuss.
   - Treat existing docs only as revision inputs after re-checking their claims against repo sources.
   - Capture the design tensions each page must illuminate:
     - reasons for the current design
     - tradeoffs and rejected alternatives
     - boundaries between adjacent subsystems
     - consequences for operators or developers
   - When the verified context is large, assemble a temporary context file so K2 receives raw excerpts and exact file grounding instead of a thin summary.

5. Prepare all K2 prompt files first, then execute them in parallel.
   - Create one fully populated prompt file per target page or materially different explanatory framing under `docs/tmp/prompts/`.
   - Each prompt file must include:
     - the page goal, audience, and user need
     - the explanation-form Diataxis constraints
     - the verified facts and non-facts
     - the required page boundaries or headings
     - any raw repo excerpts needed for grounding
     - the instruction that diagrams remain placeholders such as `[diagram about DCS trust inputs]`
   - Use `ask-k2-docs` for every initial draft and every prose revision.
   - Only after the full prompt-file set is prepared should the run pipe those prompt files into K2 in parallel where pages are independent.

6. Check and edit each K2 result before writing it into the book.
   - Verify every factual claim against the prepared repo evidence.
   - Remove any procedural drift, reference-style cataloging, or hand-wavy explanation that does not actually answer a "why", "what tradeoff", or "how should this be understood" question.
   - Keep explanation pages bounded. If a result tries to absorb step-by-step instructions or neutral API facts, move or cut that content instead of mixing forms.
   - If prose quality still needs work after factual correction, run another `ask-k2-docs` revision using the corrected facts and specific rewrite guidance.

7. Write only the checked explanation outputs and update navigation deliberately.
   - Add or revise explanation pages in `docs/src/` only after their K2 output has passed the skeptical review.
   - Use `update-docs` when revising an existing docs page or `docs/src/SUMMARY.md`.
   - Ensure the navigation labels distinguish explanation pages from the existing reference chapter and do not create empty structural buckets.
   - Remove or replace any stale explanation draft or placeholder structure encountered during the run instead of preserving it for backwards compatibility.

8. Validate the run using the task-local docs policy and record any blocker explicitly.
   - For this task, obey the task-local validation contract: run `docs-lint` and `docs-build`, and do not run test commands that the task text forbids.
   - If repository-level finish instructions still appear to require commands that conflict with this task's explicit "NEVER run tests" policy, stop and record that conflict as a blocker instead of silently violating the task contract.
   - Re-read the written pages and `docs/src/SUMMARY.md` skeptically after the tool-based docs checks pass.

9. Finish only after the explanation scope for this task is actually complete.
   - Tick acceptance boxes only after the corresponding work has been done in reality.
   - Keep `<passes>false</passes>` until the explanation chapter for this story is fully complete across however many capped runs are needed.
   - Once the task is genuinely complete, set `<passes>true</passes>`, run `/bin/bash .ralph/task_switch.sh`, commit all changes including `.ralph` state with the required `task finished ...` message, push, add any truly reusable AGENTS.md learning if needed, and quit immediately.

NOW EXECUTE: Re-read the mandatory Diataxis sources and the three docs skills, treat this task's 10-prompt rule as the override over the reusable generic caps, prepare the full `docs/tmp/prompts/` set before any K2 call, then execute, check, write, docs-validate, and quit immediately at the run cap.
</implementation_plan>
