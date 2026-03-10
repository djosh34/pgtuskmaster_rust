## Task: Add Broken Replica Rejoin Does Not Block Healthy Quorum Availability E2E Coverage <status>not_started</status> <passes>false</passes>

<priority>high</priority>

<description>
**Goal:** Add a realistic recovery scenario where a replica returns in a broken state during or after failover, and the healthy quorum must stay available regardless. The higher-order goal is to make explicit that a bad rejoiner is not allowed to hold the cluster hostage.

**Execution contract for this task:** The HA E2E coverage added here must remain safe under parallel execution. Do not solve harness contention or runtime-binary locking by serializing the suite. The task must preserve or improve nextest-friendly execution with one built runtime binary reused across isolated test namespaces.

**Scope:**
- Extend HA E2E coverage in:
- `tests/ha/support/multi_node.rs`
- `tests/ha_multi_node_failover.rs`
- reuse fault-wrapper support or startup-failure helpers where useful
- Add a scenario where:
- failover has already produced one healthy primary,
- another node attempts to rejoin but PostgreSQL startup or recovery is broken,
- the healthy quorum remains with one stable primary and successful writes,
- the broken rejoiner stays non-primary until fixed.

**Context from research:**
- The suite already has clone/rewind failure tests, but this operator-facing acceptance is broader: a broken returning replica must not reduce availability of the already healthy quorum.
- This is the same principle as your “one good returning node should be enough” requirement, applied to ongoing availability during bad rejoin attempts.
- The user wants HA E2E to keep running in parallel; this task must not degrade into serial-only execution.

**Expected outcome:**
- The suite proves bad rejoiners are isolated from cluster availability.
- Recovery remains progressive instead of all-or-nothing.

</description>

<acceptance_criteria>
- [ ] Add at least one scenario where the cluster already has a healthy primary after failover or recovery, and another node then attempts a broken rejoin.
- [ ] The scenario must prove the healthy primary stays stable and accepts proof writes during the broken rejoin attempt.
- [ ] The broken rejoining node must never appear as primary or as a successful leadership candidate while broken.
- [ ] The scenario must assert no dual-primary window and verify final proof-row convergence if the broken node is later fixed within the same test.
- [ ] The added scenario remains compatible with parallel `nextest`-style execution and does not require suite-wide serialization or repeated in-test runtime-binary builds.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
