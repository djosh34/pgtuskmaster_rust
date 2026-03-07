## Task: Run Final Accuracy Verification And Create Bugs <status>not_started</status> <passes>false</passes> <priority>high</priority>

<description>
**Goal:** Perform the final accuracy-only verification pass after the authoring and navigation tasks are complete. This is the only task in the story that should introduce and use `docs/verifications/`. Its purpose is to check truth, not to do more drafting.

The higher-order goal is to separate authoring from factual validation. Earlier tasks draft, check/edit, revise, and publish the current best pages. This task verifies those pages against the actual repository and creates bug tasks for any factual inaccuracy or unsupported claim.

**Scope:**
- Work in:
  - `docs/src/`
  - `docs/verifications/`
  - `.ralph/tasks/bugs/`
  - `.ralph/tasks/story-build-docs-diataxis-from-zero/`
- Do not use this task to rewrite prose except where a tiny correction is unavoidable to complete the verification pass.
- Focus on truth only:
  - factual claims
  - command names and behavior
  - config names and defaults
  - architectural statements
  - step accuracy
  - cross-page contradictions

**Mandatory reread before this run:**
- `.agents/skills/create-docs/references/diataxis.fr/start-here/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/compass/index.md`
- `.agents/skills/create-docs/references/diataxis.fr/how-to-use-diataxis/index.md`
- `./.agents/skills/add-bug/SKILL.md`
- all story tasks in `./.ralph/tasks/story-build-docs-diataxis-from-zero/`

**Required execution loop:**
1. Create `docs/verifications/` if it does not already exist.
2. Review the authored pages in `docs/src/` form by form.
3. Check each factual claim against code, config, commands, tests, and runnable behavior.
4. Record the verification work under `docs/verifications/`.
5. For every factual inaccuracy, unsupported claim, or contradiction discovered, create a bug task using the `add-bug` skill rules in `.ralph/tasks/bugs/`.
6. Group related failures into one bug only when they are genuinely the same problem. Otherwise create separate bugs.
7. If a page is accurate, record that outcome in `docs/verifications/`.
8. If a tiny factual correction is safe and obvious, it may be fixed inline, but the main purpose is detection and bug creation, not another drafting pass.
9. Append progress and quit.

**Expected outcome:**
- `docs/verifications/` contains the accuracy-verification artifacts for the docs set.
- Every factual inaccuracy found during the pass has a corresponding bug task under `.ralph/tasks/bugs/`.
- The story now cleanly separates authoring from final truth checking.
- Verification for this docs task must always run `make docs-build`, `make docs-lint`, `make check`, and `make lint`; the expected docs-creation case is zero changes under `src/` or `tests/`; use `git` plus common sense, and do not run `make test` or `make test-long` unless the work intentionally changed behavior under `src/` or `tests/`.

</description>

<acceptance_criteria>
- [ ] `docs/verifications/` exists and contains the final accuracy-verification artifacts
- [ ] The verification pass checks factual claims against the actual repository rather than against earlier drafts
- [ ] Every factual inaccuracy, unsupported claim, or contradiction found results in a bug task created under `.ralph/tasks/bugs/` following the `add-bug` skill format
- [ ] Related failures are grouped only when they are genuinely the same underlying problem
- [ ] This task focuses on truth rather than re-running the earlier drafting loop
- [ ] `make docs-build` — passes cleanly
- [ ] `make docs-lint` — passes cleanly
- [ ] `make check` — passes cleanly
- [ ] Expected docs-creation case: `git` shows no intentional changes under `src/` or `tests/`, so `make test` and `make test-long` are not run
- [ ] Only if `git` shows intentional changes under `src/` or `tests/`, and common sense says behavior may have changed: `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] Only if `git` shows intentional changes under `src/` or `tests/`, and those changes impact ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
