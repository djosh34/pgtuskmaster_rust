## Task: Final double-check gate for real testing completeness <status>done</status> <passes>true</passes> <priority>ultra_high</priority>

<blocked_by>13-task-e2e-multi-node-real-ha-loops-scenario-matrix,14-task-security-auth-tls-real-cluster-tests</blocked_by>

<description>
**Goal:** Perform final independent verification that all components are truly tested, all required features exist and work, and all suites pass with no exceptions before final completion tasks.

**Scope:**
- Audit test quality across unit, integration, and e2e layers.
- Confirm tests are real behavior tests (not trivial assertions, not test-driven HA action execution).
- Confirm all planned features/types/functions are implemented and exercised.
- Execute full suite one final time and verify no failures.

**Context from research:**
- User requires explicit STOP gating and prohibition on fake/shortcut tests.

**Expected outcome:**
- Final verification report is complete and ready for the last task to conclude the loop.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] Test-quality audit confirms no mock-only fake coverage for critical HA paths and no tautological tests (`assert(true)` style) in meaningful suites. Evidence: `.ralph/evidence/15-final-gate/test-quality-audit.md`.
- [x] Test-quality audit confirms e2e/integration tests do not perform HA transitions directly; HA loops must produce behavior autonomously. Evidence: `.ralph/evidence/15-final-gate/ha-autonomy-audit.md`.
- [x] Feature audit confirms all plan features are present, working, and backed by tests. Evidence: `.ralph/evidence/15-final-gate/feature-trace-matrix.md`.
- [x] Run full suite with no exceptions: `make check`, `make test`, `make test-long`, `make lint`. Evidence: `.ralph/evidence/15-final-gate/make-check.log`, `.ralph/evidence/15-final-gate/make-test.log`, `.ralph/evidence/15-final-gate/make-test-long.log`, `.ralph/evidence/15-final-gate/make-lint.log`.
- [x] If any audit or suite step fails, use `$add-bug` skill to create bug task(s) for each issue. Result: no failed audit/gate step observed, so no bug task required.
- [x] Do not write `.ralph/STOP` in this task; STOP is handled only in the final story task.
</acceptance_criteria>

<execution_plan>
## Detailed Implementation Plan (planning run on 2026-03-03)

Research snapshot from parallel exploration sweep (10 probes):
- Blocker tasks `13` and `14` are currently marked as done with `<passes>` tags.
- Mandatory gates exist in `Makefile`: `check`, `test`, `test-long`, `lint`.
- Static grep found no `assert!(true)` tautology assertions, but this must still be validated by manual test intent review.
- Current workspace is not clean (`.ralph/current_tasks.md` modified; `.ralph/current_task.txt` and `.ralph/progress/25.jsonl` untracked), so this task must avoid clobbering unrelated state.

1. Preflight and baseline capture
- [x] Verify blocker tasks (`13`, `14`) are still done/passes true at execution start.
- [x] Capture baseline workspace status (`git status --short`) into `.ralph/evidence/15-final-gate/`.
- [x] Create deterministic evidence directory for this task: `.ralph/evidence/15-final-gate/`.

2. Test-quality audit: tautology and fake-coverage checks
- [x] Run static scan for low-value assertions and fake placeholders (`assert!(true)`, hardcoded pass markers, dead helper tests).
- [x] Manually review high-risk test modules (`src/ha/e2e_multi_node.rs`, `src/ha/worker.rs`, `src/dcs/worker.rs`, `src/process/worker.rs`, `tests/bdd_api_http.rs`) to confirm assertions validate behavior rather than implementation internals.
- [x] Record findings (or explicit no-findings evidence) in `.ralph/evidence/15-final-gate/test-quality-audit.md`.

3. HA autonomy audit (no direct transition driving inside e2e/integration)
- [x] Inspect e2e/integration flows for direct calls that would enact HA transitions (for example direct decide/action mutation paths used as scenario drivers).
- [x] Confirm fault injection is external-input only (API switchover request, process stop, DCS key stimuli), with HA loops performing actual transitions.
- [x] Explicitly flag any scenario driver that writes leader-transition ownership keys directly (for example `/{scope}/leader`) as a potential autonomy violation requiring either a bug task or a documented exception rationale.
- [x] Record proof snippets and file/line evidence in `.ralph/evidence/15-final-gate/ha-autonomy-audit.md`.

4. Feature-completeness audit (plan-to-code-to-tests traceability)
- [x] Build a trace matrix from `RUST_SYSTEM_HARNESS_PLAN.md` + completed story tasks to current code modules and test coverage.
- [x] Confirm required feature areas are implemented and exercised: config, state/watch channels, pginfo, DCS, process worker, HA decide/worker, API/debug workers, real PG/etcd harness, multi-node e2e, TLS/auth.
- [x] Record any gap as either immediate fix candidate or explicit bug task requirement.

5. Real-binary test reality checks
- [x] Verify tests that claim real PG16/etcd3 behavior actually resolve binaries via harness helpers and fail clearly when binaries are unavailable.
- [x] Confirm no newly introduced optional skip behavior conflicts with repository rule "No test must be optional".
- [x] Document real-binary evidence in `.ralph/evidence/15-final-gate/real-binary-audit.md`.

6. Full mandatory gate execution (strict sequential order)
- [x] Run `bash -lc 'set -o pipefail; CARGO_BUILD_JOBS=1 make check 2>&1 | tee .ralph/evidence/15-final-gate/make-check.log'`.
- [x] Run `bash -lc 'set -o pipefail; CARGO_BUILD_JOBS=1 make test 2>&1 | tee .ralph/evidence/15-final-gate/make-test.log'`.
- [x] Run `bash -lc 'set -o pipefail; CARGO_BUILD_JOBS=1 make test-long 2>&1 | tee .ralph/evidence/15-final-gate/make-test-long.log'`.
- [x] Run `bash -lc 'set -o pipefail; CARGO_BUILD_JOBS=1 make lint 2>&1 | tee .ralph/evidence/15-final-gate/make-lint.log'`.
- [x] Summarize gate outcomes with command timestamps in `.ralph/evidence/15-final-gate/gate-summary.md`.

7. Failure protocol (mandatory)
- [x] For every failed audit assertion or gate command, invoke the `$add-bug` skill and create one bug task per distinct issue (or one per tightly-coupled cluster). Result: no failed audit assertion or gate command in this run.
- [x] Each bug must include exact repro command, failing assertion/output excerpt, and evidence file paths under `.ralph/evidence/15-final-gate/`. Result: not applicable (no failures).
- [x] Do not mark this task done if any unresolved bug remains. Result: no unresolved bug created in this run.

8. Task completion updates (only after all checks are green)
- [x] Tick all acceptance criteria checkboxes with direct evidence references.
- [x] Update task header tags to `<status>done</status>` and `<passes>true</passes>`.
- [x] Add `<passes>true</passes>` in this task file only after all required commands pass.
- [x] Run `/bin/bash .ralph/task_switch.sh`.
- [x] Commit all changes (including `.ralph` artifacts) with:
- [x] `task finished 15-task-final-double-check-and-stop-gate: <summary, gate evidence, and implementation challenges>`.
- [x] Append durable learnings/surprises from this task to `AGENTS.md`.
</execution_plan>

NOW EXECUTE

<evidence>
- Preflight/blockers: `.ralph/evidence/15-final-gate/preflight-blockers.txt`, `.ralph/evidence/15-final-gate/git-status-before.txt`
- Test quality: `.ralph/evidence/15-final-gate/test-quality-audit.md`
- HA autonomy: `.ralph/evidence/15-final-gate/ha-autonomy-audit.md`
- Feature trace matrix: `.ralph/evidence/15-final-gate/feature-trace-matrix.md`
- Real binary audit: `.ralph/evidence/15-final-gate/real-binary-audit.md`
- Gate summary + logs: `.ralph/evidence/15-final-gate/gate-summary.md`, `.ralph/evidence/15-final-gate/make-check.log`, `.ralph/evidence/15-final-gate/make-test.log`, `.ralph/evidence/15-final-gate/make-test.log`, `.ralph/evidence/15-final-gate/make-lint.log`
</evidence>
