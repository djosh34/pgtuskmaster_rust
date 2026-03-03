---
## Task: Do meta-task 18 deep skeptical review pass 1 <status>done</status> <passes>true</passes> <passing>true</passing>

<blocked_by>18-task-recurring-meta-deep-skeptical-codebase-review</blocked_by>

<description>
**Goal:** Do meta-task `18-task-recurring-meta-deep-skeptical-codebase-review` as the first scheduled run.

**Scope:**
- Execute the meta-task exactly as written.
- Keep this task intentionally short and handoff-focused.

**Expected outcome:**
- One completed fresh run of meta-task 18 with findings/task fanout logged there.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] Meta-task 18 is executed as a fresh run.
- [x] Findings and follow-up tasks are recorded under meta-task 18.
</acceptance_criteria>

<execution_plan>
## Detailed Implementation Plan (Draft 1)

### Research snapshot (parallel exploration complete)
- Reviewed `19-task-do-meta-deep-skeptical-review-pass-1.md`, `18-task-recurring-meta-deep-skeptical-codebase-review.md`, and adjacent pass files (`20`, `21`) to confirm sequencing and expected deliverables.
- Verified current `.ralph/model.txt` is `normal_high`, so first execution run of meta-task 18 must enforce the preflight model gate before any audit work.
- Verified recurring-task rule in task 18: it must remain `<passes>meta-task</passes>` and must never be ticked to done.
- Verified required gate set for task 18 closeout from task 18 text: `make check`, `make test`, `make lint`, and `make test-bdd`; for this pass completion, enforce user-required order `make check`, `make test`, `make test-bdd`, `make lint`.
- Verified this pass-1 task is currently not started and has no lifecycle marker yet, so next handoff marker should be `TO BE VERIFIED`.

### Planned execution phases (to run after promotion to `NOW EXECUTE`)
1. Preflight model gate and handoff control
- Confirm `.ralph/model.txt` exact content before any review.
- If content is not exactly `deep_review`, set it to exactly `deep_review`, log that in both task 18 exploration notes and progress log, then stop that run immediately.
- Resume only when preflight is satisfied as required by meta-task 18.

2. Fresh-run setup and evidence directory bootstrap
- Create a run-specific evidence folder, for example `.ralph/evidence/meta-18-pass1-<timestamp>/`.
- Capture baseline state (`git status --short`, current task pointer, model file content).
- Add a new dated entry under the `## Exploration` section in task 18 to declare this as a fresh pass.

3. Deep skeptical code audit (broad and parallelized)
- Run a broad module sweep across runtime and tests (`src/dcs`, `src/ha`, `src/process`, `src/pginfo`, `src/config`, `src/api`, `src/test_harness`, `tests/`).
- Treat all assumptions as untrusted; verify behavior using code-path evidence, not prior task status.
- Validate anti-silent-pass safeguards, including real-binary enforcement and failure-path assertions.
- Record concrete findings (file/function + risk + expected behavior) in task 18 exploration notes.

4. Fanout discipline for findings
- Convert each small bounded issue into a bug file using the `add-bug` skill workflow.
- Convert each larger refactor or cross-module issue into a task file using `add-task-as-agent`.
- Keep one issue per task where possible; avoid bundling unrelated remediations.
- Reference each created bug/task from the active exploration entry in task 18.

5. Final mandatory gates for this fresh run
- Execute sequentially with log capture and `set -o pipefail`:
- `make check`
- `make test`
- `make test-bdd`
- `make lint`
- Persist outputs to evidence logs and create grep artifacts for `make test`/`make lint` marker checks when requested by task conventions.
- For any gate failure, immediately create follow-up bug task(s) before closeout.
- Re-run `git status --short` after gates and before closeout to verify every generated evidence artifact is tracked intentionally.

6. Meta-task closeout and model reset
- Update task 18 exploration entry with audited modules, findings summary, fanout links, and gate outcomes.
- Set `.ralph/model.txt` back to exactly `normal_high` only after all audit/fanout/gate obligations are complete.
- Keep task 18 tags compliant (`<passes>meta-task</passes>` unchanged).

7. Pass-1 task completion protocol
- Update this pass-1 task file checklist and status tags only after task 18 run evidence is complete.
- Run `/bin/bash .ralph/task_switch.sh` as required when handing off.
- Commit all changes (including `.ralph/*` evidence/bookkeeping) with required message format.
- Append durable learning to `AGENTS.md` if any new stable insight is discovered.

### Parallel work allocation plan (for execution run)
- Track A (preflight + bookkeeping): model gating, task pointers, progress logs.
- Track B (runtime audit): `src/*` skeptical inspection and defect candidate list.
- Track C (test reality audit): real binary usage, integration/e2e integrity, anti-silent-pass checks.
- Track D (fanout): create bug/task files with explicit acceptance criteria.
- Track E (verification): run and archive all mandatory make gates sequentially.

### Risks and controls
- Risk: accidental reuse of old assumptions from previous reviews.
- Control: explicit fresh-run log section in task 18 with dated evidence and independent findings.
- Risk: missing mandatory fanout for discovered issues.
- Control: no closeout until each finding has either a fix or a created bug/task file.
- Risk: false green from piped gate commands.
- Control: enforce `set -o pipefail` for all logged gate runs.
- Risk: forgetting to restore model profile.
- Control: closeout checklist requires final `.ralph/model.txt == normal_high` verification.
</execution_plan>

NOW EXECUTE

## Execution Notes (2026-03-03)
- Fresh run entry recorded under task 18 `## Exploration`.
- Evidence/logs: `.ralph/evidence/meta-18-pass1-20260303T020551Z` (includes `make-check.log`, `make-test.log`, `make-test-bdd.log`, `make-lint.log`, `make-test-real.log`).
- Model profile reset: `.ralph/model.txt` restored to `normal_high` after gates.
