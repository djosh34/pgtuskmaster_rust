---
## Task: Run full regression suite end-to-end <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
**Goal:** Execute the entire validation suite in one pass to confirm holistic repository health.

**Scope:**
- Run all required project-level verification commands sequentially.
- Produce a consolidated pass/fail report.

**Context from research:**
- This is the explicit full-suite step requested in the plan conversion.
- Full-suite failure discovery must feed directly into tracked bug tasks.

**Expected outcome:**
- A single full-suite status snapshot exists.
- All failures are tracked as actionable bug tasks.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] Run full suite in order: `make check`, `make test`, `make lint`.
- [x] Capture command outputs and summarize exact failing stages. (No failing stages observed.)
- [x] For each failure found, use `$add-bug` skill to create bug task(s) in `.ralph/tasks/bugs/` including repro command, logs, and scope. (No failures found; no new bug tasks required.)
- [x] If all commands pass, record full-suite pass evidence in the task update.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] BDD features pass (covered by `make test`).
</acceptance_criteria>

<implementation_plan>
## Execution Plan (Draft 2, Skeptically Verified)

### Skeptical verification completed (16 parallel tracks)
- Track 1: Re-read task 03 acceptance criteria for exact gate order and required outputs.
- Track 2: Re-read `Makefile` to validate that all targets map to Cargo commands and may be sensitive to artifact races.
- Track 3: Re-read task 01 full execution record for status/tag conventions.
- Track 4: Re-read task 02 execution record for evidence log naming and marker-grep conventions.
- Track 5: Re-read task 04 to ensure task 03 failure outputs are consumable by downstream remediation.
- Track 6: Re-read `$add-bug` skill schema/rules for bug file requirements.
- Track 7: Re-read `.ralph/task_switch.sh` behavior to confirm it removes `.ralph/current_task.txt`.
- Track 8: Scanned AGENTS learnings for stale artifact and serial-run guidance.
- Track 9: Scanned `.ralph/evidence/` examples for accepted marker-grep artifact formats.
- Track 10: Scanned story-full-verification files for status tag consistency (`<status>`, `<passes>`, `<passing>`).
- Track 11: Scanned for lifecycle marker patterns (`TO BE VERIFIED`, `NOW EXECUTE`) to mirror established flow.
- Track 12: Rechecked current git working tree shape so pre-existing `.ralph` changes are not lost.
- Track 13: Revalidated that command exit status is the source of truth and grep markers are secondary policy evidence.
- Track 14: Revalidated serial execution requirement to avoid misleading Cargo cache contention.
- Track 15: Revalidated bug-folder inventory to avoid duplicate bug-file naming if failures appear.
- Track 16: Revalidated accepted gate summary style used in completed sibling tasks.

### Key amendment from skeptical review
- Updated execution to run all suite commands with `CARGO_BUILD_JOBS=1` to reduce intermittent object/archive races during full-suite validation while keeping commands serial.

### Execution phases for `NOW EXECUTE`
1. Prepare deterministic evidence capture
- Create `.ralph/evidence/story-full-verification/03-task-run-full-suite-regression-pass/`.
- Run each command via `bash -lc 'set -o pipefail; CARGO_BUILD_JOBS=1 <cmd> | tee <log>'` so outputs are archived and exit codes are preserved.
- Planned logs:
- `make-check.log`
- `make-test.log`
- `make-lint.log`
- `make-test.log`
- `grep-make-test-markers.log`
- `grep-make-lint-markers.log`
- `make-test-failures.log`

2. Execute full suite in required order (single serial pass)
- Execute:
- `make check`
- `make test`
- `make lint`
- `make test`
- After `make test` and `make lint`, grep logs for `congratulations|evaluation failed`; if absent, write explicit `not found`.
- Extract BDD failure lines from `make-test.log` with `rg -n "FAILED|failures|error"`; if none, write explicit `not found`.

3. Triage failures and create bug task(s) when needed
- Summarize exact failing stage(s) with direct evidence log paths.
- For each distinct failure behavior, create bug file(s) in `.ralph/tasks/bugs/` per `$add-bug` rules including repro, expected vs actual, scope, and logs.
- If stale artifact signatures appear (`failed to build archive` or missing `*.rcgu.o`), run one `cargo clean`, rerun the full serial suite once, and retain pre/post-clean logs.

4. Finalize task state and bookkeeping
- If all required commands pass, tick acceptance checkboxes and set header to:
- `<status>done</status> <passes>true</passes> <passing>true</passing>`
- Append `<execution_report>` with evidence file names and pass/fail summary.
- Run `/bin/bash .ralph/task_switch.sh`.
- Commit all modified files (including `.ralph` files) with message:
- `task finished 03-task-run-full-suite-regression-pass: <summary with evidence and implementation notes>`
- Append one new learning/surprise to `AGENTS.md`.

### Risks and controls
- `make lint` runs multiple clippy invocations; preserve first failing excerpt if red.
- Grep markers are acceptance evidence only; gate pass/fail is based on command exit status.
- Keep full-suite execution strictly serial and bounded to one canonical pass unless stale-artifact recovery is triggered.
</implementation_plan>

NOW EXECUTE

<execution_report>
- Evidence directory: `.ralph/evidence/story-full-verification/03-task-run-full-suite-regression-pass/`
- `make check`: pass (`make-check.log`)
- `make test`: pass (`make-test.log`)
- `make lint`: pass (`make-lint.log`)
- `make test`: pass (`make-test.log`)
- Marker grep (`make test`): `not found: congratulations`, `not found: evaluation failed` (`grep-make-test-markers.log`)
- Marker grep (`make lint`): `not found: congratulations`, `not found: evaluation failed` (`grep-make-lint-markers.log`)
- BDD failure extract: `not found` (`make-test-failures.log`)
</execution_report>
