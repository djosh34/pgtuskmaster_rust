---
## Task: Add network partition e2e chaos tests with proxy fault injection <status>not_started</status> <passes>false</passes>

<blocked_by>24-task-real-e2e-harness-3nodes-3etcd</blocked_by>

<description>
**Goal:** Validate split-brain safety and recovery under true network partition conditions using a controllable proxy layer for etcd, postgres, and API traffic.

**Scope:**
- Introduce a network-fault harness (toxiproxy integration or in-repo proxy utility) to inject partitions, latency, and disconnects.
- Route etcd, postgres, and API connections through the proxy in dedicated e2e tests.
- Add partition matrix scenarios (minority isolation, primary isolation, API path isolation, heal/rejoin) with explicit split-brain assertions.
- Verify post-heal convergence and data integrity through API and SQL checks.

**Context from research:**
- Current e2e tests do not model real network partitions.
- Requirement explicitly asks for separate e2e partition tests and no split-brain guarantees.
- Existing test harness can host additional helper processes and deterministic lifecycle control.

**Expected outcome:**
- Dedicated partition chaos e2e suites prove no dual-primary behavior and safe recovery for partition/heal cycles.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Full exhaustive checklist completed with concrete module requirements: new proxy harness module(s) (for example `src/test_harness/net_proxy.rs`), e2e partition scenario files, fixture wiring for etcd/postgres/api via proxy endpoints, assertions covering split-brain prevention and post-heal convergence/data checks
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test-bdd` — all BDD features pass
</acceptance_criteria>
