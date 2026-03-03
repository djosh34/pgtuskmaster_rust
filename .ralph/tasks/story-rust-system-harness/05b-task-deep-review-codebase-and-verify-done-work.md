---
## Task: Deep review codebase quality and verify done tasks are truly complete <status>done</status> <passes>true</passes> <passing>true</passing> <priority>ultra_high</priority>

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
- [x] Check whether `.ralph/model.txt` exists.
- [ ] If missing, create it with only `deep_review` as content.
- [ ] If present but content is not exactly `deep_review`, overwrite it so only `deep_review` remains.
- [ ] If file was created or changed to `deep_review` in this run, stop immediately after writing and quit this task run.
- [x] If file already contained exactly `deep_review`, continue with the deep review in the same run.
- [x] Deep review is performed across the current codebase (code smells, untested code, general quality) using reviewer judgment rather than a fixed smell checklist.
- [x] Every task marked done is validated against actual code/tests and any false-complete state is reported as finding(s).
- [x] If tests are missing and not already planned in existing tasks, create new tasks specifically to add those tests.
- [x] Test creation expectations cover unit tests, integration tests against real binaries, and expanded BDD/e2e scenario matrices.
- [x] Verification explicitly confirms whether tests against real `etcd3` and real PostgreSQL 16 binaries exist and pass; if missing/inadequate, create remediation tasks and mark as major finding(s).
- [x] Verification explicitly confirms linting catches and forbids `panic`, `unwrap`, and `expect` usage in runtime code.
- [x] For each small bug finding, either fix it immediately or create a task via `$add-bug`.
- [x] For each large change/refactor finding, create a task via `$add-task-as-agent`; set `<priority>high</priority>` only when immediate action is required, otherwise leave priority unset.
- [x] Newly created follow-up tasks are placed at sensible positions in story ordering (not dumped at end by default).
- [x] All created task files and related bookkeeping changes are committed.
- [x] Final step only: set `.ralph/model.txt` content to exactly `normal_high`.
- [x] `make check` — passes cleanly, or failure is captured with follow-up task(s).
- [x] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail); failures become follow-up task(s).
- [x] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail); failures become follow-up task(s).
- [x] `make test` — all BDD features pass, or failures become follow-up task(s).
</acceptance_criteria>

<execution_plan>
## Detailed Implementation Plan (planning run on 2026-03-02)

Research snapshot from deep skeptical verification (parallel 16-probe sweep):
- `.ralph/model.txt` currently exists and is `deep_review`.
- Story done-task set currently includes `01, 02, 03, 04, 05, 05a, 06, 07, 10`.
- Required make targets exist in root `Makefile`: `check`, `test`, `test`, `lint`.
- Clippy deny policy exists in `src/lib.rs`; scoped test harness allowance exists in `src/test_harness/mod.rs`.
- `panic!/expect/unwrap` matches are present broadly in `src/` and `tests/`; runtime-vs-test classification must be done explicitly before filing findings.
- Grep evidence for `congratulations` / `evaluation failed` is not yet persisted to deterministic paths; execution must tee outputs to fixed log files.
- Prior repo learning warns against parallel top-level Cargo/make runs due to intermittent lock/link artifacts; gate runs must be strictly sequential.

Execution notes (2026-03-02):
- Evidence and findings summary are recorded in `.ralph/evidence/05b/review_summary.md`.
- Fixed a port-reservation lifetime bug in real-binary tests so they can bind ports correctly when binaries are installed.
- Added follow-up tasks:
  - `10a` to enforce real-binary execution in CI/dev when desired.
  - `10b` to add a real etcd3-backed DCS store adapter + integration tests.

1. Phase A: preflight gate handoff run (must terminate early if changed)
- [x] Inspect `.ralph/model.txt` exact content as first operation.
- [ ] If file is missing or not exactly `deep_review`, write only `deep_review` and stop immediately for this run.
- [x] If already exactly `deep_review`, continue to full deep review run.

2. Phase B: baseline evidence capture
- [x] Capture repository baseline (`git status --short`) for auditability before running gates.
- [x] Create deterministic evidence directory (for example `.ralph/evidence/05b/`) and run every gate sequentially with `tee` to fixed log files.
- [x] Execute baseline gates in strict order (no parallel make/cargo):
- [x] `make check | tee .ralph/evidence/05b/make-check.log`
- [x] `make test | tee .ralph/evidence/05b/make-test.log`
- [x] `make test | tee .ralph/evidence/05b/make-test.log`
- [x] `make lint | tee .ralph/evidence/05b/make-lint.log`
- [x] For `make test` and `make lint`, grep saved logs for `congratulations` and `evaluation failed` exactly per task rule and store grep outputs under the same evidence directory.
- [x] Convert any gate failure into explicit follow-up tasks during this same run (no deferred TODOs).

3. Phase C: done-task truth audit (all done tasks, not sampled)
- [x] Enumerate all task files currently marked `<status>done</status>` across `.ralph/tasks/`.
- [x] For each done task, verify:
- [x] code claims exist in current tree,
- [x] acceptance criteria are still true,
- [x] linked tests still exist and pass or are covered by current gate runs.
- [x] Verify each done task header/tag consistency (`<status>done</status>`, `<passes>true</passes>`, and `<passing>true</passing>` where required by that task workflow) and record discrepancies.
- [x] Record mismatches as findings with concrete file/function evidence.
- [x] Create remediation tasks for each false-complete or regressed done task.

4. Phase D: deep quality review (reviewer-judgment driven)
- [x] Inspect high-risk modules first: `src/process/`, `src/dcs/`, `src/ha/`, `src/pginfo/`, `src/config/`, `src/test_harness/`.
- [x] Identify defect risks, brittle state transitions, weak error handling, and untested branches.
- [x] Prioritize findings by severity and blast radius.
- [x] Create `$add-bug` tasks for bounded fixes and `$add-task-as-agent` tasks for larger refactors.

5. Phase E: test sufficiency audit (primary focus)
- [x] Build a coverage map across:
- [x] unit tests,
- [x] integration tests with real binaries,
- [x] BDD/e2e scenarios and matrix breadth.
- [x] Verify whether missing test areas are already planned in not-done tasks.
- [x] If not already planned, create new explicit test-creation tasks immediately with target modules and pass criteria.

6. Phase F: hard verification of real-binary requirements
- [x] Verify real `etcd3` integration coverage (harness plus at least one behavioral consumer path).
- [x] Verify real PostgreSQL 16 integration coverage (harness plus worker/runtime consumer paths).
- [x] Confirm these tests run successfully in current environment or clearly document environment-gated skips and required prerequisites.
- [x] Treat missing or insufficient real-binary coverage as major findings with high-priority remediation tasks.

7. Phase G: lint enforcement proof
- [x] Verify current lint policy truly blocks runtime `panic`, `unwrap`, and `expect`.
- [x] Distinguish runtime code from test-only allowances and ensure policy boundary is intentional.
- [x] Perform canary-style proof (temporary forbidden use in runtime path, then revert) or equivalent deterministic validation.
- [x] After canary revert, confirm clean tree (`git diff --exit-code`) before proceeding so temporary probe edits cannot leak into follow-up commits.
- [x] If policy is bypassable, create remediation tasks with concrete enforcement changes.

8. Phase H: task creation and ordering discipline
- [x] Create one follow-up task per material finding (no bundled mega-task that hides independent work).
- [x] Apply `<priority>high</priority>` only when immediate unblock/safety is required; otherwise leave unset.
- [x] Insert new tasks in dependency-correct order within story files, not appended blindly.
- [x] Ensure all new task files and related index/bookkeeping edits are staged for commit.

9. Phase I: closeout protocol (only after all above complete)
- [x] Commit all created/updated task artifacts with evidence-oriented message.
- [x] Set `.ralph/model.txt` to exactly `normal_high` as final action of completed review cycle.
- [x] Re-run any required final checks if closeout edits touched executable code (expected: task-only changes, so likely unnecessary).

NOW EXECUTE
</execution_plan>
