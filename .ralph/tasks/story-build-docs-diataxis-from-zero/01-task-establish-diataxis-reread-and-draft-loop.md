## Task: Establish Diataxis Reread And K2 Draft Loop <status>not_started</status> <passes>false</passes> <priority>high</priority>

<description>
**Goal:** Establish the documentation-production method for this story. This task defines how later docs tasks must gather repo facts, ground themselves in Diataxis, and use K2 for all prose drafting and prose revision. It must not author final docs pages itself.

**Scope:**
- Work only in:
  - `docs/drafts/`
  - `.ralph/tasks/story-build-docs-diataxis-from-zero/`
- Do not write final docs pages under `docs/src/` in this task.
- Do not create speculative mdBook structure.

**Mandatory source reread before every later docs run:**
- `.agents/skills/create-docs/references/diataxis.fr/start-here/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/compass/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/how-to-use-diataxis/index.md`
- plus the form-specific Diataxis page for the current task

**Required method for later docs tasks:**
1. Re-read the relevant Diataxis sources at the start of every run. Use the correct form language: tutorial, how-to, reference, explanation.
2. Gather facts directly from code, config, tests, runnable assets, and the Diataxis references. The task text must provide context sources and constraints, but must not try to write the docs prose itself.
3. Use the `ask-k2-docs` skill for every docs draft and every prose revision. The agent must not hand-write final docs prose except tiny factual repairs during the final verification task.
4. Use the `update-docs` skill whenever an existing docs page or mdBook navigation page is being revised or promoted.
5. Give K2 a large, explicit context payload instead of a thin summary when needed:
   - create a temporary context file if that is the clearest way to package repo facts and Diataxis excerpts
   - pipe that context into the K2/opencode workflow used by `ask-k2-docs`
   - include long relevant Diataxis excerpts or summaries when they help keep the form strict
6. Generate multiple materially different K2 prompts when comparing structure, tone, or update strategy would improve the page. Do not ask the same prompt repeatedly with tiny wording changes.
7. When revising or promoting docs, ask K2 not only for better prose but also for how the page or docs structure should be updated continuously as the docs set grows, while still staying inside Diataxis boundaries.
8. Tell K2 to write only the page prose. For diagrams, instruct it to leave placeholders such as `[diagram about failover state transitions]`.
9. Each execution run may draft or revise at most 3 docs pages. After the capped work for that run is complete, quit immediately.
10. A task is not complete just because one run finished. Keep `<passes>false</passes>` until all pages, revisions, and related navigation work required by that specific task are fully done.
11. Only set `<passes>true</passes>` once the entire task scope is complete and the required verification for that task has passed.

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
