---
## Task: Run full verification for secure explicit config refactor <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Execute full validation gates after the config refactor and convert any failures into actionable bug tasks.

**Scope:**
- Run full required project gates after merging upstream tasks.
- Record evidence logs and failure signatures.
- Create bug tasks for residual regressions with exact repro details.

**Context from research:**
- This refactor touches shared config, runtime wiring, auth, and tests; broad verification is mandatory.
- Real-binary and BDD coverage are required by project policy.

**Expected outcome:**
- End-state confidence that explicit secure startup config works across compile, lint, unit/integration, and BDD flows.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Full exhaustive checklist of all files/modules to modify with specific requirements for each
- [ ] Evidence logs are captured for each gate and linked from the task update
- [ ] Any failing gate has corresponding bug task(s) with repro and impacted modules
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test` — all BDD features pass
</acceptance_criteria>
