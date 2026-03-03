---
## Task: Resolve discovered failures and revalidate full suite <status>done</status> <passes>true</passes> <passing>true</passing>

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
- [x] Pull all relevant bug tasks from `.ralph/tasks/bugs/` created by `$add-bug` in prior tasks.
- [x] Implement fixes per bug task with targeted verification for each. (No active bug tasks remained; closure verified via `bug-inventory.log` and `bug-pending-status.log`.)
- [x] Run full suite after fix batches: `make check`, `make test`, `make lint`.
- [x] If new failures appear, create follow-up bug task(s) using `$add-bug` and continue triage. (No new failures appeared.)
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] BDD features pass (covered by `make test`).
</acceptance_criteria>

<implementation_plan>
## Execution Plan (Draft 2, Skeptically Verified)

### Deep skeptical verification completed (16 parallel tracks)
- Track 1: Re-read task 04 acceptance criteria and required gate list.
- Track 2: Enumerated all bug task files under `.ralph/tasks/bugs/`.
- Track 3: Scanned bug tasks for `<status>`, `<passes>`, and `<passing>` completion tags.
- Track 4: Scanned bug tasks for residual lifecycle markers (`TO BE VERIFIED`, `NOW EXECUTE`) that might indicate incomplete closure.
- Track 5: Re-read story-full-verification task 01 for completion/reporting conventions.
- Track 6: Re-read story-full-verification task 02 for evidence naming and marker-grep conventions.
- Track 7: Re-read story-full-verification task 03 for finalization/tag expectations.
- Track 8: Revalidated top-level gate wiring from `Makefile`.
- Track 9: Revalidated marker-policy strings (`congratulations`, `evaluation failed`) across tasks/evidence.
- Track 10: Revalidated stale artifact recovery guidance (`cargo clean`) in AGENTS + task notes.
- Track 11: Revalidated that current evidence tree has no task-04 evidence directory yet.
- Track 12: Revalidated git working tree so in-flight `.ralph` state is preserved.
- Track 13: Checked bug inventory cardinality and naming stability.
- Track 14: Revalidated `<execution_report>` usage in sibling tasks for consistency.
- Track 15: Revalidated full-suite order and serial execution requirement.
- Track 16: Spot-checked codebase for disallowed `unwrap`/`expect`/`panic!` pattern regressions before execution.

### Key amendment from skeptical review
- Added an explicit pre-gate, auditable inventory artifact (`bug-inventory.log`) plus a pending-status check (`bug-pending-status.log`) before any full-suite command. This closes a traceability gap in Draft 1, where the "all bugs closed" claim was not preserved as evidence.

### Current snapshot from research
- All currently discovered bug tasks in `.ralph/tasks/bugs/` are tagged done with passing metadata.
- No bug file currently has `<status>not_started</status>` or `<status>in_progress</status>`.
- Task 04 remains open because it still needs its own explicit closure run, evidence, and bookkeeping.

### Execution phases for `NOW EXECUTE`
1. Build and archive bug-closure inventory
- Create evidence directory:
- `.ralph/evidence/story-full-verification/04-task-resolve-failures-and-revalidate-full-suite/`
- Generate inventory artifact:
- `bug-inventory.log` containing each bug filename with status tags.
- Generate pending-status artifact:
- `bug-pending-status.log` containing any `not_started`/`in_progress` bug statuses, or explicit `not found`.
- Mirror the same inventory summary in this task file for handoff readability.

2. Perform targeted revalidation for any active/suspicious bug area
- If `bug-pending-status.log` shows active bug tasks, run minimal targeted command(s) for each first and archive logs under:
- `.ralph/evidence/story-full-verification/04-task-resolve-failures-and-revalidate-full-suite/targeted/`
- If no active bug tasks are found, explicitly record that targeted reruns were not required in `<execution_report>`.
- If any targeted run fails, stop broad verification, create follow-up bug task(s) via `$add-bug`, fix, and rerun targeted checks.

3. Execute one canonical full-suite confirmation pass (serial, deterministic)
- Run gates with preserved exit codes and archived output:
- `bash -lc 'set -o pipefail; CARGO_BUILD_JOBS=1 make check | tee make-check.log'`
- `bash -lc 'set -o pipefail; CARGO_BUILD_JOBS=1 make test | tee make-test.log'`
- `bash -lc 'set -o pipefail; CARGO_BUILD_JOBS=1 make test | tee make-test.log'`
- `bash -lc 'set -o pipefail; CARGO_BUILD_JOBS=1 make lint | tee make-lint.log'`
- Run marker-grep artifacts for policy tracking:
- `grep-make-test-markers.log` from `make-test.log`
- `grep-make-lint-markers.log` from `make-lint.log`
- Record explicit `not found` lines when markers are absent.
- Produce `make-test-failures.log` by extracting `FAILED|failures|error` lines, or explicit `not found`.

4. Triage any newly surfaced failures
- For each distinct new failure behavior, create a new bug task using `$add-bug` with:
- exact repro command
- expected vs actual behavior
- impacted files/modules
- evidence log path(s)
- Implement fixes without introducing `unwrap`/`expect`/`panic`.
- Re-run targeted commands after each fix batch, then re-run the full suite.

5. Finalize task 04 when all required gates are green
- Tick all acceptance checkboxes in this task file.
- Set task header tags to:
- `<status>done</status> <passes>true</passes> <passing>true</passing>`
- Add an `<execution_report>` block with command outcomes and evidence filenames.
- Run `/bin/bash .ralph/task_switch.sh`.
- Commit all repo changes (including `.ralph` files/evidence) with:
- `task finished 04-task-resolve-failures-and-revalidate-full-suite: <summary with gate evidence and remediation notes>`
- Append one concise learning/surprise to `AGENTS.md`.

### Risks and controls
- Marker grep strings are policy metadata only; command exit status remains the source of truth.
- Keep top-level Cargo gates strictly serial; do not run concurrent full-suite commands.
- If stale artifact signatures appear (`failed to build archive`, missing `*.rcgu.o`), run `cargo clean` once and rerun affected gates with pre/post evidence preserved.
</implementation_plan>

NOW EXECUTE

### Bug inventory summary
- `.ralph/tasks/bugs/bug-remove-unwrap-panic-allow.md` -> done/true/true
- `.ralph/tasks/bugs/dcs-watch-refresh-errors-ignored.md` -> done/true/true
- `.ralph/tasks/bugs/pginfo-standby-polling-test-configure-primary-db-error.md` -> done/true/true
- `.ralph/tasks/bugs/process-worker-real-job-tests-accept-failure-outcomes.md` -> done/true/true
- `.ralph/tasks/bugs/process-worker-real-job-tests-state-channel-closed.md` -> done/true/true
- `.ralph/tasks/bugs/real-binary-tests-fail-when-port-allocation-is-blocked.md` -> done/true/true
- `.ralph/tasks/bugs/remove-panics-expects-unwraps.md` -> done/true/true
- `.ralph/tasks/bugs/test-harness-binary-check-panics.md` -> done/true/true
- `.ralph/tasks/bugs/worker-contract-tests-assert-only-callability.md` -> done/true/true

<execution_report>
- Evidence directory: `.ralph/evidence/story-full-verification/04-task-resolve-failures-and-revalidate-full-suite/`
- Bug inventory artifacts:
- `bug-inventory.log`
- `bug-pending-status.log` (`not found`)
- `bug-count.log` (`9`)
- No active bug tasks were found, so targeted reruns under `targeted/` were not required.
- `make check`: pass (`make-check.log`)
- `make test`: pass (`make-test.log`)
- `make test`: pass (`make-test.log`)
- `make lint`: pass (`make-lint.log`)
- Marker grep (`make test`): `not found: congratulations`, `not found: evaluation failed` (`grep-make-test-markers.log`)
- Marker grep (`make lint`): `not found: congratulations`, `not found: evaluation failed` (`grep-make-lint-markers.log`)
- BDD failure extract: `not found` (`make-test-failures.log`)
- New bug tasks created in this run: none (no new failures surfaced).
</execution_report>
