---
## Task: Migrate all node-starting tests to unified entrypoint (config-only) <status>not_started</status> <passes>false</passes> <priority>high</priority>

<blocked_by>34-task-add-non-test-unified-node-entrypoint-autobootstrap-and-ha-loop</blocked_by>

<description>
**Goal:** Ensure every test that starts a `pgtuskmaster` node uses the same new production entrypoint and only provides configuration.

**Scope:**
- Inventory all tests/harnesses that currently spin nodes directly or through ad-hoc startup wiring.
- Replace those startup paths to call the new non-test unified entry API only.
- Remove or stop using alternative startup wiring in tests that bypasses production entry semantics.
- If required config is missing for test scenarios, extend shared config structs and fixtures so tests remain config-driven.
- Ensure startup semantics do not diverge by test type: all node-starting tests must invoke the same entry path.

**Context from research:**
- Request explicitly calls out inconsistent test startup behavior for "main/entry stuff".
- Desired behavior is production parity: tests and runtime should enter through one canonical startup flow.
- Any missing config needed by this flow is itself a required follow-up change, not a reason to bypass entrypoint.

**Expected outcome:**
- Node startup behavior is unified across test suites and production, reducing scenario skew and startup drift.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Full exhaustive checklist completed with concrete module requirements: all test harness modules and test files that start nodes are enumerated and migrated to the unified entrypoint
- [ ] No test that starts a node uses bespoke direct startup orchestration outside the unified entry API
- [ ] Shared config structs/fixtures are updated where required so startup remains config-only
- [ ] Add/update regression guard(s) that fail if new node-starting tests bypass the unified entrypoint
- [ ] `make check --all-targets` (or stricter equivalent) passes after config surface changes
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test-bdd` — all BDD features pass
</acceptance_criteria>
