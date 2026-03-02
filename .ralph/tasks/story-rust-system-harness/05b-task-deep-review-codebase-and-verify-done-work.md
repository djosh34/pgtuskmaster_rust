---
## Task: Deep review codebase quality and verify done tasks are truly complete <status>not_started</status> <passes>false</passes> <priority>ultra_high</priority>

<description>
**Goal:** Perform a deep end-to-end review of current repository quality, test reality, and completion truthfulness of all tasks already marked done.

**Scope:**
- Enforce preflight model-profile gate through `.ralph/model.txt` before any review work.
- Deeply review runtime and test code for quality issues, untested behavior, and code smells chosen by reviewer judgment (no fixed smell checklist).
- Verify every task currently marked done is actually complete in code and tests.
- Treat test sufficiency as primary: verify and expand unit, integration (real binaries), and BDD/e2e scenario coverage.
- Verify test coverage includes real-binary usage for `etcd3` and PostgreSQL 16 paths.
- Verify linting truly forbids `panic`, `unwrap`, and `expect` in runtime code and catches violations.
- Create follow-up tasks for every material finding and place them in sensible task order.

**Context from research:**
- Current active story has multiple done tasks plus in-progress/not-started tasks in `.ralph/tasks/story-rust-system-harness/`.
- Existing lint policy task (`05a`) is marked done and must be independently verified as effective, not assumed.
- User requires reviewer-selected deep inspection rather than a predefined smell rubric.

**Expected outcome:**
- Review either confirms correctness with evidence or produces concrete follow-up tasks for every gap.
- If tests are missing and not already represented in planned tasks, new test-creation tasks are added immediately.
- Any missing real-binary tests for `etcd3` and PostgreSQL 16 are treated as major findings and turned into remediation tasks.
- `.ralph/model.txt` is restored to `normal_high` only after all review obligations, task creation, and commits are fully complete.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Preflight model-file gate is enforced exactly:
- [ ] Check whether `.ralph/model.txt` exists.
- [ ] If missing, create it with only `deep_review` as content.
- [ ] If present but content is not exactly `deep_review`, overwrite it so only `deep_review` remains.
- [ ] If file was created or changed to `deep_review` in this run, stop immediately after writing and quit this task run.
- [ ] If file already contained exactly `deep_review`, continue with the deep review in the same run.
- [ ] Deep review is performed across the current codebase (code smells, untested code, general quality) using reviewer judgment rather than a fixed smell checklist.
- [ ] Every task marked done is validated against actual code/tests and any false-complete state is reported as finding(s).
- [ ] If tests are missing and not already planned in existing tasks, create new tasks specifically to add those tests.
- [ ] Test creation expectations cover unit tests, integration tests against real binaries, and expanded BDD/e2e scenario matrices.
- [ ] Verification explicitly confirms whether tests against real `etcd3` and real PostgreSQL 16 binaries exist and pass; if missing/inadequate, create remediation tasks and mark as major finding(s).
- [ ] Verification explicitly confirms linting catches and forbids `panic`, `unwrap`, and `expect` usage in runtime code.
- [ ] For each small bug finding, create a task via `$add-bug`.
- [ ] For each large change/refactor finding, create a task via `$add-task-as-agent`; set `<priority>high</priority>` only when immediate action is required, otherwise leave priority unset.
- [ ] Newly created follow-up tasks are placed at sensible positions in story ordering (not dumped at end by default).
- [ ] All created task files and related bookkeeping changes are committed.
- [ ] Final step only: set `.ralph/model.txt` content to exactly `normal_high`.
- [ ] `make check` — passes cleanly, or failure is captured with follow-up task(s).
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail); failures become follow-up task(s).
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail); failures become follow-up task(s).
- [ ] `make test-bdd` — all BDD features pass, or failures become follow-up task(s).
</acceptance_criteria>

<execution_plan>
## Detailed Implementation Plan

1. Preflight model gate (mandatory first action)
- [ ] Inspect `.ralph/model.txt` existence and exact content.
- [ ] If absent or not exactly `deep_review`, write file to contain only `deep_review` and then quit immediately (do not continue review in that run).
- [ ] If already exactly `deep_review`, continue.

2. Establish review baseline
- [ ] Collect current done-task set from `.ralph/tasks/story-rust-system-harness/` and any other active story directories with done tasks.
- [ ] Run baseline quality/test gates: `make check`, `make test`, `make lint`, `make test-bdd`.
- [ ] Capture evidence for pass/fail and map failures to actionable findings.

3. Deep codebase review (reviewer-driven)
- [ ] Perform deep inspection of current codebase quality, including code smells and untested behavior selected by reviewer judgment.
- [ ] Avoid prescriptive smell checklists; choose the most meaningful inspections based on repository reality.

4. Done-task truth audit
- [ ] For each task marked done, verify:
- [ ] Claimed scope exists in code.
- [ ] Acceptance expectations are truly satisfied.
- [ ] Validation evidence is still valid (not regressed).
- [ ] For any mismatch, record concrete finding and create remediation task(s).

5. Real-binary test verification (hard requirement)
- [ ] Confirm there are tests using real `etcd3` binaries.
- [ ] Confirm there are tests using real PostgreSQL 16 binaries.
- [ ] Confirm those tests execute and pass in expected environments.
- [ ] If either coverage is missing/inadequate, create major remediation tasks immediately.

6. Test sufficiency and planning-gap audit (hard requirement)
- [ ] Determine whether current tests are sufficient across:
- [ ] unit-level behavior,
- [ ] integration behavior with real binaries,
- [ ] broad BDD/e2e scenario variety.
- [ ] For any missing test coverage not already represented by existing planned tasks, create explicit new test-creation tasks immediately.
- [ ] Ensure those new tasks are concrete about target modules/scenarios and expected pass criteria.

7. Lint enforcement verification (hard requirement)
- [ ] Confirm lint configuration and commands forbid `panic`, `unwrap`, and `expect` in runtime code.
- [ ] Validate with targeted canary checks or equivalent proof that forbidden usage is caught.
- [ ] If enforcement is weak or bypassable, create remediation task(s).

8. Follow-up task creation and placement
- [ ] Route small bugs to `$add-bug`.
- [ ] Route larger changes/refactors to `$add-task-as-agent`.
- [ ] For urgent findings, set `<priority>high</priority>`; otherwise leave priority unset.
- [ ] Place each new task at a sensible sequence point in its story, based on dependencies and urgency.

9. Closeout and restoration
- [ ] Commit all created task files and bookkeeping updates.
- [ ] After everything is fully complete, set `.ralph/model.txt` to exactly `normal_high` as the final acceptance action.
</execution_plan>
