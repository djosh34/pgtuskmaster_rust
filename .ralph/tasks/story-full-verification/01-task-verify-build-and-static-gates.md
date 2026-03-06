---
## Task: Verify build and static quality gates <status>done</status> <passes>true</passes>

<description>
**Goal:** Validate the codebase can build and pass core static gates before deeper test execution.

**Scope:**
- Run initial repo health checks and build validation.
- Capture and classify any failures with concrete reproduction commands.

**Context from research:**
- This is the first scoped verification task in the full-verification plan.
- Keep execution deterministic and log exact command outputs for follow-up tasks.

**Expected outcome:**
- Build/static status is explicitly known.
- Any failures are converted into actionable bug tasks.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] Run `make check` and record pass/fail with relevant error excerpts.
- [x] Run `make lint` and record pass/fail with relevant error excerpts.
- [x] If either command fails, use `$add-bug` skill to create bug task(s) in `.ralph/tasks/bugs/` with repro steps and affected files. (No failures observed; no bug files required.)
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] BDD features pass (covered by `make test`).
</acceptance_criteria>

<implementation_plan>
## Execution Plan (Draft 2, Skeptically Verified)

### Skeptical verification completed (16 parallel tracks)
- Re-read `Makefile` and confirmed real command bodies for `check`, `test`, `test`, and `lint`.
- Re-read this task for acceptance criteria and marker semantics.
- Validated sibling story task names/paths (`02`, `03`, `04`) to avoid stale references.
- Re-read `.agents/skills/add-bug/SKILL.md` for required bug-file schema and placement.
- Re-read `AGENTS.md` constraints for no skipped tests and no parallel top-level cargo gates.
- Re-read `.ralph/task_switch.sh` side effects before completion actions.
- Searched repository for `TO BE VERIFIED`/`NOW EXECUTE` examples to mirror established lifecycle.
- Searched repository for status tags (`<status>`, `<passes>`) to keep finalization format consistent.
- Searched for acceptance-marker usage (`congratulations`, `evaluation failed`) and confirmed these markers are task-policy checks, not guaranteed tool output.
- Checked `.ralph/evidence` structure to use existing evidence conventions.
- Checked `.ralph/tasks/bugs` existing bug patterns for grouping and naming consistency.
- Reviewed recent commit message format to match required `task finished ...` style.
- Reviewed current working tree state so execution accounts for pre-existing modified files.
- Verified no plan step requires speculative code changes before gate evidence exists.
- Verified stale-artifact recovery rule (`cargo clean` on missing `*.rcgu.o`/archive failures) remains valid from AGENTS learnings.
- Verified gate sequencing should stay strictly serial for deterministic outcomes.

### Key amendment from skeptical review
- Changed plan to remove redundant duplicate gate runs. Prior draft scheduled `make check`/`make lint` twice; revised plan runs one canonical serial suite (`make check`, `make test`, `make lint`) and uses the same logs to satisfy both build/static and global acceptance requirements.

### Execution phases for `NOW EXECUTE`
1. Prepare deterministic evidence capture
- Create `.ralph/evidence/story-full-verification/01-task-verify-build-and-static-gates/`.
- Run each gate through `bash -lc 'set -o pipefail; ... | tee ...'` so exit codes are preserved.
- Write logs:
- `make-check.log`
- `make-test.log`
- `make-test-long.log`
- `make-lint.log`
- `grep-make-test-markers.log`
- `grep-make-lint-markers.log`

2. Run the canonical gate sequence exactly once (serial)
- Execute in order:
- `make check`
- `make test`
- `make test-long`
- `make lint`
- After `make test` and `make lint`, run marker greps for `congratulations|evaluation failed`; if absent, record explicit `not found` output to the grep logs.

3. Classify failures and create bug tasks when needed
- For each distinct failing behavior, create/extend bug task entries in `.ralph/tasks/bugs/` using `$add-bug` format with:
- exact repro command
- expected vs actual behavior
- impacted modules/files
- evidence log paths
- If failure matches stale artifact signature (`failed to build archive`/missing `*.rcgu.o`), perform one `cargo clean` and rerun the full canonical sequence, preserving both pre-clean and post-clean evidence.

4. Update task state with evidence-backed outcomes
- Fill acceptance checkboxes according to gate results and evidence.
- Keep task status non-done if any gate remains red after allowed recovery + bug creation.
- Only if all required gates pass:
- set header to `<status>done</status> <passes>true</passes>`
- append/set `<passes>true</passes>`
- run `/bin/bash .ralph/task_switch.sh`
- commit all changes (including `.ralph` artifacts and task file) with message:
- `task finished 01-task-verify-build-and-static-gates: <summary including gate evidence and challenges>`
- append new learning/surprise to `AGENTS.md`

### Risks to watch
- `make lint` executes multiple clippy passes; first failure should still preserve full error excerpt in log before any remediation.
- Marker grep checks are policy evidence, not primary gate truth; command exit codes remain source of pass/fail.
- Avoid scope creep into future verification tasks unless required to make current mandatory gates pass.
</implementation_plan>

NOW EXECUTE

<execution_report>
- Evidence directory: `.ralph/evidence/story-full-verification/01-task-verify-build-and-static-gates/`
- `make check`: pass (`make-check.log`)
- `make test`: pass (`make-test.log`)
- `make test`: pass (`make-test.log`)
- `make lint`: pass (`make-lint.log`)
- Marker grep (`make test`): `not found` (`grep-make-test-markers.log`)
- Marker grep (`make lint`): `not found` (`grep-make-lint-markers.log`)
</execution_report>
