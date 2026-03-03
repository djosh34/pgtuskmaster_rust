---
## Task: Ultra-high-priority migrate repo gates to `make test` + `make test-long` only <status>not_started</status> <passes>false</passes> <priority>ultra-high</priority>

<description>
**Goal:** Complete and verify the global migration from legacy test targets to only two test groups: `make test` (regular) and `make test-long` (ultra-long only).

**Scope:**
- Enforce Makefile target surface to only `test` and `test-long` (remove all legacy extra test targets).
- Keep `make test` as the default frequently-run suite and ensure it excludes only the identified ultra-long tests.
- Keep `make test-long` scoped strictly to ultra-long tests and print a clear warning that long tests must be moved back to `make test` when they become short.
- Replace legacy references in:
- all `.ralph/tasks/**`
- all `.agents/skills/**`
- the rest of the repository (excluding `.ralph/progress/**` and `.ralph/archive/**`)
- Normalize wording after replacement so no stale/duplicated gate text remains.

**Context from research:**
- Current developer workflow is blocked by very long test execution windows.
- Prior multi-target naming caused gate drift and duplicated wording across tasks/skills.
- A strict two-group model is required to keep fast loops fast while still preserving long-run coverage.

**Expected outcome:**
- Repository text and Makefile consistently reference only `make test` and `make test-long`.
- No legacy gate names remain in active tasks/skills/repo documentation, except historical log artifacts if unavoidable.
- Team guidance clearly distinguishes default vs ultra-long execution flow.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Makefile exposes only `test` and `test-long` as test targets (no legacy extra test target definitions).
- [ ] `make test` excludes only the agreed ultra-long tests and remains the default frequent-run gate.
- [ ] `make test-long` runs only the ultra-long tests and prints a warning to move tests back to regular flow when shortened.
- [ ] Legacy target mentions are migrated across tasks, skills, and repo content (excluding `.ralph/progress/**` and `.ralph/archive/**`).
- [ ] No duplicate/contradictory gate wording remains after migration.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly
- [ ] `make lint` — passes cleanly
</acceptance_criteria>
