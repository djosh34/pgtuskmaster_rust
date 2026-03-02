---
## Task: Final double-check gate for real testing completeness <status>not_started</status> <passes>false</passes> <priority>ultra_high</priority>

<blocked_by>13-task-e2e-multi-node-real-ha-loops-scenario-matrix,14-task-security-auth-tls-real-cluster-tests</blocked_by>

<description>
**Goal:** Perform final independent verification that all components are truly tested, all required features exist and work, and all suites pass with no exceptions before final completion tasks.

**Scope:**
- Audit test quality across unit, integration, and e2e layers.
- Confirm tests are real behavior tests (not trivial assertions, not test-driven HA action execution).
- Confirm all planned features/types/functions are implemented and exercised.
- Execute full suite one final time and verify no failures.

**Context from research:**
- User requires explicit STOP gating and prohibition on fake/shortcut tests.

**Expected outcome:**
- Final verification report is complete and ready for the last task to conclude the loop.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Test-quality audit confirms no mock-only fake coverage for critical HA paths and no tautological tests (`assert(true)` style) in meaningful suites.
- [ ] Test-quality audit confirms e2e/integration tests do not perform HA transitions directly; HA loops must produce behavior autonomously.
- [ ] Feature audit confirms all plan features are present, working, and backed by tests.
- [ ] Run full suite with no exceptions: `make check`, `make test`, `make lint`, `make test-bdd`.
- [ ] If any audit or suite step fails, use `$add-bug` skill to create bug task(s) for each issue.
- [ ] Do not write `.ralph/STOP` in this task; STOP is handled only in the final story task.
</acceptance_criteria>
