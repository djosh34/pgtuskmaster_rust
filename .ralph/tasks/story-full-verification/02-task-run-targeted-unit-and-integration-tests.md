---
## Task: Run targeted unit and integration test suites <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
**Goal:** Execute and validate non-e2e automated tests after static/build gates to identify functional regressions early.

**Scope:**
- Run project test commands for unit/integration coverage.
- Isolate failures to test, code, or environment causes.

**Context from research:**
- This task depends on basic build/static signal from task 01.
- Keep failures grouped by module/domain to enable parallel bug resolution.

**Expected outcome:**
- Unit/integration regression status is clear.
- Failing areas are captured as bug tasks with evidence.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] Run `make test` and capture whether output indicates `congratulations` or `evaluation failed`. (Gate passed; marker grep logged `not found`.)
- [x] Run `make test-bdd` and capture passing/failing feature files. (All BDD tests passed; failure extract logged `not found`.)
- [x] For each distinct failing behavior, use `$add-bug` skill to create bug task(s) in `.ralph/tasks/bugs/` with exact repro command and expected vs actual behavior. (No failing behavior observed; no bug files required.)
- [x] Re-run impacted test command(s) after fixes to confirm outcome. (No fixes were needed; required commands executed and passed in this run.)
- [x] `make check` — passes cleanly
- [x] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [x] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [x] `make test-bdd` — all BDD features pass
</acceptance_criteria>

<implementation_plan>
## Execution Plan (Draft 1)

### Research tracks completed in parallel
- Track 1: Read current task requirements and acceptance criteria.
- Track 2: Read `Makefile` targets for `check`, `test`, `test-bdd`, and `lint`.
- Track 3: Read `$add-bug` skill format and placement requirements.
- Track 4: Read completed sibling task 01 for lifecycle and evidence formatting conventions.
- Track 5: Search repo for marker policy strings (`congratulations`, `evaluation failed`).
- Track 6: Validate task/story file paths and naming consistency.
- Track 7: Confirm `.ralph/evidence` and `.ralph/tasks/bugs` locations for outputs.
- Track 8: Reconfirm AGENTS constraints that tests are never optional and cargo gates should be run serially for deterministic outcomes.

### Planned execution phases once promoted to `NOW EXECUTE`
1. Create deterministic evidence workspace
- Create `.ralph/evidence/story-full-verification/02-task-run-targeted-unit-and-integration-tests/`.
- Run commands via `bash -lc 'set -o pipefail; <cmd> | tee <log>'` so each exit status is preserved.
- Planned log files:
- `make-test.log`
- `make-test-bdd.log`
- `make-test-bdd-failures.log`
- `make-check.log`
- `make-lint.log`
- `grep-make-test-markers.log`
- `grep-make-lint-markers.log`

2. Execute the canonical gate sequence serially
- Run, in order:
- `make test`
- `make test-bdd`
- `make check`
- `make lint`
- Perform marker greps after `make test` and `make lint` for `congratulations|evaluation failed`.
- Extract failing BDD feature/scenario lines into `make-test-bdd-failures.log` using `rg -n "FAILED|failures|error" make-test-bdd.log`; if no lines match, record explicit `not found`.
- If marker strings are absent, save explicit `not found` evidence in grep logs (still relying on command exit codes as truth).

3. Failure triage and bug task creation
- Group failures by distinct behavior/domain (for example: bdd-only failure vs clippy failure vs unit runtime failure).
- For each distinct failure cluster, create bug files in `.ralph/tasks/bugs/` using `$add-bug` template with:
- exact repro command
- expected behavior vs actual behavior
- affected modules/files
- evidence log path(s)
- BDD-only failures must include exact failing feature/test names copied from `make-test-bdd-failures.log`.
- If failures match stale target-artifact signatures (`failed to build archive` or missing `*.rcgu.o`), run one `cargo clean`, rerun impacted command(s), and keep pre/post-clean logs.

4. Fix and revalidation loop
- Address each bug cluster in small commits or grouped edits without using unwrap/expect/panic.
- Re-run only impacted commands after each fix to verify local regression closure.
- After all targeted failures are resolved, rerun full required gate suite:
- `make check` (final)
- `make test` (final)
- `make test-bdd` (final)
- `make lint` (final)

5. Task finalization criteria
- Tick acceptance checkboxes with evidence-backed outcomes.
- Only when all required commands pass:
- update header to done/passing values
- set `<passing>true</passing>`
- run `/bin/bash .ralph/task_switch.sh`
- commit all changed files (including `.ralph` evidence/task updates and any code fixes) with message:
- `task finished 02-task-run-targeted-unit-and-integration-tests: <summary with evidence + challenges>`
- append relevant learning/surprise to `AGENTS.md`

### Risks and controls
- `make lint` includes multiple clippy passes with strict restriction lints; preserve earliest failing excerpt in logs before any remediation.
- BDD failures may reflect environment timing; preserve exact feature/test names to avoid over-broad bug grouping.
- Keep gate runs serial to avoid cargo cache contention and misleading object/archive errors.
</implementation_plan>

NOW EXECUTE

<execution_report>
- Evidence directory: `.ralph/evidence/story-full-verification/02-task-run-targeted-unit-and-integration-tests/`
- `make test`: pass (`make-test.log`)
- `make test-bdd`: pass (`make-test-bdd.log`)
- `make check`: pass (`make-check.log`)
- `make lint`: pass (`make-lint.log`)
- Marker grep (`make test`): `not found` (`grep-make-test-markers.log`)
- Marker grep (`make lint`): `not found` (`grep-make-lint-markers.log`)
- BDD failure extract: `not found` (`make-test-bdd-failures.log`)
</execution_report>
