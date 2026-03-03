---
## Task: Add unassisted failover e2e with before/after SQL consistency proof <status>not_started</status> <passes>false</passes>

<blocked_by>24-task-real-e2e-harness-3nodes-3etcd</blocked_by>
<blocked_by>25-task-enforce-e2e-api-only-control-no-direct-dcs</blocked_by>

<description>
**Goal:** Create a skeptical e2e test proving full failover completes after killing one postgres instance, with no further interventions beyond API reads and SQL validation.

**Scope:**
- Add a dedicated e2e scenario that writes SQL before failure, kills the active primary postgres process, then performs no control actions.
- Poll only exposed API status endpoints to detect health/recovery completion.
- After recovery, write new SQL and read back both pre-failure and post-failure data to validate continuity and correctness.
- Assert demotion/promotion/fencing outcomes from observable API state transitions (not direct binary hooks).

**Context from research:**
- Existing e2e matrix currently steers failover by direct DCS writes, which is no longer acceptable for strict proof.
- Requested acceptance bar is explicit: kill one pg instance, do nothing else, wait for API to report healthy, then validate SQL read/write behavior.
- Existing postgres harness helpers and process controls can be reused for the single injected failure action.

**Expected outcome:**
- A high-confidence failover proof exists showing autonomous HA recovery and preserved write/read correctness across failure.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Full exhaustive checklist completed with concrete module requirements: e2e scenario file(s) (`src/ha/e2e_multi_node.rs` or new `tests/e2e_*`), SQL helper/wait utilities in harness modules (`src/test_harness/pg16.rs` or new helper), API polling assertions for healthy convergence, explicit data before/after write-read checks persisted through failover
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test-bdd` — all BDD features pass
</acceptance_criteria>
