---
## Task: Recurring meta-task for deep skeptical codebase quality verification <status>not_started</status> <passes>meta-task</passes>
NEVER TICK OFF THIS TASK. ALWAYS KEEP <passes>meta-task</passes>. This is a recurring deep verification task.

<description>
This is a **RECURRING META-TASK**.

Every time this task is picked up, the engineer must run a **FRESH verification** from scratch:
- Before starting: ensure `.ralph/model.txt` is exactly `deep_review`; if not, set it and quit immediately to switch model.
- Perform deep skeptical review across the full codebase quality surface: trust nothing, assume nothing.
- Validate test reality and anti-silent-pass guarantees, including real pg16 and real `etcd` binary usage.
- Validate e2e/integration behavior comes from real implementation, not accidental effects.
- Audit all code smells and broader quality concerns with nothing out of scope.
- Create `$add-bug` tasks for small findings and `$add-task-as-agent` tasks for larger findings.
- Final phase after findings/code-smell audit: run `make check`, `make test`, `make lint`, and `make test-bdd`.
- If any final-phase test gate fails, create bug task(s) with `$add-bug` for each failing area before closeout.
- Only after the full review/fanout is complete, set `.ralph/model.txt` back to exactly `normal_high`.

**NEVER set this task's passes to anything other than meta-task.**

## Exploration
### 2026-03-03 (fresh run, pass-1 full review in progress)
- Reviewer: codex
- Evidence directory: `.ralph/evidence/meta-18-pass1-20260303T020551Z`
- Preflight model check result: `.ralph/model.txt` was already `deep_review` (precondition satisfied).
- Files/modules audited:
  - Strict lint policy (`src/lib.rs`, `Makefile`)
  - Config schema/parser validation (`src/config/schema.rs`, `src/config/parser.rs`)
  - Real-binary enforcement policy (`src/test_harness/binaries.rs`)
  - Real etcd store adapter/watch semantics (`src/dcs/etcd_store.rs`)
  - HA e2e multi-node test fixture + port reservation discipline (`src/ha/e2e_multi_node.rs`, `src/test_harness/ports.rs`)
  - BDD tests (HTTP + state channel) (`tests/bdd_api_http.rs`, `tests/bdd_state_watch.rs`)
- Findings summary:
  - No `unwrap()`/`expect()`/`panic!()`/`todo!()`/`unimplemented!()` occurrences found in `src/` or `tests/`.
  - Real-binary tests are intentionally optional by default (skip when binaries missing), with deterministic enforcement available via `PGTUSKMASTER_REQUIRE_REAL_BINARIES=1` and `make test-real` (verified by running it below).
  - etcd watch bootstrap uses `get(prefix)` snapshot then `watch(prefix)` from `snapshot_revision + 1`, and reconnects/resnapshots on canceled/compacted watch responses.
- Small issues -> bug tasks: none for this pass (no issues found that required remediation tasks).
- Large issues -> agent tasks: none for this pass.
- Gate outcomes:
  - `make check`: pass (`make-check.log`)
  - `make test`: pass (`make-test.log`)
  - `make test-bdd`: pass (`make-test-bdd.log`)
  - `make lint`: pass (`make-lint.log`)
  - Real-binary enforcement gate: `make test-real`: pass (`make-test-real.log`)
- Closeout model reset to `normal_high`: done (verified `.ralph/model.txt` is `normal_high`).

### 2026-03-03 (fresh run, pass-1 preflight only)
- Reviewer: codex
- Preflight model check result: `.ralph/model.txt` was `normal_high` (mismatch), updated to `deep_review` to satisfy run precondition.
- Files/modules audited: none yet (execution halted at preflight by task contract).
- Findings summary: no code findings yet; this entry only documents mandatory model gate enforcement.
- Small issues -> bug tasks: none (not started due preflight halt).
- Large issues -> agent tasks: none (not started due preflight halt).
- Closeout model reset to `normal_high`: not applicable yet; full review not started.

### YYYY-MM-DD (fresh run)
- Reviewer:
- Preflight model check result:
- Files/modules audited:
- Findings summary:
- Small issues -> bug tasks:
- Large issues -> agent tasks:
- Closeout model reset to `normal_high`:

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] This run is treated as a fresh skeptical verification pass (no assumptions from previous runs).
- [ ] Preflight model gate is enforced (`deep_review`, or set+quit immediately when mismatched).
- [ ] Test reality and silent-pass resistance are verified, including real pg16/etcd binary usage and real implementation behavior in integration/e2e tests.
- [ ] Code smells and broader quality issues are audited across the full codebase.
- [ ] Every small issue is turned into a bug task via `$add-bug`.
- [ ] Every larger issue is turned into a task via `$add-task-as-agent`.
- [ ] Final verification runs all gates: `make check`, `make test`, `make lint`, `make test-bdd`.
- [ ] Every failing final-phase gate results in bug task(s) via `$add-bug` with actionable failure details.
- [ ] Final closeout step sets `.ralph/model.txt` to exactly `normal_high`.
- THIS TASK STAYS AS meta-task FOREVER
</acceptance_criteria>
