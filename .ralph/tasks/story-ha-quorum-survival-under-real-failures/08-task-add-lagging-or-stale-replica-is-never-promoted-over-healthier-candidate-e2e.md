## Task: Add Lagging Or Stale Replica Is Never Promoted Over Healthier Candidate E2E Coverage <status>not_started</status> <passes>false</passes>

<priority>high</priority>

<description>
**Goal:** Add realistic HA coverage for replica-quality-based promotion safety: when one candidate is stale, lagging, or otherwise behind, the healthier candidate must be preferred. The higher-order goal is to stop treating all surviving replicas as equally good promotion targets when operator reality says they are not.

**Execution contract for this task:** The HA E2E coverage added here must remain safe under parallel execution. If the scenario needs shared binaries or fault-injection helpers, solve the runner concerns through nextest-friendly prebuild/reuse and namespace isolation rather than serializing the suite.

**Scope:**
- Extend HA E2E coverage in:
- `tests/ha/support/multi_node.rs`
- `tests/ha_partition_isolation.rs` or `tests/ha_multi_node_failover.rs`
- relevant harness helpers for inducing controlled lag or stale upstream state
- Add a scenario where one replica becomes meaningfully worse than another before the primary is lost.
- Then force failover and require:
- the healthier candidate becomes primary,
- the stale/lagging candidate is not promoted,
- post-failover writes succeed and converge.

**Context from research:**
- There is already one degraded-replica failover test, but the acceptance should be raised to a more operator-meaningful “never promote the clearly worse candidate” story.
- This is a common real-world failure mode when one standby is disconnected, lagging, or stale before the main outage.
- The user wants HA E2E to keep running in parallel; this task must not rely on one-at-a-time scheduling.

**Expected outcome:**
- Promotion behavior becomes explicitly quality-aware in the test suite.
- The suite proves that a visibly worse standby is not treated as an equally valid leader candidate.

</description>

<acceptance_criteria>
- [ ] Add one deterministic way to make one replica materially less healthy or more stale than another before primary loss.
- [ ] Add at least one scenario where the primary is then lost and the healthier candidate must become the only new primary before heal.
- [ ] The scenario must explicitly fail if the stale/lagging candidate becomes primary.
- [ ] The scenario must include a bounded pre-heal assertion that the chosen healthier candidate accepts a proof write as primary while the worse candidate remains non-primary.
- [ ] The scenario must assert no dual-primary window and verify final proof-row convergence once all expected nodes are online.
- [ ] The added scenario remains compatible with parallel `nextest`-style execution and does not require suite-wide serialization or repeated in-test runtime-binary builds.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
