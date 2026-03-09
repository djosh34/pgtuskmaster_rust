## Task: Add Old Primary Returns As Replica Only After Majority Failover E2E Coverage <status>not_started</status> <passes>false</passes>

<priority>high</priority>

<description>
**Goal:** Add a realistic scenario where the old primary is isolated or taken down as a whole node, the majority elects a new primary, and the old primary later returns but is only allowed to rejoin as a replica. The higher-order goal is to prove that a recovered former leader never reclaims leadership automatically after the cluster has already moved on.

**Scope:**
- Extend HA E2E coverage in:
- `tests/ha/support/multi_node.rs`
- `tests/ha/support/partition.rs`
- `tests/ha_multi_node_failover.rs`
- or `tests/ha_partition_isolation.rs`, whichever gives the cleanest deterministic setup
- Use a majority-failover trigger that forces leadership away from the old primary before it returns.
- If the trigger is a whole-node outage rather than a network partition, reuse the explicit clean-stop or hard-kill semantics from task 02 rather than inventing a softer node-stop meaning here.
- Then bring the old primary back and require:
- it does not become primary on return,
- it rejoins as replica only,
- it catches up from the new primary,
- it does not cause a second unsafe leadership change.

**Context from research:**
- This is one of the most operator-realistic post-failover questions.
- The current suite covers some rejoin paths, but not this exact “majority already moved on, old primary comes back later and must stay follower” story as a dedicated acceptance target.

**Expected outcome:**
- The suite proves that returning former leaders cannot steal authority back after majority failover.
- Rejoin behavior is explicit and regression-resistant.

</description>

<acceptance_criteria>
- [ ] Add at least one scenario where a majority-side failover completes before the old primary returns.
- [ ] Before the old primary returns, the scenario must prove there is exactly one stable new primary and successful proof writes on it.
- [ ] After the old primary returns, the scenario must prove it rejoins only as replica and never appears as primary during the bounded observation window.
- [ ] The scenario must verify catch-up of proof rows from the new primary to the returning old primary.
- [ ] The scenario must assert no dual-primary window across the entire failover-and-return interval.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
