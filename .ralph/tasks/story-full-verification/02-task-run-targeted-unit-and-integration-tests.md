---
## Task: Run targeted unit and integration test suites <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Execute and validate non-e2e automated tests after static/build gates to identify functional regressions early.

**Scope:**
- Run project test commands for unit/integration coverage.
- Isolate failures to test, code, or environment causes.

**Context from research:**
- This task depends on basic build/static signal from task 01.
- Keep failures grouped by module/domain to enable parallel bug resolution.

**Expected outcome:**
- Unit/integration regression status is clear.
- Failing areas are captured as bug tasks with evidence.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Run `make test` and capture whether output indicates `congratulations` or `evaluation failed`.
- [ ] Run `make test-bdd` and capture passing/failing feature files.
- [ ] For each distinct failing behavior, use `$add-bug` skill to create bug task(s) in `.ralph/tasks/bugs/` with exact repro command and expected vs actual behavior.
- [ ] Re-run impacted test command(s) after fixes to confirm outcome.
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test-bdd` — all BDD features pass
</acceptance_criteria>
