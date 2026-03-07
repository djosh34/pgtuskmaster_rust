## Task: Run Final Accuracy Verification And Create Bugs <status>not_started</status> <passes>false</passes> <priority>high</priority>

<description>
**Goal:** Perform the final accuracy-only verification pass after the authoring and navigation tasks are complete. This task verifies K2-authored docs against the repository and creates bug tasks for unsupported or inaccurate claims. It is not a drafting task.

**Scope:**
- Work in:
  - `docs/src/`
  - `docs/verifications/`
  - `.ralph/tasks/bugs/`
  - `.ralph/tasks/story-build-docs-diataxis-from-zero/`
- Focus on truth only:
  - factual claims
  - command names and behavior
  - config names and defaults
  - architectural statements
  - step accuracy
  - cross-page contradictions

**Validation policy for this task:**
- NEVER run tests in this task.
- You may read test files as verification evidence, but do not execute any test command.
- The only allowed validation commands in this task are `docs-lint` and `docs-build`.

**Mandatory reread before each run:**
- `.agents/skills/create-docs/references/diataxis.fr/start-here/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/compass/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/how-to-use-diataxis/index.md`
- `./.agents/skills/add-bug/SKILL.md`
- all story tasks in `./.ralph/tasks/story-build-docs-diataxis-from-zero/`

**Verification constraints:**
- Verify docs against code, config, tests, commands, and runnable behavior, not against earlier drafts.
- Create bug tasks for inaccuracies instead of silently tolerating them.
- Only do tiny inline doc corrections when they are safe and unavoidable to complete the verification pass.
- Do not let verification notes or prior prose override what the repository actually shows.

**Run requirements:**
1. Review the current authored docs and identify the next verification slice.
2. Check each claim against repo facts and runnable behavior where needed.
3. Record verification artifacts under `docs/verifications/`.
4. Use the `add-bug` skill for every factual inaccuracy, unsupported claim, contradiction, or major Diataxis-form error that should be tracked separately.
5. If a tiny doc correction is unavoidable, use `update-docs` for that edit instead of freehand drifting into another drafting pass.
6. Verify at most 3 docs pages per run, then quit immediately.
7. Keep `<passes>false</passes>` until the full authored-doc set has been verified and every necessary bug has been created.

**Context to provide during verification instead of rewriting docs from task text:**
- which pages were authored in earlier tasks
- where their source facts are expected to come from in the repo
- which verification artifacts or bug tasks already exist
</description>

<acceptance_criteria>
- [ ] Verification proceeds against the actual repository and runnable behavior
- [ ] Every discovered docs bug is turned into a bug task with the `add-bug` skill
- [ ] Any unavoidable tiny docs correction goes through `update-docs`
- [ ] Each run is capped at 3 docs pages and ends immediately after that capped work
- [ ] `<passes>true</passes>` is set only once the full verification task scope is complete
</acceptance_criteria>
