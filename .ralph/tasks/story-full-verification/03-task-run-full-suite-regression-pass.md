---
## Task: Run full regression suite end-to-end <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Execute the entire validation suite in one pass to confirm holistic repository health.

**Scope:**
- Run all required project-level verification commands sequentially.
- Produce a consolidated pass/fail report.

**Context from research:**
- This is the explicit full-suite step requested in the plan conversion.
- Full-suite failure discovery must feed directly into tracked bug tasks.

**Expected outcome:**
- A single full-suite status snapshot exists.
- All failures are tracked as actionable bug tasks.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Run full suite in order: `make check`, `make test`, `make lint`, `make test-bdd`.
- [ ] Capture command outputs and summarize exact failing stages.
- [ ] For each failure found, use `$add-bug` skill to create bug task(s) in `.ralph/tasks/bugs/` including repro command, logs, and scope.
- [ ] If all commands pass, record full-suite pass evidence in the task update.
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test-bdd` — all BDD features pass
</acceptance_criteria>
