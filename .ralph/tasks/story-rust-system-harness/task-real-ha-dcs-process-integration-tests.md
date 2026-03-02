---
## Task: Add real HA+DCS+Process integration tests <status>not_started</status> <passes>false</passes>

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
- [ ]  Full exhaustive checklist of all files/modules to modify with specific requirements for each
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test-bdd` — all BDD features pass
</acceptance_criteria>
