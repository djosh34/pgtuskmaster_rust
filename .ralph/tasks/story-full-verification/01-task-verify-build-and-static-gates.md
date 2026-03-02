---
## Task: Verify build and static quality gates <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Validate the codebase can build and pass core static gates before deeper test execution.

**Scope:**
- Run initial repo health checks and build validation.
- Capture and classify any failures with concrete reproduction commands.

**Context from research:**
- This is the first scoped verification task in the full-verification plan.
- Keep execution deterministic and log exact command outputs for follow-up tasks.

**Expected outcome:**
- Build/static status is explicitly known.
- Any failures are converted into actionable bug tasks.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Run `make check` and record pass/fail with relevant error excerpts.
- [ ] Run `make lint` and record pass/fail with relevant error excerpts.
- [ ] If either command fails, use `$add-bug` skill to create bug task(s) in `.ralph/tasks/bugs/` with repro steps and affected files.
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test-bdd` — all BDD features pass
</acceptance_criteria>
