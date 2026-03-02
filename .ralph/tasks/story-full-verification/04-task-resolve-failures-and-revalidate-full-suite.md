---
## Task: Resolve discovered failures and revalidate full suite <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Drive failure resolution from created bug tasks and confirm full-suite green status after fixes.

**Scope:**
- Execute bug tasks generated from prior verification tasks.
- Re-run full validation after each meaningful fix batch.

**Context from research:**
- Upstream verification tasks explicitly require creating bug tasks via `$add-bug`.
- This task closes the loop by fixing tracked failures and proving stability.

**Expected outcome:**
- All discovered failures are resolved or explicitly documented as blocked.
- Final full-suite verification is complete with clear status.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Pull all relevant bug tasks from `.ralph/tasks/bugs/` created by `$add-bug` in prior tasks.
- [ ] Implement fixes per bug task with targeted verification for each.
- [ ] Run full suite after fix batches: `make check`, `make test`, `make lint`, `make test-bdd`.
- [ ] If new failures appear, create follow-up bug task(s) using `$add-bug` and continue triage.
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test-bdd` — all BDD features pass
</acceptance_criteria>
