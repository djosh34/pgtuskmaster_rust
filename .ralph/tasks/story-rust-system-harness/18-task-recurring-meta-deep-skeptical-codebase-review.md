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
- Final phase after findings/code-smell audit: run `make check`, `make test`, `make lint`, and `make test-long`.
- If any final-phase test gate fails, create bug task(s) with `$add-bug` for each failing area before closeout.
- If `make test-long` fails, also create a follow-up task to add a shorter real-binary e2e regression that reproduces the same failure.
- Only after the full review/fanout is complete, set `.ralph/model.txt` back to exactly `normal_high`.

**NEVER set this task's passes to anything other than meta-task.**

## Exploration
### 2026-03-03 (fresh run, pass-3 full review complete)
- Reviewer: codex
- Evidence directory: `.ralph/evidence/meta-18-pass3-20260303T022727Z`
- Preflight model check result: `.ralph/model.txt` was `deep_review` at substantive run start (precondition satisfied).
- Files/modules audited:
  - No-panic/no-unwrap/no-expect scan across runtime and test surfaces (`src/config`, `src/dcs`, `src/ha`, `src/process`, `src/pginfo`, `src/api`, `src/test_harness`, `tests/`) using 16 parallel skeptical tracks.
  - Strict schema guards via `#[serde(deny_unknown_fields)]` coverage (`src/config/schema.rs`, `src/api/mod.rs`, `src/api/fallback.rs`).
  - Real-binary enforcement path (`Makefile:test-long`, `src/test_harness/binaries.rs`) with explicit environment-gated enforcement.
  - etcd bootstrap/fault handling contracts (`src/dcs/etcd_store.rs`, `src/dcs/worker.rs`, `src/dcs/store.rs`) confirming snapshot+watch and unhealthy-path handling.
  - HA and BDD/integration realism surfaces (`src/ha/worker.rs`, `src/ha/e2e_multi_node.rs`, `tests/bdd_api_http.rs`, `tests/bdd_state_watch.rs`).
- Findings summary:
  - No `unwrap()`/`expect()`/`panic!()`/`todo!()`/`unimplemented!()` occurrences found in audited `src/` and `tests/` paths.
  - No new actionable code-smell or behavior regressions were identified in pass-3; no fanout tasks required.
  - Audit artifacts saved under the pass-3 evidence directory (`audit-track*.txt`, `audit-summary.txt`, `gate-log-presence.txt`).
- Small issues -> bug tasks: none for this pass.
- Large issues -> agent tasks: none for this pass.
- Gate outcomes:
  - `make check`: pass (`make-check.log`)
  - `make test`: pass (`make-test.log`)
  - Real-binary enforcement gate: `make test-long`: pass (`make-test-long.log`)
  - `make test`: pass (`make-test.log`)
  - `make lint`: pass (`make-lint.log`)
- Closeout model reset to `normal_high`: done (after full review + gates).

### 2026-03-03 (fresh run, pass-3 preflight only)
- Reviewer: codex
- Preflight model check result: `.ralph/model.txt` was `normal_high` (mismatch), updated to `deep_review` to satisfy run precondition.
- Files/modules audited: none yet (execution paused at preflight gate before substantive review).
- Findings summary: no code findings in this preflight-only step.
- Small issues -> bug tasks: none (not started due preflight gate).
- Large issues -> agent tasks: none (not started due preflight gate).
- Closeout model reset to `normal_high`: not applicable yet; full review still pending.

### 2026-03-03 (fresh run, pass-2 full review complete)
- Reviewer: codex
- Evidence directory: `.ralph/evidence/meta-18-pass2-20260303T021732Z`
- Preflight model check result: `.ralph/model.txt` was `deep_review` (precondition satisfied).
- Files/modules audited:
  - Gate definitions + strict lint posture (`Makefile`, `src/lib.rs`)
  - Config schema/parser strictness (`src/config/schema.rs`, `src/config/parser.rs`)
  - Real-binary enforcement policy (`src/test_harness/binaries.rs`, `Makefile:test-long`)
  - DCS adapter/watch semantics + error handling (`src/dcs/etcd_store.rs`, `src/dcs/store.rs`, `src/dcs/worker.rs`)
  - HA decision + ordered multi-worker step discipline (`src/ha/decide.rs`, `src/ha/worker.rs`)
  - Process worker job lifecycle + timeouts (`src/process/worker.rs`)
  - PG info polling + readiness mapping (`src/pginfo/worker.rs`, `src/pginfo/conninfo.rs`)
  - API worker routing/auth/TLS behaviors + fallback endpoints (`src/api/worker.rs`, `src/api/controller.rs`, `src/api/fallback.rs`)
  - Integration/e2e realism checks and port reservation discipline (`src/ha/e2e_multi_node.rs`, `src/test_harness/ports.rs`, `src/test_harness/namespace.rs`)
  - BDD/API/state contract tests (`tests/bdd_api_http.rs`, `tests/bdd_state_watch.rs`)
- Findings summary:
  - No `unwrap()`/`expect()`/`panic!()`/`todo!()`/`unimplemented!()` occurrences found in `src/` or `tests/`.
  - Config structs keep strict `#[serde(deny_unknown_fields)]` coverage in schema/API surfaces.
  - etcd watch bootstrap uses `get(prefix)` snapshot then `watch(prefix)` from `snapshot_revision + 1`, and treats canceled/compacted watch responses as unhealthy (forcing reconnect+resnapshot).
  - Real-binary enforcement gate `make test-long` is available and passes with `PGTUSKMASTER_REQUIRE_REAL_BINARIES=1`.
- Small issues -> bug tasks: none for this pass.
- Large issues -> agent tasks: none for this pass.
- Gate outcomes:
  - `make check`: pass (`make-check.log`)
  - `make test`: pass (`make-test.log`)
  - `make test-long`: pass (`make-test-long.log`)
  - `make lint`: pass (`make-lint.log`)
  - Real-binary enforcement gate: `make test-long`: pass (`make-test-long.log`)
- Closeout model reset to `normal_high`: done (after gates; see `.ralph/model.txt`)

### 2026-03-03 (fresh run, pass-2 preflight only)
- Reviewer: codex
- Preflight model check result: `.ralph/model.txt` was `normal_high` (mismatch), updated to `deep_review` to satisfy run precondition.
- Files/modules audited: none yet (execution halted at preflight by task contract).
- Findings summary: no code findings yet; this entry only documents mandatory model gate enforcement for pass-2.
- Small issues -> bug tasks: none (not started due preflight halt).
- Large issues -> agent tasks: none (not started due preflight halt).
- Closeout model reset to `normal_high`: not applicable yet; full review not started.

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
  - Real-binary tests are intentionally optional by default (skip when binaries missing), with deterministic enforcement available via `PGTUSKMASTER_REQUIRE_REAL_BINARIES=1` and `make test-long` (verified by running it below).
  - etcd watch bootstrap uses `get(prefix)` snapshot then `watch(prefix)` from `snapshot_revision + 1`, and reconnects/resnapshots on canceled/compacted watch responses.
- Small issues -> bug tasks: none for this pass (no issues found that required remediation tasks).
- Large issues -> agent tasks: none for this pass.
- Gate outcomes:
  - `make check`: pass (`make-check.log`)
  - `make test`: pass (`make-test.log`)
  - `make test-long`: pass (`make-test-long.log`)
  - `make lint`: pass (`make-lint.log`)
  - Real-binary enforcement gate: `make test-long`: pass (`make-test-long.log`)
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
- [ ] Final verification runs all gates: `make check`, `make test`, `make lint`, `make test-long`.
- [ ] Every failing final-phase gate results in bug task(s) via `$add-bug` with actionable failure details.
- [ ] Final closeout step sets `.ralph/model.txt` to exactly `normal_high`.
- THIS TASK STAYS AS meta-task FOREVER
</acceptance_criteria>
