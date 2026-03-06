---
## Task: Add real HA+DCS+Process integration tests <status>done</status> <passes>true</passes>

<description>
**Goal:** Build integration tests that wire real PG16 binaries, a real etcd-backed DCS store, the process worker, pginfo worker, and HA worker so failures cannot pass silently.

**Scope:**
- Use the existing test harness spawners in `src/test_harness/pg16.rs`, `src/test_harness/etcd3.rs`, `src/test_harness/namespace.rs`, and `src/test_harness/ports.rs`.
- Add integration tests under `tests/` or a dedicated `src/ha/worker` test module that:
  - Start etcd and postgres using real binaries.
  - Run the process worker + pginfo worker and feed their state into the HA worker.
  - Assert HA actions produce real effects: leader key written to etcd, postgres started, and state transitions observed.
- Ensure tests fail if binaries are missing or if actions fail to execute.

**Context from research:**
- Current real-binary tests exist for `pginfo` and `process` only; HA/DCS integration uses fake stores and queues.
- `worker_contract_tests.rs` only checks callability, not behavior.
- Existing harness helpers already enforce real binaries via `.tools/postgres16` and `.tools/etcd`.

**Expected outcome:**
- At least one deterministic integration test that proves the real worker pipeline works end-to-end.
- Tests should fail on real execution errors, not accept `JobOutcome::Failure` for success-path cases.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x]  Full exhaustive checklist of all files/modules to modify with specific requirements for each
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] BDD features pass (covered by `make test`).
</acceptance_criteria>

<execution_plan>
## Detailed Implementation Plan (Draft 2, Skeptical Verification Applied)

### Deep skeptical verification completed (18 tracks)
- [x] Re-validated harness spawners (`spawn_pg16`, `spawn_etcd3`) and their port-reservation contract.
- [x] Re-validated real-binary gating (`require_*_for_real_tests`) and enforcement env (`PGTUSKMASTER_REQUIRE_REAL_BINARIES`).
- [x] Re-validated `step_once` entry points across pginfo/dcs/ha/process workers.
- [x] Re-validated HA worker tests and identified existing integration tests are mostly stubbed store/runner based.
- [x] Re-validated existing real HA pipeline test in `src/ha/e2e_multi_node.rs` (real etcd + real process runner + real pginfo + real HA + etcd-backed DCS).
- [x] Re-validated DCS real-etcd step tests and fixture patterns in `src/dcs/etcd_store.rs`.
- [x] Re-validated `Makefile` mandatory gates and strict lint policy.
- [x] Re-validated `state::new_state_channel` semantics (`Version(0)` baseline, publish increments).
- [x] Re-validated current task marker is `TO BE VERIFIED` and updated per workflow.

### Mandatory plan changes from Draft 1 (at least one altered item)
- [x] **Changed primary implementation target** from `src/ha/worker.rs` to `src/ha/e2e_multi_node.rs` for real-binary integration work, because that module already owns real cluster fixtures and avoids duplicating heavyweight harness setup in worker unit/integration tests.
- [x] **Added explicit enforced-real-binary verification** (`PGTUSKMASTER_REQUIRE_REAL_BINARIES=1`) before final gates so “missing binaries must fail” is actually validated instead of silently skipped.
- [x] **Added stale-satisfaction branch**: if current code already satisfies task outcomes and enforced runs pass, complete via evidence + task bookkeeping rather than speculative rework.

### Full exhaustive file/module checklist to modify
1. `.ralph/tasks/story-rust-system-harness/task-real-ha-dcs-process-integration-tests.md`
- [x] Keep checklist/phase progress synchronized with actual execution.
- [x] Tick acceptance criteria only after logs/evidence are written.
- [x] Set `<status>done</status> <passes>true</passes>` only after all required gates pass.

2. `src/ha/e2e_multi_node.rs` (primary implementation file)
- [x] If gap exists, add/adjust one deterministic real integration scenario that explicitly validates: leader key write, postgres start effect, and HA phase transition observation. (No gap required code changes; enforced-real pipeline test already satisfied this.)
- [x] Keep error messages diagnostic with observable state snapshots and timeline artifact path.
- [x] Preserve non-panicking error flow (`Result`-based assertions/teardown patterns).

3. `src/ha/mod.rs` (conditional)
- [x] Only modify if a new HA real-test module is introduced (for example, `e2e_single_node.rs`). (Not needed; no new module introduced.)

4. `AGENTS.md`
- [x] Append one concrete learning from this task after final verification.

5. `.ralph/evidence/task-real-ha-dcs-process-integration-tests/` (new)
- [x] Save logs for mandatory gates: `make check`, `make test`, `make lint`.
- [x] Save grep artifacts for `make test` and `make lint` containing `congratulations` and `evaluation failed` checks.
- [x] Save targeted enforced-real-binary test logs used to validate non-skipping behavior.

### Execution phases (NOW EXECUTE will follow these exactly)
1. Baseline + enforced-real-binary verification (no speculative edits)
- [x] Run targeted HA real integration with enforcement:
- [x] `PGTUSKMASTER_REQUIRE_REAL_BINARIES=1 cargo test --all-targets ha::e2e_multi_node::e2e_multi_node_real_ha_scenario_matrix`
- [x] Assess whether this already satisfies task outcomes (real etcd DCS + process worker + pginfo + HA with real effects).

2. Implement only if a real functional gap is proven
- [x] If required assertions are missing, patch `src/ha/e2e_multi_node.rs` to assert all required real effects explicitly: (No assertion gaps found in enforced run.)
- [x] leader key written to etcd,
- [x] postgres-start behavior observed through process/pginfo state,
- [x] expected HA phase transitions observed.
- [x] Keep retries bounded and condition-based; do not add unconditional sleeps.

3. Stabilize and verify targeted tests
- [x] Run focused tests for changed modules until deterministic.
- [x] Ensure failure paths surface actionable diagnostics (state snapshots/timeline path).

4. Run mandatory repository gates sequentially
- [x] `make check`
- [x] `make test`
- [x] `make test`
- [x] `make lint`
- [x] Capture all outputs under `.ralph/evidence/task-real-ha-dcs-process-integration-tests/`.
- [x] Generate required grep marker artifacts for `make test` + `make lint`.

5. Task completion bookkeeping
- [x] Tick acceptance criteria checkboxes in task file.
- [x] Set `<status>done</status> <passes>true</passes>`.
- [x] Run `/bin/bash .ralph/task_switch.sh`.
- [x] Commit all changed files (including `.ralph/*`) with message:
- [x] `task finished task-real-ha-dcs-process-integration-tests: <summary, gate evidence, implementation notes>`
- [x] Append AGENTS.md learning and stop immediately.

### Risks and mitigations
- [x] Risk: real test silently skips when binaries are absent.
- [x] Mitigation: run targeted command with `PGTUSKMASTER_REQUIRE_REAL_BINARIES=1` and keep output evidence.
- [x] Risk: asynchronous readiness races produce false negatives.
- [x] Mitigation: retain bounded retry loops with explicit readiness/phase predicates.
- [x] Risk: teardown masks primary failure.
- [x] Mitigation: preserve “run_result + teardown_result + artifact_result” combined error reporting pattern.
- [x] Risk: cargo artifact races in full gates.
- [x] Mitigation: run gates serially; if archive/object corruption appears, `cargo clean` and rerun with logs.
</execution_plan>

NOW EXECUTE
