---
## Task: Enforce post-startup hands-off test policy (no direct coordination) <status>not_started</status> <passes>false</passes> <priority>high</priority>

<blocked_by>35-task-migrate-all-node-startup-tests-to-unified-entrypoint-config-only</blocked_by>

<description>
**Goal:** After cluster/node startup, tests must not perform direct internal coordination or DCS steering; they may only observe/listen plus allowed external actions.

**Scope:**
- Define and enforce a strict post-start policy for tests:
- allowed: observation/listening assertions, SQL writes/reads when test intent requires data mutation, and allowed admin API requests (for example switchover flows),
- forbidden: direct coordination mutations, direct DCS key steering, or internal state forcing once startup completes.
- Refactor any tests that currently violate this policy to use external behavior-driven stimuli instead of internals.
- Add/strengthen policy guard tests/scripts to block regressions.
- Ensure exception rules are explicit, minimal, and documented in test policy comments/docs.

**Context from research:**
- Request requires production-parity behavior once the cluster is running: tests should not "drive" internals that production cannot rely on.
- Existing test suites historically had some internal coordination shortcuts; those must be removed or gated.
- This builds on unified startup so all scenarios share the same initial runtime semantics.

**Expected outcome:**
- Test behavior from startup onward mirrors production constraints: observe externally, stimulate through allowed interfaces, and never directly coordinate internals.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Full exhaustive checklist completed with concrete module requirements: e2e/integration test files and harness helpers audited, violations removed, and policy guard coverage added/updated
- [ ] Policy guard fails if tests reintroduce forbidden post-start direct coordination patterns (direct DCS writes/deletes, internal coordination forcing)
- [ ] Allowed exception paths are explicitly codified: SQL data writes where needed, and approved API actions (including switchover requests)
- [ ] Existing HA scenario tests continue to validate role/fencing/safety behavior using external-observable flows
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test-bdd` — all BDD features pass
</acceptance_criteria>
