## Task: Run Reference Pages Through K2 Draft Check Edit Revise <status>in_progress</status> <passes>false</passes> <priority>high</priority>

<description>
**Goal:** Build the reference chapter through repeated capped runs. Every reference page must be drafted and revised by K2 under strict Diataxis reference guidance. The agent must gather facts and shape prompts, not pre-write the reference prose in this task file.

**Scope:**
- Work in:
  - `docs/src/`
  - `docs/drafts/`
  - `docs/src/SUMMARY.md`
- Build real reference pages only from actual machinery boundaries.
- Do not add empty sections or speculative placeholders.

**Validation policy for this task:**
- NEVER run tests in this task.
- You may read test files as grounding sources, but do not execute any test command.
- The only allowed validation commands in this task are `docs-lint` and `docs-build`.

**Mandatory reread before each run:**
- `.agents/skills/create-docs/references/diataxis.fr/start-here/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/compass/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/how-to-use-diataxis/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/reference/index.md`
- `./.ralph/tasks/story-build-docs-diataxis-from-zero/01-task-establish-diataxis-reread-and-draft-loop.md`

**Reference constraints:**
- Every page must classify as `cognition + application`.
- Reference must `describe and only describe`.
- Structure should mirror the machinery, not a guessed user journey.
- If explanation or procedure appears necessary, link to the right page type instead of mixing forms.

**Run requirements:**
1. Gather the concrete facts for the next candidate pages directly from code, config, tests, CLI/API surfaces, and runnable behavior. Use existing docs only as revision inputs after re-checking the facts against the repository.
2. Build rich K2 prompt context from those facts and from the Diataxis reference guidance. Use a temporary context file whenever that is the clearest way to pass enough material.
3. Use a `prepare -> execute -> write` flow:
   - prepare the K2 input files first, with one prepared input per target page or materially different prompt variant
   - execute the prepared K2 generations after preparation is complete, running multiple independent K2 doc generations in parallel whenever the prepared inputs do not depend on one another
   - write the returned docs only after checking each K2 result against the prepared facts and Diataxis constraints
4. Use `ask-k2-docs` for every initial reference draft and every prose revision.
5. Use differing prompts when comparing alternative page structures, coverage splits, or update strategies would be useful.
6. Ask K2 for prose and structure only. If a diagram would help, instruct it to emit a placeholder such as `[diagram about runtime config loading]`.
7. Check/edit K2 output for invented facts, explanation drift, procedural drift, or duplicated pages.
8. Use `update-docs` when revising an existing reference page or `docs/src/SUMMARY.md`.
9. Draft or revise at most 10 pages in a single run, then quit immediately.
10. Keep the task open across runs until the reference chapter for this story is complete. Only then set `<passes>true</passes>`.

**Context to provide to K2 instead of pre-writing prose here:**
- current reference-page targets and any already-authored replacements
- exact source modules and tests that ground each page
- any existing draft prose only after its technical claims have been re-checked against repo sources
- required boundaries, terms, and non-goals
- the Diataxis reference rules that must constrain the output
</description>

<acceptance_criteria>
 - [x] Every drafted or revised reference page is written through `ask-k2-docs`
 - [x] Every page is explicitly kept in the Diataxis reference form
 - [x] The task text supplies context sources and constraints instead of writing the docs prose itself
 - [x] Each run is capped at 10 docs pages and ends immediately after that capped work
 - [ ] `<passes>true</passes>` is set only once the full reference task scope is complete
</acceptance_criteria>

<implementation_plan>
1. Resume from skeptical-review completion into direct execution readiness.
   - Treat this reviewed plan as the authority for the next run and replace the verification marker with `NOW EXECUTE` once at least one concrete improvement has been made during review.
   - Keep `<passes>false</passes>` unchanged until the reference chapter is actually complete.
   - Leave the acceptance criteria intact unless the repository state proves they are incomplete or inaccurate.

2. Reconfirm the hard task contract before any future execution run.
   - Re-read the mandatory Diataxis sources listed in this task and re-read task `01` so the run starts from the story-wide workflow contract rather than memory.
   - Reconfirm that this task is reference-only: `cognition + application`, `describe and only describe`, structure led by machinery boundaries rather than by a user journey.
   - Reconfirm that this task may touch only `docs/src/`, `docs/drafts/`, and `docs/src/SUMMARY.md`, and that each execution run must stop after at most 10 docs pages are drafted or revised.
   - Reconfirm that the validation policy written in this task allows only `docs-lint` and `docs-build` during execution of this task, even if broader repository completion gates exist elsewhere.

3. Re-establish the actual repository baseline at the start of every execution run.
   - Inspect `git status --short` first so the run distinguishes this slice's intended docs work from unrelated in-progress changes already present in the worktree.
   - Inspect the current `docs/src/` tree and `docs/src/SUMMARY.md` to see which reference pages already exist, which entries are stale, and which real machinery boundaries remain undocumented.
   - Treat existing docs as revision inputs only after re-checking their claims against the source tree.
   - Treat `docs/drafts/diataxis-run-checklist.md` as process scaffolding only, never as a factual source.
   - Use `src/lib.rs` and the concrete module tree as the boundary map for deciding whether a page is a real reference candidate.
   - Build a quick gap matrix between the live reference pages and the current module families before choosing the run scope, so the next slice is selected from present evidence rather than from remembered backlog.

4. Maintain a source-backed inventory for all candidate reference pages.
   - Runtime configuration pages must be grounded in `src/config/mod.rs`, `src/config/schema.rs`, `src/config/parser.rs`, `src/config/defaults.rs`, and any checked-in runtime config examples under `docker/configs/`.
   - Node runtime and process-surface pages must be grounded in `src/bin/pgtuskmaster.rs`, `src/runtime/mod.rs`, `src/runtime/node.rs`, `src/process/*.rs`, and `src/postgres_managed*.rs`.
   - Control-plane CLI pages must be grounded in `src/bin/pgtuskmasterctl.rs`, `src/cli/args.rs`, `src/cli/client.rs`, `src/cli/output.rs`, and `tests/cli_binary.rs`.
   - HTTP/API reference pages must be grounded in `src/api/mod.rs`, `src/api/controller.rs`, `src/api/worker.rs`, `tests/bdd_api_http.rs`, and `tests/policy_e2e_api_only.rs`.
   - HA and DCS pages must be grounded in `src/ha/*.rs`, `src/dcs/*.rs`, `src/state/*.rs`, `tests/ha_multi_node_failover.rs`, `tests/ha_partition_isolation.rs`, and `tests/bdd_state_watch.rs`.
   - Shared-state, logging, TLS, and PostgreSQL observation pages must be grounded in their current implementation modules and integration tests rather than treated as already-settled because pages with those names already exist.
   - Add new candidates only when the repository shows a real stable machinery boundary; do not invent pages to satisfy an imagined chapter outline.

5. Choose the next capped execution slice from the verified inventory, not from memory.
   - For each run, pick at most 10 docs pages total, and count any substantial `docs/src/SUMMARY.md` restructuring toward that cap.
   - Prefer maintenance of existing reference pages when they are stale, duplicated, over-broad, or structurally misplaced; do not force new pages when cleanup is the higher-value move.
   - Prefer the earliest pages that create or tighten a coherent reference spine for later explanation, how-to, and tutorial work.
   - Re-select the run scope after re-checking the live tree in the same run; do not trust any previous run summary as authoritative.

6. Package explicit factual context for K2 before every draft or revision.
   - Use `ask-k2-docs` for every initial draft and every prose revision.
   - Build a source-first prompt payload containing the page goal, Diataxis reference constraints, exact grounding files, verified commands/flags/endpoints/fields/enum values, non-goals, and any facts that must not be generalized or invented.
   - Include existing draft prose only after each technical claim in that prose has been re-checked against repository sources.
   - Prefer a temporary context file when the page depends on many symbols, routes, or config fields.
   - Prepare the full input-file set before executing K2 so the run can batch independent generations in parallel.
   - When coverage splits or page boundaries are ambiguous, compare materially different K2 prompts instead of repeatedly nudging a single framing.

7. Run a skeptical reference-editing loop on every K2 output before patching docs.
   - Ask K2 only for prose and structure. If visual support would help, require a placeholder rather than an invented diagram.
   - Reject, rewrite, or narrow any output that explains rationale, gives procedures, invents unsupported defaults or guarantees, or merges separate machinery boundaries into one page.
   - Keep examples short and descriptive rather than tutorial-like.
   - Use `update-docs` whenever revising an existing reference page or `docs/src/SUMMARY.md`.
   - Update navigation only after the corresponding authored pages justify the entry; never create empty future-facing buckets.

8. Validate each capped execution run with the docs-only commands allowed by this task.
   - Run `make docs-lint` after the chosen page set is written.
   - Run `make docs-build` after lint passes.
   - If either command fails, fix the in-scope docs issues before ending the run.
   - Do not run `make check`, `make test`, `make test-long`, or `make lint` as part of this task unless the task contract itself is later changed to allow them.

9. Keep the task open until the reference chapter is actually exhausted and consistent.
   - After each run, tick only the acceptance boxes whose underlying work has really happened.
   - Leave `<passes>false</passes>` until the repo contains the full intended reference chapter for this story, the related navigation is complete, and the final capped run passes both `make docs-lint` and `make docs-build`.
   - Once the obvious page backlog is exhausted, do a skeptical reread of the authored reference set and `docs/src/SUMMARY.md` to confirm there are no missing high-value machinery boundaries, duplicated pages, or form drift into explanation or how-to content.

10. Only after that skeptical completion check should the task move into final completion handling.
   - Set `<passes>true</passes>` only when the reference chapter work for this task is truly complete.
   - Before claiming final completion, explicitly reconcile the broader repository completion gates from the operator workflow (`make check`, `make test`, `make test-long`, `make lint`, and docs updates) against this task's own docs-only execution contract, so the finisher does not silently violate either rule set.
   - Then perform the Ralph task-switch, commit, and push sequence required by the repository workflow.

NOW EXECUTE
</implementation_plan>
