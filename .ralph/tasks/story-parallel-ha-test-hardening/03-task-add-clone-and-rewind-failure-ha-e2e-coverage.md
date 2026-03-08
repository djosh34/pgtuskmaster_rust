## Task: Add Clone And Rewind Failure HA E2E Coverage <status>not_started</status> <passes>false</passes>

<priority>low</priority>

<description>
**Goal:** Add explicit HA behavioural coverage for failure paths around `pg_basebackup` clone/startup and `pg_rewind` rejoin. The higher-order goal is to validate not only the happy-path recovery machinery, but also the cases where the cluster cannot successfully clone or rewind a node and must fail safely, surface the problem, and recover cleanly once the fault is removed.

This task should treat clone and rewind failures as first-class operational scenarios, not as rare internal implementation details. These are exactly the places where orchestration often looks correct in clean demos and then breaks under real conditions.

**Scope:**
- Identify the current hooks for making clone and rewind fail in the HA harness or process dispatch layer.
- Add focused behavioural scenarios that force:
- initial clone/basebackup failure for a joining or rejoining replica
- rewind failure for a former primary that needs to rejoin after failover
- eventual recovery once the injected failure is removed, if the scenario is meant to test retry/recovery
- Ensure the tests assert safe cluster behaviour while the clone or rewind step is failing:
- no dual-primary evidence
- no false declaration of a healthy replica when clone/rewind did not complete
- existing primary remains authoritative and writable when expected
- recovered node converges correctly after the fault is cleared
- Keep the scope strictly on node-management/HA paths, not CLI.

**Context from research:**
- The current suite proves that a former primary can rejoin after failover in the happy path.
- There is no current behavioural coverage for `pg_rewind` failure or `pg_basebackup` failure.
- These failure modes were explicitly called out during research as meaningful testing gaps.
- The user wants more HA failure tests and wants them specified precisely.

**Expected outcome:**
- The suite stops assuming clone and rewind always succeed.
- HA orchestration is validated under realistic rejoin failure conditions.
- Recovery logic becomes safer because the failure path is now executable and asserted.

</description>

<acceptance_criteria>
- [ ] Identify the exact harness or subprocess hooks needed to inject deterministic `pg_basebackup` and `pg_rewind` failures without relying on flaky timing.
- [ ] Add at least one new scenario for clone/basebackup failure and at least one new scenario for rewind failure.
- [ ] Each scenario explicitly verifies that the cluster remains safe while the operation is failing and does not falsely report successful convergence.
- [ ] Each scenario explicitly verifies no dual-primary evidence during the failure window.
- [ ] Where recovery-after-fix is part of the scenario, the test verifies the node eventually rejoins correctly and data converges afterward.
- [ ] If a failure is meant to remain unrecovered, the scenario verifies that the cluster stabilizes in the expected degraded state instead of oscillating silently.
- [ ] The implementation uses deterministic failure injection rather than probabilistic shell hacks or sleeps that make the test fragile.
- [ ] The scenarios remain parallel-safe through isolated namespaces, ports, data dirs, and artifact names.
- [ ] Add any needed helper abstractions in the harness or process-dispatch test support so these failure modes are easy to reason about and maintain.
- [ ] The implementation does not touch CLI code.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If the new scenarios belong in the long-running gate: `make test-long` — passes cleanly
</acceptance_criteria>
