---
## Task: Do meta-task 18 deep skeptical review pass 2 <status>done</status> <passes>true</passes> <passing>true</passing>

<blocked_by>19-task-do-meta-deep-skeptical-review-pass-1</blocked_by>

<description>
**Goal:** Do meta-task `18-task-recurring-meta-deep-skeptical-codebase-review` again in the next ordered slot.

**Scope:**
- Execute the same meta-task as a new fresh run.
- Keep this task intentionally short and sequence-oriented.

**Expected outcome:**
- Another independent pass is completed and logged in meta-task 18 exploration notes.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] Meta-task 18 is executed again as a fresh run.
- [x] Findings and follow-up tasks are recorded under meta-task 18.
</acceptance_criteria>

<execution_plan>
## Detailed Implementation Plan (Draft 2, skeptical-verified)

### Research snapshot (parallel exploration complete)
- Completed a broad parallel context scan (12 tracks) across task docs and implementation surfaces:
  - Task flow/sequencing: `18-task-recurring-meta-deep-skeptical-codebase-review.md`, `19-task-do-meta-deep-skeptical-review-pass-1.md`, current pass-2 file, and neighboring pass file `21`.
  - Gate definitions and strict lint posture: `Makefile`, `src/lib.rs`.
  - Real-binary policy and skip/enforcement behavior: `src/test_harness/binaries.rs`.
  - etcd watch/session behavior: `src/dcs/etcd_store.rs`.
  - HA real integration surfaces and reservation usage: `src/ha/e2e_multi_node.rs`, `src/test_harness/ports.rs`.
  - BDD/API/state behavior checks: `tests/bdd_api_http.rs`, `tests/bdd_state_watch.rs`.
- Confirmed pass-1 already completed and task 20 remains `not_started` with no lifecycle marker.
- Confirmed required final gates for this pass closeout must be run and green: `make check`, `make test`, `make lint`.
- Confirmed meta-task 18 recurring contract: keep `<passes>meta-task</passes>` unchanged, enforce model preflight, and record fresh-run exploration evidence.

### Execution phases (to run only after promotion to `NOW EXECUTE`)
1. Preflight contract and model gate
- Read `.ralph/model.txt`.
- If value is not exactly `deep_review`, set it to exactly `deep_review`, log preflight in meta-task 18 exploration + progress log, then stop immediately (no audit yet).
- Resume execution only when preflight is already satisfied.

2. Fresh run declaration and evidence bootstrap
- Create a run-specific evidence folder, for example:
  - `.ralph/evidence/meta-18-pass2-<timestamp>/`
- Capture baseline artifacts:
  - `git status --short`
  - `cat .ralph/current_task.txt`
  - `cat .ralph/model.txt`
- Add a dated fresh-run section under `## Exploration` in meta-task 18 before audit work starts.

3. Deep skeptical audit (parallel execution tracks)
- Run audit with at least 15 parallel tracks covering runtime + tests + harness:
  - Config parse/validate path strictness.
  - DCS watch refresh/error/fault publication behavior.
  - HA decision/action dispatch idempotency and ordering assumptions.
  - Process worker real-job transitions and cleanup guarantees.
  - PG info state mapping and readiness transitions.
  - API auth/tls behaviors and fallback endpoints.
  - Test harness real-binary policy and failure semantics.
  - Integration/e2e realism checks (no accidental mocks masking behavior).
- Treat prior pass conclusions as untrusted; re-derive each claim from code and/or runnable evidence.

4. Findings fanout discipline
- For each small bounded issue: create bug via `add-bug`.
- For each larger cross-module issue: create follow-up task via `add-task-as-agent`.
- Reference every created task/bug inside the active meta-task 18 exploration entry.
- Do not close this pass until each finding is either fixed now or represented by a created task.

5. Mandatory gates with auditable logs
- Run sequentially with `set -o pipefail` and `tee` into evidence logs.
- Pin Cargo parallelism for each gate to reduce known archive/object race noise:
  - `CARGO_BUILD_JOBS=1 make check`
  - `CARGO_BUILD_JOBS=1 make test`
  - `CARGO_BUILD_JOBS=1 make test`
  - `CARGO_BUILD_JOBS=1 make lint`
- Enforce real-binary coverage as a required gate for this pass:
  - `CARGO_BUILD_JOBS=1 make test`
- If real-binary prerequisites are missing, install/fix prerequisites and rerun; if still failing, create bug task(s) before closeout.
- Preserve auditable command logs:
  - `make check`
  - `make test`
  - `make test-long`
  - `make lint`
- `make test`
- If any gate fails, create bug task(s) immediately before closeout.

6. Meta-task 18 closeout
- Update meta-task 18 exploration section with:
  - audited files/modules,
  - concrete findings summary,
  - links to created follow-up tasks/bugs,
  - gate result log paths.
- Restore `.ralph/model.txt` to exactly `normal_high` only after all above obligations are complete.

7. Pass-2 task closeout
- Mark pass-2 acceptance criteria as done only after meta-task 18 entry and gate evidence are complete.
- Set this pass-2 task tags to done/passing.
- Run `/bin/bash .ralph/task_switch.sh`.
- Commit all changes (including `.ralph/*`) with required message format:
  - `task finished [task name]: [summary with evidence/challenges]`
- Append any stable new learning to `AGENTS.md`.

### Planned skeptical verification step (required before execution)
- When this file is in `TO BE VERIFIED`, perform a deep skeptical plan review with 16+ parallel verification tracks.
- During that review, alter at least one concrete plan item (scope/order/control), and record what changed and why.
- Replace marker with `NOW EXECUTE` only after that alteration is complete and justified.

### Skeptical verification delta (2026-03-03)
- Verified with 18 parallel tracks across meta-task docs, gate definitions, runtime workers, DCS adapter, harness binary policy, and BDD/e2e surfaces.
- Alteration 1: changed `make test` from optional to mandatory to satisfy no-optional-real-test policy.
- Alteration 2: required `CARGO_BUILD_JOBS=1` for all gate runs to control known Cargo archive race noise.
- Rationale: both changes reduce false-green risk and tighten pass evidence fidelity.

### Risks and controls
- Risk: carrying assumptions from pass-1 into pass-2.
- Control: explicit fresh-run declaration and independent evidence capture.
- Risk: false green from piped commands.
- Control: mandatory `set -o pipefail` for each logged gate.
- Risk: discovered issues not tracked.
- Control: strict fanout rule (`add-bug` / `add-task-as-agent`) before closeout.
- Risk: forgetting model reset.
- Control: final closeout checklist includes explicit `.ralph/model.txt == normal_high` validation.
</execution_plan>

NOW EXECUTE

## Execution Notes (2026-03-03)
- Fresh run entry recorded under meta-task 18 `## Exploration` (pass-2 section).
- Evidence/logs: `.ralph/evidence/meta-18-pass2-20260303T021732Z` (includes `make-check.log`, `make-test.log`, `make-lint.log`, `make-test-long.log`, plus audit grep/excerpts).
- Findings: no follow-up tasks/bugs created for this pass (no issues found that warranted fanout).
- Model profile reset: `.ralph/model.txt` restored to `normal_high` after gates.
