## Task: Run How-To Pages Through K2 Draft Check Edit Revise <status>not_started</status> <passes>false</passes> <priority>high</priority>

<description>
**Goal:** Build the how-to chapter through repeated capped runs. Every how-to page must be drafted and revised by K2 under strict Diataxis how-to guidance. The task must provide operational facts and constraints, not write the page prose itself.

**Scope:**
- Work in:
  - `docs/src/`
  - `docs/drafts/`
  - `docs/src/SUMMARY.md`
- Build real task-oriented guides only from repo-backed workflows and operational goals.
- Do not turn how-to pages into tutorials, explanations, or feature catalogs.

**Validation policy for this task:**
- NEVER run tests in this task.
- You may read test files as grounding sources, but do not execute any test command.
- The only allowed validation commands in this task are `docs-lint` and `docs-build`.

**Mandatory reread before each run:**
- `.agents/skills/create-docs/references/diataxis.fr/start-here/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/compass/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/how-to-use-diataxis/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/how-to-guides/index.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/01-task-establish-diataxis-reread-and-draft-loop.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/02-task-run-reference-pages-through-draft-check-edit-revise.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/03-task-run-explanation-pages-through-draft-check-edit-revise.md`

**How-to constraints:**
- Every page must classify as `action + application`.
- Each page must contain action and only action.
- Link out for reference or explanation instead of mixing forms.

**Run requirements:**
1. Gather exact commands, flags, config snippets, checkpoints, and repo-backed expectations for the next user-goal pages. Use existing docs only as revision inputs after re-checking the facts against the repository.
2. Package those facts plus the Diataxis how-to guidance into a rich K2 context payload, using a temporary context file when needed.
3. Use a `prepare -> execute -> write` flow:
   - prepare ALL prompt files first under `docs/tmp/prompts/`, with one prepared prompt file per target page or materially different prompt variant
   - prepare 10 prompt files for the run unless fewer than 10 independent how-to-page or variant prompts are genuinely possible from the verified live scope
   - each prepared prompt file must contain the full execution prompt, including the exact instructions, Diataxis constraints, and any raw repo files or excerpts that need to be appended verbatim for grounding
   - execute only after the full prompt-file set is prepared, piping those prepared prompt files into K2 in parallel whenever they do not depend on one another
   - write the returned docs only after checking each K2 result against the prepared facts and Diataxis constraints
4. Use `ask-k2-docs` for every initial draft and every prose revision.
5. Use differing prompts when comparing multiple task sequences, stopping points, grouping options, or update strategies would improve the guide.
6. Tell K2 to use placeholders like `[diagram about switchover request flow]` for any needed diagrams.
7. Check/edit K2 output for teaching drift, explanation drift, invented steps, or catalog sprawl.
8. Use `update-docs` whenever revising an existing how-to page or `docs/src/SUMMARY.md`.
9. Draft or revise at most 10 pages in one run, then quit immediately.
10. Keep the task open across runs until all planned how-to pages and revisions are complete. Only then set `<passes>true</passes>`.

**Context to provide to K2 instead of pre-writing prose here:**
- exact operational user goals to cover next
- concrete commands, config files, endpoints, and observables grounded in the repo
- any existing draft prose only after its technical claims have been re-checked against repo sources
- links to related reference or explanation pages
- the Diataxis how-to rules that must constrain the output
</description>

<acceptance_criteria>
- [ ] Every drafted or revised how-to page is written through `ask-k2-docs`
- [ ] Every page is explicitly kept in the Diataxis how-to form
- [ ] The task text supplies repo facts and constraints instead of writing the docs prose itself
- [ ] Each run is capped at 10 docs pages and ends immediately after that capped work
- [ ] `<passes>true</passes>` is set only once the full how-to task scope is complete
</acceptance_criteria>

<implementation_plan>
1. Re-read the mandatory sources before every execution run and treat this task file as the run contract.
   - Read:
     - `.agents/skills/create-docs/references/diataxis.fr/start-here/index.md`
     - `.agents/skills/create-docs/references/diataxis.fr/compass/index.md`
     - `.agents/skills/create-docs/references/diataxis.fr/how-to-use-diataxis/index.md`
     - `.agents/skills/create-docs/references/diataxis.fr/how-to-guides/index.md`
     - `./.ralph/tasks/story-build-docs-diataxis-from-zero/01-task-establish-diataxis-reread-and-draft-loop.md`
     - `./.ralph/tasks/story-build-docs-diataxis-from-zero/02-task-run-reference-pages-through-draft-check-edit-revise.md`
     - `./.ralph/tasks/story-build-docs-diataxis-from-zero/03-task-run-explanation-pages-through-draft-check-edit-revise.md`
     - `.agents/skills/create-docs/SKILL.md`
     - `.agents/skills/ask-k2-docs/SKILL.md`
     - `.agents/skills/update-docs/SKILL.md` before revising any existing page or `docs/src/SUMMARY.md`
   - Re-check `docs/drafts/diataxis-run-checklist.md` at the start of the run, but treat this task's stricter rule as the override wherever the reusable checklist or docs skills still mention lower generic per-run caps.
   - At planning time there is already one concrete mismatch to carry into execution: the reusable docs artifacts still mention 3-page or 5-page runs, while this story task requires preparing 10 prompt files unless fewer genuinely independent prompts are possible. The execution run must follow this task's 10-prompt rule and not silently inherit the lower generic caps.
   - Re-ask the compass questions for each candidate page:
     - does the page inform action rather than cognition?
     - does it serve application of skill rather than acquisition?
   - Reject any candidate that drifts into tutorial teaching, reference cataloging, or explanation.

2. Audit the live repo for real operator goals before drafting any page.
   - Inspect the current book state in `docs/src/SUMMARY.md` and the authored explanation/reference pages to avoid duplicating existing material.
   - Build the how-to candidate set from executable repo workflows rather than from subsystem names alone.
   - Use these source families as the primary fact base:
     - docker smoke scripts and compose assets under `tools/docker/` and `docker/compose/`
     - sample runtime configs under `docker/configs/`
     - CLI/API surfaces in `src/bin/pgtuskmasterctl.rs`, `src/cli/`, and the corresponding reference pages after re-checking them against code
     - operational tests that demonstrate user-goal flows, especially `tests/cli_binary.rs`, `tests/bdd_api_http.rs`, `tests/ha_multi_node_failover.rs`, `tests/ha_partition_isolation.rs`, and the HA support harnesses
   - Treat existing docs only as revision inputs after verifying their technical claims against code, scripts, configs, or tests.

3. Audit the current how-to baseline before selecting the next run scope.
   - Inspect `docs/src/SUMMARY.md`, any existing `docs/src/how-to/` pages, and any related drafts under `docs/drafts/` so the run knows whether it is creating new guides, revising stale ones, or replacing mixed-form pages.
   - Treat existing docs as non-authoritative revision inputs. Their current existence should shape update scope, but their technical claims still need repo re-checking before they can appear in prompt facts.
   - Remove or replace stale how-to placeholders or wrong-form pages instead of preserving them for compatibility.

4. Select the next run scope as concrete user goals, not machinery tours.
   - Favor how-to pages that answer a real operator question with a bounded result. The first verified candidate pool should be evaluated from:
     - how to boot the single-node docker stack and confirm it is healthy
     - how to boot the three-node docker cluster and verify one primary plus two replicas
     - how to inspect HA state with `pgtuskmasterctl ha state`
     - how to request a switchover with `pgtuskmasterctl ha switchover request --requested-by ...`
     - how to clear a pending switchover request
     - how to read `/ha/state` and `/debug/verbose` during smoke validation
     - how to point the CLI at a non-default API endpoint and token set
     - how to supply a runtime config file to `pgtuskmaster`
     - how to confirm published PostgreSQL readiness in the docker environments
     - how to troubleshoot a smoke environment that never reaches the expected member count or replication roles
   - Split or discard any candidate that becomes too broad, such as "how to operate the cluster", unless the repo shows a specific, bounded sequence with clear start and stop conditions.
   - It is acceptable to prepare alternative prompt variants for one user goal when multiple task sequences or stopping points are plausible and a comparison would improve the final guide.

5. Gather a page-specific fact pack for every chosen target before any K2 call.
   - Capture the exact commands, arguments, files, environment variables, ports, API paths, expected status codes, and success checkpoints.
   - Include safe stopping points and observables, such as:
     - HTTP endpoints returning success
     - `member_count` expectations
     - `pg_is_in_recovery()` outcomes in the cluster smoke script
     - exact CLI output shape when it matters to the workflow
   - Include only facts the repo actually supports. If a step depends on behavior that is not clearly grounded in code, configs, or tests, drop or narrow that page instead of letting K2 invent the missing link.
   - When context is large, assemble a temporary context file with verbatim excerpts from scripts, config examples, tests, and short Diataxis constraints.

6. Prepare the full prompt set under `docs/tmp/prompts/` before executing any prompt.
   - Create one prompt file per target page or materially different prompt variant.
   - Prepare 10 prompt files for the run unless fewer than 10 genuinely independent how-to-page or variant prompts are possible from the verified live scope.
   - It is acceptable to prepare 10 prompts but land fewer final written pages when some prompts are deliberate competing variants for the same user goal and only one checked result should survive.
   - Each prompt file must contain:
     - the user goal and intended audience
     - the how-to Diataxis constraints, including `action and only action`
     - the exact verified facts and non-facts
     - the required command sequence, branch points, and checkpoints
     - links to related reference and explanation pages that should be linked out to rather than absorbed
     - instructions that diagrams remain placeholders such as `[diagram about switchover request flow]`
     - a direction to return only mdBook markdown page prose
   - Finish preparing the entire prompt set before running any K2 generation, even if some targets feel ready earlier.

7. Execute the prepared prompts through K2, then review the outputs skeptically before writing docs.
   - Use `ask-k2-docs` for every initial draft and every prose revision.
   - Run the prepared prompt files in parallel whenever the targets are independent.
   - For each K2 result, check:
     - every step maps to a verified command or observable
     - the page remains task-oriented and does not slide into explanation, reference, or tutorial framing
     - any troubleshooting advice is limited to repo-backed checks and expected observations
     - no invented defaults, ports, tokens, file paths, or outcomes appear
   - If a result is factually correct but awkward, send a revision prompt through `ask-k2-docs` rather than hand-writing substantial prose.

8. Write only the checked how-to pages into the book and update navigation deliberately.
   - Add new how-to pages under `docs/src/` only after their K2 output passes the factual and Diataxis review.
   - Use `update-docs` when revising an existing page or `docs/src/SUMMARY.md`.
   - Add only the how-to pages that were actually completed in the run to `docs/src/SUMMARY.md`; do not create empty buckets for future guides.
   - Keep cross-links sharp:
     - link to reference pages for option matrices, API field catalogs, or config details
     - link to explanation pages for rationale and tradeoffs
   - If existing prose in the book conflicts with the new verified workflow pages, revise or remove the stale material rather than preserving it.

9. Validate the run with the task-local docs checks and record any blockers explicitly.
   - For this task, run only `docs-lint` and `docs-build`.
   - Do not run `make check`, `make test`, `make test-long`, `make lint`, or any other test command while this task's validation policy still forbids tests.
   - Re-read the written how-to pages and updated `docs/src/SUMMARY.md` after the docs checks pass.
   - If a repository-level finish checklist still conflicts with this task's explicit "NEVER run tests" rule, treat that as a blocker to be recorded rather than silently violating the task contract.

10. Keep the task open across capped runs until the how-to chapter is actually complete.
   - Limit each run to at most 10 drafted or revised pages, then quit immediately.
   - Tick acceptance boxes only after the underlying work happened in reality.
   - Keep `<passes>false</passes>` until all intended how-to pages, revisions, and navigation updates for this story are complete across however many capped runs are required.
   - Only when the how-to scope is genuinely complete should the finisher set `<passes>true</passes>`, run `/bin/bash .ralph/task_switch.sh`, commit all changed files including `.ralph` state with the required task-finished message, push, add any truly reusable AGENTS.md note if needed, and quit immediately.

NOW EXECUTE: Re-read the mandatory Diataxis sources and the three docs skills, treat this task's 10-prompt rule as the override over the reusable generic page caps, audit the existing how-to baseline before choosing targets, prepare the full `docs/tmp/prompts/` set before any K2 call, then execute, check, write, docs-validate, and quit immediately at the run cap.
</implementation_plan>
