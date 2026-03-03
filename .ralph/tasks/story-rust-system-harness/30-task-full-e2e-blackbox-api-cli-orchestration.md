---
## Task: Migrate full e2e suites to black-box API and CLI orchestration <status>not_started</status> <passes>false</passes>

<blocked_by>22-task-ha-admin-api-read-write-surface</blocked_by>
<blocked_by>23-task-ha-admin-cli-over-api</blocked_by>
<blocked_by>25-task-enforce-e2e-api-only-control-no-direct-dcs</blocked_by>

<description>
**Goal:** Convert full-system e2e tests into black-box tests that interact through public API/CLI surfaces rather than internal worker channels or binary-specific control paths.

**Scope:**
- Refactor full e2e suites to drive administrative actions through API endpoints or `pgtuskmasterctl` only.
- Replace internal-state peeking where possible with API-visible status checks and SQL-level behavior assertions.
- Define allowed test-only failure injection hooks clearly (for example process kill) while keeping control/verification surfaces public.
- Add documentation for e2e black-box policy and update test conventions.

**Context from research:**
- Existing e2e implementation is tightly coupled to internal fixtures/subscribers and direct store operations.
- Requested direction is external operator parity: control via admin API, observe via admin/read API plus SQL.
- This task aligns e2e signal with production usage and reduces hidden coupling.

**Expected outcome:**
- Full e2e flows are operator-realistic black-box tests that exercise system behavior through public control/read interfaces.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Full exhaustive checklist completed with concrete module requirements: full e2e scenario files migrated to API/CLI control paths, direct internal control helpers removed from full e2e codepaths, API status assertions replacing internal-only checks where feasible, e2e policy doc/conventions updated under repository docs/task notes
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test-bdd` — all BDD features pass
</acceptance_criteria>
